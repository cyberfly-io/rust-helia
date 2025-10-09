//! Block storage and retrieval interfaces

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

use crate::{AbortOptions, AwaitIterable, HeliaError, ProgressOptions};

/// A key-value pair for block operations
#[derive(Debug, Clone)]
pub struct Pair {
    pub cid: Cid,
    pub block: Bytes,
}

/// Input pair that may not have been stored yet
#[derive(Debug, Clone)]
pub struct InputPair {
    pub cid: Option<Cid>,
    pub block: Bytes,
}

/// Options for specifying content providers
#[derive(Debug, Clone, Default)]
pub struct ProviderOptions {
    /// An optional list of peers known to host at least the root block of the DAG
    /// that will be fetched.
    ///
    /// If this list is omitted, or if the peers cannot supply the root or any
    /// child blocks, a `findProviders` routing query will be run to find peers
    /// that can supply the blocks.
    pub providers: Vec<ProviderInfo>,
}

/// Information about a content provider
#[derive(Debug, Clone)]
pub enum ProviderInfo {
    /// Provider identified by peer ID
    PeerId(PeerId),
    /// Provider identified by address
    Multiaddr(Multiaddr),
    /// Provider with multiple addresses
    MultipleAddrs(Vec<Multiaddr>),
}

/// Progress events for checking if a block exists
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HasBlockProgressEvents {
    #[serde(rename = "blocks:put:duplicate")]
    Duplicate { cid: Cid },
    #[serde(rename = "blocks:put:providers:notify")]
    ProvidersNotify { cid: Cid },
    #[serde(rename = "blocks:put:blockstore:put")]
    BlockstorePut { cid: Cid },
}

/// Progress events for putting a single block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PutBlockProgressEvents {
    #[serde(rename = "blocks:put:duplicate")]
    Duplicate { cid: Cid },
    #[serde(rename = "blocks:put:providers:notify")]
    ProvidersNotify { cid: Cid },
    #[serde(rename = "blocks:put:blockstore:put")]
    BlockstorePut { cid: Cid },
}

/// Progress events for putting multiple blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PutManyBlocksProgressEvents {
    #[serde(rename = "blocks:put-many:duplicate")]
    Duplicate { cid: Cid },
    #[serde(rename = "blocks:put-many:providers:notify")]
    ProvidersNotify { cid: Cid },
    #[serde(rename = "blocks:put-many:blockstore:put-many")]
    BlockstorePutMany,
}

/// Progress events for getting a block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GetBlockProgressEvents {
    #[serde(rename = "blocks:get:providers:want")]
    ProvidersWant { cid: Cid },
    #[serde(rename = "blocks:get:blockstore:get")]
    BlockstoreGet { cid: Cid },
    #[serde(rename = "blocks:get:blockstore:put")]
    BlockstorePut { cid: Cid },
}

/// Progress events for getting multiple blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GetManyBlocksProgressEvents {
    #[serde(rename = "blocks:get-many:providers:want")]
    ProvidersWant { cid: Cid },
    #[serde(rename = "blocks:get-many:blockstore:get")]
    BlockstoreGet { cid: Cid },
    #[serde(rename = "blocks:get-many:blockstore:put")]
    BlockstorePut { cid: Cid },
}

/// Progress events for getting all blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GetAllBlocksProgressEvents {
    #[serde(rename = "blocks:get-all:providers:want")]
    ProvidersWant { cid: Cid },
    #[serde(rename = "blocks:get-all:blockstore:get")]
    BlockstoreGet { cid: Cid },
    #[serde(rename = "blocks:get-all:blockstore:put")]
    BlockstorePut { cid: Cid },
}

/// Progress events for deleting blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeleteManyBlocksProgressEvents {
    #[serde(rename = "blocks:delete-many:blockstore:delete")]
    BlockstoreDelete { cid: Cid },
}

/// Options for getting a block
#[derive(Debug, Default)]
pub struct GetBlockOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<GetBlockProgressEvents>,
    pub provider: ProviderOptions,
}

impl Clone for GetBlockOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
            provider: self.provider.clone(),
        }
    }
}

/// Options for getting multiple blocks
#[derive(Debug, Default)]
pub struct GetManyOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<GetManyBlocksProgressEvents>,
    pub provider: ProviderOptions,
}

impl Clone for GetManyOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
            provider: self.provider.clone(),
        }
    }
}

/// Options for getting all blocks
#[derive(Debug, Default)]
pub struct GetAllOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<GetAllBlocksProgressEvents>,
    pub provider: ProviderOptions,
}

impl Clone for GetAllOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
            provider: self.provider.clone(),
        }
    }
}

/// Options for putting a block
#[derive(Debug, Default)]
pub struct PutBlockOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<PutBlockProgressEvents>,
}

impl Clone for PutBlockOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
        }
    }
}

/// Options for putting multiple blocks
#[derive(Debug, Default)]
pub struct PutManyOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<PutManyBlocksProgressEvents>,
}

impl Clone for PutManyOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
        }
    }
}

/// Options for checking if blocks exist
#[derive(Debug, Default)]
pub struct HasOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<HasBlockProgressEvents>,
}

impl Clone for HasOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
        }
    }
}

/// Options for deleting blocks
#[derive(Debug, Default)]
pub struct DeleteManyOptions {
    pub abort: AbortOptions,
    pub progress: ProgressOptions<DeleteManyBlocksProgressEvents>,
}

impl Clone for DeleteManyOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
        }
    }
}

/// Block storage interface
#[async_trait]
pub trait Blocks: Send + Sync {
    /// Retrieve a block from the blockstore
    async fn get(&self, cid: &Cid, options: Option<GetBlockOptions>) -> Result<Bytes, HeliaError>;

    /// Retrieve multiple blocks from the blockstore
    async fn get_many_cids(
        &self,
        cids: Vec<Cid>,
        options: Option<GetManyOptions>,
    ) -> Result<AwaitIterable<Result<Pair, HeliaError>>, HeliaError>;

    /// Retrieve all blocks from the blockstore
    async fn get_all(
        &self,
        options: Option<GetAllOptions>,
    ) -> Result<AwaitIterable<Pair>, HeliaError>;

    /// Store a block in the blockstore
    async fn put(
        &self,
        cid: &Cid,
        block: Bytes,
        options: Option<PutBlockOptions>,
    ) -> Result<Cid, HeliaError>;

    /// Store multiple blocks in the blockstore
    async fn put_many_blocks(
        &self,
        blocks: Vec<InputPair>,
        options: Option<PutManyOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError>;

    /// Check if blocks exist in the blockstore
    async fn has(&self, cid: &Cid, options: Option<HasOptions>) -> Result<bool, HeliaError>;

    /// Check if multiple blocks exist in the blockstore
    async fn has_many_cids(
        &self,
        cids: Vec<Cid>,
        options: Option<HasOptions>,
    ) -> Result<AwaitIterable<bool>, HeliaError>;

    /// Delete multiple blocks from the blockstore
    async fn delete_many_cids(
        &self,
        cids: Vec<Cid>,
        options: Option<DeleteManyOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError>;
}
