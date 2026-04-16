// Destination: crates/conker-npc-check/src/scan.rs

use crate::model::{InputPaths, SessionProfile, WorldData};
use anyhow::Result;
use conker_schema::{ConkerMapRecipe, NpcContract};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn load_all(paths: &InputPaths) -> Result<(Vec<(String, ConkerMapRecipe)>, Vec<(String, NpcContract)>, SessionProfile)> {
    let maps = load_maps(&paths.maps_dir)?;
    let npcs = load_npcs(&paths.npcs_dir)?;
    let session = load_session(&paths.session_file)?;
    Ok((maps, npcs, session))
}

fn load_maps(root: &Path) -> Result<Vec<(String, ConkerMapRecipe)>> {
    let mut result = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !is_json(path) {
            continue;
        }
        let text = fs::read_to_string(path)?;
        let map: ConkerMapRecipe = serde_json::from_str(&text)?;
        let id = map.id.clone();
        result.push((id, map));
    }

    Ok(result)
}

fn load_npcs(root: &Path) -> Result<Vec<(String, NpcContract)>> {
    let mut result = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !is_json(path) {
            continue;
        }

        let text = fs::read_to_string(path)?;
        // Optionally validate against JSON Schema before deserializing.
        validate_against_schema_if_present(path, &text)?;

        let npc: NpcContract = serde_json::from_str(&text)?;
        let id = npc.id.clone();
        result.push((id, npc));
    }

    Ok(result)
}

fn load_session(path: &Path) -> Result<SessionProfile> {
    let text = fs::read_to_string(path)?;
    let profile: SessionProfile = serde_json::from_str(&text)?;
    Ok(profile)
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext == "json")
        .unwrap_or(false)
}

/// Optional hook to run JSON Schema validation for NpcContract files.
///
/// If you already run jsonschema-cli in CI, you can stub this out to a no-op.
fn validate_against_schema_if_present(path: &Path, text: &str) -> Result<()> {
    // Example: look for a sibling schema file or a known path.
    // For now, this simply parses to ensure syntactic JSON.
    let _: Value = serde_json::from_str(text)?;
    Ok(())
}
