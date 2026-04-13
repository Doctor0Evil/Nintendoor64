use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildIntent {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub priority: u8, // 0=background, 1=normal, 2=blocking
}

#[derive(Debug, Serialize)]
pub enum BuildEvent {
    CompilationStarted { intent_id: String },
    Diagnostic { file: String, line: u32, level: String, message: String },
    Success { intent_id: String, artifacts: Vec<String> },
    Failure { intent_id: String, reason: String, diagnostics: Vec<serde_json::Value> },
}

pub struct BuildDaemon {
    queue: mpsc::UnboundedSender<BuildIntent>,
    executor: tokio::task::JoinHandle<()>,
}

impl BuildDaemon {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            while let Some(intent) = rx.recv().await {
                Self::execute_intent(intent).await;
            }
        });
        Self { queue: tx, executor: handle }
    }

    pub fn enqueue(&self, intent: BuildIntent) -> Result<(), String> {
        self.queue.send(intent).map_err(|e| e.to_string())
    }

    async fn execute_intent(intent: BuildIntent) {
        // 1. Validate against SessionProfile invariants
        // 2. Spawn isolated cargo process
        // 3. Stream structured events back to AI session
        // 4. Update artifact registry & CI digest
    }
}
