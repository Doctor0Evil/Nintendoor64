use serde::{Deserialize, Serialize};

use crate::session::profile::{SessionProfile, N64SessionConstraints};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintScoreBreakdown {
    pub hard_violations: Vec<String>,
    pub soft_scores: Vec<(String, f32)>,
    pub total_score: f32,
}

/// Minimal scoring function; extend with weights and more constraints later.
pub fn score_proposal_against_session(
    session: &SessionProfile,
    estimated_rom_size_bytes: Option<u64>,
    proposal_is_deterministic: bool,
) -> ConstraintScoreBreakdown {
    let mut hard_violations = Vec::new();
    let mut soft_scores = Vec::new();

    // Hard constraint: N64 ROM ceiling.
    if let Some(n64) = &session.n64_constraints {
        if n64.active {
            if let Some(size) = estimated_rom_size_bytes {
                if size > n64.n64_rom_ceiling_bytes {
                    hard_violations.push(format!(
                        "ROM size {} exceeds N64 ceiling {}",
                        size, n64.n64_rom_ceiling_bytes
                    ));
                }
            }
        }
    }

    // Hard constraint: determinism required unless experimental flag set.
    if !proposal_is_deterministic {
        if let Some(n64) = &session.n64_constraints {
            if n64.active && !n64.allow_non_deterministic_experiments {
                hard_violations.push("Non-deterministic proposal under deterministic N64 session"
                    .to_string());
            }
        }
    }

    // Simple scoring: each satisfied soft constraint adds +1.0.
    // In a later pass, attach explicit weights and richer soft constraints.

    if hard_violations.is_empty() {
        soft_scores.push(("base".to_string(), 0.0));
    }

    let total_score = if hard_violations.is_empty() {
        soft_scores.iter().map(|(_, s)| *s).sum()
    } else {
        f32::NEG_INFINITY
    };

    ConstraintScoreBreakdown {
        hard_violations,
        soft_scores,
        total_score,
    }
}
