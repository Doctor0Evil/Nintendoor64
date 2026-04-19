use sonia_featurelayout::FeatureLayout;
use sonia_core::SessionProfile;

use crate::invariant_dsl::invariants_allow_system;

/// Check whether a Starzip preview command identified by its feature command ID
/// is permitted under the current SessionProfile invariants.
pub fn starzip_preview_allowed(
    session: &SessionProfile,
    feature_layout: &FeatureLayout,
    command_id: &str,
) -> bool {
    let feature = match feature_layout
        .features
        .iter()
        .find(|f| f.commands.iter().any(|c| c == command_id))
    {
        Some(f) => f,
        None => return false,
    };

    // Assume each feature has at least one system; use the first as the system ID.
    let system_id = match feature.systems.first() {
        Some(s) => s.as_str(),
        None => return false,
    };

    invariants_allow_system(session, feature_layout, system_id, &feature.tags)
}
