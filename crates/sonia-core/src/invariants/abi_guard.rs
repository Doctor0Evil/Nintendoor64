use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};

/// Minimal representation of a C symbol extracted from cbindgen/bindgen output.
/// You can refine this as needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiSymbol {
    pub name: String,
    pub kind: String,      // "function", "struct", "enum", ...
    pub signature: String, // canonical string, e.g., "fn foo(i32) -> i32"
    pub stability: Option<String>, // "public", "internal", "experimental"
}

/// ABI descriptor file format: map from symbol name to symbol metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiDescriptor {
    pub symbols: BTreeMap<String, AbiSymbol>,
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

    fn load_descriptor(path: &str) -> anyhow::Result<AbiDescriptor> {
        let text = std::fs::read_to_string(path)?;
        let desc: AbiDescriptor = serde_json::from_str(&text)?;
        Ok(desc)
    }

    fn is_public(symbol: &AbiSymbol) -> bool {
        match &symbol.stability {
            Some(s) if s.eq_ignore_ascii_case("public") => true,
            _ => false,
        }
    }
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
                        message:
                            "ABI guard check skipped: no abi_baseline_path provided".to_string(),
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

        let baseline = Self::load_descriptor(baseline_path)?;
        let current = Self::load_descriptor(current_path)?;

        let mut messages = Vec::new();
        let mut passed = true;

        let baseline_names: BTreeSet<_> = baseline.symbols.keys().cloned().collect();
        let current_names: BTreeSet<_> = current.symbols.keys().cloned().collect();

        // Removed symbols
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
                message: format!(
                    "Public symbol '{}' was removed from the C API",
                    name
                ),
                file: Some(baseline_path.clone()),
                line: None,
                column: None,
                details: serde_json::to_value(details)?,
            });
        }

        // Added or changed symbols
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

        // It is usually fine to add new symbols, but you may choose to treat
        // additions as warnings for AI so it knows the surface expanded.
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
