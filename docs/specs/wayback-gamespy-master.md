# Wayback GameSpy Master Server Specification

This document defines the protocol model, Rust types, and crate boundaries for `crates/wayback-master-gs`, a GameSpy‑compatible master server implementation used by the Internet of Games / Wayback stack to revive legacy titles that relied on GameSpy matchmaking. [333networks](https://333networks.com/howitworks)

The goals of `wayback-master-gs` are:

- Emulate GameSpy v0/v1 master server behavior for a curated set of titles without modifying the original game binaries. [github](https://github.com/GameProgressive/GameSpyDocs)
- Provide a schema‑first, Rust‑typed model of heartbeats, challenges, validation, and server lists so AI‑Chat and tools operate via JSON and CLIs instead of ad‑hoc strings. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Integrate cleanly with the Wayback networking stack (TUN/TAP, DNS override, engine detection) and Nintendoor64’s knowledge graph and CI infrastructure. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## 1. Protocol Overview

GameSpy master communication uses a **key/value dictionary protocol** over backslash‑delimited text, primarily over UDP between game servers and master, and TCP between game clients and master. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)

General dictionary format:

- A message is a sequence of `\key\value` pairs, concatenated.
- Messages normally terminate with `\final\` as the last key, though fragments may omit it. [333networks](https://333networks.com/howitworks)
- Servers and clients agree on a small vocabulary of keys (`heartbeat`, `gamename`, `secure`, `validate`, `enctype`, `ip`, etc.). [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)

Two major flows:

1. **Server ↔ Master (UDP):**
   - Heartbeats: `\heartbeat\[query_port]\gamename\[name]\final` every 30–300 seconds. [333networks](https://333networks.com/howitworks)
   - Master replies with `\heartbeatresult\0\final` on success and may issue a `\secure\CHALL\enctype\0\final` challenge. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)
   - Server responds with `\validate\RESPONSE\final` based on gamename‑specific gamekey and challenge. [github](https://github.com/GameProgressive/GameSpyDocs)

2. **Client ↔ Master (TCP):**
   - Client opens a connection, sends a dictionary specifying `gamename`, filters, and list type (e.g., `\list\cmp\gamename\...`). [333networks](https://333networks.com/howitworks)
   - Master responds with server list data:
     - Plain format: `\ip\IP:PORT\ip\IP:PORT\...` in text. [333networks](https://333networks.com/howitworks)
     - Compact format: raw `IP`/`PORT` tuples in binary, often encrypted with gamekey/enctype. [int64](http://int64.org/docs/gamestat-protocols/gamespy2.html)

`wayback-master-gs` focuses on v0‑style keyword/value communication and per‑game pluggable challenge/validate and list encoding logic. [github](https://github.com/GameProgressive/GameSpyDocs)

## 2. Crate Boundaries and Files

`crates/wayback-master-gs` is organized as follows:

```text
crates/wayback-master-gs/
  src/
    lib.rs              # high-level API, shared types
    udp_heartbeat.rs    # UDP listener and heartbeat flow
    tcp_client.rs       # TCP list server for clients
    dict_codec.rs       # key/value backslash dictionary codec
    challenge.rs        # secure/validate challenge handling
    encrypt.rs          # server list encryption / compact encoding
    game_registry.rs    # per-game metadata (gamename, gamekey, enctype)
    filters.rs          # client filter parsing (map, version, flags)
    store.rs            # in-memory / persistent server store
  config/
    games.toml          # per-game definitions (gamename -> keys, ports)
  schemas/
    gamespy.master.schema.json  # JsonSchema for config and server records
```

This crate is **headless**: it exposes Rust APIs and a small JSON‑driven CLI for use by `wayback-core-net`, `wayback-orchestrator`, and AI‑Chat, rather than binding directly to sockets at the repo root. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

## 3. Core Data Model

### 3.1 Dictionary messages

All GameSpy messages are parsed into a normalized `GsDict` structure:

```rust
/// Backslash-delimited GameSpy dictionary message: \key\value\key\value\... \final\
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GsDict {
    pub entries: Vec<(String, String)>,
}

impl GsDict {
    /// Retrieve the first value for a key, if present.
    pub fn get(&self, key: &str) -> Option<&str> { /* ... */ }

    /// Set or overwrite a key's value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) { /* ... */ }

    /// Remove a key.
    pub fn remove(&mut self, key: &str) { /* ... */ }
}
```

Encoding and decoding:

- `dict_codec.rs` implements:
  - `fn parse_dict(raw: &[u8]) -> Result<GsDict>` – splits on `\` characters, pairing keys and values until `final`. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)
  - `fn encode_dict(dict: &GsDict) -> Vec<u8>` – serializes back to `\key\value...` and appends `\final\` if missing. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)

### 3.2 Heartbeats and server records

Heartbeats are dictionaries with at least `heartbeat` and `gamename` keys. [333networks](https://333networks.com/howitworks)

Rust type:

```rust
/// Parsed heartbeat from a game server.
#[derive(Debug, Clone)]
pub struct Heartbeat {
    pub gamename: String,
    pub query_port: u16,
    pub addr: std::net::SocketAddr,
    pub raw: GsDict,
}
```

Server records stored in the master:

```rust
use std::time::Instant;
use serde::{Serialize, Deserialize};

/// Internal representation of a registered game server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRecord {
    pub gamename: String,
    pub addr: std::net::SocketAddr,
    pub query_port: u16,
    pub last_heartbeat: Instant,
    /// Key/value settings reported by the server (map, players, version,...).
    pub info: GsDict,
    /// Whether challenge/validate succeeded for this server.
    pub validated: bool,
}
```

These structs are the basis for JSON Schemas and CLI outputs so AI‑Chat can inspect or manipulate master state without touching raw protocol wire formats. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

### 3.3 Game registry and keys

Different titles share the GameSpy protocol but use unique `gamename`, `gamekey`, and `enctype` values for validation and list encryption. [github](https://github.com/GameProgressive/GameSpyDocs)

`game_registry.rs` defines:

```rust
/// Game-specific parameters for GameSpy emulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub gamename: String,
    pub display_name: String,
    /// Secret key used in validate/seckey calculations.
    pub gamekey: String,
    /// Enctype for list encryption (0, 1, 2...).
    pub enctype: u8,
    /// Default master UDP/TCP ports for this game.
    pub master_udp_port: u16,
    pub master_tcp_port: u16,
}

#[derive(Debug, Clone)]
pub struct GameRegistry {
    by_name: std::collections::HashMap<String, GameConfig>,
}
```

Config is loaded from `config/games.toml` and validated against `gamespy.master.schema.json`. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

## 4. UDP Heartbeat Flow (`udp_heartbeat.rs`)

The UDP listener accepts packets from game servers on `master_udp_port` per game (or a multiplexing port if desired).

Core responsibilities:

1. Parse incoming datagram into `GsDict` with `parse_dict`.
2. Detect heartbeats via presence of `heartbeat` key. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)
3. Extract `gamename` and query port, construct `Heartbeat`.
4. Look up `GameConfig` for `gamename`.
5. Store/update `ServerRecord` in `store.rs`.
6. Issue challenge and heartbeat response as needed.

### 4.1 Challenge/validate

Per GameSpy v0 behavior, the master replies to a heartbeat with a `secure` challenge and later verifies a `validate` response before fully trusting the server. [333networks](https://333networks.com/howitworks)

Challenge:

- Generate random 6‑character ASCII string: `[A‑Z0‑9]` or similar.
- Store outstanding challenge keyed by `(gamename, addr)` with timestamp.
- Reply:

  ```text
  \secure\KGVCQR\enctype\0\final
  ```

Rust type:

```rust
#[derive(Debug, Clone)]
pub struct Challenge {
    pub gamename: String,
    pub addr: std::net::SocketAddr,
    pub challenge: String,
    pub issued_at: Instant,
}
```

Validation:

- When a dictionary with `validate` key is received, look up outstanding challenge.
- Use `GameConfig.gamekey` and the documented algorithm for that `gamename` to compute expected validate/seckey. [github](https://github.com/GameProgressive/GameSpyDocs)
- If match, mark `ServerRecord.validated = true`.

The exact algorithms for specific games are plugged in via per‑game helpers or plugin modules, using existing reverse‑engineering work as a reference. [github](https://github.com/startersclan/PRMasterServer)

### 4.2 Heartbeat result

On processing a heartbeat, reply:

- Success: `\heartbeatresult\0\final`
- Error (e.g., invalid `gamename`): `\heartbeatresult\1\err\unknown_gamename\final`

These responses are defined as:

```rust
pub enum HeartbeatResultCode {
    Ok,
    UnknownGame,
    BadRequest,
}

pub fn build_heartbeat_result(code: HeartbeatResultCode) -> GsDict {
    // ...
}
```

## 5. TCP Client List Flow (`tcp_client.rs`)

Game clients connect via TCP to retrieve server lists. [333networks](https://333networks.com/howitworks)

Responsibilities:

1. Accept a TCP connection.
2. Read a `GsDict` request from the client.
3. Determine gamename and list format (`list` key, e.g., `cmp` for compact).
4. Query the `ServerStore` for matching servers (optionally filtered).
5. Encode the response either as:
   - Text `\ip\IP:PORT\ip\IP:PORT\...` dictionary. [333networks](https://333networks.com/howitworks)
   - Compact/encrypted binary list as per game’s `enctype` and `gamekey`. [int64](http://int64.org/docs/gamestat-protocols/gamespy2.html)

### 5.1 Filters and queries

Filters are passed via keys like `map`, `gamever`, `location`, `password`, etc. [333networks](https://333networks.com/howitworks)

`filters.rs` defines:

```rust
/// Client query filters parsed from a GameSpy dictionary.
#[derive(Debug, Clone)]
pub struct ClientQuery {
    pub gamename: String,
    pub list_mode: ListMode,
    pub want_rules: bool,
    pub want_players: bool,
    pub want_teams: bool,
    pub filters: Vec<ServerFilter>,
}

#[derive(Debug, Clone)]
pub enum ListMode {
    PlainIp,    // \ip\IP:PORT ...
    Compact,    // IP/port tuples, maybe encrypted
}

#[derive(Debug, Clone)]
pub enum ServerFilter {
    MapEquals(String),
    VersionAtLeast(String),
    Dedicated(bool),
    Secure(bool),
    Location(i32),
    // ...
}
```

These are used to select appropriate `ServerRecord`s before encoding.

### 5.2 Server list encoding and encryption

For plain text lists:

- Emit `\ip\IP:PORT` pairs into a `GsDict`, optionally followed by per‑server keys (`map`, `numplayers`, etc.). [333networks](https://333networks.com/howitworks)

For compact lists:

- Output a binary blob of IP/port pairs in network order (4 bytes IP, 2 bytes port) for each server. [333networks](https://333networks.com/howitworks)
- If `enctype != 0`, run the game‑specific encryption algorithm, using `GameConfig.gamekey` and the requested `enctype`. [int64](http://int64.org/docs/gamestat-protocols/gamespy2.html)
- Prepend any required headers (e.g., `\\basic\\` markers) according to the specific protocol version for that title. [int64](http://int64.org/docs/gamestat-protocols/gamespy2.html)

`encrypt.rs` provides a pluggable trait:

```rust
pub trait ListEncoder {
    fn encode_plain(&self, servers: &[ServerRecord]) -> Vec<u8>;
    fn encode_compact(&self, servers: &[ServerRecord]) -> Vec<u8>;
}

pub trait ListEncryptor {
    fn encrypt(&self, gamename: &str, enctype: u8, data: &[u8]) -> Vec<u8>;
}
```

Concrete encoder/encryptor implementations can reuse algorithms from reference master servers such as RetroSpyServer, PRMasterServer, or NightfireSpy, adapted to Rust. [github](https://github.com/GameProgressive)

## 6. Storage and Pruning (`store.rs`)

The master maintains an in‑memory store of active servers:

```rust
use std::collections::HashMap;

pub struct ServerStore {
    by_game: HashMap<String, Vec<ServerRecord>>,
    // optional index by addr for quick update
}
```

Responsibilities:

- Add/update `ServerRecord` when a heartbeat or validate is received.
- Prune records that have not sent heartbeats within a timeout (e.g., 5 minutes). [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)
- Provide filterable views for `tcp_client.rs` to respond to client queries.

Persisted state (if desired) can be stored in a small local database (e.g., SQLite) but is not mandatory for the first pass.

## 7. CLI and JSON Interfaces

To make `wayback-master-gs` AI‑ and automation‑friendly, expose a JSON/CLI layer:

- `wayback-master-gs serve` – start UDP/TCP listeners based on `config/games.toml`.
- `wayback-master-gs dump --gamename <name>` – print current `ServerRecord`s as JSON.
- `wayback-master-gs simulate-heartbeat --gamename <name> --addr 1.2.3.4:27015 --port 27015` – inject a synthetic heartbeat for testing.

CLI commands accept/return JSON, with schemas:

- `schemas/gamespy.master.schema.json` – validates `GameConfig`, `ServerRecord`, and CLI request/response objects, following the same pattern as Nintendoor64 schemas. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## 8. Integration with Wayback and Nintendoor64

`wayback-master-gs` is not responsible for DNS rewriting or raw socket interception; those live in `wayback-core-net` and `wayback-dns`. Integration happens via:

- DNS override: map legacy GameSpy hostnames (e.g., `master.gamespy.com`) to the IP of the host running `wayback-master-gs`. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)
- Transparent proxy: `wayback-core-net` or `wayback-proxy` forwards UDP/TCP traffic destined for GameSpy masters to `wayback-master-gs`, preserving source addresses for correct challenge handling. [github](https://github.com/libpnet/libpnet/blob/master/examples/packetdump.rs)
- Knowledge graph: add SystemNodes like `systems.iog.gamespy.master` pointing to this crate, its schemas, and config, so AI‑Chat can discover and use it. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)

Nintendoor64–specific integration:

- For N64/PS1 games that used GameSpy, per‑game configs can live in Nintendoor64’s knowledge graph and be referenced by `wayback-master-gs` to ensure correct `gamename`, `gamekey`, and `enctype` values. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/a7d23c58-ddb5-4dd1-84ae-061d341ae629/here-are-ten-nintendoor64-ai-c-nwQDMQxMTLSK79e7E6.cew.md)
- Session profiles can include invariants like “GameSpy emulation must be enabled for game X” or “heartbeat interval must match original,” feeding into CI checks. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

## 9. First Implementation Milestones

To get from spec to working code, the first concrete tasks are:

1. Implement `dict_codec.rs` with round‑trip tests based on examples from 333Networks and Tiberian Technologies docs. [tiberiantechnologies](https://tiberiantechnologies.org/Docs/?page=GameSpy+Protocol+and+New+Broadcaster)
2. Implement `HeartBeat` parsing and basic `ServerStore`, with synthetic heartbeats accepted from a test harness.
3. Implement UDP `\heartbeat\` handling and `\heartbeatresult\0\final` replies for a single `gamename` with no challenge/validate, then add secure/validate once the gamekey algorithm is known. [github](https://github.com/GameProgressive/GameSpyDocs)
4. Implement a minimal TCP list handler returning plain `\ip\IP:PORT` lists for that same game. [333networks](https://333networks.com/howitworks)
5. Add `GameConfig` loading from `config/games.toml` and JSON Schema validation for that config. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_187fc3bd-787d-42f7-b3a0-32a177dab406/3f978e5a-687d-436c-b958-6f08f58fb097/this-research-focuses-on-creat-HjKimWiCSuykHqnf0GB8AQ.md)

Once those are stable, you can layer in:

- Per‑game validate/seckey logic, borrowing from reference implementations. [github](https://github.com/GameProgressive)
- Compact/encrypted list formats for bandwidth parity and binary compatibility with original clients. [int64](http://int64.org/docs/gamestat-protocols/gamespy2.html)
- More advanced filters and anti‑abuse/ratelimiting as needed.

This spec now gives `crates/wayback-master-gs` a clear, file‑level roadmap that matches the historical GameSpy protocol while fitting neatly into the broader Internet‑of‑Games and Nintendoor64 tooling ecosystem.
