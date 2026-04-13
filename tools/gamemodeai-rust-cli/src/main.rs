use serde::Deserialize;

// ...

#[derive(Debug, Deserialize)]
#[serde(tag = "reason")]
#[serde(rename_all = "kebab-case")]
enum CargoMessage {
    CompilerMessage { message: CompilerMessageInner },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct CompilerMessageInner {
    code: Option<CompilerMessageCode>,
    level: String,
    message: String,
    spans: Vec<CompilerMessageSpan>,
}

#[derive(Debug, Deserialize)]
struct CompilerMessageCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct CompilerMessageSpan {
    file_name: String,
    line_start: u32,
    column_start: u32,
}

fn run_cargo(
    req: &RustCargoRequest,
) -> Result<(i32, Vec<gamemodeai_rust_core::RustDiagnostic>, Option<String>)> {
    let workspace = PathBuf::from(&req.workspace_root);

    let mut cmd = Command::new("cargo");

    if let Some(toolchain) = &req.toolchain {
        if let Some(ch) = &toolchain.channel {
            cmd.arg(format!("+{}", ch));
        }
    }

    let sub = match req.command {
        CargoCommand::Check => "check",
        CargoCommand::Test => "test",
        CargoCommand::Clippy => "clippy",
        CargoCommand::Build => "build",
        CargoCommand::Doc => "doc",
        CargoCommand::fmt => "fmt",
    };
    cmd.arg(sub);

    if let Some(pkg) = &req.package {
        cmd.arg("--package").arg(pkg);
    }
    if let Some(tgt) = &req.target {
        cmd.arg("--bin").arg(tgt);
    }
    for a in &req.args {
        cmd.arg(a);
    }

    // Always ask cargo for JSON messages.
    cmd.arg("--message-format=json");

    cmd.current_dir(&workspace);
    cmd.envs(&req.env);
    cmd.env("RUSTFLAGS", "-D warnings");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn cargo in {}", workspace.display()))?;

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_string(&mut stdout)?;
    }
    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        err.read_to_string(&mut stderr)?;
    }

    let status = child.wait()?;

    // Write combined log for CI debugging.
    let log_rel = ".sonia/logs/cargo-last.log";
    let log_path = workspace.join(log_rel);
    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut combined = Vec::new();
    combined.extend_from_slice(stdout.as_bytes());
    combined.extend_from_slice(stderr.as_bytes());
    fs::write(&log_path, &combined)
        .with_context(|| format!("failed to write cargo log at {}", log_path.display()))?;

    let diags = summarize_diagnostics_from_json_stream(&stdout);

    let code = status.code().unwrap_or(-1);
    Ok((code, diags, Some(log_rel.to_string())))
}

fn summarize_diagnostics_from_json_stream(
    stdout: &str,
) -> Vec<gamemodeai_rust_core::RustDiagnostic> {
    let mut diags = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parsed: Result<CargoMessage, _> = serde_json::from_str(line);
        let CargoMessage::CompilerMessage { message } = match parsed {
            Ok(m) => m,
            Err(_) => continue,
        };

        let level = message.level;
        // Only propagate errors and warnings for now.
        if level != "error" && level != "warning" {
            continue;
        }

        let code = message.code.map(|c| c.code);
        let primary_span = message.spans.iter().find(|s| {
            // cargo marks primary span in newer formats; fallback is first span.
            true
        });

        let (file, line_no, col) = if let Some(span) = primary_span {
            (
                Some(span.file_name.clone()),
                Some(span.line_start),
                Some(span.column_start),
            )
        } else {
            (None, None, None)
        };

        diags.push(gamemodeai_rust_core::RustDiagnostic {
            level,
            code,
            message: message.message,
            file,
            line: line_no,
            column: col,
            spans: None,
        });
    }

    diags
}
