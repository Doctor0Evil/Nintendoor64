use serde::{Deserialize, Serialize};

/// High-level preview commands that Starzip exposes to Sonia / AIChat.
///
/// This enum is intentionally BinarySafe: it only carries logical identifiers
/// and JSON file paths, never raw ROM bytes or patches.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "command")]
pub enum PreviewCommand {
    /// Read-only query against a ROM layout description.
    ///
    /// `rom_id` is a logical identifier (e.g. "conker-uncut-dev"),
    /// and `layout_path` is a repository-relative JSON file describing
    /// segments, files, and VRAM mappings.
    RomQuery {
        rom_id: String,
        layout_path: String,
    },

    /// Safe, non-mutating patch preview. This never emits raw ROM bytes.
    ///
    /// `base_rom_id` is a logical identifier, `layout_path` is the layout
    /// JSON, and `patch_path` is a PatchSpec JSON. The optional flags
    /// control how much work is done:
    ///
    /// - `dry_run`: if true, compute the impact report without running
    ///   the full patch pipeline or touching any ROM files.
    /// - `budget_only`: if true, restrict the computation to budget
    ///   deltas (e.g. per-segment size changes) and skip more expensive
    ///   structural checks. When both flags are false, a full preview
    ///   is performed.
    PatchPreview {
        base_rom_id: String,
        layout_path: String,
        patch_path: String,
        #[serde(default)]
        dry_run: bool,
        #[serde(default)]
        budget_only: bool,
    },
}

/// Unified envelope for all preview responses.
///
/// This is designed to mirror the wider Sonia protocol: a `version` for
/// forward compatibility, a `status` discriminator, and either a `data`
/// payload or an `error` object, but never both at the same time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewEnvelope {
    /// Protocol version. Start at 1; bump only on breaking changes.
    pub version: u32,

    /// `"ok"` on success, `"error"` on failure.
    pub status: String,

    /// Success payload. For `RomQuery` this will typically be a
    /// RomLayoutPreview JSON; for `PatchPreview` it will be a
    /// PatchImpactPreview JSON. This field is absent on error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Error payload. Present only when `status == "error"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<PreviewError>,
}

/// Machine-readable error details for preview operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewError {
    /// Stable, machine-readable error code, e.g.:
    /// "InvalidRequest", "LayoutNotFound", "PatchNotFound",
    /// "PreviewComputationFailed".
    pub code: String,

    /// Human-readable, log-friendly message.
    pub message: String,

    /// Optional structured diagnostics payload. This can hold
    /// additional, schema-conformant objects such as a list of
    /// invalid segments or budget violations, but must never
    /// include raw ROM bytes or large unstructured text logs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl PreviewEnvelope {
    /// Convenience constructor for an `"ok"` envelope with a JSON payload.
    pub fn ok(version: u32, data: serde_json::Value) -> Self {
        PreviewEnvelope {
            version,
            status: "ok".to_string(),
            data: Some(data),
            error: None,
        }
    }

    /// Convenience constructor for an `"error"` envelope.
    ///
    /// Prefer passing a stable `code` that can be used by AI / CI
    /// for branching, and a concise `message` suitable for logs and
    /// human debugging.
    pub fn err(version: u32, code: impl Into<String>, message: impl Into<String>) -> Self {
        PreviewEnvelope {
            version,
            status: "error".to_string(),
            data: None,
            error: Some(PreviewError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
        }
    }

    /// Attach a structured `details` payload to an existing error envelope.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        if let Some(ref mut err) = self.error {
            err.details = Some(details);
        }
        self
    }
}
