use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SegmentBudget {
    segment: String,
    max_bytes: u32,
    mutable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct N64Constraints {
    profile_id: String,
    rom_max_bytes: u32,
    rdram_bytes: u32,
    segment_budgets: Vec<SegmentBudget>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Segment {
    name: String,
    rom_offset: u32,
    rom_size: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry {
    path: String,
    segment: String,
    offset_in_segment: u32,
    length: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RomLayout {
    entrypoint: u32,
    rom_size: u32,
    segments: Vec<Segment>,
    files: Vec<FileEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum PatchEdit {
    ReplaceFile {
        logical_path: String,
        payload_ref: String,
        encoding: String,
    },
    BootHook {
        hook_kind: String,
        #[serde(default)]
        params: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
struct PatchSpec {
    version: u32,
    #[allow(dead_code)]
    base_rom_id: String,
    #[allow(dead_code)]
    layout_id: String,
    edits: Vec<PatchEdit>,
}

#[derive(Debug)]
pub struct ChecklistIssue {
    pub id: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ChecklistResult {
    pub ok: bool,
    pub issues: Vec<ChecklistIssue>,
}

pub fn ai_checklist(
    layout: &RomLayout,
    constraints: &N64Constraints,
    patch: &PatchSpec,
) -> ChecklistResult {
    let mut issues = Vec::new();

    if layout.rom_size > constraints.rom_max_bytes {
        issues.push(ChecklistIssue {
            id: "rom.size.exceedsProfile".to_string(),
            message: format!(
                "ROM size {} exceeds profile max {}",
                layout.rom_size, constraints.rom_max_bytes
            ),
        });
    }

    for seg in &layout.segments {
        let Some(budget) = constraints
            .segment_budgets
            .iter()
            .find(|b| b.segment == seg.name)
        else {
            issues.push(ChecklistIssue {
                id: "segment.budget.missing".to_string(),
                message: format!("No budget defined for segment {}", seg.name),
            });
            continue;
        };

        if seg.rom_size > budget.max_bytes {
            issues.push(ChecklistIssue {
                id: "segment.budget.exceeded".to_string(),
                message: format!(
                    "Segment {} size {} exceeds budget {}",
                    seg.name, seg.rom_size, budget.max_bytes
                ),
            });
        }
    }

    for edit in &patch.edits {
        match edit {
            PatchEdit::ReplaceFile { logical_path, .. } => {
                if !layout.files.iter().any(|f| &f.path == logical_path) {
                    issues.push(ChecklistIssue {
                        id: "patch.replaceFile.unknownPath".to_string(),
                        message: format!("Patch targets unknown file {}", logical_path),
                    });
                }
            }
            PatchEdit::BootHook { params, .. } => {
                if let Some(seg_name) = params.get("segment").and_then(|v| v.as_str()) {
                    if let Some(budget) =
                        constraints.segment_budgets.iter().find(|b| b.segment == seg_name)
                    {
                        if !budget.mutable {
                            issues.push(ChecklistIssue {
                                id: "patch.bootHook.immutableSegment".to_string(),
                                message: format!(
                                    "BootHook targets immutable segment {}",
                                    seg_name
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    ChecklistResult {
        ok: issues.is_empty(),
        issues,
    }
}
