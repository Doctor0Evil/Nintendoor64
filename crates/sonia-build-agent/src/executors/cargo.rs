use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn run_cargo_check(crate_name: &str, features: &[&str]) -> Vec<BuildEvent> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check")
       .arg("-p")
       .arg(crate_name)
       .arg("--message-format=json-diagnostic-rendered-ansi");
    
    for f in features {
        cmd.arg("--features").arg(f);
    }

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut events = vec![];

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if msg.get("reason") == Some(&serde_json::json!("compiler-message")) {
                    events.push(BuildEvent::Diagnostic { /* parse fields */ });
                }
            }
        }
    }
    events
}
