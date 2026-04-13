// Filename: cratessonia-core/src/invariants/determinism.rs

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};

/// Structured details for simple, string-based determinism violations found by the checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeterminismViolationDetails {
    pub crate_root: String,
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub rule_id: String,
    pub suggestion: Option<String>,
}

/// High-level role for a crate or module from the determinism perspective.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeterminismRole {
    /// Must be fully deterministic: no OS time, no thread RNG, no IO in ECS.
    DeterministicCore,
    /// May use nondeterminism, but should be sandboxed away from core ECS.
    SandboxOk,
    /// Tooling or build-time only; rules are advisory.
    ToolOnly,
}

/// Logical scope where a rule applies.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    Crate,
    Module,
    Function,
}

/// Simple features extracted from a crate’s source tree by a lightweight scanner.
///
/// This is the F in r_k(F) from the notes: features like imports I, collections C, calls A.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramFeatures {
    pub crate_name: String,
    pub role: DeterminismRole,
    pub imported_symbols: Vec<String>,
    pub used_types: Vec<String>,
    pub called_functions: Vec<String>,
    pub map_iterations: Vec<MapIterationFeature>,
    pub io_usage: Vec<IoUsageFeature>,
}

/// Specific feature: iteration over a potentially unstable map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapIterationFeature {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    /// e.g. "std::collections::HashMap"
    pub map_type: String,
    /// Whether the iteration appears wrapped in a sort call.
    pub sorted: bool,
}

/// Specific feature: IO usage in code that may run inside ECS systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IoUsageFeature {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub api: String,
}

/// Rule identifier for determinism invariants.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum DeterminismRuleId {
    ForbiddenImportRandThreadRng,
    ForbiddenImportStdInstant,
    ForbiddenImportStdSystemTime,
    ForbiddenOsTimer,
    ForbiddenUnseededRng,
    UnstableHashMapIteration,
    IoInEcsSystem,
}

/// Logical evaluation result of a rule r_k(F) ∈ {0,1}.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuleSatisfaction {
    Satisfied,
    Violated,
}

/// A single rule evaluation over extracted ProgramFeatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeterminismRuleResult {
    pub rule_id: DeterminismRuleId,
    pub satisfaction: RuleSatisfaction,
    /// Optional violations contributing to the failure.
    pub violations: Vec<DeterminismViolation>,
}

/// Shared diagnostic/fix structure for determinism violations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeterminismViolation {
    pub rule_id: DeterminismRuleId,
    pub location: Option<SourceLocation>,
    pub explanation: String,
    pub suggested_patches: Vec<SuggestedPatch>,
}

/// Generic source location for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
}

/// High-level edit kind AI/tools can apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditKind {
    ReplaceImport,
    ReplaceType,
    AddSeedParam,
    WrapWithSortedIteration,
    MoveToSandboxCrate,
}

/// Minimal patch template for autocorrection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedPatch {
    pub edit_kind: EditKind,
    /// e.g. "rand::thread_rng", "std::collections::HashMap"
    pub target: String,
    /// Template or short snippet explaining the replacement pattern.
    pub template: String,
}

/// Combined static + dynamic view for a crate’s determinism status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeterminismReport {
    pub crate_name: String,
    pub role: DeterminismRole,
    pub rules: Vec<DeterminismRuleResult>,
    /// Optional hash comparison failures from replay harness.
    pub replay_failures: Vec<ReplayFailure>,
}

/// Dynamic determinism failure from the ECS replay harness.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFailure {
    pub test_name: String,
    pub tick: u32,
    pub hash_run1: String,
    pub hash_run2: String,
    /// Static rule_ids that were already known to be violated in this crate.
    pub suspected_rules: Vec<DeterminismRuleId>,
}

/// Evaluate all core determinism rules over extracted ProgramFeatures.
///
/// This is the logical conjunction ⋀_k r_k(F) = 1; violations correspond to rules where
/// satisfaction = Violated.
pub fn evaluate_rules(features: &ProgramFeatures) -> DeterminismReport {
    let mut rules = Vec::new();

    rules.push(check_forbidden_import(
        features,
        DeterminismRuleId::ForbiddenImportRandThreadRng,
        "rand::thread_rng",
        "Use a centralized, seeded RNG passed into systems instead of rand::thread_rng().",
        SuggestedPatch {
            edit_kind: EditKind::ReplaceImport,
            target: "rand::thread_rng".to_string(),
            template: "fn system(rng: &mut impl Rng, ...) { /* use rng instead of thread_rng() */ }"
                .to_string(),
        },
    ));

    rules.push(check_forbidden_import(
        features,
        DeterminismRuleId::ForbiddenImportStdInstant,
        "std::time::Instant",
        "Use fixed-step simulation ticks or a deterministic frame index instead of std::time::Instant.",
        SuggestedPatch {
            edit_kind: EditKind::ReplaceType,
            target: "std::time::Instant".to_string(),
            template: "struct FrameTime { tick: u64 } // pass tick from the scheduler".to_string(),
        },
    ));

    rules.push(check_forbidden_import(
        features,
        DeterminismRuleId::ForbiddenImportStdSystemTime,
        "std::time::SystemTime",
        "Avoid wall-clock time in deterministic cores; depend only on input frames and ticks.",
        SuggestedPatch {
            edit_kind: EditKind::ReplaceType,
            target: "std::time::SystemTime".to_string(),
            template: "Use a deterministic tick counter or simulation time scalar instead."
                .to_string(),
        },
    ));

    rules.push(check_hashmap_iteration(features));
    rules.push(check_io_in_ecs_system(features));

    DeterminismReport {
        crate_name: features.crate_name.clone(),
        role: features.role,
        rules,
        replay_failures: Vec::new(),
    }
}

fn check_forbidden_import(
    features: &ProgramFeatures,
    rule_id: DeterminismRuleId,
    forbidden: &str,
    explanation: &str,
    patch: SuggestedPatch,
) -> DeterminismRuleResult {
    let mut violations = Vec::new();

    for imp in &features.imported_symbols {
        if imp == forbidden {
            violations.push(DeterminismViolation {
                rule_id,
                location: None,
                explanation: explanation.to_string(),
                suggested_patches: vec![patch.clone()],
            });
        }
    }

    let satisfaction = if violations.is_empty() {
        RuleSatisfaction::Satisfied
    } else {
        RuleSatisfaction::Violated
    };

    DeterminismRuleResult {
        rule_id,
        satisfaction,
        violations,
    }
}

fn check_hashmap_iteration(features: &ProgramFeatures) -> DeterminismRuleResult {
    let rule_id = DeterminismRuleId::UnstableHashMapIteration;
    let mut violations = Vec::new();

    for it in &features.map_iterations {
        if it.map_type == "std::collections::HashMap" && !it.sorted {
            violations.push(DeterminismViolation {
                rule_id,
                location: Some(SourceLocation {
                    file: it.file.clone(),
                    line: it.line,
                    column: it.column,
                }),
                explanation: "Iteration over std::collections::HashMap without explicit sorting \
                              is not deterministic across runs or platforms."
                    .to_string(),
                suggested_patches: vec![
                    SuggestedPatch {
                        edit_kind: EditKind::WrapWithSortedIteration,
                        target: "std::collections::HashMap".to_string(),
                        template: "let mut items: Vec<_> = map.iter().collect(); \
                                   items.sort_by_key(|(k, _)| *k); \
                                   for (k, v) in items { /* ... */ }"
                            .to_string(),
                    },
                    SuggestedPatch {
                        edit_kind: EditKind::ReplaceType,
                        target: "std::collections::HashMap".to_string(),
                        template: "use indexmap::IndexMap; // preserves insertion order"
                            .to_string(),
                    },
                ],
            });
        }
    }

    let satisfaction = if violations.is_empty() {
        RuleSatisfaction::Satisfied
    } else {
        RuleSatisfaction::Violated
    };

    DeterminismRuleResult {
        rule_id,
        satisfaction,
        violations,
    }
}

fn check_io_in_ecs_system(features: &ProgramFeatures) -> DeterminismRuleResult {
    let rule_id = DeterminismRuleId::IoInEcsSystem;
    let mut violations = Vec::new();

    if matches!(features.role, DeterminismRole::DeterministicCore) {
        for io in &features.io_usage {
            violations.push(DeterminismViolation {
                rule_id,
                location: Some(SourceLocation {
                    file: io.file.clone(),
                    line: io.line,
                    column: io.column,
                }),
                explanation: "Filesystem or network IO inside deterministic ECS systems breaks \
                              replay determinism. Move this call into a driver crate and pass \
                              results as data."
                    .to_string(),
                suggested_patches: vec![SuggestedPatch {
                    edit_kind: EditKind::MoveToSandboxCrate,
                    target: io.api.clone(),
                    template: "Create a separate IO or driver crate that performs this call \
                               and passes data into ECS via events or components."
                        .to_string(),
                }],
            });
        }
    }

    let satisfaction = if violations.is_empty() {
        RuleSatisfaction::Satisfied
    } else {
        RuleSatisfaction::Violated
    };

    DeterminismRuleResult {
        rule_id,
        satisfaction,
        violations,
    }
}

/// Attach dynamic replay failures to an existing static report.
pub fn attach_replay_failures(
    mut report: DeterminismReport,
    failures: Vec<ReplayFailure>,
) -> DeterminismReport {
    report.replay_failures = failures;
    report
}

/// Heuristic, file-system based determinism checker used by the AI checklist.
///
/// This sits on top of the static rule model but currently only emits string-based diagnostics.
pub struct DeterminismChecker;

impl DeterminismChecker {
    pub fn new() -> Self {
        Self
    }

    fn walk_rust_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut stack = vec![root.to_path_buf()];

        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path
                    .extension()
                    .map(|e| e == "rs")
                    .unwrap_or(false)
                {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    fn scan_forbidden_patterns(
        crate_root: &Path,
        file: &Path,
        contents: &str,
        messages: &mut Vec<CheckMessage>,
    ) -> anyhow::Result<()> {
        let crate_root_str = crate_root.to_string_lossy().to_string();
        let file_str = file
            .strip_prefix(crate_root)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string();

        let mut add_violation = |rule_id: &str, msg: &str, suggestion: Option<&str>| {
            let details = DeterminismViolationDetails {
                crate_root: crate_root_str.clone(),
                file: file_str.clone(),
                line: None,
                column: None,
                rule_id: rule_id.to_string(),
                suggestion: suggestion.map(|s| s.to_string()),
            };

            messages.push(CheckMessage {
                code: format!("DETERMINISM_RULE_{}", rule_id),
                message: format!("{} in {}", msg, file_str),
                file: Some(file_str.clone()),
                line: None,
                column: None,
                details: serde_json::to_value(details).unwrap_or(serde_json::Value::Null),
            });
        };

        // Heuristic string-based checks; can be replaced by AST analysis later.

        if contents.contains("rand::thread_rng") || contents.contains("thread_rng()") {
            add_violation(
                "FORBID_THREAD_RNG",
                "Use of rand::thread_rng (non-deterministic RNG)",
                Some("Replace with a seeded RNG instance passed through system state"),
            );
        }

        if contents.contains("std::time::Instant") || contents.contains("std::time::SystemTime") {
            add_violation(
                "FORBID_WALLCLOCK_TIME",
                "Use of std::time (wall-clock dependent)",
                Some(
                    "Inject time as an explicit input or use a deterministic tick counter in world state",
                ),
            );
        }

        if contents.contains("HashMap<") || contents.contains("std::collections::HashMap") {
            add_violation(
                "FORBID_HASHMAP",
                "Use of HashMap may lead to non-deterministic iteration order",
                Some("Replace HashMap with indexmap::IndexMap or a stable-ordered map type"),
            );
        }

        if contents.contains("HashSet<") || contents.contains("std::collections::HashSet") {
            add_violation(
                "FORBID_HASHSET",
                "Use of HashSet may lead to non-deterministic iteration order",
                Some("Replace HashSet with indexmap::IndexSet or a stable-ordered set type"),
            );
        }

        Ok(())
    }
}

impl Check for DeterminismChecker {
    fn run(&self, input: &crate::aichecklist::ChecklistInput) -> anyhow::Result<CheckResult> {
        let root_str = match &input.rust_crate_root {
            Some(r) => r.clone(),
            None => {
                return Ok(CheckResult {
                    check: CheckCode::Determinism,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "DETERMINISM_SKIPPED_NO_CRATE_ROOT".to_string(),
                        message: "Determinism check skipped: no rust_crate_root provided"
                            .to_string(),
                        file: None,
                        line: None,
                        column: None,
                        details: serde_json::Value::Null,
                    }],
                });
            }
        };

        let crate_root = Path::new(&root_str);
        let rust_files = Self::walk_rust_files(crate_root)?;

        let mut messages = Vec::new();

        for file in rust_files {
            let contents = std::fs::read_to_string(&file)?;
            Self::scan_forbidden_patterns(crate_root, &file, &contents, &mut messages)?;
        }

        let passed = messages.is_empty();

        Ok(CheckResult {
            check: CheckCode::Determinism,
            passed,
            severity: if passed {
                Severity::Info
            } else {
                Severity::Error
            },
            messages,
        })
    }
}
