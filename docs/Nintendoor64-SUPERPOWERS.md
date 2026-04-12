## 1. ROM Layout Oracle

An AI that understands your ROM layout and memory map instead of guessing.

It can answer questions like “where does this level live in the ROM?” and “which segment contains Bond’s weapons?” by querying Starzip’s structured view of the ROM: file table, code/data segments, compressed assets, and RAM/ROM address mappings. It would expose this as a navigation API in the knowledge graph, so AI‑Chat can say “replace the title screen texture” and automatically select the correct segment, format, and alignment rules. This is powered by wrapping tools like Z64‑style filesystem parsers and address converters behind a stable schema.

***

## 2. Safe Patch Synthesizer

A superpower that turns high‑level edits into safe, diff‑style patches.

Instead of rewriting entire ROMs, AI‑Chat would propose BPS/IPS/xdelta patches scoped to specific assets or code regions. It would know the allowed patch boundaries (from Nintendoor64 layout schemas) and generate patches that pass validation: correct CRCs, no overlap with boot code, and reversible changes. This enables workflows like “make the guards 20% more accurate” resulting in a tiny patch that mutates a BalancingConfig block, not arbitrary hex.

***

## 3. Pattern‑Aware System Composer

An AI that builds new mechanics by composing your pattern library, not inventing from scratch.

Given nodes like `systems.bondfpscore.stealth_ai`, `systems.bondfpscore.lockon`, `systems.bondfpscore.missions`, it can assemble feature sets: “make a Perfect Dark‑style infiltration mission” maps to enabling stealth, objective DAGs, and lock‑on in a specific mission profile. It understands which JSON schemas and Rust modules are involved, and wires them together by editing data files and mission scripts, leaving the deterministic cores untouched.

***

## 4. Retro Build Conductor

A superpower that orchestrates PS1/N64‑style “fast builds” end‑to‑end.

The AI can plan and execute a pipeline: modify data → run Sonia to write artifacts → call Starzip to repack the ROM → trigger emulator reload. It knows the minimal rebuild set, so a change to `data/bondfpscore/stealth/default.json` only rebuilds the stealth pack, not the entire project. This gets you near‑instant iteration loops reminiscent of original dev kits, but fully scripted and documented by the knowledge graph.

***

## 5. Emulator Scenario Director

An AI that sets up and drives deterministic test scenarios inside an emulator.

It can spawn a “test harness ROM” with scripted inputs (via your bitmask hook), pre‑configured save‑states, and debug overlays to stress‑test a mechanic. For example, “show me a replay where three guards detect the player at different light levels” becomes: build a tiny test map, place guards, configure stealth params, run the scenario, and capture traces. Over time, this evolves into an automated regression test suite that runs on every PR.

***

## 6. Cross‑Game Mechanic Transplant Surgeon

A tool‑assisted way to transplant mechanics between classic games in a principled way.

Using decomp and data formats described in the knowledge graph, the AI can map a mechanic like GoldenEye’s room‑based visibility or Conker’s contextual animations into your Nintendoor64 ECS equivalents. It would identify structural correspondences (e.g., STAN rooms ↔ BondNavgraph rooms, Conker script opcodes ↔ Conk64 Lua functions) and generate migration adapters, so you can say “import this SM64‑style triple jump into my Bond‑like prototype” and get a clean data + code translation instead of a brittle hack.

***

## 7. Visual Diff and Telemetry Analyst

An AI that compares ROM builds and in‑game metrics to explain gameplay differences.

Given two ROMs or two config profiles, it can show “what changed” in terms of data and behavior: stealth becomes harsher because `t_alert` dropped and light exponent increased; lock‑on feels snappier because cone angle widened. It would combine static diffs (JSON, assets) with telemetry captured from emulator runs (awareness curves, hit statistics) to produce human‑readable change reports that guide tuning.

***

## 8. Narrative Graph Cartographer

A superpower for designing and validating PD2‑style narrative structures.

It can read mission DAGs and higher‑level season graphs, detect cycles or unreachable branches, and suggest alternate routes: “this branch is impossible to see unless you deliberately fail objective 2,” or “to get a true stealth ending, your stealth flags must remain below threshold X across these missions.” For Nintendoor64, it ties objective DAGs, stealth outcomes, and mission packs together, ensuring data contracts are satisfied (e.g., any node tagged “stealth route” references a valid stealth profile).

***

## 9. Hardware‑Aware Budget Planner

An AI that optimizes features under N64‑style constraints.

Given constraints like RDRAM size, cartridge ROM size, and performance budgets, it can reason about trade‑offs: reducing texture resolution vs. reducing mission count, or swapping dynamic lights for baked ones. It combines knowledge of dev toolchains (n64chain‑style pipelines), asset sizes, and your layout schemas to propose concrete changes: “repack these three textures as 16‑bit instead of 32‑bit to free 1 MB for new cutscenes.”

***

## 10. Schema‑First Game Designer

A design superpower that starts from schemas and knowledge graph nodes as the “UI.”

Instead of free‑form prose, AI‑Chat can propose new content entirely in terms of your schemas: stealth profiles, mission DAGs, lock‑on configs, enemy stat blocks. It uses the knowledge graph to discover what’s possible, then instantiates and edits those contracts safely. That allows prompts like “create a three‑mission mini‑campaign with escalating stealth difficulty” to turn directly into new validated JSON/TOML files, Lua handlers, and Starzip build recipes—no changes to core Rust required.
