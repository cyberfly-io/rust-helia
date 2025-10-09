use crate::{CarBlock, CarHeader, Result};
use cid::Cid;
use helia_interface::HeliaError;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use unsigned_varint::encode as varint_encode;

/// Writer for CAR (Content Addressed aRchive) v1 files
///
/// CAR v1 format:
/// - Header: varint length + DAG-CBOR encoded header {version: 1, roots: [CID...]}
/// - Blocks: repeated (varint length + CID bytes + block data)
pub struct CarWriter<W> {
    writer: W,
    header_written: bool,
}

impl<W> CarWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Create a new CAR v1 writer
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            header_written: false,
        }
    }

    /// Write the CAR file header
    ///
    /// The header is:
    /// 1. Varint length of header
    /// 2. DAG-CBOR encoded { version: 1, roots: [CID...] }
    pub async fn write_header(&mut self, header: &CarHeader) -> Result<()> {
        if self.header_written {
            return Err(HeliaError::other("Header already written"));
        }

        // Validate version
        if header.version != 1 {
            return Err(HeliaError::other(format!(
                "Unsupported CAR version: {}",
                header.version
            )));
        }

        // Serialize header to DAG-CBOR
        let header_bytes = serde_ipld_dagcbor::to_vec(header)
            .map_err(|e| HeliaError::other(format!("Failed to serialize header: {}", e)))?;

        // Encode length as varint
        let mut length_buf = varint_encode::u64_buffer();
        let length_bytes = varint_encode::u64(header_bytes.len() as u64, &mut length_buf);

        // Write varint length
        self.writer
            .write_all(length_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write header length: {}", e)))?;

        // Write DAG-CBOR header data
        self.writer
            .write_all(&header_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write header data: {}", e)))?;

        self.header_written = true;
        Ok(())
    }

    /// Write a block to the CAR file
    ///
    /// Each block is:
    /// 1. Varint length of (CID + data)
    /// 2. CID bytes (varint CID version + multicodec + multihash)
    /// 3. Block data
    pub async fn write_block(&mut self, block: &CarBlock) -> Result<()> {
        if !self.header_written {
            return Err(HeliaError::other("Must write header first"));
        }

        // Get CID bytes
        let cid_bytes = block.cid.to_bytes();

        // Calculate total length (CID + data)
        let total_length = cid_bytes.len() + block.data.len();

        // Encode length as varint
        let mut length_buf = varint_encode::u64_buffer();
        let length_bytes = varint_encode::u64(total_length as u64, &mut length_buf);

        // Write varint length
        self.writer
            .write_all(length_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write block length: {}", e)))?;

        // Write CID bytes
        self.writer
            .write_all(&cid_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write CID: {}", e)))?;

        // Write block data
        self.writer
            .write_all(&block.data)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write block data: {}", e)))?;

        Ok(())
    }

    /// Write multiple blocks to the CAR file
    pub async fn write_blocks(&mut self, blocks: &[CarBlock]) -> Result<()> {
        for block in blocks {
            self.write_block(block).await?;
        }
        Ok(())
    }

    /// Write a block with raw CID and data
    ///
    /// Helper method to write a block without wrapping in CarBlock first
    pub async fn write_raw_block(&mut self, cid: &Cid, data: &[u8]) -> Result<()> {
        if !self.header_written {
            return Err(HeliaError::other("Must write header first"));
        }

        let cid_bytes = cid.to_bytes();
        let total_length = cid_bytes.len() + data.len();

        let mut length_buf = varint_encode::u64_buffer();
        let length_bytes = varint_encode::u64(total_length as u64, &mut length_buf);

        self.writer
            .write_all(length_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write block length: {}", e)))?;

        self.writer
            .write_all(&cid_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write CID: {}", e)))?;

        self.writer
            .write_all(data)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to write block data: {}", e)))?;

        Ok(())
    }

    /// Finish writing and flush the writer
    pub async fn finish(mut self) -> Result<()> {
        self.writer
            .flush()
            .await
            .map_err(|e| HeliaError::other(format!("Failed to flush writer: {}", e)))?;
        Ok(())
    }

    /// Get a reference to the underlying writer
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Get a mutable reference to the underlying writer
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }
}

// Tests have been moved to tests/car_v1_format.rs
