//! Pinning example
//!
//! This example demonstrates:
//! - Pinning content to prevent garbage collection
//! - Checking pin status
//! - Listing all pins
//! - Unpinning content

use rust_helia::create_helia;
use helia_interface::{Helia, Blocks, Pins};
use helia_unixfs::{UnixFS, UnixFSInterface};
use bytes::Bytes;
use std::sync::Arc;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pinning Example ===\n");

    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;
    
    let fs = UnixFS::new(helia.clone());

    // 1. Create some content
    println!("1. Creating content...");
    let file1 = Bytes::from("Important file that should be pinned");
    let cid1 = fs.add_bytes(file1, None).await?;
    println!("   ✓ File 1 CID: {}", cid1);
    
    let file2 = Bytes::from("Another important file");
    let cid2 = fs.add_bytes(file2, None).await?;
    println!("   ✓ File 2 CID: {}", cid2);
    
    let file3 = Bytes::from("Temporary file");
    let cid3 = fs.add_bytes(file3, None).await?;
    println!("   ✓ File 3 CID: {}\n", cid3);

    // 2. Pin the first two files
    println!("2. Pinning files...");
    helia.pins().add(&cid1, None).await?;
    println!("   ✓ Pinned file 1");
    
    helia.pins().add(&cid2, None).await?;
    println!("   ✓ Pinned file 2\n");

    // 3. Check pin status
    println!("3. Checking pin status...");
    let is_pinned_1 = helia.pins().is_pinned(&cid1, None).await?;
    println!("   File 1 is pinned: {}", is_pinned_1);
    
    let is_pinned_2 = helia.pins().is_pinned(&cid2, None).await?;
    println!("   File 2 is pinned: {}", is_pinned_2);
    
    let is_pinned_3 = helia.pins().is_pinned(&cid3, None).await?;
    println!("   File 3 is pinned: {}\n", is_pinned_3);

    // 4. List all pins
    println!("4. Listing all pins...");
    let mut pin_stream = helia.pins().ls(None).await?;
    let mut pin_count = 0;
    
    while let Some(pin) = pin_stream.next().await {
        pin_count += 1;
        println!("   - Pin {}: {}", pin_count, pin.cid);
    }
    println!("   ✓ Total pins: {}\n", pin_count);

    // 5. Create a directory and pin it
    println!("5. Creating and pinning a directory...");
    let dir_cid = fs.add_directory(None, None).await?;
    let dir_cid = fs.cp(&cid1, &dir_cid, "file1.txt", None).await?;
    let dir_cid = fs.cp(&cid2, &dir_cid, "file2.txt", None).await?;
    
    helia.pins().add(&dir_cid, None).await?;
    println!("   ✓ Directory CID: {}", dir_cid);
    println!("   ✓ Directory pinned\n");

    // 6. Unpin a file
    println!("6. Unpinning file 1...");
    helia.pins().rm(&cid1, None).await?;
    println!("   ✓ File 1 unpinned");
    
    let still_pinned = helia.pins().is_pinned(&cid1, None).await?;
    println!("   File 1 is still pinned: {}\n", still_pinned);

    // 7. Final pin status
    println!("7. Final pin status:");
    println!("   File 1: {}", helia.pins().is_pinned(&cid1, None).await?);
    println!("   File 2: {}", helia.pins().is_pinned(&cid2, None).await?);
    println!("   File 3: {}", helia.pins().is_pinned(&cid3, None).await?);
    println!("   Directory: {}\n", helia.pins().is_pinned(&dir_cid, None).await?);

    // 8. List all pins again
    println!("8. Final pin list:");
    let mut final_pins = helia.pins().ls(None).await?;
    let mut count = 0;
    
    while let Some(pin) = final_pins.next().await {
        count += 1;
        println!("   - Pin {}: {}", count, pin.cid);
    }
    println!("   ✓ Total pins: {}\n", count);

    helia.stop().await?;
    println!("Example completed successfully!");
    
    Ok(())
}
