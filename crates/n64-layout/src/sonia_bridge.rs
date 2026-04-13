// crates/n64-layout/src/sonia_bridge.rs

use serde::{Deserialize, Serialize};

use crate::layout::{RomLayout, Segment, FileEntry};

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

/// Minimal index of payload lengths Sonia has written under `artifacts/`.
#[derive(Debug, Clone)]
pub struct PayloadIndex {
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

/// JSON-friendly per-segment usage report for CI and AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentPatchUsage {
    pub segment_name: String,
    pub segment_kind: String,
    pub rom_offset: u32,
    pub rom_size: u32,
    pub current_bytes: u64,
    pub added_bytes: u64,
    pub max_bytes: u64,
}

/// Summary covering all segments touched by a PatchSpec.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchImpactReport {
    pub layout_id: String,
    pub base_rom_id: String,
    pub total_added_bytes: u64,
    pub per_segment: Vec<SegmentPatchUsage>,
}

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

    /// Validate a PatchSpec and return an impact report if successful.
    /// This:
    /// - checks that logical paths exist
    /// - checks payload sizes fit file lengths
    /// - aggregates per-segment added bytes and maximum capacity
    pub fn validate_patch_spec_with_report(
        &self,
        spec: &PatchSpec,
        payload_index: &PayloadIndex,
    ) -> Result<PatchImpactReport, SoniaBridgeError> {
        use std::collections::HashMap;

        let mut per_segment_added: HashMap<String, u64> = HashMap::new();

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

                    *per_segment_added.entry(file.segment.clone()).or_insert(0) += payload_len;
                }
                PatchEdit::BootHook { .. } => {
                    // Boot hooks typically go into a dedicated segment;
                    // you can wire that here once the layout marks it.
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
                    // Raw intervals are handled by Starzip. For now we do not
                    // attribute added bytes here since size is constrained by max_bytes.
                }
            }
        }

        let mut per_segment_reports = Vec::new();
        let mut total_added_bytes = 0;

        for segment in &self.layout.segments {
            let added = per_segment_added
                .get(&segment.name)
                .copied()
                .unwrap_or(0);

            if added == 0 {
                continue;
            }

            let current_bytes = segment.romsize as u64;
            let max_bytes = segment.romsize as u64; // can be extended later with budget profiles

            total_added_bytes += added;

            per_segment_reports.push(SegmentPatchUsage {
                segment_name: segment.name.clone(),
                segment_kind: format!("{:?}", segment.kind),
                rom_offset: segment.romoffset,
                rom_size: segment.romsize,
                current_bytes,
                added_bytes: added,
                max_bytes,
            });
        }

        Ok(PatchImpactReport {
            layout_id: spec.layout_id.clone(),
            base_rom_id: spec.base_rom_id.clone(),
            total_added_bytes,
            per_segment: per_segment_reports,
        })
    }

    /// Backwards-compatible helper if caller only cares about validity.
    pub fn validate_patch_spec(
        &self,
        spec: &PatchSpec,
        payload_index: &PayloadIndex,
    ) -> Result<(), SoniaBridgeError> {
        let _ = self.validate_patch_spec_with_report(spec, payload_index)?;
        Ok(())
    }
}

/// Convenience to build test artifacts for the tiny N64 slice.
pub fn build_test_slice_artifacts(
    layout: &RomLayout,
    base_rom_id: &str,
    layout_id: &str,
    logical_path: &str,
    payload_filename: &str,
    payload_bytes: &[u8],
) -> Result<(ArtifactSpec, ArtifactSpec, PatchSpec), SoniaBridgeError> {
    let bridge = SoniaBridge::new(layout)?;

    let layout_artifact =
        bridge.layout_to_artifact("artifacts/layouts/n64/test-layout.json");
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
