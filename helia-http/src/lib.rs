// Minimal HTTP-only Helia implementation

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use cid::Cid;
use bytes::Bytes;
use futures::stream;
use tokio::sync::RwLock;
use libp2p::PeerId;
use trust_dns_resolver::TokioAsyncResolver;

use helia_interface::{
    Blocks, HeliaError, Pins, Routing, ComponentLogger, Datastore, 
    Helia, GcOptions, Metrics, Codec, Hasher
};

pub struct HttpBlocks;

#[async_trait]
impl Blocks for HttpBlocks {
    async fn get(&self, cid: &Cid, _options: Option<helia_interface::GetBlockOptions>) -> Result<Bytes, HeliaError> {
        Err(HeliaError::other(format!("Block not found: {}", cid)))
    }

    async fn get_many_cids(
        &self,
        _cids: Vec<Cid>,
        _options: Option<helia_interface::GetManyOptions>,
    ) -> Result<helia_interface::AwaitIterable<Result<helia_interface::Pair, HeliaError>>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn get_all(&self, _options: Option<helia_interface::GetAllOptions>) -> Result<helia_interface::AwaitIterable<helia_interface::Pair>, HeliaError> {
        Err(HeliaError::other("get_all not supported"))
    }

    async fn put(&self, cid: &Cid, _block: Bytes, _options: Option<helia_interface::PutBlockOptions>) -> Result<Cid, HeliaError> {
        Ok(*cid)
    }

    async fn put_many_blocks(
        &self,
        _blocks: Vec<helia_interface::InputPair>,
        _options: Option<helia_interface::PutManyOptions>,
    ) -> Result<helia_interface::AwaitIterable<Cid>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn has(&self, _cid: &Cid, _options: Option<helia_interface::HasOptions>) -> Result<bool, HeliaError> {
        Ok(false)
    }

    async fn has_many_cids(
        &self,
        _cids: Vec<Cid>,
        _options: Option<helia_interface::HasOptions>,
    ) -> Result<helia_interface::AwaitIterable<bool>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn delete_many_cids(
        &self,
        cids: Vec<Cid>,
        _options: Option<helia_interface::DeleteManyOptions>,
    ) -> Result<helia_interface::AwaitIterable<Cid>, HeliaError> {
        let s = stream::iter(cids);
        Ok(Box::pin(s))
    }
}

pub struct HttpPins;

#[async_trait]
impl Pins for HttpPins {
    async fn add(&self, _cid: &Cid, _options: Option<helia_interface::AddOptions>) -> Result<(), HeliaError> {
        Err(HeliaError::other("pinning not supported"))
    }

    async fn rm(&self, _cid: &Cid, _options: Option<helia_interface::RmOptions>) -> Result<(), HeliaError> {
        Err(HeliaError::other("rm not supported"))
    }

    async fn ls(&self, _options: Option<helia_interface::LsOptions>) -> Result<helia_interface::AwaitIterable<helia_interface::pins::Pin>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn is_pinned(&self, _cid: &Cid, _options: Option<helia_interface::IsPinnedOptions>) -> Result<bool, HeliaError> {
        Ok(false)
    }
}

pub struct HttpRouting;

#[async_trait]
impl Routing for HttpRouting {
    async fn find_providers(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::FindProvidersOptions>,
    ) -> Result<helia_interface::AwaitIterable<helia_interface::Provider>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn provide(&self, _cid: &Cid, _options: Option<helia_interface::ProvideOptions>) -> Result<(), HeliaError> {
        Err(HeliaError::other("provide not supported"))
    }

    async fn find_peers(
        &self,
        _peer_id: &PeerId,
        _options: Option<helia_interface::FindPeersOptions>,
    ) -> Result<helia_interface::AwaitIterable<helia_interface::PeerInfo>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn get(
        &self,
        _key: &[u8],
        _options: Option<helia_interface::GetOptions>,
    ) -> Result<Option<helia_interface::RoutingRecord>, HeliaError> {
        Ok(None)
    }

    async fn put(
        &self,
        _key: &[u8],
        _value: &[u8],
        _options: Option<helia_interface::PutOptions>,
    ) -> Result<(), HeliaError> {
        Err(HeliaError::other("put not supported"))
    }
}

pub struct SimpleLogger;

impl ComponentLogger for SimpleLogger {
    fn debug(&self, message: &str) {
        eprintln!("[DEBUG] {}", message);
    }

    fn info(&self, message: &str) {
        eprintln!("[INFO] {}", message);
    }

    fn warn(&self, message: &str) {
        eprintln!("[WARN] {}", message);
    }

    fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}

pub struct MemoryDatastore {
    data: Arc<RwLock<HashMap<Vec<u8>, Bytes>>>,
}

impl MemoryDatastore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Datastore for MemoryDatastore {
    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, HeliaError> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn put(&self, key: &[u8], value: Bytes) -> Result<(), HeliaError> {
        let mut data = self.data.write().await;
        data.insert(key.to_vec(), value);
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), HeliaError> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    async fn has(&self, key: &[u8]) -> Result<bool, HeliaError> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn query(&self, _prefix: Option<&[u8]>) -> Result<helia_interface::AwaitIterable<Bytes>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }
}

pub struct HeliaHttp {
    blockstore: Arc<HttpBlocks>,
    datastore: Arc<MemoryDatastore>,
    pins: Arc<HttpPins>,
    routing: Arc<HttpRouting>,
    logger: Arc<SimpleLogger>,
    dns: TokioAsyncResolver,
}

impl HeliaHttp {
    pub fn new() -> Self {
        Self {
            blockstore: Arc::new(HttpBlocks),
            datastore: Arc::new(MemoryDatastore::new()),
            pins: Arc::new(HttpPins),
            routing: Arc::new(HttpRouting),
            logger: Arc::new(SimpleLogger),
            dns: TokioAsyncResolver::tokio_from_system_conf().unwrap(),
        }
    }
}

#[async_trait]
impl Helia for HeliaHttp {
    fn blockstore(&self) -> &dyn Blocks {
        self.blockstore.as_ref()
    }

    fn datastore(&self) -> &dyn Datastore {
        self.datastore.as_ref()
    }

    fn pins(&self) -> &dyn Pins {
        self.pins.as_ref()
    }

    fn logger(&self) -> &dyn ComponentLogger {
        self.logger.as_ref()
    }

    fn routing(&self) -> &dyn Routing {
        self.routing.as_ref()
    }

    fn dns(&self) -> &TokioAsyncResolver {
        &self.dns
    }

    fn metrics(&self) -> Option<&dyn Metrics> {
        None
    }

    async fn start(&self) -> Result<(), HeliaError> {
        self.logger.info("Starting HTTP-only Helia node");
        Ok(())
    }

    async fn stop(&self) -> Result<(), HeliaError> {
        self.logger.info("Stopping HTTP-only Helia node");
        Ok(())
    }

    async fn gc(&self, _options: Option<GcOptions>) -> Result<(), HeliaError> {
        Ok(())
    }

    async fn get_codec(&self, _code: u64) -> Result<Box<dyn Codec>, HeliaError> {
        Err(HeliaError::other("codecs not supported"))
    }

    async fn get_hasher(&self, _code: u64) -> Result<Box<dyn Hasher>, HeliaError> {
        Err(HeliaError::other("hashers not supported"))
    }
}

pub async fn create_helia_http() -> Result<Arc<HeliaHttp>, HeliaError> {
    Ok(Arc::new(HeliaHttp::new()))
}

