use std::fs;
use std::path::PathBuf;

use clap::Parser;
use jsonschema::JSONSchema;
use serde_json::Value;

#[derive(Debug, Parser)]
struct Args {
    /// Path to the JSON Schema file
    #[arg(long)]
    schema: PathBuf,
    /// One or more JSON instance files to validate
    #[arg(long)]
    instances: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let schema_str = fs::read_to_string(&args.schema)
        .expect("read schema");
    let schema_json: Value = serde_json::from_str(&schema_str)
        .expect("parse schema");

    let compiled = JSONSchema::compile(&schema_json)?;

    let mut had_error = false;

    for instance_path in &args.instances {
        let text = fs::read_to_string(instance_path)
            .unwrap_or_else(|_| panic!("read {:?}", instance_path));
        let instance: Value = serde_json::from_str(&text)
            .unwrap_or_else(|_| panic!("parse {:?}", instance_path));

        if let Err(errors) = compiled.validate(&instance) {
            had_error = true;
            eprintln!("Validation failed for {:?}:", instance_path);
            for e in errors {
                eprintln!("- {}", e);
            }
        } else {
            println!("OK {:?}", instance_path);
        }
    }

    if had_error {
        std::process::exit(1);
    }

    Ok(())
}
