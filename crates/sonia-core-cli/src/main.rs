fn cmd_list_kind(kind: Option<String>, repo_root: &Path) -> Result<()> {
    let mut artifacts = Vec::new();
    let root = repo_root.join("artifacts");

    if !root.exists() {
        let env = SoniaOkEnvelope {
            ok: true,
            path: None,
            bytes_written: None,
            artifacts: Some(serde_json::json!([])),
            session: None,
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }

    let walker = match walkdir::WalkDir::new(&root).into_iter().collect::<Result<Vec<_>, _>>() {
        Ok(entries) => entries,
        Err(e) => {
            let env = SoniaErrorEnvelope {
                ok: false,
                error_code: SoniaErrorCode::ListFailed,
                message: format!("failed to walk artifacts directory: {}", e),
                details: Some(serde_json::json!({
                    "root": root.to_string_lossy()
                })),
            };
            println!("{}", serde_json::to_string_pretty(&env)?);
            std::process::exit(SoniaErrorCode::ListFailed.exit_code());
        }
    };

    for entry in walker {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = match path.strip_prefix(repo_root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if let Some(ref k) = kind {
            if !rel.contains(k) {
                continue;
            }
        }
        artifacts.push(serde_json::json!({ "filename": rel }));
    }

    let env = SoniaOkEnvelope {
        ok: true,
        path: None,
        bytes_written: None,
        artifacts: Some(serde_json::Value::Array(artifacts)),
        session: None,
    };
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
}

fn cmd_get_session(repo_root: &Path) -> Result<()> {
    let path = session_path(repo_root);

    if !path.exists() {
        let profile = SessionProfile {
            repo: repo_root
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            branch: std::env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            activecrate: None,
            featureflags: Vec::new(),
            invariants: Vec::new(),
            cistatus: sonia_core::CiStatusReport {
                lastrunid: None,
                lastresult: Some(sonia_core::CiResult::Unknown),
                failingcrates: Vec::new(),
                failingsystems: Vec::new(),
            },
        };

        let env = SoniaOkEnvelope {
            ok: true,
            path: Some(path.to_string_lossy().to_string()),
            bytes_written: None,
            artifacts: None,
            session: Some(serde_json::to_value(profile)?),
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }

    let text = fs::read_to_string(&path).map_err(|e| {
        anyhow::anyhow!("failed to read session profile {}: {}", path.display(), e)
    })?;
    let profile: SessionProfile = serde_json::from_str(&text)?;

    let env = SoniaOkEnvelope {
        ok: true,
        path: Some(path.to_string_lossy().to_string()),
        bytes_written: None,
        artifacts: None,
        session: Some(serde_json::to_value(profile)?),
    };
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
}

fn cmd_update_session(profile_path: &Path, repo_root: &Path) -> Result<()> {
    let text = fs::read_to_string(profile_path).map_err(|e| {
        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: SoniaErrorCode::GenericInvalidInput,
            message: format!(
                "failed to read session profile patch {}: {}",
                profile_path.display(),
                e
            ),
            details: None,
        };
        println!("{}", serde_json::to_string_pretty(&env).unwrap());
        std::process::exit(SoniaErrorCode::GenericInvalidInput.exit_code());
    })?;

    let profile: SessionProfile = serde_json::from_str(&text).map_err(|e| {
        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: SoniaErrorCode::GenericInvalidInput,
            message: format!("invalid SessionProfile JSON: {}", e),
            details: None,
        };
        println!("{}", serde_json::to_string_pretty(&env).unwrap());
        std::process::exit(SoniaErrorCode::GenericInvalidInput.exit_code());
    })?;

    let path = session_path(repo_root);
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            let env = SoniaErrorEnvelope {
                ok: false,
                error_code: SoniaErrorCode::SessionWriteFailed,
                message: format!(
                    "failed to create session directory {}: {}",
                    parent.display(),
                    e
                ),
                details: None,
            };
            println!("{}", serde_json::to_string_pretty(&env).unwrap());
            std::process::exit(SoniaErrorCode::SessionWriteFailed.exit_code());
        }
    }

    if let Err(e) = fs::write(&path, serde_json::to_string_pretty(&profile)?) {
        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: SoniaErrorCode::SessionWriteFailed,
            message: format!("failed to write session profile {}: {}", path.display(), e),
            details: None,
        };
        println!("{}", serde_json::to_string_pretty(&env).unwrap());
        std::process::exit(SoniaErrorCode::SessionWriteFailed.exit_code());
    }

    let env = SoniaOkEnvelope {
        ok: true,
        path: Some(path.to_string_lossy().to_string()),
        bytes_written: None,
        artifacts: None,
        session: Some(serde_json::to_value(profile)?),
    };
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
}
