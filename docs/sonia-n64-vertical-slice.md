# Sonia N64 Vertical Slice – End-to-End Automation Spec

This document defines the canonical, end-to-end automation path for the Nintendo 64 vertical slice under the Sonia AI model. It binds together contracts, bridge code, CLIs, and CI into a single reproducible “green path” from an AI-generated JSON proposal to a validated, patched, and emulated ROM.

## 1. Scope and goals

The N64 vertical slice proves that:

- All AI-visible changes are expressed as `ArtifactSpec`, `RomLayout`, and `PatchSpec` JSON, never raw ROM bytes.
- Every artifact passes schema validation and invariant checks before any patch is applied.
- A single GitHub Actions workflow (`sonia-ai-n64-slice.yml`) can build, validate, patch, emulate, and feed a CI digest back into `SessionProfile`, closing the loop for AI conditioning.[file:1][file:2]

This slice is the template other platforms (NES, PS1) will clone and adapt.

## 2. Contracts and types

### 2.1 Core Sonia contracts

- `ArtifactSpec` (in `crates/sonia-core`):

  - `kind`: `N64RomPatch | N64Layout | N64PatchSpec | Other`
  - `filename`: repo-relative path under `artifacts/**`
  - `encoding`: `Text | Hex | Base64` (default `Text`)
  - `content`: encoded payload as string

- `SessionProfile` (in `gamemodeai-session`):

  - `repo`, `branch`, `activeCrate`
  - `featureFlags`
  - `invariants` (determinism, ROM budgets, ABI stability)
  - `ciStatus` (list of CI digest entries, failures like `BudgetOverflow`, `EmulatorCrash`).[file:1]

Schemas are generated via `schema-gen` into `schemas/artifact-spec.schema.json` and `schemas/session.session.schema.json` and enforced in CI.[file:1]

### 2.2 N64-specific contracts

- `RomLayout` (in `crates/n64-layout`):

  - `entrypoint`, `romSize`
  - `segments: Vec<Segment>` (name, kind, romoffset, romsize, vramstart, compression)
  - `files: Vec<FileEntry>` (path, segment, offsetInSegment, length, contentType).[file:2]

- `PatchSpec` (in `crates/starzip-core`):

  - `version`, `baseRomId`, `layoutId`
  - `edits: Vec<PatchEdit>` with variants like `ReplaceFile`, `BootHook`, `JsonPatch`, `RawIntervalPatch`.[file:2]

Schemas live under `schemas/romlayout.schema.json` and `schemas/patchspec.schema.json` and are validated in CI.[file:2]

## 3. Sonia bridge for N64 (`sonia_bridge.rs`)

### 3.1 Responsibilities

The bridge in `crates/n64-layout/src/sonia_bridge.rs` is responsible for:

- Converting `RomLayout` and patch payloads into `ArtifactSpec` instances that `sonia-core` can validate and write.
- Validating a `PatchSpec` against the layout and known payload sizes (file existence and length constraints).
- Computing a `PatchImpactReport` for CI and AI: per-segment added bytes and total impact.[file:2]

### 3.2 Key functions

The module exposes:

- `SoniaBridge::new(layout: &RomLayout) -> Result<SoniaBridge, SoniaBridgeError>` – ensures layout has segments and files.
- `layout_to_artifact(&self, path: &str) -> ArtifactSpec` – serializes the layout as a `Text` JSON artifact under `artifacts/layouts/n64/`.[file:2]
- `rom_patch_to_artifact(&self, filename: &str, bytes: &[u8]) -> ArtifactSpec` – wraps a patch payload as a `Base64` `N64RomPatch` artifact under `artifacts/patches/n64/`.[file:2]
- `validate_patch_spec_with_report(&self, spec: &PatchSpec, payloads: &PayloadIndex) -> Result<PatchImpactReport, SoniaBridgeError>` – checks logical paths, payload lengths, and aggregates per-segment added bytes.[file:2]
- `validate_patch_spec(&self, spec: &PatchSpec, payloads: &PayloadIndex) -> Result<(), SoniaBridgeError>` – convenience wrapper when caller does not need the report.[file:2]
- `build_test_slice_artifacts(...) -> (ArtifactSpec, ArtifactSpec, PatchSpec)` – helper to construct a tiny N64 test layout artifact, patch payload artifact, and patch spec for the CI slice.[file:2]

### 3.3 Patch impact report

The `PatchImpactReport` is JSON-serializable and intended for CI logs and AI consumption:

- `layout_id`, `base_rom_id`
- `total_added_bytes`
- `per_segment: Vec<SegmentPatchUsage>` where each entry contains:

  - `segment_name`, `segment_kind`
  - `rom_offset`, `rom_size`
  - `current_bytes`
  - `added_bytes`
  - `max_bytes` (provisionally equal to `rom_size`, extendable with budget data).[file:2]

This report is the core of “patch impact introspection,” giving CI and AI a structured understanding of which segments and asset classes are affected by a patch.

## 4. N64 examples bundle (`examples/n64`)

To make the pipeline immediately runnable in CI, the repo includes:

- `examples/n64/roms/test-rom.z64` – small test ROM.
- `examples/n64/layouts/test-layout.json` – minimal `RomLayout` with `boot`, `main`, and `assets` segments and three files (title texture, one map, one script).[file:2]
- `examples/n64/artifacts/test-texture-payload.json` – `ArtifactSpec` for a new title-screen texture, encoded as `Base64` and targeting `artifacts/patches/n64/test-title-screen.bin`.[file:2]
- `examples/n64/patches/test-patch.json` – `PatchSpec` that issues a single `ReplaceFile` edit for `textures/title-screen.rgba16` using the payload filename from the artifact.[file:2]

These files are validated against their schemas and then consumed by `sonia-core` and `starzip-cli` inside CI.

## 5. CI workflow: `sonia-ai-n64-slice.yml`

### 5.1 Overview

The CI workflow `.github/workflows/sonia-ai-n64-slice.yml` defines a directed acyclic graph of pure steps that collectively prove the validity of an AI-driven N64 patch:

1. Checkout and toolchain setup.
2. Build required CLIs (`schema-gen`, `sonia-core`, `n64-layout`, `starzip-cli`, `gamemodeai-session`, and optionally `jsonschema-validate`, `conk64-cli`).[file:1]
3. Regenerate all JSON schemas.
4. Validate N64 layouts, patch specs, and ArtifactSpecs with a JSON Schema validator.
5. Use `sonia-core validate` and `sonia-core write` to materialize artifacts into `artifacts/**`.
6. Run `starzip-cli rom-query` and `starzip-cli patch` on the test ROM and N64 layout.
7. Run a headless `conk64-cli run-headless` smoke test on the patched ROM.
8. Produce a CI digest JSON and call `gamemodeai-session update-ci-status`.[file:1][file:2]

### 5.2 Schema and artifact validation

The workflow runs:

- `schema-gen --out-dir schemas` – regenerates schemas from Rust types.
- `jsonschema-validate` for:

  - `schemas/romlayout.schema.json` vs `examples/n64/layouts/*.json`
  - `schemas/patchspec.schema.json` vs `examples/n64/patches/*.json`
  - `schemas/artifact-spec.schema.json` vs `examples/n64/artifacts/*.json`.[file:1][file:2]

Then, for each artifact spec, it runs:

- `sonia-core validate --spec <artifact.json>`
- `sonia-core write --spec <artifact.json> --repo-root .`.[file:1]

### 5.3 Patching and emulation

Once artifacts are written, the workflow:

- Calls `starzip-cli rom-query --rom examples/n64/roms/test-rom.z64 --layout examples/n64/layouts/test-layout.json --addr 0x1000` as a quick layout sanity check.[file:2]
- Validates the patch with `starzip-cli validate-patch --layout ... --patch ...` (which internally uses the RomLayout and PatchSpec invariants).[file:2]
- Applies the patch via `starzip-cli patch --rom ... --layout ... --spec ... --out target/n64/test-rom-patched.z64`.[file:2]
- Boots the patched ROM in `conk64-cli run-headless --rom target/n64/test-rom-patched.z64 --frames 300 --assert-vram-write 0x80300000` to confirm it reaches a known hook.[file:2]

### 5.4 CI digest and session update

At the end, a CI digest JSON is written:

- `job`: `"sonia-ai-n64-slice"`
- `status`: `"ok"` or `"failed"`
- `summary`: human-readable text
- `timestamp`: GitHub run ID or timestamp
- `failures`: array of structured entries (e.g., `"BudgetOverflow"`, `"EmulatorCrash"`) with optional metadata.[file:1]

The workflow then calls:

- `gamemodeai-session update-ci-status --digest target/n64/sonia-n64-slice-ci-digest.json`

This updates `SessionProfile.ciStatus`, closing the loop so AI can read the fresh CI state on the next turn.[file:1]

## 6. Mathematical invariants and validators

### 6.1 Patch safety (interval arithmetic)

Patch safety is enforced by the Safe Patch Synthesizer and mirrored in the bridge and Starzip validators:

- Each segment is an interval \([S, E]\) on the ROM address line.
- Each patch edit is an interval \([o, o + \ell]\) within a segment.

Safety requires:

- Containment: \(S \le o\) and \(o + \ell \le E\) for the chosen segment.
- Disjointness: for any two patches \(i, j\), either \(o_i + \ell_i \le o_j\) or \(o_j + \ell_j \le o_i\).[file:2]

The implementation uses sorted intervals or an interval tree and is tested with unit tests plus property-based fuzzing to reject overlapping or out-of-bounds patches and accept well-formed ones.[file:2]

### 6.2 Budget constraints (polyhedral checks)

N64 hardware budgets are represented as linear inequalities over resource sums:

- ROM: \(\sum_i \text{rom}_i \le \text{ROM}_{\text{max}}\)
- Texture VRAM: \(\sum_j \text{tex}_j \le \text{VRAM}_{\text{tex,max}}\)
- Audio VRAM, runtime RAM, CPU cycles per frame: similar inequalities.[file:2]

The checker computes:

- `usage` vector from existing assets plus proposed patches.
- Slack per dimension: `slack_k = b_k - usage_k`.

A negative slack indicates overflow and is mapped to `BudgetOverflow` in CI. The `starzip-budget` tool emits a `BudgetReport` and `is_within_budget` flag, which CI asserts for the N64 slice.[file:2]

## 7. Actionable implementation checklist

To finish and harden this N64 slice:

1. **Finalize bridge:**

   - Keep `sonia_bridge.rs` as the single adapter between `RomLayout`/`PatchSpec` and `ArtifactSpec`.
   - Ensure it uses the latest `RomLayout` and `PatchSpec` types, and returns `PatchImpactReport` for every validated spec.[file:2]

2. **Wire bridge into CI:**

   - Add a small CLI or test harness that calls `validate_patch_spec_with_report` and prints the JSON report.
   - Capture that report in CI logs for every N64 slice run.[file:2]

3. **Complete examples bundle:**

   - Replace the placeholder Base64 payload with a real texture chunk of appropriate size.
   - Make sure `test-rom.z64` and `test-layout.json` align (segment offsets and file lengths match the actual ROM).[file:2]

4. **Strengthen validators:**

   - Integrate Safe Patch Synthesizer checks in `starzip-cli validate-patch` and assert they pass in CI.
   - Run a fuzzing job (locally or in a dedicated lane) against the patch and budget checkers to stress-test invariants.[file:2]

5. **Keep documentation in sync:**

   - Update this spec and `docs/sonia-ai-model.md` whenever schemas, CLIs, or invariants evolve so the N64 slice remains the reference pattern for future platforms.[file:1][file:2]

Once this lane is reliably green, you can clone the same pattern for NES and PS1 with new layout/patch types, emulators, and budget checkers, while reusing Sonia, SessionProfile, and the CI orchestration model.
