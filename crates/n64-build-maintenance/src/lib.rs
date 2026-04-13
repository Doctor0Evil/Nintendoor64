use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::path::{Path, PathBuf};

/// Maintenance command the AI can request.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum MaintenanceCommand {
    /// Clean the Cargo target/ directory for this workspace.
    CleanTarget,
}

/// First-phase request: ask what a destructive command would do.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaintenancePreviewRequest {
    /// Which maintenance operation is being requested.
    pub command: MaintenanceCommand,

    /// Workspace root (usually repo root).
    pub workspace_root: String,
}

/// Summary of what a maintenance command will do.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceImpact {
    /// Human-readable summary of what will be deleted/affected.
    pub impact_summary: String,

    /// Approximate bytes that will be freed, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_to_free: Option<u64>,

    /// Paths that will be removed or altered.
    #[serde(default)]
    pub paths: Vec<String>,
}

/// First-phase response: preview only, no side effects.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaintenancePreviewResponse {
    /// Token that must be echoed in the confirm call.
    pub confirmation_token: String,

    /// Impact summary the UI should show to humans.
    pub impact: MaintenanceImpact,
}

/// Second-phase request: actually run the maintenance operation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceConfirmRequest {
    /// Must match the token from the preview response.
    pub confirmation_token: String,

    /// Same command that was previewed.
    pub command: MaintenanceCommand,

    /// Workspace root (for safety, re-validated).
    pub workspace_root: String,
}

/// Result of a destructive maintenance command.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceConfirmResponse {
    /// True if the command completed successfully.
    pub ok: bool,

    /// Human-friendly message (for logs / UI).
    pub message: String,

    /// Total bytes actually freed, if measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_freed: Option<u64>,
}

/// Internal helper: compute target/ path under workspace_root.
fn target_dir(root: &Path) -> PathBuf {
    root.join("target")
}

/// Roughly estimate the size of a directory (best-effort).
fn dir_size_bytes(root: &Path) -> std::io::Result<u64> {
    let mut total: u64 = 0;
    if !root.exists() {
        return Ok(0);
    }
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Ok(total)
}

/// Recursively remove a directory (equivalent to `rm -rf` / `fs::remove_dir_all`).
fn remove_dir_all(root: &Path) -> std::io::Result<()> {
    if root.exists() {
        std::fs::remove_dir_all(root)?;
    }
    Ok(())
}

/// Compute a stable confirmation token from command + workspace_root.
/// For now this is just a hash of the inputs; you can harden later.
fn compute_confirmation_token(cmd: &MaintenanceCommand, root: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}::{}", cmd, root));
    let digest = hasher.finalize();
    format!("{:x}", digest)
}

/// Public API: preview a maintenance command without side-effects.
pub fn preview(req: MaintenancePreviewRequest) -> anyhow::Result<MaintenancePreviewResponse> {
    let workspace = PathBuf::from(&req.workspace_root);
    match req.command {
        MaintenanceCommand::CleanTarget => {
            let target = target_dir(&workspace);
            let approx = dir_size_bytes(&target).unwrap_or(0);
            let token = compute_confirmation_token(&req.command, &req.workspace_root);

            let impact = MaintenanceImpact {
                impact_summary: if target.exists() {
                    format!(
                        "This will delete the Cargo target directory at '{}' and free approximately {} bytes.",
                        target.display(),
                        approx
                    )
                } else {
                    format!(
                        "No target directory found at '{}'; nothing will be deleted.",
                        target.display()
                    )
                },
                bytes_to_free: Some(approx),
                paths: vec![target.to_string_lossy().to_string()],
            };

            Ok(MaintenancePreviewResponse {
                confirmation_token: token,
                impact,
            })
        }
    }
}

/// Public API: execute a previously previewed maintenance command.
pub fn confirm(req: MaintenanceConfirmRequest) -> anyhow::Result<MaintenanceConfirmResponse> {
    let expected_token = compute_confirmation_token(&req.command, &req.workspace_root);
    if expected_token != req.confirmation_token {
        anyhow::bail!("confirmationToken mismatch; refusing to run destructive command");
    }

    let workspace = PathBuf::from(&req.workspace_root);
    match req.command {
        MaintenanceCommand::CleanTarget => {
            let target = target_dir(&workspace);
            let before = dir_size_bytes(&target).unwrap_or(0);
            remove_dir_all(&target)?;
            let after = dir_size_bytes(&target).unwrap_or(0);
            let freed = before.saturating_sub(after);

            Ok(MaintenanceConfirmResponse {
                ok: true,
                message: format!(
                    "Deleted Cargo target directory at '{}'; freed approximately {} bytes.",
                    target.display(),
                    freed
                ),
                bytes_freed: Some(freed),
            })
        }
    }
}
