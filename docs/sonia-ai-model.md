# Sonia AI Model: Orchestration Framework for Deterministic Game Development

## Overview

Sonia is the AI-facing orchestration layer for GAMEMODE.ai, providing a schema-first, CLI-centric interface for artifact management, feature discovery, and session-aware development. It serves as the canonical bridge between AI-generated content and the deterministic Rust ECS core.

## Core Principles

1. **Schema-First Contracts**: All data exchanged with AI must conform to JSON Schema definitions generated from Rust types via `schemars`.

2. **CLI-Centric Orchestration**: All operations flow through well-defined, idempotent CLI commands with JSON-in/JSON-out protocols.

3. **Determinism by Default**: No non-deterministic operations (random, IO, time) are permitted in core simulation paths.

4. **Context-Aware Sessions**: AI behavior is gated by session profiles containing active invariants, CI status, and project state.

5. **Knowledge Graph Integration**: Features, systems, and schemas are indexed in a machine-readable graph for AI navigation.

## Crate Architecture

### sonia-core
- **Purpose**: Canonical artifact sink and validation layer
- **Key Types**: `ArtifactSpec`, `SessionProfile`, `InvariantRule`
- **CLI Commands**:
  - `validate --spec <path>`: Validate artifact against schema and semantic rules
  - `write --spec <path>`: Decode and write artifact to `artifacts/<kind>/`
  - `list --kind <type>`: Query existing artifacts by type
  - `get-session`: Retrieve current development context
  - `update-session --profile <path>`: Update session with new invariants or CI status

### sonia-featurelayout
- **Purpose**: AI navigation surface for discovering platform capabilities
- **Key Types**: `FeatureLayout`, `FeatureEntry`
- **CLI Commands**:
  - `list-by-tag <tag>`: Return features matching semantic tag
  - `get <id>`: Return full details for a specific feature
  - `list-commands <id>`: Return recommended CLI invocations for a feature

### gamemodeai-kg
- **Purpose**: Knowledge graph management and query interface
- **Key Types**: `SystemNode`, `Tag`
- **CLI Commands**:
  - `getsystem <id>`: Retrieve system metadata
  - `listsystemsbytag <tag>`: List systems by semantic role
  - `listdependents <id>`: Find systems that depend on a given system

### gamemodeai-session
- **Purpose**: Session profile management and CI integration
- **Key Types**: `SessionProfile`, `CiFailure`, `InvariantRule`
- **CLI Commands**:
  - `getsession`: Dump current session state
  - `updatesession --merge <json>`: Apply partial updates to session
  - `updatecistatus --digest <json>`: Ingest CI results into session

## Data Contracts

### ArtifactSpec Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ArtifactSpec",
  "type": "object",
  "required": ["kind", "filename", "encoding", "content"],
  "properties": {
    "kind": {
      "type": "string",
      "enum": [
        "N64RomPatch", "Ps1IsoPatch", "LuaScript",
        "InputMapperConfig", "ScenarioSpec", "NarrativeGraph", "Other"
      ]
    },
    "filename": {
      "type": "string",
      "pattern": "^[a-zA-Z0-9_./-]+$",
      "description": "Repository-relative path under artifacts/"
    },
    "encoding": {
      "type": "string",
      "enum": ["Text", "Hex", "Base64"]
    },
    "content": {
      "type": "string",
      "description": "Encoded payload; raw binary forbidden"
    },
    "metadata": {
      "type": "object",
      "properties": {
        "base_rom_id": {"type": "string"},
        "source_recipe_id": {"type": "string"},
        "size_bytes": {"type": "integer", "minimum": 0}
      }
    }
  }
}
```

### FeatureEntry Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureEntry",
  "type": "object",
  "required": ["id", "title", "tags"],
  "properties": {
    "id": {"type": "string", "pattern": "^[a-z.]+[a-z0-9_-]*$"},
    "title": {"type": "string"},
    "description": {"type": "string"},
    "tags": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "Nintendoor64", "Starzip", "Sonia", "Conk64",
          "BondFPS", "RetroNES", "Deterministic",
          "PatchSynthesizer", "ScenarioDirector", "NarrativeCartographer",
          "BudgetPlanner", "SchemaDesigner", "BinarySafe"
        ]
      }
    },
    "systems": {
      "type": "array",
      "items": {"type": "string"},
      "description": "SystemNode IDs from knowledge graph"
    },
    "schemas": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Paths to JSON Schema files"
    },
    "examples": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Paths to example JSON/TOML files"
    },
    "commands": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Recommended CLI invocations"
    },
    "roles": {
      "type": "array",
      "items": {"type": "string"},
      "description": "High-level capability labels"
    }
  }
}
```

## AI Interaction Protocol

### Happy Path Workflow
1. AI calls `gamemodeai-session getsession` to load current context
2. AI queries `sonia-featurelayout list-by-tag Nintendoor64` for relevant features
3. AI selects a feature and retrieves its schemas via `sonia-featurelayout get <id>`
4. AI generates `ArtifactSpec` JSON conforming to retrieved schema
5. AI submits artifact via `sonia-core write --spec artifact.json`
6. CI validates artifact against schema and semantic rules
7. Downstream tools (starzip-cli, retro-cli) consume validated artifacts

### Invariant Enforcement
Session profiles contain enforceable rules that gate AI proposals:
```json
{
  "invariants": [
    {
      "rule_id": "deterministic_core_no_rand",
      "description": "No non-deterministic RNG in core_ecs",
      "scope": ["crates/core_ecs"],
      "check": "static_analysis"
    },
    {
      "rule_id": "n64_rom_size_limit",
      "description": "N64 ROM must not exceed 64MB",
      "scope": ["artifacts/patches/n64"],
      "check": "size_ceiling",
      "params": {"max_bytes": 67108864}
    },
    {
      "rule_id": "public_abi_stability",
      "description": "C API exports must maintain backward compatibility",
      "scope": ["crates/c_api"],
      "check": "header_diff"
    }
  ]
}
```

## Integration Points

### With Knowledge Graph
- Each `FeatureEntry.systems` field references `SystemNode` IDs
- `SystemNode` entries include `schemas`, `commands`, and `invariants` metadata
- AI can traverse from feature → system → code location → schema

### With CI Pipeline
- Schema validation job regenerates schemas from Rust types
- Semantic validation job runs `sonia-core validate` on changed specs
- CI digest job parses test failures and updates session profile
- Failed invariants prevent artifact promotion to downstream tools

### With Toolchain
- `starzip-cli` consumes `ArtifactSpec` for N64RomPatch kinds
- `retro-cli` consumes `GameRecipe` artifacts for NES/SNES builds
- `conk64-lua` consumes `LuaScript` artifacts for runtime injection
- All tools validate input against schemas before processing

## Extending Sonia

### Adding a New Artifact Type
1. Extend `ArtifactType` enum in `sonia-core/src/model.rs`
2. Add corresponding directory mapping in `artifacts/`
3. Update `artifact-spec.schema.json` via `cargo run -p schema-gen`
4. Add size/type constraints to validation logic
5. Document usage in `docs/sonia-ai-model.md`

### Adding a New Feature
1. Create `FeatureEntry` in `knowledgegraph/features.sonia.json`
2. Reference existing or new `SystemNode` IDs in `systems` field
3. List relevant schema paths in `schemas` field
4. Provide at least one example in `examples` field
5. Tag appropriately for AI discovery (e.g., `Nintendoor64`, `PatchSynthesizer`)

### Adding a New Invariant
1. Define `InvariantRule` struct with unique `rule_id`
2. Implement validation logic in `sonia-core/src/invariants.rs`
3. Register rule in session profile schema
4. Add CI check that enforces rule on relevant scopes
5. Document rule semantics and failure modes

## Testing Strategy

### Unit Tests
- Schema generation: Verify `schemars` output matches expected JSON Schema
- Validation: Test `sonia-core validate` against valid and invalid specs
- CLI: Test JSON-in/JSON-out protocol for all commands

### Integration Tests
- End-to-end: Artifact generation → validation → tool consumption
- Session flow: Profile update → invariant check → AI proposal rejection
- KG linkage: Feature → SystemNode → code location resolution

### Determinism Tests
- Replay harness: Run fixed input sequence twice, compare state hashes
- Snapshot/rollback: Save state at tick N, restore, reapply inputs, verify equivalence
- Parallel execution: Run systems in different orders, verify identical results

## Security Considerations

- Path sanitization: Reject `..`, absolute paths, or symlinks in `ArtifactSpec.filename`
- Size limits: Enforce per-type ceilings to prevent resource exhaustion
- Encoding validation: Verify Base64/Hex decodes to expected byte length
- Schema versioning: Include `$schema` field in all JSON for forward compatibility
- Audit logging: Log all artifact writes with caller identity and timestamp
