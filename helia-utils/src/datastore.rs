//! Datastore implementations

use async_trait::async_trait;
use bytes::Bytes;
use sled::Db;
use futures::stream;

use helia_interface::*;
use crate::DatastoreConfig;

/// Sled-based datastore implementation
pub struct SledDatastore {
    db: Db,
}

impl SledDatastore {
    pub fn new(config: DatastoreConfig) -> Result<Self, HeliaError> {
        let db = if let Some(path) = config.path {
            sled::open(path).map_err(|e| HeliaError::datastore(format!("Failed to open datastore: {}", e)))?
        } else {
            sled::Config::new()
                .temporary(true)
                .open()
                .map_err(|e| HeliaError::datastore(format!("Failed to create temporary datastore: {}", e)))?
        };

        Ok(Self { db })
    }
}

#[async_trait]
impl Datastore for SledDatastore {
    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, HeliaError> {
        match self.db.get(key) {
            Ok(Some(data)) => Ok(Some(Bytes::from(data.to_vec()))),
            Ok(None) => Ok(None),
            Err(e) => Err(HeliaError::datastore(format!("Datastore get error: {}", e))),
        }
    }

    async fn put(&self, key: &[u8], value: Bytes) -> Result<(), HeliaError> {
        self.db
            .insert(key, value.as_ref())
            .map_err(|e| HeliaError::datastore(format!("Datastore put error: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), HeliaError> {
        self.db
            .remove(key)
            .map_err(|e| HeliaError::datastore(format!("Datastore delete error: {}", e)))?;
        Ok(())
    }

    async fn has(&self, key: &[u8]) -> Result<bool, HeliaError> {
        match self.db.contains_key(key) {
            Ok(exists) => Ok(exists),
            Err(e) => Err(HeliaError::datastore(format!("Datastore has error: {}", e))),
        }
    }

    async fn query(&self, prefix: Option<&[u8]>) -> Result<AwaitIterable<Bytes>, HeliaError> {
        let mut results = Vec::new();
        
        if let Some(prefix) = prefix {
            // Iterate through keys with the given prefix
            for item in self.db.scan_prefix(prefix) {
                match item {
                    Ok((_key, value)) => {
                        results.push(Bytes::from(value.to_vec()));
                    }
                    Err(e) => {
                        return Err(HeliaError::datastore(format!("Query error: {}", e)));
                    }
                }
            }
        } else {
            // Iterate through all keys
            for item in self.db.iter() {
                match item {
                    Ok((_key, value)) => {
                        results.push(Bytes::from(value.to_vec()));
                    }
                    Err(e) => {
                        return Err(HeliaError::datastore(format!("Query error: {}", e)));
                    }
                }
            }
        }
        
        Ok(Box::pin(stream::iter(results)))
    }
}