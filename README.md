# Nintendoor64

Nintendoor64 is a schema‑first, Rust‑powered toolchain for building, unpacking, and tuning retro and retro‑style games across classic console targets, under AI‑safe contracts and CI‑guarded workflows.[file:2][file:4] It is designed so AI‑Chat systems and human developers can orchestrate full ROM/ISO builds through small, typed JSON payloads instead of raw binaries or ad‑hoc shell scripts.[file:2][file:4]

---

## Project goals

Nintendoor64 has three primary goals:[file:2][file:4]

- Treat every AI‑visible action (patching, packing, budgeting, scenario testing) as a **contract** defined by Rust types and generated JSON Schemas.  
- Expose a coherent CLI surface (Starzip, Sonia, Conk64, retro‑cli, gamemodeai‑build) that is JSON‑in/JSON‑out, deterministic, and friendly to GitHub Actions and AI‑Chat orchestration.[file:2][file:4]  
- Support multiple hardware generations (8‑bit, 16‑bit, 32/64‑bit, optical media) behind one recipe and constraint layer, so the same high‑level game design can target NES, SNES, N64, PS1, GameCube, and Dreamcast backends over time.[file:4]

---

## Supported and planned platforms

Nintendoor64 is structured around a dual‑layer model: a hardware‑aware **constraint** layer (ROM size, VRAM, audio, CPU, controller limits) and a platform‑agnostic **recipe** layer (maps, missions, entities, weapons, audio cues).[file:4] Each platform is gradually wired into this model.

### NES (8‑bit)

For NES, Nintendoor64 focuses on NROM‑style cartridges with fixed PRG/CHR sizes:[file:4]

- `NesConstraints` describe PRG size, CHR bank size, nametable size, and tile budgets.[file:4]  
- `RetroRecipe` encodes maps and tilesets; the NES packer turns recipes + CHR data into CHR banks and 32×30 nametables under strict tile limits.[file:4]  
- A `retro-cli build-nes-map` command provides a deterministic pipeline from recipe JSON to NES‑ready binary chunks, with CI smoke tests via an emulator harness.[file:4]

**Status:** early vertical slice in design and code; CHR + nametable packing and constraints are implemented as a reference for the dual‑layer model.[file:4] Future work includes full attribute table generation, palette management, and iNES ROM assembly, plus knowledge‑graph registration of NES packer systems.[file:4]

### SNES (16‑bit)

SNES support extends the NES ideas to richer VRAM, tile modes, and audio pipelines:[file:4]

- Planned `SnesConstraints` will encode VRAM budgets, BG layer layouts, Mode 7, and BRR audio profiles.[file:4]  
- Recipes will describe tilemaps, parallax layers, palettes, and music cues, with backends that generate SNES‑ready assets and ROM images.[file:4]  
- CI and AI‑Chat flows will mirror NES and N64: schema‑guarded recipe JSON, constraint enforcement, and emulator scenarios for regression tests.[file:4]

**Status:** defined in the research plan; SNES is a near‑term backend target once NES and N64 slices are stable.[file:4]

### N64 (32/64‑bit)

N64 is the flagship platform for Nintendoor64 and has the deepest concrete implementation today:[file:2][file:4]

- `RomLayout` models segments, file entries, VRAM mappings, compression, and mutability.[file:2]  
- `PatchSpec` defines safe high‑level edits (ReplaceFile, BootHook, JsonPatch, RawIntervalPatch), compiled by Starzip into byte‑level patches under segment constraints.[file:2][file:4]  
- `N64Constraints`, `N64AssetManifest`, and `BudgetReport` encode ROM, RDRAM, texture/audio/script/data pools, and CPU budgets, with a `starzip-budget` CLI that emits JSON reports and non‑zero exits for over‑budget builds.[file:4]  
- Sonia’s `ArtifactSpec` is the universal payload format; `sonia-core` validates and writes artifacts; `n64-layout`’s `sonia_bridge.rs` adapts between RomLayout/PatchSpec and ArtifactSpec, and computes patch impact summaries.[file:2][file:4]  
- A full GitHub Actions workflow (`sonia-ai-n64-slice.yml`) regenerates schemas, validates JSON (layout, patches, artifacts), runs N64 checklist and budget checks, applies patches with Starzip, boots the ROM in a headless emulator (Conk64), and feeds a CI digest into SessionProfile for AI conditioning.[file:2]

**Status:** primary reference implementation; the N64 vertical slice is being hardened as the canonical blueprint that other platforms clone.[file:2]

### PS1 (optical disc)

PS1 support generalizes the N64 constraint and layout model to sector‑based discs and XA audio:[file:4]

- Planned `Ps1DiscLayout` and `Ps1Constraints` types will model tracks, sectors, XA streams, VRAM budgets, and disc‑level capacities.[file:4]  
- A PS1 budget planner will treat disc sectors, XA bitrates, and VRAM as constrained resources, feeding into Starzip/PS1 packers and CI.[file:4]  
- Recipes and build contracts will target PS1 via backends that generate disc layouts, assets, and ISO images, all validated against JSON Schemas and constraints.[file:4]

**Status:** defined as a next step after N64; the research map covers disc layouts, XA audio budgets, and PS1‑specific constraint structs to be implemented in `ps1-layout` and `ps1-constraints` crates.[file:4]

### GameCube

GameCube support is part of the long‑term roadmap, extending the same pattern to a more modern optical platform:[file:4]

- Constraints will cover disc size, memory card usage, ARAM/VRAM budgets, and CPU/GPU load, encoded as serializable Rust types with JSON Schemas.[file:4]  
- Layout models will describe file system hierarchies instead of pure segment tables, but will still integrate with recipe schemas and Starzip‑like packers.[file:4]  
- AI‑Chat will operate through build contracts and ArtifactSpecs, never raw disc images, with CI enforcing constraints and running GameCube emulator scenarios.

**Status:** planned; GameCube shares many concepts with PS1 but with richer hardware, and will be added once PS1 workflows are proven.[file:4]

### Dreamcast

Dreamcast is also targeted for future support, with emphasis on GD‑ROM layout and streaming:[file:4]

- Constraint profiles will capture GD‑ROM organization, VMU usage, VRAM, and sound RAM budgets, again expressed as inequality systems over bytes and cycles.[file:4]  
- Layout and recipe layers will let the same high‑level GameRecipe drive Dreamcast builds via dedicated backends and CLIs.[file:4]  
- Dreamcast emulation scenarios will plug into the same ScenarioSpec and CI digest pipeline used for N64 and PS1.

**Status:** long‑range; Dreamcast shares patterns with PS1 and GameCube, and will reuse the schema‑first, contract‑driven approach once those stacks are mature.[file:4]

---

## Architecture overview

Nintendoor64 unifies all these platforms behind a common architecture:[file:2][file:4]

- **Constraint layer:** per‑platform structs like `NesConstraints`, `SnesConstraints`, `N64Constraints`, `Ps1Constraints` (and later GameCube/Dreamcast equivalents) describe hardware budgets as explicit inequalities over ROM, VRAM, audio, CPU, and tile/triangle counts.[file:4]  
- **Recipe layer:** platform‑agnostic schemas such as `RetroRecipe`, `GameRecipe`, `MissionDAG`, `NarrativeGraph` capture maps, missions, stealth parameters, and narrative structure.[file:4]  
- **Tool layer:** small Rust CLIs (Starzip, Sonia, retro‑cli, gamemodeai‑build, gamemodeai‑session, gamemodeai‑kg) are JSON‑driven, idempotent, and registered in a knowledge graph for AI discovery.[file:2][file:4]  
- **CI layer:** GitHub Actions workflows regenerate schemas, validate all JSON, run constraint/budget checks, apply patches, and execute emulator scenarios, emitting structured CI digests consumed by SessionProfile and AI‑Chat.[file:2]

---

## AI‑Chat and Rust interoperability

Nintendoor64 is explicitly engineered to make AI‑Chat a safe orchestrator over Rust and GitHub:[file:2][file:4]

- AI emits small JSON contracts (`ArtifactSpec`, `PatchSpec`, `RomLayout` edits, `BuildContract`, `ScenarioSpec`) instead of free‑form commands.[file:2][file:4]  
- Rust types derive `JsonSchema`, and `schema-gen` keeps `schemas/*.schema.json` in sync, so CI can validate every AI‑generated file before any build runs.[file:2]  
- `gamemodeai-session` maintains a `SessionProfile` with invariants (determinism, budgets, ABI) and `ciStatus` (normalized `CiFailure` entries) that condition future AI proposals.[file:2][file:4]  
- The knowledge graph and feature index (`knowledgegraph/*.json`, `features.sonia.json`) expose all systems and superpowers (ROM Layout Oracle, Safe Patch Synthesizer, Budget Planner, Scenario Director, Schema‑First Designer) as addressable capabilities with schemas and examples.[file:4]

This combination lets Nintendoor64 scale from NES test ROMs to N64/PS1‑scale projects, and eventually to GameCube and Dreamcast, while keeping AI‑driven workflows constrained, explainable, and reproducible.[file:2][file:4]

---

## Repository layout (high level)

The workspace is organized into crates, tools, and docs that correspond to the architecture above:[file:2][file:4]

- `crates/` – Rust crates (`n64-layout`, `starzip-core`, `starzip-cli`, `n64-constraints`, `retro-nes-core`, planned `retro-snes-core`, `ps1-layout`, `ps1-constraints`, etc.).  
- `tools/` – Helper CLIs like `schema-gen`, `kg-feature-sync`.[file:2][file:4]  
- `schemas/` – Generated JSON Schemas for all AI‑visible types.[file:2]  
- `examples/` – Layout, patch, artifact, recipe, and scenario examples for NES, N64, and future PS1/SNES/GameCube/Dreamcast slices.[file:2][file:4]  
- `docs/` – Design documents such as `sonia-ai-model.md` and `sonia-n64-vertical-slice.md` that describe end‑to‑end AI‑driven pipelines.[file:2]

Each directory is designed to be discoverable via the knowledge graph, so AI‑Chat can navigate from system IDs and tags to actual schemas, examples, and CLIs.[file:4]

---

## Getting involved

Nintendoor64 is an active research and development workspace.[file:2][file:4] Contributions are welcome in:

- Implementing and hardening platform constraint crates (SNES, PS1, GameCube, Dreamcast).  
- Extending Starzip, Sonia, and retro‑cli backends for new consoles and engines.  
- Improving schemas, CI workflows, and knowledge‑graph metadata to make AI‑Chat navigation and orchestration more precise.  

The end goal is a shared, contract‑driven environment where NES, SNES, N64, PS1, GameCube, and Dreamcast projects can be unpacked, analyzed, and rebuilt by Rust tools and AI‑Chat together, with GitHub providing the backbone for CI and collaboration.[file:4]
