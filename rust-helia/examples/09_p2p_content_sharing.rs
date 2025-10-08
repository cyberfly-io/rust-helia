//! Example 09: P2P Content Sharing
//! 
//! This example demonstrates why blocks can't be found across P2P nodes:
//! - Bitswap protocol is not fully implemented yet
//! - Blocks are only stored locally, not shared over network
//! - Shows workaround using shared blockstore directory
//! 
//! Usage:
//! cargo run --example 09_p2p_content_sharing -- store "content"
//! cargo run --example 09_p2p_content_sharing -- get <CID>

use bytes::Bytes;
use cid::Cid;
use helia_interface::Helia;
use helia_utils::{HeliaConfig, BlockstoreConfig};
use rust_helia::create_helia;
use sha2::{Sha256, Digest};
use std::env;
use std::str::FromStr;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌐 Helia P2P Content Sharing Example\n");
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "store" => {
            if args.len() < 3 {
                println!("❌ Error: Please provide content to store");
                return Ok(());
            }
            let content = args[2..].join(" ");
            store_content(&content).await?;
        }
        "get" => {
            if args.len() < 3 {
                println!("❌ Error: Please provide a CID to retrieve");
                return Ok(());
            }
            let cid_str = &args[2];
            retrieve_content(cid_str).await?;
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}

async fn store_content(content: &str) -> anyhow::Result<()> {
    println!("📝 Starting Store Node...\n");
    
    // Use shared blockstore for demonstration
    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(PathBuf::from("/tmp/helia-p2p-demo")),
        create_if_missing: true,
    };
    
    let helia = create_helia(Some(config)).await?;
    helia.start().await?;
    println!("✅ Helia node started with shared blockstore\n");
    
    // Create content and CID
    let data = Bytes::from(content.to_string());
    println!("📦 Storing content: \"{}\"", content);
    
    // Create SHA-256 hash for content
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash_result = hasher.finalize();
    
    // Build multihash (sha2-256 code 0x12, length 0x20)
    let mut mh_bytes = vec![0x12, 0x20];
    mh_bytes.extend_from_slice(&hash_result);
    let mh = multihash::Multihash::from_bytes(&mh_bytes)?;
    let cid = Cid::new_v1(0x55, mh); // raw codec
    
    // Store the block
    helia.blockstore().put(&cid, data.clone(), None).await?;
    println!("✅ Content stored successfully!\n");
    
    println!("🔑 CID: {}", cid);
    println!("\n📋 To retrieve this content, run:");
    println!("   cargo run --example 09_p2p_content_sharing -- get {}\n", cid);
    
    helia.stop().await?;
    Ok(())
}

async fn retrieve_content(cid_str: &str) -> anyhow::Result<()> {
    println!("📥 Starting Retrieve Node...\n");
    
    // Parse CID
    let cid = match Cid::from_str(cid_str) {
        Ok(c) => c,
        Err(e) => {
            println!("❌ Error: Invalid CID format: {}", e);
            return Ok(());
        }
    };
    
    // Use same shared blockstore
    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(PathBuf::from("/tmp/helia-p2p-demo")),
        create_if_missing: true,
    };
    
    let helia = create_helia(Some(config)).await?;
    
        helia.start().await?;
    println!("✅ Retrieve node started with shared blockstore\n");
    
    println!("🔍 Attempting to retrieve content with CID: {}", cid);
    
    match helia.blockstore().get(&cid, None).await {
        Ok(data) => {
            println!("✅ Content retrieved successfully!\n");
            match String::from_utf8(data.to_vec()) {
                Ok(text) => {
                    println!("📄 Content: \"{}\"", text);
                }
                Err(_) => {
                    println!("📄 Content (binary): {} bytes", data.len());
                }
            }
            println!("\n🎉 Content sharing successful!");
        }
        Err(e) => {
            println!("❌ Failed to retrieve content: {}", e);
            println!("\n💡 Why blocks can't be found in P2P:");
            println!("   1. ❌ Bitswap protocol is not fully implemented yet");
            println!("   2. ❌ Blocks are only stored locally, not shared over network");
            println!("   3. ❌ P2P block exchange doesn't work between separate nodes");
            println!("   4. ✅ Workaround: Both nodes use same shared blockstore directory");
            println!("\n📝 Current Implementation Status:");
            println!("   - ✅ mDNS peer discovery works");
            println!("   - ✅ Local blockstore operations work");
            println!("   - ❌ P2P block exchange is in development");
            println!("   - ✅ Shared blockstore demonstrates the concept");
            println!("\n🔧 To make this work:");
            println!("   1. Run store command first to create content");
            println!("   2. Both nodes use /tmp/helia-p2p-demo directory");
            println!("   3. Wait for full Bitswap implementation for true P2P");
        }
    }
    
    helia.stop().await?;
    Ok(())
}

fn print_usage() {
    println!("🌐 Helia P2P Content Sharing Example");
    println!("\n💡 This example explains why blocks can't be found across P2P nodes");
    println!("\nUsage:");
    println!("  Store content:");
    println!("    cargo run --example 09_p2p_content_sharing -- store \"Hello World\"");
    println!("\n  Retrieve content:");
    println!("    cargo run --example 09_p2p_content_sharing -- get <CID>");
    println!("\n🚧 Current Limitations:");
    println!("   - Bitswap protocol is in development");
    println!("   - P2P block exchange doesn't work yet");
    println!("   - Uses shared blockstore as workaround");
    println!("\n✅ What Works:");
    println!("   - mDNS peer discovery");
    println!("   - Local blockstore operations");
    println!("   - Content addressing with CIDs");
}
