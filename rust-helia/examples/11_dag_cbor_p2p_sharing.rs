//! DAG-CBOR P2P Sharing Example
//!
//! Demonstrates storing and retrieving DAG-CBOR data over IPFS P2P network.
//! DAG-CBOR is a more compact binary format compared to JSON.
//! Similar to JS Helia: `const cbor = dagCbor(helia); await cbor.add(data)`
//!
//! Run in two terminals:
//! Terminal 1: cargo run --example 11_dag_cbor_p2p_sharing -- store
//! Terminal 2: cargo run --example 11_dag_cbor_p2p_sharing -- get <CID>

use helia_dag_cbor::{dag_cbor, DagCborInterface};
use helia_interface::Helia;
use helia_utils::{BlockstoreConfig, HeliaConfig};
use rust_helia::create_helia;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Document {
    title: String,
    version: u32,
    authors: Vec<String>,
    metadata: Metadata,
    content: Vec<u8>, // Binary data - more efficient in CBOR
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Metadata {
    created: String,
    tags: Vec<String>,
    size_bytes: usize,
    is_public: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} store              - Store DAG-CBOR data", args[0]);
        eprintln!(
            "  {} get <CID>          - Retrieve DAG-CBOR data from network",
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

/// Store DAG-CBOR data and keep node running
async fn run_store() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Helia DAG-CBOR P2P Sharing Example\n");
    println!("📝 Starting Store Node...\n");

    // Create Helia with custom config for store node
    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(PathBuf::from("/tmp/helia-cbor-store")),
        create_if_missing: true,
    };

    let helia = create_helia(Some(config)).await?;

    // Start the node (enables P2P networking)
    helia.start().await?;
    println!("✅ Helia store node started\n");

    // Create DAG-CBOR instance - just like JS: const cbor = dagCbor(helia)
    let cbor = dag_cbor(Arc::new(helia));

    // Create sample binary document
    let content = b"This is binary content that will be efficiently stored in CBOR format!";

    let document = Document {
        title: "Rust Helia DAG-CBOR Demo".to_string(),
        version: 1,
        authors: vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ],
        metadata: Metadata {
            created: "2025-10-09T12:00:00Z".to_string(),
            tags: vec![
                "rust".to_string(),
                "ipfs".to_string(),
                "dag-cbor".to_string(),
                "p2p".to_string(),
            ],
            size_bytes: content.len(),
            is_public: true,
        },
        content: content.to_vec(),
    };

    println!("📦 Storing DAG-CBOR document:");
    println!("   Title: {}", document.title);
    println!("   Version: {}", document.version);
    println!("   Authors: {:?}", document.authors);
    println!("   Tags: {:?}", document.metadata.tags);
    println!("   Content size: {} bytes\n", document.content.len());

    // Add DAG-CBOR - just like JS: const cid = await cbor.add(data)
    let cid = cbor.add(&document, None).await?;

    println!("✅ DAG-CBOR data stored successfully!");
    println!("🔑 CID: {}\n", cid);

    println!("📋 To retrieve from NETWORK (different node), run in another terminal:");
    println!(
        "   cargo run --example 11_dag_cbor_p2p_sharing -- get {}\n",
        cid
    );

    println!("⏳ Keep this terminal running to serve data over P2P...");
    println!("   Press Ctrl+C to stop\n");

    // Keep the node running to serve blocks
    tokio::signal::ctrl_c().await?;

    println!("\n🛑 Shutting down store node...");

    Ok(())
}

/// Retrieve DAG-CBOR data from network
async fn run_get(cid_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Helia DAG-CBOR P2P Sharing Example\n");
    println!("📥 Starting Retrieve Node...\n");

    // Create Helia with custom config for retrieve node (different blockstore)
    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(PathBuf::from("/tmp/helia-cbor-retrieve")),
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
    println!("🔍 Retrieving DAG-CBOR data for CID: {}\n", cid);

    // Create DAG-CBOR instance
    let cbor = dag_cbor(Arc::new(helia));

    // Get DAG-CBOR - just like JS: const data = await cbor.get(cid)
    println!("⏳ Fetching from network (may take a few seconds for peer discovery)...\n");
    let document: Document = cbor.get(&cid, None).await?;

    println!("✅ DAG-CBOR data retrieved successfully!\n");
    println!("📄 Document Details:");
    println!("   Title: {}", document.title);
    println!("   Version: {}", document.version);
    println!("   Authors: {:?}", document.authors);
    println!("   Created: {}", document.metadata.created);
    println!("   Tags: {:?}", document.metadata.tags);
    println!("   Content size: {} bytes", document.content.len());
    println!(
        "   Content preview: {}\n",
        String::from_utf8_lossy(&document.content[..50.min(document.content.len())])
    );

    println!("🎉 P2P DAG-CBOR retrieval successful!");
    println!("   Data was fetched from the network, not from local storage!");
    println!("   Binary data efficiently encoded with CBOR!\n");

    Ok(())
}
