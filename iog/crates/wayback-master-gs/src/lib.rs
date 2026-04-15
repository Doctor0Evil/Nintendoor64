//! wayback-master-gs
//!
//! Skeleton GameSpy v3 master server for the Internet of Games (IoG).
//!
//! Responsibilities:
//! - Listen for GameSpy QR heartbeats on UDP (e.g., 27900).
//! - Use GameSpyIdentity to know which game/region/ports are in play.
//! - Invoke a Lua advertisement script to build a structured capability payload.
//! - Publish server adverts into libp2p (Kademlia + gossipsub) for decentralized discovery.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub,
    gossipsub::IdentTopic,
    kad,
    kad::record::Key as KadKey,
    noise, swarm::NetworkBehaviour, swarm::Swarm, tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use mlua::{Lua, Value};
use tokio::{net::UdpSocket, select, sync::mpsc, task, time};

use iog_protocol_model::gamespy_identity::GameSpyIdentity;

/// Topic for broadcasting server adverts via gossipsub.
/// In a real implementation, this might be namespaced per game/region.
const GOSSIP_TOPIC: &str = "iog/gamespy/server_adverts";

/// Periodic interval for heartbeats to DHT/gossipsub.
const ADVERT_INTERVAL: Duration = Duration::from_secs(30);

/// Basic server advert type that we publish into libp2p.
/// This should be mirrored by a JSON Schema so AI/Lua can shape it safely.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerAdvert {
    pub server_id: String,
    pub game_id: String,
    pub region_code: Option<String>,
    pub address: String,
    pub port: u16,
    pub capabilities: serde_json::Value,
    pub timestamp: i64,
}

#[derive(NetworkBehaviour)]
pub struct MasterBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

pub struct MasterConfig {
    pub identity: GameSpyIdentity,
    pub listen_addr: SocketAddr,
    pub lua_script_path: String,
}

/// Main handle for the master server.
pub struct GameSpyMaster {
    swarm: Swarm<MasterBehaviour>,
    gossip_topic: IdentTopic,
    advert_tx: mpsc::Sender<ServerAdvert>,
}

impl GameSpyMaster {
    /// Construct a new libp2p swarm + behaviour for the master.
    pub fn new(config: &MasterConfig) -> Result<Self> {
        // Build libp2p swarm (tokio + TCP + Noise + Yamux).
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|key| {
                // Configure gossipsub.
                let message_id_fn = |message: &gossipsub::Message| {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };

                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn)
                    .build()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                // Configure Kademlia DHT with in-memory store.
                let local_peer_id = PeerId::from(key.public());
                let store = kad::store::MemoryStore::new(local_peer_id);
                let kademlia = kad::Behaviour::new(local_peer_id, store);

                Ok(MasterBehaviour { gossipsub, kademlia })
            })?
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on an ephemeral TCP port for libp2p.
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse::<Multiaddr>()?)?;

        let gossip_topic = IdentTopic::new(GOSSIP_TOPIC);
        swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&gossip_topic)?;

        let (advert_tx, advert_rx) = mpsc::channel::<ServerAdvert>(128);

        // Spawn a task to process swarm events and outgoing adverts.
        let mut swarm_clone = swarm.clone();
        let gossip_topic_clone = gossip_topic.clone();

        task::spawn(async move {
            Self::run_swarm_loop(&mut swarm_clone, gossip_topic_clone, advert_rx).await;
        });

        Ok(Self {
            swarm,
            gossip_topic,
            advert_tx,
        })
    }

    /// Start a UDP listener for GameSpy heartbeats and drive the master logic.
    ///
    /// For now, this is a minimal echo/parse stub that triggers Lua + DHT publish
    /// whenever we see a heartbeat packet.
    pub async fn run(self, cfg: MasterConfig) -> Result<()> {
        let socket = UdpSocket::bind(cfg.listen_addr).await?;
        tracing::info!("GameSpy master listening on {}", cfg.listen_addr);

        let lua = Arc::new(Lua::new());
        // Preload script once.
        let server_ref = load_lua_server(lua.clone(), &cfg.lua_script_path)?;

        let mut buf = [0u8; 2048];

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;
            let packet = &buf[..len];

            if is_gamespy_heartbeat(packet) {
                // In a real implementation, parse packet for server address, key fields, etc.
                let advert = build_advert_from_lua(
                    &cfg.identity,
                    lua.clone(),
                    server_ref.clone(),
                    src,
                )?;

                // Publish into libp2p (gossipsub + DHT).
                let _ = self.advert_tx.send(advert).await;
            }

            // Optionally: send a GameSpy heartbeat response here.
        }
    }

    async fn run_swarm_loop(
        swarm: &mut Swarm<MasterBehaviour>,
        gossip_topic: IdentTopic,
        mut advert_rx: mpsc::Receiver<ServerAdvert>,
    ) {
        loop {
            select! {
                Some(advert) = advert_rx.recv() => {
                    // 1) Publish via gossipsub.
                    if let Ok(payload) = serde_json::to_vec(&advert) {
                        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(gossip_topic.clone(), payload) {
                            tracing::warn!("Failed to publish gossipsub advert: {:?}", e);
                        }
                    }

                    // 2) Publish via Kademlia under game key derived from (game_id, region).
                    let key = make_kad_key(&advert);
                    let value = match serde_json::to_vec(&advert) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!("Failed to serialize advert for DHT: {:?}", e);
                            continue;
                        }
                    };
                    swarm.behaviour_mut().kademlia.put_record(
                        kad::Record::new(key, value),
                        kad::Quorum::One,
                    ).unwrap_or_else(|e| tracing::warn!("Failed to put DHT record: {:?}", e));
                }

                event = swarm.select_next_some() => {
                    match event {
                        libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("libp2p listening on {address}");
                        }
                        libp2p::swarm::SwarmEvent::Behaviour(MasterBehaviourEvent::Gossipsub(ev)) => {
                            tracing::debug!("Gossipsub event: {:?}", ev);
                        }
                        libp2p::swarm::SwarmEvent::Behaviour(MasterBehaviourEvent::Kademlia(ev)) => {
                            tracing::debug!("Kademlia event: {:?}", ev);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Very minimal placeholder: recognizes `\heartbeat\` messages.
/// Real implementation should parse QR protocol properly using `nom`.
fn is_gamespy_heartbeat(packet: &[u8]) -> bool {
    // GameSpy QR heartbeat often starts with backslash-delimited keys.
    packet.starts_with(b"\\heartbeat\\") || packet.windows(11).any(|w| w == b"\\heartbeat\\")
}

/// Load the Lua server script and return a handle to the `Server` table.
fn load_lua_server(lua: Arc<Lua>, path: &str) -> Result<mlua::Table> {
    let script = std::fs::read_to_string(path)?;
    let server: mlua::Table = lua.load(&script).eval()?;
    Ok(server)
}

/// Build a ServerAdvert by invoking `Server:on_heartbeat()` in Lua.
fn build_advert_from_lua(
    identity: &GameSpyIdentity,
    lua: Arc<Lua>,
    server: mlua::Table,
    src_addr: SocketAddr,
) -> Result<ServerAdvert> {
    // Call Lua: server:on_heartbeat()
    let on_heartbeat = server.get::<_, mlua::Function>("on_heartbeat")?;
    let res: Value = on_heartbeat.call(())?;

    // Expect the Lua function to return a table that we can serialize to JSON.
    let json_value = lua_to_json(res)?;

    let server_id = json_value
        .get("server_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let region_code = identity.region.as_ref().map(|r| r.code.clone());

    Ok(ServerAdvert {
        server_id,
        game_id: identity.game_id.clone(),
        region_code,
        address: src_addr.ip().to_string(),
        port: src_addr.port(),
        capabilities: json_value,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// Convert a Lua value into serde_json::Value.
/// For a first pass, rely on mlua's built-in serde support.
fn lua_to_json(v: Value) -> Result<serde_json::Value> {
    let json_value = match v {
        Value::Table(t) => mlua::serde::to_value(&t)?,
        _ => serde_json::Value::Null,
    };
    Ok(json_value)
}

/// Derive a Kademlia key from advert fields (game_id + region).
fn make_kad_key(advert: &ServerAdvert) -> KadKey {
    let mut key_str = advert.game_id.clone();
    if let Some(ref region) = advert.region_code {
        key_str.push_str("::");
        key_str.push_str(region);
    }
    KadKey::new(&key_str)
}
