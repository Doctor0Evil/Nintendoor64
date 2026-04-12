// crates/retro-nes-core/src/lib.rs
mod constraints;
mod ines;
mod packer;

pub use constraints::{NesConstraints, NesMapperProfile};
pub use packer::{build_chr_and_nametable_for_map, ChrBank, NameTable};

/// Compile a single-screen NROM-128 ROM from preprocessed inputs.
///
/// - `map_tiles`: 32x30 tile indices into `raw_tileset_chr`.
/// - `raw_tileset_chr`: NES 2bpp tiles, 16 bytes per tile.
pub fn compile_to_rom(
    constraints: &NesConstraints,
    map_tiles: &[u16],
    raw_tileset_chr: &[u8],
) -> Result<Vec<u8>, String> {
    let (chr_bank, _nametable) =
        build_chr_and_nametable_for_map(constraints, map_tiles, raw_tileset_chr)?;

    let header = ines::build_ines_header(constraints);
    let prg = ines::build_prg_stub(constraints);

    let mut rom = Vec::with_capacity(header.len() + prg.len() + chr_bank.bytes.len());
    rom.extend_from_slice(&header);
    rom.extend_from_slice(&prg);
    rom.extend_from_slice(&chr_bank.bytes);

    Ok(rom)
}
