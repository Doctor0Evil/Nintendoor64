use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "command")]
pub enum PreviewCommand {
    RomQuery {
        rom_id: String,
        layout_path: String,
    },
    PatchPreview {
        base_rom_id: String,
        layout_path: String,
        patch_path: String,
        #[serde(default)]
        dry_run: bool,
        #[serde(default)]
        budget_only: bool,
    }
}
