// Nintendoor64/crates/sonia-build-agent/src/bin/sonia-build-cli.rs
//! JSON-RPC CLI that accepts build intents from stdin, routes to daemon,
//! and prints structured events to stdout. Designed for AI-Chat tool calling.

use clap::Parser;
use sonia_build_agent::{BuildDaemon, BuildEvent, BuildIntent, IntentPriority};
use std::io::{self, BufRead};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "sonia-build-agent", about = "Terminal-less AI build orchestrator")]
struct Args {
    #[arg(long, default_value = "info")]
    log_level: String,
}

fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.log_level))
        .without_time()
        .init();

    let daemon = BuildDaemon::new();
    let stdin = io::stdin().lock();
    let mut lines = stdin.lines();

    println!("{{\"status\":\"ready\",\"message\":\"sonia-build-agent listening for JSON-RPC intents\"}}");

    while let Some(Ok(line)) = lines.next() {
        if line.trim().is_empty() { continue; }
        match serde_json::from_str::<BuildIntent>(&line) {
            Ok(intent) => {
                daemon.enqueue(intent).unwrap_or_else(|e| {
                    eprintln!("{{\"error\":\"enqueue_failed\",\"reason\":\"{}\"}}", e);
                });
            }
            Err(e) => {
                eprintln!("{{\"error\":\"invalid_intent\",\"reason\":\"{}\"}}", e);
            }
        }
    }
}
