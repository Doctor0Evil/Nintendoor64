// Filename: crates/sonia-core/src/invariants/abi_guard.rs

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};

/// Semantic version triple (M, m, p).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Public classification of a C symbol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AbiVisibility {
    Public,
    Internal,
    Experimental,
}

/// C ABI kind of symbol.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AbiSymbolKind {
    Function,
    Struct,
    Enum,
    Constant,
}

/// A single symbol snapshot in the ABI set A = { s_i }.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiSymbol {
    pub name: String,
    pub kind: AbiSymbolKind,
    /// e.g. "fn(void*, const GmWorld*, float) -> int"
    pub signature: String,
    /// Hash over layout details: sizes, alignments, field order.
    pub layout_hash: u64,
    pub visibility: AbiVisibility,
}

/// Full ABI snapshot for a crate/release.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiSnapshot {
    pub crate_name: String,
    pub version: SemVer,
    pub symbols: Vec<AbiSymbol>,
}

/// Kind of ABI change detected by diff.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AbiChangeKind {
    AddSymbol,
    RemoveSymbol,
    SignatureChanged,
    LayoutChanged,
}

/// One detected change between two ABI snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiChange {
    pub kind: AbiChangeKind,
    pub symbol_name: String,
    pub visibility: AbiVisibility,
    pub old: Option<AbiSymbol>,
    pub new: Option<AbiSymbol>,
}

/// Classification of breaking vs non-breaking changes.
///
/// For simplicity and safety, any change to Public symbols is treated as breaking
/// except for purely additive additions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiDiff {
    pub old_version: SemVer,
    pub new_version: SemVer,
    pub changes: Vec<AbiChange>,
    pub breaking_changes: Vec<AbiChange>,
    pub additive_changes: Vec<AbiChange>,
}

/// Migration plan entry for a single breaking change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationEntry {
    pub change: AbiChange,
    /// Suggested wrapper/adapter name or deprecation stub.
    pub migration_note: String,
}

/// Overall migration plan proposed by AI/tools for a diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPlan {
    pub from_version: SemVer,
    pub to_version: SemVer,
    pub entries: Vec<MigrationEntry>,
}

/// Minimal representation of a C symbol extracted from cbindgen/bindgen output.
/// This is used by the ABI guard check over descriptor JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiDescriptorSymbol {
    pub name: String,
    pub kind: String,             // "function", "struct", "enum", ...
    pub signature: String,        // canonical string, e.g., "fn foo(i32) -> i32"
    pub stability: Option<String> // "public", "internal", "experimental"
}

/// ABI descriptor file format: map from symbol name to symbol metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiDescriptor {
    pub symbols: BTreeMap<String, AbiDescriptorSymbol>,
}

/// Machine-readable ABI violation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiViolationDetails {
    pub symbol: String,
    pub violation_code: String,
    pub baseline_signature: Option<String>,
    pub current_signature: Option<String>,
    pub stability: Option<String>,
}

pub struct AbiGuardChecker;

impl AbiGuardChecker {
    pub fn new() -> Self {
        Self
    }

    fn load_descriptor(path: &Path) -> anyhow::Result<AbiDescriptor> {
        let text = fs::read_to_string(path)?;
        let desc: AbiDescriptor = serde_json::from_str(&text)?;
        Ok(desc)
    }

    fn is_public(symbol: &AbiDescriptorSymbol) -> bool {
        match &symbol.stability {
            Some(s) if s.eq_ignore_ascii_case("public") => true,
            _ => false,
        }
    }
}

/// Compute the ABI diff A_old, A_new and classify breaking vs additive changes.
///
/// Breaking changes:
/// - Public symbol removed.
/// - Public symbol signature changed.
/// - Public symbol layout_hash changed.
pub fn diff_abi(old: &AbiSnapshot, new: &AbiSnapshot) -> AbiDiff {
    let mut changes = Vec::new();

    let mut old_map: BTreeMap<String, &AbiSymbol> = BTreeMap::new();
    let mut new_map: BTreeMap<String, &AbiSymbol> = BTreeMap::new();

    for s in &old.symbols {
        old_map.insert(s.name.clone(), s);
    }
    for s in &new.symbols {
        new_map.insert(s.name.clone(), s);
    }

    let old_names: BTreeSet<_> = old_map.keys().cloned().collect();
    let new_names: BTreeSet<_> = new_map.keys().cloned().collect();

    for name in old_names.difference(&new_names) {
        if let Some(sym) = old_map.get(name) {
            changes.push(AbiChange {
                kind: AbiChangeKind::RemoveSymbol,
                symbol_name: name.clone(),
                visibility: sym.visibility,
                old: Some((*sym).clone()),
                new: None,
            });
        }
    }

    for name in new_names.difference(&old_names) {
        if let Some(sym) = new_map.get(name) {
            changes.push(AbiChange {
                kind: AbiChangeKind::AddSymbol,
                symbol_name: name.clone(),
                visibility: sym.visibility,
                old: None,
                new: Some((*sym).clone()),
            });
        }
    }

    for name in old_names.intersection(&new_names) {
        let old_sym = old_map.get(name).unwrap();
        let new_sym = new_map.get(name).unwrap();

        if old_sym.signature != new_sym.signature {
            changes.push(AbiChange {
                kind: AbiChangeKind::SignatureChanged,
                symbol_name: name.clone(),
                visibility: new_sym.visibility,
                old: Some((*old_sym).clone()),
                new: Some((*new_sym).clone()),
            });
        } else if old_sym.layout_hash != new_sym.layout_hash {
            changes.push(AbiChange {
                kind: AbiChangeKind::LayoutChanged,
                symbol_name: name.clone(),
                visibility: new_sym.visibility,
                old: Some((*old_sym).clone()),
                new: Some((*new_sym).clone()),
            });
        }
    }

    let mut breaking_changes = Vec::new();
    let mut additive_changes = Vec::new();

    for ch in &changes {
        match ch.kind {
            AbiChangeKind::AddSymbol => {
                additive_changes.push(ch.clone());
            }
            AbiChangeKind::RemoveSymbol
            | AbiChangeKind::SignatureChanged
            | AbiChangeKind::LayoutChanged => {
                if matches!(ch.visibility, AbiVisibility::Public) {
                    breaking_changes.push(ch.clone());
                } else {
                    // For internal/experimental, treat as additive for now; can be tightened later.
                    additive_changes.push(ch.clone());
                }
            }
        }
    }

    AbiDiff {
        old_version: old.version,
        new_version: new.version,
        changes,
        breaking_changes,
        additive_changes,
    }
}

/// Enforce semantic versioning constraints given a diff:
///
/// - If breaking_changes is non-empty, require M_new > M_old.
/// - If only additive changes, allow M_new == M_old with m_new ≥ m_old and p_new ≥ p_old.
pub fn check_version_compatibility(diff: &AbiDiff) -> Result<(), String> {
    let old = diff.old_version;
    let new = diff.new_version;

    if !diff.breaking_changes.is_empty() {
        if new.major <= old.major {
            return Err(format!(
                "ABI breaking changes detected but major version did not increase: old={:?}, new={:?}",
                old, new
            ));
        }
        return Ok(());
    }

    if new.major < old.major {
        return Err(format!(
            "New major version {} is less than old major {}",
            new.major, old.major
        ));
    }

    if new.major == old.major {
        if new.minor < old.minor {
            return Err(format!(
                "New minor version {} is less than old minor {}",
                new.minor, old.minor
            ));
        }
        if new.minor == old.minor && new.patch < old.patch {
            return Err(format!(
                "New patch version {} is less than old patch {}",
                new.patch, old.patch
            ));
        }
    }

    Ok(())
}

/// Verify that a MigrationPlan covers every breaking change by providing
/// at least one entry whose change.symbol_name matches.
pub fn validate_migration_plan(diff: &AbiDiff, plan: &MigrationPlan) -> Result<(), String> {
    if plan.from_version != diff.old_version || plan.to_version != diff.new_version {
        return Err("MigrationPlan version range does not match ABI diff".to_string());
    }

    let mut covered: BTreeSet<String> = BTreeSet::new();
    for entry in &plan.entries {
        covered.insert(entry.change.symbol_name.clone());
    }

    for ch in &diff.breaking_changes {
        if !covered.contains(&ch.symbol_name) {
            return Err(format!(
                "Breaking ABI change for symbol '{}' is not covered by migration plan",
                ch.symbol_name
            ));
        }
    }

    Ok(())
}

impl Check for AbiGuardChecker {
    fn run(&self, input: &crate::aichecklist::ChecklistInput) -> anyhow::Result<CheckResult> {
        let baseline_path = match &input.abi_baseline_path {
            Some(p) => p,
            None => {
                return Ok(CheckResult {
                    check: CheckCode::AbiGuard,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "ABI_GUARD_SKIPPED_NO_BASELINE".to_string(),
                        message: "ABI guard check skipped: no abi_baseline_path provided"
                            .to_string(),
                        file: None,
                        line: None,
                        column: None,
                        details: serde_json::Value::Null,
                    }],
                });
            }
        };

        let current_path = match &input.abi_current_path {
            Some(p) => p,
            None => {
                return Ok(CheckResult {
                    check: CheckCode::AbiGuard,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "ABI_GUARD_SKIPPED_NO_CURRENT".to_string(),
                        message: "ABI guard check skipped: no abi_current_path provided"
                            .to_string(),
                        file: None,
                        line: None,
                        column: None,
                        details: serde_json::Value::Null,
                    }],
                });
            }
        };

        let baseline = Self::load_descriptor(Path::new(baseline_path))?;
        let current = Self::load_descriptor(Path::new(current_path))?;

        let mut messages = Vec::new();
        let mut passed = true;

        let baseline_names: BTreeSet<_> = baseline.symbols.keys().cloned().collect();
        let current_names: BTreeSet<_> = current.symbols.keys().cloned().collect();

        // Removed symbols.
        for name in baseline_names.difference(&current_names) {
            let sym = &baseline.symbols[name];
            if !Self::is_public(sym) {
                continue;
            }
            passed = false;

            let details = AbiViolationDetails {
                symbol: name.clone(),
                violation_code: "ABI_SYMBOL_REMOVED".to_string(),
                baseline_signature: Some(sym.signature.clone()),
                current_signature: None,
                stability: sym.stability.clone(),
            };

            messages.push(CheckMessage {
                code: "ABI_BREAKING_CHANGE_PUBLIC_SYMBOL".to_string(),
                message: format!("Public symbol '{}' was removed from the C API", name),
                file: Some(baseline_path.clone()),
                line: None,
                column: None,
                details: serde_json::to_value(details)?,
            });
        }

        // Changed symbols.
        for name in current_names.intersection(&baseline_names) {
            let base_sym = &baseline.symbols[name];
            let curr_sym = &current.symbols[name];

            if !Self::is_public(base_sym) {
                continue;
            }

            if base_sym.signature != curr_sym.signature {
                passed = false;

                let details = AbiViolationDetails {
                    symbol: name.clone(),
                    violation_code: "ABI_SYMBOL_SIGNATURE_CHANGED".to_string(),
                    baseline_signature: Some(base_sym.signature.clone()),
                    current_signature: Some(curr_sym.signature.clone()),
                    stability: base_sym.stability.clone(),
                };

                messages.push(CheckMessage {
                    code: "ABI_BREAKING_CHANGE_PUBLIC_SYMBOL".to_string(),
                    message: format!(
                        "Public symbol '{}' changed signature from '{}' to '{}'",
                        name, base_sym.signature, curr_sym.signature
                    ),
                    file: Some(current_path.clone()),
                    line: None,
                    column: None,
                    details: serde_json::to_value(details)?,
                });
            }
        }

        // Added symbols (informational for AI).
        for name in current_names.difference(&baseline_names) {
            let sym = &current.symbols[name];
            if !Self::is_public(sym) {
                continue;
            }

            let details = AbiViolationDetails {
                symbol: name.clone(),
                violation_code: "ABI_SYMBOL_ADDED".to_string(),
                baseline_signature: None,
                current_signature: Some(sym.signature.clone()),
                stability: sym.stability.clone(),
            };

            messages.push(CheckMessage {
                code: "ABI_PUBLIC_SYMBOL_ADDED".to_string(),
                message: format!("New public symbol '{}' added to the C API", name),
                file: Some(current_path.clone()),
                line: None,
                column: None,
                details: serde_json::to_value(details)?,
            });
        }

        Ok(CheckResult {
            check: CheckCode::AbiGuard,
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
