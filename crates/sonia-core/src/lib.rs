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
pub enum ArtifactEncoding {
    Text,
    Hex,
    Base64,
}

impl Default for ArtifactEncoding {
    fn default() -> Self {
        ArtifactEncoding::Text
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSpec {
    pub kind: ArtifactType,
    pub filename: String,
    /// How `content` is encoded. Defaults to Text.
    #[serde(default)]
    pub encoding: ArtifactEncoding,
    /// Content as text, hex string, or base64 depending on `encoding`.
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

        let bytes = decode_content(&spec)?;

        fs::write(&target_path, &bytes)?;

        Ok(SoniaResult {
            ok: true,
            message: format!("Sonia committed artifact '{}'", spec.filename),
            path: Some(relative_path(&target_path, &self.repo_root)),
        })
    }
}

fn decode_content(spec: &ArtifactSpec) -> Result<Vec<u8>> {
    use anyhow::anyhow;
    use ArtifactEncoding::*;

    match spec.encoding {
        Text => Ok(spec.content.clone().into_bytes()),
        Hex => {
            let s = spec.content.trim();
            let s = s.strip_prefix("0x").unwrap_or(s);
            if s.len() % 2 != 0 {
                return Err(anyhow!("Hex content length must be even"));
            }
            let mut out = Vec::with_capacity(s.len() / 2);
            for chunk in s.as_bytes().chunks(2) {
                let hi = (chunk[0] as char)
                    .to_digit(16)
                    .ok_or_else(|| anyhow!("Invalid hex digit"))?;
                let lo = (chunk[1] as char)
                    .to_digit(16)
                    .ok_or_else(|| anyhow!("Invalid hex digit"))?;
                out.push(((hi << 4) | lo) as u8);
            }
            Ok(out)
        }
        Base64 => {
            let decoded = base64::decode(&spec.content)?;
            Ok(decoded)
        }
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
