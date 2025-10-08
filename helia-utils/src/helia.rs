//! Main Helia implementation

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use tokio::sync::{RwLock, Mutex};
use trust_dns_resolver::TokioAsyncResolver;
use futures::stream;
use libp2p::Swarm;

use helia_interface::*;
use helia_interface::pins::Pin as HeliaPin;

use crate::{HeliaConfig, SledBlockstore, SledDatastore, TracingLogger, 
           HeliaBehaviour, create_swarm};

/// Main implementation of the Helia trait
pub struct HeliaImpl {
    libp2p: Arc<Mutex<Swarm<HeliaBehaviour>>>,
    blockstore: Arc<SledBlockstore>,
    datastore: Arc<SledDatastore>,
    pins: Arc<SimplePins>,
    logger: Arc<TracingLogger>,
    routing: Arc<DummyRouting>,
    dns: TokioAsyncResolver,
    metrics: Option<Arc<dyn Metrics>>,
    started: Arc<RwLock<bool>>,
}

impl HeliaImpl {
    pub async fn new(mut config: HeliaConfig) -> Result<Self, HeliaError> {
        let blockstore = Arc::new(SledBlockstore::new(config.blockstore)?);
        let datastore = Arc::new(SledDatastore::new(config.datastore)?);
        let pins = Arc::new(SimplePins::new(datastore.clone()));
        let logger = Arc::new(TracingLogger::new(config.logger));
        let routing = Arc::new(DummyRouting::new());
        
        // Use provided libp2p swarm or create a new one
        let libp2p = if let Some(swarm) = config.libp2p.take() {
            swarm
        } else {
            let swarm = create_swarm().await
                .map_err(|e| HeliaError::network(format!("Failed to create libp2p swarm: {}", e)))?;
            Arc::new(Mutex::new(swarm))
        };
        
        let dns = config.dns.unwrap_or_else(|| {
            TokioAsyncResolver::tokio_from_system_conf().expect("Failed to create DNS resolver")
        });

        Ok(Self {
            libp2p,
            blockstore,
            datastore,
            pins,
            logger,
            routing,
            dns,
            metrics: config.metrics,
            started: Arc::new(RwLock::new(false)),
        })
    }
}

#[async_trait]
impl Helia for HeliaImpl {
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
        self.metrics.as_ref().map(|m| m.as_ref())
    }

    async fn start(&self) -> Result<(), HeliaError> {
        let mut started = self.started.write().await;
        if *started {
            return Ok(());
        }

        // Start libp2p swarm
        let mut swarm = self.libp2p.lock().await;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .map_err(|e| HeliaError::network(format!("Failed to start listening: {}", e)))?;

        self.logger.info("Helia node started");
        *started = true;
        Ok(())
    }

    async fn stop(&self) -> Result<(), HeliaError> {
        let mut started = self.started.write().await;
        if !*started {
            return Ok(());
        }

        self.logger.info("Helia node stopped");
        *started = false;
        Ok(())
    }    async fn gc(&self, _options: Option<GcOptions>) -> Result<(), HeliaError> {
        // TODO: Implement garbage collection
        self.logger.info("Garbage collection not yet implemented");
        Ok(())
    }

    async fn get_codec(&self, code: u64) -> Result<Box<dyn Codec>, HeliaError> {
        // TODO: Implement codec loading
        Err(HeliaError::CodecNotFound { code })
    }

    async fn get_hasher(&self, code: u64) -> Result<Box<dyn Hasher>, HeliaError> {
        // TODO: Implement hasher loading  
        Err(HeliaError::HasherNotFound { code })
    }
}

#[async_trait]
impl HeliaWithLibp2p<HeliaBehaviour> for HeliaImpl {
    fn libp2p(&self) -> Arc<Mutex<Swarm<HeliaBehaviour>>> {
        self.libp2p.clone()
    }
}

/*
/// Dummy libp2p implementation for now - DEPRECATED: Now using real libp2p
pub struct DummyLibp2p {
    started: Arc<RwLock<bool>>,
}

impl DummyLibp2p {
    pub fn new() -> Self {
        Self {
            started: Arc::new(RwLock::new(false)),
        }
    }
}

#[async_trait]
impl Libp2p for DummyLibp2p {
    fn is_started(&self) -> bool {
        false
    }

    fn peer_id(&self) -> libp2p::PeerId {
        libp2p::PeerId::random()
    }

    fn listeners(&self) -> Vec<libp2p::Multiaddr> {
        vec![]
    }

    async fn start(&self) -> Result<(), HeliaError> {
        let mut started = self.started.write().await;
        *started = true;
        Ok(())
    }

    async fn stop(&self) -> Result<(), HeliaError> {
        let mut started = self.started.write().await;
        *started = false;
        Ok(())
    }
}
*/

/// Dummy routing implementation
pub struct DummyRouting;

impl DummyRouting {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Routing for DummyRouting {
    async fn find_providers(
        &self,
        _cid: &Cid,
        _options: Option<FindProvidersOptions>,
    ) -> Result<AwaitIterable<Provider>, HeliaError> {
        Err(HeliaError::routing("Routing not yet implemented"))
    }

    async fn provide(&self, _cid: &Cid, _options: Option<ProvideOptions>) -> Result<(), HeliaError> {
        Err(HeliaError::routing("Routing not yet implemented"))
    }

    async fn find_peers(
        &self,
        _peer_id: &libp2p::PeerId,
        _options: Option<FindPeersOptions>,
    ) -> Result<AwaitIterable<PeerInfo>, HeliaError> {
        Err(HeliaError::routing("Routing not yet implemented"))
    }

    async fn get(
        &self,
        _key: &[u8],
        _options: Option<GetOptions>,
    ) -> Result<Option<RoutingRecord>, HeliaError> {
        Err(HeliaError::routing("Routing not yet implemented"))
    }

    async fn put(
        &self,
        _key: &[u8],
        _value: &[u8],
        _options: Option<PutOptions>,
    ) -> Result<(), HeliaError> {
        Err(HeliaError::routing("Routing not yet implemented"))
    }
}

/// Simple pins implementation  
pub struct SimplePins {
    datastore: Arc<dyn Datastore>,
}

impl SimplePins {
    pub fn new(datastore: Arc<dyn Datastore>) -> Self {
        Self { datastore }
    }

    fn pin_key(&self, cid: &Cid) -> Vec<u8> {
        format!("pin:{}", cid).into_bytes()
    }

    fn pin_to_bytes(&self, pin: &HeliaPin) -> Result<Bytes, HeliaError> {
        serde_json::to_vec(pin)
            .map(Bytes::from)
            .map_err(|e| HeliaError::other(format!("Failed to serialize pin: {}", e)))
    }

    fn bytes_to_pin(&self, data: &[u8]) -> Result<HeliaPin, HeliaError> {
        serde_json::from_slice(data)
            .map_err(|e| HeliaError::other(format!("Failed to deserialize pin: {}", e)))
    }
}

#[async_trait]
impl Pins for SimplePins {
    async fn add(&self, cid: &Cid, options: Option<AddOptions>) -> Result<(), HeliaError> {
        let options = options.unwrap_or_default();
        
        let pin = HeliaPin {
            cid: *cid,
            depth: options.depth.unwrap_or(u64::MAX), // Default to recursive (infinite depth)
            metadata: options.metadata,
        };

        let key = self.pin_key(cid);
        let value = self.pin_to_bytes(&pin)?;
        
        self.datastore.put(&key, value).await?;
        Ok(())
    }

    async fn rm(&self, cid: &Cid, _options: Option<RmOptions>) -> Result<(), HeliaError> {
        let key = self.pin_key(cid);
        self.datastore.delete(&key).await?;
        Ok(())
    }

    async fn ls(&self, options: Option<LsOptions>) -> Result<AwaitIterable<HeliaPin>, HeliaError> {
        let options = options.unwrap_or_default();
        
        // If filtering by specific CID
        if let Some(filter_cid) = options.cid {
            let key = self.pin_key(&filter_cid);
            match self.datastore.get(&key).await? {
                Some(data) => {
                    let pin = self.bytes_to_pin(&data)?;
                    Ok(Box::pin(stream::iter(vec![pin])))
                }
                None => Ok(Box::pin(stream::iter(vec![]))),
            }
        } else {
            // List all pins - get all entries with "pin:" prefix
            let mut pins = Vec::new();
            let mut query_stream = self.datastore.query(Some(b"pin:")).await?;
            
            use futures::StreamExt;
            while let Some(data) = query_stream.next().await {
                match self.bytes_to_pin(&data) {
                    Ok(pin) => pins.push(pin),
                    Err(_) => continue, // Skip invalid pin entries
                }
            }
            
            Ok(Box::pin(stream::iter(pins)))
        }
    }

    async fn is_pinned(&self, cid: &Cid, _options: Option<IsPinnedOptions>) -> Result<bool, HeliaError> {
        let key = self.pin_key(cid);
        self.datastore.has(&key).await
    }
}