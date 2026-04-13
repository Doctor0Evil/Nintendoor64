use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContract {
    pub id: String,
    pub version: String,
    pub target: BuildTarget,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default = "default_workspace_root")]
    pub workspace_root: String,
    pub inputs: BuildInputs,
    pub steps: Vec<BuildStep>,
    pub outputs: BuildOutputs,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildTarget {
    Nes,
    Snes,
    N64,
}

fn default_profile() -> String {
    "debug".to_string()
}

fn default_workspace_root() -> String {
    ".".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInputs {
    pub recipe: String,
    pub layout: String,
    #[serde(default = "default_artifact_root")]
    pub artifact_root: String,
    #[serde(default = "default_build_root")]
    pub build_root: String,
}

fn default_artifact_root() -> String {
    "artifacts".to_string()
}

fn default_build_root() -> String {
    "build".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStep {
    pub id: String,
    pub kind: StepKind,
    pub tool: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub inputs: StepInputs,
    pub outputs: StepOutputs,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub env: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    Retro,
    N64,
    Starzip,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepInputs {
    #[serde(default)]
    pub recipe: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub rom: Option<String>,
    #[serde(default)]
    pub patch_spec: Option<String>,
    #[serde(default)]
    pub extra: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepOutputs {
    #[serde(default)]
    pub rom: Option<String>,
    #[serde(default)]
    pub assets_dir: Option<String>,
    #[serde(default)]
    pub logs_dir: Option<String>,
    #[serde(default)]
    pub extra: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutputs {
    pub rom: String,
    #[serde(default)]
    pub artifact_manifest: Option<String>,
}
