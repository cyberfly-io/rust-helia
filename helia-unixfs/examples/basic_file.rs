//! Basic file operations with UnixFS
//!
//! This example demonstrates how to:
//! - Add a file to UnixFS storage
//! - Retrieve the file by its CID
//! - Work with file metadata

use bytes::Bytes;
use helia_unixfs::{UnixFS, UnixFSInterface, AddOptions};
use rust_helia::create_helia_default;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Helia node
    println!("🚀 Initializing Helia node...");
    let helia = Arc::new(create_helia_default().await?);
    
    // Create UnixFS instance
    let fs = UnixFS::new(helia);
    
    // Example 1: Add a simple text file
    println!("\n📄 Example 1: Adding a text file");
    let content = "Hello, IPFS! This is a test file stored using UnixFS.";
    let data = Bytes::from(content);
    
    let cid = fs.add_bytes(data.clone(), None).await?;
    println!("✅ File added successfully!");
    println!("   CID: {}", cid);
    
    // Retrieve the file
    println!("\n📥 Retrieving file by CID...");
    let retrieved = fs.cat(&cid, None).await?;
    let retrieved_text = String::from_utf8(retrieved.to_vec())?;
    println!("✅ File retrieved: \"{}\"", retrieved_text);
    
    // Verify content matches
    assert_eq!(retrieved_text, content);
    println!("✅ Content verification passed!");
    
    // Example 2: Add file with raw leaves option
    println!("\n📄 Example 2: Adding file with RAW codec");
    let options = AddOptions {
        raw_leaves: true,
        ..Default::default()
    };
    
    let cid_raw = fs.add_bytes(Bytes::from("Raw codec content"), Some(options)).await?;
    println!("✅ File with RAW codec added!");
    println!("   CID: {}", cid_raw);
    
    let retrieved_raw = fs.cat(&cid_raw, None).await?;
    println!("✅ Retrieved: \"{}\"", String::from_utf8(retrieved_raw.to_vec())?);
    
    // Example 3: Get file statistics
    println!("\n📊 Example 3: File statistics");
    let stat = fs.stat(&cid, None).await?;
    println!("✅ File stats:");
    println!("   {:?}", stat);
    
    println!("\n🎉 All examples completed successfully!");
    
    Ok(())
}
