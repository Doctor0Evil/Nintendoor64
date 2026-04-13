use crate::command_descriptor::{CommandDescriptor, ComputeMode};
use crate::sessionprofile::SessionProfile;

/// Pure function deciding whether a command is permitted under the session.
pub fn allowed(command: &CommandDescriptor, session: &SessionProfile) -> bool {
    // 1. Compute mode gating.
    if !command.modes_permitted.is_empty() {
        let current_mode = session.compute_mode.unwrap_or(ComputeMode::LocalInteractive);
        if !command.modes_permitted.iter().any(|m| *m == current_mode) {
            return false;
        }
    }

    // 2. Optional future: target / platform gating via tags vs session.targets.
    if let Some(targets) = &session.targets {
        if !targets.is_empty() {
            let has_matching_target = command
                .tags
                .iter()
                .any(|t| targets.iter().any(|target| target == t));
            if !has_matching_target {
                return false;
            }
        }
    }

    // 3. Optional: invariant-based gating can be layered here later.

    true
}
