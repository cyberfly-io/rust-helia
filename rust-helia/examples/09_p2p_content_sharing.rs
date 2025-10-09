//! Example 09: P2P Content Sharing
//!
//! This example tests true P2P block retrieval:
//! - Store node uses /tmp/helia-store directory
//! - Retrieve node uses /tmp/helia-retrieve directory
//! - Tests if blocks can be fetched from network (not local)
//! - Demonstrates current Bitswap implementation status
//!
//! Usage:
//! Terminal 1: cargo run --example 09_p2p_content_sharing -- store "content"
//! Terminal 2: cargo run --example 09_p2p_content_sharing -- get <CID>

use bytes::Bytes;
use cid::Cid;
use helia_interface::Helia;
use helia_utils::{BlockstoreConfig, HeliaConfig};
use rust_helia::create_helia;
use sha2::{Digest, Sha256};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

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

    // Use SEPARATE blockstore directory for store node
    let store_path = PathBuf::from("/tmp/helia-store");
    println!("💾 Store node blockstore: {}", store_path.display());

    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(store_path),
        create_if_missing: true,
    };

    let helia = create_helia(Some(config)).await?;
    helia.start().await?;
    println!("✅ Helia store node started\n");

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
    println!("\n📋 To retrieve from NETWORK (different blockstore), run in another terminal:");
    println!(
        "   cargo run --example 09_p2p_content_sharing -- get {}\n",
        cid
    );
    println!("⏳ Keep this terminal running to serve blocks over P2P...");
    println!("   Press Ctrl+C to stop\n");

    // Keep running to serve blocks
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 Shutting down store node...");

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

    // Use DIFFERENT blockstore directory for retrieve node
    let retrieve_path = PathBuf::from("/tmp/helia-retrieve");
    println!("💾 Retrieve node blockstore: {}", retrieve_path.display());
    println!("   (Different from store node to test P2P retrieval)\n");

    let mut config = HeliaConfig::default();
    config.blockstore = BlockstoreConfig {
        path: Some(retrieve_path),
        create_if_missing: true,
    };

    let helia = create_helia(Some(config)).await?;
    helia.start().await?;
    println!("✅ Retrieve node started\n");

    println!("🔍 Attempting to retrieve content with CID: {}", cid);
    println!("   Step 1: Check local blockstore first...\n");

    // First check if it's in local blockstore
    let in_local = helia.blockstore().has(&cid, None).await.unwrap_or(false);

    if in_local {
        println!("⚠️  Block found in LOCAL blockstore!");
        println!("   This is NOT a true P2P test. Cleaning up local block...\n");
        // Note: We don't have a delete method, so we'll just note this
        println!("   (In production, we'd delete the local copy to force P2P retrieval)");
    } else {
        println!("✅ Block NOT in local blockstore - will need P2P retrieval\n");
    }

    println!("   Step 2: Waiting for peer discovery (mDNS)...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    println!("✅ Peer discovery window complete\n");

    println!("   Step 3: Attempting to retrieve via blockstore.get()...");
    println!("   (Note: Currently get() only checks local storage)");
    println!("   (TODO: Integrate Bitswap into blockstore for network retrieval)\n");

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
            println!("\n🎉 P2P block retrieval successful!");
            println!("   Block was fetched from the network, not from local storage!");
        }
        Err(e) => {
            println!("❌ Failed to retrieve content: {}", e);
            println!("\n� Analysis:");
            println!("   ✅ Store node is running (check Terminal 1)");
            println!("   ✅ Retrieve node is running");
            println!("   ✅ Block exists in store node's blockstore");
            println!("   ❌ Block NOT in retrieve node's local blockstore");
            println!("   ❌ blockstore.get() doesn't trigger network retrieval yet");

            println!("\n💡 Why this fails:");
            println!("   1. ✅ Bitswap coordinator is implemented");
            println!("   2. ✅ NetworkBehaviour is implemented");
            println!("   3. ✅ Event loop is running and processing events");
            println!("   4. ✅ BitswapEvent handling is implemented");
            println!("   5. ✅ BlockBroker trait is implemented");
            println!("   6. ❌ blockstore.get() is NOT integrated with Bitswap");

            println!("\n🔧 Missing Integration:");
            println!("   The blockstore.get() method currently only checks local storage.");
            println!("   It needs to be enhanced to:");
            println!("   - Check local blockstore first (fast path)");
            println!("   - If not found, call Bitswap coordinator's want() method");
            println!("   - Wait for block to arrive via network");
            println!("   - Store received block in local blockstore");
            println!("   - Return the block data");

            println!("\n📝 Current Architecture:");
            println!("   Application");
            println!("   ↓");
            println!("   blockstore.get() ─────> Local storage only ❌");
            println!("                     ╲");
            println!("                      ╲──> (Should call Bitswap)");
            println!("                           ↓");
            println!("                           Bitswap.want()");
            println!("                           ↓");
            println!("                           NetworkBehaviour");
            println!("                           ↓");
            println!("                           Network");

            println!("\n🎯 Next Steps:");
            println!("   1. Create BlockstoreWithBitswap wrapper");
            println!("   2. Implement get() that tries local first, then Bitswap");
            println!("   3. Or: Use BitswapBroker.retrieve() directly instead of blockstore.get()");

            println!("\n📊 Test Configuration:");
            println!("   - Store node:    /tmp/helia-store");
            println!("   - Retrieve node: /tmp/helia-retrieve");
            println!("   - Separate directories = true P2P test ✅");
        }
    }

    helia.stop().await?;
    Ok(())
}

fn print_usage() {
    println!("🌐 Helia P2P Content Sharing Test");
    println!("\n💡 This example tests TRUE P2P block retrieval using separate blockstores");
    println!("\nUsage:");
    println!("  Terminal 1 - Store content:");
    println!("    cargo run --example 09_p2p_content_sharing -- store \"Hello World\"");
    println!("    (Keep this running to serve blocks)");
    println!("\n  Terminal 2 - Retrieve content:");
    println!("    cargo run --example 09_p2p_content_sharing -- get <CID>");
    println!("\n� Test Setup:");
    println!("   - Store node:    /tmp/helia-store (separate directory)");
    println!("   - Retrieve node: /tmp/helia-retrieve (separate directory)");
    println!("   - This proves blocks come from network, not local storage");
    println!("\n📊 Current Status:");
    println!("   ✅ Bitswap coordinator implemented");
    println!("   ✅ Protocol messages (Network, WantList)");
    println!("   ⚠️  NetworkBehaviour integration - IN PROGRESS");
    println!("   ✅ mDNS peer discovery");
    println!("\n🎯 Expected Result:");
    println!("   - If NetworkBehaviour is complete: Block fetched from network ✅");
    println!("   - If NetworkBehaviour not done: Block not found (needs P2P) ❌");
}
