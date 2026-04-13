use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Top-level CLI for AI-facing build orchestration in Nintendoor64 / GAMEMODE.ai.
///
/// This binary is intentionally JSON-in/JSON-out only. It never talks to Git,
/// never touches secrets, and only shells out to Cargo and other CLIs (like
/// retro tools) via schema-backed JSON contracts.
#[derive(Debug, Parser)]
#[command(name = "gamemodeai-toolchain")]
#[command(version)]
#[command(about = "AI-safe toolchain orchestrator for Nintendoor64 / GAMEMODE.ai")]
struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// Run a single Rust build/check/test job via gamemodeai-rust-cli and emit structured telemetry.
    RunRustJob {
        /// Path to a JSON file containing a RunRustJobRequest.
        #[arg(long)]
        job: PathBuf,
        /// Repository root (workspace root). Defaults to current directory.
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Profile a build.rs or tool binary and emit a flamegraph-compatible JSON summary.
    ///
    /// This does not generate SVG directly; instead it writes a trace file suitable
    /// for flamegraph or similar tools to consume in CI.
    FlamegraphBuild {
        /// Path to a JSON file containing a FlamegraphBuildRequest.
        #[arg(long)]
        job: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Run a schema generator under Miri without building a full binary, to probe UB.
    ///
    /// This assumes the repo has a cargo+miri setup and that `schema_gen_target`
    /// is a binary or test target that exercises the schema generator.
    MiriSchemaCheck {
        /// Path to a JSON file containing a MiriSchemaCheckRequest.
        #[arg(long)]
        job: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Run a custom sanitizer over a generated N64 binary blob before it is written to disk.
    ///
    /// This is purely a post-processing step over a temporary output path.
    SanitizeN64Binary {
        /// Path to a JSON file containing a SanitizeN64BinaryRequest.
        #[arg(long)]
        job: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },

    /// Emit a refactor suggestion for a build script based on previously recorded telemetry.
    ///
    /// Telemetry is assumed to be JSON written by RunRustJob / FlamegraphBuild into
    /// a toolchain-telemetry directory.
    SuggestBuildRefactor {
        /// Path to a JSON file containing a SuggestBuildRefactorRequest.
        #[arg(long)]
        job: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
}

/// Envelope for all toolchain responses, for easy wiring into Sonia / gamemodeai-session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainEnvelope<T> {
    version: u32,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ToolchainError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolchainError {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

/// Lightweight mirror of the existing RunCargoParams / RunCargoResult, but wrapped with
/// repo+session metadata so a single JSON file can drive the orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunRustJobRequest {
    /// Opaque ID for telemetry correlation.
    job_id: String,
    /// JSON payload to pipe into gamemodeai-rust-cli.
    rust_params: serde_json::Value,
    /// Optional logical session path for SessionProfile integration.
    session_profile_path: Option<String>,
    /// Optional label (e.g., "nintendoor64.n64-layout.build") to tag telemetry.
    label: Option<String>,
    /// If true, skip writing telemetry to disk.
    #[serde(default)]
    skip_telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunRustJobResult {
    job_id: String,
    exit_code: i32,
    status: String,
    diagnostics: Vec<serde_json::Value>,
    log_events: Vec<serde_json::Value>,
    /// Optional location where telemetry was written.
    telemetry_path: Option<String>,
}

/// Request to profile a build.rs or tool binary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlamegraphBuildRequest {
    job_id: String,
    /// Path to the binary to profile (relative to repo root or absolute).
    binary_path: String,
    /// Arguments to pass to the binary.
    #[serde(default)]
    args: Vec<String>,
    /// Output trace path (relative to repo root) for flamegraph.
    trace_output_path: String,
    /// Optional environment variables for the profiled process.
    #[serde(default)]
    env: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlamegraphBuildResult {
    job_id: String,
    trace_output_path: String,
    sample_count: u64,
    /// Implementation-defined metadata (e.g., perf command line).
    metadata: serde_json::Value,
}

/// Request to run Miri on a schema generator target.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MiriSchemaCheckRequest {
    job_id: String,
    /// Cargo target spec, e.g. "test schema_gen" or "run schema-gen".
    cargo_subcommand: String,
    /// Extra args passed to `cargo miri <subcommand>`.
    #[serde(default)]
    args: Vec<String>,
    /// Optional environment overrides.
    #[serde(default)]
    env: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MiriSchemaCheckResult {
    job_id: String,
    exit_code: i32,
    status: String,
    /// Raw stdout/stderr captured from Miri.
    stdout: String,
    stderr: String,
}

/// Request to sanitize a temporary N64 binary before final write.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SanitizeN64BinaryRequest {
    job_id: String,
    /// Absolute or repo-relative path to the input binary to sanitize.
    input_path: String,
    /// Absolute or repo-relative path where the sanitized result should be written.
    output_path: String,
    /// Name of sanitizer script or CLI, e.g. "n64-sanitize-binary".
    sanitizer_command: String,
    /// Arguments for the sanitizer, with "{input}" and "{output}" substitution.
    #[serde(default)]
    sanitizer_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SanitizeN64BinaryResult {
    job_id: String,
    exit_code: i32,
    status: String,
    output_path: String,
}

/// Request to suggest build.rs / toolchain refactors based on telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuggestBuildRefactorRequest {
    job_id: String,
    /// Directory under repo_root where telemetry JSON files live.
    telemetry_dir: String,
    /// Optional filter label to restrict which jobs to consider.
    label_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildRefactorSuggestion {
    /// Human-readable suggestion text (for logs / docs).
    suggestion: String,
    /// Structured hint for AI (e.g., "split_build_rs", "move_asset_pipeline").
    code: String,
    /// Optional hint which file(s) to edit.
    files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuggestBuildRefactorResult {
    job_id: String,
    suggestions: Vec<BuildRefactorSuggestion>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CommandKind::RunRustJob { job, repo_root } => {
            let res = handle_run_rust_job(&job, &repo_root);
            emit(res)
        }
        CommandKind::FlamegraphBuild { job, repo_root } => {
            let res = handle_flamegraph_build(&job, &repo_root);
            emit(res)
        }
        CommandKind::MiriSchemaCheck { job, repo_root } => {
            let res = handle_miri_schema_check(&job, &repo_root);
            emit(res)
        }
        CommandKind::SanitizeN64Binary { job, repo_root } => {
            let res = handle_sanitize_n64_binary(&job, &repo_root);
            emit(res)
        }
        CommandKind::SuggestBuildRefactor { job, repo_root } => {
            let res = handle_suggest_build_refactor(&job, &repo_root);
            emit(res)
        }
    }
}

fn emit<T: Serialize>(res: Result<T>) -> Result<()> {
    match res {
        Ok(data) => {
            let env = ToolchainEnvelope {
                version: 1,
                status: "ok".to_string(),
                data: Some(data),
                error: None,
            };
            let out = serde_json::to_string_pretty(&env)?;
            println!("{out}");
            Ok(())
        }
        Err(e) => {
            let env: ToolchainEnvelope::<serde_json::Value> = ToolchainEnvelope {
                version: 1,
                status: "error".to_string(),
                data: None,
                error: Some(ToolchainError {
                    code: "ToolchainError".to_string(),
                    message: e.to_string(),
                    details: None,
                }),
            };
            let out = serde_json::to_string_pretty(&env)?;
            println!("{out}");
            std::process::exit(1);
        }
    }
}

fn handle_run_rust_job(job_path: &Path, repo_root: &Path) -> Result<RunRustJobResult> {
    let text = fs::read_to_string(job_path)
        .with_context(|| format!("failed to read RunRustJobRequest from {}", job_path.display()))?;
    let req: RunRustJobRequest =
        serde_json::from_str(&text).context("parsing RunRustJobRequest JSON")?;

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "gamemodeai-rust-cli"])
        .current_dir(repo_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("spawning gamemodeai-rust-cli")?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("failed to open stdin for gamemodeai-rust-cli"))?;
        let envelope = serde_json::json!({
            "kind": "RunCargo",
            "id": req.job_id,
            "params": req.rust_params,
        });
        let payload = serde_json::to_string(&envelope)?;
        stdin.write_all(payload.as_bytes())?;
    }

    let output = child
        .wait_with_output()
        .context("waiting for gamemodeai-rust-cli")?;

    if !output.status.success() {
        return Err(anyhow!(
            "gamemodeai-rust-cli failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout_str = String::from_utf8(output.stdout)?;
    let value: serde_json::Value = serde_json::from_str(&stdout_str)
        .context("parsing gamemodeai-rust-cli response JSON")?;

    let data = value
        .get("data")
        .ok_or_else(|| anyhow!("missing data in gamemodeai-rust-cli envelope"))?
        .clone();

    let exit_code = data
        .get("exitcode")
        .or_else(|| data.get("exitCode"))
        .and_then(|v| v.as_i64())
        .unwrap_or(-1) as i32;

    let status = data
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let diagnostics = data
        .get("diagnostics")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let log_events = data
        .get("logevents")
        .or_else(|| data.get("logEvents"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut telemetry_path = None;
    if !req.skip_telemetry {
        let dir = repo_root.join(".gamemodeai").join("toolchain-telemetry");
        fs::create_dir_all(&dir)?;
        let filename = format!(
            "rust-job-{}-{}.json",
            req.job_id,
            Utc::now().format("%Y%m%dT%H%M%SZ")
        );
        let path = dir.join(filename);
        let telemetry = serde_json::json!({
            "jobId": req.job_id,
            "label": req.label,
            "timestampUtc": Utc::now().to_rfc3339(),
            "exitCode": exit_code,
            "status": status,
            "diagnostics": diagnostics,
            "logEvents": log_events,
        });
        fs::write(&path, serde_json::to_vec_pretty(&telemetry)?)?;
        telemetry_path = Some(path.strip_prefix(repo_root).unwrap_or(&path).to_string_lossy().to_string());
    }

    Ok(RunRustJobResult {
        job_id: req.job_id,
        exit_code,
        status,
        diagnostics,
        log_events,
        telemetry_path,
    })
}

fn handle_flamegraph_build(job_path: &Path, repo_root: &Path) -> Result<FlamegraphBuildResult> {
    let text = fs::read_to_string(job_path)
        .with_context(|| format!("failed to read FlamegraphBuildRequest from {}", job_path.display()))?;
    let req: FlamegraphBuildRequest =
        serde_json::from_str(&text).context("parsing FlamegraphBuildRequest JSON")?;

    let bin = resolve_path(repo_root, &req.binary_path);
    let trace_path = resolve_path(repo_root, &req.trace_output_path);

    let mut cmd = Command::new("perf");
    cmd.arg("record")
        .arg("-F")
        .arg("99")
        .arg("-g")
        .arg("--output")
        .arg(&trace_path)
        .arg(&bin);

    for (k, v) in &req.env {
        cmd.env(k, v);
    }

    cmd.args(&req.args);

    let status = cmd
        .status()
        .with_context(|| format!("failed to run perf on {}", bin.display()))?;

    if !status.success() {
        return Err(anyhow!(
            "perf record failed with status {}",
            status.code().unwrap_or(-1)
        ));
    }

    let meta = serde_json::json!({
        "tool": "perf",
        "frequency": 99,
        "binary": bin.to_string_lossy(),
    });

    Ok(FlamegraphBuildResult {
        job_id: req.job_id,
        trace_output_path: trace_path
            .strip_prefix(repo_root)
            .unwrap_or(&trace_path)
            .to_string_lossy()
            .to_string(),
        sample_count: 0,
        metadata: meta,
    })
}

fn handle_miri_schema_check(job_path: &Path, repo_root: &Path) -> Result<MiriSchemaCheckResult> {
    let text = fs::read_to_string(job_path)
        .with_context(|| format!("failed to read MiriSchemaCheckRequest from {}", job_path.display()))?;
    let req: MiriSchemaCheckRequest =
        serde_json::from_str(&text).context("parsing MiriSchemaCheckRequest JSON")?;

    let mut cmd = Command::new("cargo");
    cmd.arg("miri")
        .arg(&req.cargo_subcommand)
        .current_dir(repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (k, v) in &req.env {
        cmd.env(k, v);
    }

    let output = cmd
        .output()
        .context("running cargo miri schema-check command")?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let exit_code = output.status.code().unwrap_or(-1);
    let status = if output.status.success() {
        "ok".to_string()
    } else {
        "error".to_string()
    };

    Ok(MiriSchemaCheckResult {
        job_id: req.job_id,
        exit_code,
        status,
        stdout,
        stderr,
    })
}

fn handle_sanitize_n64_binary(
    job_path: &Path,
    repo_root: &Path,
) -> Result<SanitizeN64BinaryResult> {
    let text = fs::read_to_string(job_path).with_context(|| {
        format!(
            "failed to read SanitizeN64BinaryRequest from {}",
            job_path.display()
        )
    })?;
    let req: SanitizeN64BinaryRequest =
        serde_json::from_str(&text).context("parsing SanitizeN64BinaryRequest JSON")?;

    let input = resolve_path(repo_root, &req.input_path);
    let output = resolve_path(repo_root, &req.output_path);

    let mut args = Vec::with_capacity(req.sanitizer_args.len());
    for a in &req.sanitizer_args {
        let mut replaced = a.replace("{input}", &input.to_string_lossy());
        replaced = replaced.replace("{output}", &output.to_string_lossy());
        args.push(replaced);
    }

    let mut cmd = Command::new(&req.sanitizer_command);
    cmd.args(&args)
        .current_dir(repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output_proc = cmd
        .output()
        .with_context(|| format!("running sanitizer {:?}", req.sanitizer_command))?;

    let exit_code = output_proc.status.code().unwrap_or(-1);
    let status = if output_proc.status.success() {
        "ok".to_string()
    } else {
        "error".to_string()
    };

    Ok(SanitizeN64BinaryResult {
        job_id: req.job_id,
        exit_code,
        status,
        output_path: output
            .strip_prefix(repo_root)
            .unwrap_or(&output)
            .to_string_lossy()
            .to_string(),
    })
}

fn handle_suggest_build_refactor(
    job_path: &Path,
    repo_root: &Path,
) -> Result<SuggestBuildRefactorResult> {
    let text = fs::read_to_string(job_path).with_context(|| {
        format!(
            "failed to read SuggestBuildRefactorRequest from {}",
            job_path.display()
        )
    })?;
    let req: SuggestBuildRefactorRequest =
        serde_json::from_str(&text).context("parsing SuggestBuildRefactorRequest JSON")?;

    let telemetry_root = resolve_path(repo_root, &req.telemetry_dir);
    let mut suggestions = Vec::new();

    if telemetry_root.is_dir() {
        let entries = fs::read_dir(&telemetry_root)?;
        let mut long_jobs: Vec<(String, u64)> = Vec::new();
        let mut heavy_build_rs: Vec<String> = Vec::new();

        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&path)?;
            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(label) = &req.label_filter {
                let lbl = v
                    .get("label")
                    .and_then(|x| x.as_str())
                    .unwrap_or_default()
                    .to_string();
                if lbl != *label {
                    continue;
                }
            }

            if let Some(ms) = v
                .get("durationMs")
                .and_then(|x| x.as_u64())
            {
                if ms > 2500 {
                    let file = path
                        .strip_prefix(repo_root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    long_jobs.push((file, ms));
                }
            }

            if let Some(events) = v.get("logEvents").and_then(|x| x.as_array()) {
                for e in events {
                    if let Some(msg) = e.get("message").and_then(|x| x.as_str()) {
                        if msg.contains("build.rs") && msg.contains("slow path") {
                            let file = path
                                .strip_prefix(repo_root)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();
                            heavy_build_rs.push(file);
                        }
                    }
                }
            }
        }

        if !long_jobs.is_empty() {
            let mut top = long_jobs.clone();
            top.sort_by_key(|(_, ms)| *ms);
            top.reverse();
            let top_files: Vec<String> = top
                .iter()
                .take(5)
                .map(|(f, ms)| format!("{f} (~{ms}ms)"))
                .collect();

            suggestions.push(BuildRefactorSuggestion {
                suggestion: format!(
                    "Split hot build.rs steps into separate tools for these slow jobs: {}",
                    top_files.join(", ")
                ),
                code: "split_build_rs".to_string(),
                files: heavy_build_rs.clone(),
            });
        }

        if !heavy_build_rs.is_empty() {
            suggestions.push(BuildRefactorSuggestion {
                suggestion: format!(
                    "Move asset conversion and N64 layout work out of build.rs into dedicated CLIs; flagged telemetry files: {}",
                    heavy_build_rs.join(", ")
                ),
                code: "move_asset_pipeline_from_build_rs".to_string(),
                files: heavy_build_rs,
            });
        }
    }

    Ok(SuggestBuildRefactorResult {
        job_id: req.job_id,
        suggestions,
    })
}

fn resolve_path(root: &Path, p: &str) -> PathBuf {
    let path = PathBuf::from(p);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}
