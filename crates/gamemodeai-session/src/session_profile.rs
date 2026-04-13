// Nintendoor64/crates/gamemodeai-session/src/session_profile.rs (Diff/Addition)
//! Adds `build_queue` to SessionProfile. AI-Chat writes intents here instead of
//! invoking shell commands. The build daemon polls this queue and executes
//! based on priority and CI state.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProfile {
    // ... existing fields ...
    
    /// AI-Chat Exclusive: **Declarative Build Queue**
    /// AI pushes structured intents here. The build daemon processes them
    /// asynchronously, respecting session invariants and CI digests.
    #[serde(default)]
    pub build_queue: Vec<sonia_build_agent::BuildIntent>,

    /// Current orchestration mode dictated by CI health
    #[serde(default = "default_compute_mode")]
    pub compute_mode: ComputeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComputeMode {
    Explore,
    FixOnly,
    Benchmark,
}

fn default_compute_mode() -> ComputeMode { ComputeMode::Explore }

// Integration note: `gamemodeai-session` CLI exposes `updatesession --add-build-intent <json>`
// which appends to `build_queue`. The build daemon watches this file or polls via CLI.
