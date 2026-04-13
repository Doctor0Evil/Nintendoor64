use sonia_core::command_descriptor::{CommandDescriptor, ComputeMode};
use sonia_core::sessionprofile::SessionProfile;
use sonia_core::session_mode::allowed;

fn base_command() -> CommandDescriptor {
    CommandDescriptor {
        name: "createartifact".to_string(),
        full_command: "createartifact".to_string(),
        summary: "Create artifact".to_string(),
        input_schemas: vec!["schemas/artifact-spec.schema.json".to_string()],
        output_schemas: vec![],
        invariants: vec![],
        tags: vec!["Nintendoor64".to_string()],
        modes_permitted: vec![],
    }
}

fn base_session() -> SessionProfile {
    SessionProfile {
        repo: "Doctor0Evil/Nintendoor64".to_string(),
        branch: "main".to_string(),
        activecrate: Some("crates/sonia-core".to_string()),
        featureflags: vec![],
        invariants: vec![],
        // add fields as your concrete SessionProfile requires
        compute_mode: Some(ComputeMode::LocalInteractive),
        targets: Some(vec!["Nintendoor64".to_string()]),
        cistatus: Default::default(),
        recenttodos: vec![],
    }
}

#[test]
fn command_allowed_when_modes_empty() {
    let c = base_command();
    let s = base_session();
    assert!(allowed(&c, &s));
}

#[test]
fn command_forbidden_when_mode_mismatch() {
    let mut c = base_command();
    c.modes_permitted = vec![ComputeMode::CiBatch];
    let mut s = base_session();
    s.compute_mode = Some(ComputeMode::LocalInteractive);
    assert!(!allowed(&c, &s));
}

#[test]
fn command_allowed_when_mode_matches() {
    let mut c = base_command();
    c.modes_permitted = vec![ComputeMode::CiBatch];
    let mut s = base_session();
    s.compute_mode = Some(ComputeMode::CiBatch);
    assert!(allowed(&c, &s));
}
