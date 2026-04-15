# IoG Crate Research (wayback-* Integration Guide)

This document summarizes the current state and recommended usage patterns for key crates used by the Internet of Games (IoG) stack under `iog/`. It focuses on how each crate should be wired into the `wayback-*` crates and what constraints to keep in mind when designing APIs and plugin systems.

---

## tun (TUN/TAP devices)

**Role in IoG**

- Primary building block for transparent interception in `wayback-core-net`.
- Used to create cross-platform virtual network interfaces that capture legacy game traffic at L3 (TUN) or L2 (TAP). [file:1][web:12]

**Key points**

- Use `tun` (not `tun2`): `tun2` functionality is merged back; treat `tun` as the canonical crate. [file:1][web:11]
- Linux: TUN/TAP via kernel drivers; best supported.
- macOS: `utun`-style TUN; TAP is less common.
- Windows: requires Wintun driver and admin privileges; no reliable “no-admin” mode. [file:1][web:14]

**IoG integration patterns**

- In `wayback-core-net`:
  - Expose a `TunLink` abstraction returning a struct with:
    - `fd`/handle (for low-level integration and `AsyncFd`).
    - Async read/write interface (behind `tokio`).
    - Metadata: MTU, interface name, address. [file:1]
  - Implement an echo diagnostic binary (`wayback-core-net/src/bin/echo-tun.rs`) that sets up a TUN interface and loops packets back as a smoke test. [file:1]
- For cross-platform support:
  - Feature-gate Windows-specific code (Wintun) and document the requirement for `wintun.dll` plus admin in `docs/iog-net-architecture.md`. [file:1]
  - Expose “userspace only” mode where TUN is optional and game interception falls back to DNS/API hooking if privileges are unavailable. [file:1]

---

## hickory-resolver (DNS stub resolver)

**Role in IoG**

- Core DNS override engine in `wayback-dns`.
- Used to selectively redirect legacy domains (e.g., `*.gamespy.com`, `*.westwood.com`) to IoG relays or DHT-discovered nodes without affecting general system DNS. [file:1][web:16]

**Key points**

- Async resolver built on `tokio`, replacing old `trust-dns-resolver`. [file:1]
- Configurable `ResolverConfig` and `ResolverOpts`, plus fine-grained `NameServerConfigGroup`. [file:1]
- DNSSEC and ANAME support available when needed. [file:1]

**IoG integration patterns**

- In `wayback-dns`:
  - Implement a `SelectiveResolver` type that:
    - Accepts a list of legacy suffixes and IoG discovery endpoints.
    - On lookup:
      - If name matches a legacy suffix, resolve to a local IoG relay (e.g., `127.0.0.1` + IoG port) or a node discovered via `wayback-dht`.
      - Otherwise, forward to system resolvers using `ResolverConfig::default()` or `/etc/resolv.conf`. [file:1]
  - Provide a JSON config (validated via `schemars`) describing override rules, so AI tools can safely modify DNS behaviors without editing code. [file:1]
- For IoG-wide behavior:
  - Add DoH fallback for environments blocking UDP/53, while still applying selective overrides before sending DoH queries. [file:1]
  - Integrate with `wayback-proxy` to allow Lua/WASM plugins to see “virtual” hostnames but not modify core override rules directly. [file:1]

---

## libp2p (kad, gossipsub, relay v2, autonat, rendezvous)

**Role in IoG**

- Implements decentralized discovery and governance in `wayback-dht`.
- Kademlia: distributed game server listings.
- Gossipsub: community voting, ban lists, announcements.
- Relay v2 + DCUtR: NAT traversal for IoG nodes with limited connectivity.
- Autonat + Identify: reachability detection.
- Rendezvous: room-based peer discovery (e.g., “featured game lobby”). [file:1][web:17]

**Key points**

- Modern libp2p for Rust (0.53–0.56+) uses `tokio` and a `Swarm`-based design. [web:17]
- Behaviours combine via `#[derive(NetworkBehaviour)]`. [web:17]
- `KademliaConfig` and `PeerScoreParams` are tunable per IoG requirements. [file:1]

**IoG integration patterns**

- In `wayback-dht`:
  - Define a behaviour combining:
    - `Kademlia` for `GameKey` → `Multiaddr` provider records.
    - `Gossipsub` for votes, ban-list CRDT updates, and governance topics.
    - `Identify`, `Autonat`, and `Relay` for connection management and NAT classification. [file:1]
  - Provide a high-level IoG API:
    - `announce_server(game_id, endpoint)`.
    - `find_servers(game_id) -> Vec<ServerInfo>`.
    - `publish_vote(topic_id, payload)`.
    - `subscribe_topic(topic_id) -> Stream<Message>`. [file:1]
- For IoG protocol contracts:
  - Use `iog-protocol-model` to define JSON-serializable structs for DHT records and gossip messages; include these as schema-aware types so AI-chat and scripting can reason about them. [file:1]
- For NAT/relay:
  - Allow `wayback-webrtc-bridge` and `wayback-core-net` to query `wayback-dht` for candidate IoG relay nodes when direct connections fail. [file:1]

---

## str0m (Sans I/O WebRTC)

**Role in IoG**

- Underpins `wayback-webrtc-bridge`, providing a low-overhead WebRTC data channel implementation suitable for bridging WebRTC to raw UDP sockets. [file:1][web:18]
- Ideal for:
  - Browser↔desktop game connections.
  - Virtual LAN over the internet.
  - TURN-less peers with IoG relays as WebRTC endpoints. [file:1]

**Key points**

- Sans I/O: you own UDP sockets and event loop; `str0m` provides `Rtc`, `Input`/`Output` enums and channel events. [file:1]
- Supports ICE, DTLS, SRTP internally; no built-in TURN server. [file:1]

**IoG integration patterns**

- In `wayback-webrtc-bridge`:
  - Implement a `WebRtcUdpBridge`:
    - Expose a `bind_udp_bridge(game_id, local_udp_port)` API.
    - On browser connection (via signaling), create a `Rtc` instance and link `ChannelData` events to a `UdpSocket` connected to the legacy game’s port.
    - Inject UDP responses back into WebRTC as `Output::ChannelData`. [file:1]
  - Provide a small signaling abstraction usable by:
    - An HTTP/WebSocket server (e.g., `warp + tokio-tungstenite`).
    - A libp2p-based rendezvous channel for peer-to-peer setups. [file:1]
- For virtual LANs:
  - Allow `wayback-core-net` to plug a TUN/TAP device into `WebRtcUdpBridge` so multiple browsers share a virtual subnet where legacy games see broadcast/multicast discovery. [file:1]

---

## ggrs (rollback netcode)

**Role in IoG**

- Lives primarily in `wayback-netcode` (planned crate) and example games under `iog/examples`.
- Provides rollback netcode for proof-of-concept (e.g., Pong) and potentially for adapted legacy emulators. [file:1]

**Key points**

- Three main session types:
  - `P2PSession` for live multiplayer.
  - `SyncTestSession` for determinism testing.
  - `SpectatorSession` for watchers. [file:1]
- Transport-agnostic: expects a reliable ordered channel; can be backed by WebRTC, TCP, or custom reliability layers. [file:1]

**IoG integration patterns**

- In `wayback-netcode`:
  - Wrap `ggrs` sessions behind a `NetcodeBackend` trait that abstracts:
    - Input collection.
    - Frame advance.
    - Network send/receive hooks (pluggable transports: WebRTC, libp2p, TCP). [file:1]
  - Integrate with `wayback-webrtc-bridge` for WebRTC-based rollback (browser clients). [file:1]
- In examples:
  - Provide a `wayback-pong` vertical slice using:
    - `ggrs::P2PSession`.
    - WebRTC transport supplied by `wayback-webrtc-bridge`.
    - IoG discovery via `wayback-dht` so players can find each other. [file:1]

---

## aya (eBPF, XDP/TC hooks)

**Role in IoG**

- Optional high-performance packet interception for Linux-only nodes in `wayback-core-net`.
- Provides an alternative to TUN devices for capturing or redirecting packets at the earliest possible point in the stack. [file:1][web:20]

**Key points**

- Pure Rust eBPF framework, no libbpf dependency. [web:20]
- Supports XDP and TC hooks with examples and attach APIs. [file:1]

**IoG integration patterns**

- In `wayback-core-net`:
  - Introduce an optional feature (e.g., `ebpf-xdp`) which:
    - Builds a small eBPF program with Aya that:
      - Matches known game ports / IPs.
      - Redirects or mirrors those packets to a userspace socket used by the IoG proxy.
    - Provides a control-plane API to dynamically update match rules based on game sessions. [file:1]
  - Document limitations: Linux-only, requires root or capabilities, not suitable for all users. [file:1]

---

## wasmtime (WASM runtime for plugins)

**Role in IoG**

- Back-end for WASM plugins in `wayback-scripting`.
- Executes protocol handlers and game-specific logic safely with resource limits. [file:1]

**Key points**

- `wasmtime` core plus `wasmtime-wasi` for WASI integration. [file:1]
- Fuel and epoch-based limits for CPU control. [file:1]

**IoG integration patterns**

- In `wayback-scripting`:
  - Define a `WasmPlugin` interface:
    - Load `.wasm` modules from a plugin directory.
    - Instantiate with a restricted WASI context (no raw filesystem/network by default).
    - Export a single entry point like `handle_packet(buf_ptr, len, ctx_ptr) -> usize` where return value is a code indicating “drop/forward/modify”. [file:1]
  - Integrate fuel and epoch limits:
    - Assign a fuel budget per packet or per second.
    - Trap and unload or disable plugins that exceed limits. [file:1]
- At `wayback-proxy` level:
  - Chain plugins so Lua/WASM handlers can cooperate:
    - Example: decode GameSpy packets in WASM, but allow Lua scripts to manipulate high-level fields. [file:1]

---

## mlua (Lua embedding)

**Role in IoG**

- Scripting layer in `wayback-scripting` for rapid community contributions and simpler protocol tweaks. [file:1]

**Key points**

- Supports multiple Lua variants; sandboxing support is strongest with Luau and custom environment restrictions. [file:1]
- `UserData` is used to expose safe Rust handles into Lua. [file:1]

**IoG integration patterns**

- In `wayback-scripting`:
  - Provide a `LuaRuntime` that:
    - Registers read-only `Packet`/`Header` types as `UserData` with only necessary accessors.
    - Exposes a small host API: logging, simple key-value config, DHT lookups; no raw filesystem or arbitrary sockets. [file:1]
    - Runs scripts under strict scopes and optional `lua.sandbox(true)` for Luau builds. [file:1]
- In `wayback-proxy`:
  - Allow games to declare which Lua handlers they need via JSON configuration; tie this into schema-validated manifests so AI cannot load arbitrary scripts. [file:1]

---

## pnet (packet parsing) + nom (parsers)

**Role in IoG**

- `pnet`: low-level Ethernet/IP/UDP/TCP parsing and crafting, mostly for advanced analysis or when TUN is bypassed. [file:1]
- `nom`: higher-level protocol parsing (GameSpy, WOL, Quake, etc.). [file:1]

**Key points**

- `pnet` is synchronous and often paired with pcap or raw sockets. [file:1]
- `nom` provides zero-copy combinators ideal for binary and text protocol grammars. [file:1]

**IoG integration patterns**

- In `iog-protocol-model`:
  - Implement `nom` parsers for:
    - GameSpy `\key\value\` strings.
    - Compact server list formats.
    - WOL TLV records, Quake info strings. [file:1]
  - Expose strongly-typed Rust structs that are serialized to JSON for AI/tools. [file:1]
- In `wayback-core-net` and `wayback-proxy`:
  - Use `pnet` where raw L2/L3 parsing is necessary (e.g., IPX over Ethernet) and `nom`-based decoders to turn payloads into typed messages for scripting and AI. [file:1]

---

## detour-rs / retour (function hooking)

**Role in IoG**

- Optional Windows-only fallback path in `wayback-core-net` or a dedicated `wayback-hook-win` crate for intercepting Winsock API calls (e.g., `sendto`, `recvfrom`, `gethostbyname`) when TUN/DNS interception are insufficient. [file:1]

**Key points**

- The maintained crate is `retour` (`GenericDetour` API). [file:1]
- High risk with anti-cheat systems; must be opt-in and clearly marked experimental. [file:1]

**IoG integration patterns**

- In a Windows-specific hook crate:
  - Provide a DLL that:
    - Uses `retour` to hook selected Winsock functions.
    - Forwards calls through `wayback-core-net`/`wayback-proxy` logic or rewrites addresses toward IoG relays. [file:1]
  - Do not make this a default path; keep under a separate feature and distribution channel. [file:1]

---

## smoltcp (userspace TCP/IP stack)

**Role in IoG**

- Optional component in `wayback-core-net` for advanced virtual LAN and QoS experiments.
- Useful for implementing fully userspace “virtual internet segments” for certain games. [file:1]

**Key points**

- `Interface` plus socket sets provide IP-level connectivity independent of OS stack. [file:1]
- Tun/Tap integration is strongest on Linux. [file:1]

**IoG integration patterns**

- In `wayback-core-net`:
  - Wrap `smoltcp` behind a `VirtualNet` abstraction:
    - Accepts a TUN/TAP device and runs a userspace IP stack on top.
    - Allows traffic shaping, per-game routing, and virtual subnets without touching host routing tables. [file:1]
  - Use selectively for “LAN over internet” mode or for games that dislike OS TCP/IP behavior. [file:1]

---

## Next suggested steps

1. Add this file as `iog/docs/research_crates_iog.md` and link it from `iog/README.md` so contributors and AI tools can discover crate-level guidance easily. [file:1]  
2. For each crate above, create a minimal “hello world” example binary under `iog/examples/` (e.g., `examples/tun_echo`, `examples/hickory_selective_dns`, `examples/libp2p_dht`) wired into the relevant `wayback-*` crate, to serve as executable specs for the intended IoG usage. [file:1]  
3. Extend the `workspace.metadata.iog` section in `iog/Cargo.toml` to reference both `docs/research_crates_iog.md` and the 100-question research index so AI/CI tools can treat these documents as first-class design contracts. [file:1]
