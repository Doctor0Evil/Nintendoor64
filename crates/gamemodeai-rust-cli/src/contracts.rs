use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// High-level operation to run: check vs build.
/// You can extend this with test, clippy, doc later.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CargoOpKind {
    Check,
    Build,
}

/// Which crates / packages to operate on.
/// For now we support three modes to cover real-world workflows:
/// - workspace: build/check entire workspace
/// - packages: specific package names
/// - manifest_path: explicit Cargo.toml path (for nested workspaces)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CrateSelection {
    Workspace,
    Packages(Vec<String>),
    ManifestPath(String),
}

/// Parameters for a single cargo run, suitable as JSON-in for the CLI.
///
/// This is intentionally stable and small; anything not here is considered
/// out of scope for AI and must be wired via orchestrator (env, profiles, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunCargoParams {
    /// What to do: check vs build.
    pub op: CargoOpKind,

    /// Which crates / packages to affect.
    #[serde(default = "default_crate_selection")]
    pub selection: CrateSelection,

    /// Features to enable, passed as `--features` or `--all-features`.
    #[serde(default)]
    pub features: Vec<String>,

    /// If true, use `--all-features`.
    #[serde(default)]
    pub all_features: bool,

    /// Optional target triple, e.g. "x86_64-unknown-linux-gnu" or "mips64-unknown-elf".
    #[serde(default)]
    pub target: Option<String>,

    /// Build profile: "debug", "release", or CI-specific like "ci".
    /// Mapped to `--release` or custom profiles by server policy.
    #[serde(default = "default_profile")]
    pub profile: String,

    /// Optional `CARGO_TARGET_DIR` override (relative to workspace root).
    #[serde(default)]
    pub target_dir: Option<String>,

    /// Extra flags allowed by policy (e.g. `["--locked"]`), explicitly whitelisted.
    #[serde(default)]
    pub extra_flags: Vec<String>,
}

fn default_profile() -> String {
    "debug".to_string()
}

fn default_crate_selection() -> CrateSelection {
    CrateSelection::Workspace
}

/// Simplified diagnostic level.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
    Unknown,
}

/// Simplified span into source files.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticSpan {
    pub file: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub column_start: Option<u32>,
    pub column_end: Option<u32>,
}

/// A single diagnostic message distilled from `cargo --message-format=json`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub message: String,
    #[serde(default)]
    pub spans: Vec<DiagnosticSpan>,
    /// Optional rendered string for human display; AI can ignore if it has spans.
    #[serde(default)]
    pub rendered: Option<String>,
}

/// Simple log event stream, for streaming over WebSocket if desired.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildLogEvent {
    pub kind: String, // "Info", "Warning", "Error", "Progress"
    pub message: String,
    #[serde(default)]
    pub timestamp_utc: Option<String>,
}

/// Final aggregated result of a cargo run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RunCargoResult {
    /// "ok" or "error".
    pub status: String,
    pub exit_code: i32,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub log_events: Vec<BuildLogEvent>,
}
