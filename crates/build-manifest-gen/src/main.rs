use std::{fs, path::PathBuf};
use build_manifest_core::{BuildSchemaManifest, PlannedOutput};
use schemars::schema_for;

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");

    let schemas_dir = root.join("schemas");
    fs::create_dir_all(&schemas_dir)?;

    // Emit schema for the manifest itself so AI & CI can validate.
    let schema = schema_for!(BuildSchemaManifest);
    fs::write(
        schemas_dir.join("build-schema-manifest.schema.json"),
        serde_json::to_vec_pretty(&schema)?,
    )?;

    // Seed a concrete manifest for n64-ai-gen-schemas.
    let manifest = BuildSchemaManifest {
        id: "n64-ai-gen-schemas".into(),
        crate_name: "n64-ai-checklist".into(),
        command: "cargo run -p n64-ai-checklist --bin n64-ai-gen-schemas".into(),
        default_features: vec!["default".into()],
        all_outputs: vec![
            PlannedOutput {
                path: "schemas/n64-romlayout.schema.json".into(),
                kind: "SchemaJson".into(),
                encoding: "Utf8".into(),
                features: vec!["default".into()],
            },
            PlannedOutput {
                path: "schemas/n64-constraints.schema.json".into(),
                kind: "SchemaJson".into(),
                encoding: "Utf8".into(),
                features: vec!["default".into()],
            },
            PlannedOutput {
                path: "schemas/n64-patchspec.schema.json".into(),
                kind: "SchemaJson".into(),
                encoding: "Utf8".into(),
                features: vec!["full".into()],
            },
        ],
    };

    let manifest_path = root.join("artifacts/meta/build-manifests/n64-ai-gen-schemas.json");
    fs::create_dir_all(manifest_path.parent().unwrap())?;
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(())
}
