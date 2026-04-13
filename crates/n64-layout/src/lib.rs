use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// High-level classification of what a segment contains.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SegmentKind {
    Code,
    Data,
    Texture,
    Audio,
    Map,
    Script,
    Other(String),
}

/// Compression format of a segment or file payload.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Compression {
    None,
    Yaz0,
    MIO0,
    Custom(String),
}

/// Segment on the ROM address line.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub name: String,
    pub kind: SegmentKind,

    /// ROM offset in bytes from start of file.
    pub rom_offset: u32,

    /// Size on ROM in bytes.
    pub rom_size: u32,

    /// VRAM base address where this segment is loaded (if applicable).
    pub vram_start: u32,

    pub compression: Compression,

    /// Whether AI/patch tools may modify data in this segment.
    #[serde(default)]
    pub mutable: bool,
}

/// Logical file/resource within a segment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Logical path, e.g. "maps/level3/geo.bin".
    pub path: String,

    /// Name of owning Segment (Segment.name).
    pub segment: String,

    /// Offset in bytes from the segment's rom_offset.
    pub offset_in_segment: u32,

    /// Length in bytes on ROM.
    pub length: u32,

    /// Free-form content type, e.g. "texture.rgba16", "mips.code", "map.geo".
    pub content_type: String,
}

/// Top-level layout for an N64 ROM.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RomLayout {
    /// ROM entrypoint virtual address.
    pub entrypoint: u32,

    /// Total ROM size in bytes.
    pub rom_size: u32,

    pub segments: Vec<Segment>,
    pub files: Vec<FileEntry>,
}

impl RomLayout {
    /// Helper: total ROM size from segments, for cross-checking rom_size.
    pub fn rom_size_bytes(&self) -> u32 {
        self.rom_size
    }

    /// Find the segment that covers a ROM offset.
    pub fn segment_for_rom_offset(&self, addr: u32) -> Option<&Segment> {
        self.segments.iter().find(|seg| {
            let start = seg.rom_offset;
            let end = seg.rom_offset.saturating_add(seg.rom_size);
            addr >= start && addr < end
        })
    }

    /// Find the file that covers a ROM offset.
    pub fn file_for_rom_offset(&self, addr: u32) -> Option<&FileEntry> {
        self.segment_for_rom_offset(addr).and_then(|seg| {
            self.files.iter().find(|f| {
                if f.segment != seg.name {
                    return false;
                }
                let start = seg.rom_offset + f.offset_in_segment;
                let end = start + f.length;
                addr >= start && addr < end
            })
        })
    }

    /// Compute VRAM address for a ROM offset, if mapped.
    pub fn vram_for_rom_offset(&self, addr: u32) -> Option<u32> {
        self.segment_for_rom_offset(addr).map(|seg| {
            let delta = addr.saturating_sub(seg.rom_offset);
            seg.vram_start + delta
        })
    }

    /// True if [start, end) lies entirely inside some mutable segment.
    pub fn is_interval_in_mutable_segment(&self, start: u32, end: u32) -> bool {
        if end < start {
            return false;
        }
        self.segments.iter().any(|seg| {
            if !seg.mutable {
                return false;
            }
            let s0 = seg.rom_offset;
            let e0 = seg.rom_offset.saturating_add(seg.rom_size);
            start >= s0 && end <= e0
        })
    }

    /// Sum bytes for files whose content_type starts with a given prefix.
    pub fn total_bytes_for_class(&self, class_prefix: &str) -> u64 {
        self.files
            .iter()
            .filter(|f| f.content_type.starts_with(class_prefix))
            .map(|f| f.length as u64)
            .sum()
    }
}
