//! `helia-strings` makes working with strings in Helia simple & straightforward.
//!
//! This crate provides a simple and intuitive interface for adding and retrieving strings
//! from your Helia node. Strings are stored using the raw codec (0x55) and content-addressed
//! using SHA-256, making them verifiable and immutable.
//!
//! # Overview
//!
//! The Strings API is the simplest way to work with IPFS/Helia, providing an easy-to-use
//! abstraction over content-addressed storage. It's perfect for:
//!
//! - Learning IPFS concepts (content addressing, CIDs, immutability)
//! - Storing simple text data (notes, messages, configuration)
//! - Building text-based applications
//! - Prototyping IPFS integrations
//!
//! Under the hood, strings are:
//! 1. Encoded as UTF-8 bytes
//! 2. Hashed with SHA-256
//! 3. Wrapped in a CID (v1, raw codec 0x55)
//! 4. Stored in the blockstore
//!
//! # Quick Start
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_strings::{strings, StringsInterface};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Helia node
//!     let helia = std::sync::Arc::new(create_helia_default().await?);
//!     
//!     // Create strings interface
//!     let str_interface = strings(helia);
//!     
//!     // Add a string and get its CID
//!     let cid = str_interface.add("hello world", None).await?;
//!     println!("Stored with CID: {}", cid);
//!     
//!     // Retrieve the string using its CID
//!     let retrieved = str_interface.get(cid, None).await?;
//!     println!("Retrieved: {}", retrieved); // "hello world"
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Core Concepts
//!
//! ## Content Addressing
//!
//! Strings are content-addressed, meaning the CID is derived from the content itself:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = strings(helia);
//!
//! // Same content always produces same CID
//! let cid1 = str_interface.add("hello", None).await?;
//! let cid2 = str_interface.add("hello", None).await?;
//! assert_eq!(cid1, cid2); // CIDs are identical
//!
//! // Different content produces different CID
//! let cid3 = str_interface.add("world", None).await?;
//! assert_ne!(cid1, cid3); // CIDs are different
//! # Ok(())
//! # }
//! ```
//!
//! ## Immutability
//!
//! Once stored, strings cannot be modified. To "update" a string, you add a new version:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = strings(helia);
//!
//! // Store original version
//! let v1 = str_interface.add("Version 1", None).await?;
//!
//! // "Update" by storing new version (original is unchanged)
//! let v2 = str_interface.add("Version 2", None).await?;
//!
//! // Both versions remain accessible
//! assert_eq!(str_interface.get(v1, None).await?, "Version 1");
//! assert_eq!(str_interface.get(v2, None).await?, "Version 2");
//! # Ok(())
//! # }
//! ```
//!
//! ## Codec Compatibility
//!
//! The Strings API accepts CIDs with these codecs:
//!
//! - **0x55 (raw)**: Primary codec for strings
//! - **0x0129 (json)**: JSON-encoded strings
//! - **0x0200 (dag-json)**: DAG-JSON strings
//!
//! This allows interoperability with other IPFS tools and libraries.
//!
//! # Usage Patterns
//!
//! ## Working with Unicode
//!
//! Full Unicode support including emojis and multi-byte characters:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = strings(helia);
//!
//! // Unicode strings work seamlessly
//! let text = "Hello, 世界! 🌍";
//! let cid = str_interface.add(text, None).await?;
//! let retrieved = str_interface.get(cid, None).await?;
//! assert_eq!(retrieved, text);
//! # Ok(())
//! # }
//! ```
//!
//! ## Multiline Content
//!
//! Strings can contain newlines and formatting:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = strings(helia);
//!
//! let document = "\
//! # Title
//! 
//! Paragraph 1
//! 
//! Paragraph 2
//! ";
//!
//! let cid = str_interface.add(document, None).await?;
//! let retrieved = str_interface.get(cid, None).await?;
//! assert_eq!(retrieved, document);
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! Handle common error scenarios gracefully:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface, StringsError};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = strings(helia);
//!
//! // Parse CID from string
//! let cid_str = "bafkreigh2akiscaildcqabsyg3dfr6chu3fgpregiymsck7e7aqa4s52zy";
//! let cid: cid::Cid = cid_str.parse()?;
//!
//! // Handle retrieval errors
//! match str_interface.get(cid, None).await {
//!     Ok(content) => {
//!         println!("Retrieved: {}", content);
//!     }
//!     Err(StringsError::InvalidCodec(msg)) => {
//!         eprintln!("Wrong codec: {}", msg);
//!     }
//!     Err(StringsError::Blockstore(msg)) => {
//!         eprintln!("Block not found: {}", msg);
//!     }
//!     Err(StringsError::Utf8(e)) => {
//!         eprintln!("Invalid UTF-8: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Building a Simple Key-Value Store
//!
//! Use strings to build a simple in-memory key-value store:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface};
//! # use std::collections::HashMap;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = strings(helia);
//! let mut store: HashMap<String, cid::Cid> = HashMap::new();
//!
//! // Store key-value pairs
//! let cid = str_interface.add("value for key1", None).await?;
//! store.insert("key1".to_string(), cid);
//!
//! // Retrieve by key
//! if let Some(cid) = store.get("key1") {
//!     let value = str_interface.get(*cid, None).await?;
//!     println!("key1 = {}", value);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! ## Memory Usage
//!
//! - **Storage**: Each string is stored once in the blockstore
//! - **Deduplication**: Identical strings share the same CID and block
//! - **Overhead**: CID overhead is ~36-38 bytes per string
//!
//! ## Speed
//!
//! - **Add operation**: O(n) where n is string length (hashing + storage)
//! - **Get operation**: O(1) lookup + O(n) UTF-8 validation
//! - **Typical latency**: Microseconds for small strings, milliseconds for large ones
//!
//! ## Best Practices
//!
//! - **Small to medium strings**: Ideal for <1MB strings
//! - **Large strings**: Consider chunking or using UnixFS for >10MB
//! - **Batch operations**: Add multiple strings in parallel with `tokio::join!`
//! - **Caching**: Store frequently-accessed CIDs to avoid repeated lookups
//!
//! # Comparison with Other Formats
//!
//! | Feature | Strings | JSON | DAG-CBOR | UnixFS |
//! |---------|---------|------|----------|--------|
//! | **Simplicity** | ✅ Easiest | 🟡 Simple | 🟡 Moderate | 🔴 Complex |
//! | **Use Case** | Plain text | Structured data | Binary data | Files/dirs |
//! | **Overhead** | Minimal | Low | Low | Higher |
//! | **Best For** | Learning, text | Config, API | Efficient data | Large files |
//!
//! # Thread Safety
//!
//! The `StringsInterface` trait requires `Send + Sync`, making it safe to use across
//! async tasks:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_strings::{strings, StringsInterface};
//! # use std::sync::Arc;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = std::sync::Arc::new(create_helia_default().await?);
//! let str_interface = Arc::new(strings(helia));
//!
//! // Use across multiple tasks
//! let str1 = str_interface.clone();
//! let task1 = tokio::spawn(async move {
//!     str1.add("task 1 data", None).await
//! });
//!
//! let str2 = str_interface.clone();
//! let task2 = tokio::spawn(async move {
//!     str2.add("task 2 data", None).await
//! });
//!
//! let (cid1, cid2) = tokio::try_join!(task1, task2)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Examples
//!
//! See the `examples/` directory for complete working examples:
//!
//! - `basic_strings.rs` - Simple add/get operations
//! - `unicode_strings.rs` - Working with Unicode text
//! - `string_store.rs` - Building a key-value store
//! - `batch_operations.rs` - Parallel string operations
//!
//! # Integration with IPFS Ecosystem
//!
//! Strings stored with helia-strings are compatible with:
//!
//! - **JavaScript Helia**: `@helia/strings` package
//! - **IPFS CLI**: `ipfs block get <CID>` (raw bytes)
//! - **HTTP Gateways**: `https://ipfs.io/ipfs/<CID>` (displays as text)
//! - **Other IPFS implementations**: Kubo, Iroh, etc.
//!
//! # Limitations
//!
//! - **Size limit**: Depends on blockstore (typically <10MB recommended)
//! - **No encryption**: Strings are stored as plaintext
//! - **No compression**: Raw UTF-8 encoding (consider compressing large text)
//! - **Immutable**: Cannot modify existing strings (must add new versions)
//!
//! # See Also
//!
//! - [`helia-json`](../helia_json) - For JSON data structures
//! - [`helia-dag-json`](../helia_dag_json) - For linked JSON (DAG)
//! - [`helia-unixfs`](../helia_unixfs) - For files and directories
//! - [`helia-interface`](../helia_interface) - Core Helia traits

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

    #[tokio::test]
    async fn test_empty_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let empty = "";
        let cid = str_interface.add(empty, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, empty);
        assert_eq!(retrieved.len(), 0);
    }

    #[tokio::test]
    async fn test_very_long_string() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        // Create a 10KB string
        let long_string = "a".repeat(10_000);
        let cid = str_interface.add(&long_string, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, long_string);
        assert_eq!(retrieved.len(), 10_000);
    }

    #[tokio::test]
    async fn test_special_characters() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let special = "!@#$%^&*()_+-=[]{}|;:',.<>?/\\`~\"";
        let cid = str_interface.add(special, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, special);
    }

    #[tokio::test]
    async fn test_whitespace_only() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let whitespace = "   \t\n\r   ";
        let cid = str_interface.add(whitespace, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, whitespace);
    }

    #[tokio::test]
    async fn test_json_string_roundtrip() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let json_str = r#"{"name":"Alice","age":30,"active":true}"#;
        let cid = str_interface.add(json_str, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, json_str);
    }

    #[tokio::test]
    async fn test_multiple_emojis() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let emojis = "😀😃😄😁😆😅🤣😂🙂🙃";
        let cid = str_interface.add(emojis, None).await.unwrap();
        let retrieved = str_interface.get(cid, None).await.unwrap();

        assert_eq!(retrieved, emojis);
        // Verify emoji count (each emoji is multiple bytes)
        assert_eq!(retrieved.chars().count(), 10);
    }

    #[tokio::test]
    async fn test_cid_string_format() {
        let helia = create_test_helia().await;
        let str_interface = strings(helia);

        let text = "test content";
        let cid = str_interface.add(text, None).await.unwrap();

        // Verify CID properties
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), 0x55); // raw codec
        assert_eq!(cid.hash().code(), 0x12); // SHA-256

        // CID should be a valid string
        let cid_string = cid.to_string();
        assert!(cid_string.starts_with("bafkrei"));
    }

    #[tokio::test]
    async fn test_concurrent_adds() {
        use tokio::task::JoinSet;
        
        let helia = create_test_helia().await;

        // Add multiple strings concurrently
        let mut set = JoinSet::new();
        for i in 0..10 {
            let helia_clone = Arc::clone(&helia);
            set.spawn(async move {
                let str_interface = strings(helia_clone);
                str_interface
                    .add(&format!("concurrent test {}", i), None)
                    .await
            });
        }

        // All should succeed
        let mut count = 0;
        while let Some(result) = set.join_next().await {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
            count += 1;
        }
        assert_eq!(count, 10);
    }
}
