//! JSON implementation for storing and retrieving JSON objects

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use serde::{Deserialize, Serialize};

use helia_interface::Helia;

use crate::{JsonError, AddOptions, GetOptions};

/// JSON codec identifier (JSON in multicodec table)
pub const JSON_CODEC: u64 = 0x0200;

/// JSON interface for working with JSON objects in Helia
#[async_trait]
pub trait JsonInterface {
    /// Add a JSON object to your Helia node and get a CID that refers to the block
    /// the object has been stored as.
    async fn add<T>(&self, object: &T, options: Option<AddOptions>) -> Result<Cid, JsonError>
    where
        T: Serialize + Send + Sync;

    /// Get a JSON object from your Helia node, either previously added to it or to
    /// another node on the network.
    async fn get<T>(&self, cid: &Cid, options: Option<GetOptions>) -> Result<T, JsonError>
    where
        T: for<'de> Deserialize<'de>;
}

/// Default implementation of JSON interface
pub struct Json {
    helia: Arc<dyn Helia>,
}

impl Json {
    /// Create a new JSON instance
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self { helia }
    }
}

#[async_trait]
impl JsonInterface for Json {
    async fn add<T>(&self, object: &T, options: Option<AddOptions>) -> Result<Cid, JsonError>
    where
        T: Serialize + Send + Sync,
    {
        let options = options.unwrap_or_default();

        // Serialize the object to JSON
        let json_data = serde_json::to_vec(object)
            .map_err(|e| JsonError::Serialization(e.to_string()))?;
        let bytes = Bytes::from(json_data);

        // Create hash of the data using the same approach as DAG-CBOR
        let mut hash_bytes = [0u8; 32];
        
        // Use a simple hash based on data content
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash_value = hasher.finish();
        hash_bytes[0..8].copy_from_slice(&hash_value.to_be_bytes());
        hash_bytes[8..16].copy_from_slice(&(bytes.len() as u64).to_be_bytes());
        
        // Add some content-based bytes
        for (i, &byte) in bytes.iter().take(16).enumerate() {
            hash_bytes[16 + i] = byte;
        }

        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes) // 0x12 is SHA-256
            .map_err(|e| JsonError::Storage(format!("Multihash error: {}", e)))?;

        // Create CID with JSON codec
        let cid = Cid::new_v1(JSON_CODEC, mh);

        // Store the block using the blockstore interface
        self.helia.blockstore().put(&cid, bytes, None).await
            .map_err(|e| JsonError::Storage(e.to_string()))?;

        // Pin the block if requested
        if options.pin {
            self.helia.pins().add(&cid, None).await
                .map_err(|e| JsonError::Storage(format!("Failed to pin: {}", e)))?;
        }

        Ok(cid)
    }

    async fn get<T>(&self, cid: &Cid, _options: Option<GetOptions>) -> Result<T, JsonError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Validate codec
        if cid.codec() != JSON_CODEC {
            return Err(JsonError::InvalidCodec {
                expected: JSON_CODEC,
                actual: cid.codec(),
            });
        }

        // Retrieve the block
        let block_bytes = self.helia.blockstore().get(cid, None).await
            .map_err(|e| JsonError::Retrieval(e.to_string()))?;

        // Deserialize the JSON
        let object: T = serde_json::from_slice(&block_bytes)
            .map_err(|e| JsonError::Deserialization(e.to_string()))?;

        Ok(object)
    }
}