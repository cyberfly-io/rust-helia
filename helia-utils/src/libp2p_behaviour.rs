//! libp2p behavior implementation for Helia

use libp2p::{
    autonat, dcutr, gossipsub, identify, kad, mdns, noise, ping, relay, swarm::NetworkBehaviour,
    tcp, yamux, StreamProtocol, Swarm, SwarmBuilder,
};
use libp2p::identity::Keypair;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// The combined libp2p behavior for Helia
#[derive(NetworkBehaviour)]
pub struct HeliaBehaviour {
    /// Ping protocol for liveness checking
    pub ping: ping::Behaviour,
    /// Identify protocol for peer identification
    pub identify: identify::Behaviour,
    /// Kademlia DHT for content and peer routing
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// Gossipsub for pubsub messaging
    pub gossipsub: gossipsub::Behaviour,
    /// mDNS for local peer discovery
    pub mdns: mdns::tokio::Behaviour,
    /// AutoNAT for NAT detection
    pub autonat: autonat::Behaviour,
    /// Relay support
    pub relay: relay::Behaviour,
    /// DCUtR (Direct Connection Upgrade through Relay)
    pub dcutr: dcutr::Behaviour,
}

/// Create a libp2p Swarm with Helia's default configuration
pub async fn create_swarm() -> Result<Swarm<HeliaBehaviour>, Box<dyn std::error::Error>> {
    // Generate a random keypair for this node
    let local_key = Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();

    // Create the behaviour
    let behaviour = create_behaviour(local_key.clone(), local_peer_id).await?;

    // Build the swarm
    let swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}

/// Create a libp2p Swarm with custom keypair
pub async fn create_swarm_with_keypair(
    keypair: Keypair,
) -> Result<Swarm<HeliaBehaviour>, Box<dyn std::error::Error>> {
    let local_peer_id = keypair.public().to_peer_id();
    
    // Create the behaviour
    let behaviour = create_behaviour(keypair.clone(), local_peer_id).await?;

    // Build the swarm
    let swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}

async fn create_behaviour(
    local_key: Keypair,
    local_peer_id: libp2p::PeerId,
) -> Result<HeliaBehaviour, Box<dyn std::error::Error>> {
    // Create ping behaviour
    let ping = ping::Behaviour::new(ping::Config::new());

    // Create identify behaviour
    let identify = identify::Behaviour::new(identify::Config::new(
        "/helia/1.0.0".to_string(),
        local_key.public(),
    ));

    // Create Kademlia behaviour
    let mut kademlia_config = kad::Config::default();
    kademlia_config.set_protocol_names(vec![StreamProtocol::new("/ipfs/kad/1.0.0")]);
    let store = kad::store::MemoryStore::new(local_peer_id);
    let kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);

    // Create Gossipsub behaviour
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Valid config");
    
    // Generate a deterministic message-id function from the peer ID
    let _message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };
    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .expect("Correct configuration");

    // Create mDNS behaviour
    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

    // Create AutoNAT behaviour
    let autonat = autonat::Behaviour::new(local_peer_id, autonat::Config::default());

    // Create Relay behaviour  
    let relay = relay::Behaviour::new(local_peer_id, relay::Config::default());

    // Create DCUtR behaviour
    let dcutr = dcutr::Behaviour::new(local_peer_id);

    Ok(HeliaBehaviour {
        ping,
        identify,
        kademlia,
        gossipsub,
        mdns,
        autonat,
        relay,
        dcutr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_swarm() {
        let swarm = create_swarm().await;
        assert!(swarm.is_ok());
    }

    #[tokio::test]
    async fn test_create_swarm_with_keypair() {
        let keypair = Keypair::generate_ed25519();
        let swarm = create_swarm_with_keypair(keypair).await;
        assert!(swarm.is_ok());
    }
}