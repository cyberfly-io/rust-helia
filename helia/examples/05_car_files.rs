//! CAR file import/export example
//!
//! This example demonstrates:
//! - Creating content and exporting to CAR files
//! - Importing content from CAR files
//! - Working with multiple root blocks
//! - Round-trip CAR operations

use helia::create_helia;
use helia_car::{import_car, export_car};
use helia_unixfs::{UnixFS, UnixFSInterface};
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CAR File Operations Example ===\n");

    // Setup temporary directory for CAR files
    let temp_dir = std::env::temp_dir();
    let export_path = temp_dir.join("helia_export.car");
    let roundtrip_path = temp_dir.join("helia_roundtrip.car");

    // 1. Create content to export
    println!("1. Creating content...");
    let helia1 = Arc::new(create_helia(None).await?);
    helia1.start().await?;
    
    let fs = UnixFS::new(helia1.clone());
    
    let content1 = Bytes::from("Hello from CAR file!");
    let cid1 = fs.add_bytes(content1, None).await?;
    println!("   ✓ Created file 1: {}", cid1);
    
    let content2 = Bytes::from("Second file in the CAR archive");
    let cid2 = fs.add_bytes(content2, None).await?;
    println!("   ✓ Created file 2: {}\n", cid2);

    // 2. Export to CAR file
    println!("2. Exporting to CAR file...");
    export_car(helia1.clone(), &cid1, &export_path, None).await?;
    println!("   ✓ Exported to: {}\n", export_path.display());

    // 3. Create a new node and import
    println!("3. Creating new Helia node for import...");
    let helia2 = Arc::new(create_helia(None).await?);
    helia2.start().await?;
    println!("   ✓ New node created\n");

    println!("4. Importing from CAR file...");
    let roots = import_car(helia2.clone(), &export_path, None).await?;
    println!("   ✓ Imported {} root block(s):", roots.len());
    for root in &roots {
        println!("     - {}", root);
    }
    println!();

    // 4. Verify imported content
    println!("5. Verifying imported content...");
    let fs2 = UnixFS::new(helia2.clone());
    let imported_data = fs2.cat(&roots[0], None).await?;
    let imported_text = String::from_utf8(imported_data.to_vec())?;
    println!("   ✓ Imported content: \"{}\"\n", imported_text);

    // 5. Round-trip test: export again
    println!("6. Round-trip test: exporting again...");
    export_car(helia2.clone(), &roots[0], &roundtrip_path, None).await?;
    println!("   ✓ Exported to: {}\n", roundtrip_path.display());

    // 6. Compare file sizes
    println!("7. Comparing CAR files...");
    let original_size = std::fs::metadata(&export_path)?.len();
    let roundtrip_size = std::fs::metadata(&roundtrip_path)?.len();
    println!("   ✓ Original CAR size: {} bytes", original_size);
    println!("   ✓ Round-trip CAR size: {} bytes", roundtrip_size);
    if original_size == roundtrip_size {
        println!("   ✓ Sizes match!\n");
    } else {
        println!("   ⚠ Sizes differ (this is expected for different encodings)\n");
    }

    // 7. Export directory structure
    println!("8. Creating and exporting a directory...");
    let dir_cid = fs.add_directory(None, None).await?;
    let file_a = Bytes::from("File A content");
    let file_a_cid = fs.add_bytes(file_a, None).await?;
    let dir_cid = fs.cp(&file_a_cid, &dir_cid, "file_a.txt", None).await?;
    
    let file_b = Bytes::from("File B content");
    let file_b_cid = fs.add_bytes(file_b, None).await?;
    let dir_cid = fs.cp(&file_b_cid, &dir_cid, "file_b.txt", None).await?;
    
    let dir_car_path = temp_dir.join("helia_directory.car");
    export_car(helia1.clone(), &dir_cid, &dir_car_path, None).await?;
    println!("   ✓ Exported directory: {}", dir_cid);
    println!("   ✓ CAR file: {}\n", dir_car_path.display());

    // Cleanup
    helia1.stop().await?;
    helia2.stop().await?;

    println!("Example completed successfully!");
    println!("\nCAR files created:");
    println!("  - {}", export_path.display());
    println!("  - {}", roundtrip_path.display());
    println!("  - {}", dir_car_path.display());
    
    Ok(())
}
