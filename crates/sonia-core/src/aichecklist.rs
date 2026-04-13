// Filename: cratessonia-core/src/aichecklist.rs

use serde::{Deserialize, Serialize};

use crate::invariants::{
    abi_guard,
    determinism,
    hardware_budget,
    patch_safety,
};

use crate::invariants::{
    abi_guard::{AbiGuardChecker},
    determinism::DeterminismChecker,
    hardware_budget::HardwareBudgetChecker,
    patch_safety::PatchSafetyChecker,
};

/// High-level classification of what is being checked.
/// This lets AI and CI route different artifact bundles to different subsets of checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactKind {
    /// Generic artifact bundle: may contain source, JSON specs, and layouts.
    Generic,
    /// N64 patch bundle: PatchSpec + ArtifactSpecs + RomLayout.
    N64Patch,
    /// N64/PS1 build bundle with layout and constraints.
    ConsoleBuild,
    /// Rust crate or workspace for determinism / ABI analysis.
    RustCrate,
}

/// Represents a logical unit of work to validate.
/// In practice this will be derived from SessionProfile plus Sonia / Starzip artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistInput {
    pub kind: ArtifactKind,

    /// Optional path to a RomLayout JSON for N64.
    pub n64_rom_layout_path: Option<String>,

    /// Optional path to an N64 constraints JSON profile.
    pub n64_constraints_path: Option<String>,

    /// Optional path to a PatchSpec JSON for N64 patching.
    pub n64_patch_spec_path: Option<String>,

    /// Optional bundle of ArtifactSpecs for patch payloads, configs, etc.
    /// Typically a small JSON file that lists multiple ArtifactSpec entries.
    pub artifact_specs_path: Option<String>,

    /// Root directory of a Rust crate or workspace to analyze.
    pub rust_crate_root: Option<String>,

    /// Optional path to a baseline C API descriptor (for ABI diffing).
    pub abi_baseline_path: Option<String>,

    /// Optional path to the current C API descriptor.
    pub abi_current_path: Option<String>,

    /// Optional SessionProfile JSON path so the checklist can read invariants
    /// and adjust which checks are active or how strict they are.
    pub session_profile_path: Option<String>,
}

/// Machine-readable classification for checklist results.
/// Keep these stable: AI and CI will branch on these codes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckCode {
    PatchSafety,
    HardwareBudget,
    Determinism,
    AbiGuard,
}

/// Severity is separate from pass/fail: CI may treat WARN as non-fatal
/// while AI still uses it as guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// A single violation or informational note from a check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckMessage {
    /// Short machine-readable identifier.
    pub code: String,

    /// Human-friendly description for logs and developers.
    pub message: String,

    /// Optional file path, if the violation is associated with a specific file.
    pub file: Option<String>,

    /// Optional 1-based line number.
    pub line: Option<u32>,

    /// Optional 1-based column number.
    pub column: Option<u32>,

    /// Optional arbitrary JSON payload for AI/CI consumers (e.g. budget deltas, suggestions).
    pub details: serde_json::Value,
}

/// A single check result row in the checklist matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub check: CheckCode,
    pub passed: bool,
    pub severity: Severity,
    pub messages: Vec<CheckMessage>,
}

/// Top-level response produced by ai_checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistReport {
    /// Echoes the input kind for clarity.
    pub kind: ArtifactKind,

    /// Overall pass/fail flag: false if any Error-level check fails.
    pub overall_passed: bool,

    /// Individual check results.
    pub results: Vec<CheckResult>,
}

/// Trait implemented by each invariant module.
pub trait Check {
    fn run(&self, input: &ChecklistInput) -> anyhow::Result<CheckResult>;
}

/// A small struct that owns the four core checks and can be extended later.
pub struct Checklist {
    patch_safety: PatchSafetyChecker,
    hardware_budget: HardwareBudgetChecker,
    determinism: DeterminismChecker,
    abi_guard: AbiGuardChecker,
}

impl Checklist {
    pub fn new() -> Self {
        Self {
            patch_safety: PatchSafetyChecker::new(),
            hardware_budget: HardwareBudgetChecker::new(),
            determinism: DeterminismChecker::new(),
            abi_guard: AbiGuardChecker::new(),
        }
    }

    pub fn run_all(&self, input: &ChecklistInput) -> anyhow::Result<ChecklistReport> {
        let mut results = Vec::new();

        // Patch safety is only meaningful when working with an N64 patch bundle or console build.
        if matches!(input.kind, ArtifactKind::N64Patch | ArtifactKind::ConsoleBuild) {
            if let Some(result) = self.run_patch_safety_if_applicable(input)? {
                results.push(result);
            }
        }

        // Hardware budgets apply to console builds and N64 patch bundles that might change assets.
        if matches!(input.kind, ArtifactKind::N64Patch | ArtifactKind::ConsoleBuild) {
            if let Some(result) = self.run_hardware_budget_if_applicable(input)? {
                results.push(result);
            }
        }

        // Determinism is relevant whenever a Rust crate is in play.
        if input.rust_crate_root.is_some() {
            let result = self.determinism.run(input)?;
            results.push(result);
        }

        // ABI guard is only relevant when C API descriptors are present.
        if input.abi_baseline_path.is_some() && input.abi_current_path.is_some() {
            let result = self.abi_guard.run(input)?;
            results.push(result);
        }

        let overall_passed = results.iter().all(|r| {
            r.passed || matches!(r.severity, Severity::Info | Severity::Warning)
        });

        Ok(ChecklistReport {
            kind: input.kind.clone(),
            overall_passed,
            results,
        })
    }

    fn run_patch_safety_if_applicable(
        &self,
        input: &ChecklistInput,
    ) -> anyhow::Result<Option<CheckResult>> {
        if input.n64_rom_layout_path.is_none() || input.n64_patch_spec_path.is_none() {
            return Ok(None);
        }
        let result = self.patch_safety.run(input)?;
        Ok(Some(result))
    }

    fn run_hardware_budget_if_applicable(
        &self,
        input: &ChecklistInput,
    ) -> anyhow::Result<Option<CheckResult>> {
        if input.n64_rom_layout_path.is_none() || input.n64_constraints_path.is_none() {
            return Ok(None);
        }
        let result = self.hardware_budget.run(input)?;
        Ok(Some(result))
    }
}

// JSON-facing wrapper suitable for Sonia's JSON envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistRequest {
    pub input: ChecklistInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistResponse {
    pub report: ChecklistReport,
}

/// Library entry point: checks are pure with respect to JSON inputs and filesystem.
pub fn run_checklist(input: ChecklistInput) -> anyhow::Result<ChecklistReport> {
    let checklist = Checklist::new();
    checklist.run_all(&input)
}

/// CLI adapter that can be called from the Sonia JSON envelope handler.
/// - Request:  { "version": 1, "command": "runChecklist", "params": { ...ChecklistRequest... } }
/// - Response: { "version": 1, "status": "ok", "data": { "report": ... } }
pub fn handle_run_checklist(params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let req: ChecklistRequest = serde_json::from_value(params)?;
    let report = run_checklist(req.input)?;
    let resp = ChecklistResponse { report };
    Ok(serde_json::to_value(resp)?)
}
