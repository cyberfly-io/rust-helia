//! Block storage and retrieval example
//!
//! This example demonstrates:
//! - Storing raw blocks
//! - Retrieving blocks by CID
//! - Checking block existence
//! - Deleting blocks
//! - Batch operations

use helia::create_helia;
use helia_interface::{Blocks, InputPair};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Block Storage Example ===\n");

    let helia = create_helia(None).await?;
    helia.start().await?;

    // 1. Store a single block
    println!("1. Storing a single block...");
    let data = Bytes::from("Hello, IPFS! This is my first block.");
    let cid = helia.blockstore().put(data.clone(), None).await?;
    println!("   ✓ Stored block with CID: {}\n", cid);

    // 2. Retrieve the block
    println!("2. Retrieving the block...");
    let retrieved = helia.blockstore().get(&cid, None).await?;
    let text = String::from_utf8(retrieved.to_vec())?;
    println!("   ✓ Retrieved: \"{}\"\n", text);

    // 3. Check if block exists
    println!("3. Checking block existence...");
    let exists = helia.blockstore().has(&cid, None).await?;
    println!("   ✓ Block exists: {}\n", exists);

    // 4. Store multiple blocks at once
    println!("4. Storing multiple blocks...");
    let blocks = vec![
        InputPair {
            cid: None,
            block: Bytes::from("Block 1: First batch block"),
        },
        InputPair {
            cid: None,
            block: Bytes::from("Block 2: Second batch block"),
        },
        InputPair {
            cid: None,
            block: Bytes::from("Block 3: Third batch block"),
        },
    ];
    
    let cids = helia.blockstore().put_many(blocks, None).await?;
    println!("   ✓ Stored {} blocks:", cids.len());
    for (i, cid) in cids.iter().enumerate() {
        println!("     - Block {}: {}", i + 1, cid);
    }
    println!();

    // 5. Retrieve all stored blocks
    println!("5. Retrieving all stored blocks...");
    for (i, cid) in cids.iter().enumerate() {
        let data = helia.blockstore().get(cid, None).await?;
        let text = String::from_utf8(data.to_vec())?;
        println!("   - Block {}: \"{}\"", i + 1, text);
    }
    println!();

    // 6. Delete a block
    println!("6. Deleting first block...");
    helia.blockstore().delete(&cid, None).await?;
    let still_exists = helia.blockstore().has(&cid, None).await?;
    println!("   ✓ Block deleted. Still exists: {}\n", still_exists);

    helia.stop().await?;
    println!("Example completed successfully!");
    
    Ok(())
}
