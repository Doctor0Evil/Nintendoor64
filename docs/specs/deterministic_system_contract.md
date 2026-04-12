# Deterministic System Contract

This document defines rules for systems that must remain deterministic under all supported platforms and builds.

## Deterministic System Definition

A system is deterministic if, given the same initial ECS state and the same sequence of inputs, it always produces the same sequence of world states, regardless of scheduling and hardware.

Deterministic systems must:

- Not call nondeterministic APIs:
  - No wall-clock time.
  - No file IO.
  - No network IO.
- Not use ad-hoc randomness:
  - No direct use of thread-local RNGs or global RNGs.
  - All randomness must be derived from a simulation RNG stored in ECS state.
- Not depend on nondeterministic iteration order:
  - No maps or sets with implementation-defined order in critical logic.
  - If maps are used, iteration order must be made deterministic (e.g., sorted keys).

## System Metadata

Each gameplay system has a system contract:

- id: unique string, e.g. "systems.bondfpscore.stealth_ai"
- deterministic: boolean
- allowed_side_effects: list of allowed effect types (usually empty)
- uses_rng: boolean (must be false for deterministic systems)
- input_components: list of component types (read-only)
- output_components: list of component types (read-write)

This metadata is stored in a machine-readable file so tools and AI can enforce rules.
