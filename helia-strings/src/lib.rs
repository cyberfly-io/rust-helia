//! `helia-strings` makes working with strings in Helia simple & straightforward.
//!
//! This crate provides a simple and intuitive interface for adding and retrieving strings
//! from your Helia node. Strings are stored using the raw codec.
//!
//! # Example
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_strings::{strings, StringsInterface};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = std::sync::Arc::new(create_helia_default().await?);
//!     let str_interface = strings(helia);
//!     
//!     let cid = str_interface.add("hello world", None).await?;
//!     let retrieved = str_interface.get(cid, None).await?;
//!     
//!     println!("{}", retrieved); // "hello world"
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use helia_interface::Helia;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Error types for string operations
#[derive(Debug, thiserror::Error)]
pub enum StringsError {
    #[error("Invalid codec error: {0}")]
    InvalidCodec(String),
    #[error("Blockstore error: {0}")]
    Blockstore(String),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Options for adding strings
#[derive(Default)]
pub struct AddOptions {
    // Future options can be added here
}

/// Options for getting strings
#[derive(Default)]
pub struct GetOptions {
    // Future options can be added here
}

/// The Strings interface provides a simple and intuitive way to add/get strings
/// with your Helia node and is a great place to start learning about IPFS.
#[async_trait]
pub trait StringsInterface: Send + Sync {
    /// Add a string to your Helia node and get a CID that refers to the block the
    /// string has been stored as.
    async fn add(&self, string: &str, options: Option<AddOptions>) -> Result<Cid, StringsError>;

    /// Get a string from your Helia node, either previously added to it or to
    /// another node on the network.
    async fn get(&self, cid: Cid, options: Option<GetOptions>) -> Result<String, StringsError>;
}

/// Default implementation of the Strings interface
pub struct DefaultStrings {
    helia: Arc<dyn Helia>,
}

impl DefaultStrings {
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self { helia }
    }
}

#[async_trait]
impl StringsInterface for DefaultStrings {
    async fn add(&self, string: &str, options: Option<AddOptions>) -> Result<Cid, StringsError> {
        let _options = options.unwrap_or_default();
        let data = string.as_bytes();

        // Use SHA-256 hasher (matching JavaScript implementation)
        let mut sha256 = Sha256::new();
        sha256.update(data);
        let hash_bytes = sha256.finalize();

        // Create multihash with SHA-256 (0x12)
        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes)
            .map_err(|e| StringsError::Blockstore(format!("Multihash error: {}", e)))?;

        // Create CID v1 with raw codec (0x55)
        let cid = Cid::new_v1(0x55, mh);

        // Store the raw bytes as Bytes
        let bytes = Bytes::from(data.to_vec());
        self.helia
            .blockstore()
            .put(&cid, bytes, None)
            .await
            .map_err(|e| StringsError::Blockstore(format!("Failed to store block: {}", e)))?;

        Ok(cid)
    }

    async fn get(&self, cid: Cid, _options: Option<GetOptions>) -> Result<String, StringsError> {
        // Check codec - allow raw (0x55), JSON (0x0129), and DAG-JSON (0x0200)
        // This matches the JavaScript implementation behavior
        match cid.codec() {
            0x55 | 0x0129 | 0x0200 => {
                // Valid codecs
            }
            _ => {
                return Err(StringsError::InvalidCodec(
                    "The passed CID had an incorrect codec, it may correspond to a block data that cannot be interpreted as a string".to_string()
                ));
            }
        }

        let data = self
            .helia
            .blockstore()
            .get(&cid, None)
            .await
            .map_err(|e| StringsError::Blockstore(format!("Failed to get block: {}", e)))?;

        String::from_utf8(data.to_vec()).map_err(StringsError::Utf8)
    }
}

/// Create a StringsInterface instance for use with Helia
///
/// # Example
///
/// ```no_run
/// use rust_helia::create_helia_default;
/// use helia_strings::{strings, StringsInterface};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let helia = std::sync::Arc::new(create_helia_default().await?);
/// let str_interface = strings(helia);
/// let cid = str_interface.add("hello world", None).await?;
/// # Ok(())
/// # }
/// ```
pub fn strings(helia: Arc<dyn Helia>) -> impl StringsInterface {
    DefaultStrings::new(helia)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_helia::create_helia_default;
    use std::sync::Arc;

    async fn create_test_helia() -> Arc<dyn Helia> {
        Arc::new(
            create_helia_default()
                .await
                .expect("Failed to create Helia instance"),
        )
    }

    #[tokio::test]
    async fn test_add_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let cid = str_interface.add("hello world", None).await.unwrap();

        // Verify CID format
        assert_eq!(cid.codec(), 0x55); // raw codec
    }

    #[tokio::test]
    async fn test_get_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let original = "hello world";
        let cid = str_interface.add(original, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, original);
    }

    #[tokio::test]
    async fn test_add_get_empty_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let original = "";
        let cid = str_interface.add(original, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, original);
    }

    #[tokio::test]
    async fn test_add_get_unicode_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let original = "Hello, 世界! 🌍";
        let cid = str_interface.add(original, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, original);
    }

    #[tokio::test]
    async fn test_add_get_multiline_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let original = "Line 1\nLine 2\nLine 3";
        let cid = str_interface.add(original, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, original);
    }

    #[tokio::test]
    async fn test_deterministic_cids() {
        let helia1 = create_test_helia().await;
        let helia2 = create_test_helia().await;
        let str1 = strings(helia1);
        let str2 = strings(helia2);

        let content = "deterministic content";
        let cid1 = str1.add(content, None).await.unwrap();
        let cid2 = str2.add(content, None).await.unwrap();

        assert_eq!(cid1, cid2, "Same content should produce same CID");
    }

    #[tokio::test]
    async fn test_invalid_codec_error() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        // Create a CID with an invalid codec for strings (using DAG-CBOR codec)
        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &[0u8; 32]).unwrap();
        let invalid_cid = Cid::new_v1(0x71, mh);

        let result = str_interface.get(invalid_cid, None).await;
        assert!(result.is_err());

        if let Err(StringsError::InvalidCodec(msg)) = result {
            assert!(msg.contains("incorrect codec"));
        } else {
            panic!("Expected InvalidCodec error");
        }
    }

    #[tokio::test]
    async fn test_get_nonexistent_cid() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        // Create a valid raw CID that doesn't exist in the blockstore
        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &[1u8; 32]).unwrap();
        let nonexistent_cid = Cid::new_v1(0x55, mh);

        let result = str_interface.get(nonexistent_cid, None).await;
        assert!(result.is_err());

        if let Err(StringsError::Blockstore(_)) = result {
            // Expected error type
        } else {
            panic!("Expected Blockstore error");
        }
    }
}
