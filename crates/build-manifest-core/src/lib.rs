use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Single planned build output under a specific feature set.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlannedOutput {
    pub path: String,          // e.g. "schemas/n64-romlayout.schema.json"
    pub kind: String,          // "SchemaJson", "BinaryBlob", ...
    pub encoding: String,      // "Utf8", "Binary"
    pub features: Vec<String>, // canonical feature set that produces this file
}

/// Manifest for a single build recipe (one CLI, one crate, one mode).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildSchemaManifest {
    pub id: String,                    // e.g. "n64-ai-gen-schemas"
    pub crate_name: String,
    pub command: String,               // "cargo run -p n64-ai-gen-schemas"
    pub default_features: Vec<String>,
    pub all_outputs: Vec<PlannedOutput>,
}
