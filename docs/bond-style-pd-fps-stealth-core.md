## 1. Research objects for the Bond/PD‑style stealth core

From your design docs, the stealth/missions slice for a GoldenEye/Perfect Dark–like core decomposes into these concrete research objects. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

1. **Perception geometry and room/portal visibility**

   You already called out STAN/PAD‑style room visibility, vision cones, and line-of-sight. For our Rust ECS core, we want:
   - A room/sector index per entity (STAN/PAD analogue).
   - A sparse graph of adjacency between rooms (portals/doors).
   - A deterministic LOS test: “player visible from guard?” that checks both room connectivity and geometry, but can be stubbed as “same room or adjacent and no hard blocker” in the first pass. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

2. **Vision score model**

   Your doc already defines a scalar visibility score \(V\) combining distance, light, posture, and movement: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   \[
   V = B_{\text{base}} \cdot f_d(d) \cdot f_L(L) \cdot f_P(P) \cdot f_M(M)
   \]

   where:
   - \(B_{\text{base}}\) is base detectability of the player (per difficulty/mission).
   - \(d\) is distance guard–player; \(f_d\) is a falloff.
   - \(L\) is light level at player; \(f_L\) increases danger in bright light.
   - \(P\) is posture (standing/crouched); \(f_P\) reduces detectability when crouched.
   - \(M\) is movement category (still/walk/run); \(f_M\) increases detectability when moving fast. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   A simple deterministic falloff you sketched is:

   \[
   f_d(d) = \max\left(0,\ 1 - \frac{d}{d_{\max}}\right)
   \]

   with clamp to \([0,1]\), and:

   \[
   f_L(L) = L, \quad f_P \in \{1.0, 0.6\}, \quad f_M \in \{0.5, 1.0, 1.5\}.
   \]  [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

3. **Sound propagation and hearing**

   You described sound radii and awareness spikes; we can model:

   - Each “noise event” \(E\) has:
     - Center position, radius \(r\), and strength \(S\).
   - For a guard at distance \(d\) from \(E\), hearing contribution:

     \[
     H = S \cdot \max\left(0,\ 1 - \frac{d}{r}\right)
     \]

   - Guards only consider events within their current or connected rooms; this is a discrete analogue to GoldenEye’s “noise cells.” [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

4. **Awareness accumulation and thresholds**

   Your doc already suggests an awareness accumulator \(A_t\) with decay: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   \[
   A_{t + \Delta t} = \max\left(0,\ A_t + (V + H)\Delta t - k_{\text{decay}}\Delta t\right)
   \]

   with thresholds:

   - \(A < T_{\text{susp}}\): Idle/Patrol.
   - \(T_{\text{susp}} \le A < T_{\text{alert}}\): Suspicious/Investigate.
   - \(A \ge T_{\text{alert}}\): Alert/Attack. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   These thresholds are mission/difficulty tunables you want in JSON/TOML.

5. **AI state graph (STAN‑like)**

   You already fixed the canonical states: Idle, Patrol, Suspicious, Investigate, Alert, Attack, Flee, Surrender, and transitions based on awareness, LOS, damage, and scripted triggers.  For this module we can focus on: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   - Idle, Patrol, Suspicious, Investigate, Alert, Attack.

   Transitions must be deterministic and purely a function of current state, awareness, LOS, and recent events (damage/sound). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

6. **Mission state and objective DAG hooks**

   You already formalized objectives as nodes in a DAG: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   \[
   \text{complete}(O_i, s) = C_i(s) \land \bigwedge_{O_j \in \text{pre}(O_i)} \text{complete}(O_j, s)
   \]

   For stealth, the research object is: “how do AI events feed the DAG?” e.g.:

   - “No alarms raised” objective fails if any guard enters Alert and triggers an alarm.
   - “Avoid casualties” fails if any civilian entity dies.
   - “Remain undetected” fails if awareness for any guard hits the Alert threshold while LOS to player is true. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

   That means the stealth system must emit deterministic mission events (AlarmRaised, PlayerDetected, CivilianKilled) that the DAG evaluation system consumes.

7. **Data schemas and KG entries**

   Your design requires each pattern to have:

   - A Rust crate (e.g. `bondfpscore::stealth_ai`).
   - A machine‑readable schema file for tunables.
   - Unit tests and determinism checks.
   - A SystemNode in `knowledgegraph/systems.json` tying IDs like `systems.bondfpscore.stealth_ai` to source files and schemas. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

***

## 2. Math formulas to lock down for this module

Given the above, here are the specific formulas worth “freezing” in code and schemas now, so they form the canonical Bond/PD stealth reference you can reuse. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

### 2.1 Vision visibility score

We adopt and slightly generalize the formula already in your doc: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

\[
V = 
\begin{cases}
0 & \text{if not in FOV or LOS blocked} \\
B_{\text{base}} \cdot f_d(d) \cdot f_L(L) \cdot f_P(P) \cdot f_M(M) & \text{otherwise}
\end{cases}
\]

where:

- \(f_d(d) = \max\left(0,\ 1 - \left(\frac{d}{d_{\max}}\right)^{\gamma_d}\right)\); \(\gamma_d\) controls how fast visibility drops with distance.
- \(f_L(L) = L^{\gamma_L}\) with \(L \in [0,1]\).
- \(f_P(P)\) is a per‑posture coefficient (standing/crouched/prone).
- \(f_M(M)\) is a per‑movement coefficient (still/walk/run). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

All these are scalar multipliers that we store in data so PD vs GoldenEye vs Tomorrow Never Dies can share code but differ in tables.

### 2.2 FOV and room constraints

FOV test mirrors your GoldenEye auto‑aim math: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

- Let guard forward vector be \(g\), and unit vector to player \(p\).
- Compute \(\cos\theta = g \cdot p\).
- Player in vision cone if \(\cos\theta \ge \cos(\theta_{\max})\) and \(d \le d_{\max}\).

Room visibility:

- Define integer `room_id` for each entity, and a small adjacency matrix or list.
- A fast approximation:

  - Player is “potentially visible” if `room_id_guard == room_id_player` or if `(room_id_guard, room_id_player)` is in the adjacency set.
  - A more advanced version can require passing through at most `N` portals (configurable). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

### 2.3 Sound contribution and decay

For each sound event E in the last frame: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

\[
H_E = 
\begin{cases}
S \cdot \max\left(0,\ 1 - \frac{d}{r}\right) & d \le r \\
0 & d > r
\end{cases}
\]

Total hearing contribution:

\[
H = \sum_E H_E
\]

You can clamp H per tick to a maximum to prevent pathological spikes.

### 2.4 Awareness integration

Per guard, with fixed tick \(\Delta t\) (your engine already runs fixed step): [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

\[
A_{t + \Delta t} = \mathrm{clamp}\left(A_t + (V + H)\Delta t - k_{\text{decay}}\Delta t,\ 0,\ A_{\max}\right)
\]

Where:

- \(k_{\text{decay}}\) is a per‑AI or per‑difficulty decay rate.
- \(A_{\max}\) is a ceiling (e.g. 1.0 or 100) to avoid numeric blowup. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

State transitions:

- If \(A\) crosses \(T_{\text{susp}}\) upward: Idle/Patrol → Suspicious.
- If \(A\) crosses \(T_{\text{alert}}\) upward or guard takes damage: Suspicious/Investigate → Alert/Attack.
- If \(A\) falls below \(T_{\text{susp}} - H_{\text{hyst}}\): Suspicious → Idle/Patrol.
- If LOS lost and \(A\) < \(T_{\text{alert}}\): Alert/Attack → Investigate (after a timeout). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

You can encode this as a small deterministic state machine with thresholds and hysteresis in data.

### 2.5 Hooks into objectives

When a state transition fires, you map to mission events:

- `Idle/Patrol -> Alert/Attack` and `can_trigger_alarm == true` → emit `MissionEvent::AlarmRaised`.
- First time any guard’s `A` exceeds `T_alert` while LOS to player is true → emit `MissionEvent::PlayerDetected`.
- Guard shoots or hits player → emit `MissionEvent::CombatEngaged`. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

These are evaluated by your generic objective DAG system, so stealth rules are data, not hard‑coded. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

***

## 3. Concrete Rust module: `crates/bondfpscore/src/stealth_ai.rs`

Below is a minimal but full Rust module that implements the formulas above as deterministic systems, ready to be plugged into your existing ECS core. It assumes:

- A simple ECS API with `World`, `Component` derive, and query iteration (you can adapt to your chosen ECS).
- Fixed timestep (pass `delta_seconds` into the update).
- No randomness, no IO; pure function systems only. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

`crates/bondfpscore/src/stealth_ai.rs`:

```rust
// File: crates/bondfpscore/src/stealth_ai.rs

use serde::{Deserialize, Serialize};

/// Scalar type for stealth math; use f32 but keep ranges tight and deterministic.
pub type Scalar = f32;

/// Identifier for a logical "room" or sector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(pub u16);

/// Simple 3D vector for geometry; replace with your math crate if desired.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl Vec3 {
    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }

    pub fn dot(self, other: Vec3) -> Scalar {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(self) -> Scalar {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len <= 0.0 {
            Vec3 { x: 0.0, y: 0.0, z: 0.0 }
        } else {
            Vec3 { x: self.x / len, y: self.y / len, z: self.z / len }
        }
    }
}

/// Player posture categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Posture {
    Standing,
    Crouched,
    Prone,
}

/// Player movement categories.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Movement {
    Still,
    Walk,
    Run,
}

/// AI awareness-driven state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StealthState {
    Idle,
    Patrol,
    Suspicious,
    Investigate,
    Alert,
    Attack,
}

/// Component: where an entity is, which room it belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StealthTransform {
    pub position: Vec3,
    pub room: RoomId,
}

/// Component: attached to the player, providing posture, movement, and light level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerStealthSignature {
    pub posture: Posture,
    pub movement: Movement,
    /// Ambient light level at player position, [0, 1].
    pub light_level: Scalar,
}

/// Component: per-guard stealth state and awareness meter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GuardStealth {
    pub state: StealthState,
    /// Awareness accumulator, [0, config.awareness_max].
    pub awareness: Scalar,
    /// Whether this guard can trigger alarms.
    pub can_trigger_alarm: bool,
    /// Time since last saw player, for investigate decay.
    pub time_since_last_visual: Scalar,
}

/// Component: guard vision parameters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GuardVision {
    /// Forward direction (normalized).
    pub forward: Vec3,
    /// Max vision distance.
    pub max_distance: Scalar,
    /// Cosine of half FOV angle (precomputed).
    pub cos_fov_half: Scalar,
    /// Base detectability constant B_base.
    pub base_detectability: Scalar,
}

/// Room adjacency: a symmetric relation for "potentially visible rooms".
/// In a full engine, this would likely be a resource, not a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAdjacency {
    /// Adjacency matrix indexed by RoomId. Rooms are assumed dense small IDs.
    /// adjacency[i][j] == true means rooms i and j are connected.
    pub adjacency: Vec<Vec<bool>>,
}

impl RoomAdjacency {
    pub fn is_potentially_visible(&self, a: RoomId, b: RoomId) -> bool {
        let ia = a.0 as usize;
        let ib = b.0 as usize;
        if ia >= self.adjacency.len() || ib >= self.adjacency.len() {
            return false;
        }
        self.adjacency[ia][ib]
    }
}

/// A single sound event emitted this tick.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SoundEvent {
    pub position: Vec3,
    /// Radius within which guards can hear this sound.
    pub radius: Scalar,
    /// Strength coefficient.
    pub strength: Scalar,
}

/// World resource: collection of transient sound events for current tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEvents {
    pub events: Vec<SoundEvent>,
}

/// Tunable stealth configuration; loaded from TOML/JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    // Distance falloff parameters.
    pub max_vision_distance: Scalar,
    pub vision_distance_gamma: Scalar,

    // Light / posture / movement multipliers.
    pub light_gamma: Scalar,
    pub posture_standing: Scalar,
    pub posture_crouched: Scalar,
    pub posture_prone: Scalar,
    pub movement_still: Scalar,
    pub movement_walk: Scalar,
    pub movement_run: Scalar,

    // Awareness parameters.
    pub awareness_max: Scalar,
    pub awareness_decay_per_sec: Scalar,
    pub threshold_suspicious: Scalar,
    pub threshold_alert: Scalar,
    pub hysteresis_suspicious: Scalar,

    // Timeouts for state transitions (e.g., investigate -> patrol).
    pub investigate_timeout_sec: Scalar,
}

/// Mission events emitted by stealth system; another system will consume them.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StealthMissionEvent {
    AlarmRaised,
    PlayerDetected,
    CombatEngaged,
}

/// Resource: per-tick mission events from stealth system.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StealthMissionEvents {
    pub events: Vec<StealthMissionEvent>,
}

/// Evaluate distance falloff f_d(d).
fn distance_falloff(d: Scalar, cfg: &StealthConfig) -> Scalar {
    if d >= cfg.max_vision_distance || cfg.max_vision_distance <= 0.0 {
        0.0
    } else {
        let ratio = d / cfg.max_vision_distance;
        let pow = cfg.vision_distance_gamma.max(0.0001);
        let val = 1.0 - ratio.powf(pow);
        if val < 0.0 { 0.0 } else if val > 1.0 { 1.0 } else { val }
    }
}

/// Light factor f_L(L).
fn light_factor(light: Scalar, cfg: &StealthConfig) -> Scalar {
    let l = if light < 0.0 { 0.0 } else if light > 1.0 { 1.0 } else { light };
    let gamma = cfg.light_gamma.max(0.0001);
    l.powf(gamma)
}

/// Posture factor f_P(P).
fn posture_factor(posture: Posture, cfg: &StealthConfig) -> Scalar {
    match posture {
        Posture::Standing => cfg.posture_standing,
        Posture::Crouched => cfg.posture_crouched,
        Posture::Prone => cfg.posture_prone,
    }
}

/// Movement factor f_M(M).
fn movement_factor(movement: Movement, cfg: &StealthConfig) -> Scalar {
    match movement {
        Movement::Still => cfg.movement_still,
        Movement::Walk => cfg.movement_walk,
        Movement::Run => cfg.movement_run,
    }
}

/// Compute vision-derived visibility score V for a given guard vs player.
/// Returns 0.0 if player is outside FOV, range, or room visibility.
fn compute_visibility_score(
    guard_pos: Vec3,
    guard_room: RoomId,
    vision: &GuardVision,
    player_pos: Vec3,
    player_room: RoomId,
    player_sig: &PlayerStealthSignature,
    rooms: &RoomAdjacency,
    cfg: &StealthConfig,
) -> Scalar {
    // Room-level visibility check.
    if !rooms.is_potentially_visible(guard_room, player_room) {
        return 0.0;
    }

    let to_player = player_pos.sub(guard_pos);
    let distance = to_player.length();
    if distance <= 0.0 || distance > vision.max_distance {
        return 0.0;
    }

    let dir = to_player.normalize();
    let cos_theta = vision.forward.dot(dir);

    if cos_theta < vision.cos_fov_half {
        return 0.0;
    }

    let fd = distance_falloff(distance, cfg);
    if fd <= 0.0 {
        return 0.0;
    }

    let fl = light_factor(player_sig.light_level, cfg);
    let fp = posture_factor(player_sig.posture, cfg);
    let fm = movement_factor(player_sig.movement, cfg);

    let mut v = vision.base_detectability * fd * fl * fp * fm;
    if v < 0.0 {
        v = 0.0;
    }
    v
}

/// Compute total hearing contribution H for a guard from all sound events.
fn compute_hearing_score(
    guard_pos: Vec3,
    guard_room: RoomId,
    rooms: &RoomAdjacency,
    sounds: &SoundEvents,
) -> Scalar {
    let mut total = 0.0;
    for ev in &sounds.events {
        // Simple room constraint: same or adjacent room only.
        // In a more advanced model, sound events would have their own room IDs.
        let same_room = true; // Placeholder: assume omni-room until room-aware sound is added.
        if !same_room {
            continue;
        }

        let to_sound = ev.position.sub(guard_pos);
        let distance = to_sound.length();
        if distance > ev.radius || ev.radius <= 0.0 {
            continue;
        }

        let ratio = distance / ev.radius;
        let val = ev.strength * (1.0 - ratio.max(0.0).min(1.0));
        total += val;
    }
    if total < 0.0 {
        0.0
    } else {
        total
    }
}

/// The main stealth update system.
/// - `delta_seconds`: fixed timestep.
/// - Updates GuardStealth components and emits StealthMissionEvents.
pub fn update_stealth_system(
    delta_seconds: Scalar,
    cfg: &StealthConfig,
    rooms: &RoomAdjacency,
    sounds: &SoundEvents,
    player_transform: &StealthTransform,
    player_sig: &PlayerStealthSignature,
    guards: &mut [(&mut GuardStealth, &GuardVision, &StealthTransform)],
    mission_events: &mut StealthMissionEvents,
) {
    // Clear events; each frame, we append fresh ones.
    mission_events.events.clear();

    let mut any_player_detected = false;
    let mut any_alarm_raised = false;
    let mut any_combat_engaged = false;

    for (guard_stealth, guard_vis, guard_xform) in guards.iter_mut() {
        // Compute visibility and hearing scores.
        let v = compute_visibility_score(
            guard_xform.position,
            guard_xform.room,
            guard_vis,
            player_transform.position,
            player_transform.room,
            player_sig,
            rooms,
            cfg,
        );

        let h = compute_hearing_score(guard_xform.position, guard_xform.room, rooms, sounds);

        // Integrate awareness.
        let delta_awareness = (v + h) * delta_seconds - cfg.awareness_decay_per_sec * delta_seconds;
        let mut awareness = guard_stealth.awareness + delta_awareness;
        if awareness < 0.0 {
            awareness = 0.0;
        }
        if awareness > cfg.awareness_max {
            awareness = cfg.awareness_max;
        }

        guard_stealth.awareness = awareness;

        // Track visual contact time.
        if v > 0.0 {
            guard_stealth.time_since_last_visual = 0.0;
        } else {
            guard_stealth.time_since_last_visual += delta_seconds;
        }

        // Determine new state based on thresholds and hysteresis.
        let old_state = guard_stealth.state;
        let new_state = next_state(old_state, awareness, v > 0.0, cfg, guard_stealth.time_since_last_visual);

        guard_stealth.state = new_state;

        // Emit mission events on transitions.
        if (old_state == StealthState::Idle || old_state == StealthState::Patrol)
            && (new_state == StealthState::Alert || new_state == StealthState::Attack)
        {
            any_player_detected = true;
            if guard_stealth.can_trigger_alarm {
                any_alarm_raised = true;
            }
            any_combat_engaged = true;
        } else if (old_state == StealthState::Suspicious || old_state == StealthState::Investigate)
            && (new_state == StealthState::Alert || new_state == StealthState::Attack)
        {
            any_player_detected = true;
            if guard_stealth.can_trigger_alarm {
                any_alarm_raised = true;
            }
            any_combat_engaged = true;
        }
    }

    if any_player_detected {
        mission_events.events.push(StealthMissionEvent::PlayerDetected);
    }
    if any_alarm_raised {
        mission_events.events.push(StealthMissionEvent::AlarmRaised);
    }
    if any_combat_engaged {
        mission_events.events.push(StealthMissionEvent::CombatEngaged);
    }
}

fn next_state(
    current: StealthState,
    awareness: Scalar,
    has_visual: bool,
    cfg: &StealthConfig,
    time_since_last_visual: Scalar,
) -> StealthState {
    let ts = cfg.threshold_suspicious;
    let ta = cfg.threshold_alert;
    let hs = cfg.hysteresis_suspicious;

    match current {
        StealthState::Idle | StealthState::Patrol => {
            if awareness >= ta || has_visual {
                StealthState::Alert
            } else if awareness >= ts {
                StealthState::Suspicious
            } else {
                current
            }
        }
        StealthState::Suspicious => {
            if awareness >= ta || has_visual {
                StealthState::Alert
            } else if awareness < (ts - hs).max(0.0) {
                StealthState::Patrol
            } else {
                StealthState::Suspicious
            }
        }
        StealthState::Investigate => {
            if awareness >= ta || has_visual {
                StealthState::Alert
            } else if time_since_last_visual >= cfg.investigate_timeout_sec {
                StealthState::Patrol
            } else {
                StealthState::Investigate
            }
        }
        StealthState::Alert | StealthState::Attack => {
            if awareness < ts && !has_visual {
                StealthState::Investigate
            } else {
                current
            }
        }
    }
}
```

This module is deterministic, pure (no global state), and uses only scalar math you can unit‑test easily. It is ready to be wrapped in ECS queries and registered as a system. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

***

## 4. Stealth config TOML schema and example

You asked for machine‑readable JSON/TOML schemas. Here’s a TOML config plus an implied JSON Schema; store it as `config/bondfpscore_stealth.toml` and mirror it with a JSON schema in `schemas/bondfpscore_stealth.schema.json` for CI validation. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

`config/bondfpscore_stealth.toml`:

```toml
# File: config/bondfpscore_stealth.toml

[stealth]
max_vision_distance      = 25.0
vision_distance_gamma    = 1.5

light_gamma              = 1.2

posture_standing         = 1.0
posture_crouched         = 0.6
posture_prone            = 0.3

movement_still           = 0.5
movement_walk            = 1.0
movement_run             = 1.5

awareness_max            = 100.0
awareness_decay_per_sec  = 2.0
threshold_suspicious     = 20.0
threshold_alert          = 60.0
hysteresis_suspicious    = 5.0

investigate_timeout_sec  = 5.0
```

You can load this into `StealthConfig` via `toml` crate and expose an engine‑side API like `Engine.SetStealthParams(cfg)` in Lua, mirroring how you load combat schemas for the arena shooter. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

***

## 5. Unit tests for determinism and correctness

Finally, add unit tests that (a) check formulas, (b) enforce determinism over multiple runs, and (c) validate threshold behavior. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

`crates/bondfpscore/src/stealth_ai_tests.rs`:

```rust
// File: crates/bondfpscore/src/stealth_ai_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> StealthConfig {
        StealthConfig {
            max_vision_distance: 25.0,
            vision_distance_gamma: 1.5,
            light_gamma: 1.2,
            posture_standing: 1.0,
            posture_crouched: 0.6,
            posture_prone: 0.3,
            movement_still: 0.5,
            movement_walk: 1.0,
            movement_run: 1.5,
            awareness_max: 100.0,
            awareness_decay_per_sec: 2.0,
            threshold_suspicious: 20.0,
            threshold_alert: 60.0,
            hysteresis_suspicious: 5.0,
            investigate_timeout_sec: 5.0,
        }
    }

    fn simple_rooms() -> RoomAdjacency {
        RoomAdjacency {
            adjacency: vec![
                vec![true, true],
                vec![true, true],
            ],
        }
    }

    #[test]
    fn visibility_zero_outside_fov_or_room() {
        let cfg = default_cfg();
        let rooms = simple_rooms();
        let player_sig = PlayerStealthSignature {
            posture: Posture::Standing,
            movement: Movement::Still,
            light_level: 1.0,
        };

        let guard_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let player_pos = Vec3 { x: 100.0, y: 0.0, z: 0.0 };

        let vis = GuardVision {
            forward: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            max_distance: 25.0,
            cos_fov_half: (45.0f32.to_radians()).cos(),
            base_detectability: 1.0,
        };

        let v = compute_visibility_score(
            guard_pos,
            RoomId(0),
            &vis,
            player_pos,
            RoomId(0),
            &player_sig,
            &rooms,
            &cfg,
        );
        assert_eq!(v, 0.0);
    }

    #[test]
    fn awareness_accumulates_and_triggers_alert() {
        let cfg = default_cfg();
        let rooms = simple_rooms();
        let sounds = SoundEvents { events: Vec::new() };

        let player_xform = StealthTransform {
            position: Vec3 { x: 5.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };
        let player_sig = PlayerStealthSignature {
            posture: Posture::Standing,
            movement: Movement::Walk,
            light_level: 1.0,
        };

        let mut guard_stealth = GuardStealth {
            state: StealthState::Patrol,
            awareness: 0.0,
            can_trigger_alarm: true,
            time_since_last_visual: 0.0,
        };
        let guard_vis = GuardVision {
            forward: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            max_distance: 25.0,
            cos_fov_half: (60.0f32.to_radians()).cos(),
            base_detectability: 1.0,
        };
        let guard_xform = StealthTransform {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };

        let mut events = StealthMissionEvents::default();

        // Simulate a few seconds of the player in plain sight.
        let mut guards: Vec<(&mut GuardStealth, &GuardVision, &StealthTransform)> =
            vec![(&mut guard_stealth, &guard_vis, &guard_xform)];

        for _ in 0..120 {
            update_stealth_system(
                1.0 / 30.0,
                &cfg,
                &rooms,
                &sounds,
                &player_xform,
                &player_sig,
                &mut guards,
                &mut events,
            );
        }

        assert!(
            guard_stealth.awareness >= cfg.threshold_alert,
            "guard should reach alert threshold"
        );
        assert!(
            matches!(guard_stealth.state, StealthState::Alert | StealthState::Attack),
            "guard should be in alert or attack state"
        );
        assert!(
            events.events.iter().any(|e| matches!(e, StealthMissionEvent::AlarmRaised)),
            "alarm should be raised at least once"
        );
    }

    #[test]
    fn deterministic_update_for_same_inputs() {
        let cfg = default_cfg();
        let rooms = simple_rooms();
        let sounds = SoundEvents { events: Vec::new() };

        let player_xform = StealthTransform {
            position: Vec3 { x: 5.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };
        let player_sig = PlayerStealthSignature {
            posture: Posture::Standing,
            movement: Movement::Walk,
            light_level: 1.0,
        };

        let base_guard_stealth = GuardStealth {
            state: StealthState::Patrol,
            awareness: 0.0,
            can_trigger_alarm: true,
            time_since_last_visual: 0.0,
        };
        let guard_vis = GuardVision {
            forward: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
            max_distance: 25.0,
            cos_fov_half: (60.0f32.to_radians()).cos(),
            base_detectability: 1.0,
        };
        let guard_xform = StealthTransform {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            room: RoomId(0),
        };

        let mut seq1 = base_guard_stealth;
        let mut seq2 = base_guard_stealth;

        let mut events1 = StealthMissionEvents::default();
        let mut events2 = StealthMissionEvents::default();

        // Run two independent simulations with identical inputs.
        let mut guards1: Vec<(&mut GuardStealth, &GuardVision, &StealthTransform)> =
            vec![(&mut seq1, &guard_vis, &guard_xform)];
        let mut guards2: Vec<(&mut GuardStealth, &GuardVision, &StealthTransform)> =
            vec![(&mut seq2, &guard_vis, &guard_xform)];

        for _ in 0..180 {
            update_stealth_system(
                1.0 / 30.0,
                &cfg,
                &rooms,
                &sounds,
                &player_xform,
                &player_sig,
                &mut guards1,
                &mut events1,
            );
            update_stealth_system(
                1.0 / 30.0,
                &cfg,
                &rooms,
                &sounds,
                &player_xform,
                &player_sig,
                &mut guards2,
                &mut events2,
            );
        }

        assert!(
            (seq1.awareness - seq2.awareness).abs() <= 0.0001,
            "awareness should be identical for deterministic simulation"
        );
        assert_eq!(seq1.state, seq2.state, "states should match");
    }
}
```

These tests give you an automated determinism harness for the stealth slice, mirroring what you already planned for the arena shooter. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

***

## 6. Knowledge graph registration

Finally, add a SystemNode entry so AI‑Chat and tools can find this module and its schema. Extend `knowledgegraph/systems.json` (or a Bond‑specific KG file) with: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

```json
{
  "id": "systems.bondfpscore.stealth_ai",
  "title": "Bond/PD-Style Stealth AI",
  "description": "Deterministic stealth perception and awareness model (vision, sound, thresholds) for GoldenEye/Perfect Dark-style missions, with mission event hooks.",
  "files": [
    "crates/bondfpscore/src/stealth_ai.rs",
    "crates/bondfpscore/src/stealth_ai_tests.rs",
    "config/bondfpscore_stealth.toml",
    "schemas/bondfpscore_stealth.schema.json"
  ],
  "tags": [
    "BondFPSCore",
    "Stealth",
    "Deterministic",
    "LuaFacing",
    "ConfigSchema"
  ],
  "related": [
    "systems.bondfpscore.movement",
    "systems.bondfpscore.weapons",
    "systems.mission.objective_dag"
  ]
}
```

This ties the stealth module into your KG so AI‑Chat can navigate from “Bond stealth” to concrete files, schemas, and mission hooks. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

***

## 7. Next research actions

To push this toward a complete N64/PS1‑era reference:

1. **Refine room/portal logic to match STAN/PAD**

   - Add a small “room graph + portals” resource, and study GoldenEye/Perfect Dark STAN docs to approximate their visibility cells more closely (e.g., limited depth search, door state gating). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)
   - Instrument simple test maps where you flip door states and verify guards lose sight/hearing correctly.

2. **Calibrate parameters against real GoldenEye/PD behavior**

   - Use recorded playthroughs and modding docs (e.g., TND 64 analysis) to tune `B_base`, falloff, and thresholds so detection timing feels faithful on similar distances and lighting. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)
   - Encode these as named presets in TOML: `stealth_profile.easy/normal/00_agent`.

3. **Integrate mission DAG and narrative hooks**

   - Wire `StealthMissionEvents` into your objective DAG evaluation, then add PD‑style conditional objectives (“no alarms”, “no casualties”) and validate in unit tests that certain scripted setups succeed/fail as expected. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)
   - Add KG nodes linking `systems.bondfpscore.stealth_ai` to `systems.narrative.perfectdarklike` so AI‑Chat can follow the chain from stealth into branching missions. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

4. **Expose to Lua and JSON schemas**

   - Add Lua bindings (via `mlua`) that let mission scripts query guard states, awareness, and subscribe to mission events, without breaking determinism (all state remains in Rust). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)
   - Formalize `bondfpscore_stealth.schema.json` using `schemars` so CI can reject bad configs before runtime, and so AI‑Chat can auto‑generate valid parameter blocks. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

5. **Cross‑module reuse**

   - Once stable, treat this stealth module as a template and clone its pattern for lock‑on targeting, damage/armor curves, and objective DAGs: “math + config schema + deterministic Rust + tests + KG node.” [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)
   - Use the KG to index these patterns under canonical IDs (e.g., `systems.shared.lockon`, `systems.shared.damage_model`) so future projects and AI‑Chat can reuse them consistently. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
