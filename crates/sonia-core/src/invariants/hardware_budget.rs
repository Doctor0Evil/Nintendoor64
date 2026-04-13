// Filename: cratessonia-core/src/invariants/hardware_budget.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};
use crate::n64::{constraints::N64Constraints, layout::RomLayout};

/// Summary of over/under usage for a single resource dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDelta {
    pub resource: String,
    pub used_bytes: u64,
    pub limit_bytes: u64,
    /// Positive = over budget, negative = slack.
    pub delta_bytes: i64,
}

/// High-level hardware budget report suitable for AI consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareBudgetDetails {
    pub rom_totals: ResourceDelta,
    pub texture_totals: Option<ResourceDelta>,
    pub audio_totals: Option<ResourceDelta>,
    pub other_resources: Vec<ResourceDelta>,
}

/// Per-constraint slack/overflow classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSlack {
    pub constraint_id: String,
    pub slack_bytes: i64,
    pub over_budget_bytes: u64,
}

/// Budget suggestion entry for AI-facing tradeoff hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSuggestion {
    pub asset_id: String,
    pub change_kind: ChangeKind,
    pub estimated_bytes_saved_rom: u32,
    pub estimated_bytes_saved_texture_pool: u32,
    pub estimated_value_loss: f32,
    /// Convenience ratio Δv / Δs_rom for sorting.
    pub value_loss_per_rom_byte: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeKind {
    DownsampleTexture,
    CompressAudio,
    MergeMissions,
    RemoveOptionalAsset,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum N64AssetClass {
    Code,
    Texture,
    Audio,
    Script,
    MissionData,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64AssetEntry {
    pub id: String,
    pub segment: String,
    pub class: N64AssetClass,
    pub size_bytes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64AssetManifest {
    pub build_id: Option<String>,
    pub assets: Vec<N64AssetEntry>,
}

/// Minimal mirror of your existing N64 constraints and asset manifest types.
/// These should be kept in sync with crates/n64-constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64ConstraintsMirror {
    pub rom_size_bytes: u32,
    pub texture_pool_bytes: u32,
    pub audio_pool_bytes: u32,
    pub script_pool_bytes: u32,
    pub data_pool_bytes: u32,
    pub segment_rom_budgets: HashMap<String, u32>,
}

/// Top-level budget analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareBudgetReport {
    pub build_id: Option<String>,
    pub rom_used_bytes: u32,
    pub rom_budget_bytes: u32,
    pub rom_over_budget_bytes: i64,
    pub per_class_used_bytes: HashMap<N64AssetClass, u32>,
    pub per_class_budget_bytes: HashMap<N64AssetClass, u32>,
    pub per_class_over_budget_bytes: HashMap<N64AssetClass, i64>,
    pub per_segment_used_bytes: HashMap<String, u32>,
    pub per_segment_budget_bytes: HashMap<String, u32>,
    pub per_segment_over_budget_bytes: HashMap<String, i64>,
    pub slacks: Vec<BudgetSlack>,
    pub suggestions: Vec<BudgetSuggestion>,
}

/// Compute hardware budget inequalities and slacks given constraints and manifest.
///
/// Encodes the inequalities:
///   Σ_i s_i^{rom} ≤ C_rom
///   Σ_{i∈Textures} s_i^{vram} ≤ C_tex
///   Σ_{i∈segment j} s_i^{rom} ≤ C_j
pub fn analyze_budget(
    constraints: &N64ConstraintsMirror,
    manifest: &N64AssetManifest,
) -> HardwareBudgetReport {
    let mut rom_used: u64 = 0;
    let mut per_class_used: HashMap<N64AssetClass, u64> = HashMap::new();
    let mut per_segment_used: HashMap<String, u64> = HashMap::new();

    for asset in &manifest.assets {
        rom_used = rom_used.saturating_add(asset.size_bytes as u64);

        let class_entry = per_class_used.entry(asset.class).or_insert(0);
        *class_entry = class_entry.saturating_add(asset.size_bytes as u64);

        let seg_entry = per_segment_used.entry(asset.segment.clone()).or_insert(0);
        *seg_entry = seg_entry.saturating_add(asset.size_bytes as u64);
    }

    let rom_budget = constraints.rom_size_bytes as i64;
    let rom_slack = rom_budget - rom_used as i64;

    let mut slacks = Vec::new();

    slacks.push(BudgetSlack {
        constraint_id: "rom.global".to_string(),
        slack_bytes: rom_slack,
        over_budget_bytes: if rom_slack < 0 { (-rom_slack) as u64 } else { 0 },
    });

    // Per-class budgets (texture/audio/script/data vs. global pools).
    let mut per_class_budget_bytes: HashMap<N64AssetClass, u32> = HashMap::new();
    per_class_budget_bytes.insert(N64AssetClass::Texture, constraints.texture_pool_bytes);
    per_class_budget_bytes.insert(N64AssetClass::Audio, constraints.audio_pool_bytes);
    per_class_budget_bytes.insert(N64AssetClass::Script, constraints.script_pool_bytes);
    per_class_budget_bytes.insert(N64AssetClass::MissionData, constraints.data_pool_bytes);

    let mut per_class_over: HashMap<N64AssetClass, i64> = HashMap::new();

    for (class, used) in &per_class_used {
        if let Some(&budget) = per_class_budget_bytes.get(class) {
            let slack = budget as i64 - *used as i64;
            per_class_over.insert(*class, slack);
            slacks.push(BudgetSlack {
                constraint_id: format!("class.{class:?}"),
                slack_bytes: slack,
                over_budget_bytes: if slack < 0 { (-slack) as u64 } else { 0 },
            });
        }
    }

    // Per-segment budgets.
    let per_segment_budget_bytes: HashMap<String, u32> = constraints.segment_rom_budgets.clone();
    let mut per_segment_over: HashMap<String, i64> = HashMap::new();

    for (segment, used) in &per_segment_used {
        if let Some(&budget) = per_segment_budget_bytes.get(segment) {
            let slack = budget as i64 - *used as i64;
            per_segment_over.insert(segment.clone(), slack);
            slacks.push(BudgetSlack {
                constraint_id: format!("segment.{segment}"),
                slack_bytes: slack,
                over_budget_bytes: if slack < 0 { (-slack) as u64 } else { 0 },
            });
        }
    }

    let suggestions = plan_greedy_tradeoffs(
        constraints,
        manifest,
        &per_class_budget_bytes,
        &per_class_used,
        rom_slack,
    );

    HardwareBudgetReport {
        build_id: manifest.build_id.clone(),
        rom_used_bytes: rom_used.min(u32::MAX as u64) as u32,
        rom_budget_bytes: constraints.rom_size_bytes,
        rom_over_budget_bytes: rom_slack,
        per_class_used_bytes: per_class_used
            .into_iter()
            .map(|(k, v)| (k, v.min(u32::MAX as u64) as u32))
            .collect(),
        per_class_budget_bytes,
        per_class_over_budget_bytes: per_class_over,
        per_segment_used_bytes: per_segment_used
            .into_iter()
            .map(|(k, v)| (k, v.min(u32::MAX as u64) as u32))
            .collect(),
        per_segment_budget_bytes,
        per_segment_over_budget_bytes: per_segment_over,
        slacks,
        suggestions,
    }
}

/// Very simple greedy planner: for each over-budget pool, create candidate
/// tradeoffs and sort by Δv/Δs_rom, as sketched in the knapsack section.
fn plan_greedy_tradeoffs(
    _constraints: &N64ConstraintsMirror,
    manifest: &N64AssetManifest,
    _per_class_budget: &HashMap<N64AssetClass, u32>,
    _per_class_used: &HashMap<N64AssetClass, u64>,
    rom_slack: i64,
) -> Vec<BudgetSuggestion> {
    if rom_slack >= 0 {
        return Vec::new();
    }

    let mut candidates = Vec::new();

    for asset in &manifest.assets {
        let mut change_kind = ChangeKind::RemoveOptionalAsset;
        let mut saved_rom = (asset.size_bytes as f32 * 0.5) as u32;
        let mut saved_tex = 0u32;

        match asset.class {
            N64AssetClass::Texture => {
                change_kind = ChangeKind::DownsampleTexture;
                saved_rom = asset.size_bytes / 2;
                saved_tex = saved_rom;
            }
            N64AssetClass::Audio => {
                change_kind = ChangeKind::CompressAudio;
                saved_rom = (asset.size_bytes as f32 * 0.3) as u32;
            }
            N64AssetClass::MissionData => {
                change_kind = ChangeKind::MergeMissions;
                saved_rom = (asset.size_bytes as f32 * 0.25) as u32;
            }
            _ => {}
        }

        if saved_rom == 0 {
            continue;
        }

        let estimated_value_loss = match asset.class {
            N64AssetClass::Texture => 0.2,
            N64AssetClass::Audio => 0.15,
            N64AssetClass::MissionData => 0.4,
            _ => 0.3,
        };

        let ratio = estimated_value_loss / saved_rom.max(1) as f32;

        candidates.push(BudgetSuggestion {
            asset_id: asset.id.clone(),
            change_kind,
            estimated_bytes_saved_rom: saved_rom,
            estimated_bytes_saved_texture_pool: saved_tex,
            estimated_value_loss,
            value_loss_per_rom_byte: ratio,
        });
    }

    candidates.sort_by(|a, b| {
        a.value_loss_per_rom_byte
            .partial_cmp(&b.value_loss_per_rom_byte)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates
}

pub struct HardwareBudgetChecker;

impl HardwareBudgetChecker {
    pub fn new() -> Self {
        Self
    }

    fn load_layout(path: &str) -> anyhow::Result<RomLayout> {
        let text = std::fs::read_to_string(path)?;
        let layout: RomLayout = serde_json::from_str(&text)?;
        Ok(layout)
    }

    fn load_constraints(path: &str) -> anyhow::Result<N64ConstraintsMirror> {
        let text = std::fs::read_to_string(path)?;
        let constraints: N64ConstraintsMirror = serde_json::from_str(&text)?;
        Ok(constraints)
    }

    fn compute_budget_details(
        layout: &RomLayout,
        constraints: &N64ConstraintsMirror,
    ) -> HardwareBudgetDetails {
        let used_rom = layout.rom_size_bytes() as u64;
        let rom_limit = constraints.rom_size_bytes as u64;
        let rom_delta = used_rom as i64 - rom_limit as i64;

        let rom_totals = ResourceDelta {
            resource: "rom_total".to_string(),
            used_bytes: used_rom,
            limit_bytes: rom_limit,
            delta_bytes: rom_delta,
        };

        let used_textures = layout.total_bytes_for_class("texture");
        let tex_limit = constraints.texture_pool_bytes as u64;
        let tex_delta = used_textures as i64 - tex_limit as i64;
        let texture_totals = Some(ResourceDelta {
            resource: "texture_pool".to_string(),
            used_bytes: used_textures,
            limit_bytes: tex_limit,
            delta_bytes: tex_delta,
        });

        let used_audio = layout.total_bytes_for_class("audio");
        let audio_limit = constraints.audio_pool_bytes as u64;
        let audio_delta = used_audio as i64 - audio_limit as i64;
        let audio_totals = Some(ResourceDelta {
            resource: "audio_pool".to_string(),
            used_bytes: used_audio,
            limit_bytes: audio_limit,
            delta_bytes: audio_delta,
        });

        HardwareBudgetDetails {
            rom_totals,
            texture_totals,
            audio_totals,
            other_resources: Vec::new(),
        }
    }
}

impl Check for HardwareBudgetChecker {
    fn run(&self, input: &crate::aichecklist::ChecklistInput) -> anyhow::Result<CheckResult> {
        let layout_path = match &input.n64_rom_layout_path {
            Some(p) => p,
            None => {
                return Ok(CheckResult {
                    check: CheckCode::HardwareBudget,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "HARDWARE_BUDGET_SKIPPED_NO_LAYOUT".to_string(),
                        message: "Hardware budget check skipped: no n64_rom_layout_path provided"
                            .to_string(),
                        file: None,
                        line: None,
                        column: None,
                        details: serde_json::Value::Null,
                    }],
                });
            }
        };

        let constraints_path = match &input.n64_constraints_path {
            Some(p) => p,
            None => {
                return Ok(CheckResult {
                    check: CheckCode::HardwareBudget,
                    passed: true,
                    severity: Severity::Info,
                    messages: vec![CheckMessage {
                        code: "HARDWARE_BUDGET_SKIPPED_NO_CONSTRAINTS".to_string(),
                        message:
                            "Hardware budget check skipped: no n64_constraints_path provided"
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
        let constraints = Self::load_constraints(constraints_path)?;
        let details = Self::compute_budget_details(&layout, &constraints);

        let mut messages = Vec::new();
        let mut passed = true;

        if details.rom_totals.delta_bytes > 0 {
            passed = false;
            messages.push(CheckMessage {
                code: "HARDWARE_BUDGET_ROM_OVERFLOW".to_string(),
                message: format!(
                    "ROM size exceeds cartridge capacity by {} bytes (used {}, limit {})",
                    details.rom_totals.delta_bytes,
                    details.rom_totals.used_bytes,
                    details.rom_totals.limit_bytes
                ),
                file: Some(layout_path.clone()),
                line: None,
                column: None,
                details: serde_json::to_value(&details)?,
            });
        }

        if let Some(tex) = &details.texture_totals {
            if tex.delta_bytes > 0 {
                passed = false;
                messages.push(CheckMessage {
                    code: "HARDWARE_BUDGET_TEXTURE_OVERFLOW".to_string(),
                    message: format!(
                        "Texture pool exceeds budget by {} bytes (used {}, limit {})",
                        tex.delta_bytes, tex.used_bytes, tex.limit_bytes
                    ),
                    file: Some(layout_path.clone()),
                    line: None,
                    column: None,
                    details: serde_json::to_value(&details)?,
                });
            }
        }

        if let Some(audio) = &details.audio_totals {
            if audio.delta_bytes > 0 {
                passed = false;
                messages.push(CheckMessage {
                    code: "HARDWARE_BUDGET_AUDIO_OVERFLOW".to_string(),
                    message: format!(
                        "Audio pool exceeds budget by {} bytes (used {}, limit {})",
                        audio.delta_bytes, audio.used_bytes, audio.limit_bytes
                    ),
                    file: Some(layout_path.clone()),
                    line: None,
                    column: None,
                    details: serde_json::to_value(&details)?,
                });
            }
        }

        if passed {
            messages.push(CheckMessage {
                code: "HARDWARE_BUDGET_OK".to_string(),
                message: "All hardware budget constraints satisfied".to_string(),
                file: Some(layout_path.clone()),
                line: None,
                column: None,
                details: serde_json::to_value(&details)?,
            });
        }

        Ok(CheckResult {
            check: CheckCode::HardwareBudget,
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
