use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// High-level direction of a packet at the proxy boundary.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PacketDirection {
    ClientToServer,
    ServerToClient,
}

/// Transport kind the proxy sees on its outer interface.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransportKind {
    Udp,
    Tcp,
    IpxOverUdp,
    WebRtcData,
}

/// Minimal envelope the core passes into Lua/WASM handlers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PacketEnvelope {
    pub conn_id: String,
    pub direction: PacketDirection,
    pub transport: TransportKind,
    /// Raw bytes as base64; plugins decide how to parse.
    pub payload_b64: String,
    /// Optional game-engine fingerprint (e.g., "gamespy", "quake3").
    pub protocol_hint: Option<String>,
}

/// Result of a handler invocation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PacketHandlerResult {
    /// If true, drop the packet entirely.
    pub drop: bool,
    /// Optional new bytes to forward instead of the original.
    pub rewritten_payload_b64: Option<String>,
}

/// Registration record for a protocol plugin.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolPluginDescriptor {
    pub id: String,
    pub display_name: String,
    /// For matching detection fingerprints (DNS names, ports, first bytes).
    pub fingerprints: Vec<ProtocolFingerprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolFingerprint {
    pub hostname_suffix: Option<String>,
    pub udp_port: Option<u16>,
    pub tcp_port: Option<u16>,
    /// Optional hex prefix for first N bytes.
    pub payload_prefix_hex: Option<String>,
}
