// crates/starzip-cli/src/bin/budget.rs

use anyhow::{Context, Result};
use clap::Parser;
use n64_constraints::{analyze_budget, N64AssetManifest, N64Constraints};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;

/// Analyze N64 ROM budgets for a given build manifest and constraints.
///
/// Usage:
///   starzip-budget --constraints constraints.n64.json --manifest assets.n64-manifest.json
#[derive(Debug, Parser)]
#[command(
    name = "starzip-budget",
    about = "N64 hardware-aware budget report for Starzip builds"
)]
struct Args {
    /// Path to N64Constraints JSON file.
    #[arg(long)]
    constraints: PathBuf,

    /// Path to N64AssetManifest JSON file.
    #[arg(long)]
    manifest: PathBuf,

    /// Optional path to write the JSON BudgetReport; defaults to stdout.
    #[arg(long)]
    out: Option<PathBuf>,
}

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let value = serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;
    Ok(value)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let constraints: N64Constraints = read_json(&args.constraints)?;
    let manifest: N64AssetManifest = read_json(&args.manifest)?;

    let report = analyze_budget(&constraints, &manifest);

    let json = serde_json::to_string_pretty(&report)
        .context("Failed to serialize BudgetReport")?;

    if let Some(out_path) = args.out {
        fs::write(&out_path, json)
            .with_context(|| format!("Failed to write {}", out_path.display()))?;
    } else {
        println!("{json}");
    }

    // Non-zero exit if over budget, so CI can gate on this.
    if report.is_within_budget() {
        Ok(())
    } else {
        // Use 2 to distinguish from IO/parse errors (1 from anyhow).
        std::process::exit(2);
    }
}
