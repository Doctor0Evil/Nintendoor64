use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Top-level container for the feature index.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FeatureLayout")]
pub struct FeatureLayout {
    /// Repository this index describes.
    pub repo: String,
    
    /// Schema version for forward compatibility.
    pub version: String,
    
    /// List of discoverable features.
    pub features: Vec<FeatureEntry>,
}

/// A single feature that AI can discover and use.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FeatureEntry")]
pub struct FeatureEntry {
    /// Globally unique identifier, dot-separated path style.
    /// Example: "nintendoor64.starzip.patch-synthesizer"
    pub id: String,
    
    /// Human-readable title for display.
    pub title: String,
    
    /// Detailed description of what this feature does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Controlled vocabulary tags for semantic queries.
    pub tags: Vec<FeatureTag>,
    
    /// SystemNode IDs from the knowledge graph that implement this feature.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems: Vec<String>,
    
    /// Paths to JSON Schema files that define contracts for this feature.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schemas: Vec<String>,
    
    /// Paths to example JSON/TOML files demonstrating usage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    
    /// Recommended CLI invocations for common tasks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
    
    /// High-level capability labels for role-based discovery.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<FeatureRole>,
    
    /// Session tags: only show this feature when these session conditions hold.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_tags: Vec<SessionTag>,
}

/// Controlled vocabulary for feature tagging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FeatureTag")]
pub enum FeatureTag {
    // Platform tags
    Nintendoor64,
    Starzip,
    Sonia,
    Conk64,
    BondFPS,
    RetroNES,
    RetroSNES,
    RetroN64,
    RetroPS1,
    
    // Capability tags
    Deterministic,
    PatchSynthesizer,
    ScenarioDirector,
    NarrativeCartographer,
    BudgetPlanner,
    SchemaDesigner,
    BinarySafe,
    LuaFacing,
    
    // Genre tags
    ArenaShooter,
    FantasyFPS,
    ThirdPersonShooter,
    DestructionBrawler,
    Platformer2D,
    
    // Integration tags
    KnowledgeGraph,
    CIIntegrated,
    AIFacing,
}

/// High-level roles for coarse-grained discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FeatureRole")]
pub enum FeatureRole {
    /// Generates or modifies binary ROM/ISO patches
    PatchSynthesizer,
    /// Defines mission objectives and trigger logic
    ScenarioDirector,
    /// Manages branching narrative structures
    NarrativeCartographer,
    /// Analyzes and suggests resource optimizations
    BudgetPlanner,
    /// Defines and validates JSON Schema contracts
    SchemaDesigner,
    /// Provides AI navigation over repo structure
    FeatureNavigator,
    /// Manages artifact lifecycle and validation
    ArtifactManager,
}

/// Conditions under which a feature should be visible to AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "SessionTag")]
pub enum SessionTag {
    /// Feature requires N64 toolchain to be active
    RequiresN64,
    /// Feature requires PS1 toolchain to be active
    RequiresPS1,
    /// Feature only valid when determinism invariant is enabled
    RequiresDeterminism,
    /// Feature requires retro-backend feature flag
    RequiresRetroBackend,
    /// Feature is experimental and requires opt-in
    Experimental,
}
