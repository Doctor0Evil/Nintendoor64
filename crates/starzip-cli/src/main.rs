use crate::preview_impl::{run_rom_query_preview, run_patch_preview};

match cmd {
    PreviewCommand::RomQuery { rom_id, layout_path } => {
        let preview = run_rom_query_preview(&rom_id, &layout_path)?;
        let env = PreviewEnvelope {
            version: 1,
            status: "ok".to_string(),
            data: Some(serde_json::to_value(preview)?),
            error: None,
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
    }
    PreviewCommand::PatchPreview {
        base_rom_id,
        layout_path,
        patch_path,
        dry_run,
        budget_only,
    } => {
        let impact = run_patch_preview(&base_rom_id, &layout_path, &patch_path, budget_only)?;
        let env = PreviewEnvelope {
            version: 1,
            status: "ok".to_string(),
            data: Some(serde_json::to_value(impact)?),
            error: None,
        };
        println!("{}", serde_json::to_string_pretty(&env)?);

        if !dry_run {
            // Optional: log to a local preview cache, or emit a separate ArtifactSpec
            // describing the preview report for CI or human inspection.
        }
    }
}
