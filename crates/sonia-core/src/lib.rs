use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactType {
    N64RomPatch,
    Ps1IsoPatch,
    LuaScript,
    InputMapperConfig,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSpec {
    pub kind: ArtifactType,
    pub filename: String,
    /// Content as UTF-8 text or ASCII-safe data (hex, base64, etc.).
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoniaResult {
    pub ok: bool,
    pub message: String,
    pub path: Option<String>,
}

pub struct SoniaUploader {
    repo_root: PathBuf,
}

impl SoniaUploader {
    pub fn new<P: AsRef<Path>>(repo_root: P) -> Self {
        Self {
            repo_root: repo_root.as_ref().to_path_buf(),
        }
    }

    pub fn upload(&self, spec: ArtifactSpec) -> Result<SoniaResult> {
        let artifacts_dir = self.repo_root.join("artifacts");
        fs::create_dir_all(&artifacts_dir)?;

        let target_path = artifacts_dir.join(&spec.filename);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // For now, treat content as raw text bytes.
        fs::write(&target_path, spec.content.as_bytes())?;

        Ok(SoniaResult {
            ok: true,
            message: format!("Sonia committed artifact '{}'", spec.filename),
            path: Some(relative_path(&target_path, &self.repo_root)),
        })
    }
}

fn relative_path(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// Read a JSON ArtifactSpec from stdin.
pub fn read_spec_from_stdin() -> Result<ArtifactSpec> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let spec: ArtifactSpec = serde_json::from_str(&buf)?;
    Ok(spec)
}
