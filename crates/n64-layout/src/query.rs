// crates/n64-layout/src/query.rs
impl RomLayout {
    pub fn segment_for_rom_offset(&self, addr: u32) -> Option<&Segment> { /* interval search */ }
    pub fn file_for_rom_offset(&self, addr: u32) -> Option<&FileEntry> { /* search within segments */ }
    pub fn vram_for_rom_offset(&self, addr: u32) -> Option<u32> {
        self.segment_for_rom_offset(addr).map(|seg| seg.vram_start + (addr - seg.rom_offset))
    }
}
