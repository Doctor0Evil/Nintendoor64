use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use jsonschema::JSONSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sonia_core::{ArtifactSpec, ArtifactValidationError, SessionProfile};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "sonia-core", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an ArtifactSpec JSON file.
    Validate {
        #[arg(long)]
        spec: PathBuf,
    },
    /// Write an ArtifactSpec payload into the artifacts/ tree.
    Write {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    /// List artifacts under artifacts/, optionally filtered by kind substring.
    List {
        #[arg(long)]
        kind: Option<String>,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    /// Get the current SessionProfile JSON.
    GetSession {
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    /// Overwrite the current SessionProfile with a JSON file.
    UpdateSession {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    /// Describe available commands and their machine-readable capabilities.
    Describe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandDescriptor {
    name: String,
    params_schema: String,
    result_schema: String,
    platforms: Vec<String>,
    artifact_kinds: Vec<String>,
    invariants_touched: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Validate { spec } => cmd_validate(&spec),
        Commands::Write { spec, repo_root } => cmd_write(&spec, &repo_root),
        Commands::List { kind, repo_root } => cmd_list(kind, &repo_root),
        Commands::GetSession { repo_root } => cmd_get_session(&repo_root),
        Commands::UpdateSession { profile, repo_root } => {
            cmd_update_session(&profile, &repo_root)
        }
        Commands::Describe => cmd_describe(),
    }
}

fn load_schema(name: &str) -> Result<JSONSchema> {
    let schema_path = Path::new("schemas").join(name);
    let raw = fs::read_to_string(&schema_path)
        .with_context(|| format!("failed to read schema {}", schema_path.display()))?;
    let v: Value = serde_json::from_str(&raw)?;
    Ok(JSONSchema::compile(&v)?)
}

fn cmd_validate(spec_path: &Path) -> Result<()> {
    let text = fs::read_to_string(spec_path)
        .with_context(|| format!("failed to read spec {}", spec_path.display()))?;
    let v: Value = serde_json::from_str(&text)?;

    let schema = load_schema("artifact-spec.schema.json")?;
    let result = schema.validate(&v);

    if let Err(errors) = result {
        let errs: Vec<_> = errors
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();
        let out = serde_json::json!({
            "ok": false,
            "errors": errs,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        std::process::exit(1);
    }

    let spec: ArtifactSpec = serde_json::from_value(v)?;
    if let Err(e) = spec.validate_semantics() {
        let out = serde_json::json!({
            "ok": false,
            "errors": [format!("{}", e)],
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        std::process::exit(1);
    }

    let out = serde_json::json!({ "ok": true });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn cmd_write(spec_path: &Path, repo_root: &Path) -> Result<()> {
    let text = fs::read_to_string(spec_path)
        .with_context(|| format!("failed to read spec {}", spec_path.display()))?;
    let spec: ArtifactSpec = serde_json::from_str(&text)?;

    spec.validate_semantics().map_err(|e| match e {
        ArtifactValidationError::DecodeError(msg) => anyhow::anyhow!(msg),
        other => anyhow::anyhow!(other.to_string()),
    })?;

    let target = spec.target_path(repo_root)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = spec.decode_content()?;
    fs::write(&target, &bytes)?;

    let out = serde_json::json!({
        "ok": true,
        "path": target.to_string_lossy(),
        "bytes_written": bytes.len(),
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn cmd_list(kind: Option<String>, repo_root: &Path) -> Result<()> {
    let mut artifacts = Vec::new();
    let root = repo_root.join("artifacts");
    if !root.exists() {
        let out = serde_json::json!({ "artifacts": [] });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(&root) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(repo_root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if let Some(ref k) = kind {
            if !rel.contains(k) {
                continue;
            }
        }
        artifacts.push(serde_json::json!({ "filename": rel }));
    }

    let out = serde_json::json!({ "artifacts": artifacts });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn session_path(repo_root: &Path) -> PathBuf {
    let branch = std::env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string());
    repo_root
        .join(".sonia")
        .join("session")
        .join(format!("{branch}.json"))
}

fn cmd_get_session(repo_root: &Path) -> Result<()> {
    let path = session_path(repo_root);
    if !path.exists() {
        let profile = SessionProfile {
            repo: repo_root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            branch: std::env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            active_crate: None,
            feature_flags: Vec::new(),
            invariants: Vec::new(),
            ci_status: sonia_core::CiStatusReport {
                last_run_id: None,
                last_result: Some(sonia_core::CiResult::Unknown),
                failing_crates: Vec::new(),
                failing_systems: Vec::new(),
            },
        };
        println!("{}", serde_json::to_string_pretty(&profile)?);
        return Ok(());
    }

    let text = fs::read_to_string(&path)?;
    let profile: SessionProfile = serde_json::from_str(&text)?;
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

fn cmd_update_session(profile_path: &Path, repo_root: &Path) -> Result<()> {
    let text = fs::read_to_string(profile_path)?;
    let profile: SessionProfile = serde_json::from_str(&text)?;
    let path = session_path(repo_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&profile)?)?;
    let out = serde_json::json!({ "ok": true, "path": path.to_string_lossy() });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn cmd_describe() -> Result<()> {
    // For now, stub platform and artifact kind affinities.
    let descriptors = vec![
        CommandDescriptor {
            name: "validate".to_string(),
            params_schema: "schemas/sonia-protocol-request.schema.json".to_string(),
            result_schema: "schemas/sonia-protocol-response.schema.json".to_string(),
            platforms: vec!["Multi".to_string()],
            artifact_kinds: vec![
                "N64RomPatch".to_string(),
                "Ps1IsoPatch".to_string(),
                "LuaScript".to_string(),
                "InputMapperConfig".to_string(),
                "ScenarioSpec".to_string(),
                "NarrativeGraph".to_string(),
                "Other".to_string(),
            ],
            invariants_touched: vec!["ArtifactSchemaValidation".to_string()],
        },
        CommandDescriptor {
            name: "write".to_string(),
            params_schema: "schemas/sonia-protocol-request.schema.json".to_string(),
            result_schema: "schemas/sonia-protocol-response.schema.json".to_string(),
            platforms: vec!["Multi".to_string()],
            artifact_kinds: vec![
                "N64RomPatch".to_string(),
                "Ps1IsoPatch".to_string(),
                "LuaScript".to_string(),
                "InputMapperConfig".to_string(),
                "ScenarioSpec".to_string(),
                "NarrativeGraph".to_string(),
                "Other".to_string(),
            ],
            invariants_touched: vec![
                "ArtifactSchemaValidation".to_string(),
                "ArtifactFilesystemBoundary".to_string(),
            ],
        },
        CommandDescriptor {
            name: "list".to_string(),
            params_schema: "schemas/sonia-protocol-request.schema.json".to_string(),
            result_schema: "schemas/sonia-protocol-response.schema.json".to_string(),
            platforms: vec!["Multi".to_string()],
            artifact_kinds: Vec::new(),
            invariants_touched: vec![],
        },
        CommandDescriptor {
            name: "get-session".to_string(),
            params_schema: "schemas/sonia-protocol-request.schema.json".to_string(),
            result_schema: "schemas/session-profile.schema.json".to_string(),
            platforms: vec!["Multi".to_string()],
            artifact_kinds: Vec::new(),
            invariants_touched: vec!["SessionProfileRead".to_string()],
        },
        CommandDescriptor {
            name: "update-session".to_string(),
            params_schema: "schemas/session-profile.schema.json".to_string(),
            result_schema: "schemas/sonia-protocol-response.schema.json".to_string(),
            platforms: vec!["Multi".to_string()],
            artifact_kinds: Vec::new(),
            invariants_touched: vec!["SessionProfileInvariants".to_string()],
        },
    ];

    let out = serde_json::json!({ "commands": descriptors });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
