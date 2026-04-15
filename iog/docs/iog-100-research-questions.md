# Internet of Games Research Index (100 Questions)

This document tracks the core research questions that define the Internet of Games (IoG) “Wayback Machine for Multiplayer” stack, grouped by subsystem and cross-linked to concrete crates, specs, and implementation statuses.

- Architecture reference: `docs/iog-overview.md`
- Net architecture reference: `docs/iog-net-architecture.md`
- Workspace root: `iog/Cargo.toml`
- Core crates:
  - `crates/wayback-core-net`
  - `crates/wayback-dns`
  - `crates/wayback-dht`
  - `crates/wayback-webrtc-bridge`
  - `crates/wayback-proxy`
  - `crates/wayback-scripting`
  - `crates/iog-protocol-model`

Each question has:
- **ID** – stable anchor used in code comments, issues, and PRs.
- **Status** – `todo | designing | prototyping | implemented | verified`.
- **Crates** – primary crates that should own the implementation.
- **Artifacts** – schemas, specs, or example files that should exist when the question is considered “done”.

---

## Section 1 – Core Architecture & Transparent Network Interception (Q1–Q20)

Goal: intercept and redirect legacy game traffic without modifying binaries or global OS configuration, using TUN/TAP, IP_TRANSPARENT, io_uring, eBPF, and optional API hooking.

### Q1 – Userspace TUN/TAP without admin privileges

**Question:**  
How can we implement a userspace TUN/TAP device in Rust that operates entirely without kernel modules or administrator privileges on Windows, macOS, and Linux?

- **ID:** `Q1-tun-userspace`
- **Status:** `designing`
- **Crates:** `wayback-core-net`
- **Artifacts:**
  - `crates/wayback-core-net/src/lib.rs` – cross-platform `TunLink` abstraction.
  - `crates/wayback-core-net/src/bin/echo-tun.rs` – echo proxy demo.
- **Research objects:**
  - `tun` crate
  - `nix`
  - `libpnet::datalink`
  - Wintun bindings

Implementation notes:

- Phase 1 uses `tun` crate and a simple echo loop (already sketched in `wayback-core-net`), primarily for Linux/macOS. [file:1]
- For Windows, investigate Wintun-based userspace adapters and document any unavoidable privilege requirements in `docs/iog-net-architecture.md`. [file:1]

---

### Q2 – TUN FD sharing across async tasks

**Question:**  
What are the memory safety and concurrency implications of sharing a raw file descriptor for a TUN device across multiple async tasks in a `tokio` runtime?

- **ID:** `Q2-tun-fd-sharing`
- **Status:** `todo`
- **Crates:** `wayback-core-net`
- **Artifacts:**
  - `docs/iog-net-architecture.md` section `Core Net: TUN FD ownership and concurrency`.
  - Benchmark or test: `crates/wayback-core-net/tests/tun_shared_fd.rs`.
- **Research objects:**
  - `tokio::io::unix::AsyncFd`
  - `mio`
  - `socket2::SockRef`

Implementation notes:

- Decide whether to multiplex reads in a single task with fan-out, or allow multiple `AsyncFd` owners with careful read loops.
- Establish an ownership/routing model that avoids head-of-line blocking when multiple games share one TUN. [file:1]

---

### Q3 – IP_TRANSPARENT sockets for non-local bind

**Question:**  
How can we implement IP_TRANSPARENT socket option support in Rust to bind a proxy to a non-local IP address (e.g., a defunct master server’s IP) on Linux?

- **ID:** `Q3-ip-transparent`
- **Status:** `todo`
- **Crates:** `wayback-core-net`
- **Artifacts:**
  - Utility: `crates/wayback-core-net/src/ip_transparent.rs`
- **Research objects:**
  - `socket2::Socket`
  - `libc::IP_TRANSPARENT`
  - `nix::sys::socket::setsockopt`

Implementation notes:

- Document capabilities and routing table requirements in `docs/iog-net-architecture.md`, and how this interacts with tproxy-based redirection. [file:1]

---

### Q4 – Zero-copy TUN↔UDP with io_uring

**Question:**  
What is the most efficient method for zero-copy packet forwarding between a TUN device and a userspace UDP socket using `io_uring` in Rust?

- **ID:** `Q4-iouring-forwarding`
- **Status:** `todo`
- **Crates:** `wayback-core-net`
- **Artifacts:**
  - Experimental module: `crates/wayback-core-net/src/iouring_bridge.rs`
- **Research objects:**
  - `tokio-uring`
  - `io_uring` crate
  - raw syscalls using `libc`

Implementation notes:

- Treat this as an opt-in Linux-only fast path, with feature gating in `Cargo.toml`. [file:1]

---

### Q5–Q20

Repeat the same pattern for Questions 5–20:

- Create per-question subsections with:
  - `ID` (e.g., `Q5-virtual-net-adapter`, `Q7-ebpf-xdp`, `Q11-winsock-hooking`).
  - `Status`.
  - `Crates` (often `wayback-core-net`, `wayback-proxy`, `wayback-scripting`).
  - `Artifacts` (specific files to be created).
  - `Research objects` (the crates/protocols you already listed).
  - Short implementation notes tying back to the architectural roles described in `this-is-an-excellent-and-ambit-…md`. [file:1]

---

## Section 2 – DNS Redirection & Protocol Poisoning (Q21–Q35)

Goal: selectively override DNS for legacy game domains while preserving modern HTTPS and DNSSEC behavior.

For each question (21–35), follow the same template.

### Q21 – Selective DNS stub resolver

**Question:**  
How can we implement a selective, asynchronous DNS stub resolver in Rust using `hickory-resolver` that intercepts only specific domain suffixes (e.g., `*.gamespy.com`) and forwards all other queries to the system’s default resolver?

- **ID:** `Q21-selective-dns`
- **Status:** `prototyping`
- **Crates:** `wayback-dns`
- **Artifacts:**
  - `crates/wayback-dns/src/lib.rs` – `SelectiveResolver` type, already sketched. [file:1]
  - Config schema: `schemas/iog.dns-override-rule.schema.json`.
- **Research objects:**
  - `hickory-resolver::Resolver`
  - `hickory-resolver::config::ResolverConfig`

Implementation notes:

- The current `SelectiveResolver::lookup_ipv4` implementation should be extended into a full stub resolver (A, AAAA, and optional TXT). [file:1]
- Configuration for legacy domains comes from a JSON file validated against the schema, enabling AI-chat to safely edit override rules. [file:1]

…and so on through ANAME handling, DNSSEC, DoH fallback, LD_PRELOAD-based fallbacks, and nftables/TPROXY integration, each linked to `wayback-dns` and `wayback-core-net` with explicit file targets. [file:1]

---

## Section 3 – Protocol Archaeology & Legacy Matchmaking (Q36–Q55)

Tie questions into `iog-protocol-model`, `wayback-proxy`, and per-protocol crates or modules (e.g., `wayback-master-gs`). [file:1]

Example:

### Q36 – GameSpy v3 heartbeat and list state machine

- **ID:** `Q36-gamespy-state-machine`
- **Status:** `designing`
- **Crates:** `iog-protocol-model`, `wayback-proxy`, `wayback-master-gs`
- **Artifacts:**
  - `crates/iog-protocol-model/src/gamespy.rs` – typed packet structs, JSON schemas. [file:1]
  - `crates/wayback-master-gs/src/lib.rs` – heartbeat/list handlers.
  - Replay tests under `crates/wayback-master-gs/tests/pcap_replay.rs`.
- **Research objects:** GameSpy docs, `nom`, `HashMap`.

Implementation notes:

- Align structs with the contract-first pattern already used for other IoG models, so Lua/WASM handlers get a consistent `PacketEnvelope` plus protocol-specific decoded fields. [file:1]

Continue through Q37–Q55 with similar definitions, incorporating fuzzing plans (cargo-fuzz), `.pcap` replay harnesses, Steamworks and WOL protocol support, and WASM plugin loading via `wayback-scripting`. [file:1]

---

## Section 4 – Decentralized Orchestration & P2P Discovery (Q56–Q75)

Anchor these questions directly to `wayback-dht` and governance docs. [file:1]

Example:

### Q56 – Kademlia DHT for game servers

- **ID:** `Q56-kademlia-game-providers`
- **Status:** `prototyping`
- **Crates:** `wayback-dht`
- **Artifacts:**
  - `crates/wayback-dht/src/lib.rs` – `IogNode`, `GameKey` types, already sketched. [file:1]
  - `docs/iog-governance.md` section `Discovery and provider records`.
- **Research objects:**
  - `libp2p::kad`
  - `libp2p::swarm::Swarm`
  - `Multiaddr`

Implementation notes:

- Treat game servers as providers for `GameKey` hashes; use XOR distance as described in the architecture doc. [file:1]

Similarly, document PoG, CRDT-based lists, gossipsub scoring, autonat, persistent routing tables, and rendezvous protocols with file targets (`wayback-dht`, `iog-governance.md`). [file:1]

---

## Section 5 – NAT Traversal & WebRTC Bridging (Q76–Q90)

Attach these to `wayback-webrtc-bridge` and the WASM/browser lobby crates. [file:1]

Example:

### Q76 – Integrate str0m into async runtime

- **ID:** `Q76-str0m-integration`
- **Status:** `designing`
- **Crates:** `wayback-webrtc-bridge`
- **Artifacts:**
  - `crates/wayback-webrtc-bridge/src/lib.rs` – thin wrapper over `str0m` that exposes UDP-like channels to `wayback-core-net`. [file:1]
  - `docs/iog-net-architecture.md` section `Connectivity Bridge Layer – WebRTC`. [file:1]

Keep mapping through STUN/TURN, signaling via `warp`/WebSockets, LAN-over-WebRTC, ICE tuning, mesh topologies, and fallback over WebSocket TCP relays. [file:1]

---

## Section 6 – Rollback Netcode & Deterministic Lockstep (Q91–Q100)

Tie these to a dedicated `wayback-netcode` crate and the example game slice (Pong/RA2-style RTS). [file:1]

Example:

### Q91 – Integrate ggrs into Rust engine

- **ID:** `Q91-ggrs-integration`
- **Status:** `designing`
- **Crates:** `wayback-netcode`, `examples/wayback-pong`
- **Artifacts:**
  - `crates/wayback-netcode/src/ggrs_adapter.rs`
  - `examples/wayback-pong/src/main.rs` – simple vertical slice with `ggrs::Session`. [file:1]
- **Research objects:**
  - `ggrs`
  - `ggrs::Session`
  - `ggrs::PlayerHandle`

Implementation notes:

- Mirror the rollback design described in your architecture doc: deterministic simulation, savestates via `rkyv` or `bincode`, and a WebRTC-backed transport where applicable. [file:1]

Continue through deterministic input representation, tick scheduling, floating-point determinism, spectator mode, and a unified `NetcodeBackend` abstraction. [file:1]

---

## 2. Workspace hook so the questions are “live”

To keep these 100 questions hooked into your existing flow, extend the IoG workspace manifest so this index is visible to tools and CI. [file:1]

**Filename:**  
`iog/Cargo.toml`  

Add a `package.metadata` section:

```toml
[workspace]
members = [
  "crates/wayback-core-net",
  "crates/wayback-dns",
  "crates/wayback-dht",
  "crates/wayback-webrtc-bridge",
  "crates/wayback-scripting",
  "crates/wayback-proxy",
  "crates/iog-protocol-model",
]

[workspace.metadata.iog.research]
index = "docs/iog-100-research-questions.md"
sections = [
  "Core Architecture & Interception",
  "DNS Redirection & Protocol Poisoning",
  "Protocol Archaeology & Matchmaking",
  "Decentralized Orchestration & P2P Discovery",
  "NAT Traversal & WebRTC Bridging",
  "Rollback Netcode & Deterministic Lockstep",
]
```

This lets future CLIs (and AI-chat tooling) locate and introspect the research questions as first-class data, similar to how you already treat schemas and knowledge-graph nodes. [file:1]

---

## 3. Next objectives and improvement suggestions

1. Promote key questions (your suggested slice: 1, 21, 36, 56, 76, 91) into explicit issues that reference the IDs defined above and link directly to crate files; this keeps implementation and research tightly coupled. [file:1]  
2. Add a tiny `iog-tools` CLI (e.g., `crates/iog-tools/src/bin/research-lint.rs`) that checks `docs/iog-100-research-questions.md` for missing `Artifacts` files and emits a machine-readable report so CI can enforce that high-priority questions always have concrete code or specs attached. [file:1]  
3. Extend your knowledge graph (`knowledgegraph.systems.json` / `sonia-featurelayout`) with a `FeatureEntry` for “Internet of Games – Wayback Layer” that links these questions, crates, and schemas, so AI-chat can jump from a question ID like `Q36-gamespy-state-machine` directly into the right Rust modules and docs. [file:1]
