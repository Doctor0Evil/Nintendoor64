// crates/n64-layout/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    pub rom_offset: u32,
    pub rom_size: u32,
    pub vram_start: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub segment: String,
    pub offset_in_segment: u32,
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomLayout {
    pub entry_point: u32,
    pub segments: Vec<Segment>,
    pub files: Vec<FileEntry>,
}
