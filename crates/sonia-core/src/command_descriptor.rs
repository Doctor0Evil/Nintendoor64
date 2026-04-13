use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Execution mode for Sonia commands; extend as SessionProfile grows.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ComputeMode {
    /// Default interactive / local execution.
    LocalInteractive,
    /// CI or batch execution (e.g., GitHub Actions).
    CiBatch,
    /// High-cost or GPU/offload execution.
    HighThroughput,
}

/// Short descriptor for a Sonia CLI command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandDescriptor {
    /// Logical command name, e.g. "createartifact", "validateartifact".
    pub name: String,
    /// Fully-qualified envelope command string, if different from name.
    pub full_command: String,
    /// Short, human-readable explanation (for docs and AI context).
    pub summary: String,
    /// JSON Schema IDs or filenames this command expects as input.
    pub input_schemas: Vec<String>,
    /// JSON Schema IDs or filenames this command promises for outputs.
    pub output_schemas: Vec<String>,
    /// Optional list of invariants this command enforces or is gated by.
    pub invariants: Vec<String>,
    /// Optional list of tags for routing (e.g., ["Nintendoor64","ArtifactSink"]).
    pub tags: Vec<String>,
    /// If non-empty, restricts this command to specific compute modes
    /// (e.g., only allowed in CiBatch).
    pub modes_permitted: Vec<ComputeMode>,
}
