use serde::{Deserialize, Serialize};

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};
use crate::n64::{layout::RomLayout, patchspec::PatchSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSafetyViolationDetails {
    pub patch_id: String,
    pub op_index: usize,
    pub reason: String,
    pub segment_name: Option<String>,
    pub rom_start: u32,
    pub rom_end: u32,
    pub conflicting_patch_id: Option<String>,
    pub conflicting_op_index: Option<usize>,
}

pub struct PatchSafetyChecker;

impl PatchSafetyChecker {
    pub fn new() -> Self {
        Self
    }

    fn load_layout(path: &str) -> anyhow::Result<RomLayout> {
        let text = std::fs::read_to_string(path)?;
        let layout: RomLayout = serde_json::from_str(&text)?;
        Ok(layout)
    }

    fn load_patch_spec(path: &str) -> anyhow::Result<PatchSpec> {
        let text = std::fs::read_to_string(path)?;
        let spec: PatchSpec = serde_json::from_str(&text)?;
        Ok(spec)
    }
}

impl Check for PatchSafetyChecker {
    fn run(&self, input: &crate::aichecklist::ChecklistInput) -> anyhow::Result<CheckResult> {
        let layout_path = match &input.n64_rom_layout_path {
            Some(p) => p,
            None => {
                return Ok(CheckResult {
                    check: CheckCode::PatchSafety,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "PATCH_SAFETY_SKIPPED_NO_LAYOUT".to_string(),
                        message: "Patch safety check skipped: no n64_rom_layout_path provided"
                            .to_string(),
                        file: None,
                        line: None,
                        column: None,
                        details: serde_json::Value::Null,
                    }],
                });
            }
        };

        let patch_path = match &input.n64_patch_spec_path {
            Some(p) => p,
            None => {
                return Ok(CheckResult {
                    check: CheckCode::PatchSafety,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "PATCH_SAFETY_SKIPPED_NO_PATCHSPEC".to_string(),
                        message: "Patch safety check skipped: no n64_patch_spec_path provided"
                            .to_string(),
                        file: None,
                        line: None,
                        column: None,
                        details: serde_json::Value::Null,
                    }],
                });
            }
        };

        let layout = Self::load_layout(layout_path)?;
        let spec = Self::load_patch_spec(patch_path)?;

        let mut messages = Vec::new();
        let mut intervals: Vec<(u32, u32, String, usize)> = Vec::new();

        // Map logical patch operations to ROM intervals using the RomLayout helpers.
        for (idx, op) in spec.ops.iter().enumerate() {
            match op.to_rom_interval(&layout) {
                Ok((start, end, segment_name)) => {
                    // Check containment: interval must lie within a mutable segment.
                    if !layout.is_interval_in_mutable_segment(start, end) {
                        let details = PatchSafetyViolationDetails {
                            patch_id: spec.id.clone(),
                            op_index: idx,
                            reason: "Interval not contained within a mutable segment".to_string(),
                            segment_name: Some(segment_name.clone()),
                            rom_start: start,
                            rom_end: end,
                            conflicting_patch_id: None,
                            conflicting_op_index: None,
                        };
                        messages.push(CheckMessage {
                            code: "PATCH_SAFETY_INTERVAL_NOT_CONTAINED".to_string(),
                            message: format!(
                                "Patch op {} writes [{:#08X}, {:#08X}) outside any mutable segment ({})",
                                idx, start, end, segment_name
                            ),
                            file: Some(spec.source_file_path.clone().unwrap_or_default()),
                            line: op.source_line,
                            column: op.source_column,
                            details: serde_json::to_value(details)?,
                        });
                    }

                    intervals.push((start, end, spec.id.clone(), idx));
                }
                Err(err) => {
                    let details = PatchSafetyViolationDetails {
                        patch_id: spec.id.clone(),
                        op_index: idx,
                        reason: format!("Failed to resolve patch operation to ROM interval: {err}"),
                        segment_name: None,
                        rom_start: 0,
                        rom_end: 0,
                        conflicting_patch_id: None,
                        conflicting_op_index: None,
                    };
                    messages.push(CheckMessage {
                        code: "PATCH_SAFETY_INTERVAL_RESOLUTION_FAILED".to_string(),
                        message: format!(
                            "Patch op {} could not be resolved to a ROM interval: {err}",
                            idx
                        ),
                        file: Some(spec.source_file_path.clone().unwrap_or_default()),
                        line: op.source_line,
                        column: op.source_column,
                        details: serde_json::to_value(details)?,
                    });
                }
            }
        }

        // Disjointness: sort by start offset and check for overlaps.
        intervals.sort_by_key(|(start, _, _, _)| *start);

        for window in intervals.windows(2) {
            let (s1, e1, id1, idx1) = window[0].clone();
            let (s2, e2, id2, idx2) = window[1].clone();

            if s2 < e1 {
                let details = PatchSafetyViolationDetails {
                    patch_id: id2.clone(),
                    op_index: idx2,
                    reason: "Patch intervals overlap another patch".to_string(),
                    segment_name: None,
                    rom_start: s2,
                    rom_end: e2,
                    conflicting_patch_id: Some(id1.clone()),
                    conflicting_op_index: Some(idx1),
                };

                messages.push(CheckMessage {
                    code: "PATCH_SAFETY_INTERVAL_OVERLAP".to_string(),
                    message: format!(
                        "Patch op {} (interval [{:#08X}, {:#08X})) overlaps previous op {} for patch {}",
                        idx2, s2, e2, idx1, id1
                    ),
                    file: Some(patch_path.clone()),
                    line: None,
                    column: None,
                    details: serde_json::to_value(details)?,
                });
            }
        }

        let passed = messages.is_empty();

        let result = CheckResult {
            check: CheckCode::PatchSafety,
            passed,
            severity: if passed {
                Severity::Info
            } else {
                Severity::Error
            },
            messages,
        };

        Ok(result)
    }
}
