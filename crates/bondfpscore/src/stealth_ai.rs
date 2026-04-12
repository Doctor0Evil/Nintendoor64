//! Bond-like FPS Stealth AI: detection and awareness model.
//!
//! This module implements a continuous visibility score V and an awareness
//! accumulator A per guard, using tunable parameters so designers (or Lua)
//! can tweak stealth behavior without touching the core math.

use std::f32::consts::PI;

/// Basic vector3 helpers – you can swap to glam or nalgebra later.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }

    #[inline]
    pub fn dot(self, rhs: Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len <= 1e-6 {
            self
        } else {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        }
    }
}

/// Player posture affects visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Posture {
    Standing,
    Crouched,
    Prone,
}

/// Movement state affects visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementState {
    Still,
    Walking,
    Running,
}

/// Discrete guard awareness state, driven by the continuous awareness meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwarenessState {
    Idle,
    Suspicious,
    Alert,
}

/// Per-guard stealth state – store this as a component.
#[derive(Debug, Clone, Copy)]
pub struct GuardStealthState {
    /// Normalized [0, 1] awareness meter.
    pub awareness: f32,
    pub state: AwarenessState,
}

impl GuardStealthState {
    #[inline]
    pub fn new() -> Self {
        Self {
            awareness: 0.0,
            state: AwarenessState::Idle,
        }
    }
}

/// Tunable stealth parameters. These should be loaded from data (JSON/Lua)
/// and cached in a resource in your ECS world.
#[derive(Debug, Clone)]
pub struct StealthParams {
    /// Base detectability constant.
    pub base_visibility: f32,
    /// Maximum distance at which player can ever be detected.
    pub max_distance: f32,
    /// Exponent for distance falloff (>= 1).
    pub distance_exponent: f32,
    /// Exponent for light influence (>= 1).
    pub light_exponent: f32,
    /// Posture factors.
    pub posture_standing: f32,
    pub posture_crouched: f32,
    pub posture_prone: f32,
    /// Movement factors.
    pub move_still: f32,
    pub move_walking: f32,
    pub move_running: f32,
    /// Awareness decay per second.
    pub awareness_decay: f32,
    /// Thresholds for state transitions (0 <= suspicious < alert <= 1).
    pub threshold_suspicious: f32,
    pub threshold_alert: f32,
}

impl Default for StealthParams {
    fn default() -> Self {
        Self {
            base_visibility: 1.0,
            max_distance: 30.0,
            distance_exponent: 1.5,
            light_exponent: 1.2,
            posture_standing: 1.0,
            posture_crouched: 0.6,
            posture_prone: 0.3,
            move_still: 0.5,
            move_walking: 1.0,
            move_running: 1.5,
            awareness_decay: 0.4,
            threshold_suspicious: 0.4,
            threshold_alert: 0.8,
        }
    }
}

/// Continuous detection inputs for a single guard evaluating a single player.
#[derive(Debug, Clone, Copy)]
pub struct DetectionInputs {
    /// Guard eye position.
    pub guard_pos: Vec3,
    /// Guard forward direction (normalized).
    pub guard_forward: Vec3,
    /// Player position (at head/torso).
    pub player_pos: Vec3,
    /// Ambient light at player [0, 1].
    pub light_level: f32,
    pub posture: Posture,
    pub movement: MovementState,
    /// Whether there is a clear line of sight.
    pub has_line_of_sight: bool,
}

/// Compute distance-based falloff f_d(d) in [0, 1].
fn distance_factor(d: f32, max_distance: f32, exponent: f32) -> f32 {
    if d <= 0.0 {
        1.0
    } else if d >= max_distance {
        0.0
    } else {
        let t = 1.0 - (d / max_distance).clamp(0.0, 1.0);
        t.powf(exponent.max(1.0))
    }
}

/// Light factor f_L(L) in [0, 1]; higher light means more visible.
fn light_factor(light_level: f32, exponent: f32) -> f32 {
    let l = light_level.clamp(0.0, 1.0);
    l.powf(exponent.max(1.0))
}

/// Posture factor f_P(P).
fn posture_factor(posture: Posture, params: &StealthParams) -> f32 {
    match posture {
        Posture::Standing => params.posture_standing,
        Posture::Crouched => params.posture_crouched,
        Posture::Prone => params.posture_prone,
    }
}

/// Movement factor f_M(M).
fn movement_factor(movement: MovementState, params: &StealthParams) -> f32 {
    match movement {
        MovementState::Still => params.move_still,
        MovementState::Walking => params.move_walking,
        MovementState::Running => params.move_running,
    }
}

/// Compute instantaneous visibility score V.
///
/// If there is no line of sight, V is forced to 0. You can extend this later
/// with sound-based detection by adding a separate hearing term.
pub fn compute_visibility(params: &StealthParams, inputs: &DetectionInputs) -> f32 {
    if !inputs.has_line_of_sight {
        return 0.0;
    }

    let delta = inputs.player_pos.sub(inputs.guard_pos);
    let distance = delta.length();

    if distance > params.max_distance {
        return 0.0;
    }

    let fd = distance_factor(distance, params.max_distance, params.distance_exponent);
    let fl = light_factor(inputs.light_level, params.light_exponent);
    let fp = posture_factor(inputs.posture, params);
    let fm = movement_factor(inputs.movement, params);

    let v = params.base_visibility * fd * fl * fp * fm;
    v.max(0.0)
}

/// Update awareness A over a timestep dt (seconds) and return new state.
///
/// A_{t+dt} = clamp(A_t + V dt - k_decay dt, 0, 1),
/// then thresholds are applied to determine the discrete AwarenessState.
pub fn update_awareness(
    params: &StealthParams,
    mut guard_state: GuardStealthState,
    visibility: f32,
    dt: f32,
) -> GuardStealthState {
    let dv = visibility * dt;
    let decay = params.awareness_decay * dt;

    let mut a = guard_state.awareness + dv - decay;
    if a < 0.0 {
        a = 0.0;
    } else if a > 1.0 {
        a = 1.0;
    }
    guard_state.awareness = a;

    guard_state.state = if a >= params.threshold_alert {
        AwarenessState::Alert
    } else if a >= params.threshold_suspicious {
        AwarenessState::Suspicious
    } else {
        AwarenessState::Idle
    };

    guard_state
}

/// Optional helper: compute whether player is within the guard's vision cone.
///
/// This is separate from visibility; you might require cone + LoS to be true
/// before calling `compute_visibility`.
pub fn within_vision_cone(
    guard_forward: Vec3,
    guard_to_player: Vec3,
    cone_angle_degrees: f32,
) -> bool {
    let f = guard_forward.normalize();
    let dir = guard_to_player.normalize();
    let dot = f.dot(dir).clamp(-1.0, 1.0);
    let angle = dot.acos(); // radians
    let cone_radians = cone_angle_degrees * (PI / 180.0);
    angle <= cone_radians * 0.5
}

/// Example of a single-tick update for one guard observing one player.
///
/// In an ECS, you would run a system that:
/// - Iterates guards + their GuardStealthState.
/// - Gathers DetectionInputs per guard (from components for player, light, LoS).
/// - Updates the GuardStealthState and branches AI behavior based on state.
pub fn tick_guard_stealth(
    params: &StealthParams,
    guard_state: GuardStealthState,
    inputs: &DetectionInputs,
    dt: f32,
    cone_angle_degrees: f32,
) -> GuardStealthState {
    let delta = inputs.player_pos.sub(inputs.guard_pos);
    let in_cone = within_vision_cone(inputs.guard_forward, delta, cone_angle_degrees);

    let visibility = if in_cone {
        compute_visibility(params, inputs)
    } else {
        0.0
    };

    update_awareness(params, guard_state, visibility, dt)
}
