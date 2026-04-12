// crates/starzip-cli/src/lib.rs
pub struct StarzipPatcher {
    layout: RomLayout,
    rom: Vec<u8>,
}

impl StarzipPatcher {
    pub fn from_rom(rom: Vec<u8>, layout: RomLayout) -> Self { /* ... */ }

    pub fn replace_file(&mut self, path: &str, data: &[u8]) -> anyhow::Result<()> {
        // find FileEntry and overwrite bytes in self.rom safely
        Ok(())
    }

    pub fn inject_boot_hook(&mut self, hook_bytes: &[u8]) -> anyhow::Result<()> {
        let boot_offset = 0x1000; // configurable per template
        self.rom[boot_offset..boot_offset + hook_bytes.len()]
            .copy_from_slice(hook_bytes);
        Ok(())
    }

    pub fn into_rom(self) -> Vec<u8> {
        self.rom
    }
}
