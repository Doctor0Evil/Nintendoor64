use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GameSpyPortMap {
    /// Primary heartbeat port (usually UDP 27900, or game-relative offset).
    pub heartbeat_udp: u16,
    /// Master list TCP (often 28900).
    pub master_list_tcp: Option<u16>,
    /// Game-specific query/“status” UDP (often 6500 or offset).
    pub query_udp: Option<u16>,
    /// Connection and search manager ports (29900/29901 or offsets).
    pub conn_mgr_tcp: Option<u16>,
    pub search_mgr_tcp: Option<u16>,
    /// Optional chat/IRC port (e.g., 6667).
    pub chat_tcp: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GameSpyRegion {
    /// Region code, e.g. "US", "EU", "JP", "NTSC-U", "PAL".
    pub code: String,
    /// Optional platform disambiguator (pc, ps2, xbox, gc).
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GameSpyIdentity {
    /// Short gamename string used on the wire (QR, GP, etc.).
    pub game_id: String,
    /// Human-readable title.
    pub title: String,
    /// Optional region/sku separation.
    pub region: Option<GameSpyRegion>,
    /// Static or offset-aware port configuration.
    pub ports: GameSpyPortMap,
}
