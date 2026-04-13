//! Terminal-less build daemon that accepts structured JSON-RPC intents,
//! routes them to deterministic executors, and streams machine-readable events.
//!
//! AI-Chat Exclusive Feature: **Intent-Over-Command Routing**
//! AI submits declarative goals (e.g., `generate_schemas`, `cargo_build`),
//! never raw shell commands. The daemon enforces session invariants before execution.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, warn};

pub mod executors;

/// Priority levels for build intents. AI agents use these to schedule background
/// vs blocking operations based on CI state and session conditioning.[file:2]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentPriority {
    Background = 0,
    Normal     = 1,
    Blocking   = 2,
}

/// Structured intent submitted by AI-Chat or CI pipelines.[file:2]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildIntent {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub priority: IntentPriority,
    /// Optional session identifier for invariant checks and routing.[file:1]
    pub session_id: Option<String>,
}

/// Machine-readable events streamed back to AI-Chat during execution.[file:2]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum BuildEvent {
    CompilationStarted {
        intent_id: String,
    },
    Diagnostic {
        intent_id: String,
        file: String,
        line: u32,
        column: u32,
        level: String, // warning, error, note
        message: String,
        suggestion: Option<String>,
    },
    ArtifactEmitted {
        intent_id: String,
        path: String,
        /// Logical artifact type, e.g. "binary", "schema.json", "artifact_spec".[file:2]
        artifact_type: String,
    },
    Success {
        intent_id: String,
        artifacts: Vec<String>,
        /// Wall-clock duration in milliseconds for this intent.[file:2]
        duration_ms: u64,
    },
    Failure {
        intent_id: String,
        reason: String,
        /// Structured diagnostics, typically Sonia-style Diagnostic JSON.[file:1]
        diagnostics: Vec<serde_json::Value>,
    },
    /// Emitted when session invariants block execution for safety.[file:1]
    SessionInvariantBlocked {
        intent_id: String,
        violated_rule: String,
        session_state: serde_json::Value,
    },
}

/// Core build daemon that queues intents, validates against session invariants,
/// and dispatches to deterministic executors.[file:1]
pub struct BuildDaemon {
    tx: mpsc::UnboundedSender<BuildIntent>,
    handle: tokio::task::JoinHandle<()>,
}

impl BuildDaemon {
    /// Spawn a new build daemon with an internal task that processes intents
    /// sequentially in FIFO order.[file:1]
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            while let Some(intent) = rx.recv().await {
                Self::handle_intent(intent).await;
            }
        });

        Self { tx, handle }
    }

    /// Enqueue a new build intent for processing.
    /// Returns an error if the daemon task has been dropped.[file:1]
    pub fn enqueue(&self, intent: BuildIntent) -> Result<(), String> {
        self.tx.send(intent).map_err(|e| e.to_string())
    }

    /// AI-Chat Exclusive: **Session-Aware Conditioning Gate**
    ///
    /// Validates intent against active SessionProfile invariants before execution.[file:1]
    async fn handle_intent(intent: BuildIntent) {
        let intent_id = intent.id.clone();
        info!(%intent_id, method = %intent.method, "processing build intent");

        // 1. Session invariant validation stub.
        // In a full implementation, this would call into a Sonia/session
        // service to load the SessionProfile and evaluate invariants for
        // the given session_id and method.[file:1]
        if let Err(violation) = Self::check_session_invariants(&intent).await {
            let evt = BuildEvent::SessionInvariantBlocked {
                intent_id,
                violated_rule: violation,
                session_state: serde_json::json!({
                    "mode": "fix_only",
                    "ci_failing": true,
                }),
            };
            warn!(?evt, "intent blocked by session conditioning");
            // TODO: push `evt` into an event stream or callback once wired.[file:1]
            return;
        }

        // 2. Route to executor based on method.
        match intent.method.as_str() {
            "generate_schemas" => {
                executors::cargo::run_generate_schemas(&intent).await;
            }
            "cargo_build" | "cargo_check" => {
                executors::cargo::run_cargo_pipeline(&intent).await;
            }
            other => {
                let evt = BuildEvent::Failure {
                    intent_id,
                    reason: format!("unknown method: {other}"),
                    diagnostics: vec![],
                };
                warn!(?evt, "routing failed for build intent");
                // TODO: push `evt` into an event stream or callback once wired.[file:2]
            }
        }
    }

    /// Stubbed invariant check: in production this would consider
    /// SessionProfile.ci_status, invariants, and compute_mode.[file:1]
    async fn check_session_invariants(intent: &BuildIntent) -> Result<(), String> {
        // Example placeholder rule:
        // allow background `cargo_build`, future logic will load SessionProfile
        // and block unsafe operations when CI is red or invariants are violated.[file:1]
        if intent.method == "cargo_build" && intent.priority == IntentPriority::Background {
            return Ok(());
        }

        Ok(())
    }
}

impl Drop for BuildDaemon {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
