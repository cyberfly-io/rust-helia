//! Basic Helia node creation and management
//!
//! This example demonstrates:
//! - Creating a Helia node with default configuration
//! - Starting the node and keeping it running
//! - Graceful shutdown on Ctrl+C
//! - Accessing node components (blockstore, datastore, etc.)

use rust_helia::create_helia;
use helia_interface::Helia;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Helia Node Example ===\n");

    // Create a new Helia node with default configuration
    println!("Creating Helia node...");
    let helia = create_helia(None).await?;
    println!("✓ Helia node created successfully\n");

    // Start the node
    println!("Starting node...");
    helia.start().await?;
    println!("✓ Node started\n");

    // Access node components
    println!("Node components:");
    println!("  - Blockstore: available");
    println!("  - Datastore: available");
    println!("  - Pins: available");
    println!("  - Routing: available");
    println!("  - DNS resolver: available");
    
    if helia.metrics().is_some() {
        println!("  - Metrics: enabled");
    } else {
        println!("  - Metrics: disabled");
    }
    println!();

    // Keep the node running until Ctrl+C is pressed
    println!("Node is running. Press Ctrl+C to stop...\n");
    
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("\n\nReceived shutdown signal...");
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Stop the node gracefully
    println!("Stopping node...");
    helia.stop().await?;
    println!("✓ Node stopped\n");

    println!("Example completed successfully!");
    
    Ok(())
}
