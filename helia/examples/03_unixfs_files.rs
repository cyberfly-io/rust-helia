//! UnixFS file operations example
//!
//! This example demonstrates:
//! - Adding files to IPFS
//! - Reading file content
//! - Creating directories
//! - Adding files to directories
//! - Listing directory contents
//! - Getting file statistics

use helia::create_helia;
use helia_unixfs::{UnixFS, UnixFSInterface, AddBytesOptions};
use bytes::Bytes;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== UnixFS File Operations Example ===\n");

    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;
    
    let fs = UnixFS::new(helia.clone());

    // 1. Add a simple file
    println!("1. Adding a text file...");
    let content = Bytes::from("Hello, UnixFS! This is a simple text file.");
    let file_cid = fs.add_bytes(content, None).await?;
    println!("   ✓ File CID: {}\n", file_cid);

    // 2. Read the file back
    println!("2. Reading the file...");
    let retrieved = fs.cat(&file_cid, None).await?;
    let text = String::from_utf8(retrieved.to_vec())?;
    println!("   ✓ Content: \"{}\"\n", text);

    // 3. Get file statistics
    println!("3. Getting file statistics...");
    let stats = fs.stat(&file_cid, None).await?;
    println!("   ✓ File type: {:?}", stats.file_type);
    println!("   ✓ File size: {} bytes", stats.file_size);
    println!("   ✓ Block count: {}\n", stats.blocks);

    // 4. Create a directory
    println!("4. Creating a directory...");
    let dir_cid = fs.add_directory(None, None).await?;
    println!("   ✓ Directory CID: {}\n", dir_cid);

    // 5. Add multiple files to the directory
    println!("5. Adding files to directory...");
    
    let file1_data = Bytes::from("This is file 1");
    let file1_cid = fs.add_bytes(file1_data, None).await?;
    let dir_cid = fs.cp(&file1_cid, &dir_cid, "file1.txt", None).await?;
    println!("   ✓ Added file1.txt");

    let file2_data = Bytes::from("This is file 2 with more content");
    let file2_cid = fs.add_bytes(file2_data, None).await?;
    let dir_cid = fs.cp(&file2_cid, &dir_cid, "file2.txt", None).await?;
    println!("   ✓ Added file2.txt");

    let file3_data = Bytes::from("File 3 content here");
    let file3_cid = fs.add_bytes(file3_data, None).await?;
    let dir_cid = fs.cp(&file3_cid, &dir_cid, "file3.txt", None).await?;
    println!("   ✓ Added file3.txt\n");

    // 6. List directory contents
    println!("6. Listing directory contents...");
    let entries = fs.ls(&dir_cid, None).await?;
    println!("   Directory contains {} entries:", entries.len());
    for entry in entries {
        println!("     - {} ({})", entry.name, entry.cid);
        if let Some(size) = entry.size {
            println!("       Size: {} bytes", size);
        }
    }
    println!();

    // 7. Add a large file with custom chunking
    println!("7. Adding a large file with custom chunking...");
    let large_content = Bytes::from(vec![42u8; 512 * 1024]); // 512 KB
    let options = AddBytesOptions {
        chunk_size: Some(256 * 1024), // 256 KB chunks
        ..Default::default()
    };
    let large_file_cid = fs.add_bytes(large_content, Some(options)).await?;
    println!("   ✓ Large file CID: {}", large_file_cid);
    
    let large_stats = fs.stat(&large_file_cid, None).await?;
    println!("   ✓ Large file size: {} bytes", large_stats.file_size);
    println!("   ✓ Large file blocks: {}\n", large_stats.blocks);

    helia.stop().await?;
    println!("Example completed successfully!");
    
    Ok(())
}
