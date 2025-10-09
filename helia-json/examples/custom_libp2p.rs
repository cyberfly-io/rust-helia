//! Retrieve JSON from a remote peer using custom libp2p configuration with PSK
//!
//! This example demonstrates:
//! - Building a custom libp2p swarm with PSK (Pre-Shared Key) support
//! - Configuring transport encryption for private networks
//! - Connecting to a specific remote peer
//! - Retrieving and decoding JSON content by CID using helia-json

use cid::Cid;
use helia_interface::Helia;
use helia_json::{Json, JsonInterface};
use helia_utils::{HeliaBehaviour, HeliaConfig, HeliaImpl};
use libp2p::multiaddr::Protocol;
use libp2p::{
    core::transport::upgrade::Version,
    identity::Keypair,
    kad, noise,
    pnet::{PnetConfig, PreSharedKey},
    swarm::dial_opts::{DialOpts, PeerCondition},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm, Transport,
};
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

// Swarm key for private network
const SWARM_KEY: &str = r#"/key/swarm/psk/1.0.0/
/base16/
8463a7707bad09f63538d273aa769cbdd732e43b07f207d88faa323566168ad3"#;

// Remote peer information
const REMOTE_PEER_ID: &str = "12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw";

// Candidate addresses exposed by the remote node (TCP, WebSockets, QUIC)
const REMOTE_PEER_ADDRS: &[&str] = &[
    "/dns4/node.cyberfly.io/tcp/31001/p2p/12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw",
    "/dns4/node.cyberfly.io/tcp/31002/ws/p2p/12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw",
    "/dns4/node.cyberfly.io/tcp/31002/wss/p2p/12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw",
    "/dns4/node.cyberfly.io/udp/31001/quic-v1/p2p/12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw",
];

// Topic used by the remote node for pubsub-based peer discovery
const PEER_DISCOVERY_TOPIC: &str = "cyberfly._peer-discovery._p2p._pubsub";

// JSON CID to retrieve
const JSON_CID: &str = "bagaaiera7ggi35jy6tuckbxctbkjuozkcxd33kvfuoc2jc4hp5sxogyez73a";

/// Create a custom libp2p swarm with PSK (Pre-Shared Key) protection
async fn create_swarm_with_psk(
    psk: PreSharedKey,
    keypair: Keypair,
) -> Result<Swarm<HeliaBehaviour>, Box<dyn std::error::Error>> {
    use helia_bitswap::BitswapBehaviour;
    use libp2p::{autonat, dcutr, gossipsub, identify, kad, mdns, ping, relay};

    let peer_id = keypair.public().to_peer_id();

    println!("   PSK fingerprint: {}", psk.fingerprint());

    // Build transport with PSK
    let base_transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true));
    let psk_for_transport = psk.clone();
    let transport = base_transport
        .and_then(move |socket, _| PnetConfig::new(psk_for_transport.clone()).handshake(socket))
        .upgrade(Version::V1Lazy)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .boxed();

    // Create behaviours
    let ping = ping::Behaviour::new(ping::Config::new());

    let identify = identify::Behaviour::new(identify::Config::new(
        "/libp2p/1.0.0".to_string(),
        keypair.public(),
    ));

    let mut kademlia_config = kad::Config::default();
    kademlia_config.set_protocol_names(vec![StreamProtocol::new("/ipfs/kad/1.0.0")]);
    let store = kad::store::MemoryStore::new(peer_id);
    let kademlia = kad::Behaviour::with_config(peer_id, store, kademlia_config);

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .allow_self_origin(true)
        .build()
        .expect("Valid config");

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
    let autonat = autonat::Behaviour::new(peer_id, autonat::Config::default());
    let relay = relay::Behaviour::new(peer_id, relay::Config::default());
    let dcutr = dcutr::Behaviour::new(peer_id);
    let bitswap = BitswapBehaviour::new();

    let behaviour = HeliaBehaviour {
        ping,
        identify,
        kademlia,
        gossipsub,
        mdns,
        autonat,
        relay,
        dcutr,
        bitswap,
    };

    let mut swarm = Swarm::new(
        transport,
        behaviour,
        peer_id,
        libp2p::swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(Duration::from_secs(60)),
    );

    let discovery_topic = gossipsub::IdentTopic::new(PEER_DISCOVERY_TOPIC);
    if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&discovery_topic) {
        println!("   ⚠️  Failed to subscribe to discovery topic: {e}");
    }

    Ok(swarm)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Helia JSON - Custom libp2p Configuration Example");
    println!("===================================================\n");

    // Step 1: Parse the swarm key
    println!("📝 Step 1: Parsing swarm key...");
    let psk = SWARM_KEY.parse::<PreSharedKey>()?;
    println!("✅ Swarm key parsed successfully");
    println!("   PSK fingerprint: {}", psk.fingerprint());

    // Step 2: Parse remote peer information
    println!("\n🌐 Step 2: Parsing remote peer addresses...");
    let remote_multiaddrs: Vec<Multiaddr> = REMOTE_PEER_ADDRS
        .iter()
        .filter_map(|addr| match addr.parse::<Multiaddr>() {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                println!("   ⚠️  Skipping invalid multiaddr '{addr}': {e}");
                None
            }
        })
        .collect();

    if remote_multiaddrs.is_empty() {
        return Err("No valid remote multiaddrs available".into());
    }

    for addr in &remote_multiaddrs {
        println!("   ✅ Parsed remote multiaddr: {}", addr);
    }

    println!("   🔍 Resolving DNS entries where needed...");
    let resolved_multiaddrs = resolve_dns_multiaddrs(&remote_multiaddrs).await?;
    if resolved_multiaddrs.is_empty() {
        return Err("No resolvable remote multiaddrs available".into());
    }

    for (original, resolved) in &resolved_multiaddrs {
        if original == resolved {
            println!("      🔁 Using literal multiaddr: {}", resolved);
        } else {
            println!("      🌐 {} → {}", original, resolved);
        }
    }

    // Extract peer ID from the first multiaddr
    let peer_id = extract_peer_id(&remote_multiaddrs[0])?;
    println!("✅ Peer ID: {}", peer_id);
    if peer_id.to_string() != REMOTE_PEER_ID {
        println!(
            "   ⚠️  Parsed peer ID differs from expected constant ({}).",
            REMOTE_PEER_ID
        );
    }

    // Step 3: Parse the target CID
    println!("\n🎯 Step 3: Parsing target CID...");
    let cid = Cid::from_str(JSON_CID)?;
    println!("✅ CID parsed successfully");
    println!("   CID: {}", cid);
    println!("   Codec: 0x{:x}", cid.codec());
    println!("   Version: {:?}", cid.version());

    // Step 4: Create custom libp2p swarm with PSK
    println!("\n⚙️  Step 4: Creating custom libp2p swarm with PSK...");
    println!("   Generating keypair...");

    let keypair = Keypair::generate_ed25519();
    let local_peer_id = keypair.public().to_peer_id();
    println!("   Local Peer ID: {}", local_peer_id);

    println!("   Building swarm with PSK-protected transport...");
    let swarm = create_swarm_with_psk(psk, keypair).await?;
    println!("✅ Custom libp2p swarm created with PSK protection");

    // Step 5: Create Helia instance with custom swarm
    println!("\n🔧 Step 5: Creating Helia instance with custom swarm...");
    let swarm_arc = Arc::new(Mutex::new(swarm));
    let config = HeliaConfig {
        libp2p: Some(swarm_arc.clone()),
        ..Default::default()
    };

    let helia = Arc::new(HeliaImpl::new(config).await?);
    println!("✅ Helia node created with custom PSK-protected swarm");

    // Ensure Helia (and the libp2p swarm) is running so that dials progress
    println!("\n▶️  Starting Helia node...");
    helia.start().await?;
    println!("✅ Helia node started");

    // Step 6: Create JSON instance
    println!("\n📦 Step 6: Creating JSON instance...");
    let json_store = Json::new(helia.clone());
    println!("✅ JSON instance ready");

    // Step 7: Add remote peer and dial
    println!("\n🔗 Step 7: Adding and dialing remote peer...");
    println!("   Target peer: {}", peer_id);

    let discovery_topic = libp2p::gossipsub::IdentTopic::new(PEER_DISCOVERY_TOPIC);

    // Get access to the swarm and dial the remote peer
    {
        let mut swarm = swarm_arc.lock().await;

        // Add peer to Kademlia routing table
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
        println!("   ✅ Added peer as explicit gossipsub peer");

        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&discovery_topic) {
            println!("   ⚠️  Failed to ensure discovery topic subscription: {e}");
        }

        for (original_addr, resolved_addr) in &resolved_multiaddrs {
            let mut addr_no_p2p = resolved_addr.clone();
            let had_peer_component = matches!(addr_no_p2p.pop(), Some(Protocol::P2p(_)));
            if !had_peer_component {
                println!("   ⚠️  Address missing /p2p component: {}", original_addr);
            }

            swarm
                .behaviour_mut()
                .kademlia
                .add_address(&peer_id, addr_no_p2p.clone());
            println!("   ✅ Peer address added to Kademlia: {}", addr_no_p2p);

            let dial_opts = DialOpts::peer_id(peer_id.clone())
                .condition(PeerCondition::Disconnected)
                .addresses(vec![addr_no_p2p.clone()])
                .build();

            println!(
                "   📞 Dialing remote peer at {} (resolved from {})...",
                addr_no_p2p, original_addr
            );
            match swarm.dial(dial_opts) {
                Ok(_) => println!("   ✅ Dial initiated successfully"),
                Err(e) => println!("   ⚠️  Dial error for {}: {}", addr_no_p2p, e),
            }
        }

        if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
            println!("   ⚠️  Kademlia bootstrap error: {:?}", e);
        }
    }

    println!("\n🧭 Step 8: Querying for additional providers (Kademlia)...");
    {
        let mut swarm = swarm_arc.lock().await;
        let provider_multihash = cid.hash().to_bytes();
        let provider_key = kad::RecordKey::new(&provider_multihash);
        let query_id = swarm
            .behaviour_mut()
            .kademlia
            .get_providers(provider_key.clone());
        println!("   🔍 Provider lookup started (query id: {:?})", query_id);

        let closest_query = swarm
            .behaviour_mut()
            .kademlia
            .get_closest_peers(peer_id.clone());
        println!(
            "   🔍 Closest peers lookup started (query id: {:?})",
            closest_query
        );
    }

    println!("\n⏳ Waiting for connection establishment and provider responses...");
    sleep(Duration::from_secs(20)).await;

    // Check connection status
    {
        let swarm = swarm_arc.lock().await;
        let connected = swarm.is_connected(&peer_id);
        if connected {
            println!("✅ Connected to remote peer!");
        } else {
            println!("⚠️  Not yet connected to remote peer");
        }
    }

    // Step 7: Attempt to retrieve JSON content
    println!("\n📄 Step 9: Retrieving JSON content...");
    println!("   CID: {}", JSON_CID);

    match json_store.get::<Value>(&cid, None).await {
        Ok(json_data) => {
            println!("\n🎉 SUCCESS! JSON content retrieved!\n");
            println!("📋 JSON Content:");
            println!("{}", serde_json::to_string_pretty(&json_data)?);

            // Show some metadata
            println!("\n📊 Metadata:");
            match &json_data {
                Value::Object(map) => {
                    println!("   Type: JSON Object");
                    println!("   Keys: {}", map.len());
                    println!("   Keys: {:?}", map.keys().collect::<Vec<_>>());
                }
                Value::Array(arr) => {
                    println!("   Type: JSON Array");
                    println!("   Length: {}", arr.len());
                }
                _ => {
                    println!("   Type: {}", type_name(&json_data));
                }
            }
        }
        Err(e) => {
            println!("\n❌ Could not retrieve JSON content");
            println!("   Error: {}\n", e);

            println!("💡 Troubleshooting Guide:");
            println!("   ┌─────────────────────────────────────────────────────");
            println!("   │ 1. Network Connection:");
            println!("   │    - Ensure the remote peer is online and reachable");
            println!("   │    - Check network connectivity to node.cyberfly.io");
            println!("   │");
            println!("   │ 2. Swarm Key Configuration:");
            println!("   │    - Verify swarm key matches the private network");
            println!("   │    - Implement PSK transport wrapper (see below)");
            println!("   │");
            println!("   │ 3. Content Availability:");
            println!("   │    - Confirm the CID exists on the remote peer");
            println!("   │    - Check if content is pinned on the remote node");
            println!("   │");
            println!("   │ 4. Libp2p Configuration:");
            println!("   │    - Ensure Bitswap protocol is enabled");
            println!("   │    - Configure proper transport with swarm key");
            println!("   │    - Add peer to DHT and address book");
            println!("   └─────────────────────────────────────────────────────");

            println!("\n🔧 Implementation needed for production:");
            println!("   1. Create custom libp2p transport with PSK");
            println!("   2. Configure swarm with private network key");
            println!("   3. Implement peer discovery and connection");
            println!("   4. Enable Bitswap for content retrieval");
        }
    }

    println!("\n🎉 Example completed!");
    println!("\n📚 Next Steps:");
    println!("   - Implement custom libp2p transport (see function below)");
    println!("   - Configure Bitswap with remote peer");
    println!("   - Add peer management and reconnection logic");
    println!("   - Implement proper error handling and retries");

    helia.stop().await.ok();

    Ok(())
}

/// Extract PeerId from a multiaddr
fn extract_peer_id(addr: &Multiaddr) -> Result<PeerId, Box<dyn std::error::Error>> {
    use libp2p::multiaddr::Protocol;

    for proto in addr.iter() {
        if let Protocol::P2p(peer_id) = proto {
            return Ok(peer_id);
        }
    }

    Err("No peer ID found in multiaddr".into())
}

async fn resolve_dns_multiaddrs(
    addrs: &[Multiaddr],
) -> Result<Vec<(Multiaddr, Multiaddr)>, Box<dyn std::error::Error>> {
    use std::collections::HashSet;
    use std::net::IpAddr;

    let mut resolved = Vec::new();
    let mut seen = HashSet::new();

    for addr in addrs {
        let addr_str = addr.to_string();

        if let Some(domain_rest) = addr_str.strip_prefix("/dns4/") {
            let (domain, rest) = match domain_rest.split_once('/') {
                Some((domain, rest)) => (domain, rest),
                None => (domain_rest, ""),
            };
            let rest_suffix = if rest.is_empty() {
                String::new()
            } else {
                format!("/{}", rest)
            };

            let mut found = false;
            for host in tokio::net::lookup_host((domain, 0)).await? {
                if let IpAddr::V4(ip) = host.ip() {
                    let candidate = format!("/ip4/{}{}", ip, rest_suffix);
                    if seen.insert(candidate.clone()) {
                        resolved.push((addr.clone(), candidate.parse()?));
                    }
                    found = true;
                }
            }

            if !found {
                println!(
                    "   ⚠️  DNS lookup returned no IPv4 addresses for {}",
                    domain
                );
                if seen.insert(addr_str.clone()) {
                    resolved.push((addr.clone(), addr.clone()));
                }
            }
        } else if let Some(domain_rest) = addr_str.strip_prefix("/dns6/") {
            let (domain, rest) = match domain_rest.split_once('/') {
                Some((domain, rest)) => (domain, rest),
                None => (domain_rest, ""),
            };
            let rest_suffix = if rest.is_empty() {
                String::new()
            } else {
                format!("/{}", rest)
            };

            let mut found = false;
            for host in tokio::net::lookup_host((domain, 0)).await? {
                if let IpAddr::V6(ip) = host.ip() {
                    let candidate = format!("/ip6/{}{}", ip, rest_suffix);
                    if seen.insert(candidate.clone()) {
                        resolved.push((addr.clone(), candidate.parse()?));
                    }
                    found = true;
                }
            }

            if !found {
                println!(
                    "   ⚠️  DNS lookup returned no IPv6 addresses for {}",
                    domain
                );
                if seen.insert(addr_str.clone()) {
                    resolved.push((addr.clone(), addr.clone()));
                }
            }
        } else {
            if seen.insert(addr_str.clone()) {
                resolved.push((addr.clone(), addr.clone()));
            }
        }
    }

    Ok(resolved)
}

/// Get a human-readable type name for JSON values
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "Null",
        Value::Bool(_) => "Boolean",
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Array(_) => "Array",
        Value::Object(_) => "Object",
    }
}
