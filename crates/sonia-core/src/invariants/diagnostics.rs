// Filename: cratessonia-core/src/invariants/diagnostics.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level violation kind taxonomy, shared across invariants.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ViolationKind {
    DeterminismViolation,
    BudgetOverflow,
    AbiBreakage,
    SchemaViolation,
}

/// Generic source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

/// Shape of a machine-readable fix suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixSuggestion {
    pub edit_kind: String,
    pub target: String,
    pub template: String,
}

/// Standard diagnostic payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub kind: ViolationKind,
    pub location: Option<Location>,
    pub constraint_id: String,
    pub explanation: String,
    pub fix_suggestions: Vec<FixSuggestion>,
}
