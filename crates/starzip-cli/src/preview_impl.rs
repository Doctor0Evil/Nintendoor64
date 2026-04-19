use n64_layout::RomLayout;
use n64_layout::soniabridge::{PatchImpactReport, SoniaBridge};

pub fn run_patch_preview(
    base_rom_id: &str,
    layout_path: &str,
    patch_path: &str,
    budget_only: bool,
) -> anyhow::Result<PatchImpactReport> {
    let layout_text = std::fs::read_to_string(layout_path)?;
    let layout: RomLayout = serde_json::from_str(&layout_text)?;

    let patch_text = std::fs::read_to_string(patch_path)?;
    let patch: crate::soniabridge::PatchSpec = serde_json::from_str(&patch_text)?;

    let bridge = SoniaBridge::new(layout)?;
    let payload_index = crate::soniabridge::PayloadIndex::new();

    let mut report = bridge.compute_patch_impact(&patch, &payload_index, base_rom_id)?;

    if budget_only {
        // If budget-only mode, strip any fine-grained edit details
        // and keep only summary and segment-level budget data.
        report.edits.clear();
    }

    Ok(report)
}
