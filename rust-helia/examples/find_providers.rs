//! Example: Finding Providers for a CID using libp2p routing
//!
//! This example demonstrates how to:
//! 1. Create a libp2p swarm with Kademlia DHT
//! 2. Initialize the routing system
//! 3. Find providers for a specific CID
//! 4. Process the results
//!
//! Run with: cargo run --example find_providers

use cid::Cid;
use futures::stream::StreamExt;
use helia_interface::Routing;
use helia_routers::libp2p_routing::libp2p_routing;
use helia_utils::create_swarm;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting libp2p routing example - Find Providers");

    // Step 1: Create a libp2p swarm
    info!("Creating libp2p swarm with Kademlia DHT...");
    let swarm = create_swarm().await?;
    let swarm = Arc::new(Mutex::new(swarm));

    // Step 2: Create routing instance
    info!("Initializing libp2p routing...");
    let routing = libp2p_routing(swarm.clone());

    // Step 3: Parse a CID to search for
    // Example CID (IPFS logo): QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco
    let cid_str = "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco";
    let cid: Cid = cid_str.parse()?;
    
    info!("Searching for providers for CID: {}", cid);

    // Step 4: Find providers
    // Note: Current implementation initiates DHT query but doesn't yet stream results
    // This will be enhanced when event handling is implemented
    match routing.find_providers(&cid, None).await {
        Ok(mut provider_stream) => {
            info!("Provider query initiated successfully!");
            
            // Collect providers (with timeout)
            let timeout = Duration::from_secs(30);
            let start = std::time::Instant::now();
            
            let mut provider_count = 0;
            while let Some(provider) = tokio::time::timeout(
                timeout.saturating_sub(start.elapsed()),
                provider_stream.next()
            ).await.ok().flatten() {
                provider_count += 1;
                info!("Found provider #{}: {:?}", provider_count, provider.peer_info.id);
            }

            if provider_count == 0 {
                info!(
                    "No providers found yet. Note: Event handling implementation is needed to \
                    receive results from the DHT query. The query was initiated successfully."
                );
            } else {
                info!("Total providers found: {}", provider_count);
            }
        }
        Err(e) => {
            eprintln!("Error finding providers: {:?}", e);
        }
    }

    info!("Example complete!");
    Ok(())
}
