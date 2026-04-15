// crates/wayback-detect/src/signatures.rs

use std::fs;
use std::path::{Path, PathBuf};

use hex::FromHex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Logical engine identifier (GameSpy, IPX, Quake3, GoldSrc, etc.).
pub type EngineId = String;

/// A single detection signature loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: String,
    pub engine: EngineId,
    pub protocol: String, // "udp" or "tcp"
    pub priority: i32,
    pub kind: SignatureKind,
    pub offset: usize,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub hex: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,

    // Computed field: decoded bytes for hex or pattern, populated at load time.
    #[serde(skip)]
    compiled_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureKind {
    /// Check for a UTF-8 substring at or after offset.
    Substring,
    /// Check that payload[offset..] starts with compiled_bytes.
    PrefixBytes,
}

/// Top-level YAML file format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureFile {
    pub version: u32,
    pub signatures: Vec<Signature>,
}

/// Errors from the detection system.
#[derive(Debug, Error)]
pub enum DetectError {
    #[error("IO error while reading signatures: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("invalid signature {id}: {msg}")]
    InvalidSignature { id: String, msg: String },
}

/// Result of a detection attempt.
#[derive(Debug, Clone)]
pub struct DetectResult {
    pub signature_id: String,
    pub engine: EngineId,
    pub protocol: String,
    pub tags: Vec<String>,
    pub priority: i32,
}

/// Detector holds a compiled list of signatures and can classify payloads.
#[derive(Debug, Clone)]
pub struct Detector {
    signatures: Vec<Signature>,
    source_path: PathBuf,
}

impl Detector {
    /// Load and compile signatures from a YAML file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, DetectError> {
        let path = path.as_ref().to_path_buf();
        let raw = fs::read_to_string(&path)?;
        let mut file: SignatureFile = serde_yaml::from_str(&raw)?;

        // Compile pattern/hex into bytes.
        for sig in &mut file.signatures {
            let compiled = compile_signature_bytes(sig).map_err(|msg| DetectError::InvalidSignature {
                id: sig.id.clone(),
                msg,
            })?;
            sig.compiled_bytes = compiled;
        }

        // Sort by descending priority so highest-priority matches first.
        file.signatures.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(Self {
            signatures: file.signatures,
            source_path: path,
        })
    }

    /// Detect the engine for a given protocol and payload.
    ///
    /// Returns the first (highest-priority) matching signature, if any.
    pub fn detect(&self, protocol: &str, payload: &[u8]) -> Option<DetectResult> {
        for sig in &self.signatures {
            if sig.protocol != protocol {
                continue;
            }

            if matches_signature(sig, payload) {
                return Some(DetectResult {
                    signature_id: sig.id.clone(),
                    engine: sig.engine.clone(),
                    protocol: sig.protocol.clone(),
                    tags: sig.tags.clone(),
                    priority: sig.priority,
                });
            }
        }

        None
    }

    /// Return the path the detector loaded its signatures from.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
}

/// Turn `pattern` or `hex` into a compiled byte sequence.
///
/// Rules:
///   - If `hex` is present, decode it as lowercase/uppercase hex.
///   - Else if `pattern` is present, use its UTF-8 bytes.
///   - Else error.
fn compile_signature_bytes(sig: &Signature) -> Result<Vec<u8>, String> {
    if let Some(ref hex_str) = sig.hex {
        // Strip whitespace to be forgiving in YAML.
        let cleaned: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
        let bytes = Vec::from_hex(cleaned).map_err(|e| format!("invalid hex: {e}"))?;
        if bytes.is_empty() {
            return Err("hex pattern must not be empty".into());
        }
        Ok(bytes)
    } else if let Some(ref pat) = sig.pattern {
        if pat.is_empty() {
            return Err("pattern must not be empty".into());
        }
        Ok(pat.as_bytes().to_vec())
    } else {
        Err("signature must define either hex or pattern".into())
    }
}

/// Check whether a signature matches the given payload.
fn matches_signature(sig: &Signature, payload: &[u8]) -> bool {
    match sig.kind {
        SignatureKind::Substring => {
            if sig.compiled_bytes.is_empty() || sig.offset >= payload.len() {
                return false;
            }
            let haystack = &payload[sig.offset..];
            // For substring, compiled_bytes represent UTF-8, but we don't
            // assume payload is valid UTF-8. We just search raw bytes.
            find_subsequence(haystack, &sig.compiled_bytes).is_some()
        }
        SignatureKind::PrefixBytes => {
            let needed_len = sig.offset + sig.compiled_bytes.len();
            if needed_len > payload.len() {
                return false;
            }
            let slice = &payload[sig.offset..needed_len];
            slice == sig.compiled_bytes.as_slice()
        }
    }
}

/// Return Some(start_index) if `needle` appears in `haystack`, else None.
///
/// This simple O(n*m) scan is fine for short signatures. If needed,
/// you can replace this with a more advanced algorithm later.
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_bytes_match() {
        let sig = Signature {
            id: "test".into(),
            engine: "TestEngine".into(),
            protocol: "udp".into(),
            priority: 10,
            kind: SignatureKind::PrefixBytes,
            offset: 0,
            pattern: None,
            hex: Some("ffff".into()),
            tags: vec![],
            compiled_bytes: vec![0xff, 0xff],
        };

        assert!(matches_signature(&sig, &[0xff, 0xff, 0x01]));
        assert!(!matches_signature(&sig, &[0x00, 0xff, 0xff]));
    }

    #[test]
    fn test_substring_match() {
        let mut sig = Signature {
            id: "gamespy-heartbeat".into(),
            engine: "GameSpy".into(),
            protocol: "udp".into(),
            priority: 100,
            kind: SignatureKind::Substring,
            offset: 0,
            pattern: Some("\\heartbeat\\".into()),
            hex: None,
            tags: vec![],
            compiled_bytes: vec![],
        };
        sig.compiled_bytes = compile_signature_bytes(&sig).unwrap();

        let payload = b"\\heartbeat\\6556\\gamename\\jbnightfire\\final";
        assert!(matches_signature(&sig, payload));
    }

    #[test]
    fn test_detector_from_yaml() {
        let yaml = r#"
version: 1
signatures:
  - id: "ipx-over-udp"
    engine: "IPX"
    protocol: "udp"
    priority: 80
    kind: "prefix-bytes"
    offset: 0
    hex: "ffff"
"#;
        let mut file: SignatureFile = serde_yaml::from_str(yaml).unwrap();
        for sig in &mut file.signatures {
            sig.compiled_bytes = compile_signature_bytes(sig).unwrap();
        }
        let detector = Detector {
            signatures: file.signatures,
            source_path: PathBuf::from("inline"),
        };

        let payload = [0xff, 0xff, 0x10, 0x20];
        let res = detector.detect("udp", &payload).unwrap();
        assert_eq!(res.engine, "IPX");
        assert_eq!(res.signature_id, "ipx-over-udp");
    }
}
