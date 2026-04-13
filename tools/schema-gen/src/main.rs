use schemars::schema_for;
use serde::Serialize;
use std::{fs, path::Path};

use sonia_core::command_descriptor::{CommandDescriptor, ComputeMode};

fn write_schema<T: schemars::JsonSchema + Serialize>(name: &str, out_dir: &Path) -> anyhow::Result<()> {
    let schema = schema_for!(T);
    let json = serde_json::to_string_pretty(&schema)?;
    let path = out_dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let out_dir = Path::new("schemas");

    // Existing schemas:
    // write_schema::<sonia_core::ArtifactSpec>("artifact-spec.schema.json", out_dir)?;
    // write_schema::<sonia_core::SessionProfile>("session.schema.json", out_dir)?;
    // ...

    // New command descriptor schema:
    write_schema::<CommandDescriptor>("sonia-command-descriptor.schema.json", out_dir)?;

    Ok(())
}
