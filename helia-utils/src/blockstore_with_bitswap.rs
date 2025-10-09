//! Blockstore with Bitswap integration
//!
//! This module provides a blockstore wrapper that integrates local storage
//! with Bitswap for network-based block retrieval.

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use helia_bitswap::{Bitswap, WantOptions, NotifyOptions};
use helia_interface::{
    blocks::{Blocks, GetBlockOptions, PutBlockOptions, GetManyOptions, GetAllOptions, 
             PutManyOptions, HasOptions, DeleteManyOptions, Pair, InputPair},
    HeliaError, AwaitIterable,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::SledBlockstore;

/// Blockstore that integrates local storage with Bitswap for network retrieval
pub struct BlockstoreWithBitswap {
    /// Local blockstore (fast path)
    local: Arc<SledBlockstore>,
    /// Bitswap coordinator (network path)
    bitswap: Arc<Bitswap>,
}

impl BlockstoreWithBitswap {
    /// Create a new blockstore with Bitswap integration
    pub fn new(local: Arc<SledBlockstore>, bitswap: Arc<Bitswap>) -> Self {
        Self { local, bitswap }
    }

    /// Get the underlying local blockstore
    pub fn local(&self) -> &Arc<SledBlockstore> {
        &self.local
    }

    /// Get the Bitswap coordinator
    pub fn bitswap(&self) -> &Arc<Bitswap> {
        &self.bitswap
    }
}

#[async_trait]
impl Blocks for BlockstoreWithBitswap {
    async fn get(
        &self,
        cid: &Cid,
        options: Option<GetBlockOptions>,
    ) -> Result<Bytes, HeliaError> {
        debug!("BlockstoreWithBitswap: get() called for CID: {}", cid);

        // Try local blockstore first (fast path)
        debug!("  Step 1: Checking local blockstore...");
        match self.local.get(cid, options.clone()).await {
            Ok(data) => {
                debug!("  ✅ Found in local blockstore ({} bytes)", data.len());
                return Ok(data);
            }
            Err(_) => {
                debug!("  ⚠️  Not in local blockstore");
            }
        }

        // Not in local storage, fetch via Bitswap (slow path)
        info!(
            "  Step 2: Block not in local storage, fetching via Bitswap: {}",
            cid
        );

        let want_options = WantOptions {
            timeout: Some(Duration::from_secs(30)),
            priority: 10,
            accept_block_presence: true,
            peer: None,
        };

        match self.bitswap.want(cid, want_options).await {
            Ok(data) => {
                info!("  ✅ Retrieved from network ({} bytes)", data.len());

                // Store in local blockstore for future use
                debug!("  Step 3: Storing in local blockstore for caching...");
                if let Err(e) = self.local.put(cid, data.clone(), None).await {
                    warn!("  ⚠️  Failed to cache block locally: {}", e);
                    // Don't fail the operation if caching fails
                }

                Ok(data)
            }
            Err(e) => {
                warn!("  ❌ Failed to retrieve from network: {}", e);
                Err(e)
            }
        }
    }

    async fn put(
        &self,
        cid: &Cid,
        data: Bytes,
        options: Option<PutBlockOptions>,
    ) -> Result<Cid, HeliaError> {
        debug!("BlockstoreWithBitswap: put() called for CID: {}", cid);

        // Store in local blockstore first
        debug!("  Step 1: Storing in local blockstore...");
        let returned_cid = self.local.put(cid, data.clone(), options).await?;
        debug!("  ✅ Stored locally");

        // Announce to network via Bitswap
        debug!("  Step 2: Announcing to network via Bitswap...");
        let notify_options = NotifyOptions { broadcast: true };

        match self
            .bitswap
            .notify_new_blocks(vec![(*cid, data)], notify_options)
            .await
        {
            Ok(_) => {
                debug!("  ✅ Announced to network");
                Ok(returned_cid)
            }
            Err(e) => {
                warn!("  ⚠️  Failed to announce to network: {}", e);
                // Don't fail the operation if announcement fails
                // The block is already stored locally
                Ok(returned_cid)
            }
        }
    }

    async fn has(
        &self,
        cid: &Cid,
        options: Option<HasOptions>,
    ) -> Result<bool, HeliaError> {
        // Only check local blockstore
        // We don't query the network for has() to avoid unnecessary traffic
        self.local.has(cid, options).await
    }

    async fn get_many_cids(
        &self,
        cids: Vec<Cid>,
        options: Option<GetManyOptions>,
    ) -> Result<AwaitIterable<Result<Pair, HeliaError>>, HeliaError> {
        // For each CID, try local first, then network
        // This is similar to get() but for multiple CIDs
        self.local.get_many_cids(cids, options).await
    }

    async fn get_all(
        &self,
        options: Option<GetAllOptions>,
    ) -> Result<AwaitIterable<Pair>, HeliaError> {
        // Only return blocks from local storage
        self.local.get_all(options).await
    }

    async fn put_many_blocks(
        &self,
        blocks: Vec<InputPair>,
        options: Option<PutManyOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError> {
        // Store locally first
        let cids = self.local.put_many_blocks(blocks.clone(), options).await?;

        // Announce all blocks to network
        let blocks_to_announce: Vec<(Cid, Bytes)> = blocks
            .into_iter()
            .filter_map(|input_pair| input_pair.cid.map(|cid| (cid, input_pair.block)))
            .collect();

        if !blocks_to_announce.is_empty() {
            let notify_options = NotifyOptions { broadcast: true };
            if let Err(e) = self
                .bitswap
                .notify_new_blocks(blocks_to_announce, notify_options)
                .await
            {
                warn!("Failed to announce blocks to network: {}", e);
                // Don't fail the operation if announcement fails
            }
        }

        Ok(cids)
    }

    async fn has_many_cids(
        &self,
        cids: Vec<Cid>,
        options: Option<HasOptions>,
    ) -> Result<AwaitIterable<bool>, HeliaError> {
        // Only check local blockstore
        self.local.has_many_cids(cids, options).await
    }

    async fn delete_many_cids(
        &self,
        cids: Vec<Cid>,
        options: Option<DeleteManyOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError> {
        // Only delete from local blockstore
        // We can't "un-announce" to the network
        self.local.delete_many_cids(cids, options).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockstoreConfig;
    use helia_bitswap::BitswapConfig;

    #[tokio::test]
    async fn test_blockstore_with_bitswap_creation() {
        let local = Arc::new(SledBlockstore::new(BlockstoreConfig::default()).unwrap());
        let bitswap = Arc::new(
            Bitswap::new(local.clone() as Arc<dyn Blocks>, BitswapConfig::default())
                .await
                .unwrap(),
        );

        let blockstore = BlockstoreWithBitswap::new(local.clone(), bitswap.clone());

        assert!(Arc::ptr_eq(blockstore.local(), &local));
        assert!(Arc::ptr_eq(blockstore.bitswap(), &bitswap));
    }

    #[tokio::test]
    async fn test_local_get_works() {
        let local = Arc::new(SledBlockstore::new(BlockstoreConfig::default()).unwrap());
        let bitswap = Arc::new(
            Bitswap::new(local.clone() as Arc<dyn Blocks>, BitswapConfig::default())
                .await
                .unwrap(),
        );

        let blockstore = BlockstoreWithBitswap::new(local.clone(), bitswap);

        // Create a test block
        let data = Bytes::from("test data");
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(&data);
        let hash = hasher.finalize();

        let mut mh_bytes = vec![0x12, 0x20];
        mh_bytes.extend_from_slice(&hash);
        let mh = multihash::Multihash::from_bytes(&mh_bytes).unwrap();
        let cid = Cid::new_v1(0x55, mh);

        // Store directly in local blockstore
        local.put(&cid, data.clone(), None).await.unwrap();

        // Get via wrapper should find it locally (no network call)
        let retrieved = blockstore.get(&cid, None).await.unwrap();
        assert_eq!(retrieved, data);
    }
}
