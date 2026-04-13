use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "gamemodeai-session", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    GetSession {
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    UpdateSession {
        #[arg(long)]
        patch: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    UpdateCiStatus {
        #[arg(long)]
        digest: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
    /// New: run a cargo job and record its result.
    RunCargo {
        /// JSON RustCargoRequest file (without computeMode / workspaceRoot).
        #[arg(long)]
        job: PathBuf,
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SoniaEnvelope<T> {
    version: u32,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<SoniaError>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SoniaError {
    code: String,
    message: String,
}
