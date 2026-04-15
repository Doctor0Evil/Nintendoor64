use anyhow::Result;
use libp2p::{
    gossipsub, identity,
    kad::{record::store::MemoryStore, Kademlia, KademliaConfig, QueryId},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    Multiaddr, PeerId, Transport,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GameKey {
    pub gamename: String,
}

#[derive(NetworkBehaviour)]
pub struct IogBehaviour {
    pub kad: Kademlia<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

pub struct IogNode {
    swarm: Swarm<IogBehaviour>,
    providers: HashSet<String>,
}

impl IogNode {
    pub fn new(listen: Multiaddr) -> Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        let transport = libp2p::tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(libp2p::noise::Config::new(&local_key)?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed();

        let store = MemoryStore::new(local_peer_id);
        let mut kad = Kademlia::with_config(local_peer_id, store, KademliaConfig::default());

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub::Config::default(),
        )?;

        let behaviour = IogBehaviour { kad, gossipsub };
        let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

        Swarm::listen_on(&mut swarm, listen)?;

        Ok(Self {
            swarm,
            providers: HashSet::new(),
        })
    }

    pub fn advertise_game(&mut self, key: GameKey) -> QueryId {
        let k = libp2p::kad::record::Key::new(&key.gamename);
        info!("Advertising game {}", key.gamename);
        self.swarm.behaviour_mut().kad.start_providing(k).unwrap()
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {address}");
                }
                SwarmEvent::Behaviour(ev) => {
                    info!("Behaviour event: {:?}", ev);
                }
                other => {
                    warn!("Unhandled swarm event: {:?}", other);
                }
            }
        }
    }
}
