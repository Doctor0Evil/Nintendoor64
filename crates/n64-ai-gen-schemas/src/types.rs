// Nintendoor64/crates/n64-ai-gen-schemas/src/types.rs
//! Core type definitions referenced by schema generation.
//! These are the canonical Rust types that AI-generated artifacts must match.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Classification of artifact content for routing and validation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    /// N64 ROM patch: byte-level modifications to cartridge image
    N64RomPatch,
    /// N64 ROM layout specification: segment definitions and constraints
    N64RomLayout,
    /// NES CHR bank data: 8KB pattern table contents
    NesChrBank,
    /// NES nametable: 1KB background tile map
    NesNametable,
    /// PS1 ISO patch: sector-level modifications to disc image
    Ps1IsoPatch,
    /// Lua gameplay script for mission/AI logic
    LuaScript,
    /// JSON configuration for tuning parameters
    JsonConfig,
    /// Plain text asset (dialogue, documentation, etc.)
    TextAsset,
    /// Opaque binary blob with platform-specific interpretation
    BinaryBlob,
}

/// How payload bytes are represented for JSON transport.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum EncodingMode {
    /// UTF-8 text, no encoding needed
    Text,
    /// Hexadecimal string (2 chars per byte)
    Hex,
    /// Base64 encoding (standard RFC 4648)
    Base64,
    /// Reference to external binary file (payload_ref must be set)
    BinaryRef,
}

/// Canonical contract for any AI-generated artifact in the Sonia pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactSpec {
    /// Unique identifier (UUID format recommended)
    pub id: String,
    /// What kind of content this artifact contains
    pub kind: ArtifactType,
    /// How the payload is encoded for JSON transport
    pub encoding: EncodingMode,
    /// Path reference to actual payload file (relative to artifacts/ root)
    pub payload_ref: String,
    /// Optional contextual metadata for AI conditioning and invariant checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ArtifactMetadata>,
    /// SHA-256 hex digest of payload for integrity verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Contextual metadata attached to artifacts for conditioning.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactMetadata {
    /// Target console platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,
    /// ROM segment or memory region this artifact affects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<String>,
    /// List of invariant rule IDs this artifact has been validated against
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invariants_checked: Option<Vec<String>>,
}

/// Supported console platforms for artifact targeting.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum Platform {
    Nintendo64,
    Nes,
    Snes,
    PlayStation1,
    Multi,
}
