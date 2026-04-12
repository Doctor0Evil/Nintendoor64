use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use sonia_core::{read_spec_from_stdin, SoniaResult, SoniaUploader};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "sonia-core")]
#[command(about = "Sonia: AI-Chat artifact sink for Nintendoor64 and retro pipelines")]
struct Cli {
    /// Override repository root (defaults to current working directory).
    #[arg(global = true, long)]
    repo_root: Option<PathBuf>,

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
            let spec = read_spec_from_stdin().map_err(|e| {
                anyhow!("Failed to parse ArtifactSpec from stdin: {}", e)
            })?;

            let uploader = SoniaUploader::new(repo_root);
            let result = uploader.upload(spec)?;

            // Emit a machine-readable JSON result to stdout for AI-Chat / CI.
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
    }

    Ok(())
}
