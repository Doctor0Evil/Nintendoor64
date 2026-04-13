use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::contract::BuildStep;

/// A single event for a tool invocation in a build contract run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildEvent {
    /// Stable contract id, if known.
    pub contract_id: Option<String>,
    /// Build step id, e.g. "step.n64.build".
    pub step_id: String,
    /// Optional logical step kind (retro, n64, starzip, custom).
    pub step_kind: Option<String>,
    /// Tool binary name, e.g. "n64-build" or "starzip-cli".
    pub tool: String,
    /// Resolved working directory (repo-relative).
    pub workdir: String,
    /// Command line args as executed.
    pub args: Vec<String>,
    /// Environment overrides for this invocation (if any).
    pub env: Vec<EnvPair>,
    /// Wall-clock start timestamp (UTC, RFC 3339).
    pub started_at: String,
    /// Wall-clock end timestamp (UTC, RFC 3339).
    pub ended_at: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Exit status code, if the process ran.
    pub exit_code: Option<i32>,
    /// True if the process exited successfully.
    pub success: bool,
    /// Optional short digest of stdout/stderr (first N lines).
    pub stdout_digest: Option<LogDigest>,
    pub stderr_digest: Option<LogDigest>,
}

/// Simple key/value pair for process environment overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvPair {
    pub key: String,
    pub value: String,
}

/// Small, token-friendly log summary instead of full streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogDigest {
    /// First N lines of the stream.
    pub head_lines: Vec<String>,
    /// Total number of lines observed.
    pub total_lines: usize,
}

/// Trace for a single `gamemodeai-build` contract execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTrace {
    /// Contract id (gm.build.*) if present in the JSON.
    pub contract_id: Option<String>,
    /// Workspace root directory (as passed by contract).
    pub workspaceroot: String,
    /// Build profile, e.g. "debug", "release", "ci".
    pub profile: String,
    /// Timestamp when `gamemodeai-build` started.
    pub started_at: String,
    /// Timestamp when `gamemodeai-build` finished.
    pub ended_at: String,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Ordered events, one per external tool invocation.
    pub events: Vec<BuildEvent>,
}

impl BuildTrace {
    pub fn new(
        contract_id: Option<String>,
        workspaceroot: impl AsRef<Path>,
        profile: impl Into<String>,
    ) -> BuildTrace {
        let now = SystemTime::now();
        BuildTrace {
            contract_id,
            workspaceroot: path_to_string(workspaceroot.as_ref()),
            profile: profile.into(),
            started_at: to_rfc3339(now),
            ended_at: to_rfc3339(now),
            duration_ms: 0,
            events: Vec::new(),
        }
    }

    pub fn record_event(&mut self, event: BuildEvent) {
        self.events.push(event);
    }

    pub fn finalize(&mut self, started: SystemTime, ended: SystemTime) {
        self.started_at = to_rfc3339(started);
        self.ended_at = to_rfc3339(ended);
        self.duration_ms = duration_to_ms(ended.duration_since(started).unwrap_or_default());
    }

    /// Writes the trace JSON to a stable path under artifacts/meta.
    ///
    /// For now we always write:
    ///   {workspaceroot}/artifacts/meta/build-trace.json
    pub fn write_to_disk(&self) -> io::Result<PathBuf> {
        let root = Path::new(&self.workspaceroot);
        let meta_dir = root.join("artifacts").join("meta");
        fs::create_dir_all(&meta_dir)?;
        let path = meta_dir.join("build-trace.json");
        let json = serde_json::to_vec_pretty(self).expect("serialize BuildTrace");
        fs::write(&path, json)?;
        Ok(path)
    }
}

/// Helper used by `main.rs` to build a pre-filled event.
/// The caller is expected to fill stdout/stderr digests and exit fields.
pub fn new_step_event(
    contract_id: Option<&str>,
    step: &BuildStep,
    tool: &str,
    workdir: &Path,
    args: &[String],
    env: &[(String, String)],
    start: SystemTime,
    end: SystemTime,
    status_code: Option<i32>,
    success: bool,
    stdout_digest: Option<LogDigest>,
    stderr_digest: Option<LogDigest>,
) -> BuildEvent {
    let step_kind = Some(format!("{:?}", step.kind)).map(|s| s.to_lowercase());
    BuildEvent {
        contract_id: contract_id.map(|s| s.to_owned()),
        step_id: step.id.clone(),
        step_kind,
        tool: tool.to_string(),
        workdir: path_to_string(workdir),
        args: args.to_vec(),
        env: env
            .iter()
            .map(|(k, v)| EnvPair {
                key: k.clone(),
                value: v.clone(),
            })
            .collect(),
        started_at: to_rfc3339(start),
        ended_at: to_rfc3339(end),
        duration_ms: duration_to_ms(end.duration_since(start).unwrap_or_default()),
        exit_code: status_code,
        success,
        stdout_digest,
        stderr_digest,
    }
}

/// Capture only the first N lines of a log stream for token-efficient digests.
pub fn digest_log(raw: &str, max_lines: usize) -> LogDigest {
    let mut lines = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        if idx < max_lines {
            lines.push(line.to_owned());
        }
    }
    LogDigest {
        head_lines: lines,
        total_lines: raw.lines().count(),
    }
}

fn to_rfc3339(t: SystemTime) -> String {
    // Use a small, dependency-free RFC3339-ish formatting.
    // If `chrono` or `time` is already in the workspace, you can swap this out.
    use std::time::UNIX_EPOCH;

    match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => {
            let secs = dur.as_secs();
            let nanos = dur.subsec_nanos();
            // This is intentionally simple; timezone is treated as UTC ("Z").
            format!("{secs}.{nanos:09}Z")
        }
        Err(_) => "0.000000000Z".to_string(),
    }
}

fn duration_to_ms(d: Duration) -> u64 {
    d.as_secs()
        .saturating_mul(1000)
        .saturating_add((d.subsec_nanos() as u64) / 1_000_000)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
