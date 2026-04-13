use std::fs;
use std::path::PathBuf;

use n64_ai_checklist::{N64Constraints, PatchSpec, RomLayout};
use schemars::schema_for;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("..")
        .canonicalize()
        .expect("workspace root");

    let schemas_dir = root.join("schemas");
    fs::create_dir_all(&schemas_dir).expect("create schemas dir");

    let layout_schema = schema_for!(RomLayout);
    let constraints_schema = schema_for!(N64Constraints);
    let patch_schema = schema_for!(PatchSpec);

    fs::write(
        schemas_dir.join("n64-romlayout.schema.json"),
        serde_json::to_vec_pretty(&layout_schema).unwrap(),
    )
    .unwrap();

    fs::write(
        schemas_dir.join("n64-constraints.schema.json"),
        serde_json::to_vec_pretty(&constraints_schema).unwrap(),
    )
    .unwrap();

    fs::write(
        schemas_dir.join("n64-patchspec.schema.json"),
        serde_json::to_vec_pretty(&patch_schema).unwrap(),
    )
    .unwrap();
}
