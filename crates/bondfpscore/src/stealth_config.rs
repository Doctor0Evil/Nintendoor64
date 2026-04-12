use crate::stealth_ai::StealthParams;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct StealthConfigFile {
    schema_version: u32,
    profile_id: String,
    global: Global,
    posture_factors: PostureFactors,
    movement_factors: MovementFactors,
    // sound omitted for now
}

#[derive(Debug, Deserialize)]
struct Global {
    base_visibility: f32,
    d_max: f32,
    k_decay: f32,
    t_suspicious: f32,
    t_alert: f32,
    light_exponent: f32,
    move_exponent: f32,
}

#[derive(Debug, Deserialize)]
struct PostureFactors {
    standing: f32,
    crouch: f32,
    prone: f32,
}

#[derive(Debug, Deserialize)]
struct MovementFactors {
    still: f32,
    walk: f32,
    run: f32,
}

pub fn load_stealth_params<P: AsRef<Path>>(path: P) -> Result<StealthParams, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cfg: StealthConfigFile = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    if cfg.schema_version != 1 {
        return Err(format!("Unsupported schema_version {}", cfg.schema_version));
    }

    let params = StealthParams {
        base_visibility: cfg.global.base_visibility,
        d_max: cfg.global.d_max,
        k_decay: cfg.global.k_decay,
        t_suspicious: cfg.global.t_suspicious,
        t_alert: cfg.global.t_alert,
        light_exponent: cfg.global.light_exponent,
        move_exponent: cfg.global.move_exponent,
        posture_standing: cfg.posture_factors.standing,
        posture_crouch: cfg.posture_factors.crouch,
        posture_prone: cfg.posture_factors.prone,
        move_still: cfg.movement_factors.still,
        move_walk: cfg.movement_factors.walk,
        move_run: cfg.movement_factors.run,
    };

    params.validate()?;
    Ok(params)
}
