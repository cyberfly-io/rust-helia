//! File metadata operations
//!
//! This example demonstrates:
//! - Adding files with custom metadata (mode, mtime)
//! - Retrieving metadata
//! - Working with file permissions

use bytes::Bytes;
use helia_unixfs::{UnixFS, UnixFSInterface, FileCandidate, UnixFSTime};
use rust_helia::create_helia_default;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing Helia node...");
    let helia = Arc::new(create_helia_default().await?);
    let fs = UnixFS::new(helia);
    
    // Example 1: Add file with custom permissions (mode)
    println!("\n🔐 Example 1: File with custom permissions");
    
    let file = FileCandidate {
        path: "custom_permissions.txt".to_string(),
        content: Bytes::from("This file has custom permissions"),
        mode: Some(0o644), // rw-r--r--
        mtime: None,
    };
    
    let cid = fs.add_file(file, None).await?;
    println!("✅ File added with mode 0o644 (rw-r--r--)");
    println!("   CID: {}", cid);
    
    // Retrieve and check metadata
    let stat = fs.stat(&cid, None).await?;
    println!("✅ File metadata:");
    println!("   {:?}", stat);
    
    // Example 2: Add file with modification time
    println!("\n🕐 Example 2: File with modification time");
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    let file = FileCandidate {
        path: "timestamped.txt".to_string(),
        content: Bytes::from("This file has a timestamp"),
        mode: Some(0o755), // rwxr-xr-x
        mtime: Some(UnixFSTime {
            seconds: now,
            nanoseconds: Some(123456789),
        }),
    };
    
    let cid = fs.add_file(file, None).await?;
    println!("✅ File added with mtime: {} seconds since epoch", now);
    println!("   Mode: 0o755 (rwxr-xr-x)");
    println!("   CID: {}", cid);
    
    let stat = fs.stat(&cid, None).await?;
    println!("✅ File metadata:");
    println!("   {:?}", stat);
    
    // Example 3: Different permission modes
    println!("\n🔐 Example 3: Various permission modes");
    
    let modes = vec![
        (0o644, "rw-r--r--", "Regular file (readable by all)"),
        (0o600, "rw-------", "Private file (owner only)"),
        (0o755, "rwxr-xr-x", "Executable (runnable by all)"),
        (0o700, "rwx------", "Private executable"),
    ];
    
    for (mode, symbolic, description) in modes {
        let file = FileCandidate {
            path: format!("file_{:o}.txt", mode),
            content: Bytes::from(format!("File with mode {:#o}", mode)),
            mode: Some(mode),
            mtime: None,
        };
        
        let cid = fs.add_file(file, None).await?;
        println!("   Mode {:#o} ({}): {}", mode, symbolic, description);
        println!("   CID: {}", cid);
    }
    
    // Example 4: Compare file stats
    println!("\n📊 Example 4: Comparing small vs large file stats");
    
    // Small file
    let small_data = Bytes::from("Small file content");
    let small_cid = fs.add_bytes(small_data.clone(), None).await?;
    let small_stat = fs.stat(&small_cid, None).await?;
    
    println!("   Small file ({} bytes):", small_data.len());
    println!("   {:?}", small_stat);
    
    // Large file (will be chunked)
    use helia_unixfs::AddOptions;
    let large_size = 2_500_000; // 2.5MB
    let large_data: Vec<u8> = (0..large_size).map(|i| (i % 256) as u8).collect();
    let large_cid = fs.add_bytes(
        Bytes::from(large_data),
        Some(AddOptions {
            raw_leaves: true,
            chunk_size: Some(1_048_576),
            ..Default::default()
        })
    ).await?;
    let large_stat = fs.stat(&large_cid, None).await?;
    
    println!("\n   Large file ({} bytes):", large_size);
    println!("   {:?}", large_stat);
    println!("   Note: Large file has multiple blocks due to chunking");
    
    // Example 5: Directory with metadata
    println!("\n📁 Example 5: Directory with metadata");
    
    use helia_unixfs::DirectoryCandidate;
    
    let dir = DirectoryCandidate {
        path: "my_directory".to_string(),
        mode: Some(0o755),
        mtime: Some(UnixFSTime {
            seconds: now,
            nanoseconds: None,
        }),
    };
    
    let dir_cid = fs.add_directory(Some(dir), None).await?;
    println!("✅ Directory created with metadata");
    println!("   Mode: 0o755");
    println!("   CID: {}", dir_cid);
    
    let dir_stat = fs.stat(&dir_cid, None).await?;
    println!("✅ Directory metadata:");
    println!("   {:?}", dir_stat);
    
    println!("\n🎉 All metadata examples completed successfully!");
    
    Ok(())
}
