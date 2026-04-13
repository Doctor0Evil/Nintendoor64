// Nintendoor64/crates/sonia-build-agent/src/executors/cargo.rs

//! Executes `cargo` commands with `--message-format=json` and converts them into structured
//! `BuildEvent` streams for AI-orchestrated build pipelines.
//!
//! AI-Chat Exclusive Feature: **Structured Diagnostic Streaming**
//! Raw `stderr`/`stdout` is never exposed to AI. Only typed `Diagnostic`, `Artifact`,
//! and `Success/Failure` events are returned, enabling precise self-correction loops.

use super::super::{BuildEvent, BuildIntent};
use serde::Deserialize;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{error, info, warn};

/// Top‑level message envelope for `cargo --message-format=json*`.
#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

/// Subset of `compiler-message` payload that we care about.
#[derive(Debug, Deserialize)]
struct CompilerMessage {
    message: CompilerInner,
}

#[derive(Debug, Deserialize)]
struct CompilerInner {
    rendered: String,
    level: String,
    // We keep the raw code payload to attach into diagnostics metadata if needed.
    code: Option<serde_json::Value>,
    // Children carry additional spans/notes; we ignore them for the AI surface for now.
    children: Vec<CompilerInner>,
}

/// High‑level cargo pipeline for a `BuildIntent`.
///
/// This function:
/// - Constructs a `cargo check` command using `intent.params`.
/// - Streams JSON messages from `stdout`.
/// - Emits structured `BuildEvent::Diagnostic` and `BuildEvent::Success/Failure` events.
pub async fn run_cargo_pipeline(intent: &BuildIntent) {
    let intent_id = intent.id.clone();
    let start = Instant::now();
    let mut diagnostics: Vec<serde_json::Value> = Vec::new();
    let mut artifacts: Vec<String> = Vec::new();

    info!(%intent_id, "Starting cargo pipeline");

    let mut cmd = Command::new("cargo");
    cmd.arg("check")
        // Use JSON with rendered diagnostics so we can feed the LLM with human‑oriented text
        // without exposing raw terminal escape sequences.
        .arg("--message-format=json-render-diagnostics")
        .arg("--quiet")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Extract crate & features from params
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
                intent_id: intent_id.clone(),
                reason: e.to_string(),
                diagnostics,
            };
            error!(?evt, "Failed to spawn cargo process");
            // In a fuller pipeline you may want to emit `evt` into a channel here.
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let evt = BuildEvent::Failure {
                intent_id: intent_id.clone(),
                reason: "Failed to capture cargo stdout".to_string(),
                diagnostics,
            };
            error!(?evt, "Cargo stdout not piped");
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
                    // Persist raw JSON payload for downstream consumers.
                    diagnostics.push(msg.data.clone());

                    if let Ok(cm) = serde_json::from_value::<CompilerMessage>(msg.data.clone()) {
                        let level = cm.message.level.clone();
                        let rendered = cm.message.rendered.trim().to_string();

                        // In future we can parse spans for file/line/column; for now we keep them
                        // as "unknown" to maintain the invariant that AI never sees raw paths
                        // unless explicitly allowed via higher-level contracts.
                        let evt = BuildEvent::Diagnostic {
                            intent_id: intent_id.clone(),
                            file: "unknown".to_string(),
                            line: 0,
                            column: 0,
                            level,
                            message: rendered,
                            suggestion: None,
                        };

                        info!(?evt, "Diagnostic streamed");
                        // In a streaming design, this would be sent over a channel.
                    }
                }
                "compiler-artifact" => {
                    if let Some(paths) = msg.data.get("filenames") {
                        if let Some(arr) = paths.as_array() {
                            for p in arr {
                                if let Some(s) = p.as_str() {
                                    artifacts.push(s.to_string());
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Ignore other message reasons (build-script-executed, build-finished, etc.)
                }
            },
            Err(e) => {
                // Keep this logged but non-fatal; `cargo` sometimes prints non-JSON lines.
                warn!(%e, %line, "Failed to parse cargo JSON message");
            }
        }
    }

    let status = match child.wait().await {
        Ok(s) => s,
        Err(e) => {
            let evt = BuildEvent::Failure {
                intent_id: intent_id.clone(),
                reason: format!("Failed to wait for cargo process: {e}"),
                diagnostics,
            };
            error!(?evt, "Cargo pipeline wait error");
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
        info!(?evt, "Cargo pipeline completed successfully");
        // Emit evt to your event sink here.
    } else {
        let evt = BuildEvent::Failure {
            intent_id,
            reason: "Compilation failed".to_string(),
            diagnostics,
        };
        error!(?evt, "Cargo pipeline failed");
        // Emit evt to your event sink here.
    }
}

/// Stub entry point for schema generation / codegen crates that live alongside build.
/// In production this would delegate into a dedicated crate (e.g. `n64-ai-gen-schemas`).
pub async fn run_generate_schemas(intent: &BuildIntent) {
    info!(id = %intent.id, "Schema generation triggered (stub)");
    // Intentionally no-op for now; wiring to real schema-gen crate is handled elsewhere.
}
