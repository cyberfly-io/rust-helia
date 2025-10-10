//! MFS (Mutable File System) Example
//!
//! This example demonstrates how to use the MFS API to work with IPFS
//! content using familiar file system operations like mkdir, write, and ls.

use rust_helia::create_helia_default;
use helia_mfs::{mfs, MfsInterface};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Helia node
    let helia = Arc::new(create_helia_default().await?);
    println!("✓ Helia node initialized");

    // Create MFS instance
    let fs = mfs(helia.clone());
    println!("✓ MFS instance created\n");

    // === Create Directory Structure ===
    println!("=== Creating Directory Structure ===");
    
    // Create nested directories (like mkdir -p)
    fs.mkdir("/docs").await?;
    println!("✓ Created /docs");
    
    fs.mkdir("/docs/tutorials").await?;
    println!("✓ Created /docs/tutorials");
    
    fs.mkdir("/projects").await?;
    println!("✓ Created /projects");
    
    fs.mkdir("/projects/rust").await?;
    println!("✓ Created /projects/rust");
    
    fs.mkdir("/projects/rust/examples").await?;
    println!("✓ Created /projects/rust/examples\n");

    // === Write Files ===
    println!("=== Writing Files ===");
    
    fs.write_bytes("/README.md", b"# My IPFS Project\n\nWelcome to my project!").await?;
    println!("✓ Written /README.md");
    
    fs.write_bytes("/docs/intro.txt", b"Introduction to IPFS and Helia").await?;
    println!("✓ Written /docs/intro.txt");
    
    fs.write_bytes(
        "/projects/hello.txt",
        b"Hello from the Rust-Helia MFS example!"
    ).await?;
    println!("✓ Written /projects/hello.txt\n");

    // === List Directory Contents ===
    println!("=== Listing Directory Contents ===");
    
    // List root directory
    let root_entries = fs.ls("/").await?;
    println!("\nContents of /:");
    for entry in root_entries {
        let type_str = match entry.type_ {
            helia_unixfs::UnixFSType::File => "📄 file",
            helia_unixfs::UnixFSType::Directory => "📁 dir ",
            _ => "❓ other",
        };
        println!("  {} {} ({} bytes) - {}", 
            type_str,
            entry.name,
            entry.size,
            entry.cid
        );
    }

    // === Get File Statistics ===
    println!("\n=== File Statistics ===\n");
    
    let readme_stat = fs.stat("/README.md").await?;
    println!("Stats for /README.md:");
    println!("  CID:  {}", readme_stat.cid);
    println!("  Size: {} bytes", readme_stat.size);
    println!("  Type: {:?}", readme_stat.type_);

    let docs_stat = fs.stat("/docs").await?;
    println!("\nStats for /docs:");
    println!("  CID:  {}", docs_stat.cid);
    println!("  Size: {} bytes", docs_stat.size);
    println!("  Type: {:?}", docs_stat.type_);

    // === Get Root CID ===
    println!("\n=== File System Root ===\n");
    
    let root_cid = fs.root_cid().await;
    if let Some(cid) = root_cid {
        println!("Current MFS root CID: {}", cid);
        println!("\nYou can share this CID to share your entire file system!");
        println!("Others can access it via: ipfs://{}", cid);
    }

    // === Flush Changes ===
    println!("\n=== Flushing Changes ===\n");
    
    let flushed_cid = fs.flush().await?;
    println!("✓ Changes flushed. Final root CID: {}", flushed_cid);

    println!("\n=== Summary ===\n");
    println!("✓ Created 5 directories");
    println!("✓ Written 3 files");
    println!("✓ All operations completed successfully");
    println!("\nThe MFS provides a mutable layer over IPFS content,");
    println!("allowing you to work with familiar file system operations");
    println!("while maintaining content-addressable storage benefits.");

    Ok(())
}
