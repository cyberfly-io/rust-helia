//! Directory operations with UnixFS
//!
//! This example demonstrates:
//! - Creating directories
//! - Adding files to directories
//! - Listing directory contents
//! - Nested directories
//! - Removing entries

use bytes::Bytes;
use futures::StreamExt;
use helia_unixfs::{UnixFS, UnixFSInterface};
use rust_helia::create_helia_default;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing Helia node...");
    let helia = Arc::new(create_helia_default().await?);
    let fs = UnixFS::new(helia);

    // Example 1: Create an empty directory
    println!("\n📁 Example 1: Creating an empty directory");
    let dir_cid = fs.add_directory(None, None).await?;
    println!("✅ Directory created!");
    println!("   CID: {}", dir_cid);

    // Example 2: Add files to directory
    println!("\n📄 Example 2: Adding files to directory");

    // Add first file
    let file1_data = Bytes::from("Hello from file1.txt");
    let file1_cid = fs.add_bytes(file1_data, None).await?;
    println!("   Created file1.txt: {}", file1_cid);

    // Add file to directory
    let dir_cid = fs.cp(&file1_cid, &dir_cid, "file1.txt", None).await?;
    println!("✅ Added file1.txt to directory");

    // Add second file
    let file2_data = Bytes::from("Content of file2.txt");
    let file2_cid = fs.add_bytes(file2_data, None).await?;
    println!("   Created file2.txt: {}", file2_cid);

    let dir_cid = fs.cp(&file2_cid, &dir_cid, "file2.txt", None).await?;
    println!("✅ Added file2.txt to directory");

    // Example 3: List directory contents
    println!("\n📋 Example 3: Listing directory contents");
    let mut entries = fs.ls(&dir_cid, None).await?;

    println!("   Directory contents:");
    while let Some(entry) = entries.next().await {
        println!(
            "   - {} (CID: {}, size: {} bytes, type: {:?})",
            entry.name, entry.cid, entry.size, entry.type_
        );
    }

    // Example 4: Create nested directory structure
    println!("\n📁 Example 4: Creating nested directories");

    // Create subdirectory
    let subdir_cid = fs.add_directory(None, None).await?;
    println!("   Created subdirectory: {}", subdir_cid);

    // Add file to subdirectory
    let nested_file_data = Bytes::from("Nested file content");
    let nested_file_cid = fs.add_bytes(nested_file_data, None).await?;
    let subdir_cid = fs
        .cp(&nested_file_cid, &subdir_cid, "nested.txt", None)
        .await?;
    println!("   Added nested.txt to subdirectory");

    // Add subdirectory to main directory
    let dir_cid = fs.cp(&subdir_cid, &dir_cid, "subdir", None).await?;
    println!("✅ Added subdirectory to main directory");

    // List updated directory
    println!("\n📋 Updated directory structure:");
    let mut entries = fs.ls(&dir_cid, None).await?;
    while let Some(entry) = entries.next().await {
        println!("   - {} (type: {:?})", entry.name, entry.type_);

        // If it's a directory, list its contents too
        if entry.name == "subdir" {
            let mut sub_entries = fs.ls(&entry.cid, None).await?;
            while let Some(sub_entry) = sub_entries.next().await {
                println!("     └─ {} (type: {:?})", sub_entry.name, sub_entry.type_);
            }
        }
    }

    // Example 5: Use mkdir convenience function
    println!("\n📁 Example 5: Using mkdir");
    let dir_cid = fs.mkdir(&dir_cid, "new_folder", None).await?;
    println!("✅ Created 'new_folder' using mkdir");

    // Example 6: Remove an entry
    println!("\n🗑️  Example 6: Removing an entry");
    let dir_cid = fs.rm(&dir_cid, "file1.txt", None).await?;
    println!("✅ Removed file1.txt from directory");

    // List final directory contents
    println!("\n📋 Final directory contents:");
    let mut entries = fs.ls(&dir_cid, None).await?;
    while let Some(entry) = entries.next().await {
        println!("   - {} (type: {:?})", entry.name, entry.type_);
    }

    // Example 7: Get directory statistics
    println!("\n📊 Example 7: Directory statistics");
    let stat = fs.stat(&dir_cid, None).await?;
    println!("✅ Directory stats:");
    println!("   {:?}", stat);

    println!("\n🎉 All directory examples completed successfully!");

    Ok(())
}
