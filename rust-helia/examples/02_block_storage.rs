//! Example 02: Block Storage
//!
//! Demonstrates low-level block storage operations including:
//! - Storing raw blocks
//! - Retrieving blocks by CID
//! - Checking block existence
//! - Deleting blocks

use bytes::Bytes;
use cid::Cid;
use helia_interface::Helia;
use rust_helia::create_helia;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔷 Helia Block Storage Example\n");

    // Initialize Helia
    let helia = create_helia(None).await?;

    helia.start().await?;
    println!("✅ Helia node started\n");

    // Create test data
    let data = Bytes::from("Hello from Helia block storage!");
    println!("📝 Test data: {:?}\n", String::from_utf8_lossy(&data));

    // Use the same approach as in blockstore_tests.rs:
    // Create a CID from a fixed hash
    let hash_bytes = [
        0x12, 0x20, // sha2-256 code (0x12) and length (0x20 = 32 bytes)
        0x9f, 0x86, 0xd0, 0x81, 0x88, 0x4c, 0x7d, 0x65, 0x9a, 0x2f, 0xea, 0xa0, 0xc5, 0x5a, 0xd0,
        0x15, 0xa3, 0xbf, 0x4f, 0x1b, 0x2b, 0x0b, 0x82, 0x2c, 0xd1, 0x5d, 0x6c, 0x15, 0xb0, 0xf0,
        0x0a, 0x08,
    ];
    let mh = multihash::Multihash::from_bytes(&hash_bytes)?;
    let cid = Cid::new_v1(0x55, mh); // 0x55 is raw codec

    println!("🔑 Generated CID: {}\n", cid);

    // Store the block
    println!("💾 Storing block...");
    helia.blockstore().put(&cid, data.clone(), None).await?;
    println!("✅ Block stored successfully\n");

    // Check if block exists
    println!("🔍 Checking if block exists...");
    let exists = helia.blockstore().has(&cid, None).await?;
    println!("✅ Block exists: {}\n", exists);

    // Retrieve the block
    println!("📥 Retrieving block...");
    let retrieved = helia.blockstore().get(&cid, None).await?;
    println!(
        "✅ Block retrieved: {:?}\n",
        String::from_utf8_lossy(&retrieved)
    );

    // Verify content matches
    if data == retrieved {
        println!("✅ Data integrity verified!\n");
    }

    // Delete the block using delete_many_cids
    println!("🗑️  Deleting block...");
    helia.blockstore().delete_many_cids(vec![cid], None).await?;
    println!("✅ Block deleted\n");

    // Verify deletion
    let exists_after = helia.blockstore().has(&cid, None).await?;
    println!("🔍 Block exists after deletion: {}\n", exists_after);

    println!("🎉 Block storage example completed successfully!");

    Ok(())
}
