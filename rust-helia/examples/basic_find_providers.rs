//! Basic Example: Find Providers for a CID
//!
//! This example demonstrates how to use libp2p routing to search for
//! content providers in the IPFS network with bootstrap node connectivity.
//!
//! Features:
//! - Connects to bootstrap nodes for DHT participation
//! - Optionally supports private network (pnet) configuration
//! - Streams provider results as they arrive
//! - Configurable timeout
//!
//! Usage:
//!   cargo run --example basic_find_providers
//!
//! Or with a custom CID:
//!   cargo run --example basic_find_providers QmYourCIDHere
//!
//! With private network:
//!   cargo run --example basic_find_providers --features pnet

use cid::Cid;
use futures::stream::StreamExt;
use helia_routers::libp2p_routing::libp2p_routing;
use helia_utils::{create_swarm, HeliaBehaviour};
use libp2p::{
    multiaddr::Protocol,
    swarm::{dial_opts::DialOpts, Swarm},
    Multiaddr,
};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

// Bootstrap nodes - IPFS public DHT bootstrap nodes
const BOOTSTRAP_NODES: &[&str] = &[
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
];

// For private networks (optional)
// Uncomment and use with --features pnet
// const SWARM_KEY: &str = r#"/key/swarm/psk/1.0.0/
// /base16/
// 8463a7707bad09f63538d273aa769cbdd732e43b07f207d88faa323566168ad3"#;

// Example CID - a well-known IPFS welcome file
const DEFAULT_CID: &str = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

/// Connect to bootstrap nodes for DHT participation
async fn connect_to_bootstrap(swarm: &mut Swarm<HeliaBehaviour>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to bootstrap nodes...");
    
    for addr_str in BOOTSTRAP_NODES {
        let addr: Multiaddr = addr_str.parse()?;
        
        // Extract peer ID from the multiaddr
        if let Some(Protocol::P2p(peer_id)) = addr.iter().last() {
            // Remove /p2p/ component for dialing
            let dial_addr: Multiaddr = addr.iter()
                .take_while(|p| !matches!(p, Protocol::P2p(_)))
                .collect();
            
            // Add to Kademlia routing table
            swarm.behaviour_mut().kademlia.add_address(&peer_id, dial_addr.clone());
            
            // Dial the bootstrap node
            println!("  Dialing bootstrap node: {}", peer_id);
            let dial_opts = DialOpts::peer_id(peer_id)
                .addresses(vec![dial_addr])
                .build();
            
            if let Err(e) = swarm.dial(dial_opts) {
                println!("  ⚠️  Failed to dial {}: {}", peer_id, e);
            }
        }
    }
    
    // Bootstrap the Kademlia DHT
    if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
        println!("  ⚠️  Kademlia bootstrap error: {:?}", e);
    } else {
        println!("  ✅ Kademlia bootstrap initiated");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Basic Provider Discovery Example\n");
    
    // Parse CID from command line or use default
    let cid_str = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CID.to_string());

    let cid: Cid = cid_str.parse()?;
    println!("📦 Searching for providers of CID: {}", cid);
    println!("   (This is the IPFS 'Hello World' welcome file)\n");

    let cid: Cid = cid_str.parse()?;
    println!("📦 Searching for providers of CID: {}", cid);
    println!("   (This is the IPFS 'Hello World' welcome file)\n");

    // Create libp2p swarm with Kademlia DHT
    println!("🌐 Creating libp2p swarm...");
    let mut swarm = create_swarm().await?;
    let local_peer_id = *swarm.local_peer_id();
    println!("   Local Peer ID: {}\n", local_peer_id);

    // Connect to bootstrap nodes
    connect_to_bootstrap(&mut swarm).await?;
    
    // Give connections a moment to establish
    println!("\n⏳ Waiting for bootstrap connections...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    let swarm = Arc::new(Mutex::new(swarm));

    // Create routing instance
    println!("🧭 Creating routing instance...");
    let routing = libp2p_routing(swarm.clone());

    // Find providers
    println!("🔎 Initiating provider search...\n");
    let mut providers = routing.find_providers(&cid, None).await?;

    // Collect results (with 60 second timeout for network queries)
    println!("📡 Listening for providers (timeout: 60s)...\n");
    let mut count = 0;
    let start = std::time::Instant::now();
    
    while let Some(provider) = tokio::time::timeout(
        Duration::from_secs(60),
        providers.next()
    ).await.ok().flatten() {
        count += 1;
        let elapsed = start.elapsed().as_secs();
        
        println!("✅ Provider {} (found after {}s):", count, elapsed);
        println!("   Peer ID: {}", provider.peer_info.id);
        
        if !provider.peer_info.multiaddrs.is_empty() {
            println!("   Addresses:");
            for addr in &provider.peer_info.multiaddrs {
                println!("     • {}", addr);
            }
        }
        
        if !provider.transport_methods.is_empty() {
            println!("   Transport: {:?}", provider.transport_methods);
        }
        println!();
    }

    let total_time = start.elapsed().as_secs();
    
    if count == 0 {
        println!("❌ No providers found after {}s", total_time);
        println!("\n💡 This could mean:");
        println!("   • The content is not available in the public DHT");
        println!("   • Bootstrap connections haven't fully established");
        println!("   • The DHT query is still propagating");
        println!("\n   Try running again or use a different CID");
    } else {
        println!("✅ Total providers found: {} (in {}s)", count, total_time);
        println!("\n💡 These peers have the content and can serve it via Bitswap!");
    }

    Ok(())
}
