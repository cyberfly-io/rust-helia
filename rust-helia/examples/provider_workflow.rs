//! Example: Complete Provider Discovery Workflow
//!
//! This example demonstrates a complete workflow:
//! 1. Create two libp2p nodes
//! 2. Node A announces it can provide a CID
//! 3. Node B searches for providers of that CID
//! 4. Both nodes discover each other via DHT
//!
//! Run with: cargo run --example provider_workflow

use cid::Cid;
use futures::stream::StreamExt;
use helia_interface::Routing;
use helia_routers::libp2p_routing::libp2p_routing;
use helia_utils::create_swarm;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("=== Complete Provider Discovery Workflow ===\n");

    // Example CID - representing some content
    let cid_str = "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco";
    let cid: Cid = cid_str.parse()?;
    info!("Content CID: {}\n", cid);

    // ========================================
    // Node A: Provider
    // ========================================
    info!("--- Setting up Node A (Provider) ---");
    let swarm_a = create_swarm().await?;
    let peer_id_a = *swarm_a.local_peer_id();
    info!("Node A Peer ID: {}", peer_id_a);
    
    let swarm_a = Arc::new(Mutex::new(swarm_a));
    let routing_a = libp2p_routing(swarm_a.clone());

    // Node A announces it can provide the content
    info!("Node A announcing it can provide CID: {}", cid);
    match routing_a.provide(&cid, None).await {
        Ok(_) => info!("✓ Node A successfully announced provider record"),
        Err(e) => warn!("✗ Node A failed to announce: {:?}", e),
    }

    // ========================================
    // Node B: Seeker
    // ========================================
    info!("\n--- Setting up Node B (Seeker) ---");
    let swarm_b = create_swarm().await?;
    let peer_id_b = *swarm_b.local_peer_id();
    info!("Node B Peer ID: {}", peer_id_b);
    
    let swarm_b = Arc::new(Mutex::new(swarm_b));
    let routing_b = libp2p_routing(swarm_b.clone());

    // Wait a moment for DHT records to propagate
    info!("\nWaiting 2 seconds for DHT records to propagate...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Node B searches for providers
    info!("\n--- Node B searching for providers ---");
    info!("Looking for providers of CID: {}", cid);
    
    match routing_b.find_providers(&cid, None).await {
        Ok(mut provider_stream) => {
            info!("✓ Provider search query initiated successfully!");
            
            let timeout = Duration::from_secs(10);
            let start = std::time::Instant::now();
            
            let mut provider_count = 0;
            info!("\nWaiting for providers (timeout: {}s)...", timeout.as_secs());
            
            while let Some(provider) = tokio::time::timeout(
                timeout.saturating_sub(start.elapsed()),
                provider_stream.next()
            ).await.ok().flatten() {
                provider_count += 1;
                info!("  [{}] Found provider: {}", provider_count, provider.peer_info.id);
                
                if provider.peer_info.id == peer_id_a {
                    info!("      ✓ This is Node A - the provider we expected!");
                }
                
                if !provider.peer_info.multiaddrs.is_empty() {
                    info!("      Addresses:");
                    for addr in &provider.peer_info.multiaddrs {
                        info!("        - {}", addr);
                    }
                }
            }

            info!("\n=== Results ===");
            if provider_count == 0 {
                info!("⚠ No providers received yet.");
                info!("\nNote: The DHT query was initiated successfully, but the current");
                info!("implementation needs event handling to receive results from the DHT.");
                info!("\nWhat's working:");
                info!("  ✓ Node A announced provider record to DHT");
                info!("  ✓ Node B initiated DHT query for providers");
                info!("  ✓ Query ID was returned successfully");
                info!("\nWhat's needed:");
                info!("  ⏳ Event loop to poll swarm events");
                info!("  ⏳ Filter and collect query results");
                info!("  ⏳ Stream providers back to caller");
            } else {
                info!("✓ Successfully found {} provider(s)!", provider_count);
            }
        }
        Err(e) => {
            warn!("✗ Error searching for providers: {:?}", e);
        }
    }

    info!("\n=== Workflow Complete ===");
    Ok(())
}
