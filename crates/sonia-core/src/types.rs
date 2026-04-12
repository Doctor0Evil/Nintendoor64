// crates/sonia-core/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ArtifactType {
    N64RomPatch,
    Ps1IsoPatch,
    LuaScript,
    InputMapperConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactSpec {
    pub kind: ArtifactType,
    pub filename: String,
    /// UTF-8 text or base64-encoded binary depending on kind.
    pub content: String,
}
