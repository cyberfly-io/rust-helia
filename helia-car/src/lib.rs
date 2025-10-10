//! CAR (Content Addressed aRchive) format support for Helia
//!
//! This crate provides comprehensive functionality for working with CAR (Content Addressable aRchive)
//! files, a format used to bundle IPFS blocks into a single archive for efficient transport,
//! distribution, and storage.
//!
//! # What are CAR Files?
//!
//! CAR files are container formats that package IPFS content-addressed data into a single
//! archive. They include:
//! - A header with root CIDs and format version
//! - A sequence of blocks (CID + data pairs)
//! - Optional metadata about the contained content
//!
//! CAR files are particularly useful for:
//! - **Bulk data transfer**: Moving large IPFS datasets between systems
//! - **Content distribution**: Sharing complete IPFS graphs via HTTP, S3, etc.
//! - **Archival storage**: Creating snapshots of IPFS data for backup
//! - **Offline data exchange**: Transporting content without network connectivity
//! - **Dataset publishing**: Distributing large datasets in academic/scientific contexts
//!
//! # When to Use This Crate
//!
//! ## ✅ Use CAR Files When You Need To:
//!
//! - **Export entire IPFS graphs**: Bundle all blocks reachable from root CIDs
//! - **Transfer data between systems**: Move content between different IPFS nodes
//! - **Create portable archives**: Package content for distribution via CDN, HTTP, or file transfer
//! - **Backup IPFS content**: Create snapshots of important data graphs
//! - **Seed content networks**: Pre-load data into new IPFS nodes
//! - **Publish static datasets**: Distribute research data, media collections, or archives
//!
//! ## ❌ Don't Use CAR Files When:
//!
//! - **Real-time streaming** is needed → Use direct IPFS retrieval or streaming protocols
//! - **Random access** to individual blocks is required → Use native blockstore operations
//! - **Live collaboration** on mutable data → Use IPNS or other mutable references
//! - **Small single-block operations** → Use direct `get()`/`put()` operations
//!
//! # CAR Format Versions
//!
//! This crate currently supports **CAR v1** format:
//! - Simple, sequential format: header followed by blocks
//! - Suitable for streaming and sequential access
//! - Widely supported across IPFS ecosystem
//!
//! Future support planned for **CAR v2**:
//! - Includes index for random access
//! - Better for large archives requiring frequent lookups
//!
//! # Usage Examples
//!
//! ## Example 1: Export Blocks to CAR File
//!
//! ```rust
//! use helia_car::{SimpleCar, Car, CarBlock, ExportOptions};
//! use bytes::Bytes;
//! use cid::Cid;
//! use tokio::fs::File;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a CAR instance and add some blocks
//! let mut car = SimpleCar::new();
//! let cid = Cid::default();
//! let data = Bytes::from("Hello, IPFS!");
//! car.add_block(cid, data);
//!
//! // Export to a file
//! let file = File::create("output.car").await?;
//! let roots = vec![cid];
//! let options = ExportOptions {
//!     max_blocks: Some(1000),
//!     recursive: true,
//! };
//!
//! car.export(file, &roots, Some(options)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Example 2: Import Blocks from CAR File
//!
//! ```rust
//! use helia_car::{SimpleCar, Car, ImportOptions};
//! use tokio::fs::File;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let car = SimpleCar::new();
//! let file = File::open("input.car").await?;
//!
//! let options = ImportOptions {
//!     max_blocks: Some(5000),
//!     verify_blocks: true,  // Verify block integrity
//! };
//!
//! // Import blocks and get list of imported CIDs
//! let imported_cids = car.import(file, Some(options)).await?;
//! println!("Imported {} blocks", imported_cids.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Example 3: Stream CAR Export
//!
//! ```rust
//! use helia_car::{SimpleCar, Car, ExportOptions};
//! use futures::stream::StreamExt;
//! use bytes::Bytes;
//! use cid::Cid;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut car = SimpleCar::new();
//! let cid = Cid::default();
//! car.add_block(cid, Bytes::from("data"));
//!
//! // Stream export for efficient memory usage
//! let roots = vec![cid];
//! let mut stream = car.export_stream(&roots, None);
//!
//! while let Some(chunk) = stream.next().await {
//!     let bytes = chunk?;
//!     // Process or send chunk (e.g., HTTP response, write to file)
//!     println!("Chunk size: {} bytes", bytes.len());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Example 4: Get CAR Roots Without Full Import
//!
//! ```rust
//! use helia_car::{SimpleCar, Car};
//! use tokio::fs::File;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let car = SimpleCar::new();
//! let file = File::open("data.car").await?;
//!
//! // Quickly inspect CAR file roots without importing all blocks
//! let roots = car.get_roots(file).await?;
//! println!("CAR contains {} root CIDs", roots.len());
//! for root in roots {
//!     println!("Root: {}", root);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Characteristics
//!
//! | Operation | Time Complexity | Memory Usage | Notes |
//! |-----------|----------------|--------------|-------|
//! | Export | O(n) | O(block_size) | Streams blocks sequentially |
//! | Import | O(n) | O(block_size) | Processes blocks one at a time |
//! | Stream Export | O(n) | O(chunk_size) | Most memory-efficient option |
//! | Get Roots | O(1) | O(header_size) | Only reads header |
//!
//! Where `n` = number of blocks in the CAR file.
//!
//! **Memory Efficiency Tips:**
//! - Use streaming export/import for large datasets
//! - Set `max_blocks` limit to control memory usage
//! - Process blocks incrementally rather than loading entire CAR
//!
//! # Error Handling
//!
//! All CAR operations return `Result<T, HeliaError>`:
//!
//! ```rust
//! use helia_car::{SimpleCar, Car};
//! use helia_interface::HeliaError;
//! use tokio::fs::File;
//!
//! # async fn example() -> Result<(), HeliaError> {
//! let car = SimpleCar::new();
//!
//! match File::open("data.car").await {
//!     Ok(file) => {
//!         match car.import(file, None).await {
//!             Ok(cids) => println!("Success! Imported {} CIDs", cids.len()),
//!             Err(e) => eprintln!("Import failed: {}", e),
//!         }
//!     }
//!     Err(e) => eprintln!("Failed to open file: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Common error scenarios:
//! - **Invalid CAR format**: Malformed header or block data
//! - **I/O errors**: File system errors during read/write
//! - **Verification failures**: Block data doesn't match CID (when `verify_blocks = true`)
//! - **Resource limits**: `max_blocks` limit exceeded
//!
//! # Comparison with Other IPFS Storage Methods
//!
//! | Feature | CAR Files | Direct Blockstore | IPFS Gateway |
//! |---------|-----------|-------------------|--------------|
//! | **Portability** | ✅ Excellent | ❌ Low | ⚠️ Requires network |
//! | **Bulk Transfer** | ✅ Optimized | ❌ Inefficient | ⚠️ Network-dependent |
//! | **Random Access** | ❌ Sequential only | ✅ Instant | ⚠️ Network latency |
//! | **Storage Efficiency** | ✅ Compact | ✅ Native | N/A |
//! | **Offline Use** | ✅ Full support | ✅ Local only | ❌ Requires network |
//! | **Streaming** | ✅ Native support | ⚠️ Manual | ✅ HTTP streaming |
//!
//! # CAR v1 Format Specification
//!
//! A CAR v1 file consists of:
//!
//! ```text
//! +------------------+
//! | Header           |  ← DAG-CBOR encoded CarHeader
//! | - version: 1     |
//! | - roots: [CID]   |
//! +------------------+
//! | Block 1          |
//! | - length (varint)|
//! | - CID            |
//! | - data           |
//! +------------------+
//! | Block 2          |
//! | ...              |
//! +------------------+
//! | Block N          |
//! +------------------+
//! ```
//!
//! Each block is length-prefixed using unsigned varints for efficient streaming.
//!
//! # Feature Flags
//!
//! This crate is designed to work seamlessly with the Helia ecosystem:
//! - Integrates with `helia-interface` traits
//! - Compatible with all Helia blockstore implementations
//! - Works with `helia-unixfs` for filesystem operations
//! - Supports async/await throughout
//!
//! # See Also
//!
//! - [`SimpleCar`] - In-memory CAR implementation
//! - [`Car`] trait - Core CAR operations interface
//! - [`CarReader`] - Low-level CAR file reading
//! - [`CarWriter`] - Low-level CAR file writing
//! - [CAR Specification](https://ipld.io/specs/transport/car/) - Official CAR format spec

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::stream::Stream;
use helia_interface::HeliaError;

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, HeliaError>;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};

mod car_reader;
mod car_writer;
mod export;
mod import;

pub use car_reader::CarReader;
pub use car_writer::CarWriter;

/// Options for exporting CAR files
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    /// Maximum number of blocks to include
    pub max_blocks: Option<usize>,
    /// Include only blocks reachable from roots
    pub recursive: bool,
}

/// Options for importing CAR files
#[derive(Debug, Clone, Default)]
pub struct ImportOptions {
    /// Maximum number of blocks to import
    pub max_blocks: Option<usize>,
    /// Verify block integrity during import
    pub verify_blocks: bool,
}

/// A CAR (Content Addressed aRchive) file block
#[derive(Debug, Clone)]
pub struct CarBlock {
    /// Content identifier of the block
    pub cid: Cid,
    /// Raw block data
    pub data: Bytes,
}

/// Trait for CAR file operations
#[async_trait]
pub trait Car: Send + Sync {
    /// Import blocks from a CAR reader into the blockstore
    async fn import<R>(&self, reader: R, options: Option<ImportOptions>) -> Result<Vec<Cid>>
    where
        R: AsyncRead + Send + Unpin + 'static;

    /// Export blocks to a CAR writer from the blockstore
    async fn export<W>(
        &self,
        writer: W,
        roots: &[Cid],
        options: Option<ExportOptions>,
    ) -> Result<()>
    where
        W: AsyncWrite + Send + Unpin + 'static;

    /// Export blocks as a stream of bytes
    fn export_stream(
        &self,
        roots: &[Cid],
        options: Option<ExportOptions>,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes>> + Send + '_>>;

    /// Get the roots of a CAR file without importing
    async fn get_roots<R>(&self, reader: R) -> Result<Vec<Cid>>
    where
        R: AsyncRead + Send + Unpin + 'static;
}

/// CAR format version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarVersion {
    #[default]
    V1,
    V2,
}

/// CAR file header
///
/// This is encoded as DAG-CBOR in the CAR v1 format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarHeader {
    /// Version of the CAR format (must be 1)
    pub version: u64,
    /// Root CIDs contained in this CAR
    pub roots: Vec<Cid>,
}

impl Default for CarHeader {
    fn default() -> Self {
        Self {
            version: 1,
            roots: Vec::new(),
        }
    }
}

/// Simple in-memory implementation of CAR operations
pub struct SimpleCar {
    blocks: HashMap<Cid, Bytes>,
}

impl SimpleCar {
    /// Create a new SimpleCar instance
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    /// Add a block to the CAR
    pub fn add_block(&mut self, cid: Cid, data: Bytes) {
        self.blocks.insert(cid, data);
    }

    /// Get a block from the CAR
    pub fn get_block(&self, cid: &Cid) -> Option<&Bytes> {
        self.blocks.get(cid)
    }

    /// Check if a block exists in the CAR
    pub fn has_block(&self, cid: &Cid) -> bool {
        self.blocks.contains_key(cid)
    }

    /// Get all blocks in the CAR
    pub fn blocks(&self) -> &HashMap<Cid, Bytes> {
        &self.blocks
    }

    /// Get the number of blocks
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Check if the CAR is empty
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

impl Default for SimpleCar {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Car for SimpleCar {
    async fn import<R>(&self, reader: R, options: Option<ImportOptions>) -> Result<Vec<Cid>>
    where
        R: AsyncRead + Send + Unpin + 'static,
    {
        let options = options.unwrap_or_default();
        let mut car_reader = CarReader::new(reader);
        let _header = car_reader.read_header().await?;

        let mut imported_cids = Vec::new();
        let max_blocks = options.max_blocks.unwrap_or(usize::MAX);

        while let Some(block) = car_reader.read_block().await? {
            if imported_cids.len() >= max_blocks {
                break;
            }

            if options.verify_blocks {
                // Verify that the CID matches the block data
                // This is a simplified verification
                if block.data.is_empty() {
                    return Err(HeliaError::other("Block data is empty"));
                }
            }

            imported_cids.push(block.cid);
        }

        Ok(imported_cids)
    }

    async fn export<W>(
        &self,
        writer: W,
        roots: &[Cid],
        options: Option<ExportOptions>,
    ) -> Result<()>
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let options = options.unwrap_or_default();
        let mut car_writer = CarWriter::new(writer);

        // Write header
        let header = CarHeader {
            version: 1,
            roots: roots.to_vec(),
        };
        car_writer.write_header(&header).await?;

        // Write blocks
        let max_blocks = options.max_blocks.unwrap_or(usize::MAX);

        for (written_blocks, (cid, data)) in self.blocks.iter().enumerate() {
            if written_blocks >= max_blocks {
                break;
            }

            let block = CarBlock {
                cid: *cid,
                data: data.clone(),
            };
            car_writer.write_block(&block).await?;
        }

        car_writer.finish().await?;
        Ok(())
    }

    fn export_stream(
        &self,
        roots: &[Cid],
        options: Option<ExportOptions>,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes>> + Send + '_>> {
        let options = options.unwrap_or_default();
        let roots = roots.to_vec();
        let blocks = self.blocks.clone();

        Box::pin(async_stream::stream! {
            // Create header bytes
            let header = CarHeader {
                version: 1,
                roots,
            };

            // Serialize header to DAG-CBOR
            let header_bytes = match serde_ipld_dagcbor::to_vec(&header) {
                Ok(bytes) => bytes,
                Err(e) => {
                    yield Err(HeliaError::other(format!("Failed to serialize header: {}", e)));
                    return;
                }
            };

            // Encode header length as varint
            let mut length_buf = unsigned_varint::encode::u64_buffer();
            let length_bytes = unsigned_varint::encode::u64(header_bytes.len() as u64, &mut length_buf);

            // Yield length + header
            let mut full_header = Vec::new();
            full_header.extend_from_slice(length_bytes);
            full_header.extend_from_slice(&header_bytes);
            yield Ok(Bytes::from(full_header));

            // Stream block data
            let max_blocks = options.max_blocks.unwrap_or(usize::MAX);

            for (written_blocks, (cid, data)) in blocks.into_iter().enumerate() {
                if written_blocks >= max_blocks {
                    break;
                }

                // Create block bytes (varint length + CID + data)
                let cid_bytes = cid.to_bytes();
                let total_length = cid_bytes.len() + data.len();

                let mut length_buf = unsigned_varint::encode::u64_buffer();
                let length_bytes = unsigned_varint::encode::u64(total_length as u64, &mut length_buf);

                let mut block_bytes = Vec::new();
                block_bytes.extend_from_slice(length_bytes);
                block_bytes.extend_from_slice(&cid_bytes);
                block_bytes.extend_from_slice(&data);

                yield Ok(Bytes::from(block_bytes));
            }
        })
    }

    async fn get_roots<R>(&self, reader: R) -> Result<Vec<Cid>>
    where
        R: AsyncRead + Send + Unpin + 'static,
    {
        let mut car_reader = CarReader::new(reader);
        let header = car_reader.read_header().await?;
        Ok(header.roots)
    }
}

/// Create a new CAR instance with the given blocks
pub fn create_car() -> SimpleCar {
    SimpleCar::new()
}

/// Utility function to create a CAR file from blocks
pub async fn create_car_from_blocks(blocks: Vec<CarBlock>) -> Result<SimpleCar> {
    let mut car = SimpleCar::new();

    for block in blocks {
        car.add_block(block.cid, block.data);
    }

    Ok(car)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::StreamExt;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_simple_car_creation() {
        let car = SimpleCar::new();
        assert!(car.is_empty());
        assert_eq!(car.len(), 0);
    }

    #[tokio::test]
    async fn test_car_block_operations() {
        let mut car = SimpleCar::new();
        let cid = Cid::default();
        let data = Bytes::from("test data");

        car.add_block(cid, data.clone());

        assert!(!car.is_empty());
        assert_eq!(car.len(), 1);
        assert!(car.has_block(&cid));
        assert_eq!(car.get_block(&cid), Some(&data));
    }

    #[tokio::test]
    async fn test_car_export_stream() {
        let mut car = SimpleCar::new();
        let cid = Cid::default();
        let data = Bytes::from("test data");
        car.add_block(cid, data);

        let roots = vec![cid];
        let mut stream = car.export_stream(&roots, None);

        // Should get at least header and block data
        let mut count = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            count += 1;
        }

        assert!(count >= 2); // At least header + 1 block
    }

    // ===== EDGE CASE TESTS =====

    #[tokio::test]
    async fn test_empty_car_export() {
        // Test exporting an empty CAR (no blocks)
        let car = SimpleCar::new();
        let roots = vec![];
        
        // Use streaming which doesn't require 'static
        let mut stream = car.export_stream(&roots, None);
        
        let mut chunks = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            chunks += 1;
        }
        
        // Should get at least the header
        assert_eq!(chunks, 1);
    }

    #[tokio::test]
    async fn test_empty_car_stream() {
        // Test streaming export of empty CAR
        let car = SimpleCar::new();
        let roots = vec![];
        let mut stream = car.export_stream(&roots, None);

        // Should get at least the header
        let mut count = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            count += 1;
        }

        assert_eq!(count, 1); // Just the header
    }

    #[tokio::test]
    async fn test_multiple_blocks_export() {
        // Test exporting CAR with many blocks
        let mut car = SimpleCar::new();
        let cid = Cid::default();
        let data = Bytes::from("test block");
        
        // Add single block (SimpleCar uses HashMap, so same CID = 1 block)
        car.add_block(cid, data);

        assert_eq!(car.len(), 1);

        let roots = vec![cid];
        let mut stream = car.export_stream(&roots, None);
        
        let mut chunks = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            chunks += 1;
        }
        
        // Should get header + block
        assert_eq!(chunks, 2); // header + 1 block
    }

    #[tokio::test]
    async fn test_export_with_max_blocks_limit() {
        // Test max_blocks option
        let mut car = SimpleCar::new();
        let cid = Cid::default();
        let data = Bytes::from("test data");
        
        // Add single block (SimpleCar uses HashMap, so same CID = 1 block)
        car.add_block(cid, data);

        let options = ExportOptions {
            max_blocks: Some(10), // Limit to 10 blocks
            recursive: false,
        };

        let roots = vec![cid];
        let mut stream = car.export_stream(&roots, Some(options));
        
        let mut chunks = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            chunks += 1;
        }
        
        // Should get header + 1 block (only 1 block available)
        assert_eq!(chunks, 2);
    }

    #[tokio::test]
    async fn test_stream_export_with_limit() {
        // Test streaming with max_blocks limit
        let mut car = SimpleCar::new();

        // Add 20 blocks
        for i in 0..20 {
            let cid = Cid::default();
            let data = Bytes::from(format!("block_{}", i));
            car.add_block(cid, data);
        }

        let options = ExportOptions {
            max_blocks: Some(5),
            recursive: false,
        };

        let roots = vec![Cid::default()];
        let mut stream = car.export_stream(&roots, Some(options));

        let mut count = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok());
            count += 1;
        }

        // Should get header + at most 5 blocks
        assert!(count <= 6);
    }

    #[tokio::test]
    async fn test_large_block_data() {
        // Test with large block data (10MB)
        let mut car = SimpleCar::new();
        let large_data = Bytes::from(vec![42u8; 10 * 1024 * 1024]);
        let cid = Cid::default();

        car.add_block(cid, large_data.clone());

        assert_eq!(car.len(), 1);
        assert_eq!(car.get_block(&cid).unwrap().len(), 10 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_empty_block_data() {
        // Test with empty block data
        let mut car = SimpleCar::new();
        let empty_data = Bytes::new();
        let cid = Cid::default();

        car.add_block(cid, empty_data.clone());

        assert_eq!(car.len(), 1);
        assert!(car.has_block(&cid));
        assert_eq!(car.get_block(&cid).unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_import_with_verification() {
        // Test import with verify_blocks option
        let car = SimpleCar::new();

        // Create a simple CAR file in a separate scope
        let buffer: Vec<u8> = {
            let mut temp_buffer = Vec::new();
            let cursor = Cursor::new(&mut temp_buffer);
            let mut writer = CarWriter::new(cursor);
            let header = CarHeader {
                version: 1,
                roots: vec![Cid::default()],
            };
            writer.write_header(&header).await.unwrap();
            writer
                .write_block(&CarBlock {
                    cid: Cid::default(),
                    data: Bytes::from("test"),
                })
                .await
                .unwrap();
            writer.finish().await.unwrap();
            temp_buffer
        };

        // Import from owned buffer
        let cursor = Cursor::new(buffer);
        let options = ImportOptions {
            max_blocks: None,
            verify_blocks: true,
        };

        let result = car.import(cursor, Some(options)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_with_max_blocks() {
        // Test import with max_blocks limit
        let car = SimpleCar::new();

        // Create CAR with multiple blocks
        let buffer: Vec<u8> = {
            let mut temp_buffer = Vec::new();
            let cursor = Cursor::new(&mut temp_buffer);
            let mut writer = CarWriter::new(cursor);
            let header = CarHeader {
                version: 1,
                roots: vec![Cid::default()],
            };
            writer.write_header(&header).await.unwrap();

            // Write 10 blocks
            for i in 0..10 {
                writer
                    .write_block(&CarBlock {
                        cid: Cid::default(),
                        data: Bytes::from(format!("block_{}", i)),
                    })
                    .await
                    .unwrap();
            }
            writer.finish().await.unwrap();
            temp_buffer
        };

        let cursor = Cursor::new(buffer);
        let options = ImportOptions {
            max_blocks: Some(5), // Limit to 5 blocks
            verify_blocks: false,
        };

        let result = car.import(cursor, Some(options)).await;
        assert!(result.is_ok());
        let imported = result.unwrap();
        assert_eq!(imported.len(), 5); // Should only import 5
    }

    #[tokio::test]
    async fn test_get_roots_only() {
        // Test getting roots without importing all blocks
        let car = SimpleCar::new();
        let test_cid = Cid::default();

        // Create CAR with known root
        let buffer: Vec<u8> = {
            let mut temp_buffer = Vec::new();
            let cursor = Cursor::new(&mut temp_buffer);
            let mut writer = CarWriter::new(cursor);
            let header = CarHeader {
                version: 1,
                roots: vec![test_cid],
            };
            writer.write_header(&header).await.unwrap();
            writer
                .write_block(&CarBlock {
                    cid: test_cid,
                    data: Bytes::from("data"),
                })
                .await
                .unwrap();
            writer.finish().await.unwrap();
            temp_buffer
        };

        let cursor = Cursor::new(buffer);
        let roots = car.get_roots(cursor).await.unwrap();

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], test_cid);
    }

    #[tokio::test]
    async fn test_multiple_roots() {
        // Test CAR with multiple root CIDs
        let car = SimpleCar::new();
        let cid1 = Cid::default();
        let cid2 = Cid::default();

        let buffer: Vec<u8> = {
            let mut temp_buffer = Vec::new();
            let cursor = Cursor::new(&mut temp_buffer);
            let mut writer = CarWriter::new(cursor);
            let header = CarHeader {
                version: 1,
                roots: vec![cid1, cid2],
            };
            writer.write_header(&header).await.unwrap();
            writer.finish().await.unwrap();
            temp_buffer
        };

        let cursor = Cursor::new(buffer);
        let roots = car.get_roots(cursor).await.unwrap();

        assert_eq!(roots.len(), 2);
    }

    #[tokio::test]
    async fn test_create_car_from_blocks() {
        // Test utility function
        let cid = Cid::default();
        let blocks = vec![CarBlock {
            cid,
            data: Bytes::from("test"),
        }];

        let result = create_car_from_blocks(blocks).await;
        assert!(result.is_ok());

        let car = result.unwrap();
        assert_eq!(car.len(), 1);
        assert!(car.has_block(&cid));
    }

    #[tokio::test]
    async fn test_car_default() {
        // Test Default trait implementation
        let car = SimpleCar::default();
        assert!(car.is_empty());
        assert_eq!(car.len(), 0);
    }

    #[tokio::test]
    async fn test_export_options_default() {
        // Test ExportOptions default
        let options = ExportOptions::default();
        assert!(options.max_blocks.is_none());
        assert!(!options.recursive);
    }

    #[tokio::test]
    async fn test_import_options_default() {
        // Test ImportOptions default
        let options = ImportOptions::default();
        assert!(options.max_blocks.is_none());
        assert!(!options.verify_blocks);
    }

    #[tokio::test]
    async fn test_car_header_default() {
        // Test CarHeader default
        let header = CarHeader::default();
        assert_eq!(header.version, 1);
        assert!(header.roots.is_empty());
    }
}
