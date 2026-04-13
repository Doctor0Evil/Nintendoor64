# AI–Cargo Integration Blueprint for Nintendoor64 and GAMEMODE.ai

This document describes how an AI assistant should interact with Rust and Cargo in the Nintendoor64 / GAMEMODE.ai ecosystem, focusing on macro debugging, procedural macro failures, debugging build scripts, multi-target toolchains, and environment template generation (Docker/Nix). The core design principle is that the AI never shells out directly; it calls small, typed CLIs via JSON contracts, and CI enforces correctness and safety.

***

## 1. Explaining Macro Expansion Errors

### 1.1 Goals

- Let the AI explain macro expansion errors using the **actual expanded code** that `rustc` sees.
- Keep macro expansion behind a **safe, typed CLI**, never as an ad-hoc shell command.
- Integrate expansion artifacts with CI and existing Sonia/Sonia-core flows.

### 1.2 `rust-macro-expand` CLI

Introduce a CLI dedicated to macro expansion:

- Crate: `tools/rust-macro-expand`
- Purpose: Wrap `cargo expand` or `rustc -Z unpretty=expanded` for a constrained scope (single crate/target/file/line).
- Input: JSON request on stdin.
- Output: JSON response on stdout.

Example request:

```json
{
  "crate": "n64-layout",
  "target": "lib",
  "file": "src/controller.rs",
  "line": 123,
  "contextLines": 20
}
```

Example response:

```json
{
  "expandedSnippet": "/* Rust code rustc sees here */",
  "macroCallSpan": {
    "file": "src/controller.rs",
    "lineStart": 120,
    "lineEnd": 130
  },
  "notes": [
    "Expansion produced an impl block with conflicting method names."
  ],
  "truncated": false
}
```

Key properties:

- The CLI is the **only place** that actually runs `cargo expand` or `rustc -Z unpretty`.
- The AI receives a small, structured JSON payload containing:
  - The relevant expanded snippet.
  - Spans for the macro callsite.
  - Optional notes (e.g., truncated output, hints).

### 1.3 CI and Sonia Integration

- Optionally integrate `rust-macro-expand` into CI when macro-related `CiFailure` entries are detected. The digest job can pre-compute expansions and store them under `artifacts/meta/macro-expand/`.
- When a macro expansion error appears in `SessionProfile.ciStatus`, the AI:
  - Reads the associated expansion artifact (file path in the failure).
  - Explains the error using the **expanded** code, not the macro definition.
- For fixes, the AI:
  - Proposes edits via `apply-diff` or ArtifactSpecs.
  - Optionally re-requests expansion to verify the new code before suggesting another build.

***

## 2. Handling Procedural Macros That Panic

### 2.1 Goals

- Normalize procedural macro panics into clean, structured diagnostics.
- Avoid dumping full panic backtraces into the model.
- Attach panic context to `CiFailure` entries for the AI.

### 2.2 Compiler Flags and Diagnostics

In CI/dev images:

- Enable richer macro diagnostics (e.g., `RUSTFLAGS=-Zmacro-backtrace` where supported).
- Run builds/tests with `--message-format=json` so proc macro panics appear with structured fields (macro name, crate, spans).

### 2.3 `ProcMacroPanic` in `sessioncidigest.rs`

Extend `sessioncidigest.rs` to:

- Parse compiler messages that indicate a procedural macro panic.
- For each panic, create a `CiFailure` of kind `ProcMacroPanic` (or equivalent):

```json
{
  "cratename": "n64-ai-gen-schemas",
  "kind": "ProcMacroPanic",
  "message": "derive(N64Layout) panicked: missing segment field",
  "file": "src/layout.rs",
  "line": 87,
  "logUrl": null
}
```

Optional extras:

- Macro crate and name: `macroCrate`, `macroName`.
- A short excerpt of the macro input tokens.

These `CiFailure` entries are then ingested into `SessionProfile.ciStatus.failures`, becoming the canonical representation of macro panics for AI-chat.

### 2.4 Optional `proc-macro-dump` Helper

Add a tooling path for deeper macro debugging:

- Crate: `tools/proc-macro-dump`
- Input: macro name, crate, and callsite info from a `ProcMacroPanic` failure.
- Behavior:
  - Re-runs the macro in isolation (best-effort) on the same input tokens.
  - Records diagnostic information (tokens, panic message, spans) into a JSON artifact under `artifacts/meta/proc-macro-debug/`.
- AI-chat can then reference that artifact when explaining the failure or proposing fixes, without ever executing macros directly.

***

## 3. Debugging `build.rs` with rust-gdb

### 3.1 Goals

- Allow humans to debug `build.rs` with `rust-gdb` or `gdbserver`.
- Keep the AI as an orchestrator (setup and metadata), not as a debugger driver.
- Make build-script debug sessions reproducible and discoverable.

### 3.2 `debug-build-script` CLI

Introduce a controlled debugging entrypoint:

- Crate: `gamemodeai-rust-cli`, command: `debug-build-script`.
- Input JSON:

```json
{
  "crate": "n64-ai-gen-schemas",
  "mode": "gdbserver",
  "port": 12345,
  "args": []
}
```

- Modes:
  - `gdbserver`: Launch `gdbserver` attached to the build script binary and listen on the given port.
  - `rust-gdb`: Launch `rust-gdb` in TUI mode attached to the build script (for interactive terminals).

- Output JSON:

```json
{
  "crate": "n64-ai-gen-schemas",
  "binaryPath": "target/debug/build/n64-ai-gen-schemas-<hash>/build-script-build",
  "debugMode": "gdbserver",
  "host": "127.0.0.1",
  "port": 12345,
  "instructions": "Attach your IDE or rust-gdb to 127.0.0.1:12345."
}
```

### 3.3 AI’s Role

When a build-script-related `CiFailure` keeps recurring:

- AI suggests using `debug-build-script` and emits the JSON request.
- The CLI runs in the user’s environment (devcontainer, Codespace, local), and returns connection info in JSON.
- The AI surfaces the instructions (e.g., “Attach your IDE to 127.0.0.1:12345”) and uses subsequent `CiFailure` updates to reason about progress.

The AI never drives gdb commands directly, but it can coordinate when and how debug sessions are spawned.

***

## 4. Managing Multiple Target Triples

### 4.1 Goals

- Make switching between `x86_64` host and N64 targets (e.g. `mips64-unknown-elf`) a **data-level operation**, not raw CLI flags.
- Ensure toolchain presence is handled by CI and container images, not by the AI.

### 4.2 Build Configuration in `SessionProfile`

Augment `SessionProfile` with a `buildConfig` section:

```json
{
  "buildConfig": {
    "defaultHostTarget": "x86_64-unknown-linux-gnu",
    "defaultGameTarget": "mips64-unknown-elf",
    "allowedTargets": [
      "x86_64-unknown-linux-gnu",
      "mips64-unknown-elf"
    ],
    "toolchainId": "n64-ci-toolchain-v1"
  }
}
```

Key points:

- `toolchainId` ties back into the knowledge graph (SystemNodes) and CI configuration so that a given target triple is always associated with a vetted toolchain image.
- AI can propose changes to `buildConfig` via a session-update command, but invariants (e.g., “only known target triples allowed”) are enforced on the server side.

### 4.3 Contracts and Toolchains

- `BuildContract.target` (e.g., `nes`, `snes`, `n64`) maps to one or more concrete target triples in the CI layer.
- CI jobs (e.g., N64 vertical slice) codify:
  - Which Docker image or Nix shell to use per target.
  - Which `cargo build --target=...` or `cross build --target=...` invocation to run.

If a required target/toolchain combination is missing:

- The CI/job wrapper emits a `CiFailure` with kind `ToolchainMissing` or `ToolchainMismatch`.
- This failure flows into `SessionProfile.ciStatus`, telling the AI that the environment is broken and must be fixed by humans (image updates, new toolchains).

The AI “switches” from host to N64 by:

- Editing `BuildContract` or `SessionProfile.buildConfig` fields.
- Requesting the appropriate build via `gamemodeai-build` or a higher-level N64-specific command.
- Interpreting `CiStatus` results rather than manipulating toolchains directly.

***

## 5. Generating Dockerfiles and Nix Flakes from Missing Dependencies

### 5.1 Goals

- Turn missing system dependencies into **structured information**.
- Allow the AI to draft environment definitions (Dockerfiles, Nix flakes) based on these, but keep build/push under human control.
- Preserve auditability by storing templates as artifacts or PR changes.

### 5.2 CI Digests for Missing Dependencies

Extend `sessioncidigest.rs` and related CI tools so that infra failures include explicit dependency metadata:

Example `CiFailure` for infra:

```json
{
  "cratename": "n64-ai-gen-schemas",
  "kind": "MissingDependency",
  "message": "libyaml-dev not found when linking.",
  "file": null,
  "line": null,
  "logUrl": null,
  "systemPackages": ["libyaml-dev"],
  "langPackages": []
}
```

Or for toolchains:

```json
{
  "cratename": "n64-build",
  "kind": "ToolchainMissing",
  "message": "mips64-elf-gcc not in PATH.",
  "systemPackages": ["mips64-elf-gcc"],
  "langPackages": []
}
```

This is written into CI digest JSON and then into `SessionProfile.ciStatus`.

### 5.3 `env-template-gen` CLI

Introduce a generator tool that translates missing dependencies into environment templates:

- Crate: `tools/env-template-gen`
- Request:

```json
{
  "base": "debian:bookworm-slim",
  "missingSystemPackages": ["libyaml-dev", "mips64-elf-gcc"],
  "templateKind": "Dockerfile"
}
```

- Response:

```json
{
  "kind": "Dockerfile",
  "filename": "Dockerfile.n64-ci",
  "content": "FROM debian:bookworm-slim\nRUN apt-get update && apt-get install -y libyaml-dev mips64-elf-gcc ...\n"
}
```

You can support multiple `templateKind` values:

- `"Dockerfile"`: produce a complete `Dockerfile`.
- `"NixFlake"`: emit a `flake.nix` with appropriate `buildInputs`.
- `"Devcontainer"`: emit a `.devcontainer/devcontainer.json` plus a Dockerfile snippet.

### 5.4 AI Workflow

Given a CI failure:

1. AI reads `SessionProfile.ciStatus` and finds `MissingDependency` / `ToolchainMissing` entries with structured package lists.
2. AI calls `env-template-gen` with the base image or base flake template (which might come from KG metadata or session config).
3. The CLI returns a Dockerfile or flake content and filename.
4. AI:
   - Presents the proposed template to the user for review, and/or
   - Emits an `ArtifactSpec` describing the new file so CI and Git review can apply it.
5. Humans decide when/where to build/push images or update Nix flakes.

At no point does the AI directly call `docker build` or `nix build`; it only shapes environment definitions.

***

## 6. Summary of Design Patterns

Across all these scenarios, a few patterns stay constant:

- **JSON contracts everywhere**: The AI talks to Rust tooling via small JSON schemas, not ad-hoc command strings.
- **Sonia as control plane**: SessionProfile, CiStatus, and the knowledge graph are the AI’s sources of truth for configuration, toolchains, and infra health.
- **CI digests instead of logs**: Raw logs (macro backtraces, panic messages, linker errors) are condensed into `CiFailure` objects with spans, messages, and structured fields.
- **Human-in-the-loop for destructive/devops actions**: Debugging with gdb, changing Docker/Nix infra, and modifying toolchains are always surfaced as proposals and artifacts, not autonomously executed steps.

Implementing the described CLIs (`rust-macro-expand`, `debug-build-script`, `env-template-gen`) and wiring them through `sessioncidigest.rs` and `SessionProfile` will greatly improve AI-chat compatibility with Cargo/Rust for Nintendoor64 and GAMEMODE.ai, while preserving determinism, safety, and Git-based review.
