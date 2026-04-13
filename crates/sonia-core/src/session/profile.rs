use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComputeMode {
    /// AI can propose any changes, subject to invariants and CI.
    ExploreLayouts,
    /// AI is restricted to bugfix / optimization changes; no new systems, no ABI breaks.
    FixOnly,
    /// Experiments that may violate determinism or budgets are allowed, but must be gated.
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64SessionConstraints {
    pub active: bool,
    pub n64_rom_ceiling_bytes: u64,
    pub n64_rdram_bytes: u32,
    pub allow_non_deterministic_experiments: bool,
    /// Path to an N64Constraints JSON profile to be passed into ai_checklist.
    pub constraints_profile_path: Option<String>,
}

/// Existing CiFailure / CiStatus types are assumed; we extend them
/// only if needed for richer failure classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiFailure {
    pub crate_name: String,
    pub kind: String,
    pub message: String,
    pub log_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CiStatus {
    pub last_run_id: String,
    pub summary: String,
    pub failures: Vec<CiFailure>,
}

/// SessionProfile is the single source of truth for current constraints and CI health.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProfile {
    pub repo: String,
    pub branch: String,
    pub active_crate: String,
    pub feature_flags: Vec<String>,
    pub invariants: Vec<crate::session::InvariantRule>,
    pub recent_todos: Vec<crate::session::TodoItem>,
    pub ci_status: CiStatus,

    // New fields for platform-aware orchestration:
    pub compute_mode: ComputeMode,

    /// Optional N64-specific session constraints.
    pub n64_constraints: Option<N64SessionConstraints>,
}
