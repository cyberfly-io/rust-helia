use crate::{CarBlock, CarHeader, Result};
use bytes::Bytes;
use cid::Cid;
use helia_interface::HeliaError;
use tokio::io::{AsyncRead, AsyncReadExt};
use unsigned_varint::decode;

/// Reader for CAR (Content Addressed aRchive) v1 files
///
/// CAR v1 format:
/// - Header: varint length + DAG-CBOR encoded header {version: 1, roots: [CID...]}
/// - Blocks: repeated (varint length + CID bytes + block data)
pub struct CarReader<R> {
    reader: R,
    header_read: bool,
}

impl<R> CarReader<R>
where
    R: AsyncRead + Unpin,
{
    /// Create a new CAR v1 reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            header_read: false,
        }
    }

    /// Read a varint from the reader
    async fn read_varint(&mut self) -> Result<u64> {
        let mut buf = [0u8; 10]; // Max varint size
        let mut bytes_read = 0;

        loop {
            if bytes_read >= 10 {
                return Err(HeliaError::other("Varint too large"));
            }

            // Try to read one byte
            match self
                .reader
                .read_exact(&mut buf[bytes_read..bytes_read + 1])
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    // If we hit EOF and have no bytes, this is a normal EOF
                    if e.kind() == std::io::ErrorKind::UnexpectedEof && bytes_read == 0 {
                        return Err(HeliaError::other("Failed to read varint byte: early eof"));
                    }
                    return Err(HeliaError::other(format!(
                        "Failed to read varint byte: {}",
                        e
                    )));
                }
            }

            bytes_read += 1;

            // Check if this is the last byte (MSB is 0)
            if buf[bytes_read - 1] & 0x80 == 0 {
                break;
            }
        }

        // Decode the varint
        let (value, _) = decode::u64(&buf[..bytes_read])
            .map_err(|e| HeliaError::other(format!("Failed to decode varint: {}", e)))?;

        Ok(value)
    }

    /// Read the CAR file header
    ///
    /// The header is:
    /// 1. Varint length of header
    /// 2. DAG-CBOR encoded { version: 1, roots: [CID...] }
    pub async fn read_header(&mut self) -> Result<CarHeader> {
        if self.header_read {
            return Err(HeliaError::other("Header already read"));
        }

        // Read varint length
        let length = self.read_varint().await? as usize;

        if length == 0 || length > 1024 * 1024 {
            return Err(HeliaError::other(format!(
                "Invalid header length: {}",
                length
            )));
        }

        // Read header DAG-CBOR data
        let mut header_bytes = vec![0u8; length];
        self.reader
            .read_exact(&mut header_bytes)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to read header data: {}", e)))?;

        // Parse DAG-CBOR header
        let header: CarHeader = serde_ipld_dagcbor::from_slice(&header_bytes)
            .map_err(|e| HeliaError::other(format!("Failed to parse CAR header: {}", e)))?;

        // Validate version
        if header.version != 1 {
            return Err(HeliaError::other(format!(
                "Unsupported CAR version: {}",
                header.version
            )));
        }

        self.header_read = true;
        Ok(header)
    }

    /// Read the next block from the CAR file
    ///
    /// Each block is:
    /// 1. Varint length of (CID + data)
    /// 2. CID bytes (varint CID version + multicodec + multihash)
    /// 3. Block data
    pub async fn read_block(&mut self) -> Result<Option<CarBlock>> {
        if !self.header_read {
            return Err(HeliaError::other("Must read header first"));
        }

        // Try to read varint length
        let length = match self.read_varint().await {
            Ok(len) => len as usize,
            Err(e) => {
                // Check if we hit EOF (end of file) - this is normal when no more blocks
                let err_str = e.to_string();
                if err_str.contains("early eof")
                    || err_str.contains("failed to fill whole buffer")
                    || err_str.contains("UnexpectedEof")
                {
                    return Ok(None); // Normal end of file
                }
                return Err(e);
            }
        };

        if length == 0 {
            return Ok(None);
        }

        if length > 100 * 1024 * 1024 {
            return Err(HeliaError::other(format!(
                "Block too large: {} bytes",
                length
            )));
        }

        // Read the entire section (CID + data)
        let mut section = vec![0u8; length];
        self.reader
            .read_exact(&mut section)
            .await
            .map_err(|e| HeliaError::other(format!("Failed to read block data: {}", e)))?;

        // Parse CID from the beginning of the section
        let cid = Cid::read_bytes(&section[..])
            .map_err(|e| HeliaError::other(format!("Failed to parse CID: {}", e)))?;

        // Calculate CID byte length
        let cid_bytes = cid.to_bytes();
        let cid_len = cid_bytes.len();

        if cid_len >= length {
            return Err(HeliaError::other("Invalid block: CID larger than block"));
        }

        // The rest is the block data
        let data = Bytes::from(section[cid_len..].to_vec());

        Ok(Some(CarBlock { cid, data }))
    }

    /// Read all remaining blocks
    pub async fn read_all_blocks(&mut self) -> Result<Vec<CarBlock>> {
        let mut blocks = Vec::new();

        while let Some(block) = self.read_block().await? {
            blocks.push(block);
        }

        Ok(blocks)
    }

    /// Find a specific block by CID
    ///
    /// This iterates through all blocks until it finds the matching CID
    pub async fn find_block(&mut self, target_cid: &Cid) -> Result<Option<Bytes>> {
        while let Some(block) = self.read_block().await? {
            if &block.cid == target_cid {
                return Ok(Some(block.data));
            }
        }
        Ok(None)
    }
}

// Tests have been moved to tests/car_v1_format.rs
