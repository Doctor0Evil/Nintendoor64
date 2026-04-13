// crates/n64-layout/src/sonia_bridge.rs

use serde::{Deserialize, Serialize};

use crate::layout::{RomLayout, Segment, FileEntry};

/// ArtifactType and ArtifactEncoding are defined in cratessonia-core.
/// We re-declare minimal copies here behind a feature flag, or you can
/// depend directly on sonia-core if your workspace allows it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactType {
    N64RomPatch,
    N64Layout,
    N64PatchSpec,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactEncoding {
    Text,
    Hex,
    Base64,
}

impl Default for ArtifactEncoding {
    fn default() -> Self {
        ArtifactEncoding::Text
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSpec {
    pub kind: ArtifactType,
    pub filename: String,
    #[serde(default)]
    pub encoding: ArtifactEncoding,
    pub content: String,
}

/// High-level patch spec mirrored from crates/starzip-core/src/patch.rs.
/// This is the JSON that AI emits and Sonia validates/writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PatchEdit {
    ReplaceFile {
        logical_path: String,
        payload_ref: String,
        #[serde(default)]
        encoding: ArtifactEncoding,
    },
    BootHook {
        hook_kind: String,
        params: serde_json::Value,
    },
    JsonPatch {
        logical_path: String,
        json_pointer: String,
        value: serde_json::Value,
    },
    RawIntervalPatch {
        rom_offset: u32,
        max_bytes: u32,
        payload_ref: String,
        #[serde(default)]
        encoding: ArtifactEncoding,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchSpec {
    pub version: u32,
    pub base_rom_id: String,
    pub layout_id: String,
    pub edits: Vec<PatchEdit>,
}

/// Bridge errors kept human-readable but machine-parseable.
#[derive(Debug, thiserror::Error)]
pub enum SoniaBridgeError {
    #[error("ROM layout has no segments")]
    EmptyLayout,
    #[error("ROM layout has no files")]
    EmptyFiles,
    #[error("segment name not found: {0}")]
    UnknownSegment(String),
    #[error("file logical path not found in layout: {0}")]
    UnknownFile(String),
    #[error("payload reference missing: {0}")]
    MissingPayload(String),
    #[error("payload size {size} exceeds max {max} for file {path}")]
    PayloadTooLarge {
        path: String,
        size: u64,
        max: u64,
    },
    #[error("general bridge error: {0}")]
    Other(String),
}

/// A minimal view of payloads that Sonia has already written under `artifacts/`.
#[derive(Debug, Clone)]
pub struct PayloadIndex {
    /// Map from payload_ref (e.g. "patches/title.bin") to byte length.
    pub lengths: std::collections::HashMap<String, u64>,
}

impl PayloadIndex {
    pub fn new() -> Self {
        Self {
            lengths: std::collections::HashMap::new(),
        }
    }

    pub fn with_entry(mut self, path: impl Into<String>, len: u64) -> Self {
        self.lengths.insert(path.into(), len);
        self
    }

    pub fn get_len(&self, path: &str) -> Option<u64> {
        self.lengths.get(path).copied()
    }
}

/// Bridge helpers for turning layout + high-level patch into safer objects
/// that Sonia and Starzip can agree on.
pub struct SoniaBridge<'a> {
    layout: &'a RomLayout,
}

impl<'a> SoniaBridge<'a> {
    pub fn new(layout: &'a RomLayout) -> Result<Self, SoniaBridgeError> {
        if layout.segments.is_empty() {
            return Err(SoniaBridgeError::EmptyLayout);
        }
        if layout.files.is_empty() {
            return Err(SoniaBridgeError::EmptyFiles);
        }
        Ok(SoniaBridge { layout })
    }

    pub fn layout_to_artifact(&self, layout_path: &str) -> ArtifactSpec {
        let content = serde_json::to_string_pretty(self.layout)
            .unwrap_or_else(|_| "{}".to_string());
        ArtifactSpec {
            kind: ArtifactType::N64Layout,
            filename: layout_path.to_string(),
            encoding: ArtifactEncoding::Text,
            content,
        }
    }

    /// Quick adapter to turn a small test ROM patch binary into an ArtifactSpec.
    /// `filename` is something like "artifacts/patches/n64/test-intro-skip.bin".
    pub fn rom_patch_to_artifact(
        &self,
        filename: &str,
        bytes: &[u8],
    ) -> ArtifactSpec {
        let content = base64::encode(bytes);
        ArtifactSpec {
            kind: ArtifactType::N64RomPatch,
            filename: filename.to_string(),
            encoding: ArtifactEncoding::Base64,
            content,
        }
    }

    /// Validate a PatchSpec against the layout and known payload sizes.
    /// This does not touch ROM bytes; it only checks that:
    /// - logical paths exist in the layout
    /// - payloads referenced actually exist and fit into their targets
    pub fn validate_patch_spec(
        &self,
        spec: &PatchSpec,
        payload_index: &PayloadIndex,
    ) -> Result<(), SoniaBridgeError> {
        for edit in &spec.edits {
            match edit {
                PatchEdit::ReplaceFile {
                    logical_path,
                    payload_ref,
                    ..
                } => {
                    let file = self
                        .layout
                        .files
                        .iter()
                        .find(|f| f.path == *logical_path)
                        .ok_or_else(|| SoniaBridgeError::UnknownFile(logical_path.clone()))?;

                    let payload_len = payload_index
                        .get_len(payload_ref)
                        .ok_or_else(|| SoniaBridgeError::MissingPayload(payload_ref.clone()))?;

                    if payload_len > file.length as u64 {
                        return Err(SoniaBridgeError::PayloadTooLarge {
                            path: logical_path.clone(),
                            size: payload_len,
                            max: file.length as u64,
                        });
                    }
                }
                PatchEdit::BootHook { .. } => {
                    // Layout-based checks for boot hooks can be added here later.
                }
                PatchEdit::JsonPatch { logical_path, .. } => {
                    let _ = self
                        .layout
                        .files
                        .iter()
                        .find(|f| f.path == *logical_path)
                        .ok_or_else(|| SoniaBridgeError::UnknownFile(logical_path.clone()))?;
                }
                PatchEdit::RawIntervalPatch { .. } => {
                    // Raw intervals are checked in Starzip's Safe Patch Synthesizer.
                }
            }
        }

        Ok(())
    }
}

/// Convenience function for local testing: given a `RomLayout` and a single
/// test patch payload, build both ArtifactSpecs and a minimal PatchSpec that
/// replaces one known file.
pub fn build_test_slice_artifacts(
    layout: &RomLayout,
    base_rom_id: &str,
    layout_id: &str,
    logical_path: &str,
    payload_filename: &str,
    payload_bytes: &[u8],
) -> Result<(ArtifactSpec, ArtifactSpec, PatchSpec), SoniaBridgeError> {
    let bridge = SoniaBridge::new(layout)?;

    let layout_artifact = bridge.layout_to_artifact("artifacts/layouts/n64/test-layout.json");
    let patch_payload_artifact =
        bridge.rom_patch_to_artifact(payload_filename, payload_bytes);

    let patch_spec = PatchSpec {
        version: 1,
        base_rom_id: base_rom_id.to_string(),
        layout_id: layout_id.to_string(),
        edits: vec![PatchEdit::ReplaceFile {
            logical_path: logical_path.to_string(),
            payload_ref: payload_filename.to_string(),
            encoding: ArtifactEncoding::Base64,
        }],
    };

    Ok((layout_artifact, patch_payload_artifact, patch_spec))
}
