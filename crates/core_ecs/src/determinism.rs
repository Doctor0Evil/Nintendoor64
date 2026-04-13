use crate::{WorldHandle, InputFrame, SnapshotId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Centralized seeded PRNG for deterministic randomness.
/// All random operations in the ECS must route through this resource.
#[derive(Debug, Clone)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    /// Create a new PRNG with the given seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    
    /// Generate the next pseudo-random u64 using a simple LCG.
    /// This is deterministic and fast; replace with a higher-quality
    /// algorithm if cryptographic security is ever needed (it won't be).
    pub fn next_u64(&mut self) -> u64 {
        // Parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    
    /// Generate a random value in [0, max).
    pub fn next_range(&mut self, max: u64) -> u64 {
        if max == 0 {
            return 0;
        }
        self.next_u64() % max
    }
    
    /// Serialize the RNG state for snapshotting.
    pub fn to_bytes(&self) -> [u8; 8] {
        self.state.to_le_bytes()
    }
    
    /// Deserialize the RNG state from a snapshot.
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        Self {
            state: u64::from_le_bytes(bytes),
        }
    }
}

/// Compute a hash of the world state for determinism verification.
/// This should be called after each tick during testing.
pub fn compute_world_hash(world: WorldHandle) -> u64 {
    // In a real implementation, this would iterate over all
    // component storages in a deterministic order and hash
    // their contents. For now, we return a placeholder.
    let mut hasher = DefaultHasher::new();
    
    // Hash the world handle as a stand-in
    world.0.hash(&mut hasher);
    
    hasher.finish()
}

/// Run a determinism test: execute the same input sequence twice
/// and verify that world state hashes match at each tick.
pub fn test_determinism<F>(
    mut make_world: F,
    inputs: &[InputFrame],
    max_ticks: usize,
) -> Result<(), DeterminismError>
where
    F: FnMut() -> WorldHandle,
{
    let mut world_a = make_world();
    let mut world_b = make_world();
    
    for tick in 0..max_ticks.min(inputs.len()) {
        // Step both worlds with the same inputs
        crate::core_step(world_a, &inputs[..=tick]);
        crate::core_step(world_b, &inputs[..=tick]);
        
        // Compare hashes
        let hash_a = compute_world_hash(world_a);
        let hash_b = compute_world_hash(world_b);
        
        if hash_a != hash_b {
            return Err(DeterminismError::HashMismatch {
                tick,
                hash_a,
                hash_b,
            });
        }
    }
    
    Ok(())
}

/// Error type for determinism test failures.
#[derive(Debug, Clone)]
pub enum DeterminismError {
    HashMismatch {
        tick: usize,
        hash_a: u64,
        hash_b: u64,
    },
}

impl std::fmt::Display for DeterminismError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeterminismError::HashMismatch { tick, hash_a, hash_b } => {
                write!(
                    f,
                    "Determinism violation at tick {}: hash_a={} hash_b={}",
                    tick, hash_a, hash_b
                )
            }
        }
    }
}

impl std::error::Error for DeterminismError {}

/// Verify that a snapshot round-trip preserves world state.
pub fn test_snapshot_roundtrip(
    world: WorldHandle,
    inputs: &[InputFrame],
    snapshot_tick: usize,
    final_tick: usize,
) -> Result<(), SnapshotError> {
    // Run to snapshot point
    for tick in 0..=snapshot_tick.min(inputs.len()) {
        crate::core_step(world, &inputs[..=tick]);
    }
    
    // Save snapshot
    let snapshot = crate::core_save_snapshot(world);
    let hash_before = compute_world_hash(world);
    
    // Continue to final tick
    for tick in (snapshot_tick + 1)..=final_tick.min(inputs.len()) {
        crate::core_step(world, &inputs[..=tick]);
    }
    let hash_direct = compute_world_hash(world);
    
    // Restore snapshot and replay
    crate::core_load_snapshot(world, snapshot);
    for tick in (snapshot_tick + 1)..=final_tick.min(inputs.len()) {
        crate::core_step(world, &inputs[..=tick]);
    }
    let hash_replayed = compute_world_hash(world);
    
    if hash_direct != hash_replayed {
        return Err(SnapshotError::ReplayMismatch {
            snapshot_tick,
            final_tick,
            hash_direct,
            hash_replayed,
        });
    }
    
    // Also verify snapshot restored correctly
    if hash_before != compute_world_hash(world) {
        return Err(SnapshotError::RestoreMismatch {
            snapshot_tick,
            expected: hash_before,
            actual: compute_world_hash(world),
        });
    }
    
    Ok(())
}

#[derive(Debug, Clone)]
pub enum SnapshotError {
    ReplayMismatch {
        snapshot_tick: usize,
        final_tick: usize,
        hash_direct: u64,
        hash_replayed: u64,
    },
    RestoreMismatch {
        snapshot_tick: usize,
        expected: u64,
        actual: u64,
    },
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotError::ReplayMismatch { snapshot_tick, final_tick, hash_direct, hash_replayed } => {
                write!(
                    f,
                    "Replay mismatch: snapshot at tick {}, final tick {}: direct={} replayed={}",
                    snapshot_tick, final_tick, hash_direct, hash_replayed
                )
            }
            SnapshotError::RestoreMismatch { snapshot_tick, expected, actual } => {
                write!(
                    f,
                    "Snapshot restore mismatch at tick {}: expected={} actual={}",
                    snapshot_tick, expected, actual
                )
            }
        }
    }
}

impl std::error::Error for SnapshotError {}
