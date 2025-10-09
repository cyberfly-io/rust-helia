//! Local storage for IPNS records with caching and metadata

use crate::errors::IpnsError;
use crate::record::IpnsRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata associated with a stored IPNS record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    /// The key name used to publish this record
    pub key_name: String,

    /// Lifetime of the record in milliseconds
    pub lifetime: u64,

    /// When the record was created/stored (Unix timestamp in milliseconds)
    pub created: u64,
}

impl RecordMetadata {
    /// Create new metadata
    pub fn new(key_name: String, lifetime: u64) -> Self {
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            key_name,
            lifetime,
            created,
        }
    }

    /// Get created time as SystemTime
    pub fn created_time(&self) -> SystemTime {
        UNIX_EPOCH + std::time::Duration::from_millis(self.created)
    }

    /// Check if the record should be republished based on DHT expiry or record expiry
    pub fn should_republish(&self, dht_expiry_ms: u64, republish_threshold_ms: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let dht_expiry = self.created + dht_expiry_ms;
        let record_expiry = self.created + self.lifetime;

        // If DHT expiry is within threshold, republish
        if dht_expiry.saturating_sub(now) <= republish_threshold_ms {
            return true;
        }

        // If record expiry is within threshold, republish
        if record_expiry.saturating_sub(now) <= republish_threshold_ms {
            return true;
        }

        false
    }
}

/// Stored record with metadata
#[derive(Debug, Clone)]
pub struct StoredRecord {
    /// The marshaled IPNS record
    pub record: Vec<u8>,

    /// Metadata about the record
    pub metadata: Option<RecordMetadata>,

    /// When this record was stored locally (Unix timestamp in milliseconds)
    pub created: u64,
}

/// Local store for IPNS records
///
/// Provides caching with TTL tracking and metadata storage
#[derive(Debug, Clone)]
pub struct LocalStore {
    records: Arc<RwLock<HashMap<Vec<u8>, StoredRecord>>>,
}

impl LocalStore {
    /// Create a new local store
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store an IPNS record
    pub fn put(
        &self,
        routing_key: &[u8],
        record: Vec<u8>,
        metadata: Option<RecordMetadata>,
    ) -> Result<(), IpnsError> {
        let created = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let stored = StoredRecord {
            record,
            metadata,
            created,
        };

        let mut records = self.records.write().unwrap();
        records.insert(routing_key.to_vec(), stored);

        tracing::debug!(
            "Stored IPNS record for routing key: {}",
            bs58::encode(routing_key).into_string()
        );

        Ok(())
    }

    /// Get an IPNS record
    pub fn get(&self, routing_key: &[u8]) -> Result<StoredRecord, IpnsError> {
        let records = self.records.read().unwrap();

        records.get(routing_key).cloned().ok_or_else(|| {
            IpnsError::NotFound(format!(
                "No record found for routing key: {}",
                bs58::encode(routing_key).into_string()
            ))
        })
    }

    /// Check if a record exists
    pub fn has(&self, routing_key: &[u8]) -> bool {
        let records = self.records.read().unwrap();
        records.contains_key(routing_key)
    }

    /// Delete a record
    pub fn delete(&self, routing_key: &[u8]) -> Result<(), IpnsError> {
        let mut records = self.records.write().unwrap();

        if records.remove(routing_key).is_some() {
            tracing::debug!(
                "Deleted IPNS record for routing key: {}",
                bs58::encode(routing_key).into_string()
            );
            Ok(())
        } else {
            Err(IpnsError::NotFound(format!(
                "No record found for routing key: {}",
                bs58::encode(routing_key).into_string()
            )))
        }
    }

    /// List all stored records (for republishing)
    pub fn list(&self) -> Vec<(Vec<u8>, StoredRecord)> {
        let records = self.records.read().unwrap();
        records
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Clear all records
    pub fn clear(&self) {
        let mut records = self.records.write().unwrap();
        records.clear();
        tracing::debug!("Cleared all IPNS records from local store");
    }

    /// Get the number of stored records
    pub fn len(&self) -> usize {
        let records = self.records.read().unwrap();
        records.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for LocalStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_store_operations() {
        let store = LocalStore::new();
        let routing_key = b"test-key";
        let record = b"test-record".to_vec();

        // Initially empty
        assert!(store.is_empty());
        assert!(!store.has(routing_key));

        // Put a record
        let metadata = RecordMetadata::new("my-key".to_string(), 48 * 60 * 60 * 1000);
        store
            .put(routing_key, record.clone(), Some(metadata.clone()))
            .unwrap();

        // Should now have the record
        assert!(!store.is_empty());
        assert!(store.has(routing_key));
        assert_eq!(store.len(), 1);

        // Get the record
        let stored = store.get(routing_key).unwrap();
        assert_eq!(stored.record, record);
        assert!(stored.metadata.is_some());
        assert_eq!(stored.metadata.unwrap().key_name, "my-key");

        // Delete the record
        store.delete(routing_key).unwrap();
        assert!(store.is_empty());
        assert!(!store.has(routing_key));
    }

    #[test]
    fn test_should_republish() {
        let metadata = RecordMetadata {
            key_name: "test".to_string(),
            lifetime: 48 * 60 * 60 * 1000, // 48 hours
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                - (20 * 60 * 60 * 1000), // Created 20 hours ago
        };

        let dht_expiry_ms = 24 * 60 * 60 * 1000; // 24 hours
        let threshold_ms = 4 * 60 * 60 * 1000; // 4 hours

        // Should need republishing (DHT will expire in 4 hours)
        assert!(metadata.should_republish(dht_expiry_ms, threshold_ms));
    }
}
