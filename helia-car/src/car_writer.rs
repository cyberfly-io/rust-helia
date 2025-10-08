use crate::{CarBlock, CarHeader, Result};
use helia_interface::HeliaError;
use serde_json;
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// Writer for CAR (Content Addressed aRchive) files
pub struct CarWriter<W> {
    writer: W,
    header_written: bool,
}

impl<W> CarWriter<W>
where
    W: AsyncWrite + Unpin,
{
    /// Create a new CAR writer
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            header_written: false,
        }
    }

    /// Write the CAR file header
    pub async fn write_header(&mut self, header: &CarHeader) -> Result<()> {
        if self.header_written {
            return Err(HeliaError::other("Header already written"));
        }

        // Serialize header
        let header_bytes = serde_json::to_vec(header)
            .map_err(|e| HeliaError::other(format!("Failed to serialize header: {}", e)))?;
        
        // Write length prefix (4 bytes, big endian)
        let length = header_bytes.len() as u32;
        self.writer.write_all(&length.to_be_bytes()).await
            .map_err(|e| HeliaError::other(format!("Failed to write header length: {}", e)))?;
        
        // Write header data
        self.writer.write_all(&header_bytes).await
            .map_err(|e| HeliaError::other(format!("Failed to write header data: {}", e)))?;
        
        self.header_written = true;
        Ok(())
    }

    /// Write a block to the CAR file
    pub async fn write_block(&mut self, block: &CarBlock) -> Result<()> {
        if !self.header_written {
            return Err(HeliaError::other("Must write header first"));
        }

        // Serialize block
        let block_bytes = serde_json::to_vec(block)
            .map_err(|e| HeliaError::other(format!("Failed to serialize block: {}", e)))?;
        
        // Write length prefix (4 bytes, big endian)
        let length = block_bytes.len() as u32;
        self.writer.write_all(&length.to_be_bytes()).await
            .map_err(|e| HeliaError::other(format!("Failed to write block length: {}", e)))?;
        
        // Write block data
        self.writer.write_all(&block_bytes).await
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

    /// Finish writing and flush the writer
    pub async fn finish(mut self) -> Result<()> {
        self.writer.flush().await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarHeader;
    use bytes::Bytes;
    use cid::Cid;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_car_writer_header() {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = CarWriter::new(cursor);
        
        let header = CarHeader {
            version: 1,
            roots: vec![Cid::default()],
        };
        
        writer.write_header(&header).await.unwrap();
        writer.finish().await.unwrap();
        
        // Verify data was written
        assert!(!buffer.is_empty());
        
        // First 4 bytes should be length
        assert_eq!(buffer.len() >= 4, true);
    }

    #[tokio::test]
    async fn test_car_writer_block_without_header() {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = CarWriter::new(cursor);
        
        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::from("test data"),
        };
        
        // Should fail without header
        assert!(writer.write_block(&block).await.is_err());
    }

    #[tokio::test]
    async fn test_car_writer_complete() {
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = CarWriter::new(cursor);
        
        let header = CarHeader {
            version: 1,
            roots: vec![Cid::default()],
        };
        
        let block = CarBlock {
            cid: Cid::default(),
            data: Bytes::from("test data"),
        };
        
        writer.write_header(&header).await.unwrap();
        writer.write_block(&block).await.unwrap();
        writer.finish().await.unwrap();
        
        // Verify data was written
        assert!(buffer.len() > 8); // At least header length + block length
    }
}