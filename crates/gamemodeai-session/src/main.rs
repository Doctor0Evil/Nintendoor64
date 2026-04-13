mod cli;

use anyhow::{Context, Result};
use cli::{Cli, Command, SoniaEnvelope, SoniaError};
use clap::Parser;
use gamemodeai_rust_core::{ComputeMode, RustCargoRequest, RustCargoResponse};
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command as PCommand};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::GetSession { repo_root } => {
            // existing behavior...
            todo!()
        }
        Command::UpdateSession { patch, repo_root } => {
            // existing behavior...
            todo!()
        }
        Command::UpdateCiStatus { digest, repo_root } => {
            // existing behavior...
            todo!()
        }
        Command::RunCargo { job, repo_root } => {
            handle_run_cargo(job, repo_root)?;
        }
    }

    Ok(())
}

fn handle_run_cargo(job_path: PathBuf, repo_root: PathBuf) -> Result<()> {
    let raw = fs::read_to_string(&job_path)
        .with_context(|| format!("failed to read cargo job at {}", job_path.display()))?;
    let mut req: RustCargoRequest = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse RustCargoRequest from {}", job_path.display()))?;

    // Fill in workspace_root and compute_mode from environment / session.
    req.workspace_root = repo_root.to_string_lossy().to_string();
    if matches!(req.compute_mode, ComputeMode::BrowserWasm) {
        // Session tool itself should never run in BrowserWasm; override to CiRunner by default.
        req.compute_mode = ComputeMode::CiRunner;
    }

    // Invoke gamemodeai-rust-cli via JSON stdin/stdout.
    let mut child = PCommand::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("gamemodeai-rust-cli")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn gamemodeai-rust-cli via cargo run")?;

    {
        let stdin = child.stdin.as_mut().expect("stdin available");
        stdin.write_all(raw_request_with_overrides(&req)?.as_bytes())?;
    }

    let output = child
        .wait_with_output()
        .context("failed to run gamemodeai-rust-cli")?;
    if !output.status.success() {
        let env = SoniaEnvelope::<Value> {
            version: 1,
            status: "error".to_string(),
            data: None,
            error: Some(SoniaError {
                code: "CargoRunnerFailed".to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            }),
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(1);
    }

    let resp_val: Value = serde_json::from_slice(&output.stdout)?;
    // Expect same envelope {status,data:{..RustCargoResponse..}} as gamemodeai-rust-cli prints.
    let data = resp_val
        .get("data")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("gamemodeai-rust-cli response missing data field"))?;
    let resp: RustCargoResponse = serde_json::from_value(data)?;

    // TODO: optionally fold summary into SessionProfile.ciStatus here.

    let env = SoniaEnvelope {
        version: 1,
        status: "ok".to_string(),
        data: Some(resp),
        error: None,
    };
    println!("{}", serde_json::to_string_pretty(&env)?);

    Ok(())
}

fn raw_request_with_overrides(req: &RustCargoRequest) -> Result<String> {
    let v = serde_json::to_value(req)?;
    Ok(serde_json::to_string_pretty(&v)?)
}
