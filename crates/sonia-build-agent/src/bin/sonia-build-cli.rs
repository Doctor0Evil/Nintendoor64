// crates/sonia-build-agent/src/bin/sonia-build-cli.rs
//! JSON-RPC-ish CLI for sonia-build-agent.
//! Reads BuildIntent JSON lines on stdin, enqueues them, and prints status.

use clap::Parser;
use sonia_build_agent::{BuildDaemon, BuildIntent, IntentPriority};
use std::io::{self, BufRead};
use tracing::info;
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

    println!(
        "{}",
        r#"{"status":"ready","message":"sonia-build-agent listening for intents"}"#
    );

    while let Some(Ok(line)) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<BuildIntent>(trimmed) {
            Ok(mut intent) => {
                if intent.priority as i32 < 0 || intent.priority as i32 > 2 {
                    intent.priority = IntentPriority::Normal;
                }
                info!(id = %intent.id, method = %intent.method, "enqueue intent");
                if let Err(e) = daemon.enqueue(intent) {
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "error": "enqueue_failed",
                            "reason": e
                        })
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    serde_json::json!({
                        "error": "invalid_intent",
                        "reason": e.to_string()
                    })
                );
            }
        }
    }
}
