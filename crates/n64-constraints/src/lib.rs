// crates/n64-constraints/src/lib.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// High-level target cartridge profiles for N64.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum N64CartProfile {
    /// 16 MiB cartridge (128 Mbit).
    Size16MiB,
    /// 32 MiB cartridge (256 Mbit).
    Size32MiB,
    /// 64 MiB cartridge (512 Mbit).
    Size64MiB,
    /// Custom capacity specified directly in bytes.
    Custom,
}

/// RDRAM availability profile (base vs Expansion Pak).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum N64RamProfile {
    /// 4 MiB base RDRAM.
    Base4MiB,
    /// 8 MiB with Expansion Pak.
    Expanded8MiB,
    /// Custom total RAM in bytes.
    Custom,
}

/// Texture pixel format for RDP costing and budgeting.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum N64TextureFormat {
    Rgba16,
    Rgba32,
    Ia4,
    Ia8,
    I4,
    I8,
    Ci4,
    Ci8,
}

/// Simple helper for bytes-per-pixel approximation for budget math.
///
/// These are approximate effective storage costs for uncompressed texture data.
/// Compressed or tiled layouts can be handled at the asset-pipeline level.
impl N64TextureFormat {
    pub fn bytes_per_pixel(self) -> f32 {
        match self {
            N64TextureFormat::Rgba16 => 2.0,
            N64TextureFormat::Rgba32 => 4.0,
            N64TextureFormat::Ia4 => 0.5,
            N64TextureFormat::Ia8 => 1.0,
            N64TextureFormat::I4 => 0.5,
            N64TextureFormat::I8 => 1.0,
            N64TextureFormat::Ci4 => 0.5,
            N64TextureFormat::Ci8 => 1.0,
        }
    }
}

/// Constraints for the overall N64 target (ROM + RAM + CPU).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64Constraints {
    /// Cartridge profile.
    pub cart_profile: N64CartProfile,
    /// Total ROM size in bytes (derived from profile, or explicit for Custom).
    pub rom_size_bytes: u32,

    /// RAM profile.
    pub ram_profile: N64RamProfile,
    /// Total RDRAM in bytes (derived from profile, or explicit for Custom).
    pub rdram_bytes: u32,
    /// Bytes reserved for engine/runtime in RDRAM.
    pub engine_reserved_bytes: u32,

    /// Target FPS (e.g., 30 or 60).
    pub target_fps: u32,
    /// Estimated CPU cycles per frame budget.
    pub cpu_cycles_per_frame: u64,

    /// Global texture pool budget in bytes (across all segments).
    pub texture_pool_bytes: u32,
    /// Global audio pool budget in bytes (compressed banks in ROM).
    pub audio_pool_bytes: u32,
    /// Budget for mission/script data in bytes.
    pub script_pool_bytes: u32,
    /// Budget for miscellaneous data (cameras, tables, etc.).
    pub data_pool_bytes: u32,

    /// Optional per-segment ROM budget (segment name -> bytes).
    #[serde(default)]
    pub segment_rom_budgets: HashMap<String, u32>,
}

/// Reasonable defaults for a 32 MiB cart, 8 MiB RAM, 30 FPS target.
impl Default for N64Constraints {
    fn default() -> Self {
        Self::cart32_mib_default()
    }
}

impl N64Constraints {
    /// 32 MiB cartridge, 8 MiB RAM, conservative budgets for a mid-sized game.
    pub fn cart32_mib_default() -> Self {
        let rom_size_bytes = 32 * 1024 * 1024;
        let rdram_bytes = 8 * 1024 * 1024;
        let engine_reserved_bytes = 2 * 1024 * 1024;

        // Simple splits; real projects can override via JSON/TOML.
        let texture_pool_bytes = 10 * 1024 * 1024;
        let audio_pool_bytes = 8 * 1024 * 1024;
        let script_pool_bytes = 2 * 1024 * 1024;
        let data_pool_bytes = 4 * 1024 * 1024;

        Self {
            cart_profile: N64CartProfile::Size32MiB,
            rom_size_bytes,
            ram_profile: N64RamProfile::Expanded8MiB,
            rdram_bytes,
            engine_reserved_bytes,
            target_fps: 30,
            cpu_cycles_per_frame: 80_000_000 / 30, // ~80 MHz / 30 FPS
            texture_pool_bytes,
            audio_pool_bytes,
            script_pool_bytes,
            data_pool_bytes,
            segment_rom_budgets: HashMap::new(),
        }
    }

    /// 16 MiB cartridge, 4 MiB RAM profile.
    pub fn cart16_mib_default() -> Self {
        let rom_size_bytes = 16 * 1024 * 1024;
        let rdram_bytes = 4 * 1024 * 1024;
        let engine_reserved_bytes = 1 * 1024 * 1024;

        let texture_pool_bytes = 5 * 1024 * 1024;
        let audio_pool_bytes = 4 * 1024 * 1024;
        let script_pool_bytes = 1 * 1024 * 1024;
        let data_pool_bytes = 2 * 1024 * 1024;

        Self {
            cart_profile: N64CartProfile::Size16MiB,
            rom_size_bytes,
            ram_profile: N64RamProfile::Base4MiB,
            rdram_bytes,
            engine_reserved_bytes,
            target_fps: 30,
            cpu_cycles_per_frame: 80_000_000 / 30,
            texture_pool_bytes,
            audio_pool_bytes,
            script_pool_bytes,
            data_pool_bytes,
            segment_rom_budgets: HashMap::new(),
        }
    }

    /// 64 MiB cartridge, 8 MiB RAM profile.
    pub fn cart64_mib_default() -> Self {
        let rom_size_bytes = 64 * 1024 * 1024;
        let rdram_bytes = 8 * 1024 * 1024;
        let engine_reserved_bytes = 2 * 1024 * 1024;

        let texture_pool_bytes = 24 * 1024 * 1024;
        let audio_pool_bytes = 16 * 1024 * 1024;
        let script_pool_bytes = 4 * 1024 * 1024;
        let data_pool_bytes = 8 * 1024 * 1024;

        Self {
            cart_profile: N64CartProfile::Size64MiB,
            rom_size_bytes,
            ram_profile: N64RamProfile::Expanded8MiB,
            rdram_bytes,
            engine_reserved_bytes,
            target_fps: 30,
            cpu_cycles_per_frame: 80_000_000 / 30,
            texture_pool_bytes,
            audio_pool_bytes,
            script_pool_bytes,
            data_pool_bytes,
            segment_rom_budgets: HashMap::new(),
        }
    }

    /// Returns usable RAM for runtime assets after engine reservation.
    pub fn runtime_free_bytes(&self) -> u32 {
        self.rdram_bytes.saturating_sub(self.engine_reserved_bytes)
    }
}

/// High-level asset class for budgeting. Starzip can tag each file.
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

/// A single asset entry in the manifest used for budget analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64AssetEntry {
    /// Logical identifier or path for this asset.
    pub id: String,
    /// ROM segment name this asset belongs to (matches RomLayout segments).
    pub segment: String,
    /// Asset class (texture, audio, etc.).
    pub class: N64AssetClass,
    /// Byte size of this asset in the ROM image.
    pub size_bytes: u32,
    /// Optional approximate runtime footprint in RAM bytes.
    #[serde(default)]
    pub runtime_bytes: u32,
    /// Optional approximate CPU cost per frame (cycles).
    #[serde(default)]
    pub cpu_cycles_per_frame: u64,
}

/// Manifest of all assets in a build, for budget analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64AssetManifest {
    /// Optional identifier for this build or recipe.
    #[serde(default)]
    pub build_id: Option<String>,
    /// List of asset entries.
    pub assets: Vec<N64AssetEntry>,
}

/// Per-asset-class usage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassUsage {
    pub used_bytes: u32,
    pub budget_bytes: u32,
    pub over_budget_bytes: i64,
}

/// Per-segment ROM usage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentUsage {
    pub used_bytes: u32,
    pub budget_bytes: u32,
    pub over_budget_bytes: i64,
}

/// Top-level budget report emitted by starzip-cli.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetReport {
    /// Optional build/recipe identifier.
    #[serde(default)]
    pub build_id: Option<String>,

    /// Global ROM usage.
    pub rom_used_bytes: u32,
    pub rom_budget_bytes: u32,
    pub rom_over_budget_bytes: i64,

    /// Aggregate runtime RAM usage (approximate).
    pub runtime_used_bytes: u32,
    pub runtime_budget_bytes: u32,
    pub runtime_over_budget_bytes: i64,

    /// Aggregate CPU cost per frame.
    pub cpu_used_cycles_per_frame: u64,
    pub cpu_budget_cycles_per_frame: u64,
    pub cpu_over_budget_cycles_per_frame: i64,

    /// Usage per asset class.
    pub class_usage: HashMap<N64AssetClass, ClassUsage>,

    /// Usage per segment.
    pub segment_usage: HashMap<String, SegmentUsage>,
}

impl BudgetReport {
    /// Convenience boolean: are all budgets satisfied?
    pub fn is_within_budget(&self) -> bool {
        self.rom_over_budget_bytes <= 0
            && self.runtime_over_budget_bytes <= 0
            && self.cpu_over_budget_cycles_per_frame <= 0
            && self
                .class_usage
                .values()
                .all(|u| u.over_budget_bytes <= 0)
            && self
                .segment_usage
                .values()
                .all(|u| u.over_budget_bytes <= 0)
    }
}

/// Compute a BudgetReport from constraints + manifest.
///
/// This does not know about RomLayout offsets; it only reasons about bytes and
/// cycles per asset class and per segment. Starzip can extend this with deeper
/// layout checks later.
pub fn analyze_budget(
    constraints: &N64Constraints,
    manifest: &N64AssetManifest,
) -> BudgetReport {
    let mut rom_used: u64 = 0;
    let mut runtime_used: u64 = 0;
    let mut cpu_used: u64 = 0;

    let mut class_usage: HashMap<N64AssetClass, (u64, u32)> = HashMap::new();
    let mut segment_usage: HashMap<String, (u64, u32)> = HashMap::new();

    // Initialize class budgets from global pools.
    for class in [
        N64AssetClass::Code,
        N64AssetClass::Texture,
        N64AssetClass::Audio,
        N64AssetClass::Script,
        N64AssetClass::MissionData,
        N64AssetClass::Other,
    ]
    .iter()
    {
        let budget = match class {
            N64AssetClass::Texture => constraints.texture_pool_bytes,
            N64AssetClass::Audio => constraints.audio_pool_bytes,
            N64AssetClass::Script => constraints.script_pool_bytes,
            N64AssetClass::MissionData => constraints.data_pool_bytes,
            _ => 0,
        };
        class_usage.insert(*class, (0u64, budget));
    }

    // Initialize segment budgets from constraints map.
    for (seg, budget) in constraints.segment_rom_budgets.iter() {
        segment_usage.insert(seg.clone(), (0u64, *budget));
    }

    for asset in &manifest.assets {
        rom_used = rom_used.saturating_add(asset.size_bytes as u64);
        runtime_used = runtime_used.saturating_add(asset.runtime_bytes as u64);
        cpu_used = cpu_used.saturating_add(asset.cpu_cycles_per_frame);

        // Class usage.
        let entry = class_usage
            .entry(asset.class)
            .or_insert((0u64, 0u32));
        entry.0 = entry.0.saturating_add(asset.size_bytes as u64);

        // Segment usage.
        let seg_entry = segment_usage
            .entry(asset.segment.clone())
            .or_insert((0u64, 0u32));
        seg_entry.0 = seg_entry.0.saturating_add(asset.size_bytes as u64);
    }

    let rom_used_bytes = rom_used.min(u32::MAX as u64) as u32;
    let runtime_used_bytes = runtime_used.min(u32::MAX as u64) as u32;

    let rom_over = rom_used as i64 - constraints.rom_size_bytes as i64;
    let runtime_budget = constraints.runtime_free_bytes();
    let runtime_over = runtime_used as i64 - runtime_budget as i64;

    let cpu_over =
        cpu_used as i64 - constraints.cpu_cycles_per_frame as i64;

    // Materialize class usage summaries.
    let mut class_usage_out: HashMap<N64AssetClass, ClassUsage> = HashMap::new();
    for (class, (used, budget)) in class_usage.into_iter() {
        let used_bytes = used.min(u32::MAX as u64) as u32;
        let over = used as i64 - budget as i64;
        class_usage_out.insert(
            class,
            ClassUsage {
                used_bytes,
                budget_bytes: budget,
                over_budget_bytes: over,
            },
        );
    }

    // Materialize segment usage summaries.
    let mut segment_usage_out: HashMap<String, SegmentUsage> = HashMap::new();
    for (seg, (used, budget)) in segment_usage.into_iter() {
        let used_bytes = used.min(u32::MAX as u64) as u32;
        let over = used as i64 - budget as i64;
        segment_usage_out.insert(
            seg,
            SegmentUsage {
                used_bytes,
                budget_bytes: budget,
                over_budget_bytes: over,
            },
        );
    }

    BudgetReport {
        build_id: manifest.build_id.clone(),
        rom_used_bytes,
        rom_budget_bytes: constraints.rom_size_bytes,
        rom_over_budget_bytes: rom_over,
        runtime_used_bytes,
        runtime_budget_bytes: runtime_budget,
        runtime_over_budget_bytes: runtime_over,
        cpu_used_cycles_per_frame: cpu_used,
        cpu_budget_cycles_per_frame: constraints.cpu_cycles_per_frame,
        cpu_over_budget_cycles_per_frame: cpu_over,
        class_usage: class_usage_out,
        segment_usage: segment_usage_out,
    }
}
