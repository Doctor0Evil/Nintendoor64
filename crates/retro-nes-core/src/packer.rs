// crates/retro-nes-core/src/packer.rs
use std::collections::BTreeMap;

use crate::constraints::NesConstraints;

/// 8 KiB CHR bank for NROM-128.
pub struct ChrBank {
    pub bytes: Vec<u8>, // 8192 bytes
}

/// Full 1 KiB nametable: 960 tile indices + 64 attribute bytes.
pub struct NameTable {
    /// First 960 bytes: 32x30 tiles, row-major.
    /// Last 64 bytes: attribute table.
    pub bytes: [u8; 1024],
}

/// Build an 8 KiB CHR bank and a 1 KiB nametable for a single 32x30 map.
///
/// Assumptions:
/// - Target is NROM-128 (mapper 0).
/// - `raw_tileset_chr` contains tiles in NES 2bpp format, 16 bytes per 8x8 tile.
/// - `map_tiles` is exactly 32*30 tile indices into `raw_tileset_chr`.
pub fn build_chr_and_nametable_for_map(
    constraints: &NesConstraints,
    map_tiles: &[u16],
    raw_tileset_chr: &[u8],
) -> Result<(ChrBank, NameTable), String> {
    const NAME_TABLE_WIDTH: usize = 32;
    const NAME_TABLE_HEIGHT: usize = 30;
    const NAME_TABLE_TILES: usize = NAME_TABLE_WIDTH * NAME_TABLE_HEIGHT;
    const TILE_BYTES: usize = 16;
    const NAME_TABLE_BYTES: usize = 960;
    const ATTRIBUTE_BYTES: usize = 64;

    if map_tiles.len() != NAME_TABLE_TILES {
        return Err(format!(
            "Map tiles length mismatch: expected {}, got {}",
            NAME_TABLE_TILES,
            map_tiles.len()
        ));
    }

    if raw_tileset_chr.len() % TILE_BYTES != 0 {
        return Err(format!(
            "raw_tileset_chr length is not a multiple of {} bytes: {}",
            TILE_BYTES,
            raw_tileset_chr.len()
        ));
    }

    let total_tiles_in_source = raw_tileset_chr.len() / TILE_BYTES;

    // 1. Compute unique tile IDs used in this map.
    let mut unique_ids: BTreeMap<u16, usize> = BTreeMap::new();
    for id in map_tiles {
        *unique_ids.entry(*id).or_insert(0) += 1;
    }

    let used_tile_count = unique_ids.len() as u16;
    if used_tile_count > constraints.max_tiles_per_level {
        return Err(format!(
            "Map uses {} unique tiles, exceeds limit {}",
            used_tile_count, constraints.max_tiles_per_level
        ));
    }

    // 2. Enforce CHR bank capacity: N_tiles <= chr_bank_size / 16.
    let chr_bank_size = constraints.chr_bank_size_bytes as usize;
    let tiles_per_bank = chr_bank_size / TILE_BYTES;
    if used_tile_count as usize > tiles_per_bank {
        return Err(format!(
            "Used tiles ({}) exceed CHR bank capacity ({})",
            used_tile_count, tiles_per_bank
        ));
    }

    // 3. Build logical_id -> packed_index mapping with deterministic order.
    let mut logical_to_packed: BTreeMap<u16, u8> = BTreeMap::new();
    let mut next_index: u8 = 0;
    for logical_id in unique_ids.keys() {
        if *logical_id as usize >= total_tiles_in_source {
            return Err(format!(
                "Map references tile id {} but tileset only has {} tiles",
                logical_id, total_tiles_in_source
            ));
        }
        logical_to_packed.insert(*logical_id, next_index);
        next_index = next_index
            .checked_add(1)
            .ok_or_else(|| "Too many tiles to index in u8".to_string())?;
    }

    // 4. Build CHR bank bytes by copying used tiles in packed order.
    let mut chr_bytes = vec![0u8; chr_bank_size];
    for (logical_id, packed_index) in logical_to_packed.iter() {
        let src_offset = *logical_id as usize * TILE_BYTES;
        let dst_offset = *packed_index as usize * TILE_BYTES;
        chr_bytes[dst_offset..dst_offset + TILE_BYTES]
            .copy_from_slice(&raw_tileset_chr[src_offset..src_offset + TILE_BYTES]);
    }

    let chr_bank = ChrBank { bytes: chr_bytes };

    // 5. Build nametable bytes: first 960 tile indices.
    let mut name_table_bytes = [0u8; 1024];

    for (idx, logical_id) in map_tiles.iter().enumerate() {
        let packed = logical_to_packed
            .get(logical_id)
            .ok_or_else(|| format!("Internal error: tile id {} missing from mapping", logical_id))?;
        name_table_bytes[idx] = *packed;
    }

    // 6. Attribute table: last 64 bytes.
    //
    // NES attribute table layout:
    // - Each byte covers a 4x4 tile block (two 2x2 quadrants).
    // - 8 cells horizontally (32 / 4), 8 vertically (32 / 4), so 8*8 = 64 bytes.
    // For this first pass we assign palette 0 everywhere => all attribute bytes = 0.
    let attr_base = NAME_TABLE_BYTES;
    for i in 0..ATTRIBUTE_BYTES {
        name_table_bytes[attr_base + i] = 0;
    }

    let name_table = NameTable {
        bytes: name_table_bytes,
    };

    Ok((chr_bank, name_table))
}
