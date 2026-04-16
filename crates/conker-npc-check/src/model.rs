// Destination: crates/conker-npc-check/src/model.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use conker_schema::{ConkerMapRecipe, NpcContract};

#[derive(Debug, Clone)]
pub struct InputPaths {
    pub maps_dir: PathBuf,
    pub npcs_dir: PathBuf,
    pub session_file: PathBuf,
}

/// Minimal SessionProfile view for the checks we care about.
///
/// In the full repo you likely already have a richer session schema; this
/// struct can be wired to that to keep everything in sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProfile {
    /// Name of the design contract to enforce (e.g., "conker_n64_v1").
    pub design_contract_id: String,

    /// Whether to enforce strict headshot-only zombies.
    #[serde(default)]
    pub enforce_zombie_headshot_rule: bool,

    /// Whether to enforce pickup-based, symmetric arsenal invariants.
    #[serde(default)]
    pub enforce_pickup_only_arsenal: bool,

    /// Additional flags can be added as the design contract evolves.
}

#[derive(Debug)]
pub struct WorldData {
    pub maps: Vec<(String, ConkerMapRecipe)>,
    pub npc_contracts: Vec<(String, NpcContract)>,
    pub session: SessionProfile,
}

/// Simple aggregate error report.
#[derive(Debug, Default)]
pub struct CheckReport {
    pub errors: Vec<String>,
}

impl CheckReport {
    pub fn push<S: Into<String>>(&mut self, msg: S) {
        self.errors.push(msg.into());
    }
}
