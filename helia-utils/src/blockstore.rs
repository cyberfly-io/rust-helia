//! Blockstore implementations

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::stream;
use sled::Db;

use crate::BlockstoreConfig;
use helia_interface::*;

/// Sled-based blockstore implementation
pub struct SledBlockstore {
    db: Db,
}

impl SledBlockstore {
    pub fn new(config: BlockstoreConfig) -> Result<Self, HeliaError> {
        let db = if let Some(path) = config.path {
            sled::open(path)
                .map_err(|e| HeliaError::other(format!("Failed to open blockstore: {}", e)))?
        } else {
            sled::Config::new().temporary(true).open().map_err(|e| {
                HeliaError::other(format!("Failed to create temporary blockstore: {}", e))
            })?
        };

        Ok(Self { db })
    }

    fn cid_to_key(&self, cid: &Cid) -> Vec<u8> {
        format!("block:{}", cid).into_bytes()
    }
}

#[async_trait]
impl Blocks for SledBlockstore {
    async fn get(&self, cid: &Cid, _options: Option<GetBlockOptions>) -> Result<Bytes, HeliaError> {
        let key = self.cid_to_key(cid);
        match self.db.get(&key) {
            Ok(Some(data)) => Ok(Bytes::from(data.to_vec())),
            Ok(None) => Err(HeliaError::BlockNotFound { cid: *cid }),
            Err(e) => Err(HeliaError::other(format!("Blockstore get error: {}", e))),
        }
    }

    async fn get_many_cids(
        &self,
        cids: Vec<Cid>,
        _options: Option<GetManyOptions>,
    ) -> Result<AwaitIterable<Result<Pair, HeliaError>>, HeliaError> {
        let mut results = Vec::new();

        for cid in cids {
            let result = match self.get(&cid, None).await {
                Ok(block) => Ok(Pair { cid, block }),
                Err(e) => Err(e),
            };
            results.push(result);
        }

        Ok(Box::pin(stream::iter(results)))
    }

    async fn get_all(
        &self,
        _options: Option<GetAllOptions>,
    ) -> Result<AwaitIterable<Pair>, HeliaError> {
        let mut results = Vec::new();

        // Iterate through all blocks in the database
        for item in self.db.iter() {
            match item {
                Ok((key_bytes, value_bytes)) => {
                    // Parse the key to extract CID
                    if let Ok(key_str) = std::str::from_utf8(&key_bytes) {
                        if let Some(cid_str) = key_str.strip_prefix("block:") {
                            if let Ok(cid) = cid_str.parse::<Cid>() {
                                let block = Bytes::from(value_bytes.to_vec());
                                results.push(Pair { cid, block });
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(HeliaError::other(format!("Error iterating blocks: {}", e)));
                }
            }
        }

        Ok(Box::pin(stream::iter(results)))
    }

    async fn put(
        &self,
        cid: &Cid,
        block: Bytes,
        _options: Option<PutBlockOptions>,
    ) -> Result<Cid, HeliaError> {
        let key = self.cid_to_key(cid);
        self.db
            .insert(&key, block.as_ref())
            .map_err(|e| HeliaError::other(format!("Blockstore put error: {}", e)))?;
        Ok(*cid)
    }

    async fn put_many_blocks(
        &self,
        blocks: Vec<InputPair>,
        _options: Option<PutManyOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError> {
        let mut results = Vec::new();

        for input_pair in blocks {
            // If CID is not provided, we'd need to compute it from the block
            // For now, we'll require CID to be provided
            let cid = input_pair
                .cid
                .ok_or_else(|| HeliaError::other("CID is required for putting block"))?;

            match self.put(&cid, input_pair.block, None).await {
                Ok(returned_cid) => results.push(returned_cid),
                Err(e) => return Err(e), // Fail fast on any error
            }
        }

        Ok(Box::pin(stream::iter(results)))
    }

    async fn has(&self, cid: &Cid, _options: Option<HasOptions>) -> Result<bool, HeliaError> {
        let key = self.cid_to_key(cid);
        match self.db.contains_key(&key) {
            Ok(exists) => Ok(exists),
            Err(e) => Err(HeliaError::other(format!("Blockstore has error: {}", e))),
        }
    }

    async fn has_many_cids(
        &self,
        cids: Vec<Cid>,
        _options: Option<HasOptions>,
    ) -> Result<AwaitIterable<bool>, HeliaError> {
        let mut results = Vec::new();

        for cid in cids {
            match self.has(&cid, None).await {
                Ok(exists) => results.push(exists),
                Err(e) => return Err(e), // Fail fast on any error
            }
        }

        Ok(Box::pin(stream::iter(results)))
    }

    async fn delete_many_cids(
        &self,
        cids: Vec<Cid>,
        _options: Option<DeleteManyOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError> {
        let mut results = Vec::new();

        for cid in cids {
            let key = self.cid_to_key(&cid);
            match self.db.remove(&key) {
                Ok(_) => results.push(cid), // Successfully deleted
                Err(e) => {
                    return Err(HeliaError::other(format!(
                        "Delete error for {}: {}",
                        cid, e
                    )))
                }
            }
        }

        Ok(Box::pin(stream::iter(results)))
    }
}
