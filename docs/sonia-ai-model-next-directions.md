# Sonia AI Model – Next Directions and Alternate Build Sequences

This document extends the 20‑file Sonia AI Model blueprint with additional directions, alternate build orders, and research objectives. The goal is to keep the blueprint flexible under real project pressures (time, CI complexity, partial implementations) while preserving the schema‑first, invariant‑enforced, CLI‑driven design for Nintendoor64.

The plan is organized around the five existing groups:

- Group A: CLI Protocol & Contract Generation
- Group B: Invariants & Determinism Enforcement
- Group C: Knowledge Graph & Feature Index
- Group D: AI Conditioning & Session Intelligence
- Group E: Nintendoor64 Vertical Slice & CI Orchestration

For each group, this document proposes:

- Alternate sequencing: how to reorder work depending on priorities.
- Additional research objectives: where to deepen the work beyond the first pass.
- Cross‑group couplings: which files benefit from being implemented together.
- Future AI‑exclusive behaviors: how to evolve the AI‑only surface over time.

---

## 1. Group A – CLI Protocol & Contract Generation

**Files**

1. `crates/sonia-core/src/cli.rs`
2. `crates/sonia-featurelayout/src/cli.rs`
3. `crates/gamemodeai-session/src/cli.rs`
4. `protocols/sonia-json-rpc.toml`
5. `tools/schema-gen/main.rs`

### 1.1 Alternate build sequences

**Sequence A1 – Protocol‑first**

If the objective is to stabilize AI↔tool communication early:

1. Start with `protocols/sonia-json-rpc.toml` as the single source of truth for the envelope and error codes.
2. Implement thin CLI shims in:
   - `crates/sonia-core/src/cli.rs`
   - `crates/sonia-featurelayout/src/cli.rs`
   - `crates/gamemodeai-session/src/cli.rs`
   that do little more than:
   - Parse stdin into a JSON‑RPC‑like request.
   - Match on `command`.
   - Return a minimal response with stubbed data or `NotImplemented` errors.
3. Only after the envelope is stable, integrate `tools/schema-gen/main.rs` and wire each CLI to real Rust types and schemas.

This sequence prioritizes protocol stability and early AI integration, even while many commands still return stubs.

**Sequence A2 – Schema‑first**

If you want to guarantee strong contracts before exposing a protocol surface:

1. Implement `tools/schema-gen/main.rs` and wire it to:
   - `ArtifactSpec`
   - `SessionProfile`
   - `FeatureLayout`
   - `SystemNode`
   and any other core types.
2. Run schema‑gen as a local dev tool and CI job until the schemas settle.
3. Only then:
   - Implement `sonia-core` CLI commands against the generated schemas.
   - Implement `sonia-featurelayout` and `gamemodeai-session` CLIs around the same schemas.
4. Use `protocols/sonia-json-rpc.toml` as documentation, not a driver, until you are confident the type layer is stable.

This sequence reduces the risk of having to version the protocol early due to type churn.

**Sequence A3 – Session‑driven first**

If your immediate bottleneck is AI awareness of project state:

1. Implement `crates/gamemodeai-session/src/cli.rs` first, with:
   - `getsession`
   - `updatesession`
   - `updatecistatus`
   wired to `session.schema.json`.
2. Have AI‑Chat and CI adopt `gamemodeai-session` as the core “state server” before introducing `sonia-core` artifacts.
3. Add `sonia-core` and `sonia-featurelayout` later, but always passing `SessionProfile` references into their commands (branch, invariants, CI expectations).

This sequence makes session awareness and invariant enforcement the foundation, and bolts artifacts and features on top.

### 1.2 Additional research objectives

1. **Protocol spec generator**

   - Given `protocols/sonia-json-rpc.toml`, generate:
     - Rust request/response types.
     - Error code enums.
     - Markdown snippets for docs.
   - Long‑term direction: treat the TOML file as an IDL for Sonia‑family CLIs.

2. **Command capability discovery**

   - Add a `describe` or `list_methods` command to each CLI:
     - Returns a list of supported `command` names, parameter schemas, and sample requests.
   - AI feature: automatically introspect capability surfaces at runtime instead of relying only on static docs.

3. **Protocol evolution and negotiation**

   - Extend the envelope with:
     - Optional `min_supported_version` / `max_supported_version`.
   - Enable future tools and models to negotiate a minimal compatible subset of the protocol, useful as CLIs evolve.

---

## 2. Group B – Invariant & Determinism Enforcement

**Files**

6. `crates/sonia-core/src/invariants/determinism.rs`  
7. `crates/sonia-core/src/invariants/hardware_budget.rs`  
8. `crates/sonia-core/src/invariants/abi_guard.rs`  
9. `crates/sonia-core/src/ai_checklist.rs`

### 2.1 Alternate build sequences

**Sequence B1 – “Minimal guardrail” first**

1. Implement `ai_checklist.rs` with only:
   - Schema validation.
   - Size/encoding checks for artifacts.
2. Wire `ai_checklist` into `sonia-core` CLI `validate` and `write` commands.
3. Only then add:
   - `determinism.rs` (for forbidden imports and nondeterministic patterns).
   - `hardware_budget.rs`.
   - `abi_guard.rs`.

This offers an early, simple “pass/fail” surface to AI without immediately tackling AST and ABI analysis.

**Sequence B2 – ECS/determinism‑driven**

If rollback/netcode and deterministic simulation are your highest priority:

1. Implement `determinism.rs` first:
   - Focus on code patterns that break deterministic ECS assumptions.
   - Add CLI or library entrypoints that scan specific crates.
2. Integrate determinism results into:
   - `SessionProfile.invariants`.
   - `ci_status` digests.
3. Attach `hardware_budget.rs` and `abi_guard.rs` later as separate checklist modules.

This sequence aligns with validating the deterministic Rust ECS core early, even before full N64 budget logic is in place.

**Sequence B3 – Hardware‑first**

If the risk is more about ROM/VRAM budgets and safe patching:

1. Implement `hardware_budget.rs` first as a standalone crate or module:
   - Accepts `RomLayout`, `N64Constraints`, and `BudgetReport`.
   - Returns structured budget deltas and over‑budget diagnostics.
2. Integrate with:
   - `starzip-budget` CLI.
   - N64 vertical slice CI workflow.
3. Only later:
   - Wrap this in `ai_checklist`.
   - Add determinism and ABI checks as incremental guardrails.

### 2.2 Additional research objectives

1. **Determinism invariants as logical formulas**

   - Model determinism rules as formulas over:
     - Import sets (no `rand::thread_rng`, no `std::time`).
     - Collection types (no hash maps without fixed hasher).
   - Implement a simple logic engine that can:
     - Explain why a crate violates determinism.
     - Suggest concrete refactors (for example, replace `HashMap` with `IndexMap`).

2. **Hardware budget inequality solver**

   - Represent ROM/RDRAM/texture budgets as inequality systems:
     - Sum of segment sizes ≤ ROM size.
     - Sum of texture memory usage ≤ texture pool.
   - Long‑term goal: integrate a small solver that proposes tradeoff moves when budget is exceeded, instead of just failing.

3. **ABI versioning rules and migration scripts**

   - Extend `abi_guard.rs` to:
     - Mark certain C symbols as `Public`, `Internal`, or `Experimental`.
     - Require `version` or `changelog` entries when certain symbols change.
   - AI‑exclusive extension: given a breaking change, auto‑suggest a migration stub or wrapper to keep older callers compiling.

4. **Checklist composition language**

   - Implement a small DSL over `ai_checklist`:
     - Allows defining check “profiles”:
       - `strict_n64_core`.
       - `fast_iteration`.
       - `public_api_release`.
   - AI can select or be assigned a profile per `SessionProfile` and adapt its generation strategy accordingly.

---

## 3. Group C – Knowledge Graph & Feature Index

**Files**

10. `knowledgegraph/features.sonia.json`  
11. `crates/sonia-featurelayout/src/tag_algebra.rs`  
12. `crates/gamemodeai-kg/src/resolver.rs`  
13. `tools/kg-feature-sync/main.rs`

### 3.1 Alternate build sequences

**Sequence C1 – Hand‑curate, then automate**

1. Manually author `knowledgegraph/features.sonia.json` for:
   - N64 ROM Layout Oracle.
   - Safe Patch Synthesizer.
   - Budget Planner.
   - Session Profiles.
   - Emulator Scenario Director.
2. Implement:
   - `sonia-featurelayout` CLI (`list_by_tag` and `get_feature`) without `tag_algebra` beyond simple inclusion.
3. Later:
   - Add `tag_algebra.rs` to support AND/OR/NOT queries and more complex filters.
   - Implement `kg-feature-sync` to automatically suggest new feature skeleton entries from KG changes.

**Sequence C2 – KG‑driven feature skeletons first**

1. Implement `kg-feature-sync`:
   - Parse `knowledgegraph/systems.json`.
   - For each `SystemNode`, generate a candidate `FeatureEntry` skeleton:
     - Default tags from node tags.
     - Default schema list from node `config_schemas`.
2. Use this tool to bootstrap `features.sonia.json`.
3. Manually refine feature titles and descriptions.
4. Implement `tag_algebra` and `gamemodeai-kg` resolver as a second step.

This sequence emphasizes consistency between KG and features, reducing hand‑curation drift.

**Sequence C3 – Resolver‑first**

If you want high‑confidence KG integrity early:

1. Implement `gamemodeai-kg/src/resolver.rs` to:
   - Validate all `SystemNode` entries.
   - Check that feature references (once present) are not broken.
2. Only after KG integrity is ensured:
   - Add `features.sonia.json`.
   - Wire `sonia-featurelayout` and `tag_algebra`.

### 3.2 Additional research objectives

1. **Tag algebra metrics**

   - Extend `tag_algebra.rs` to compute:
     - Precision/recall proxies of a query over a known corpus.
     - Explainability for AI:
       - Why each `FeatureEntry` matched a given tag expression.
   - Use this to guide AI in choosing better tags or refining queries.

2. **Feature stability tiers**

   - Add a `stability` field to `FeatureEntry`:
     - `internal`, `experimental`, `stable`.
   - CI rule: AI should prefer `stable` features in suggestions unless explicitly in “experimental” mode.

3. **SystemNode <-> FeatureEntry consistency checker**

   - Extend `gamemodeai-kg/resolver.rs` to verify:
     - Every feature’s `systems[]` exists in KG.
     - Every schema path in `FeatureEntry.schemas[]` exists on disk and is listed by at least one `SystemNode`.
   - This keeps the feature surface trustworthy for AI navigation.

4. **Hierarchical feature namespaces**

   - Consider hierarchical IDs:
     - `nintendoor64.starzip.romlayout.query`.
     - `nintendoor64.starzip.romlayout.validate`.
   - Map these sub‑features to more granular CLIs and schema subsets, enabling the AI to pick “micro‑capabilities” precisely.

---

## 4. Group D – AI Conditioning & Session Intelligence

**Files**

14. `crates/sonia-core/src/session/conditioning.rs`  
15. `crates/sonia-core/src/session/ci_digest.rs`  
16. `ai_prompts/sonia-system-prompt.md`  
17. `ai_prompts/schema-guard-instructions.md`

### 4.1 Alternate build sequences

**Sequence D1 – Prompt‑first**

1. Draft `ai_prompts/sonia-system-prompt.md` and `ai_prompts/schema-guard-instructions.md` first, referencing:
   - Existing schemas.
   - CLI envelope.
   - Invariants at a conceptual level.
2. Use these prompts to configure AI behavior even while the Rust conditioning code is incomplete.
3. Implement:
   - `conditioning.rs` and `ci_digest.rs` as purely backend helpers later, gradually aligning AI behavior with the code enforcement.

**Sequence D2 – CI‑driven conditioning**

1. Implement `ci_digest.rs` first:
   - Normalize `cargo test`, `clippy`, `jsonschema`, and emulator results into `CiFailure` entries.
2. Wire these into `gamemodeai-session updatecistatus`.
3. Create an early minimal `sonia-system-prompt.md` that:
   - Emphasizes “use CI results as ground truth”.
   - Conditions AI to read `CiStatus` and treat failures as primary guidance.
4. Introduce more complex session conditioning in a second pass.

**Sequence D3 – Constraint‑first scoring**

If you want to experiment with AI “proposal scoring” early:

1. Implement `conditioning.rs` to:
   - Consume `SessionProfile.invariants`.
   - Score candidate `ArtifactSpec` or patch definitions before they reach CLI.
2. Expose a `dry_run` / “score only” mode in `sonia-core` where AI can:
   - Ask “how well would this proposal fit the current session?” without writing files.
3. Add CI digest integration and prompts later.

### 4.2 Additional research objectives

1. **Constraint satisfaction scoring function**

   - Represent invariants as weighted constraints:
     - Hard constraints (must not be violated).
     - Soft constraints (discouraged but allowed).
   - Compute a score for each AI proposal:
     - `score = hard_satisfied * big_weight + soft_satisfied * small_weight`.
   - Let AI observe these scores to iteratively refine its own candidate outputs.

2. **CI digest taxonomy**

   - Classify `CiFailure.kind`:
     - `SchemaViolation`.
     - `CompileError`.
     - `DeterminismViolation`.
     - `BudgetOverflow`.
   - Use this taxonomy in prompts to guide AI toward targeted fixes (for example, “fix budget overflow by adjusting map size or texture formats”).

3. **Adaptive prompt generation**

   - Generate a “dynamic system prompt” by serializing a subset of:
     - Active invariants.
     - Key feature entries.
     - CI failures from last run.
   - This makes `sonia-system-prompt.md` a template rather than a static file.

4. **Multi‑session conditioning**

   - Explore carrying long‑term knowledge across sessions via aggregated `SessionProfile` snapshots:
     - Frequent failure patterns.
     - Historically “expensive” invariants.
   - This can inform both AI and human decisions about where to invest refactor time.

---

## 5. Group E – Nintendoor64 Vertical Slice & CI Orchestration

**Files**

18. `crates/n64-layout/src/sonia_bridge.rs`  
19. `ci/workflows/sonia-ai-n64-slice.yml`  
20. `docs/sonia-ai-model.md`

### 5.1 Alternate build sequences

**Sequence E1 – “Docs‑first” slice**

1. Start with `docs/sonia-ai-model.md`:
   - Describe the intended N64 vertical slice in detail:
     - ROM Layout Oracle.
     - Safe Patch Synthesizer.
     - Budget Planner.
     - Emulator Scenario Director.
   - Include example CLI invocations and JSON payloads.
2. Use this document as a contract to:
   - Implement `sonia_bridge.rs` as an adapter between `RomLayout` / `PatchSpec` and `ArtifactSpec`.
   - Sketch `sonia-ai-n64-slice.yml` CI workflow that calls into these CLIs.
3. Only once these surfaces are stable, implement the actual CLI plumbing (Starzip patching, emulator harness).

**Sequence E2 – CI‑driven vertical slice**

1. Implement a simple `sonia-ai-n64-slice.yml` with:
   - `schema-gen` step.
   - `jsonschema` validation against example `RomLayout` and `PatchSpec` files.
   - A stubbed `starzip-cli patch` step that does nothing but return success.
2. Land this early as a “skeleton” CI job.
3. Gradually:
   - Replace stubs with real `starzip-cli` calls.
   - Add emulator boot smoke tests.
   - Integrate `sonia_bridge.rs` to feed artifacts into the patch step.

**Sequence E3 – Bridge‑first**

If Starzip and emulator harnesses are already in place:

1. Implement `sonia_bridge.rs` first:
   - Conversion from `RomLayout`/`PatchSpec` types into `ArtifactSpec` instances.
   - Helpers to attach or verify schema IDs and file metadata.
2. Test the bridge locally by:
   - Generating `ArtifactSpec` from `layout.json` and `patch.json`.
   - Passing them through `sonia-core` and `starzip-cli`.
3. Add the CI workflow and docs once the local path is proven.

### 5.2 Additional research objectives

1. **Patch graph introspection**

   - Extend `sonia_bridge.rs` or a companion tool to:
     - Compute patch impact regions (ROM intervals, segments, asset categories).
   - Use this for:
     - CI reporting (“this patch touched textures, but not code”).
     - AI guidance on risk (for example, “you’re editing a core layout segment; be conservative”).

2. **Scenario harness generalization**

   - Abstract the N64 emulator harness so it can:
     - Run multiple scenario specs in sequence.
     - Compare telemetry metrics between builds (for regression detection).
   - Later reuse the harness pattern for NES/SNES and PS1.

3. **Vertical slice quality metrics**

   - Define metrics for the slice:
     - Time to run full CI.
     - Coverage of patch intervals (percentage of segments touched in fuzzed tests).
     - Rate of AI‑generated patches passing all gates on first try.
   - Use them to evaluate improvements in invariants, prompts, and schema coverage.

4. **Human‑oriented “flight manual”**

   - Expand `docs/sonia-ai-model.md` into:
     - A “flight manual” for humans and AI.
     - Per‑feature quick‑start recipes: “How to add a safe N64 patch with AI assistance”.
   - Keep this doc versioned alongside schemas to prevent drift.

---

## 6. Cross‑Group Alternate Sequences

Beyond adjusting each group locally, there are macro‑scale sequences that re‑order the entire 20‑file plan.

### 6.1 Sequence X – “AI surface first”

Focus: making the system usable by AI as early as possible, even with partial enforcement.

Order:

1. Group A (protocol + CLIs): Files 1–5
2. Group C (features + KG resolver): Files 10–13
3. Group D (prompts + minimal conditioning): Files 14, 16, 17
4. Group E (stubbed CI slice): File 19 with stubbed steps
5. Group B (deep invariants and hard guards): Files 6–9, 15, 18, 20

Characteristics:

- AI gets a call‑able, documented surface early.
- Validation and invariants start softer; become strict later.
- Suitable when you want to prototype AI workflows and then progressively lock them down.

### 6.2 Sequence Y – “Hard‑contract first”

Focus: correctness and safety over early AI experiments.

Order:

1. Schema + invariants:
   - File 5 (`schema-gen`)
   - Files 6–9 (`determinism`, `hardware_budget`, `abi_guard`, `ai_checklist`)
2. Session + CI:
   - Files 3, 15 (`gamemodeai-session` CLI, `ci_digest`)
   - File 19 (full CI slice)
3. KG + features:
   - Files 10–13
4. CLIs + protocol:
   - Files 1, 2, 4
5. AI prompts and docs:
   - Files 14, 16, 17, 18, 20

Characteristics:

- Ensures robust contracts and enforcement exist before AI is given broad write access.
- Good fit when the repo is shared by many contributors and safety is paramount.

### 6.3 Sequence Z – “Vertical slice centered”

Focus: proving N64 ROM Layout Oracle + Safe Patch Synthesizer works end‑to‑end, then generalizing.

Order:

1. N64 infrastructure:
   - File 18 (`sonia_bridge.rs`)
   - Existing `n64-layout`, `starzip-*` crates
2. CI vertical slice:
   - File 19 (`sonia-ai-n64-slice.yml`)
3. Schema and KG:
   - File 5 (`schema-gen`)
   - File 10 (`features.sonia.json`)
   - File 12 (`kg resolver`)
4. Sonia CLIs:
   - Files 1–4
5. Invariants & AI conditioning:
   - Files 6–9, 14–17, 20
6. Automation and feature sync:
   - File 11 (`tag_algebra`)
   - File 13 (`kg-feature-sync`)

Characteristics:

- You get one high‑value, demonstrable path quickly.
- Subsequent work generalizes that slice into a broader AI‑oriented platform.

---

## 7. Further Definitions per File

Below is a concise “extended definition” for each of the 20 files, capturing future evolution beyond the first implementation.

1. `crates/sonia-core/src/cli.rs`  
   Evolve into a multi‑binary or subcommand‑rich CLI supporting:
   - Local and remote execution (over sockets).
   - “Dry‑run” scoring modes using `conditioning.rs`.
   - Pluggable checklists per session profile.

2. `crates/sonia-featurelayout/src/cli.rs`  
   Grow from simple `list_by_tag` / `get_feature` into:
   - Query‑by‑example (e.g., “features sharing schemas with X”).
   - Optional `commands` hints per feature, listing recommended CLI calls.

3. `crates/gamemodeai-session/src/cli.rs`  
   Extend beyond branch‑local JSON files to:
   - Multi‑repo sessions.
   - “Session overlays” for experimental branches (temporary invariants).
   - Optional HTTP mode for long‑lived session daemons.

4. `protocols/sonia-json-rpc.toml`  
   Become the canonical IDL for Sonia‑adjacent CLIs and tools, feeding:
   - Code generation.
   - Documentation generation.
   - Static analysis for backward compatibility.

5. `tools/schema-gen/main.rs`  
   Grow into a “schema governance” tool that:
   - Generates schemas.
   - Verifies versioning policies.
   - Creates migration stubs for schema changes.

6. `crates/sonia-core/src/invariants/determinism.rs`  
   Extend from pattern‑based detection toward:
   - Integration with Rust analyzer or language server for precise analysis.
   - Optional “auto‑fix” suggestions, especially around collection types and RNG usage.

7. `crates/sonia-core/src/invariants/hardware_budget.rs`  
   Add support for:
   - PS1 constraints.
   - Virtual architectures (planned budgets for unimplemented hardware).
   - Multidimensional tradeoff exploration (CPU vs memory vs ROM).

8. `crates/sonia-core/src/invariants/abi_guard.rs`  
   Extend to:
   - Automatic generation of `changelog`s or API docs on ABI changes.
   - Graph of ABI consumers so impact reports include affected frontends.

9. `crates/sonia-core/src/ai_checklist.rs`  
   Become a central policy engine:
   - Configurable via `SessionProfile` and per‑project settings.
   - Emitting machine‑readable reports that AI can parse and respond to.

10. `knowledgegraph/features.sonia.json`  
    Evolve into:
    - A multi‑platform feature index (NES, SNES, N64, PS1, PC engines).
    - With stability, ownership, and “recommended for AI” flags.

11. `crates/sonia-featurelayout/src/tag_algebra.rs`  
    Grow into:
    - A full query layer supporting nested Boolean logic and ranking.
    - With metrics that tell AI how “focused” a tag query is.

12. `crates/gamemodeai-kg/src/resolver.rs`  
    Extend to:
    - Support path queries (shortest dependencies between systems).
    - Scope queries (“what rules apply to this file/crate?”).
    - Symbol‑level indexing.

13. `tools/kg-feature-sync/main.rs`  
    Become:
    - A bidirectional sync tool:
      - From KG → features.
      - From new feature definitions → recommended KG updates (for example, missing schemas).

14. `crates/sonia-core/src/session/conditioning.rs`  
    Evolve into:
    - A constraint solver/scheduler that can:
      - Rank or reject AI proposals.
      - Suggest the “next best” features to enable given current budgets and invariants.

15. `crates/sonia-core/src/session/ci_digest.rs`  
    Expand to:
    - Normalize not only Rust/JSON failures, but also:
      - Emulator telemetry.
      - Build graph metrics.
      - Scenario analysis results.

16. `ai_prompts/sonia-system-prompt.md`  
    Gradually parameterize:
    - Turn from static text into a template driven by:
      - Active session.
      - Features discovered.
      - CI history.

17. `ai_prompts/schema-guard-instructions.md`  
    Be refined over time as:
    - More schema‑governed types are added.
    - AI failure modes become better understood.

18. `crates/n64-layout/src/sonia_bridge.rs`  
    Grow from a simple adapter into:
    - A two‑way bridge:
      - Converting between artifacts and typed layouts/patches.
      - Annotating artifacts with semantic metadata (segment kind, asset type).

19. `ci/workflows/sonia-ai-n64-slice.yml`  
    Expand as:
    - New checks and scenarios are added.
    - The vertical slice generalizes to multiple ROMs, multiple consoles, and more superpowers.

20. `docs/sonia-ai-model.md`  
    Evolve into:
    - The human‑readable canonical reference for:
      - Sonia contracts.
      - AI usage patterns.
      - Extension recipes for new platforms and superpowers.

---

## 8. Suggested Next Actions

Depending on current priorities, you can choose one of these short‑term “next passes” across the 20‑file plan:

1. **AI‑First “Usable Today” Pass**

   - Implement:
     - `protocols/sonia-json-rpc.toml`
     - Minimal CLIs for files 1–3.
     - Basic `features.sonia.json`.
     - `sonia-system-prompt.md` and `schema-guard-instructions.md`.
   - Keep invariants soft (just schema + size checks).
   - Goal: get a working AI↔Sonia loop quickly, even if some checks are not yet enforced.

2. **Contract‑Hardening Pass**

   - Focus on:
     - `schema-gen`.
     - `ai_checklist.rs` with real determinism/hardware/ABI checks.
     - `gamemodeai-session` CLI + `ci_digest.rs`.
     - N64 vertical slice CI skeleton.
   - Goal: guarantee that any AI‑generated artifact that passes the checklist is safe to run through Starzip and emulators.

3. **Vertical Slice Completion Pass**

   - Prioritize:
     - `sonia_bridge.rs`.
     - Full CI workflow for the N64 slice.
     - Concrete emulator scenario definitions.
   - Goal: complete the ROM Layout Oracle + Safe Patch Synthesizer path as a fully validated, AI‑driven pipeline.

All three passes are compatible; the main question is which layer you want to land first for the current development phase.
