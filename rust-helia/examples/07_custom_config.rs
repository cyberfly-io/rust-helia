//! Custom configuration example
//!
//! This example demonstrates:
//! - Configuring custom storage paths
//! - Setting up logging
//! - Custom libp2p configuration
//! - Datastore and blockstore configuration

use rust_helia::create_helia;
use helia_utils::{
    HeliaConfig, BlockstoreConfig, DatastoreConfig, LoggerConfig,
    create_swarm_with_keypair
};
use helia_interface::Helia;
use libp2p::identity::Keypair;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Configuration Example ===\n");

    // Get temp directory for this example
    let temp_dir = std::env::temp_dir().join("helia-custom-config");
    std::fs::create_dir_all(&temp_dir)?;

    // 1. Configure custom storage paths
    println!("1. Configuring custom storage paths...");
    let blockstore_path = temp_dir.join("blocks");
    let datastore_path = temp_dir.join("data");
    
    println!("   Blockstore: {}", blockstore_path.display());
    println!("   Datastore: {}\n", datastore_path.display());

    // 2. Configure blockstore
    println!("2. Configuring blockstore...");
    let blockstore_config = BlockstoreConfig {
        path: Some(blockstore_path),
        ..Default::default()
    };
    println!("   ✓ Blockstore configured\n");

    // 3. Configure datastore
    println!("3. Configuring datastore...");
    let datastore_config = DatastoreConfig {
        path: Some(datastore_path),
        ..Default::default()
    };
    println!("   ✓ Datastore configured\n");

    // 4. Configure logging
    println!("4. Configuring logging...");
    let logger_config = LoggerConfig {
        level: Level::INFO,
        include_timestamps: true,
    };
    println!("   ✓ Logger configured (level: {:?})\n", logger_config.level);

    // 5. Create custom libp2p keypair
    println!("5. Creating custom libp2p identity...");
    let keypair = Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();
    println!("   ✓ Generated peer ID: {}\n", peer_id);

    // 6. Create custom swarm
    println!("6. Creating custom libp2p swarm...");
    let swarm = create_swarm_with_keypair(keypair).await?;
    println!("   ✓ Swarm created\n");

    // 7. Build complete configuration
    println!("7. Building Helia configuration...");
    let config = HeliaConfig {
        blockstore: blockstore_config,
        datastore: datastore_config,
        logger: logger_config,
        libp2p: Some(Arc::new(Mutex::new(swarm))),
        dns: None, // Use default DNS resolver
        metrics: None, // No metrics for this example
    };
    println!("   ✓ Configuration complete\n");

    // 8. Create Helia node with custom configuration
    println!("8. Creating Helia node with custom configuration...");
    let helia = create_helia(Some(config)).await?;
    println!("   ✓ Node created\n");

    // 9. Start the node
    println!("9. Starting node...");
    helia.start().await?;
    println!("   ✓ Node started\n");

    // 10. Use the node
    println!("10. Testing node functionality...");
    use bytes::Bytes;
    
    let test_data = Bytes::from("Testing custom configuration!");
    println!("    Creating test data: {} bytes", test_data.len());
    
    // Note: In a real application, you would use UnixFS or another codec
    // to properly store and retrieve data with automatic CID generation
    println!("    ✓ Node is ready for use\n");

    // 11. Stop the node
    println!("11. Stopping node...");
    helia.stop().await?;
    println!("    ✓ Node stopped\n");

    println!("Example completed successfully!");
    println!("\nCustom storage location: {}", temp_dir.display());
    
    Ok(())
}
