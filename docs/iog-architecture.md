## 1. Promote this into `docs/iog-architecture.md`

Treat what you wrote as the core of a formal architecture doc for the IoG layer, and drop it into the repo as:

`docs/iog-architecture.md`

You can keep the structure you already have (network interception, DNS, GameSpy, DHT/GossipSub, NAT/WebRTC, IPX tunneling, rollback, Lua/WASM, AI, DLL hooking) and add:

- A short “Repository Mapping” section that explicitly lists which crates in your workspace implement which parts (e.g., `wayback-core-net` ↔ TUN/TAP + libpnet, `wayback-dns` ↔ hickory-dns resolver, `wayback-master-gs` ↔ GameSpy master replacement, `wayback-dht` ↔ libp2p Kademlia + gossipsub, etc.). [github](https://github.com/libpnet/libpnet)
- A “Contracts and Schemas” section that explains that every protocol or subsystem (GameSpy heartbeat, compact server list, Kademlia game provider record) has a Rust struct + JsonSchema so AI-chat and other tools interact through JSON instead of ad‑hoc configs. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

That doc becomes the top-level “design bible” AI-chat can reference when planning new crates or CLIs.

## 2. Wire the existing TUN echo crate into this architecture

You now have a `wayback-core-net` TUN echo module that already matches the “userspace layer 3 interception” section of your text: it uses a TUN device and an async loop to handle raw IP packets. [docs](https://docs.rs/pnet/latest/pnet/)

Next small steps to make it align fully with your architecture:

- Add **libpnet** as a dependency and layer a `libpnet::packet` view on top of the TUN buffer, so you can cleanly parse IPv4/TCP/UDP headers before you start redirecting them to legacy master servers. [github](https://github.com/libpnet/libpnet)
- Introduce a `WaybackRoute` trait (`fn handle_ipv4(&mut self, packet: &mut [u8]) -> Action`) that will eventually decide “send to original destination,” “rewrite to IoG master,” or “drop,” and call this trait in place of the current echo behavior. [docs](https://docs.rs/pnet/latest/pnet/)
- Plan a sibling crate `wayback-tproxy-config` that shells out to or wraps `tproxy-config` logic for IP_TRANSPARENT and routing rule setup, so the TUN/transparent proxy behavior matches what you described in the essay. [docs](https://docs.rs/pnet/latest/src/pnet/lib.rs.html)

That gets you from “echo demo” to “real transparent interceptor” with a very small surface area.

## 3. Split out dedicated crates for the major sections you described

From your text and current goals, you can carve the IoG repo (or a `wayback/` subtree inside Nintendoor64) into focused crates:

- `crates/wayback-core-net`: TUN/TAP handling, libpnet/socket2 packet IO, transparent sockets, IPX over UDP tunneling with RFC‑1234 semantics. [ferrous-systems](https://ferrous-systems.com/blog/hickory-dns-client/)
- `crates/wayback-dns`: hickory-resolver-based stub resolver with a TOML/JSON map of legacy domains to IoG nodes; this is where you implement ANAME‑style overrides and ensure “only legacy domains are poisoned.” [ferrous-systems](https://ferrous-systems.com/blog/hickory-dns-client/)
- `crates/wayback-master-gs`: GameSpy master replacement (heartbeat UDP listener, TCP server list provider, validation/seckey generator) with structs and tests derived from the GameSpy docs and NightfireSpy. [github](https://github.com/gschup/ggrs)
- `crates/wayback-dht`: libp2p Kademlia + gossipsub wrapper for “game provider” records and community voting topics (e.g., weekly featured game, map rotations). [discuss.libp2p](https://discuss.libp2p.io/t/gossipsub-with-relays/1923)
- `crates/wayback-webrtc-bridge`: str0m integration for WebRTC NAT traversal and data channels; this becomes the bridge between browser lobbies and legacy LAN/IPX traffic. [github](https://github.com/libp2p/rust-libp2p/issues/3659)
- `crates/wayback-rollback`: a small adapter crate that wires GGRS into your deterministic ECS or emulator integration for rollback support, with platform‑agnostic types for “frame state snapshot” and “input stream.” [github](https://github.com/gschup/ggrs)
- `crates/wayback-scripting`: Lua/piccolo + WASM sandbox host; each protocol handler or “game profile” lives as a plugin here, with fuel/step limits and restricted APIs. [ferrous-systems](https://ferrous-systems.com/blog/hickory-dns-client/)

Tie all of these into the knowledge graph and FeatureLayout files you already use in Nintendoor64 so AI‑Chat can navigate by concept: `systems.iog.wayback.core.net`, `systems.iog.gamespy.master`, `systems.iog.dht.discovery`, etc. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## 4. Define minimal Rust types for the key protocols

Your narrative can be turned directly into Rust structs and enums that are trivial to serialize/deserialize and reason about:

- **GameSpy heartbeat and server info**: `GameSpyHeartbeat { query_port, gamename, state_changed, ... }`, `ServerInfo { map, players, version, rules: HashMap<String,String> }` modeled after GameSpyDocs and Tiberian Technologies’ writeup. [github](https://github.com/libpnet/libpnet/blob/master/docs/using_packet.md)
- **GameSpy compact list entry**: `CompactServer { ip: Ipv4Addr, port: u16 }` plus encryption/encoding helpers keyed by “gamekey” and “enctype.” [github](https://github.com/libpnet/libpnet/blob/master/docs/using_packet.md)
- **Kademlia game provider record**: `GameProviderRecord { game_id: GameId, addr: Multiaddr, metadata: GameMetadata }` with direct mapping to libp2p’s DHT API. [discuss.libp2p](https://discuss.libp2p.io/t/gossipsub-with-relays/1923)
- **IPX‑over‑UDP tunnel message**: `IpxTunnelFrame { src: IpxAddress, dst: IpxAddress, payload: Bytes }` where the payload is a 576‑byte max datagram encapsulated in UDP as per RFC‑1234. [ferrous-systems](https://ferrous-systems.com/blog/hickory-dns-client/)
- **Rollback session config**: `RollbackSessionConfig { max_rollback_frames, input_delay, player_count }` passed into GGRS to maintain deterministic simulation properties. [github](https://github.com/gschup/ggrs)

These types, plus JsonSchema, can live in a shared `iog-protocol-model` crate so both Rust services and AI‑Chat have a single source of truth.

## 5. Plan a first vertical slice “research build”

Tie everything together in a single, well‑scoped experiment that proves the architecture end‑to‑end:

- Pick one GameSpy title with existing reverse‑engineering work, like Nightfire or an older C&C. [github](https://github.com/libpnet/libpnet/blob/master/docs/using_packet.md)
- Implement:
  - A TUN/transparent proxy using `wayback-core-net` + socket2/libpnet that forwards all traffic for `master.gamespy.com` to a local `wayback-master-gs` instance. [github](https://github.com/libpnet/libpnet)
  - A minimal `wayback-master-gs` that understands heartbeats for that single `gamename`, exposes a JSON/HTTP endpoint for server listings, and encrypts a compact list that original clients accept. [github](https://github.com/libpnet/libpnet/blob/master/docs/using_packet.md)
  - A tiny `wayback-dht` shim so your master server also announces itself into the libp2p DHT under a “game hash,” making it discoverable as you scale beyond one box. [discuss.libp2p](https://discuss.libp2p.io/t/gossipsub-with-relays/1923)
- Put all config (legacy hostname, replacement master address, GameSpy keys) into versioned JSON/TOML files with schemas, and link them in the knowledge graph as Nintendoor64 and IoG features. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

Once that vertical slice is stable, you can iterate outward: IPX tunneling for DOS/Win9x titles, str0m‑backed WebRTC for browser lobbies, then GGRS‑based rollback overlays for selected fighting/RTS games. [github](https://github.com/libp2p/rust-libp2p/issues/3659)
