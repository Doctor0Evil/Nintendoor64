// crates/retro-nes-core/src/constraints.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NesMapperProfile {
    Nrom128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NesConstraints {
    pub mapper: NesMapperProfile,
    pub prg_banks: u8,
    pub chr_banks: u8,
    pub chr_bank_size_bytes: u16,   // 8192 for NROM
    pub max_tiles_per_level: u16,   // VRAM / 16 bytes per tile
    pub max_sprites_total: u8,
    pub max_sprites_per_scanline: u8,
    pub palettes_per_bg: u8,
    pub palettes_per_sprite: u8,
    pub colors_per_palette: u8,
    pub controller_profile_id: String,
}

impl NesConstraints {
    pub fn nrom128_default() -> Self {
        Self {
            mapper: NesMapperProfile::Nrom128,
            prg_banks: 1,
            chr_banks: 1,
            chr_bank_size_bytes: 8 * 1024,
            max_tiles_per_level: 256,
            max_sprites_total: 64,
            max_sprites_per_scanline: 8,
            palettes_per_bg: 4,
            palettes_per_sprite: 4,
            colors_per_palette: 4,
            controller_profile_id: "profile.nes.standard".into(),
        }
    }
}
