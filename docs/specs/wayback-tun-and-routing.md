# Wayback TUN Core and Routing Pipeline

This document describes how the `wayback-core-net` TUN echo prototype evolves into a game-aware Internet-of-Games router. It focuses on concrete files, crates, and integration points with the Nintendoor64 toolchain and GAMEMODE.ai’s knowledge graph and session profiles.

## Scope and Objectives

The Wayback TUN layer is responsible for:

- Creating and managing a cross-platform virtual network adapter (TUN-style).
- Capturing and forwarding packets for legacy games without modifying their binaries where possible.
- Providing a trait-driven dispatch surface that can route packets into per-game protocol handlers written in Rust, Lua, or WASM.
- Enforcing safety and determinism constraints via schemas, knowledge graph metadata, and session invariants.

This spec covers the first three stages:

1. TUN echo prototype (`wayback-core-net` + `wayback-core-net-echo`).
2. Trait-driven packet dispatcher and routing core.
3. Hooks for DNS override, NAT, and Nintendoor64 knowledge graph / session profile integration.

## Current State

### TUN Echo Prototype

Implemented files:

- `crates/wayback-core-net/src/lib.rs`  
  - Exposes `TunConfig`, `TunDevice` (platform-specific), and `TunEchoProxy`.
  - Provides `start_default_tun_echo()` and `run_tun_echo_cli()` helpers.

- `crates/wayback-core-net/src/tun_unix.rs`  
  - Implements `TunDevice` on Unix-like systems using `/dev/net/tun` and `ioctl(TUNSETIFF)`.
  - Applies a basic IPv4 address via `ip addr add` and brings the interface up.

- `crates/wayback-core-net/src/tun_windows.rs`  
  - Placeholder stub; returns `Unsupported` so the crate compiles on Windows.

- `crates/wayback-core-net-echo/src/main.rs`  
  - Small Tokio binary that calls `run_tun_echo_cli()` for an easy echo test.

At this stage, the TUN handler simply reads raw packets from the virtual interface and writes them back. This establishes:

- A working async IO boundary.
- A clear place to insert protocol-aware routing.
- A minimal operational test that can be validated with `ip addr` and `ping`.

## Next Stage: Trait-Driven Packet Dispatcher

The next evolution is to replace the hard-coded echo loop with a packet dispatcher that can call into per-game handlers. The primary changes live in:

- `crates/wayback-core-net/src/lib.rs`
- `crates/wayback-core-net/src/dispatcher.rs` (new file)
- `crates/wayback-core-net/src/packet.rs` (new file)

### Packet Model

Create a minimal packet abstraction that keeps copies small but provides enough structure for dispatch:

- `crates/wayback-core-net/src/packet.rs`

```rust
// pseudo-interface sketch (not yet implemented)

pub struct RawPacket {
    pub data: Vec<u8>,
    pub len: usize,
}

pub enum LayerHint {
    Unknown,
    IPv4,
    IPv6,
    // future: IPX, ARP, etc.
}

pub struct ParsedPacket {
    pub raw: RawPacket,
    pub layer: LayerHint,
    // future: parsed headers, 5-tuple, etc.
}

impl ParsedPacket {
    pub fn from_raw(buf: &[u8]) -> Self { /* ... */ }
}
```

This struct stays intentionally lightweight; deeper parsing can live in protocol-specific crates.

### Dispatch Traits

Define a trait that all packet handlers implement:

- `crates/wayback-core-net/src/dispatcher.rs`

```rust
pub trait PacketHandler: Send + Sync {
    /// Handle an incoming packet.
    ///
    /// The handler can:
    /// - Mutate the packet.
    /// - Decide to drop it by returning `None`.
    /// - Return one or more packets to be written back to TUN or forwarded.
    fn handle_packet(&self, pkt: ParsedPacket) -> Vec<ParsedPacket>;
}
```

Then:

- `crates/wayback-core-net/src/lib.rs` adds a `TunRouter` type that:
  - Owns a `TunDevice`.
  - Holds an ordered list of `Arc<dyn PacketHandler>`.
  - Replaces the echo loop with a loop that:
    - Reads bytes into `RawPacket`.
    - Wraps them as `ParsedPacket`.
    - Passes them through the handlers in sequence.
    - Writes any resulting packets back to TUN (or to an external socket in later passes).

### Example Handlers

Initial “handlers” can be trivial and primarily diagnostic:

- `NullHandler`: drops everything.
- `EchoHandler`: behaves like the current echo (identity).
- `LogHandler`: logs basic metadata (length, first bytes) to tracing.

These can live in:

- `crates/wayback-core-net/src/handlers/echo.rs`
- `crates/wayback-core-net/src/handlers/log.rs`

This keeps the main router decoupled from handler implementations and matches the long-term need to plug in game-specific modules.

## DNS Override and NAT Hooks

While the first passes focus on the L2/L3 TUN path, the IoG router will also need:

- DNS override for legacy master-server domains.
- NAT traversal helpers for older games that predate UPnP/STUN.
- Optional IPX emulation for LAN-only titles.

These should be modeled as separate modules and crates:

- `crates/wayback-dns/src/lib.rs`
  - hickory-based resolver with override table.
  - JSON-configurable mapping (e.g., `westwood-online.net -> community node`).
  - CLI: `wayback-dnsd` for standalone DNS, and library integration for embedded mode.

- `crates/wayback-nat/src/lib.rs`
  - Abstractions for NAT probing, port mapping, and WebRTC/UDP bridging.
  - Later integration with `str0m` or similar crates.

- `crates/wayback-ipx/src/lib.rs`
  - Optional IPX/SPX emulation for classic LAN stacks.

The TUN router does not embed these directly but exposes hooks:

- `TunRouterConfig` with:
  - DNS override enable/disable and config path.
  - NAT helper enable/disable.
  - IPX emulation enable/disable.

## Integration With Nintendoor64 Knowledge Graph

Nintendoor64 already uses:

- `knowledgegraph/systems*.json` for SystemNode entries.
- `schemas/*.schema.json` for JSON contracts.
- `sonia-featurelayout` for human/AI-friendly feature navigation.

The IoG networking layer should register itself into the same system:

### SystemNode Entries

Add entries like:

- `systems.iog.net.core`
  - Crate: `crates/wayback-core-net/src/lib.rs`
  - Role: TUN setup and packet routing core.
  - Tags: `IoG`, `Networking`, `TUN`, `Rust`, `Deterministic`.

- `systems.iog.net.echo`
  - Crate: `crates/wayback-core-net-echo/src/main.rs`
  - Role: Test utility for TUN echo.

- `systems.iog.dns.override`
  - Crate: `crates/wayback-dns/src/lib.rs`
  - Role: DNS override for legacy master servers.

These entries let AI-Chat and tools jump directly from conceptual questions (“IoG TUN core”) to the Rust modules that implement them.

### Feature Layout

Extend `knowledgegraph/features.sonia.json` (or Nintendoor64’s equivalent) with a feature entry:

- `iog.wayback.networking.core`
  - Title: Internet-of-Games TUN and Routing Core.
  - Description: TUN-based virtual link, packet dispatcher, and hooks for legacy game routing.
  - Systems: `systems.iog.net.core`, `systems.iog.net.echo`.
  - Schemas: placeholder for future `TunRouterConfig` JSON Schema.

This makes the IoG stack discoverable from the same navigation surface as Starzip, Sonia, and Nintendoor64 ROM tools.

## Session Profiles and Invariants

To keep the IoG routing layer safe and predictable, define session invariants that apply when working on or running these systems:

- “No raw packet injection outside the TUN router.”
- “All routing rules must be defined via JSON/Lua/WASM configs validated against schemas.”
- “Experimental handlers must be tagged and disabled by default in production sessions.”

Implementation steps:

1. Extend `SessionProfile` schema to include a `iog_invariants` or generic `networking_invariants` section that lists rules specific to IoG.
2. Add a `gamemodeai-session` subcommand to query and update IoG-related invariants.
3. Wire CI so that any changes to IoG handler crates or configs must pass:
   - Rust tests and lints.
   - JSON Schema validation for routing configs.
   - Optional property-based tests for address and interval safety.

This follows the same contract-first pattern already in use for Nintendoor64’s ROM layout and patch pipelines.

## Roadmap Summary

Short-term, concrete tasks:

1. Keep `wayback-core-net-echo` compiling and usable as a TUN smoke test.
2. Introduce `packet.rs`, `dispatcher.rs`, and `handlers/*` to replace the echo loop with a trait-driven router.
3. Sketch JSON and Rust types for `TunRouterConfig`, with schema generation wired into CI.
4. Register IoG networking SystemNodes and a feature entry in the knowledge graph and Sonia feature layout.
5. Add initial IoG-specific invariants to session profile schemas.

Medium-term tasks:

- Implement DNS override and NAT helpers as separate crates.
- Introduce per-game routing modules that can be selected via config and knowledge graph metadata.
- Add Lua and WASM bindings so the community can extend packet handling without changing core Rust crates.

Once these pieces are in place, the TUN echo prototype will have evolved into the first version of the Internet-of-Games router, integrated with Nintendoor64’s schema stack, knowledge graph, and AI-driven workflows.
