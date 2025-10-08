//! IPNS (InterPlanetary Name System) implementation for Helia
//! 
//! IPNS provides mutable pointers to content-addressed data. This allows
//! updating IPFS content without changing its address.

use std::sync::Arc;
use std::collections::HashMap;
use std::time::SystemTime;

use async_trait::async_trait;
use cid::Cid;
use bytes::Bytes;
use serde::{Serialize, Deserialize};

use helia_interface::Helia;

/// Errors that can occur during IPNS operations
#[derive(Debug, thiserror::Error)]
pub enum IpnsError {
    #[error("Invalid IPNS name: {0}")]
    InvalidName(String),
    
    #[error("Name not found: {0}")]
    NotFound(String),
    
    #[error("Publish failed: {0}")]
    PublishFailed(String),
    
    #[error("Resolve failed: {0}")]
    ResolveFailed(String),
    
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Record expired")]
    RecordExpired,
}

/// IPNS record containing published content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpnsRecord {
    /// The CID this name points to
    pub value: String,
    
    /// Sequence number (for ordering)
    pub sequence: u64,
    
    /// Validity period start
    pub validity: String,
    
    /// Time to live in nanoseconds
    pub ttl: u64,
    
    /// Public key that signed this record
    pub public_key: Bytes,
}

/// Result of resolving an IPNS name
#[derive(Debug, Clone)]
pub struct ResolveResult {
    /// The CID the name resolves to
    pub cid: Cid,
    
    /// Optional path component
    pub path: Option<String>,
}

/// Options for publishing IPNS records
#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    /// Time to live in seconds
    pub ttl: Option<u64>,
    
    /// Key name to use for signing
    pub key: Option<String>,
    
    /// Whether to allow recursion
    pub allow_recursive: bool,
}

/// Options for resolving IPNS names
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    /// Whether to resolve recursively
    pub recursive: bool,
    
    /// Maximum number of recursive resolutions
    pub max_depth: Option<u32>,
    
    /// Timeout in seconds
    pub timeout: Option<u64>,
}

/// The main IPNS interface
#[async_trait]
pub trait IpnsInterface: Send + Sync {
    /// Publish a CID under a name
    async fn publish(&self, key: &str, cid: &Cid, options: Option<PublishOptions>) -> Result<IpnsRecord, IpnsError>;
    
    /// Resolve an IPNS name to a CID
    async fn resolve(&self, name: &str, options: Option<ResolveOptions>) -> Result<ResolveResult, IpnsError>;
    
    /// Get the local record for a key
    async fn get_local(&self, key: &str) -> Result<Option<IpnsRecord>, IpnsError>;
}

/// Default IPNS implementation
pub struct Ipns {
    helia: Arc<dyn Helia>,
    // In-memory store for simplicity (production would use datastore)
    records: Arc<tokio::sync::RwLock<HashMap<String, IpnsRecord>>>,
}

impl Ipns {
    /// Create a new IPNS instance
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self {
            helia,
            records: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    fn create_record(&self, cid: &Cid, sequence: u64, ttl: u64) -> IpnsRecord {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        
        IpnsRecord {
            value: format!("/ipfs/{}", cid),
            sequence,
            validity: format!("{:?}", now),
            ttl,
            public_key: Bytes::from(vec![0u8; 32]), // Placeholder
        }
    }
    
    fn parse_value(&self, value: &str) -> Result<(Cid, Option<String>), IpnsError> {
        // Parse /ipfs/<cid> or /ipfs/<cid>/path
        if !value.starts_with("/ipfs/") {
            return Err(IpnsError::InvalidName(format!("Invalid IPNS value: {}", value)));
        }
        
        let without_prefix = &value[6..]; // Remove "/ipfs/"
        let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
        
        let cid = Cid::try_from(parts[0])
            .map_err(|e| IpnsError::InvalidName(format!("Invalid CID: {}", e)))?;
        
        let path = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };
        
        Ok((cid, path))
    }
}

#[async_trait]
impl IpnsInterface for Ipns {
    async fn publish(&self, key: &str, cid: &Cid, options: Option<PublishOptions>) -> Result<IpnsRecord, IpnsError> {
        let options = options.unwrap_or_default();
        let ttl = options.ttl.unwrap_or(3600); // Default 1 hour
        
        // Get current sequence number
        let mut records = self.records.write().await;
        let sequence = records.get(key).map(|r| r.sequence + 1).unwrap_or(0);
        
        // Create new record
        let record = self.create_record(cid, sequence, ttl);
        
        // Store locally
        records.insert(key.to_string(), record.clone());
        
        // In a full implementation, we would:
        // 1. Sign the record with the private key
        // 2. Publish to DHT
        // 3. Publish to PubSub if configured
        
        Ok(record)
    }
    
    async fn resolve(&self, name: &str, options: Option<ResolveOptions>) -> Result<ResolveResult, IpnsError> {
        let options = options.unwrap_or_default();
        let max_depth = options.max_depth.unwrap_or(32);
        
        // Try local store first
        let records = self.records.read().await;
        if let Some(record) = records.get(name) {
            let (cid, path) = self.parse_value(&record.value)?;
            return Ok(ResolveResult { cid, path });
        }
        
        // In a full implementation, we would:
        // 1. Query DHT for the record
        // 2. Verify signature
        // 3. Check validity
        // 4. Recursively resolve if needed
        
        Err(IpnsError::NotFound(name.to_string()))
    }
    
    async fn get_local(&self, key: &str) -> Result<Option<IpnsRecord>, IpnsError> {
        let records = self.records.read().await;
        Ok(records.get(key).cloned())
    }
}

/// Create an IPNS instance
pub fn ipns(helia: Arc<dyn Helia>) -> impl IpnsInterface {
    Ipns::new(helia)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_cid() -> Cid {
        // Create a test CID
        let hash = multihash::Multihash::<64>::wrap(0x12, &[0u8; 32]).unwrap();
        Cid::new_v1(0x55, hash) // 0x55 is raw codec
    }
    
    #[tokio::test]
    async fn test_ipns_publish_and_resolve() {
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let ipns = Ipns::new(helia);
        
        let cid = create_test_cid();
        let key = "test-key";
        
        // Publish
        let record = ipns.publish(key, &cid, None).await.unwrap();
        assert_eq!(record.sequence, 0);
        assert!(record.value.contains(&cid.to_string()));
        
        // Resolve
        let result = ipns.resolve(key, None).await.unwrap();
        assert_eq!(result.cid, cid);
        assert_eq!(result.path, None);
    }
    
    #[tokio::test]
    async fn test_ipns_sequence_increment() {
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let ipns = Ipns::new(helia);
        
        let cid1 = create_test_cid();
        let key = "test-key";
        
        // First publish
        let record1 = ipns.publish(key, &cid1, None).await.unwrap();
        assert_eq!(record1.sequence, 0);
        
        // Second publish
        let record2 = ipns.publish(key, &cid1, None).await.unwrap();
        assert_eq!(record2.sequence, 1);
    }
    
    #[tokio::test]
    async fn test_ipns_not_found() {
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let ipns = Ipns::new(helia);
        
        let result = ipns.resolve("non-existent-key", None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IpnsError::NotFound(_)));
    }
    
    #[tokio::test]
    async fn test_get_local() {
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let ipns = Ipns::new(helia);
        
        let cid = create_test_cid();
        let key = "test-key";
        
        // Initially no record
        let result = ipns.get_local(key).await.unwrap();
        assert!(result.is_none());
        
        // Publish
        ipns.publish(key, &cid, None).await.unwrap();
        
        // Now should have record
        let result = ipns.get_local(key).await.unwrap();
        assert!(result.is_some());
    }
}
