use crate::{CarBlock, CarHeader, Result};
use helia_interface::HeliaError;
use serde_json;
use tokio::io::{AsyncRead, AsyncReadExt};

/// Reader for CAR (Content Addressed aRchive) files
pub struct CarReader<R> {
    reader: R,
    header_read: bool,
}

impl<R> CarReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new CAR reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            header_read: false,
        }
    }

    /// Read the CAR file header
    pub async fn read_header(&mut self) -> Result<CarHeader> {
        if self.header_read {
            return Err(HeliaError::other("Header already read"));
        }

        // Read length prefix (4 bytes)
        let mut length_bytes = [0u8; 4];
        self.reader.read_exact(&mut length_bytes).await
            .map_err(|e| HeliaError::other(format!("Failed to read header length: {}", e)))?;
        
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        // Read header data
        let mut header_bytes = vec![0u8; length];
        self.reader.read_exact(&mut header_bytes).await
            .map_err(|e| HeliaError::other(format!("Failed to read header data: {}", e)))?;
        
        // Parse header
        let header: CarHeader = serde_json::from_slice(&header_bytes)
            .map_err(|e| HeliaError::other(format!("Failed to parse header: {}", e)))?;
        
        self.header_read = true;
        Ok(header)
    }

    /// Read the next block from the CAR file
    pub async fn read_block(&mut self) -> Result<Option<CarBlock>> {
        if !self.header_read {
            return Err(HeliaError::other("Must read header first"));
        }

        // Try to read length prefix
        let mut length_bytes = [0u8; 4];
        match self.reader.read_exact(&mut length_bytes).await {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // End of file reached
                return Ok(None);
            }
            Err(e) => {
                return Err(HeliaError::other(format!("Failed to read block length: {}", e)));
            }
        }
        
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        // Read block data
        let mut block_bytes = vec![0u8; length];
        self.reader.read_exact(&mut block_bytes).await
            .map_err(|e| HeliaError::other(format!("Failed to read block data: {}", e)))?;
        
        // Parse block
        let block: CarBlock = serde_json::from_slice(&block_bytes)
            .map_err(|e| HeliaError::other(format!("Failed to parse block: {}", e)))?;
        
        Ok(Some(block))
    }

    /// Read all remaining blocks
    pub async fn read_all_blocks(&mut self) -> Result<Vec<CarBlock>> {
        let mut blocks = Vec::new();
        
        while let Some(block) = self.read_block().await? {
            blocks.push(block);
        }
        
        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarHeader;
    use cid::Cid;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_car_reader_empty() {
        let data = Vec::new();
        let cursor = Cursor::new(data);
        let mut reader = CarReader::new(cursor);
        
        // Should fail to read header from empty data
        assert!(reader.read_header().await.is_err());
    }

    #[tokio::test]
    async fn test_car_reader_header_only() {
        let header = CarHeader {
            version: 1,
            roots: vec![Cid::default()],
        };
        
        let header_bytes = serde_json::to_vec(&header).unwrap();
        let length = (header_bytes.len() as u32).to_be_bytes();
        
        let mut data = Vec::new();
        data.extend_from_slice(&length);
        data.extend_from_slice(&header_bytes);
        
        let cursor = Cursor::new(data);
        let mut reader = CarReader::new(cursor);
        
        let read_header = reader.read_header().await.unwrap();
        assert_eq!(read_header.version, 1);
        assert_eq!(read_header.roots.len(), 1);
        
        // Should return None when no blocks present
        assert!(reader.read_block().await.unwrap().is_none());
    }
}