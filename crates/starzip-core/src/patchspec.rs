use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::artifact::ArtifactEncoding;

/// High-level patch specification for N64 ROMs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PatchSpec {
    /// Schema / format version.
    pub version: u32,

    /// Identifier of the base ROM (e.g. SHA-256 of original .z64).
    pub base_rom_id: String,

    /// Identifier or path of the RomLayout this patch targets.
    pub layout_id: String,

    /// Individual edits.
    pub edits: Vec<PatchEdit>,

    /// Optional source file path for diagnostics.
    #[serde(default)]
    pub source_file_path: Option<String>,
}

/// Patch operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PatchEdit {
    /// Replace a logical file by RomLayout FileEntry.path.
    ReplaceFile {
        /// Logical FileEntry.path in RomLayout.
        logical_path: String,

        /// ArtifactSpec.filename of the payload (under artifacts tree).
        payload_ref: String,

        /// Encoding used in the ArtifactSpec content.
        encoding: ArtifactEncoding,
    },

    /// Insert a boot hook at a well-defined ROM offset.
    BootHook {
        /// Hook kind identifier (e.g. "conk64_boot_marker").
        hook_kind: String,

        /// ROM offset where hook is inserted.
        rom_offset: u32,

        /// ArtifactSpec.filename of the hook payload.
        payload_ref: String,

        /// Encoding used in the ArtifactSpec content.
        encoding: ArtifactEncoding,
    },

    /// JSON patch applied to a JSON file inside the ROM FS.
    JsonPatch {
        /// Logical path to JSON file.
        logical_path: String,

        /// RFC 6901 JSON Pointer.
        json_pointer: String,

        /// Value to apply at pointer.
        value: serde_json::Value,
    },

    /// Explicit ROM interval patch (escape hatch, still constrained by layout).
    RawIntervalPatch {
        /// Absolute ROM offset in bytes.
        rom_offset: u32,

        /// Maximum bytes that may be written.
        max_bytes: u32,

        /// ArtifactSpec.filename of the payload.
        payload_ref: String,

        /// Encoding used in the ArtifactSpec content.
        encoding: ArtifactEncoding,
    },
}
