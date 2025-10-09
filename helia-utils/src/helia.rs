//! Main Helia implementation

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use tokio::sync::{RwLock, Mutex};
use tokio::task::JoinHandle;
use trust_dns_resolver::TokioAsyncResolver;
use futures::stream;
use futures::StreamExt;
use libp2p::{Swarm, swarm::SwarmEvent};

use helia_interface::*;
use helia_interface::pins::Pin as HeliaPin;

use crate::{HeliaConfig, SledBlockstore, SledDatastore, TracingLogger, 
           HeliaBehaviour, create_swarm, BlockstoreWithBitswap};
use crate::libp2p_behaviour::HeliaBehaviourEvent;
use helia_bitswap::{BitswapEvent, Bitswap, BitswapConfig};

/// Main implementation of the Helia trait
pub struct HeliaImpl {
    libp2p: Arc<Mutex<Swarm<HeliaBehaviour>>>,
    blockstore: Arc<dyn Blocks>,
    datastore: Arc<SledDatastore>,
    pins: Arc<SimplePins>,
    logger: Arc<TracingLogger>,
    routing: Arc<DummyRouting>,
    dns: TokioAsyncResolver,
    metrics: Option<Arc<dyn Metrics>>,
    started: Arc<RwLock<bool>>,
    event_loop_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    bitswap: Arc<Bitswap>,
    outbound_rx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<helia_bitswap::coordinator::OutboundMessage>>>>,
}

impl HeliaImpl {
    pub async fn new(mut config: HeliaConfig) -> Result<Self, HeliaError> {
        // Create base infrastructure
        let local_blockstore = Arc::new(SledBlockstore::new(config.blockstore)?);
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

        // Create Bitswap coordinator
        let bitswap_config = BitswapConfig::default();
        let mut bitswap = Bitswap::new(local_blockstore.clone() as Arc<dyn Blocks>, bitswap_config)
                .await
                .map_err(|e| HeliaError::network(format!("Failed to create Bitswap: {}", e)))?;

        // Create channel for outbound Bitswap messages
        let (outbound_tx, outbound_rx) = tokio::sync::mpsc::unbounded_channel();
        bitswap.set_outbound_sender(outbound_tx);
        logger.info("Bitswap outbound message channel created");

        let bitswap = Arc::new(bitswap);

        // Connect Bitswap coordinator to the NetworkBehaviour
        // This allows the behaviour to respond to incoming WANT requests
        {
            let mut swarm_guard = libp2p.lock().await;
            swarm_guard.behaviour_mut().bitswap.set_coordinator(bitswap.clone());
            logger.info("Bitswap coordinator connected to NetworkBehaviour");
        }

        // Wrap blockstore with Bitswap integration for network retrieval
        let blockstore: Arc<dyn Blocks> = Arc::new(BlockstoreWithBitswap::new(
            local_blockstore,
            bitswap.clone(),
        ));

        logger.info("Helia node initialized with Bitswap P2P support");

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
            event_loop_handle: Arc::new(Mutex::new(None)),
            bitswap,
            outbound_rx: Arc::new(Mutex::new(Some(outbound_rx))),
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

        // Start Bitswap coordinator
        self.bitswap.start().await
            .map_err(|e| HeliaError::network(format!("Failed to start Bitswap: {}", e)))?;
        self.logger.info("Bitswap coordinator started");

        // Start libp2p swarm
        let mut swarm = self.libp2p.lock().await;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .map_err(|e| HeliaError::network(format!("Failed to start listening: {}", e)))?;
        drop(swarm); // Release lock before spawning event loop

        // Start swarm event loop
        let swarm_clone = self.libp2p.clone();
        let blockstore_clone = self.blockstore.clone();
        let logger_clone = self.logger.clone();
        let bitswap_clone = self.bitswap.clone();
        
        // Take the outbound_rx channel (only available once)
        let outbound_rx = self.outbound_rx.lock().await.take()
            .ok_or_else(|| HeliaError::other("Bitswap outbound channel already taken"))?;
        
        let handle = tokio::spawn(async move {
            run_swarm_event_loop(swarm_clone, blockstore_clone, logger_clone, bitswap_clone, outbound_rx).await;
        });
        
        *self.event_loop_handle.lock().await = Some(handle);

        self.logger.info("Helia node started");
        *started = true;
        Ok(())
    }

    async fn stop(&self) -> Result<(), HeliaError> {
        let mut started = self.started.write().await;
        if !*started {
            return Ok(());
        }

        // Stop event loop
        if let Some(handle) = self.event_loop_handle.lock().await.take() {
            handle.abort();
        }

        // Stop Bitswap coordinator
        self.bitswap.stop().await
            .map_err(|e| HeliaError::network(format!("Failed to stop Bitswap: {}", e)))?;
        self.logger.info("Bitswap coordinator stopped");

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

/// Run the libp2p swarm event loop
async fn run_swarm_event_loop(
    swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>,
    blockstore: Arc<dyn Blocks>,
    logger: Arc<TracingLogger>,
    bitswap: Arc<Bitswap>,
    mut outbound_rx: tokio::sync::mpsc::UnboundedReceiver<helia_bitswap::coordinator::OutboundMessage>,
) {
    loop {
        tokio::select! {
            // Handle swarm events
            event = async {
                let mut swarm_guard = swarm.lock().await;
                swarm_guard.select_next_some().await
            } => {
                match event {
                    SwarmEvent::Behaviour(behaviour_event) => {
                        // Handle different behaviour events
                        // The NetworkBehaviour derive macro generates HeliaBehaviourEvent
                        match behaviour_event {
                            HeliaBehaviourEvent::Bitswap(bitswap_event) => {
                                handle_bitswap_event(bitswap_event, blockstore.clone(), bitswap.clone(), logger.clone()).await;
                            }
                            HeliaBehaviourEvent::Identify(identify_event) => {
                                logger.debug(&format!("Identify event: {:?}", identify_event));
                            }
                            HeliaBehaviourEvent::Kademlia(kad_event) => {
                                logger.debug(&format!("Kademlia event: {:?}", kad_event));
                            }
                    HeliaBehaviourEvent::Gossipsub(gossip_event) => {
                        logger.debug(&format!("Gossipsub event: {:?}", gossip_event));
                    }
                    HeliaBehaviourEvent::Mdns(mdns_event) => {
                        use libp2p::mdns;
                        match mdns_event {
                            mdns::Event::Discovered(list) => {
                                for (peer_id, multiaddr) in list {
                                                                       logger.info(&format!("mDNS discovered peer: {} at {}", peer_id, multiaddr));
                                    // Dial the discovered peer to establish connection
                                    let mut swarm_guard = swarm.lock().await;
                                    if let Err(e) = swarm_guard.dial(multiaddr.clone()) {
                                        logger.warn(&format!("Failed to dial discovered peer {}: {}", peer_id, e));
                                    } else {
                                        logger.info(&format!("Dialing discovered peer: {}", peer_id));
                                    }
                                }
                            }
                            mdns::Event::Expired(list) => {
                                for (peer_id, _multiaddr) in list {
                                    logger.info(&format!("mDNS peer expired: {}", peer_id));
                                }
                            }
                        }
                    }
                    _ => {
                        // Handle other protocol events
                        logger.debug(&format!("Other behaviour event: {:?}", behaviour_event));
                    }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                logger.info(&format!("Listening on {}", address));
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                logger.info(&format!("Connection established with peer: {} at {}", peer_id, endpoint.get_remote_address()));
                // Notify Bitswap coordinator of new peer
                bitswap.add_peer(peer_id).await;
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                logger.info(&format!("Connection closed with peer: {} (cause: {:?})", peer_id, cause));
                // Notify Bitswap coordinator of disconnected peer
                bitswap.remove_peer(&peer_id).await;
            }
            SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                logger.debug(&format!("Incoming connection from {} to {}", send_back_addr, local_addr));
            }
            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                logger.warn(&format!("Incoming connection error from {} to {}: {}", send_back_addr, local_addr, error));
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    logger.warn(&format!("Outgoing connection error to {}: {}", peer_id, error));
                } else {
                    logger.warn(&format!("Outgoing connection error: {}", error));
                }
            }
            _ => {
                // Handle other events as needed
            }
        }
            }
            
            // Handle outbound Bitswap messages from coordinator
            Some(outbound_msg) = outbound_rx.recv() => {
                logger.debug(&format!("Sending Bitswap message to peer {} via swarm", outbound_msg.peer));
                let mut swarm_guard = swarm.lock().await;
                swarm_guard.behaviour_mut().bitswap.send_message(outbound_msg.peer, outbound_msg.message);
            }
        }
    }
}

/// Handle Bitswap events (MessageReceived, MessageSent, SendError)
async fn handle_bitswap_event(
    event: BitswapEvent,
    blockstore: Arc<dyn Blocks>,
    bitswap: Arc<Bitswap>,
    logger: Arc<TracingLogger>,
) {
    match event {
        BitswapEvent::MessageReceived { peer, message } => {
            logger.info(&format!("Received Bitswap message from peer: {}", peer));
            
            // Store any blocks that were received
            if !message.blocks.is_empty() {
                logger.info(&format!("Received {} blocks from peer {}", message.blocks.len(), peer));
                
                for block in message.blocks {
                    logger.debug(&format!(
                        "Block received - prefix_len: {}, data_len: {}",
                        block.prefix.len(),
                        block.data.len()
                    ));
                    
                    // Decode CID from prefix and data
                    // The prefix contains: [version, codec, ...]
                    // For now, we'll reconstruct the CID from the block data
                    // In Bitswap, the full CID can be reconstructed by hashing the data
                    match reconstruct_cid_from_block(&block.prefix, &block.data) {
                        Ok(cid) => {
                            logger.info(&format!("Storing received block: {}", cid));
                            
                            // Store in blockstore
                            if let Err(e) = blockstore.put(&cid, bytes::Bytes::from(block.data), None).await {
                                logger.warn(&format!("Failed to store received block {}: {}", cid, e));
                            } else {
                                logger.info(&format!("✅ Successfully stored block: {}", cid));
                                
                                // **OPTIMIZATION**: Immediately notify bitswap coordinator
                                // This wakes up any waiting want() calls (event-driven, not polling)
                                bitswap.notify_block_received(&cid);
                                logger.debug(&format!("Notified coordinator of block arrival: {}", cid));
                            }
                        }
                        Err(e) => {
                            logger.warn(&format!("Failed to decode CID from block prefix: {}", e));
                        }
                    }
                }
            }
            
            // Log wantlist if present
            if let Some(wantlist) = &message.wantlist {
                logger.debug(&format!(
                    "Received wantlist with {} entries (full: {})",
                    wantlist.entries.len(),
                    wantlist.full
                ));
                
                // TODO: Process wantlist and send blocks if we have them
                // This will be implemented when we connect the coordinator
            }
            
            // Log block presences if present
            if !message.block_presences.is_empty() {
                logger.debug(&format!(
                    "Received {} block presence notifications",
                    message.block_presences.len()
                ));
            }
        }
        BitswapEvent::MessageSent { peer } => {
            logger.debug(&format!("Successfully sent Bitswap message to peer: {}", peer));
        }
        BitswapEvent::SendError { peer, error } => {
            logger.warn(&format!("Failed to send Bitswap message to peer {}: {}", peer, error));
        }
    }
}

/// Reconstruct CID from Bitswap block prefix and data
/// 
/// In our implementation, the prefix contains the full CID bytes,
/// which allows us to get the exact CID without needing to re-hash.
fn reconstruct_cid_from_block(prefix: &[u8], _data: &[u8]) -> Result<cid::Cid, HeliaError> {
    // Parse CID directly from prefix bytes
    cid::Cid::try_from(prefix)
        .map_err(|e| HeliaError::network(format!("Failed to parse CID from prefix: {}", e)))
}
