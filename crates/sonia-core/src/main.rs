use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use sonia_core::{read_spec_from_stdin, ArtifactType, SoniaResult, SoniaUploader};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "sonia-core")]
#[command(about = "Sonia: AI-Chat artifact sink for Nintendoor64 and retro pipelines")]
struct Cli {
    /// Override repository root (defaults to current working directory).
    #[arg(global = true, long)]
    repo_root: Option<PathBuf>,

    /// Optional kind override for validation (e.g. N64RomPatch, LuaScript).
    #[arg(global = true, long)]
    kind: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Read a JSON ArtifactSpec from stdin and write it into artifacts/
    UploadFromStdin,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_root = if let Some(root) = cli.repo_root {
        root
    } else {
        env::current_dir()?
    };

    match cli.command {
        Command::UploadFromStdin => {
            let mut spec = read_spec_from_stdin().map_err(|e| {
                anyhow!("Failed to parse ArtifactSpec from stdin: {}", e)
            })?;

            if let Some(kind_str) = &cli.kind {
                spec.kind = parse_kind(kind_str)?;
            }

            validate_extension(&spec)?;

            let uploader = SoniaUploader::new(repo_root);
            let result = uploader.upload(spec)?;

            // Emit a machine-readable JSON result to stdout for AI-Chat / CI.
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
    }

    Ok(())
}

fn parse_kind(s: &str) -> Result<ArtifactType> {
    use ArtifactType::*;
    match s {
        "N64RomPatch" => Ok(N64RomPatch),
        "Ps1IsoPatch" => Ok(Ps1IsoPatch),
        "LuaScript" => Ok(LuaScript),
        "InputMapperConfig" => Ok(InputMapperConfig),
        "Other" => Ok(Other),
        _ => Err(anyhow!("Unknown kind '{}'", s)),
    }
}

fn validate_extension(spec: &sonia_core::ArtifactSpec) -> Result<()> {
    let ext = std::path::Path::new(&spec.filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match spec.kind {
        ArtifactType::LuaScript if ext != "lua" => Err(anyhow!(
            "LuaScript artifacts should end with .lua (got .{})",
            ext
        )),
        ArtifactType::N64RomPatch if !(ext == "ips" || ext == "bps" || ext == "z64") => Err(
            anyhow!(
                "N64RomPatch should be .ips, .bps, or .z64 (got .{})",
                ext
            ),
        ),
        ArtifactType::Ps1IsoPatch if !(ext == "ppf" || ext == "bin" || ext == "iso") => Err(
            anyhow!(
                "Ps1IsoPatch should be .ppf, .bin, or .iso (got .{})",
                ext
            ),
        ),
        _ => Ok(()),
    }
}
