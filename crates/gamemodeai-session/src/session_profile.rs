// crates/gamemodeai-session/src/session_profile.rs

use serde::{Deserialize, Serialize};
use sonia_build_agent::BuildIntent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProfile {
    // existing fields...

    #[serde(default)]
    pub build_queue: Vec<BuildIntent>,

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

fn default_compute_mode() -> ComputeMode {
    ComputeMode::Explore
}
