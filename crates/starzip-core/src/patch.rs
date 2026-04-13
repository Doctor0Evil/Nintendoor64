// crates/starzip-core/src/patch.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchOpKind {
    ReplaceFile { path: String },
    InsertBootHook { rom_offset: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchOp {
    pub op: PatchOpKind,
    pub payload_ref: String, // path under artifacts/, e.g. "patches/stealth/level3.bin"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSpec {
    pub id: String,
    pub base_rom: String,
    pub layout_path: String,
    pub ops: Vec<PatchOp>,
}
