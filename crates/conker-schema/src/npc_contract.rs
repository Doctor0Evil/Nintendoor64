// Destination: crates/conker-schema/src/npc_contract.rs

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Canonical, deterministic AI “contract” for a Conker NPC type.
///
/// This is intentionally small and explicit:
/// - What damages can kill it.
/// - Which actions it is allowed to take.
/// - How it moves (nav graph vs. simple chasing).
/// - What game-state conditions activate it.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NpcContract {
    /// Stable ID for this NPC family (e.g., "npc.zombie.standard").
    pub id: String,

    /// Short label for designers.
    pub title: String,

    /// High-level NPC category to drive tooling and filters.
    pub kind: NpcKind,

    /// Damageability rules: what counts as a lethal hit, what ignores it.
    pub damageable: Damageable,

    /// Movement contract: how it chooses paths through the world.
    pub movement: MovementContract,

    /// Action contract: allowed actions and their weights per state.
    pub behavior: BehaviorContract,

    /// Spawn and activation rules (zones, objectives, timers).
    pub spawn: SpawnContract,
}

/// Coarse taxonomy for NPC contracts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NpcKind {
    Zombie,
    Alien,
    FireImp,
    Soldier,
    Civilian,
    Other,
}

/// Damage model for an entity that can be killed.
///
/// This is the core of the zombie “only headshots / certain blasts kill” rule.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Damageable {
    /// Maximum hit points for non-lethal damage accumulation.
    /// For “only headshot kills” zombies, this may be large or ignored.
    pub max_hp: f32,

    /// Tags that count as lethal kill channels for this NPC.
    ///
    /// Examples:
    /// - ["headshot"] for graveyard zombies.
    /// - ["headshot", "shotgun"] if you want shotgun blasts to count.
    /// - ["explosive"] for enemies only killed by explosions.
    pub kill_tags: Vec<String>,

    /// Tags that this NPC fully ignores (no stun, no damage).
    ///
    /// Example: ["pistol_bodyshot"] for zombies that soak pistol fire.
    #[serde(default)]
    pub ignore_tags: Vec<String>,

    /// Optional multiplier table for non-lethal damage by tag.
    ///
    /// Example: { "pistol_bodyshot": 0.25, "rifle_bodyshot": 0.5 }.
    #[serde(default)]
    pub damage_multipliers: std::collections::BTreeMap<String, f32>,
}

/// Movement contract: how an NPC navigates the world.
///
/// This is deliberately simple and deterministic—no opaque dynamic navmesh.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MovementContract {
    /// Navigation mode:
    /// - "GROUND_CHASE": simple ground chasing on a grid / navgraph.
    /// - "NAVGRAPH": uses precomputed nav graph with allowed tags.
    /// - "VENT_CEILING": aliens moving along vent/ceiling graph only.
    pub mode: MovementMode,

    /// Optional reference to the nav graph this NPC uses.
    ///
    /// Example: "navgraph.alien_base.ceiling".
    #[serde(default)]
    pub navgraph_ref: Option<String>,

    /// Maximum movement speed in world units per second.
    pub max_speed: f32,

    /// Optional preferred nav tags (e.g., ["vent", "ceiling"] for aliens).
    #[serde(default)]
    pub preferred_nav_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MovementMode {
    GroundChase,
    Navgraph,
    VentCeiling,
}

/// Behavior contract for a simple deterministic controller.
///
/// This is not a full behavior tree runtime; it just defines states and
/// weighted actions the runtime will turn into a small state machine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorContract {
    /// Named states for this NPC (e.g., "Idle", "Chase", "Attack", "Retreat").
    pub states: Vec<AiState>,

    /// ID of the initial state.
    pub initial_state: String,
}

/// Single AI state and its action distribution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiState {
    pub id: String,

    /// Optional reference to an animation-state ID registry (ASID).
    /// Example: "ASID_ALIEN_STALK", "ASID_ZOMBIE_SHAMBLE".
    #[serde(default)]
    pub asid: Option<String>,

    /// Weighted actions that can be chosen while in this state.
    pub actions: Vec<AiActionChoice>,

    /// Transition rules to other states.
    #[serde(default)]
    pub transitions: Vec<AiTransition>,
}

/// Weighted action choice in a given state.
///
/// For aliens, these would include "Stalk", "PounceExecution", "Retreat".
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiActionChoice {
    /// Stable ID, used by the runtime to dispatch to an implementation.
    ///
    /// Examples:
    /// - "action.zombie.shuffle_forward"
    /// - "action.alien.pounce_execution"
    /// - "action.fire_imp.chase"
    pub action_id: String,

    /// Relative weight for random choice; deterministic RNG is implied.
    pub weight: u32,
}

/// Simple state transition definition.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiTransition {
    /// Condition ID understood by the runtime.
    ///
    /// Examples:
    /// - "condition.sees_target"
    /// - "condition.lost_target"
    /// - "condition.target_within_melee"
    /// - "condition.holder_has_blood_vial"
    pub condition_id: String,

    /// Next state ID if the condition is satisfied.
    pub next_state: String,
}

/// Spawn / activation rules for an NPC family.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SpawnContract {
    /// Maximum simultaneous instances this contract allows in a zone.
    pub max_concurrent: u32,

    /// Desired average spawn density per 100 square meters or per grid chunk.
    pub spawn_density: f32,

    /// Optional list of zone IDs where this NPC can spawn.
    ///
    /// Example: ["zone.spooky_cemetery", "zone.batula_basement"].
    #[serde(default)]
    pub allowed_zones: Vec<String>,

    /// Game logic conditions that must be true before this NPC can be active.
    ///
    /// For Fire Imps, this might require a specific objective being active
    /// (e.g., a player carrying a blood vial).
    #[serde(default)]
    pub activation_conditions: Vec<ActivationCondition>,

    /// Respawn rules for this NPC.
    #[serde(default)]
    pub respawn: RespawnRule,
}

/// A single activation condition, tied to game state.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivationCondition {
    /// Condition ID understood by the game logic layer.
    ///
    /// Examples:
    /// - "objective.active.blood_vial_carrier"
    /// - "round.phase == sudden_death"
    pub condition_id: String,

    /// Optional human-readable note for designers.
    #[serde(default)]
    pub description: Option<String>,
}

/// Respawn behavior for an NPC family.
///
/// Fire Imp equivalents would have a long cooldown to give players breathing room.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RespawnRule {
    /// If false, NPCs of this contract do not respawn once killed.
    #[serde(default = "default_respawn_enabled")]
    pub enabled: bool,

    /// Minimum cooldown (seconds) before a new instance can spawn in the same zone.
    #[serde(default = "default_respawn_cooldown")]
    pub cooldown_seconds: f32,
}

fn default_respawn_enabled() -> bool {
    true
}

fn default_respawn_cooldown() -> f32 {
    10.0
}
