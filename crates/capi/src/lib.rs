use arena_shooter_core::{arena_init, arena_step, ArenaConfig, WeaponConfig};
use core_ecs::{
    core_export_state, core_import_state, core_init, core_load_snapshot, core_save_snapshot,
    core_shutdown, core_step_world, CoreConfig, InputFrame, SnapshotId, WorldHandle,
};

#[repr(C)]
pub struct GMCoreConfig {
    pub max_entities: u32,
    pub max_frames_history: u32,
    pub tickrate_hz: u32,
}

#[repr(C)]
pub struct GMInputFrame {
    pub frame: u32,
    pub player_id: u8,
    pub buttons: u32,
    pub analog_x: i16,
    pub analog_y: i16,
}

#[repr(C)]
pub struct GMSnapshotId {
    pub id: u32,
}

#[repr(C)]
pub struct GMArenaConfig {
    pub num_players: u8,
    pub map_min_x: f32,
    pub map_min_z: f32,
    pub map_max_x: f32,
    pub map_max_z: f32,
    pub move_speed: f32,
}

#[repr(C)]
pub struct GMWeaponConfig {
    pub damage_per_shot: i32,
    pub fire_rate_ticks: u32,
    pub max_range_units: f32,
}

#[repr(C)]
pub struct GMWorldHandle {
    pub id: u32,
}

fn from_core_cfg(cfg: GMCoreConfig) -> CoreConfig {
    CoreConfig {
        max_entities: cfg.max_entities,
        max_frames_history: cfg.max_frames_history,
        tickrate_hz: cfg.tickrate_hz,
    }
}

fn from_arena_cfg(cfg: GMArenaConfig) -> ArenaConfig {
    ArenaConfig {
        num_players: cfg.num_players,
        map_min_x: cfg.map_min_x,
        map_min_z: cfg.map_min_z,
        map_max_x: cfg.map_max_x,
        map_max_z: cfg.map_max_z,
        move_speed: cfg.move_speed,
    }
}

fn from_weapon_cfg(cfg: GMWeaponConfig) -> WeaponConfig {
    WeaponConfig {
        damage_per_shot: cfg.damage_per_shot,
        fire_rate_ticks: cfg.fire_rate_ticks,
        max_range_units: cfg.max_range_units,
    }
}

fn from_input_slice(slice: *const GMInputFrame, len: u32) -> Vec<InputFrame> {
    if slice.is_null() || len == 0 {
        return Vec::new();
    }
    let frames = unsafe { std::slice::from_raw_parts(slice, len as usize) };
    frames
        .iter()
        .map(|f| InputFrame {
            frame: f.frame,
            player_id: f.player_id,
            buttons: f.buttons,
            analog_x: f.analog_x,
            analog_y: f.analog_y,
        })
        .collect()
}

#[no_mangle]
pub extern "C" fn gm_core_init(cfg: GMCoreConfig) -> GMWorldHandle {
    let world = core_init(from_core_cfg(cfg));
    GMWorldHandle { id: world.0 }
}

#[no_mangle]
pub extern "C" fn gm_core_shutdown(handle: GMWorldHandle) {
    let world = WorldHandle(handle.id);
    core_shutdown(world);
}

#[no_mangle]
pub extern "C" fn gm_arena_init(
    handle: GMWorldHandle,
    arena_cfg: GMArenaConfig,
    weapon_cfg: GMWeaponConfig,
) {
    let world = WorldHandle(handle.id);
    arena_init(world, from_arena_cfg(arena_cfg), from_weapon_cfg(weapon_cfg));
}

#[no_mangle]
pub extern "C" fn gm_core_step(
    handle: GMWorldHandle,
    inputs: *const GMInputFrame,
    inputs_len: u32,
) {
    let world = WorldHandle(handle.id);
    let frames = from_input_slice(inputs, inputs_len);
    arena_step(world, &frames);
}

#[no_mangle]
pub extern "C" fn gm_core_save_snapshot(handle: GMWorldHandle) -> GMSnapshotId {
    let world = WorldHandle(handle.id);
    let id = core_save_snapshot(world);
    GMSnapshotId { id: id.0 }
}

#[no_mangle]
pub extern "C" fn gm_core_load_snapshot(handle: GMWorldHandle, snap: GMSnapshotId) {
    let world = WorldHandle(handle.id);
    core_load_snapshot(world, SnapshotId(snap.id));
}

#[no_mangle]
pub extern "C" fn gm_core_export_state(
    handle: GMWorldHandle,
    out_buf: *mut u8,
    out_capacity: u32,
) -> u32 {
    let world = WorldHandle(handle.id);
    let mut tmp = Vec::new();
    core_export_state(world, &mut tmp);
    let n = tmp.len().min(out_capacity as usize);
    if n > 0 && !out_buf.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(tmp.as_ptr(), out_buf, n);
        }
    }
    n as u32
}

#[no_mangle]
pub extern "C" fn gm_core_import_state(
    handle: GMWorldHandle,
    bytes: *const u8,
    len: u32,
) {
    if bytes.is_null() || len == 0 {
        return;
    }
    let world = WorldHandle(handle.id);
    let slice = unsafe { std::slice::from_raw_parts(bytes, len as usize) };
    core_import_state(world, slice);
}
