// crates/retro-nes-core/src/ines.rs
use crate::constraints::{NesConstraints, NesMapperProfile};

pub fn build_ines_header(constraints: &NesConstraints) -> [u8; 16] {
    let mut header = [0u8; 16];

    // "NES<EOF>"
    header[0] = 0x4E;
    header[1] = 0x45;
    header[2] = 0x53;
    header[3] = 0x1A;

    header[4] = constraints.prg_banks; // PRG size in 16 KiB units
    header[5] = constraints.chr_banks; // CHR size in 8 KiB units

    let mapper = match constraints.mapper {
        NesMapperProfile::Nrom128 => 0,
    };

    // Flags 6: mapper low nibble + mirroring; use horizontal mirroring for now.
    header[6] = (mapper & 0x0F) << 4 | 0x01;
    // Flags 7: mapper high nibble, NES 2.0 bits = 0.
    header[7] = (mapper & 0xF0);

    header
}

// Very small PRG stub placeholder for now.
pub fn build_prg_stub(_constraints: &NesConstraints) -> Vec<u8> {
    // 16 KiB of zeroed PRG; real runtime will replace this.
    vec![0u8; 16 * 1024]
}
