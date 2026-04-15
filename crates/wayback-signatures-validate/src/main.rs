// crates/wayback-signatures-validate/src/main.rs

//! CLI to validate config/wayback/signatures.yaml against
//! schemas/wayback-signatures.schema.json.
//!
//! Intended for use in CI:
//!   cargo run -p wayback-signatures-validate -- \
//!       --schema schemas/wayback-signatures.schema.json \
//!       --config config/wayback/signatures.yaml

use std::fs;
use std::path::PathBuf;

use jsonschema::JSONSchema;
use serde_json::Value;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, StructOpt)]
#[structopt(name = "wayback-signatures-validate")]
struct Cli {
    /// Path to JSON Schema document (for SignatureFile).
    #[structopt(long, parse(from_os_str))]
    schema: PathBuf,

    /// Path to YAML signatures file.
    #[structopt(long, parse(from_os_str))]
    config: PathBuf,
}

#[derive(Debug, Error)]
enum ValidateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("schema validation error: {0}")]
    Schema(String),
}

fn main() -> Result<(), ValidateError> {
    let args = Cli::from_args();

    // Load and parse JSON Schema.
    let schema_str = fs::read_to_string(&args.schema)?;
    let schema_json: Value = serde_json::from_str(&schema_str)?;

    // Compile JSON Schema.
    let compiled = JSONSchema::compile(&schema_json)
        .map_err(|e| ValidateError::Schema(format!("invalid schema: {e}")))?;

    // Load YAML config and convert to JSON Value.
    let yaml_str = fs::read_to_string(&args.config)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;

    // Convert serde_yaml::Value -> serde_json::Value for validation.
    let json_value = yaml_to_json(yaml_value)?;

    // Validate.
    let result = compiled.validate(&json_value);
    if let Err(errors) = result {
        eprintln!("SignatureFile validation failed:");
        for err in errors {
            eprintln!("- at {}: {}", err.instance_path, err);
        }
        return Err(ValidateError::Schema(
            "signatures.yaml failed schema validation".into(),
        ));
    }

    Ok(())
}

/// Convert serde_yaml::Value to serde_json::Value.
/// This keeps the schema/validator JSON-based while letting config live in YAML.
fn yaml_to_json(yaml: serde_yaml::Value) -> Result<Value, ValidateError> {
    Ok(serde_json::from_str(&serde_json::to_string(&yaml)?)?)
}
