# Sonia AI Model – Next Directions and Alternate Build Sequences

This document extends the 20‑file Sonia AI Model blueprint with additional objectives and alternate implementation sequences. It is meant to guide codegen, CI wiring, and AI‑Chat orchestration for the Nintendoor64 workspace while staying inside the existing Sonia file scope.[file:1][file:2]

## 0. Overview and goals

The Sonia slice is organized into five groups:

- Group A – CLI / Protocol / Schema Generation
- Group B – Invariants and Determinism
- Group C – Knowledge Graph and Feature Index
- Group D – AI Conditioning and Session Intelligence
- Group E – N64 Vertical Slice and CI Orchestration

The core goal is to complete a “contract‑hard” N64 vertical slice while simultaneously shaping each file into an AI‑friendly surface that can later be reused for NES, SNES, PS1, and modern engine backends.[file:1][file:2]

---

## 1. Group A – CLI / Protocol / Schema Generation

**Files**

1. `crates/sonia-core/src/cli.rs`  
2. `crates/sonia-featurelayout/src/cli.rs`  
3. `crates/gamemodeai-session/src/cli.rs`  
4. `protocols/sonia-json-rpc.toml`  
5. `tools/schema-gen/main.rs`[file:1]

### 1.1 Next objectives per file

#### 1.1.1 `sonia-core/src/cli.rs`

**Objective**

Evolve from a simple artifact CLI into an AI‑navigable orchestration terminal with capability discovery and macro commands, tagged by retro platform and artifact kind.[file:1]

**Next steps**

- Add a `describe` subcommand returning JSON:

  - Command name and description.
  - Parameter schema ID and result schema ID.
  - Platform affinities (`NES`, `SNES`, `N64`, `PS1`).
  - Artifact kinds touched (`ROMPatch`, `CHRBank`, `LuaScript`, `ScenarioSpec`).[file:1]

- Implement AI‑exclusive macro verbs:

  - `plan-artifact-batch` taking a JSON list of `ArtifactSpec` and running `validate` + `write` as a single transaction.
  - `simulate-checklist` that runs `ai_checklist` in dry‑run mode over a proposed bundle without touching disk.[file:1]

- Ensure every subcommand maps back to a JSON‑RPC envelope defined in `sonia-json-rpc.toml` so AI can introspect capabilities without hard‑coding them.[file:1]

#### 1.1.2 `sonia-featurelayout/src/cli.rs`

**Objective**

Turn FeatureLayout into a retro “menu” of powers with richer tagging and basic ranking.[file:2]

**Next steps**

- Extend CLI with:

  - Multi‑tag queries (`list-by-tags`) supporting AND/OR semantics and filters for platform, genre, role, stability tier.
  - Lightweight ranking fields in responses (`match_score`, `stability_tier`) produced via `tag_algebra.rs`.[file:1][file:2]

- Add an AI‑only `suggest-tags` verb:

  - Input: natural language intent string.
  - Output: proposed tag sets and candidate `FeatureEntry.id` to query next.[file:2]

#### 1.1.3 `gamemodeai-session/src/cli.rs`

**Objective**

Make SessionProfile the single place where “retro build tempo” and console constraints live, and expose it as JSON for AI.[file:1]

**Next steps**

- Extend SessionProfile with per‑platform invariants:

  - `n64_rom_ceiling`, `n64_budget_profile`
  - `ps1_iso_budget`
  - `allow_non_deterministic_experiments`.[file:1]

- Ensure `getsession` always returns:

  - Active invariants.
  - Summarized CI degradation signals (`n64_budget_failing`, `stealth_tests_unstable`).[file:1]

- Add a `compute-mode` field (`fix_only`, `explore_layouts`, `tune_stealth`) and enforce it:

  - CLI refuses session updates or Sonia commands that violate the current mode and CI health.[file:1]

#### 1.1.4 `protocols/sonia-json-rpc.toml`

**Objective**

Promote the TOML file to the canonical IDL for all Sonia‑adjacent CLIs and evolve it with retro context.[file:1][file:4]

**Next steps**

- Add headers to the protocol:

  - Optional `platform` (`NES`, `SNES`, `N64`, `PS1`).
  - `session_id`.
  - `min_version` / `max_version` for negotiation.[file:4]

- Standardize retro‑specific error codes:

  - `BudgetOverflowN64`
  - `DeterminismViolationCoreEcs`
  - `UnsafePatchRegion`
  - `SchemaViolationArtifactSpec`.[file:4]

- Document for AI:

  - For each error code, expected retry path (“shrink texture set”, “replace RNG call”, “re‑target mutable segment only”).[file:4]

#### 1.1.5 `tools/schema-gen/main.rs`

**Objective**

Upgrade from simple schema generator to schema governance engine for all retro‑visible types.[file:1][file:4]

**Next steps**

- Track and enforce versioning:

  - Reject breaking changes (removed required fields, removed enum variants) unless schema metadata version is bumped.[file:4]

- Emit a `schemas/index-sonia-changes.json` per run:

  - Summarize what changed per schema since the last CI run (new fields, deprecated fields, added enum variants).
  - This provides AI with a small “schema changelog index”.[file:4]

### 1.2 Alternate build sequences

**Sequence A1 – Protocol‑first**

- Stabilize `sonia-json-rpc.toml` and minimal `describe` support across CLIs.
- Implement stubbed CLIs that parse the envelope and return dummy data.
- Only later attach real schemas and invariants once the envelope is stable.[file:1]

**Sequence A2 – Schema‑first**

- Finish `schema-gen` and generate schemas for `ArtifactSpec`, `FeatureLayout`, `SessionProfile`, `SystemNode`, `RomLayout`, `PatchSpec`.
- Wire CLIs strictly to schemas.
- Promote `sonia-json-rpc.toml` after types settle to reduce early churn.[file:1][file:4]

---

## 2. Group B – Invariants / Determinism

**Files**

6. `crates/sonia-core/src/invariants/determinism.rs`  
7. `crates/sonia-core/src/invariants/hardware_budget.rs`  
8. `crates/sonia-core/src/invariants/abi_guard.rs`  
9. `crates/sonia-core/src/ai_checklist.rs`[file:1]

### 2.1 Next objectives per file

#### 2.1.1 `invariants/determinism.rs`

**Objective**

Attach determinism rules to consoles and emit structured “how to fix” plans.[file:1][file:4]

**Next steps**

- Tag violations per console and semantics:

  - `N64_ECS_NonDeterministic`
  - `PS1_Stream_Nondeterministic`
  - `ReplayUnsafe` or `NetcodeUnsafe`.[file:4]

- For each violation, include:

  - Low‑level detail (forbidden imports, nondeterministic maps).
  - Structured refactor hints: `replace HashMap with IndexMap in crate X`, `inject RNG seed parameter into system Y`.[file:4]

- Define an AI‑exclusive “dry‑run patch plan” format:

  - Determinism checker returns a list of concrete edits (files, lines, change descriptions) that AI may review and choose to apply.[file:4]

#### 2.1.2 `invariants/hardware_budget.rs`

**Objective**

Expose multi‑console budgets as inequality systems with negotiation hooks for AI.[file:2][file:4]

**Next steps**

- Implement per‑platform profiles (`NES`, `SNES`, `N64`, `PS1`) with explicit inequalities:

  - ROM sizes, VRAM pools, CHR banks, audio, CPU cycles.[file:2]

- Compute granular deltas:

  - Example: `textures +4 MB over budget`, `audio −1 MB below ceiling`.[file:2]

- Add a “budget negotiation” mode:

  - Checker returns a small set of candidate tradeoffs (downsample textures, compress audio, reduce mission set) with quantized estimates of savings.[file:2]

#### 2.1.3 `invariants/abi_guard.rs`

**Objective**

Make ABI guard the canonical C‑ABI stability gate across retro engines.[file:1][file:4]

**Next steps**

- Classify C symbols:

  - `Public`, `Internal`, `Experimental` with stability tags.[file:4]

- On `Public` changes:

  - Emit “migration notes” per symbol and generate stub changelog entries for docs.[file:4]

- Add `abi-change-request` flow:

  - AI proposes change with `bump_version` flag.
  - Guard computes impact: which crates, scenes, tests, and generates a migration plan JSON for AI to implement.[file:4]

#### 2.1.4 `ai_checklist.rs`

**Objective**

Turn the checklist into a profile‑based policy engine.[file:1]

**Next steps**

- Define named profiles:

  - `strict_n64_vertical_slice`
  - `fast_ps1_exploration`
  - `retro_ci_release`.[file:1]

- For each profile:

  - Which checks are hard vs soft.
  - Score thresholds that allow writes.
  - Handling of budget violations in experimental branches.[file:1]

- Add `simulated_checklist` entry point:

  - Accepts a candidate artifact bundle, runs full checks, and returns a structured failure matrix without writing to disk.[file:1]

### 2.2 Alternate build sequences

- **B1 – Minimal guardrail first:** focus on schema, size, encoding checks in `ai_checklist` then gradually introduce determinism, budget, ABI.[file:1]
- **B2 – Determinism‑driven:** prioritize `determinism.rs` and its integration into SessionProfile and CI, then add budget and ABI once deterministic ECS core is stable.[file:4]

---

## 3. Group C – KG / Feature Index

**Files**

10. `knowledgegraph/features.sonia.json`  
11. `crates/sonia-featurelayout/src/tag_algebra.rs`  
12. `crates/gamemodeai-kg/src/resolver.rs`  
13. `tools/kg-feature-sync/main.rs`[file:1]

### 3.1 Next objectives per file

#### 3.1.1 `knowledgegraph/features.sonia.json`

**Objective**

Enrich the feature index with full metadata for the ten Nintendoor64 superpowers and prepare for multi‑platform extension.[file:2]

**Next steps**

- For each superpower (ROM Layout Oracle, Safe Patch Synthesizer, Pattern Composer, Build Conductor, Scenario Director, Mechanic Transplant, Visual Diff Analyst, Narrative Cartographer, Budget Planner, Schema Designer, Turn Harness) add:[file:2]

  - Platform tags.
  - Stability tier (`Experimental`, `Stable`, `Deprecated`).
  - Example CLI flows.
  - Recommended AI “modes” (e.g. “use only in fix‑only sessions”).[file:2]

- Extend to NES/SNES/PS1 by adding platform tags and system IDs for each backend.[file:2]

#### 3.1.2 `sonia-featurelayout/src/tag_algebra.rs`

**Objective**

Add metrics and explanations to tag matching.[file:1]

**Next steps**

- Implement tag algebra that returns:

  - Match hit counts.
  - Simple explanatory strings (“matched because tags include `Nintendoor64` and `ArtifactSink`”).[file:1]

- Provide a “query tuning” endpoint:

  - Input: target platform/genre and desired result cardinality.
  - Output: suggested tag additions/removals to get ~N results.[file:1]

#### 3.1.3 `gamemodeai-kg/src/resolver.rs`

**Objective**

Upgrade resolver into a “dependency oracle” for retro systems.[file:1]

**Next steps**

- Add queries:

  - `path-between` for shortest dependency path between two systems.
  - `scope-query` for all systems contributing to a given concern (e.g. N64 ROM budget).
  - Validation that every `FeatureEntry.systems[]` resolves to a SystemNode.[file:1]

- Provide a read‑only “impact slice” API:

  - Given a system or file, return the affected systems, files, and invariants so AI can scope edits and tests.[file:1]

#### 3.1.4 `tools/kg-feature-sync/main.rs`

**Objective**

Make feature sync bidirectional and monotone.[file:1]

**Next steps**

- Generate draft `FeatureEntry` skeletons from new `SystemNode` entries:

  - Infer tags and schemas from existing patterns.
  - Flag missing KG metadata for new features.[file:1]

- Provide a dry‑run “feature delta report” for AI:

  - Summarize proposed feature additions/updates so AI can refine and commit.[file:1]

### 3.2 Alternate build sequences

- **C1 – Hand‑curate then automate:** manually author `features.sonia.json` for core superpowers, then add `tag_algebra` and `kg-feature-sync` later.[file:1]
- **C2 – KG‑driven skeletons:** build `kg-feature-sync` first to bootstrap features from KG, then refine by hand.[file:1]

---

## 4. Group D – AI Conditioning & Session Intelligence

**Files**

14. `session/conditioning.rs`  
15. `session/ci_digest.rs`  
16. `ai_prompts/sonia-system-prompt.md`  
17. `ai_prompts/schema-guard-instructions.md`[file:1]

### 4.1 Next objectives per file

#### 4.1.1 `session/conditioning.rs`

**Objective**

Treat invariants as weighted constraints and score candidate plans.[file:1][file:2]

**Next steps**

- Represent `SessionProfile.invariants` with:

  - Hard vs soft weights.
  - Console‑specific budget and determinism rules.[file:1]

- Implement scoring for candidate bundles:

  - Include domain metrics: ROM budget utilization, stealth difficulty monotonicity, mission DAG constraints.[file:2]

- Expose a “scored preview” API for AI:

  - Given a plan (bundle of `ArtifactSpec`/patches), return a score breakdown before actually running CI.[file:1]

#### 4.1.2 `session/ci_digest.rs`

**Objective**

Normalize CI output into structured, fixable failure types.[file:1]

**Next steps**

- Define a taxonomy:

  - `SchemaViolation`, `BudgetOverflow`, `DeterminismViolation`, `NarrativeDeadEnd`, `ScenarioRegression`, etc.[file:1]

- For each failure, attach “fix hints”:

  - Small suggestion objects pointing at config fields, missions, artifact filenames for minimal edits.[file:1]

#### 4.1.3 `ai_prompts/sonia-system-prompt.md`

**Objective**

Make this a parameterized system prompt template driven by session and KG.[file:1]

**Next steps**

- Inject at runtime:

  - Active invariants and compute‑mode.
  - Available retro features from `features.sonia.json`.
  - Recent CI failures from `ci_digest`.[file:1]

- Encode strong rules:

  - Never emit raw binaries.
  - Always use `ArtifactSpec`.
  - Always consult schemas and feature metadata before generating.
  - Route by platform (e.g. “for N64, use RomLayout + PatchSpec + Sonia + Starzip”).[file:2]

#### 4.1.4 `ai_prompts/schema-guard-instructions.md`

**Objective**

Expand into a catalog of schema failure modes and recovery patterns.[file:1]

**Next steps**

- For each key schema (ArtifactSpec, PatchSpec, RomLayout, BudgetReport, ScenarioSpec, MissionGraph):

  - Provide example (invalid → error → corrected).
  - Document explicit retry steps (inspect schema, re‑query examples, regenerate minimal fix).[file:1]

- Add retro‑specific “don’t do this” patterns:

  - No bypassing RomLayout with raw intervals.
  - No simultaneous widening of stealth cones and budgets.[file:2]

---

## 5. Group E – N64 Vertical Slice & CI

**Files**

18. `n64-layout/src/sonia_bridge.rs`  
19. `.github/workflows/sonia-ai-n64-slice.yml`  
20. `docs/sonia-ai-model.md`[file:1][file:2]

### 5.1 Next objectives per file

#### 5.1.1 `n64-layout/src/sonia_bridge.rs`

**Objective**

Promote bridge to full “patch impact oracle” for N64.[file:2]

**Next steps**

- Extend current implementation to compute:

  - ROM intervals touched.
  - Affected segments and asset classes (textures, audio, scripts).
  - Cumulative per‑segment added bytes.[file:2]

- Expose a “preview patch” mode:

  - Given `PatchSpec` + payload index, return impact regions and projected budget changes without applying the patch.[file:2]

#### 5.1.2 `.github/workflows/sonia-ai-n64-slice.yml`

**Objective**

Make this workflow the canonical pattern for AI‑driven retro pipelines.[file:1][file:2]

**Next steps**

- Keep current steps (schema‑gen, schema validation, sonia‑core validation/writes, Starzip patch, Conk64 smoke test) and add:

  - Determinism checks (where applicable).
  - Budget checks via `starzip-budget`.
  - A CI digest step that writes a compact `CiStatus` and calls `gamemodeai-session update-ci-status`.[file:1][file:2]

- Name lanes explicitly for AI:

  - `n64-safe-patch`
  - `n64-budget-tuning`
  - `n64-scenario-regression`.[file:1]

#### 5.1.3 `docs/sonia-ai-model.md`

**Objective**

Turn this into the human + AI “flight manual” for the 20 Sonia files.[file:1]

**Next steps**

- Add end‑to‑end walkthroughs:

  - N64 safe patching.
  - NES CHR/nametable adjustments.
  - Bond‑style stealth tuning.
  - Narrative graph validation.[file:1][file:2]

- For each walkthrough:

  - List schemas.
  - Show CLI sequences.
  - Describe CI expectations and invariants.[file:1]

---

## 6. Cross‑group alternate build sequences

To keep the plan flexible, you can choose among several macro sequences.

### 6.1 Sequence X – AI surface first

1. Group A CLIs and protocol (Files 1–5) to give AI a callable surface quickly.[file:1]  
2. Group C KG/feature index (Files 10–13) so features are discoverable.[file:1][file:2]  
3. Group D minimal prompts/conditioning (Files 14, 16, 17).  
4. Group E N64 CI slice with some invariants stubbed (Files 18–19).[file:1]  
5. Group B deep invariants (Files 6–9).[file:1]

This is suited for early AI experiments with soft validation that hardens over time.

### 6.2 Sequence Y – Hard‑contract first

1. Schema and invariants:

   - `schema-gen` (File 5).
   - Determinism, budget, ABI, checklist (Files 6–9).[file:1]

2. Session and CI:

   - `gamemodeai-session` + `ci_digest` (Files 3, 15).
   - Full N64 CI workflow (File 19).[file:1]

3. KG and features (Files 10–13).  
4. CLIs and protocol surfaces (Files 1, 2, 4).  
5. AI prompts and docs (Files 14, 16, 17, 18, 20).[file:1]

This sequence maximizes safety and correctness before granting AI broad write access.

### 6.3 Sequence Z – Vertical slice centered

1. N64 infrastructure:

   - `sonia_bridge.rs` (File 18) plus existing `n64-layout` and Starzip crates.[file:2]

2. CI vertical slice:

   - `.github/workflows/sonia-ai-n64-slice.yml` (File 19).[file:1][file:2]

3. Schema and KG:

   - `schema-gen` (File 5) and core schemas.
   - `features.sonia.json` and KG resolver (Files 10, 12).[file:1][file:2]

4. Sonia CLIs (Files 1–3).  
5. Invariants and AI conditioning (Files 6–9, 14–17, 20).[file:1]  
6. Automation and feature sync (Files 11, 13).[file:1]

This sequence gives you a demonstrable N64 path early and then generalizes it.

---

## 7. Research actions to improve KG logic and Rust indexing

Beyond the 20 files, several research actions will sharpen knowledge‑graph behavior and make AI‑Chat navigation over Rust code more precise.[file:4]

1. **Code‑structure introspection**

   - Build a Rust tool that parses crates and attaches symbols (types, functions) to `SystemNode.symbols` in `knowledgegraphsystems.json`.[file:4]
   - Evaluate using rust‑analyzer or tree‑sitter for richer relationships and call graphs.[file:4]

2. **Multi‑repo graphs**

   - Introduce `knowledgegraphrepos.json` and add optional `repo` fields to `SystemNode`.
   - Support queries that resolve directly to GitHub URLs for files and symbols.[file:4]

3. **Determinism‑ and ABI‑aware semantics**

   - Attach invariants and ABI stability levels (`Internal`, `Public`, `Experimental`) as tags or attributes on SystemNodes.
   - Allow AI to query “what rules apply here?” before editing.[file:4]

4. **Advanced navigation queries**

   - Implement `path-between` and `scope-query` in `gamemodeai-kg`.
   - Explore using narrative DAG math (reachability, branching) as inspiration for reasoning over code dependency graphs.[file:4]

5. **Schema governance and JSON tooling**

   - Strengthen schema validation CI jobs for all KG and session JSON.
   - Add migration helpers when schemas change, including versioned SessionProfile upgrades.[file:4]

6. **Indexing performance**

   - Benchmark KG loading and queries on larger workspaces.
   - Prototype a background daemon with persistent indexes (e.g. SQLite or sled) to serve low‑latency graph queries to AI‑Chat tools.[file:4]

These directions are intentionally modular: you can pick Sequence X, Y, or Z and then layer these research tasks as the Sonia slice matures into a full, multi‑platform orchestration layer.
