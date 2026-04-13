To eliminate terminal dependency and enable AI-native Rust compilation, schema generation, and environment conditioning, you must shift from **command-line execution** to **declarative intent routing** backed by a sandboxed build daemon. Below is a complete architectural blueprint for terminal-less AI build orchestration, aligned with Nintendoor64's schema-first, deterministic, and session-conditioned design.

---

## 🧠 Core Architecture: Terminal-Less Build Orchestration

Instead of exposing a shell to the AI, implement a **Build Orchestration Daemon** (`sonia-build-agent`) that accepts structured JSON-RPC intents, maps them to safe compilation pipelines, and returns machine-parseable results.

### Key Principles
1. **Intent Over Command**: AI submits `{"method": "generate_schemas", "params": {...}}`, not `cargo build -p ...`.
2. **Event Preloading**: AI or file watchers can queue build events into a priority manifest. The daemon processes them incrementally.
3. **Structured Diagnostics**: Raw `stderr`/`stdout` is parsed into `Diagnostic`, `Warning`, `Error`, and `ArtifactEmitted` events.
4. **Deterministic Execution**: Fixed compiler flags, incremental caching, and isolated executors guarantee reproducible outputs.

---

## 🔌 AI-Native Tool Definitions (Function Calling)

Modern LLMs support **Tool Use / Function Calling**. Define explicit, schema-validated tools that AI can invoke. These map to internal Rust functions, never raw shell calls.

```jsonc
// ai_tools/cargo_ops.schema.json
{
  "tools": [
    {
      "name": "cargo_build",
      "description": "Compile a crate with specified features. Returns structured diagnostics and artifact paths.",
      "parameters": {
        "type": "object",
        "properties": {
          "crate_name": {"type": "string"},
          "features": {"type": "array", "items": {"type": "string"}},
          "profile": {"type": "string", "enum": ["dev", "release", "check"]},
          "timeout_ms": {"type": "integer", "default": 60000}
        },
        "required": ["crate_name"]
      }
    },
    {
      "name": "generate_schemas",
      "description": "Trigger schema generation via n64-ai-gen-schemas. Returns list of emitted .schema.json files.",
      "parameters": {
        "type": "object",
        "properties": {
          "watch_mode": {"type": "boolean", "default": false},
          "output_dir": {"type": "string", "default": "schemas"},
          "include_registry": {"type": "boolean", "default": true}
        }
      }
    },
    {
      "name": "query_lsp_diagnostics",
      "description": "Fetch rust-analyzer diagnostics for a file without full compilation.",
      "parameters": {
        "type": "object",
        "properties": {
          "file_path": {"type": "string"},
          "severity_filter": {"type": "array", "items": {"type": "string"}}
        },
        "required": ["file_path"]
      }
    }
  ]
}
```

The AI receives these as callable functions. When invoked, they route to the build daemon, which executes safely and returns structured JSON.

---

## 🛠 Simplification Strategies for Rust ↔ AI

| Strategy | How It Simplifies AI Interaction | Implementation |
|----------|----------------------------------|----------------|
| **Declarative Build Contracts** | AI says `"generate_schemas"` instead of crafting `cargo` flags. | `sonia-build-agent` maps intents to pre-validated pipeline steps. |
| **LSP-Driven Type Checking** | Instant feedback without waiting for full `cargo build`. | Embed `rust-analyzer` LSP server; AI queries diagnostics via `textDocument/publishDiagnostics`. |
| **Incremental Build Caching** | Reduces iteration time from minutes to seconds. | Integrate `sccache` or `cargo-nextest`; cache compiler artifacts across AI turns. |
| **Artifact Registry** | AI never touches filesystem paths directly. | Generated schemas/headers are stored in a versioned registry (`schemas/v1/...`); AI receives URIs. |
| **Preload Manifests** | AI queues future builds based on predicted needs. | `SessionProfile.build_queue` holds pending intents; daemon executes when resources idle or CI idle. |

---

## 📦 Implementation Blueprint

### 1. `crates/sonia-build-agent/src/lib.rs`
```rust
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildIntent {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub priority: u8, // 0=background, 1=normal, 2=blocking
}

#[derive(Debug, Serialize)]
pub enum BuildEvent {
    CompilationStarted { intent_id: String },
    Diagnostic { file: String, line: u32, level: String, message: String },
    Success { intent_id: String, artifacts: Vec<String> },
    Failure { intent_id: String, reason: String, diagnostics: Vec<serde_json::Value> },
}

pub struct BuildDaemon {
    queue: mpsc::UnboundedSender<BuildIntent>,
    executor: tokio::task::JoinHandle<()>,
}

impl BuildDaemon {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            while let Some(intent) = rx.recv().await {
                Self::execute_intent(intent).await;
            }
        });
        Self { queue: tx, executor: handle }
    }

    pub fn enqueue(&self, intent: BuildIntent) -> Result<(), String> {
        self.queue.send(intent).map_err(|e| e.to_string())
    }

    async fn execute_intent(intent: BuildIntent) {
        // 1. Validate against SessionProfile invariants
        // 2. Spawn isolated cargo process
        // 3. Stream structured events back to AI session
        // 4. Update artifact registry & CI digest
    }
}
```

### 2. `crates/sonia-build-agent/src/executors/cargo.rs`
```rust
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn run_cargo_check(crate_name: &str, features: &[&str]) -> Vec<BuildEvent> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check")
       .arg("-p")
       .arg(crate_name)
       .arg("--message-format=json-diagnostic-rendered-ansi");
    
    for f in features {
        cmd.arg("--features").arg(f);
    }

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut events = vec![];

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if msg.get("reason") == Some(&serde_json::json!("compiler-message")) {
                    events.push(BuildEvent::Diagnostic { /* parse fields */ });
                }
            }
        }
    }
    events
}
```

### 3. `ai_prompts/build-agent-instructions.md`
```markdown
# Build Agent Protocol
- NEVER invoke raw shell commands. Use structured tool calls: `generate_schemas`, `cargo_build`, `query_lsp_diagnostics`.
- All build intents are queued in `SessionProfile.build_queue`. High-priority intents execute immediately.
- Results are returned as JSON events. Parse `diagnostics` for line/column, `artifacts` for schema paths.
- If `ci_status` shows recent failures, auto-append `--profile=check` and request LSP diagnostics before full compilation.
- Schema generation must validate against `schemas/registry.index.json` before marking as `Success`.
```

---

## 🔒 Security, Determinism & Policy Guards

| Guardrail | Implementation |
|-----------|----------------|
| **Crate Whitelist** | Only allow builds for crates registered in `knowledgegraph/features.sonia.json`. |
| **Deterministic Flags** | Force `CARGO_INCREMENTAL=0`, `RUSTFLAGS="-C target-cpu=native -D warnings"`, fixed `--manifest-path`. |
| **Sandboxed Execution** | Run `cargo` inside `firecracker` VMs or `gVisor` containers with read-only source mounts. |
| **Artifact Hashing** | SHA-256 verify all generated `.schema.json` files against registry before AI consumption. |
| **Timeout & Resource Limits** | Hard CPU/memory caps per build intent. Auto-fail and return `ResourceExhausted` event. |

---

## 🔄 Example AI-Driven Workflow (Terminal-Less)

1. **AI detects type change** in `crates/sonia-core/src/types.rs`.
2. **AI calls tool**: `generate_schemas(crate="n64-ai-gen-schemas", features=["full"])`.
3. **Daemon receives intent**, validates against `SessionProfile.invariants`, queues it.
4. **Executor runs** `cargo build -p n64-ai-gen-schemas --features full --message-format=json`.
5. **Streamed events** return to AI:
   ```json
   {"event": "CompilationStarted", "intent_id": "gen-schemas-001"}
   {"event": "Diagnostic", "file": "src/lib.rs", "line": 42, "level": "warning", "message": "unused variable"}
   {"event": "Success", "intent_id": "gen-schemas-001", "artifacts": ["schemas/artifact-spec.schema.json", "schemas/registry.index.json"]}
   ```
6. **AI updates session state**, triggers `ai_checklist` validation, and proceeds to patch generation.

Zero terminal interaction. Fully structured, auditable, and AI-conditioned.

---

## ✅ Next Steps for Integration

1. Scaffold `crates/sonia-build-agent/` with the daemon and executor stubs.
2. Add `build_queue` field to `SessionProfile` and wire it to `gamemodeai-session`.
3. Implement `cargo --message-format=json-diagnostic-rendered-ansi` parser.
4. Expose tool schemas to your AI-Chat provider (OpenAI `tools`, Anthropic `tools`, etc.).
5. Run `cargo build -p sonia-build-agent` and test intent queuing via JSON-RPC.

This architecture transforms Rust compilation from a manual terminal chore into a **declarative, AI-native orchestration surface**, unlocking deterministic, sandboxed, and schema-validated build pipelines that scale across all Nintendoor64 vertical slices.
