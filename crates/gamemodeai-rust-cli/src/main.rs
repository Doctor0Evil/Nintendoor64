use std::io::{self, Read};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use contracts::{
    BuildLogEvent, CargoOpKind, CrateSelection, Diagnostic, DiagnosticLevel, DiagnosticSpan,
    RunCargoParams, RunCargoResult,
};
use serde::{Deserialize, Serialize};

mod contracts;

/// JSON-RPC-like request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum RequestEnvelope {
    /// Single-operation call: params in, result out.
    RunCargo { id: String, params: RunCargoParams },
}

/// JSON-RPC-like response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum ResponseEnvelope {
    RunCargoResult { id: String, result: RunCargoResult },
    Error { id: Option<String>, message: String },
}

fn main() -> Result<()> {
    // Read entire stdin as a single JSON request.
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .context("reading JSON request from stdin")?;

    let req: RequestEnvelope =
        serde_json::from_str(&buf).context("parsing JSON request into RequestEnvelope")?;

    let resp = match req {
        RequestEnvelope::RunCargo { id, params } => match handle_run_cargo(&params) {
            Ok(result) => ResponseEnvelope::RunCargoResult { id, result },
            Err(e) => ResponseEnvelope::Error {
                id: Some(id),
                message: format!("{e:#}"),
            },
        },
    };

    let out = serde_json::to_string_pretty(&resp)?;
    println!("{out}");
    Ok(())
}

fn handle_run_cargo(params: &RunCargoParams) -> Result<RunCargoResult> {
    let mut cmd = Command::new("cargo");

    match params.op {
        CargoOpKind::Check => {
            cmd.arg("check");
        }
        CargoOpKind::Build => {
            cmd.arg("build");
        }
    }

    match &params.selection {
        CrateSelection::Workspace => {
            cmd.arg("--workspace");
        }
        CrateSelection::Packages(pkgs) => {
            for p in pkgs {
                cmd.arg("-p").arg(p);
            }
        }
        CrateSelection::ManifestPath(path) => {
            cmd.arg("--manifest-path").arg(path);
        }
    }

    if params.all_features {
        cmd.arg("--all-features");
    } else if !params.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(params.features.join(","));
    }

    if let Some(target) = &params.target {
        cmd.arg("--target").arg(target);
    }

    if params.profile == "release" {
        cmd.arg("--release");
    }

    for flag in &params.extra_flags {
        cmd.arg(flag);
    }

    // Enforce JSON message format so we can parse diagnostics deterministically.
    cmd.arg("--message-format=json");

    if let Some(target_dir) = &params.target_dir {
        cmd.env("CARGO_TARGET_DIR", target_dir);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().context("spawning cargo process")?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture cargo stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("failed to capture cargo stderr"))?;

    // Read stdout completely; cargo emits JSON lines here.
    let stdout_str = {
        let mut s = String::new();
        let mut reader = io::BufReader::new(stdout);
        use std::io::Read;
        reader.read_to_string(&mut s)?;
        s
    };

    // We treat stderr as a single log blob; it's usually noise / progress.
    let stderr_str = {
        let mut s = String::new();
        let mut reader = io::BufReader::new(stderr);
        use std::io::Read;
        reader.read_to_string(&mut s)?;
        s
    };

    let status = child.wait()?;
    let exit_code = status.code().unwrap_or(-1);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut log_events: Vec<BuildLogEvent> = Vec::new();

    // Parse JSON messages from stdout, ignoring unknown variants.
    for line in stdout_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<CargoMessage>(line) {
            Ok(msg) => match msg {
                CargoMessage::CompilerMessage(m) => {
                    if let Some(diag) = convert_compiler_message(&m) {
                        diagnostics.push(diag);
                    }
                }
                CargoMessage::BuildScriptMessage(m) => {
                    log_events.push(BuildLogEvent {
                        kind: "BuildScript".to_string(),
                        message: m.message.message,
                        timestamp_utc: Some(Utc::now().to_rfc3339()),
                    });
                }
                CargoMessage::TextMessage(t) => {
                    log_events.push(BuildLogEvent {
                        kind: "Info".to_string(),
                        message: t.message,
                        timestamp_utc: Some(Utc::now().to_rfc3339()),
                    });
                }
                CargoMessage::BuildFinished(_) => {
                    // Could add a summary event later.
                }
                CargoMessage::Other => {
                    // Ignored for now; you can add richer handling later.
                }
            },
            Err(_) => {
                // If the line is not valid JSON, treat it as a plain log line.
                log_events.push(BuildLogEvent {
                    kind: "Info".to_string(),
                    message: line.to_string(),
                    timestamp_utc: Some(Utc::now().to_rfc3339()),
                });
            }
        }
    }

    if !stderr_str.trim().is_empty() {
        log_events.push(BuildLogEvent {
            kind: "Stderr".to_string(),
            message: stderr_str,
            timestamp_utc: Some(Utc::now().to_rfc3339()),
        });
    }

    let status_str = if exit_code == 0 { "ok" } else { "error" }.to_string();

    Ok(RunCargoResult {
        status: status_str,
        exit_code,
        diagnostics,
        log_events,
    })
}

// ----- Minimal re-encoding of cargo's JSON messages -----

#[derive(Debug, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
enum CargoMessage {
    #[serde(rename = "compiler-message")]
    CompilerMessage(CompilerMessage),
    #[serde(rename = "build-script-executed")]
    BuildScriptMessage(BuildScriptMessage),
    #[serde(rename = "build-finished")]
    BuildFinished(BuildFinished),
    #[serde(rename = "text")]
    TextMessage(TextMessage),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct CompilerMessage {
    message: RustcMessage,
}

#[derive(Debug, Deserialize)]
struct BuildScriptMessage {
    message: RustcMessage,
}

#[derive(Debug, Deserialize)]
struct BuildFinished {
    success: bool,
}

#[derive(Debug, Deserialize)]
struct TextMessage {
    message: String,
}

#[derive(Debug, Deserialize)]
struct RustcMessage {
    level: String,
    code: Option<RustcCode>,
    message: String,
    spans: Vec<RustcSpan>,
    rendered: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RustcCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct RustcSpan {
    file_name: String,
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
}

fn convert_level(level: &str) -> DiagnosticLevel {
    match level {
        "error" => DiagnosticLevel::Error,
        "warning" => DiagnosticLevel::Warning,
        "note" => DiagnosticLevel::Note,
        "help" => DiagnosticLevel::Help,
        _ => DiagnosticLevel::Unknown,
    }
}

fn convert_compiler_message(msg: &CompilerMessage) -> Option<Diagnostic> {
    let rm = &msg.message;
    let level = convert_level(&rm.level);
    let code = rm.code.as_ref().map(|c| c.code.clone());
    let spans = rm
        .spans
        .iter()
        .map(|s| DiagnosticSpan {
            file: Some(s.file_name.clone()),
            line_start: Some(s.line_start),
            line_end: Some(s.line_end),
            column_start: Some(s.column_start),
            column_end: Some(s.column_end),
        })
        .collect();

    Some(Diagnostic {
        level,
        code,
        message: rm.message.clone(),
        spans,
        rendered: rm.rendered.clone(),
    })
}
