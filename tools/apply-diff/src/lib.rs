use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub file: String,
    pub start_line: u32,   // 1-based inclusive
    pub end_line: u32,     // 1-based inclusive
    pub expected: Vec<String>,
    pub replacement: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApplyDiffRequest {
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HunkResult {
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub applied: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApplyDiffResult {
    pub status: String, // "ok" / "error"
    pub hunks: Vec<HunkResult>,
}
