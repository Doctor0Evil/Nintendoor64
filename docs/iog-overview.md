# Internet of Games / Nintendoor64 Wayback Layer Overview

This document describes the Nintendoor64 “Internet of Games” (IoG) layer: a Rust‑first toolchain and protocol surface for reviving, orchestrating, and safely modifying legacy console and PC multiplayer games inside the GAMEMODE.ai ecosystem. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

The goal is to treat old games’ network stacks, ROM layouts, and engine behaviors as **typed, queryable contracts** instead of opaque binaries, so AI‑Chat and human developers can reason about them using schemas, CLIs, and deterministic Rust code rather than raw hex or ad‑hoc patches. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## High‑Level Goals

The IoG layer has four primary responsibilities:

1. **Wayback networking and matchmaking**  
   Provide a transparent, cross‑platform proxy and discovery layer that can bring dead multiplayer games back online without modifying their original binaries where possible. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

2. **Schema‑first ROM and layout control (Nintendoor64)**  
   Elevate ROM layout and patching (Starzip, Sonia, RomLayout) into a **ROM database** and **Safe Patch Synthesizer**, so all low‑level edits are mediated by validated contracts. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

3. **AI‑assisted orchestration and codegen**  
   Allow AI‑Chat to design, modify, and build N64/PS1‑scale games (and mods) in one session by orchestrating Rust CLIs, asset pipelines, and knowledge‑graph metadata rather than emitting monolithic binaries in chat. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

4. **Cross‑engine, cross‑platform integration**  
   Bridge these capabilities into modern engines (Unreal, Unity, Godot) and existing GAMEMODE.ai templates (arena shooters, stealth, horror, platformers) via deterministic ECS cores and stable C/Lua/WASM interfaces. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

The Nintendoor64 repository is the home of the console‑focused vertical slice of this vision, centered on N64, PS1, and related retro pipelines. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## Repository Role in the Ecosystem

Nintendoor64 sits alongside the main GAMEMODE.ai workspace as the **console and ROM‑centric cluster**:

- It provides **N64/PS1 layout models**, patchers, and budget planners (RomLayout, PatchSpec, N64Constraints, PS1Constraints) and exposes them to AI‑Chat via schemas and CLIs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- It defines **toolchain‑agnostic recipes** and **hardware‑aware constraints** so a single JSON design can target multiple output formats (e.g., N64 ROM vs. PS1 disc vs. Godot frontend), while respecting console limits. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- It registers **SystemNodes** and feature‑layout metadata in a knowledge graph so tools and AI can navigate by concept (e.g., `systems.nintendoor64.starzip`) instead of raw paths. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

Conceptually:

- **GAMEMODE.ai** owns deterministic ECS cores, genre modules, and AI‑facing CLIs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)
- **Nintendoor64** owns ROM layouts, console constraints, and retro pipelines, wired into the same knowledge graph and session profile system. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## Core Subsystems

### 1. ROM Layout Oracle and Safe Patch Synthesizer

The ROM Layout Oracle turns an N64/PS1 ROM into a **typed database**:

- `RomLayout` captures segments, files, VRAM mappings, and entrypoints. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Validation functions enforce invariants: segments do not overlap, files stay within segments, entrypoints live inside code regions, and all address mappings are consistent. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- A query API and CLI (`starzip-cli rom-query`) answer structural questions like “which segment owns ROM 0x123456?” or “which file contains this texture?” without touching hex directly. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

On top of this, the Safe Patch Synthesizer:

- Defines `PatchSpec` / `PatchOp` schemas for logical operations like “replace logical file path X” or “inject boot hook Y,” which are compiled into disjoint, legal byte intervals. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Uses interval arithmetic and capacity constraints so patches never touch forbidden regions (bootloader, PIF) and never overflow segment budgets. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Regenerates console‑specific checksums (e.g., N64 CIC header CRC) to keep patched ROMs bootable. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

Sonia’s `ArtifactSpec` serves as the **binary patch transport**: AI‑Chat emits JSON with `kind`, `filename`, `encoding` (Text/Hex/Base64), and content; Sonia decodes and writes artifacts to disk so raw ROM binaries never transit the chat channel. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

### 2. Retro Recipes, Constraints, and Backends

Nintendoor64 shares the **dual‑layer recipe model** introduced for NES/SNES/Godot in GAMEMODE.ai: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

- A **platform‑agnostic recipe layer** describes maps, tilesets, entities, weapons, and scripts without mentioning hardware specifics. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- A **hardware‑aware constraints layer** (e.g., `N64Constraints`, `PS1Constraints`) encodes ROM size, VRAM, CHR/texture budgets, and controller layouts as explicit Rust types. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- Backend crates (e.g., `n64-build`, PS1 packers) convert constrained recipes into platform‑native assets and ROM/disc images using existing homebrew toolchains. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

CLI tools like `gamemodeai-retro-build` and Nintendoor64‑specific builders follow a standard pipeline: [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

1. Validate recipe JSON against generated JSON Schemas.
2. Apply target constraints to produce a `ConstrainedGame`.
3. Invoke a backend (NES, SNES, N64, PS1, Godot) to emit ROMs and asset bundles.
4. Optionally run headless emulator smoke tests to verify boot and simple behaviors. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

For N64/PS1, Nintendoor64 adds:

- Extended `RomLayout` metadata (segment kinds, compression, VRAM windows).
- Budget profiles and inequality‑based constraints for segment sizes, textures, audio, and missions. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Cross‑console patterns so the same high‑level recipe can target both N64 ROMs and PS1 discs, with backends sharing a common schema and knowledge graph vocabulary. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

### 3. Knowledge Graph and Feature Layout

Both GAMEMODE.ai and Nintendoor64 rely on a JSON‑based knowledge graph (`knowledgegraph/systems*.json`) and a **Sonia Feature Layout** file to make the codebase navigable to tools and AI‑Chat. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

Key pieces:

- `SystemNode` entries describe each logical system: `id`, `title`, `description`, crate, file paths, tags (e.g., `Nintendoor64`, `Deterministic`, `CLI`), and related systems. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Schemas are generated from Rust types via `schemars` and stored under `schemas/*.schema.json`, with CI jobs validating all JSON/TOML against them. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- `sonia-featurelayout` defines a `FeatureLayout` JSON listing high‑level features (e.g., “ROM Layout Oracle”, “Safe Patch Synthesizer”, “Narrative Graph Cartographer”), each linked to SystemNodes, schemas, and example artifacts. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

A small CLI (`sonia-featurelayout`) allows AI‑Chat and humans to:

- List features by tag (e.g., `Nintendoor64`).
- Retrieve feature definitions and follow links to systems, schemas, and example files. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

This turns Nintendoor64 into a **queryable map of capabilities**, not just a pile of crates.

### 4. Session Profiles, Invariants, and CI

Session profiles capture per‑branch constraints and CI status:

- `SessionProfile` records repo, branch, active crate, invariants, TODOs, and CI digests, all with JSON Schemas keeping Rust types and JSON in sync. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- Invariants encode rules like “no direct writes to `.z64` ROMs, only via Starzip+RomLayout,” “deterministic cores must not use non‑seeded RNG,” or “ROM size must not exceed N64 GamePak ceilings.” [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- A `gamemodeai-session` CLI reads/updates these profiles and ingests CI outcomes so AI‑Chat can adapt suggestions based on real failures. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

For Nintendoor64, this means:

- Every AI‑generated patch, layout, or recipe must validate against schemas and invariants before Starzip or retro builders run. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Knowledge graph entries for Nintendoor64 systems reference the relevant invariants so AI‑Chat can query “what rules apply here?” before modifying ROM layouts or constraints. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

CI pipelines wire together:

- Schema generation and validation.
- Starzip/Safe‑Patch tests.
- Retro build tests (NES/SNES/N64/PS1) and emulator smoke tests.
- Knowledge graph consistency checks. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## Internet‑of‑Games / Wayback Networking Layer

The IoG networking layer generalizes these schema‑ and CLI‑based patterns to the **network side** of “lost” games.

### Objectives

- Provide a **transparent network proxy** that can resurrect legacy matchmaking, lobbies, and multiplayer sessions (including LAN/IPX) with minimal or no changes to original binaries.
- Expose game network behaviors and quirks as typed Rust structures and Lua/WASM scripting surfaces, so the community can fix or extend them declaratively.
- Integrate with Nintendoor64’s ROM and layout models for titles that require both binary and network interventions.

### Design Sketch

The Wayback networking stack (which can live as a sibling workspace or subtree) mirrors the Nintendoor64 structure:

- A `wayback-core-net` crate for TUN/TAP adapters, DNS override, IPX emulation, and packet forwarding.
- A `wayback-proxy` crate that handles engine detection, plugin loading, Lua/WASM packet handlers, and circuit breakers.
- A `wayback-detect` crate that uses signatures to recognize engines (e.g., Quake‑like, GoldSrc‑like) from handshake bytes and `.pcap` captures.
- Web‑facing crates (`wayback-ws-bridge`, WASM builds) for browser lobbies and WebRTC bridges, reusing the same schemas and interface patterns used for Nintendoor64 builders.

Although the networking code is not yet fully implemented, it is intended to follow the same **contract‑first** approach:

- Typed Rust structs + JsonSchema.
- JSON‑over‑stdin CLIs for AI‑Chat to call.
- Knowledge graph entries that link each protocol handler or engine hook to its code and schemas. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

## How AI‑Chat Uses Nintendoor64 and IoG

Within this architecture, AI‑Chat is an **orchestrator** over Rust tools and schemas, not a raw binary emitter. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

Typical flows:

- **ROM introspection:** Call `starzip-cli rom-query` with a RomLayout JSON; get back structured answers; propose safe, schema‑valid patch specs; and let Starzip/Sonia apply them. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- **Retro build:** Generate or edit a GameRecipe JSON; call `retro-build` or Nintendoor64 builders with `--target n64` or `--target ps1`; inspect build reports and CI; refine parameters. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)
- **Navigation:** Use `sonia-featurelayout` and knowledge graph CLIs to locate relevant systems, schemas, and examples before making changes. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- **Session‑aware iteration:** Read the SessionProfile to understand invariants and recent CI failures, then only propose changes that move the project toward a green state. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

The IoG work extends this: AI‑Chat can help define protocol signatures, write Lua packet handlers, and adjust NAT/relay configurations by editing typed configs and scripts, with Rust enforcing safety and determinism at runtime. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

## Next Directions for Nintendoor64 + IoG

Concrete next implementation moves in this repo:

1. **Finalize RomLayout and PatchSpec core**  
   Ensure `n64-layout`, `starzip-core`, and schemas cover segment kinds, compression, VRAM mapping, and safe patch intervals, with interval trees and safety reports documented in `docs/nintendoor64-patch-invariants.md`. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

2. **Wire full JSON Schema + CI loop**  
   Derive and commit schemas for RomLayout, PatchSpec, N64/PS1 constraints, ArtifactSpec, SessionProfile, and FeatureLayout; add CI jobs to validate all layouts, patches, recipes, and knowledge‑graph JSON. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

3. **Implement a minimal N64 arena‑shooter recipe path**  
   Define an `arenashootern64` recipe schema and builder that take a small test recipe to an N64 ROM using the dual‑layer constraints model, providing a concrete IoG‑capable testbed. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/fc1758b1-8417-46ff-a427-e96b16d1a038/from-single-codebase-to-four-g-XcbnGapsRz.cd6vNr8plPw.md)

4. **Add IoG network spec stubs to docs**  
   Introduce placeholder docs and crate stubs (`wayback-core-net`, `wayback-proxy`, `wayback-detect`) in Nintendoor64 or a sibling workspace, so the Internet‑of‑Games work can share the same schemas, knowledge graph, and session‑profile infrastructure. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

5. **Tighten knowledge‑graph and feature‑layout coverage**  
   Enumerate all Nintendoor64 superpowers (ROM Layout Oracle, Safe Patch Synthesizer, Narrative Graph Cartographer, etc.) as FeatureEntries with concrete SystemNodes and schema links to give AI‑Chat a complete navigation surface. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

These steps keep Nintendoor64 aligned with the broader GAMEMODE.ai vision while giving the Internet‑of‑Games / Wayback project a clear, schema‑backed home in the repo.
