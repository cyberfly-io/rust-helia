//! Large file operations with automatic chunking
//!
//! This example demonstrates:
//! - Adding large files (>1MB) with automatic chunking
//! - Custom chunk sizes
//! - Reading chunked files
//! - Partial reads with offset/length

use bytes::Bytes;
use helia_unixfs::{AddOptions, CatOptions, UnixFS, UnixFSInterface};
use rust_helia::create_helia_default;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing Helia node...");
    let helia = Arc::new(create_helia_default().await?);
    let fs = UnixFS::new(helia);

    // Example 1: Add a large file (2MB)
    println!("\n📦 Example 1: Adding a 2MB file with automatic chunking");
    let size = 2_000_000; // 2MB
    let large_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    let data = Bytes::from(large_data);

    println!("   File size: {} bytes", size);

    let options = AddOptions {
        raw_leaves: true,
        chunk_size: Some(1_048_576), // 1MB chunks
        ..Default::default()
    };

    println!("   Chunk size: 1MB");
    println!("   Expected chunks: ~2");

    let cid = fs.add_bytes(data.clone(), Some(options)).await?;
    println!("✅ Large file added successfully!");
    println!("   CID: {}", cid);

    // Get statistics to see chunking info
    let stat = fs.stat(&cid, None).await?;
    println!("✅ File statistics:");
    println!("   {:?}", stat);

    // Example 2: Retrieve the complete file
    println!("\n📥 Example 2: Retrieving complete file");
    let retrieved = fs.cat(&cid, None).await?;
    println!("✅ File retrieved successfully!");
    println!("   Retrieved size: {} bytes", retrieved.len());

    // Verify content
    assert_eq!(retrieved, data);
    println!("✅ Content verification passed!");

    // Example 3: Partial read (first 1KB)
    println!("\n📖 Example 3: Reading first 1KB");
    let options = CatOptions {
        offset: Some(0),
        length: Some(1024),
        ..Default::default()
    };

    let partial = fs.cat(&cid, Some(options)).await?;
    println!("✅ Partial read successful!");
    println!("   Read {} bytes", partial.len());
    assert_eq!(partial.len(), 1024);

    // Example 4: Read from offset (skip first 1MB, read next 1KB)
    println!("\n📖 Example 4: Reading 1KB from offset 1MB");
    let options = CatOptions {
        offset: Some(1_048_576),
        length: Some(1024),
        ..Default::default()
    };

    let offset_read = fs.cat(&cid, Some(options)).await?;
    println!("✅ Offset read successful!");
    println!("   Read {} bytes from offset 1MB", offset_read.len());
    assert_eq!(offset_read.len(), 1024);

    // Example 5: Custom chunk size (512KB)
    println!("\n📦 Example 5: Custom chunk size (512KB)");
    let options = AddOptions {
        raw_leaves: true,
        chunk_size: Some(524_288), // 512KB
        ..Default::default()
    };

    let size = 1_500_000; // 1.5MB
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    println!("   File size: {} bytes", size);
    println!("   Chunk size: 512KB");
    println!("   Expected chunks: ~3");

    let cid_custom = fs.add_bytes(Bytes::from(data), Some(options)).await?;
    println!("✅ File with custom chunks added!");
    println!("   CID: {}", cid_custom);

    let stat = fs.stat(&cid_custom, None).await?;
    println!("✅ File statistics:");
    println!("   {:?}", stat);

    println!("\n🎉 All large file examples completed successfully!");

    Ok(())
}
