// tools/rust-macro-expand/src/main.rs

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MacroSpan {
    pub file: Option<String>,
    pub line_start: Option<u32>,
    pub column_start: Option<u32>,
    pub line_end: Option<u32>,
    pub column_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MacroExpandRequest {
    /// Workspace root; if omitted, use current directory.
    pub workspace_root: Option<String>,
    /// Cargo package name to expand.
    pub package: String,
    /// Optional binary/lib/test target name.
    pub target: Option<String>,
    /// Optional path to a file containing the macro callsite.
    pub file_hint: Option<String>,
    /// Optional line number for the macro callsite (1-based).
    pub line_hint: Option<u32>,
    /// Limit for expanded output bytes (to avoid megabytes of code).
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExpandedSnippet {
    /// Snippet of expanded code.
    pub code: String,
    /// Optional best-effort span mapping back to the original macro callsite.
    pub callsite_span: Option<MacroSpan>,
    /// True if the snippet is truncated due to max_bytes.
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MacroExpandError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MacroExpandResponse {
    pub ok: bool,
    pub error: Option<MacroExpandError>,
    pub snippet: Option<ExpandedSnippet>,
}

fn main() -> Result<()> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let req: MacroExpandRequest =
        serde_json::from_str(&buf).context("parsing MacroExpandRequest from stdin")?;

    let resp = handle_request(req).unwrap_or_else(|e| MacroExpandResponse {
        ok: false,
        error: Some(MacroExpandError {
            message: format!("{e:#}"),
        }),
        snippet: None,
    });

    let out = serde_json::to_string_pretty(&resp)?;
    println!("{out}");
    Ok(())
}

fn handle_request(req: MacroExpandRequest) -> Result<MacroExpandResponse> {
    let mut cmd = Command::new("cargo");
    cmd.arg("expand");
    cmd.arg("-p");
    cmd.arg(&req.package);

    if let Some(target) = &req.target {
        cmd.arg("--bin").arg(target);
    }

    if let Some(root) = &req.workspace_root {
        cmd.current_dir(root);
    }

    // For now, we expand the whole target; a later refinement can trim
    // to file_hint/line_hint by parsing the expanded output.
    let output = cmd
        .output()
        .with_context(|| "failed to spawn `cargo expand` (is it installed?)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cargo expand failed: {stderr}");
    }

    let mut expanded = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut truncated = false;
    if let Some(limit) = req.max_bytes {
        if expanded.len() > limit {
            expanded.truncate(limit);
            truncated = true;
        }
    }

    // Very naive mapping: we just echo the hint back as the callsite span.
    // A later iteration can parse rustc JSON diagnostics to find the exact expansion site.
    let callsite_span = match (req.file_hint, req.line_hint) {
        (Some(file), Some(line)) => Some(MacroSpan {
            file: Some(file),
            line_start: Some(line),
            column_start: None,
            line_end: None,
            column_end: None,
        }),
        _ => None,
    };

    Ok(MacroExpandResponse {
        ok: true,
        error: None,
        snippet: Some(ExpandedSnippet {
            code: expanded,
            callsite_span,
            truncated,
        }),
    })
}
