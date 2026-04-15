# wayback-net-architecture.md

## Scope and intent

This document turns the 100-question lattice for the "Internet of Games" / Wayback Machine for multiplayer into a concrete, multi-phase architecture and research roadmap.

It is meant to live in a repo docs/wayback-net-architecture.md and be treated as a living design doc that AI-chat and humans can both extend. It does not attempt to fully answer every question yet; instead it clusters them into systems, defines initial constraints, and specifies first-pass implementations and files for each subsystem.

## High-level system decomposition

At the top level, the platform is decomposed into:

- Core packet and transport layer: interception, replay, simulation, and adaptation of legacy game traffic in Rust.
- Scripting and behavior layer: Lua (and possibly Luau) scripts that define game-specific protocol handling and behaviors.
- AI and assistance layer: local LLMs, STT, and RAG for support, moderation, and protocol reverse engineering.
- Browser/WASM gateway: WebAssembly-based thin client and lobby, bridging WebRTC to raw sockets.
- Orchestration and governance: libp2p-based discovery, metrics, voting, and update propagation.

Each of the 5 sections of the question set maps onto these layers. The rest of this document defines concrete targets and file layouts for a first implementation pass, plus next research objectives.

## Section 1: core proxy and "Wayback" protocol layer

### S1.1 Core crate layout

Create a Rust workspace with at least these crates:

- crates/wayback-core-net: core packet IO, TUN/TAP drivers, NAT helpers, traffic shaping.
- crates/wayback-proxy: high-level proxy that wires core-net, Lua hooks, and circuit breakers.
- crates/wayback-detect: signatures and heuristics for detecting game engines and protocol variants.
- crates/wayback-config: typed config layer + hot-reload server.
- crates/wayback-ebpf: Linux-only eBPF integration.
- crates/wayback-ws-bridge: WebSocket and WebRTC bridges.

Each crate must have a clear, JSON-configurable boundary so AI-chat flows can reason about its responsibilities.

### S1.2 Transparent L2/L3 proxy (Q1, Q11, Q12, Q21, Q22)

Target: a transparent proxy that can sit on a host and intercept traffic for a set of legacy games without per-game OS configuration.

First-pass design choices:

- Use a userspace TUN/TAP adapter (e.g., via platform-specific crates: `tun` on Linux, `tap-windows` or Wintun FFI on Windows) managed by wayback-core-net.
- Use libpnet or rscap for raw packet RX/TX when TUN/TAP is unavailable or when L2 inspection is required.
- Implement a minimal IP and UDP/TCP classifier that maps traffic to per-game handlers.

File: crates/wayback-core-net/src/lib.rs

Responsibilities:

- Expose an async API:
  - `async fn start_tun(config: TunConfig) -> Result<TunHandle>`
  - `async fn run_forwarder(handle: TunHandle, rules: Arc<RoutingRules>)`
- Provide a platform-abstracted `NetPacket` type that can represent Ethernet + IPv4/IPv6 + UDP/TCP headers plus payload.
- Provide helpers to construct replies (e.g., spoofed master server packets) without copying payloads more than once, using tokio-uring on Linux for zero-copy paths.

Research objectives:

- Evaluate libpnet vs rscap and pkts.rs for raw capture and injection, especially around performance and platform support.
- For Windows and macOS, confirm the viability of TUN/TAP vs NDIS filter drivers; document limitations.
- Prototype a 1:1 packet echo proxy to validate throughput and latency budget.

### S1.3 DNS and master server redirection (Q2)

Target: override legacy DNS lookups (GameSpy, Westwood Online, Battle.net, etc.) without breaking system-wide HTTPS DNS resolution.

Strategy:

- Ship a small recursive DNS resolver in Rust using hickory-resolver for game-related domains only.
- Use per-process DNS override techniques when possible (LD_PRELOAD on Linux to override getaddrinfo, DLL injection with detours on Windows) for games launched via the Wayback client.
- For embedded stacks (inside a TUN-based virtual network), treat the resolver as authoritative for specific zones and forward all others to the OS resolver.

File: crates/wayback-core-net/src/dns.rs

Responsibilities:

- Maintain a mapping of legacy hostnames to new master server endpoints.
- Optionally read hosts overrides from a JSON or TOML config that the Lua scripting layer can modify at runtime.
- Provide a local UDP DNS listener bound to 127.0.0.1:5353 or the TUN address and respond only to configured zones.

### S1.4 Engine fingerprinting via packet signatures (Q4)

Target: passively detect engine family (Quake III, GoldSrc, RenderWare, GameSpy, etc.) from the first few packets to map a connection to a handler.

Strategy:

- Use rscap + pkts to capture packets in a BPF-filtered stream on relevant ports.
- Maintain a table of signatures keyed by a short window of bytes from the first L4 payloads (e.g., first 16 bytes of UDP or the first application-layer message in TCP).
- Use bitmask and wildcard patterns rather than exact equality where games include version numbers or timestamps.

File: crates/wayback-detect/src/signatures.rs

Responsibilities:

- Define a `Signature` struct with fields like `offset`, `pattern`, `mask`, and `engine_id`.
- Load signatures from a versioned YAML file under data/signatures/*.yml.
- Provide `fn detect_engine(payload: &[u8]) -> Option<EngineId>` that can be called from the proxy.

First-pass research objects:

- Collect known handshake examples for at least three engines and encode them as signatures.
- Validate detection against `.pcap` captures of live games.

### S1.5 Game-specific adapters and plugins (Q5, Q6, Q14, Q18)

Target: keep the core proxy generic, while game-specific quirks live in dynamically-loadable plugins that can be written in C++ or Rust and optionally expose Lua hooks.

Design:

- Define a C ABI for game plugins, with functions like:
  - `int wg_game_init(const struct wg_game_context* ctx);`
  - `int wg_game_handle_packet(const struct wg_packet* in, struct wg_packet* out);`
  - `void wg_game_shutdown(void);`
- Implement a Rust-side plugin manager in crates/wayback-proxy that loads `.so`/`.dll` files via libloading and exposes a safe wrapper.
- Provide a thin C++ helper library that game reverse engineers can link against to implement the plugin interface.

Memory safety considerations:

- Avoid passing raw pointers owned by the game into Rust code; all cross-boundary data should be copied into owned buffers or view types that use lifetimes on the Rust side.
- For injected DLLs, keep Rust in charge of allocation; C++ code should call back into Rust APIs rather than vice versa when manipulating game memory.

File: crates/wayback-proxy/src/plugin.rs

Responsibilities:

- Manage plugin discovery based on detected EngineId and game profile.
- Provide per-connection state objects so plugins are stateless across games.

### S1.6 Deterministic lockstep and rollback (Q6, Q18, Q19)

Target: provide lockstep/rollback simulation support to retro RTS and fighting games that had weak netcode primitives.

Strategy:

- Reuse or mirror the deterministic ECS design from your existing GAMEMODE.ai core; treat netcode as a separate crate `crates/wayback-netcode`.
- For RTS lockstep, define an input-recording protocol where all clients send command frames; the relay ensures ordering and delivery with sequence numbers, but no world state is transmitted.
- For fighting games, integrate a GGPO-style rollback algorithm that maintains a window of snapshots and rewinds when late inputs arrive.

File: crates/wayback-netcode/src/lockstep.rs and rollback.rs

Responsibilities:

- Abstract over raw sockets so the same logic can sit on top of UDP, WebRTC data channels, or a TUN-based channel.
- Provide pluggable latency and jitter simulation (traffic shaping) controlled via config or Lua.

### S1.7 WebSocket / WebRTC bridges and browser lobby (Q7, Q9, Q10, Q13, Q20)

Target: allow browser-based lobbies and light participation in games, while heavy traffic flows through native relays.

Design:

- Use warp or axum plus tokio-tungstenite in crates/wayback-ws-bridge to expose a JSON + binary WebSocket API.
- For P2P, wrap str0m or another WebRTC crate so that a browser client can signal via WebSocket and then talk over data channels.
- Use smoltcp in a WASM build to simulate a minimal IP stack for browser-based LAN emulation.

File: crates/wayback-ws-bridge/src/lib.rs

Responsibilities:

- Map WebSocket JSON messages representing lobby commands into internal calls.
- Offer a bridge that forwards binary payloads between WebRTC data channels and raw UDP sockets.

### S1.8 NAT traversal and IPv6 mapping (Q9, Q24)

Target: connect IPv4-only games through IPv6-only community nodes with NAT traversal.

Design:

- Implement a small STUN-like discovery and mapping protocol that uses `str0m` or another WebRTC stack to negotiate connectivity, then optionally downgrades to UDP.
- Provide an address translation module that maps IPv6 addresses to virtual IPv4 ranges inside the TUN network, with deterministic mapping.

File: crates/wayback-core-net/src/nat.rs

Responsibilities:

- Maintain a mapping table from `(peer_id, v4_addr)` to `(ipv6_addr, port)`.
- Apply translation when reading/writing packets through the TUN interface.

### S1.9 Observability, backpressure, and circuit breaking (Q16, Q17, Q23)

Target: keep the proxy healthy when community scripts misbehave.

Design:

- Instrument all async tasks with Tokio tracing+, and expose spans to tokio-console.
- Build a per-script budget system that tracks max CPU time, packets per second, and errors.
- Implement circuit breakers that can disable a script at runtime if it exceeds limits, while keeping the proxy running.

File: crates/wayback-proxy/src/circuit_breaker.rs

Responsibilities:

- Wrap calls into Lua or WASM handlers.
- Maintain per-handler metrics and expose them via Prometheus-compatible endpoints.

### S1.10 Config hot-reload (Q25)

Target: push new game definitions and Lua scripts to connected nodes without restarting.

Design:

- Use wayback-config to manage a versioned config tree under config/.
- Clients subscribe via a simple gRPC or WebSocket feed to config updates.
- Use arc-swap or RCU-like patterns in Rust to swap in new handler tables atomically.

File: crates/wayback-config/src/server.rs

Responsibilities:

- Watch the filesystem for changes.
- Push signed config bundles to clients.

## Section 2: Lua integration and community scripting

### S2.1 Lua VM strategy (Q26, Q27, Q30, Q35, Q42)

Target: isolate game scripts tightly, while exposing rich APIs.

Design:

- Prefer mlua for embedding standard Lua 5.4 with good async integration, plus optional Luau for typed scripts.
- Do not expose os.execute, io.open, or raw file system APIs; instead provide a virtual file system API in Rust.
- Maintain one Lua context per game session for predictability; optionally pool VMs for short-lived tasks.

File: crates/wayback-scripting/src/engine.rs

Responsibilities:

- Expose registration APIs for Rust to Lua (`register_function`, `register_type`).
- Manage sandboxes with explicit capabilities per script.

### S2.2 Async Rust APIs in synchronous Lua (Q26, Q32, Q36)

Target: let Lua call into async Rust without blocking the reactor.

Design:

- Use mlua's support for futures to expose functions that return promises.
- For simple synchronous Lua, wrap async tasks in a scheduler that yields via `lua_yield` and resumes when futures complete.

File: crates/wayback-scripting/src/async_bridge.rs

Responsibilities:

- Provide helper macros to expose async Rust functions as Lua functions that yield until ready.
- Ensure packet processing uses bounded queues and timeouts so backlog cannot grow unbounded.

### S2.3 Bytecode verification and resource limits (Q27, Q28)

Target: prevent infinite loops and memory bombs.

Design:

- Reject precompiled bytecode entirely for untrusted scripts; always compile from source with a restricted compiler.
- Implement an instruction count or step limit per script by instrumenting Lua hooks (debug.sethook) to yield after N instructions.
- Limit memory via a custom Lua allocator that enforces per-VM heap caps.

Files:

- crates/wayback-scripting/src/sandbox.rs
- docs/specs/wayback-lua-security.md

### S2.4 State machines for legacy services (Q29, Q39)

Target: describe legacy protocols like GameSpy login and map file formats as data-driven state machines.

Design:

- Define a JSON schema for protocol state machines:
  - States, transitions, expected packet patterns.
- Generate Lua state machines from these schemas.

Files:

- docs/schemas/protocol-sm.schema.json
- data/protocols/gamespy-login.json
- scripts/protocols/gamespy_login.lua

### S2.5 DAP and debugging support (Q33, Q38, Q44)

Target: allow step-through debugging of Lua packet handlers against `.pcap` traces.

Design:

- Implement a DAP bridge process in Rust (crates/wayback-debug-dap) that exposes a debug adapter to VS Code and internally controls the Lua VM.
- Provide a test harness CLI `wayback-lua-test` that replays `.pcap` files through a script and reports diffs.

Files:

- crates/wayback-debug-dap/src/main.rs
- crates/wayback-lua-test/src/main.rs

### S2.6 API versioning (Q34, Q40, Q41, Q45)

Target: keep Lua APIs stable while evolving the platform.

Design:

- Introduce a versioned Lua API module `wayback_v1`, `wayback_v2` etc.
- Use a manifest per game profile that pins the API version.
- Deprecate APIs by logging warnings first and later gating them behind a feature flag.

Files:

- docs/specs/wayback-lua-api-v1.md
- docs/specs/wayback-lua-api-v2.md

## Section 3: local AI-chat and LLM integration

### S3.1 LLM runtime and hardware envelope (Q46, Q52, Q56, Q60, Q64)

Target: embed a small LLM runtime capable of running 1.5B parameter models locally, but make it optional.

Design:

- Wrap candle, llama-cpp-rs, or mistral.rs in a dedicated crate `crates/wayback-llm`.
- Define a strict capability boundary: the proxy must run correctly without the LLM present.
- Use a separate OS thread pool or process for inference to avoid stealing cycles from time-critical proxy paths.

Files:

- crates/wayback-llm/src/lib.rs
- docs/specs/wayback-llm-deployment.md

### S3.2 RAG pipeline and protocol assistance (Q47, Q51, Q55, Q62)

Target: allow on-device RAG over local docs, game schemas, and packet captures.

Design:

- Use tantivy or similar for full-text indexing of docs/ and extracted protocol descriptions.
- Use a small embedding model in Rust to index game descriptions, error logs, and protocol docs.
- Expose a CLI `wayback-rag-query` that the chat UI can call with a question.

Files:

- crates/wayback-rag/src/lib.rs
- tools/wayback-rag-query/src/main.rs

### S3.3 Moderation, privacy, and NAT assistance (Q50, Q57, Q58, Q59, Q63)

Target: make AI helpful but safe for moderation and connection help.

Design:

- Use federated learning or simple on-device fine-tuning for sentiment, but never upload raw chat logs.
- Represent player network status as a small context struct that can be summarized into prompt text.
- Restrict AI from emitting sensitive data like public IPs; enforce this in the prompt templates and by post-processing responses.

Files:

- docs/specs/wayback-privacy-and-logging.md
- docs/specs/wayback-ai-moderation.md

### S3.4 Tokenization and jargon (Q54, Q61, Q65)

Target: teach tokenizers about retro FPS/RTS jargon.

Design:

- Maintain a small custom vocabulary list for common game terms and interjections.
- Apply user-defined special tokens or tokenizer training data snippets for terms like `bfg`, `gibs`, `rush B`.

Files:

- data/llm/custom_vocab.txt

## Section 4: WASM/browser gateway

### S4.1 Shared networking library for native + WASM (Q66, Q69, Q71, Q72, Q76, Q79)

Target: one Rust networking core that compiles to both native and wasm32-unknown-unknown.

Design:

- Create `crates/wayback-net-common` that is `no_std`-friendly with a small feature set.
- Wrap WASM-specific bindings in a separate crate `crates/wayback-net-wasm` using wasm-bindgen and web-sys.
- Ensure that all OS-specific networking is behind traits that have WASM implementations via WebRTC or WebSocket.

Files:

- crates/wayback-net-common/src/lib.rs
- crates/wayback-net-wasm/src/lib.rs

### S4.2 Rollback netcode in browser (Q67, Q71, Q72, Q74, Q75, Q77, Q78, Q80)

Target: allow small 2-4 player matches through browser emulators.

Design:

- Mirror the rollback engine from wayback-netcode in WASM, with snapshots stored in SharedArrayBuffer-backed ring buffers.
- Use WebRTC data channels for gamepad inputs and state synchronization.
- Bind to the browser Gamepad API, Gamepad events mapped to input frames.

Files:

- web/wayback-lobby/ (Yew or Leptos project using trunk or wasm-pack)
- web/wayback-lobby/src/netcode.rs
- web/wayback-lobby/src/gamepad.rs

## Section 5: decentralized orchestration and governance

### S5.1 Discovery and DHT (Q81, Q86, Q93)

Target: master-server-less discovery for servers and relay nodes.

Design:

- Use libp2p in `crates/wayback-orchestrator` with Kademlia DHT.
- Treat game servers and relay nodes as libp2p peers advertising records with game IDs and tags.
- Use DNS TXT records as a worst-case bootstrap for initial bootstrap peers.

Files:

- crates/wayback-orchestrator/src/lib.rs
- docs/specs/wayback-dht-and-bootstrap.md

### S5.2 Proof of Gameplay and Sybil resistance (Q82, Q83, Q90, Q95)

Target: keep votes and metrics meaningful without KYC.

Design:

- Start with simple anti-Sybil measures: per-device keys, soft IP-based rate limiting, and weight votes by gameplay hours recorded locally.
- Optionally anchor weekly aggregated vote hashes to a lightweight blockchain or log store, but do not require this for basic operation.

Files:

- docs/specs/wayback-governance-and-pog.md

### S5.3 Capacity, microVMs, and hibernation (Q88, Q96, Q97)

Target: spin up and down servers on volunteer nodes.

Design:

- Define a simple orchestrator-agent protocol where a local agent can accept tasks to spawn containers or Firecracker microVMs.
- Use eBPF or sysinfo to monitor CPU, memory, and idle status and report it back.

Files:

- crates/wayback-orchestrator/src/agent.rs
- docs/specs/wayback-orchestrator-agent.md

### S5.4 UX story and self-updating binaries (Q89, Q98, Q99, Q100)

Target: make the flow "Install, detect games, click Play" as simple as possible, while keeping the project maintainable.

Design:

- Use cargo-dist and axoupdater to produce self-updating binaries, signed and versioned.
- Embed an onboarding script that scans Steam and GOG libraries for known games based on manifests.
- Maintain thorough ADRs (Architecture Decision Records) alongside the code for each major subsystem.

Files:

- docs/adr/0001-core-proxy-and-tun.md
- docs/adr/0002-lua-scripting-model.md
- docs/adr/0003-llm-integration.md
- docs/adr/0004-p2p-discovery.md

## Next objectives and coding suggestions

1. Stand up the minimal core-net and proxy loop.
   - Implement crates/wayback-core-net with a TUN echo proxy.
   - Add crates/wayback-proxy with a simple rule that logs packets and forwards unchanged.
2. Add engine detection and a single hardcoded Quake 3 or GoldSrc handshake signature.
3. Define the plugin ABI header in C and the Rust loader; ship an empty sample plugin crate in C++ under plugins/sample-engine/.
4. Introduce mlua with a tiny scripting entry point where a Lua script can rewrite a packet field (e.g., swap ports) and validate that circuit breakers work.
5. Add a small Yew or Leptos WASM lobby that can connect over WebSocket to a local wayback instance and list active games.
6. Start ADRs immediately; treat every major design choice as an ADR entry so future contributors can reconstruct the intent.

This document should be checked into the repo at docs/wayback-net-architecture.md and treated as the root index for the Wayback networking stack. Future iterations can promote subsections here into dedicated specs as the implementation matures.