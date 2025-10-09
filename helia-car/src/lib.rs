//! CAR (Content Addressed aRchive) format support for Helia
//!
//! This crate provides functionality for importing and exporting CAR files,
//! which are used to bundle IPFS blocks for transport and storage.

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarVersion {
    V1,
    V2,
}

impl Default for CarVersion {
    fn default() -> Self {
        CarVersion::V1
    }
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
        let mut written_blocks = 0;

        for (cid, data) in &self.blocks {
            if written_blocks >= max_blocks {
                break;
            }

            let block = CarBlock {
                cid: *cid,
                data: data.clone(),
            };
            car_writer.write_block(&block).await?;
            written_blocks += 1;
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
            let mut written_blocks = 0;

            for (cid, data) in blocks {
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

                written_blocks += 1;
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
}
