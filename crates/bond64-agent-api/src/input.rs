// crates/bond64-agent-api/src/input.rs
use rdev::{listen, EventType, Key};
use std::sync::atomic::{AtomicU16, Ordering};

pub const N64_A: u16 = 0x8000;
// ... other buttons ...

static INPUT_STATE: AtomicU16 = AtomicU16::new(0);

fn update_bitmask(key: Key, pressed: bool) {
    let mask = match key {
        Key::KeyZ => N64_A,
        // map others...
        _ => 0,
    };
    if mask == 0 { return; }

    let current = INPUT_STATE.load(Ordering::SeqCst);
    let new_state = if pressed { current | mask } else { current & !mask };
    INPUT_STATE.store(new_state, Ordering::SeqCst);
}

pub fn start_input_mapper() {
    std::thread::spawn(|| {
        if let Err(err) = listen(move |event| {
            match event.event_type {
                EventType::KeyPress(key) => update_bitmask(key, true),
                EventType::KeyRelease(key) => update_bitmask(key, false),
                _ => {}
            }
        }) {
            eprintln!("Bond64 input mapper error: {:?}", err);
        }
    });
}

pub fn current_state() -> u16 {
    INPUT_STATE.load(Ordering::SeqCst)
}
