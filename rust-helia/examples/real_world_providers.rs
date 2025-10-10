//! Example: Real-world Provider Discovery
//!
//! This example demonstrates how to find providers for real IPFS content.
//! It searches for a well-known CID (IPFS logo) that should have multiple
//! providers on the public IPFS network.
//!
//! Run with: cargo run --example real_world_providers

use cid::Cid;
use futures::stream::StreamExt;
use helia_interface::Routing;
use helia_routers::libp2p_routing::libp2p_routing;
use helia_utils::create_swarm;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Real-world Provider Discovery Example ===\n");

    // Well-known CIDs that should have providers on the IPFS network
    let test_cids = vec![
        // IPFS logo
        ("IPFS Logo", "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"),
        // IPFS documentation
        ("IPFS Docs", "QmTkzDwWqPbnAh5YiV5VwcTLnGdwSNsNTn2aDxdXBFca7D"),
    ];

    // Create swarm and routing
    println!("Initializing libp2p swarm with Kademlia DHT...");
    let swarm = create_swarm().await?;
    let peer_id = *swarm.local_peer_id();
    println!("Local Peer ID: {}\n", peer_id);
    
    let swarm = Arc::new(Mutex::new(swarm));
    let routing = libp2p_routing(swarm.clone());

    // Search for providers of each CID
    for (name, cid_str) in test_cids {
        println!("--- Searching for: {} ---", name);
        println!("CID: {}", cid_str);
        
        let cid: Cid = match cid_str.parse() {
            Ok(c) => c,
            Err(e) => {
                println!("Error parsing CID: {}\n", e);
                continue;
            }
        };

        // Start provider search
        println!("Initiating DHT query...");
        let start_time = Instant::now();
        
        match routing.find_providers(&cid, None).await {
            Ok(mut providers) => {
                println!("✓ Query initiated successfully");
                println!("Waiting for providers (30s timeout)...\n");
                
                let mut count = 0;
                let timeout = Duration::from_secs(30);
                
                while let Some(provider) = tokio::time::timeout(
                    timeout.saturating_sub(start_time.elapsed()),
                    providers.next()
                ).await.ok().flatten() {
                    count += 1;
                    println!("  Provider {}: {}", count, provider.peer_info.id);
                    
                    if !provider.peer_info.multiaddrs.is_empty() {
                        println!("    Addresses: {} endpoint(s)", provider.peer_info.multiaddrs.len());
                        // Show first 3 addresses
                        for (i, addr) in provider.peer_info.multiaddrs.iter().take(3).enumerate() {
                            println!("      {}. {}", i + 1, addr);
                        }
                        if provider.peer_info.multiaddrs.len() > 3 {
                            println!("      ... and {} more", provider.peer_info.multiaddrs.len() - 3);
                        }
                    }
                    println!();
                }

                let elapsed = start_time.elapsed();
                println!("Search completed in {:.2}s", elapsed.as_secs_f64());
                
                if count == 0 {
                    println!("⚠ No providers received.");
                    println!("\nNote: The DHT query was initiated successfully.");
                    println!("Current implementation initiates queries but needs event");
                    println!("handling to receive and stream back results.\n");
                    println!("What happened:");
                    println!("  1. ✓ Kademlia DHT query was sent to the network");
                    println!("  2. ✓ Query ID was returned for tracking");
                    println!("  3. ⏳ Results will arrive via swarm events (not yet implemented)");
                    println!("  4. ⏳ Event loop needs to collect and stream results back\n");
                } else {
                    println!("✓ Found {} provider(s)!\n", count);
                }
            }
            Err(e) => {
                println!("✗ Error: {:?}\n", e);
            }
        }
        
        println!("─────────────────────────────────────────────\n");
    }

    println!("=== Summary ===");
    println!("This example demonstrated:");
    println!("  • Creating a libp2p swarm with Kademlia DHT");
    println!("  • Parsing CIDs from strings");
    println!("  • Initiating provider discovery queries");
    println!("  • Handling results with timeouts");
    println!("\nNext steps:");
    println!("  • Implement event handling for query results");
    println!("  • Connect to bootstrap nodes for better DHT connectivity");
    println!("  • Add connection management for discovered peers");

    Ok(())
}
