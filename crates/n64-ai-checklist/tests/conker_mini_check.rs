use std::fs;
use std::path::PathBuf;

use n64_ai_checklist::{ai_checklist, ChecklistResult};
use serde_json::from_str;

#[test]
fn conker_mini_layout_constraints_patch_are_ai_safe() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let layout_path =
        root.join("examples/n64/layouts/conker-mini.layout.json");
    let constraints_path =
        root.join("examples/n64/constraints/conker-mini.constraints.json");
    let patch_path =
        root.join("examples/n64/patches/conker-mini-title.patch.json");

    let layout_str = fs::read_to_string(layout_path).expect("layout");
    let constraints_str = fs::read_to_string(constraints_path).expect("constraints");
    let patch_str = fs::read_to_string(patch_path).expect("patch");

    let layout = from_str(&layout_str).expect("layout json");
    let constraints = from_str(&constraints_str).expect("constraints json");
    let patch = from_str(&patch_str).expect("patch json");

    let ChecklistResult { ok, issues } =
        ai_checklist(&layout, &constraints, &patch);

    if !ok {
        for issue in issues {
            eprintln!("- {}: {}", issue.id, issue.message);
        }
    }

    assert!(ok, "ai_checklist must pass for conker-mini vertical slice");
}
