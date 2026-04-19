use std::cell::Cell;
use std::rc::Rc;

use gamemodeai_build::dispatcher::{Dispatcher, UnknownCommandError};
use gamemodeai_build::model::{LaneSpec, LaneStep};

#[test]
fn dispatcher_is_all_or_nothing_per_lane() {
    let lane = LaneSpec {
        id: "lane-all-or-nothing".to_string(),
        steps: vec![
            LaneStep {
                id: "first-valid".to_string(),
                command: "known".to_string(),
                params: serde_json::json!({}),
            },
            LaneStep {
                id: "second-invalid".to_string(),
                command: "unknown".to_string(),
                params: serde_json::json!({}),
            },
            LaneStep {
                id: "third-valid".to_string(),
                command: "known-again".to_string(),
                params: serde_json::json!({}),
            },
        ],
    };

    let known_executed = Rc::new(Cell::new(false));
    let known_again_executed = Rc::new(Cell::new(false));

    let known_executed_clone = known_executed.clone();
    let known_again_executed_clone = known_again_executed.clone();

    let mut dispatcher = Dispatcher::new();

    dispatcher.register("known", move |_step| {
        known_executed_clone.set(true);
        Ok(serde_json::json!({"ok": true, "step": "first-valid"}))
    });

    dispatcher.register("known-again", move |_step| {
        known_again_executed_clone.set(true);
        Ok(serde_json::json!({"ok": true, "step": "third-valid"}))
    });

    let result = dispatcher.execute_lane(&lane);

    match result {
        Err(UnknownCommandError {
            lane_id,
            step_id,
            command,
        }) => {
            assert_eq!(lane_id, "lane-all-or-nothing");
            assert_eq!(step_id, "second-invalid");
            assert_eq!(command, "unknown");
        }
        Ok(_) => panic!("expected UnknownCommandError for unknown command"),
    }

    // The first known step must not run, because the dispatcher should detect
    // the unknown command before executing any steps.
    assert!(!known_executed.get());

    // The third step must also not run, because execution stops at first error.
    assert!(!known_again_executed.get());
}
