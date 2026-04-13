use jsonschema::JSONSchema;
use serde_json::Value;
use std::{process::Command, fs, path::Path};

fn load_schema(path: &Path) -> anyhow::Result<JSONSchema> {
    let raw = fs::read_to_string(path)?;
    let v: Value = serde_json::from_str(&raw)?;
    Ok(JSONSchema::compile(&v)?)
}

fn main() -> anyhow::Result<()> {
    // 1. Run `sonia-core describe` and capture stdout.
    let output = Command::new("target/debug/sonia-core")
        .arg("describe")
        .output()?;
    if !output.status.success() {
        anyhow::bail!("sonia-core describe failed with status {:?}", output.status);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let v: Value = serde_json::from_str(&stdout)?;

    // 2. Extract `data.commands` from Sonia envelope.
    let commands = v
        .get("data")
        .and_then(|d| d.get("commands"))
        .cloned()
        .unwrap_or(Value::Array(vec![]));

    // 3. Load and apply schema.
    let schema = load_schema(Path::new("schemas/sonia-command-descriptor.schema.json"))?;
    if let Err(errors) = schema.validate(&commands) {
        eprintln!("sonia-core describe output failed schema validation:");
        for err in errors {
            eprintln!("- at {}: {}", err.instance_path, err);
        }
        std::process::exit(1);
    }

    Ok(())
}
