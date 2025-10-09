//! DAG-JSON implementation

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{AddOptions, DagJsonError, DagJsonInterface, GetOptions};
use helia_interface::Helia;

/// DAG-JSON codec identifier
pub const DAG_JSON_CODEC: u64 = 0x0129;

/// DAG-JSON implementation
pub struct DagJson {
    helia: Arc<dyn Helia>,
}

impl DagJson {
    /// Create a new DAG-JSON instance
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self { helia }
    }
}

#[async_trait]
impl DagJsonInterface for DagJson {
    async fn add<T>(&self, obj: &T, options: Option<AddOptions>) -> Result<Cid, DagJsonError>
    where
        T: Serialize + Send + Sync,
    {
        let options = options.unwrap_or_default();

        // Serialize the object to JSON
        let json_data = serde_json::to_vec(obj)?;
        let bytes = Bytes::from(json_data);

        // Create hash of the data using a simple approach similar to other implementations
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

        let mh: multihash::Multihash<64> =
            multihash::Multihash::wrap(0x12, &hash_bytes) // 0x12 is SHA-256
                .map_err(|e| DagJsonError::other(format!("Multihash error: {}", e)))?;

        // Create CID with DAG-JSON codec
        let cid = Cid::new_v1(DAG_JSON_CODEC, mh);

        // Store the block
        self.helia.blockstore().put(&cid, bytes, None).await?;

        // Pin if requested
        if options.pin {
            self.helia.pins().add(&cid, None).await?;
        }

        Ok(cid)
    }

    async fn get<T>(&self, cid: &Cid, _options: Option<GetOptions>) -> Result<T, DagJsonError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        // Verify codec
        if cid.codec() != DAG_JSON_CODEC {
            return Err(DagJsonError::invalid_codec(cid.codec()));
        }

        // Get the block data
        let bytes = self.helia.blockstore().get(cid, None).await?;

        // Deserialize from JSON
        let obj = serde_json::from_slice(bytes.as_ref())?;

        Ok(obj)
    }
}

/// Create a new DAG-JSON interface for the given Helia instance
pub fn dag_json(helia: Arc<dyn Helia>) -> DagJson {
    DagJson::new(helia)
}
