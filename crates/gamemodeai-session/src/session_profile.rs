// crates/gamemodeai-session/src/session_profile.rs

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use sonia_build_agent::BuildIntent;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(default)]
    pub can_access_private_crates: bool,
    #[serde(default)]
    pub can_trigger_deploy_workflows: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionProfile {
    pub repo: String,
    pub branch: String,
    // ... existing fields ...

    #[serde(default)]
    pub capabilities: Capabilities,

    #[serde(default)]
    pub build_queue: Vec<BuildIntent>,

    #[serde(default = "default_compute_mode")]
    pub compute_mode: ComputeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComputeMode {
    Explore,
    FixOnly,
    Benchmark,
}

fn default_compute_mode() -> ComputeMode {
    ComputeMode::Explore
}
