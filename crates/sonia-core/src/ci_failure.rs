use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Machine-readable classification of CI and build failures.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CiFailureKind {
    // Existing kinds (examples; keep yours)
    CompileError,
    TestFailure,
    SchemaViolation,
    ToolchainMissing,
    ToolchainMismatch,
    BudgetOverflow,
    DeterminismViolation,

    /// Worker ran out of disk space while running a build.
    DiskFull,

    /// A non-fatal condition where maintenance is suggested
    /// (for example, large target/ or cache directories).
    MaintenanceSuggested,
}

/// One normalized CI failure entry that AI and tools consume.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CiFailure {
    /// Logical crate or subsystem name, e.g. "n64-layout" or "starzip-budget".
    pub crate_name: String,

    /// Classification of the failure.
    pub kind: CiFailureKind,

    /// Human-readable summary (1–2 sentences).
    pub message: String,

    /// Optional log URL or relative path for humans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_url: Option<String>,

    /// Optional file path associated with the failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Optional 1-based line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Optional 1-based column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Overall CI status summary stored in SessionProfile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CiStatus {
    /// Last CI run identifier (workflow run id, job id, etc.).
    pub last_run_id: String,

    /// Short summary string.
    pub summary: String,

    /// All normalized failures from the last run.
    #[serde(default)]
    pub failures: Vec<CiFailure>,
}
