//! CAR file import/export example
//!
//! This example demonstrates:
//! - Creating content and working with CAR files
//! - Using the SimpleCar in-memory implementation
//! - Adding and retrieving blocks

use bytes::Bytes;
use helia_car::SimpleCar;
use helia_interface::Helia;
use helia_unixfs::{UnixFS, UnixFSInterface};
use rust_helia::create_helia;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CAR File Operations Example ===\n");

    // Initialize Helia
    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;

    let fs = UnixFS::new(helia.clone());

    // 1. Create some content
    println!("1. Creating content...");
    let file1 = Bytes::from("This is file 1");
    let file2 = Bytes::from("This is file 2 with more content");
    let file3 = Bytes::from("File 3 content here");

    let cid1 = fs.add_bytes(file1.clone(), None).await?;
    let cid2 = fs.add_bytes(file2.clone(), None).await?;
    let cid3 = fs.add_bytes(file3.clone(), None).await?;

    println!("   ✓ File 1 CID: {}", cid1);
    println!("   ✓ File 2 CID: {}", cid2);
    println!("   ✓ File 3 CID: {}\n", cid3);

    // 2. Create a SimpleCar and add the blocks
    println!("2. Creating CAR archive...");
    let mut car = SimpleCar::new();
    car.add_block(cid1, file1.clone());
    car.add_block(cid2, file2.clone());
    car.add_block(cid3, file3.clone());
    println!("   ✓ Added {} blocks to CAR\n", car.len());

    // 3. Verify blocks in CAR
    println!("3. Verifying blocks in CAR...");
    println!("   ✓ Has CID1: {}", car.has_block(&cid1));
    println!("   ✓ Has CID2: {}", car.has_block(&cid2));
    println!("   ✓ Has CID3: {}\n", car.has_block(&cid3));

    // 4. Retrieve blocks from CAR
    println!("4. Retrieving blocks from CAR...");
    if let Some(data) = car.get_block(&cid1) {
        println!("   ✓ Retrieved file 1: {:?}", String::from_utf8_lossy(data));
    }
    if let Some(data) = car.get_block(&cid2) {
        println!("   ✓ Retrieved file 2: {:?}", String::from_utf8_lossy(data));
    }
    if let Some(data) = car.get_block(&cid3) {
        println!(
            "   ✓ Retrieved file 3: {:?}\n",
            String::from_utf8_lossy(data)
        );
    }

    // 5. List all blocks
    println!("5. Listing all blocks in CAR...");
    println!("   Total blocks: {}", car.blocks().len());
    for (cid, data) in car.blocks() {
        println!("     - {} ({} bytes)", cid, data.len());
    }
    println!();

    helia.stop().await?;
    println!("✓ CAR operations example completed successfully!");

    Ok(())
}
