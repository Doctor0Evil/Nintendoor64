// crates/sonia-build-agent/src/executors/cargo.rs
//! Structured wrapper around `cargo --message-format=json`.
//! Emits typed diagnostics and artifacts instead of raw stdout/stderr.

use crate::{BuildEvent, BuildIntent};
use serde::Deserialize;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info, warn};

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct CompilerMessage {
    message: CompilerInner,
}

#[derive(Debug, Deserialize)]
struct CompilerInner {
    rendered: String,
    level: String,
    code: Option<serde_json::Value>,
    children: Vec<CompilerInner>,
}

pub async fn run_cargo_pipeline(intent: &BuildIntent) {
    let intent_id = intent.id.clone();
    let start = Instant::now();
    let mut diagnostics = Vec::new();
    let mut artifacts = Vec::new();

    info!(%intent_id, "starting cargo pipeline");

    let mut cmd = Command::new("cargo");
    let profile = intent
        .params
        .get("profile")
        .and_then(|v| v.as_str())
        .unwrap_or("check");

    match profile {
        "release" => {
            cmd.arg("build");
            cmd.arg("--release");
        }
        "dev" => {
            cmd.arg("build");
        }
        "check" | _ => {
            cmd.arg("check");
        }
    }

    cmd.arg("--message-format=json-rendered-diagnostics")
        .arg("--quiet")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(crate_name) = intent.params.get("crate_name").and_then(|v| v.as_str()) {
        cmd.arg("-p").arg(crate_name);
    }

    if let Some(features) = intent.params.get("features").and_then(|v| v.as_array()) {
        for f in features {
            if let Some(s) = f.as_str() {
                cmd.arg("--features").arg(s);
            }
        }
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let evt = BuildEvent::Failure {
                intent_id,
                reason: format!("failed to spawn cargo: {}", e),
                diagnostics,
            };
            error!(?evt, "cargo spawn failed");
            // TODO: forward evt
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let evt = BuildEvent::Failure {
                intent_id,
                reason: "no stdout from cargo".to_string(),
                diagnostics,
            };
            error!(?evt, "cargo stdout missing");
            // TODO: forward evt
            return;
        }
    };

    let mut reader = BufReader::new(stdout).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<CargoMessage>(&line) {
            Ok(msg) => match msg.reason.as_str() {
                "compiler-message" => {
                    if let Ok(cm) = serde_json::from_value::<CompilerMessage>(msg.data.clone()) {
                        diagnostics.push(msg.data.clone());
                        let evt = BuildEvent::Diagnostic {
                            intent_id: intent_id.clone(),
                            file: "unknown".to_string(), // TODO: parse spans
                            line: 0,
                            column: 0,
                            level: cm.message.level.clone(),
                            message: cm.message.rendered.trim().to_string(),
                            suggestion: None,
                        };
                        info!(?evt, "diagnostic");
                        // TODO: forward evt
                    }
                }
                "compiler-artifact" => {
                    if let Some(filenames) = msg.data.get("filenames") {
                        if let Some(arr) = filenames.as_array() {
                            for p in arr {
                                if let Some(s) = p.as_str() {
                                    artifacts.push(s.to_string());
                                    let evt = BuildEvent::ArtifactEmitted {
                                        intent_id: intent_id.clone(),
                                        path: s.to_string(),
                                        artifact_type: "binary".to_string(),
                                    };
                                    info!(?evt, "artifact emitted");
                                    // TODO: forward evt
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(e) => {
                warn!(error = %e, line = %line, "failed to parse cargo JSON message");
            }
        }
    }

    let status = match child.wait().await {
        Ok(s) => s,
        Err(e) => {
            let evt = BuildEvent::Failure {
                intent_id,
                reason: format!("failed to wait on cargo: {}", e),
                diagnostics,
            };
            error!(?evt, "cargo wait failed");
            // TODO: forward evt
            return;
        }
    };

    let duration = start.elapsed().as_millis() as u64;

    if status.success() {
        let evt = BuildEvent::Success {
            intent_id,
            artifacts,
            duration_ms: duration,
        };
        info!(?evt, "cargo pipeline completed successfully");
        // TODO: forward evt
    } else {
        let evt = BuildEvent::Failure {
            intent_id,
            reason: "compilation failed".to_string(),
            diagnostics,
        };
        error!(?evt, "cargo pipeline failed");
        // TODO: forward evt
    }
}

pub async fn run_generate_schemas(intent: &BuildIntent) {
    let intent_id = intent.id.clone();
    info!(%intent_id, "schema generation triggered (stub)");

    // This should eventually shell out to your existing schema-gen CLI
    // (e.g. tools/schema-gen) under fixed args, or call into a Rust API
    // that generates ArtifactSpec / RomLayout / SessionProfile schemas
    // into the `schemas/` directory.

    let evt = BuildEvent::Success {
        intent_id,
        artifacts: vec!["schemas/".to_string()],
        duration_ms: 0,
    };
    info!(?evt, "generate_schemas stub completed");
    // TODO: forward evt
}
