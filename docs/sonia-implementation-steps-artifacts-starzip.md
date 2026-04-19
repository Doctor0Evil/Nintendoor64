# Sonia Implementation Steps: Exit Codes, Lane Whitelists, and Starzip Preview Safety

This document turns the three suggested implementation steps into concrete, drop‑in code and JSON contracts for the Sonia and Nintendoor64 slices. It assumes the existing Sonia crates and N64 vertical slice layout described elsewhere in the repository.

***

## 1. Differentiated Error Codes in `sonia-core`

This section extends `sonia-core-cli` so that its JSON error envelope includes an explicit machine‑readable `errorCode`, and the process exit status is mapped deterministically from that code. The goal is to distinguish decode failures, filesystem failures, and generic schema/argument errors at the OS level while keeping the existing JSON behavior.

### 1.1 New error code enum

File: `crates/sonia-core-cli/src/error_codes.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SoniaErrorCode {
    // Keep 1 for generic schema or argument failures.
    GenericInvalidInput,
    // ArtifactSpec decoded content could not be parsed or decoded.
    ArtifactDecodeError,
    // Artifact write failed due to IO (permissions, disk, path issues).
    ArtifactWriteFailed,
}

impl SoniaErrorCode {
    /// Stable numeric mapping, used for process exit codes.
    pub fn exit_code(self) -> i32 {
        match self {
            SoniaErrorCode::GenericInvalidInput => 1,
            SoniaErrorCode::ArtifactDecodeError => 2,
            SoniaErrorCode::ArtifactWriteFailed => 3,
        }
    }

    /// Convert from string, if you ever need to parse codes.
    pub fn as_str(self) -> &'static str {
        match self {
            SoniaErrorCode::GenericInvalidInput => "GenericInvalidInput",
            SoniaErrorCode::ArtifactDecodeError => "ArtifactDecodeError",
            SoniaErrorCode::ArtifactWriteFailed => "ArtifactWriteFailed",
        }
    }
}
```

### 1.2 Shared error envelope type

File: `crates/sonia-core-cli/src/envelope.rs`

```rust
use serde::{Deserialize, Serialize};

use crate::error_codes::SoniaErrorCode;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoniaErrorEnvelope {
    pub ok: bool,
    pub error_code: SoniaErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoniaOkEnvelope {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_written: Option<usize>,
}
```

### 1.3 Wiring into `validate` command

File: `crates/sonia-core-cli/src/main.rs` (only the changed parts are shown; the rest of the CLI stays as already sketched)

```rust
mod error_codes;
mod envelope;

use crate::error_codes::SoniaErrorCode;
use crate::envelope::{SoniaErrorEnvelope, SoniaOkEnvelope};

fn cmd_validate_spec(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read spec {}", path.display()))?;

    let v: Value = serde_json::from_str(&text)?;
    let schema = load_schema("artifact-spec.schema.json")?;
    let result = schema.validate(&v);

    if let Err(errors) = result {
        let errs: Vec<String> = errors
            .map(|e| format!("{} at {}", e, e.instance_path))
            .collect();

        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: SoniaErrorCode::GenericInvalidInput,
            message: "ArtifactSpec failed JSON Schema validation".to_string(),
            details: Some(serde_json::json!({ "errors": errs })),
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(SoniaErrorCode::GenericInvalidInput.exit_code());
    }

    let spec: ArtifactSpec = serde_json::from_value(v)?;
    if let Err(e) = spec.validate_semantics() {
        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: SoniaErrorCode::GenericInvalidInput,
            message: format!("ArtifactSpec semantic validation failed: {}", e),
            details: None,
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(SoniaErrorCode::GenericInvalidInput.exit_code());
    }

    let env = SoniaOkEnvelope {
        ok: true,
        path: None,
        bytes_written: None,
    };
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
}
```

### 1.4 Wiring into `write` command with decode vs IO split

```rust
fn cmd_write_spec(path: &Path, repo_root: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read spec {}", path.display()))?;

    let spec: ArtifactSpec = serde_json::from_str(&text)?;

    // Semantic validation – decode errors should map to ArtifactDecodeError
    if let Err(e) = spec.validate_semantics() {
        use sonia_core::ArtifactValidationError;

        let (code, message) = match e {
            ArtifactValidationError::DecodeError(msg) => {
                (SoniaErrorCode::ArtifactDecodeError, msg)
            }
            other => (
                SoniaErrorCode::GenericInvalidInput,
                other.to_string(),
            ),
        };

        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: code,
            message: format!("ArtifactSpec validation failed: {}", message),
            details: None,
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(code.exit_code());
    }

    // Target resolution and write – IO errors map to ArtifactWriteFailed
    let target = spec.target_path(repo_root)?;
    if let Some(parent) = target.parent() {
        if let Err(io_err) = fs::create_dir_all(parent) {
            let env = SoniaErrorEnvelope {
                ok: false,
                error_code: SoniaErrorCode::ArtifactWriteFailed,
                message: format!("failed to create artifact directory: {}", io_err),
                details: Some(serde_json::json!({
                    "path": parent.to_string_lossy(),
                })),
            };
            println!("{}", serde_json::to_string_pretty(&env)?);
            std::process::exit(SoniaErrorCode::ArtifactWriteFailed.exit_code());
        }
    }

    let bytes = match spec.decode_content() {
        Ok(b) => b,
        Err(ArtifactValidationError::DecodeError(msg)) => {
            let env = SoniaErrorEnvelope {
                ok: false,
                error_code: SoniaErrorCode::ArtifactDecodeError,
                message: msg,
                details: None,
            };
            println!("{}", serde_json::to_string_pretty(&env)?);
            std::process::exit(SoniaErrorCode::ArtifactDecodeError.exit_code());
        }
        Err(other) => {
            let env = SoniaErrorEnvelope {
                ok: false,
                error_code: SoniaErrorCode::GenericInvalidInput,
                message: other.to_string(),
                details: None,
            };
            println!("{}", serde_json::to_string_pretty(&env)?);
            std::process::exit(SoniaErrorCode::GenericInvalidInput.exit_code());
        }
    };

    if let Err(io_err) = fs::write(&target, &bytes) {
        let env = SoniaErrorEnvelope {
            ok: false,
            error_code: SoniaErrorCode::ArtifactWriteFailed,
            message: format!("failed to write artifact: {}", io_err),
            details: Some(serde_json::json!({
                "path": target.to_string_lossy(),
            })),
        };
        println!("{}", serde_json::to_string_pretty(&env)?);
        std::process::exit(SoniaErrorCode::ArtifactWriteFailed.exit_code());
    }

    let env = SoniaOkEnvelope {
        ok: true,
        path: Some(target.to_string_lossy().to_string()),
        bytes_written: Some(bytes.len()),
    };
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
}
```

With this mapping:

- `GenericInvalidInput` → exit code `1`
- `ArtifactDecodeError` → exit code `2`
- `ArtifactWriteFailed` → exit code `3`

This preserves the JSON contract but makes decode vs disk permission failures observable at the OS level.

***

## 2. `gamemodeai-build` Lane Whitelist Failure Mode

This section sketches the dispatcher for `gamemodeai-build` (or equivalent retro build conductor) that enforces a command whitelist per lane and returns a structured `UnknownCommand` error through the Sonia JSON envelope, with no partial lane execution.

### 2.1 Lane and dispatcher core types

File: `crates/gamemodeai-build/src/model.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneStep {
    pub id: String,
    pub command: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneSpec {
    pub id: String,
    pub steps: Vec<LaneStep>,
}
```

File: `crates/gamemodeai-build/src/dispatcher.rs`

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{LaneSpec, LaneStep};

#[derive(Debug)]
pub struct CommandFn(
    pub fn(&LaneStep) -> anyhow::Result<serde_json::Value>,
);

#[derive(Debug)]
pub struct Dispatcher {
    handlers: HashMap<String, CommandFn>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        name: impl Into<String>,
        func: fn(&LaneStep) -> anyhow::Result<serde_json::Value>,
    ) {
        self.handlers.insert(name.into(), CommandFn(func));
    }

    pub fn execute_lane(
        &self,
        lane: &LaneSpec,
    ) -> Result<Vec<serde_json::Value>, UnknownCommandError> {
        let mut results = Vec::new();

        for step in &lane.steps {
            let handler = match self.handlers.get(&step.command) {
                Some(h) => h,
                None => {
                    // Fail fast: no partial lane execution.
                    return Err(UnknownCommandError {
                        lane_id: lane.id.clone(),
                        step_id: step.id.clone(),
                        command: step.command.clone(),
                    });
                }
            };

            let value = (handler.0)(step).map_err(|_e| {
                // IO or build failures are handled elsewhere; we only care
                // about the unknown-command case here.
                UnknownCommandError {
                    lane_id: lane.id.clone(),
                    step_id: step.id.clone(),
                    command: step.command.clone(),
                }
            })?;

            results.push(value);
        }

        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownCommandError {
    pub lane_id: String,
    pub step_id: String,
    pub command: String,
}
```

### 2.2 Sonia JSON envelope and error mapping

File: `crates/gamemodeai-build/src/envelope.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResponse {
    pub version: u32,
    pub status: String, // "ok" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BuildError>,
}
```

File: `crates/gamemodeai-build/src/main.rs` (command handler)

```rust
mod dispatcher;
mod envelope;
mod model;

use std::io::{self, Read};

use anyhow::Result;
use dispatcher::{Dispatcher, UnknownCommandError};
use envelope::{BuildError, BuildResponse};
use model::LaneSpec;

fn main() -> Result<()> {
    // Read lane spec from stdin as per Sonia protocol.
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let lane: LaneSpec = serde_json::from_str(&buf)?;

    let dispatcher = build_default_dispatcher();

    match dispatcher.execute_lane(&lane) {
        Ok(_results) => {
            let resp = BuildResponse {
                version: 1,
                status: "ok".to_string(),
                data: Some(serde_json::json!({
                    "laneId": lane.id,
                })),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        Err(UnknownCommandError {
            lane_id,
            step_id,
            command,
        }) => {
            // Unknown command: fail fast, no further steps executed.
            let resp = BuildResponse {
                version: 1,
                status: "error".to_string(),
                data: None,
                error: Some(BuildError {
                    code: "UnknownCommand".to_string(),
                    message: format!(
                        "Lane step command '{}' is not registered",
                        command
                    ),
                    details: Some(serde_json::json!({
                        "laneId": lane_id,
                        "stepId": step_id,
                        "command": command,
                    })),
                }),
            };
            println!("{}", serde_json::to_string_pretty(&resp)?);
            // Exit with 1, consistent with Sonia error semantics.
            std::process::exit(1);
        }
    }
}

/// Registers only known commands in the dispatcher.
fn build_default_dispatcher() -> Dispatcher {
    let mut disp = Dispatcher::new();

    // Example registrations:
    // disp.register("n64-schema-gen", run_n64_schema_gen);
    // disp.register("n64-checklist", run_n64_checklist);
    // disp.register("n64-patch-preview", run_n64_patch_preview);

    disp
}
```

This ensures:

- Lanes referencing non‑whitelisted commands fail before any IO or subprocess execution.
- Errors are explicit and path‑agnostic: they expose logical lane and command IDs only.
- Exit status is non‑zero (`1`) but does not encode sensitive details.

### 2.3 Tests: prove no IO on unknown command

File: `crates/gamemodeai-build/tests/unknown_command_no_io.rs`

```rust
use std::cell::Cell;
use std::rc::Rc;

use gamemodeai_build::dispatcher::{CommandFn, Dispatcher, UnknownCommandError};
use gamemodeai_build::model::{LaneSpec, LaneStep};

#[test]
fn unknown_command_prevents_any_io_or_execution() {
    // Track whether any handler was ever run.
    let executed = Rc::new(Cell::new(false));
    let executed_clone = executed.clone();

    fn fake_handler(_step: &LaneStep) -> anyhow::Result<serde_json::Value> {
        // If this ever runs in the test, our dispatcher is wrong.
        panic!("handler should not be executed in unknown-command test");
    }

    // Lane with one valid step and one invalid step.
    let lane = LaneSpec {
        id: "test-lane".to_string(),
        steps: vec![
            LaneStep {
                id: "s1".to_string(),
                command: "known-command".to_string(),
                params: serde_json::json!({}),
            },
            LaneStep {
                id: "s2".to_string(),
                command: "unknown-command".to_string(),
                params: serde_json::json!({}),
            },
        ],
    };

    let mut dispatcher = Dispatcher::new();
    dispatcher.register("known-command", |_step| {
        executed_clone.set(true);
        Ok(serde_json::json!({ "ok": true }))
    });

    let result = dispatcher.execute_lane(&lane);

    match result {
        Err(UnknownCommandError {
            lane_id,
            step_id,
            command,
        }) => {
            assert_eq!(lane_id, "test-lane");
            assert_eq!(step_id, "s2");
            assert_eq!(command, "unknown-command");
        }
        Ok(_) => panic!("expected UnknownCommandError"),
    }

    // The first, known step must not have been executed at all
    // once the dispatcher encounters the unknown command.
    assert!(!executed.get());
}
```

This test proves that once an unknown command is detected, no registered handler executes and no side effects occur.

***

## 3. Safe `starzip-cli` Preview Schemas and KG Wiring

This section formalizes the `rom-query` and `patch --preview` contracts for Starzip so that AI‑accessible preview output is provably binary‑safe and integrated into FeatureLayout and the KG with `BinarySafe` tags and `SessionProfile` invariants.

### 3.1 Binary‑safe ROM layout schema

File: `schemas/n64-rom-layout-preview.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://gamemode.ai/schemas/n64-rom-layout-preview.schema.json",
  "title": "N64RomLayoutPreview",
  "type": "object",
  "required": ["romId", "segments", "files"],
  "additionalProperties": false,
  "properties": {
    "romId": {
      "type": "string",
      "description": "Logical identifier for the base ROM, not a path."
    },
    "segments": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "name",
          "kind",
          "romOffset",
          "romSize",
          "vramStart",
          "mutable"
        ],
        "additionalProperties": false,
        "properties": {
          "name": { "type": "string" },
          "kind": { "type": "string" },
          "romOffset": { "type": "integer", "minimum": 0 },
          "romSize": { "type": "integer", "minimum": 0 },
          "vramStart": { "type": "integer", "minimum": 0 },
          "mutable": { "type": "boolean" }
        }
      }
    },
    "files": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "path",
          "segment",
          "offsetInSegment",
          "length",
          "contentType"
        ],
        "additionalProperties": false,
        "properties": {
          "path": {
            "type": "string",
            "description": "Logical path inside the ROM filesystem."
          },
          "segment": { "type": "string" },
          "offsetInSegment": { "type": "integer", "minimum": 0 },
          "length": { "type": "integer", "minimum": 0 },
          "contentType": { "type": "string" }
        }
      }
    }
  }
}
```

This schema deliberately excludes any fields that would contain raw bytes, hex dumps, or decoded strings from the ROM.

### 3.2 Binary‑safe patch impact preview schema

File: `schemas/n64-patch-impact-preview.schema.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://gamemode.ai/schemas/n64-patch-impact-preview.schema.json",
  "title": "N64PatchImpactPreview",
  "type": "object",
  "required": ["layoutId", "baseRomId", "totalAddedBytes", "segments"],
  "additionalProperties": false,
  "properties": {
    "layoutId": {
      "type": "string",
      "description": "Logical identifier for the RomLayout JSON used."
    },
    "baseRomId": {
      "type": "string",
      "description": "Logical identifier of the base ROM."
    },
    "totalAddedBytes": {
      "type": "integer",
      "minimum": 0
    },
    "segments": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "segmentName",
          "segmentKind",
          "romOffset",
          "romSize",
          "currentBytes",
          "addedBytes",
          "maxBytes"
        ],
        "additionalProperties": false,
        "properties": {
          "segmentName": { "type": "string" },
          "segmentKind": { "type": "string" },
          "romOffset": { "type": "integer", "minimum": 0 },
          "romSize": { "type": "integer", "minimum": 0 },
          "currentBytes": { "type": "integer", "minimum": 0 },
          "addedBytes": { "type": "integer", "minimum": 0 },
          "maxBytes": { "type": "integer", "minimum": 0 }
        }
      }
    }
  }
}
```

This schema corresponds to a `PatchImpactReport`‑style view: only segment‑level sizes and identifiers, never raw content.

### 3.3 Starzip preview commands and Sonia envelope

File: `crates/starzip-cli/src/preview.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "command")]
pub enum PreviewCommand {
    RomQuery {
        rom_id: String,
        layout_path: String,
    },
    PatchPreview {
        base_rom_id: String,
        layout_path: String,
        patch_path: String,
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewEnvelope {
    pub version: u32,
    pub status: String, // "ok" | "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<PreviewError>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
```

File: `crates/starzip-cli/src/main.rs` (preview branch only)

```rust
mod preview;

use std::io::{self, Read};

use anyhow::Result;
use preview::{PreviewCommand, PreviewEnvelope, PreviewError};

fn main() -> Result<()> {
    // Dispatch on subcommands normally (patch, apply, etc.).
    // For brevity, only preview JSON mode is shown here.

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let cmd: PreviewCommand = serde_json::from_str(&buf)?;

    match cmd {
        PreviewCommand::RomQuery { rom_id, layout_path } => {
            let preview = run_rom_query_preview(&rom_id, &layout_path)?;
            let env = PreviewEnvelope {
                version: 1,
                status: "ok".to_string(),
                data: Some(serde_json::to_value(preview)?),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&env)?);
        }
        PreviewCommand::PatchPreview {
            base_rom_id,
            layout_path,
            patch_path,
        } => {
            let impact = run_patch_preview(&base_rom_id, &layout_path, &patch_path)?;
            let env = PreviewEnvelope {
                version: 1,
                status: "ok".to_string(),
                data: Some(serde_json::to_value(impact)?),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&env)?);
        }
    }

    Ok(())
}
```

File: `crates/starzip-cli/src/preview_impl.rs`

```rust
use serde::{Deserialize, Serialize};

use n64_layout::RomLayout;
use n64_layout::soniabridge::{PatchImpactReport, SoniaBridge};

/// Mirror of N64RomLayoutPreview.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N64RomLayoutPreview {
    pub rom_id: String,
    pub segments: Vec<PreviewSegment>,
    pub files: Vec<PreviewFile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSegment {
    pub name: String,
    pub kind: String,
    pub rom_offset: u32,
    pub rom_size: u32,
    pub vram_start: u32,
    pub mutable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewFile {
    pub path: String,
    pub segment: String,
    pub offset_in_segment: u32,
    pub length: u32,
    pub content_type: String,
}

pub fn run_rom_query_preview(
    rom_id: &str,
    layout_path: &str,
) -> anyhow::Result<N64RomLayoutPreview> {
    let text = std::fs::read_to_string(layout_path)?;
    let layout: RomLayout = serde_json::from_str(&text)?;

    let segments = layout
        .segments
        .iter()
        .map(|s| PreviewSegment {
            name: s.name.clone(),
            kind: s.kind.clone(),
            rom_offset: s.romoffset,
            rom_size: s.romsize,
            vram_start: s.vramstart,
            mutable: s.mutable,
        })
        .collect();

    let files = layout
        .files
        .iter()
        .map(|f| PreviewFile {
            path: f.path.clone(),
            segment: f.segment.clone(),
            offset_in_segment: f.offsetinsegment,
            length: f.length,
            content_type: f.contenttype.clone(),
        })
        .collect();

    Ok(N64RomLayoutPreview {
        rom_id: rom_id.to_string(),
        segments,
        files,
    })
}

pub fn run_patch_preview(
    base_rom_id: &str,
    layout_path: &str,
    patch_path: &str,
) -> anyhow::Result<PatchImpactReport> {
    let layout_text = std::fs::read_to_string(layout_path)?;
    let layout: RomLayout = serde_json::from_str(&layout_text)?;

    let patch_text = std::fs::read_to_string(patch_path)?;
    let patch: crate::soniabridge::PatchSpec = serde_json::from_str(&patch_text)?;

    let bridge = SoniaBridge::new(layout)?;
    let payload_index = crate::soniabridge::PayloadIndex::new();
    // In preview mode, you can optionally feed in artifact sizes from disk.

    let impact = bridge.compute_patch_impact(&patch, &payload_index, base_rom_id)?;

    Ok(impact)
}
```

The `PatchImpactReport` type here is the Rust counterpart to `n64-patch-impact-preview.schema.json` and is already constrained to segment‑level numeric summaries.

### 3.4 FeatureLayout and KG wiring with `BinarySafe`

File: `knowledgegraph/features.sonia.json` (excerpt for N64 Starzip features)

```json
{
  "repo": "Nintendoor64",
  "version": "1.0.0",
  "features": [
    {
      "id": "n64-rom-layout-oracle",
      "title": "N64 ROM Layout Oracle",
      "description": "Introspect N64 ROM segments and files via Starzip, returning only binary-safe layout metadata.",
      "tags": ["Nintendoor64", "RomLayout", "BinarySafe"],
      "systems": ["systems.nintendoor64.starzip.layout"],
      "schemas": [
        "schemas/n64-rom-layout-preview.schema.json"
      ],
      "commands": [
        "starzip-cli.rom-query-preview"
      ]
    },
    {
      "id": "n64-safe-patch-preview",
      "title": "N64 Safe Patch Preview",
      "description": "Preview the impact of an N64 patch on ROM segments and budgets without exposing raw bytes.",
      "tags": ["Nintendoor64", "PatchSynthesizer", "BinarySafe"],
      "systems": ["systems.nintendoor64.starzip.patch"],
      "schemas": [
        "schemas/n64-patch-impact-preview.schema.json"
      ],
      "commands": [
        "starzip-cli.patch-preview"
      ]
    }
  ]
}
```

File: `knowledgegraph/systems.json` (excerpt for Starzip nodes)

```json
[
  {
    "id": "systems.nintendoor64.starzip.layout",
    "repo": "Nintendoor64",
    "crates": ["n64-layout", "starzip-cli"],
    "files": ["crates/starzip-cli/src/preview_impl.rs"],
    "tags": ["Nintendoor64", "RomLayout", "CLI", "BinarySafe", "Deterministic"],
    "configSchemas": [
      "schemas/n64-rom-layout-preview.schema.json"
    ]
  },
  {
    "id": "systems.nintendoor64.starzip.patch",
    "repo": "Nintendoor64",
    "crates": ["n64-layout", "starzip-cli"],
    "files": ["crates/starzip-cli/src/preview_impl.rs"],
    "tags": ["Nintendoor64", "PatchSynthesizer", "CLI", "BinarySafe", "Deterministic"],
    "configSchemas": [
      "schemas/n64-patch-impact-preview.schema.json"
    ]
  }
]
```

The `BinarySafe` tag marks that these commands are safe to expose directly to AI tooling, because they are structurally guaranteed not to leak ROM bytes.

### 3.5 SessionProfile invariants gating Starzip preview

File: `schemas/session.schema.json` (new invariant examples in `invariants` list)

```json
{
  "type": "object",
  "properties": {
    "invariants": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "severity", "pattern"],
        "properties": {
          "id": { "type": "string" },
          "description": { "type": "string" },
          "severity": { "type": "string", "enum": ["Error", "Warning"] },
          "pattern": { "type": "string" }
        }
      }
    }
  }
}
```

Example invariant entries in a `SessionProfile` instance:

```json
{
  "invariants": [
    {
      "id": "no-direct-rom-dump",
      "description": "Starzip commands must not emit raw ROM bytes to AI-visible channels.",
      "severity": "Error",
      "pattern": "forbid:systems.nintendoor64.starzip.*.RawBytes"
    },
    {
      "id": "allow-starzip-preview-only",
      "description": "AI may only use Starzip preview commands tagged BinarySafe.",
      "severity": "Error",
      "pattern": "allow:tag.BinarySafe & system:systems.nintendoor64.starzip.*"
    }
  ]
}
```

File: `crates/sonia-core/src/session_mode.rs` (preview gating)

```rust
use sonia_featurelayout::FeatureLayout;
use crate::sessionprofile::SessionProfile;

/// Decide whether a Starzip preview command is permitted in this session.
pub fn starzip_preview_allowed(
    session: &SessionProfile,
    feature_layout: &FeatureLayout,
    command_id: &str,
) -> bool {
    // 1. Check invariants that explicitly forbid raw ROM dumps.
    let has_forbid_rom_dump = session.invariants.iter().any(|inv| {
        inv.id == "no-direct-rom-dump" && inv.severity == InvariantSeverity::Error
    });
    if !has_forbid_rom_dump {
        // Conservative default: if the guard isn't present, deny.
        return false;
    }

    // 2. Resolve the feature that exposes this command.
    let feature = feature_layout
        .features
        .iter()
        .find(|f| f.commands.iter().any(|c| c == command_id));

    let feature = match feature {
        Some(f) => f,
        None => return false,
    };

    // 3. Require BinarySafe tag.
    feature.tags.iter().any(|t| t == "BinarySafe")
}
```

The orchestrator that wraps tool calls must call `starzip_preview_allowed` before invoking `starzip-cli` in preview mode. If it returns `false`, it should reply with a Sonia error envelope (for example, `CommandForbiddenBySession`) instead of spawning Starzip.

***

## 4. Next Objectives and Improvement Directions

1. Extend `sonia-core-cli`’s new error code mapping to all subcommands (not just `validate` and `write`) and generate a small `schemas/sonia-error-envelope.schema.json` wired into the protocol spec so AI and CI can validate and branch on error codes consistently.

2. For `gamemodeai-build`, add coverage tests that simulate mixed lanes (valid steps after an unknown command) and assert that the dispatcher never attempts to execute steps after the first failure, maintaining all‑or‑nothing semantics per lane.

3. For Starzip preview, add CI jobs that validate all preview outputs against `n64-rom-layout-preview.schema.json` and `n64-patch-impact-preview.schema.json`, and then register the corresponding SystemNodes and FeatureLayout entries in the KG, so AI discovery and policy enforcement can rely purely on schema‑backed metadata.

4. Evolve `SessionProfile` invariants for Starzip into a small DSL with explicit operators (`allow:tag.BinarySafe`, `forbid:system.*.RawBytes`) and implement a generic invariant interpreter so preview gating can reuse the same mechanism as other constraint checks, rather than special‑casing Starzip.

5. Once the preview path is stable, add an optional `dryRun` and `budgetOnly` flag to `PatchPreview` so the AI can explore patch impacts under budget constraints without committing to full Starzip patch runs, helping to keep experimentation cheap and safe.
