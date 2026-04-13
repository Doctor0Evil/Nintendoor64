// crates/sonia-build-agent/src/lib.rs
//! Terminal-less build daemon that accepts structured JSON-RPC intents,
//! routes them to deterministic executors, and emits machine-readable events.
//!
//! AI-Chat operates only through declarative intents (e.g. `cargo_check`,
//! `cargo_build`, `generate_schemas`), never through raw shell commands.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub mod executors;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentPriority {
    Background = 0,
    Normal = 1,
    Blocking = 2,
}

/// Structured intent submitted by AI-Chat or CI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIntent {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub priority: IntentPriority,
    pub session_id: Option<String>,
}

/// Machine-readable events for AI-Chat.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum BuildEvent {
    CompilationStarted { intent_id: String },
    Diagnostic {
        intent_id: String,
        file: String,
        line: u32,
        column: u32,
        level: String,
        message: String,
        suggestion: Option<String>,
    },
    ArtifactEmitted {
        intent_id: String,
        path: String,
        artifact_type: String,
    },
    Success {
        intent_id: String,
        artifacts: Vec<String>,
        duration_ms: u64,
    },
    Failure {
        intent_id: String,
        reason: String,
        diagnostics: Vec<serde_json::Value>,
    },
    SessionInvariantBlocked {
        intent_id: String,
        violated_rule: String,
        session_state: serde_json::Value,
    },
}

/// Core daemon: queues intents, validates via SessionProfile, dispatches to executors.
/// Event streaming is intentionally left to the caller (e.g. a JSON-RPC layer).
pub struct BuildDaemon {
    tx: mpsc::UnboundedSender<BuildIntent>,
    handle: tokio::task::JoinHandle<()>,
}

impl BuildDaemon {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            while let Some(intent) = rx.recv().await {
                Self::handle_intent(intent).await;
            }
        });
        Self { tx, handle }
    }

    pub fn enqueue(&self, intent: BuildIntent) -> Result<(), String> {
        self.tx.send(intent).map_err(|e| e.to_string())
    }

    async fn handle_intent(intent: BuildIntent) {
        let intent_id = intent.id.clone();
        info!(%intent_id, method = %intent.method, "processing build intent");

        if let Err(violation) = Self::check_session_invariants(&intent).await {
            let evt = BuildEvent::SessionInvariantBlocked {
                intent_id,
                violated_rule: violation,
                session_state: serde_json::json!({
                    "mode": "fix_only",
                    "ci_failing": true
                }),
            };
            warn!(?evt, "intent blocked by session invariants");
            // TODO: forward evt to event sink
            return;
        }

        match intent.method.as_str() {
            "generate_schemas" => executors::cargo::run_generate_schemas(&intent).await,
            "cargo_check" | "cargo_build" => executors::cargo::run_cargo_pipeline(&intent).await,
            _ => {
                let evt = BuildEvent::Failure {
                    intent_id,
                    reason: format!("unknown method: {}", intent.method),
                    diagnostics: vec![],
                };
                warn!(?evt, "routing failed");
                // TODO: forward evt to event sink
            }
        }
    }

    async fn check_session_invariants(intent: &BuildIntent) -> Result<(), String> {
        // Hook into gamemodeai-session here:
        // - load SessionProfile for intent.session_id
        // - consult invariants + ciStatus + compute_mode
        // - return Err(reason) if this method/priority is forbidden
        if intent.method == "cargo_build" && intent.priority == IntentPriority::Blocking {
            // Example: require explicit opt-in from session invariants.
            return Err("blocking cargo_build not permitted in current compute_mode".to_string());
        }
        Ok(())
    }
}

impl Drop for BuildDaemon {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
