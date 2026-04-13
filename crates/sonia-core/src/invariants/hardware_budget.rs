use serde::{Deserialize, Serialize};

use super::super::aichecklist::{Check, CheckCode, CheckMessage, CheckResult, Severity};
use crate::n64::{constraints::N64Constraints, layout::RomLayout};

/// Summary of over/under usage for a single resource dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDelta {
    pub resource: String,
    pub used_bytes: u64,
    pub limit_bytes: u64,
    pub delta_bytes: i64, // positive = over budget, negative = slack
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

    fn load_constraints(path: &str) -> anyhow::Result<N64Constraints> {
        let text = std::fs::read_to_string(path)?;
        let constraints: N64Constraints = serde_json::from_str(&text)?;
        Ok(constraints)
    }

    fn compute_budget_details(
        layout: &RomLayout,
        constraints: &N64Constraints,
    ) -> HardwareBudgetDetails {
        // ROM total
        let used_rom = layout.rom_size_bytes() as u64;
        let rom_limit = constraints.cart_capacity_bytes as u64;
        let rom_delta = used_rom as i64 - rom_limit as i64;

        let rom_totals = ResourceDelta {
            resource: "rom_total".to_string(),
            used_bytes: used_rom,
            limit_bytes: rom_limit,
            delta_bytes: rom_delta,
        };

        // Texture pool: sum all assets marked as textures.
        let used_textures = layout.total_bytes_for_class("texture");
        let tex_limit = constraints.texture_pool_bytes as u64;
        let tex_delta = used_textures as i64 - tex_limit as i64;
        let texture_totals = Some(ResourceDelta {
            resource: "texture_pool".to_string(),
            used_bytes: used_textures,
            limit_bytes: tex_limit,
            delta_bytes: tex_delta,
        });

        // Audio pool: similar pattern; can be refined later.
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

        // ROM total
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

        // Textures
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

        // Audio
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

        // Even if everything passes, emit an info message with deltas so AI can plan trade-offs.
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
