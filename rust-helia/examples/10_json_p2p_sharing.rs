//! JSON P2P Sharing Example
//!
//! Demonstrates storing and retrieving JSON data over IPFS P2P network.
//! Similar to JS Helia: `const j = json(helia); await j.add({hello: 'world'})`
//!
//! Run in two terminals:
//! Terminal 1: cargo run --example 10_json_p2p_sharing -- store
//! Terminal 2: cargo run --example 10_json_p2p_sharing -- get <CID>

use helia_interface::Helia;
use helia_json::{json, JsonInterface};
use helia_utils::{BlockstoreConfig, HeliaConfig};
use rust_helia::create_helia;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct UserData {
    name: String,
    age: u32,
    languages: Vec<String>,
    active: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} store              - Store JSON data", args[0]);
        eprintln!(
            "  {} get <CID>          - Retrieve JSON data from network",
            args[0]
        );
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "store" => run_store().await,
        "get" => {
            if args.len() < 3 {
                eprintln!("Error: CID required for 'get' command");
                eprintln!("Usage: {} get <CID>", args[0]);
                return Ok(());
            }
            run_get(&args[2]).await
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Valid commands: store, get");
            Ok(())
        }
    }
}

/// Store JSON data and keep node running
async fn run_store() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Helia JSON P2P Sharing Example\n");
    println!("📝 Starting Store Node...\n");

    // Create Helia with custom config for store node
    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(PathBuf::from("/tmp/helia-json-store")),
        create_if_missing: true,
    };

    let helia = create_helia(Some(config)).await?;

    // Start the node (enables P2P networking)
    helia.start().await?;
    println!("✅ Helia store node started\n");

    // Create JSON instance - just like JS: const j = json(helia)
    let j = json(Arc::new(helia));

    // Create sample JSON data
    let user = UserData {
        name: "Alice".to_string(),
        age: 30,
        languages: vec![
            "Rust".to_string(),
            "JavaScript".to_string(),
            "Python".to_string(),
        ],
        active: true,
    };

    println!("📦 Storing JSON data:");
    println!("{:#?}\n", user);

    // Add JSON - just like JS: const cid = await j.add({hello: 'world'})
    let cid = j.add(&user, None).await?;

    println!("✅ JSON data stored successfully!");
    println!("🔑 CID: {}\n", cid);

    println!("📋 To retrieve from NETWORK (different node), run in another terminal:");
    println!(
        "   cargo run --example 10_json_p2p_sharing -- get {}\n",
        cid
    );

    println!("⏳ Keep this terminal running to serve data over P2P...");
    println!("   Press Ctrl+C to stop\n");

    // Keep the node running to serve blocks
    tokio::signal::ctrl_c().await?;

    println!("\n🛑 Shutting down store node...");

    Ok(())
}

/// Retrieve JSON data from network
async fn run_get(cid_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Helia JSON P2P Sharing Example\n");
    println!("📥 Starting Retrieve Node...\n");

    // Create Helia with custom config for retrieve node (different blockstore)
    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(PathBuf::from("/tmp/helia-json-retrieve")),
        create_if_missing: true,
    };

    let helia = create_helia(Some(config)).await?;

    // Start the node
    helia.start().await?;
    println!("✅ Helia retrieve node started\n");

    // Wait for peer discovery (mDNS needs time to find peers)
    println!("🔍 Waiting for peer discovery (5 seconds)...\n");
    sleep(Duration::from_secs(5)).await;

    // Parse CID
    let cid: cid::Cid = cid_str.parse()?;
    println!("🔍 Retrieving JSON data for CID: {}\n", cid);

    // Create JSON instance
    let j = json(Arc::new(helia));

    // Get JSON - just like JS: const obj = await j.get(cid)
    println!("⏳ Fetching from network (may take a few seconds for peer discovery)...\n");
    let user: UserData = j.get(&cid, None).await?;

    println!("✅ JSON data retrieved successfully!\n");
    println!("📄 Data:");
    println!("{:#?}\n", user);

    println!("🎉 P2P JSON retrieval successful!");
    println!("   Data was fetched from the network, not from local storage!\n");

    Ok(())
}
