use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};

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
        let file_str = file.strip_prefix(crate_root).unwrap_or(file).to_string_lossy().to_string();

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
                Some("Inject time as an explicit input or use a deterministic tick counter in world state"),
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
