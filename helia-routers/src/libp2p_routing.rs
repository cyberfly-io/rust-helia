//! libp2p routing implementation
//!
//! This module provides a Routing implementation that uses libp2p for
//! peer and content discovery via DHT (Kademlia) and other libp2p protocols.

use async_trait::async_trait;
use cid::Cid;
use futures::stream::{self, StreamExt};
use helia_utils::{libp2p_behaviour::HeliaBehaviourEvent, HeliaBehaviour};
use libp2p::{
    kad::{self, GetProvidersOk, RecordKey},
    swarm::SwarmEvent,
    PeerId, Swarm,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, trace, warn};

use helia_interface::{routing::*, AwaitIterable, HeliaError};

/// Result types that can be sent through query result channels
#[derive(Debug, Clone)]
enum QueryResultType {
    Provider(Provider),
    Peer(PeerInfo),
    Record(RoutingRecord),
    Complete,
    Error(String),
}

/// Manages active queries and their result channels
struct QueryManager {
    providers: HashMap<kad::QueryId, mpsc::UnboundedSender<QueryResultType>>,
    peers: HashMap<kad::QueryId, mpsc::UnboundedSender<QueryResultType>>,
    records: HashMap<kad::QueryId, mpsc::UnboundedSender<QueryResultType>>,
}

impl QueryManager {
    fn new() -> Self {
        Self {
            providers: HashMap::new(),
            peers: HashMap::new(),
            records: HashMap::new(),
        }
    }

    fn register_provider_query(
        &mut self,
        query_id: kad::QueryId,
    ) -> mpsc::UnboundedReceiver<QueryResultType> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.providers.insert(query_id, tx);
        rx
    }

    fn register_peer_query(
        &mut self,
        query_id: kad::QueryId,
    ) -> mpsc::UnboundedReceiver<QueryResultType> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.peers.insert(query_id, tx);
        rx
    }

    fn register_record_query(
        &mut self,
        query_id: kad::QueryId,
    ) -> mpsc::UnboundedReceiver<QueryResultType> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.records.insert(query_id, tx);
        rx
    }

    fn handle_kad_event(&mut self, query_id: kad::QueryId, result: kad::QueryResult) {
        trace!("Handling Kademlia event for query {:?}: {:?}", query_id, result);

        match result {
            kad::QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                providers,
                ..
            })) => {
                if let Some(tx) = self.providers.get(&query_id) {
                    for peer_id in providers {
                        let provider = Provider {
                            peer_info: PeerInfo {
                                id: peer_id,
                                multiaddrs: vec![], // Will be populated from identify
                                protocols: vec![],
                            },
                            transport_methods: vec![TransportMethod::Bitswap],
                        };
                        let _ = tx.send(QueryResultType::Provider(provider));
                    }
                }
            }
            kad::QueryResult::GetProviders(Ok(GetProvidersOk::FinishedWithNoAdditionalRecord {
                ..
            })) => {
                if let Some(tx) = self.providers.remove(&query_id) {
                    let _ = tx.send(QueryResultType::Complete);
                }
            }
            kad::QueryResult::GetProviders(Err(e)) => {
                if let Some(tx) = self.providers.remove(&query_id) {
                    let _ = tx.send(QueryResultType::Error(format!("Provider query failed: {:?}", e)));
                }
            }
            kad::QueryResult::GetClosestPeers(Ok(result)) => {
                if let Some(tx) = self.peers.get(&query_id) {
                    for libp2p_peer in result.peers {
                        // Convert libp2p::PeerInfo to helia_interface::PeerInfo
                        let peer_info = PeerInfo {
                            id: libp2p_peer.peer_id,
                            multiaddrs: libp2p_peer.addrs,
                            protocols: vec![], // Not available from GetClosestPeers
                        };
                        let _ = tx.send(QueryResultType::Peer(peer_info));
                    }
                }
                if let Some(tx) = self.peers.remove(&query_id) {
                    let _ = tx.send(QueryResultType::Complete);
                }
            }
            kad::QueryResult::GetClosestPeers(Err(e)) => {
                if let Some(tx) = self.peers.remove(&query_id) {
                    let _ = tx.send(QueryResultType::Error(format!("Peer query failed: {:?}", e)));
                }
            }
            kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))) => {
                if let Some(tx) = self.records.get(&query_id) {
                    let routing_record = RoutingRecord {
                        key: record.record.key.to_vec(),
                        value: record.record.value,
                        time_received: Some(std::time::SystemTime::now()),
                        ttl: None,
                    };
                    let _ = tx.send(QueryResultType::Record(routing_record));
                }
            }
            kad::QueryResult::GetRecord(Err(e)) => {
                if let Some(tx) = self.records.remove(&query_id) {
                    let _ = tx.send(QueryResultType::Error(format!("Record query failed: {:?}", e)));
                }
            }
            kad::QueryResult::PutRecord(Ok(_)) => {
                debug!("Successfully put record for query {:?}", query_id);
            }
            kad::QueryResult::PutRecord(Err(e)) => {
                warn!("Failed to put record for query {:?}: {:?}", query_id, e);
            }
            _ => {
                trace!("Unhandled Kademlia query result: {:?}", result);
            }
        }
    }

    fn cleanup_query(&mut self, query_id: &kad::QueryId) {
        self.providers.remove(query_id);
        self.peers.remove(query_id);
        self.records.remove(query_id);
    }
}

/// libp2p routing implementation using DHT and other libp2p protocols
pub struct Libp2pRouting {
    swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>,
    query_manager: Arc<Mutex<QueryManager>>,
    query_timeout: Duration,
    event_loop_running: Arc<Mutex<bool>>,
}

impl Libp2pRouting {
    /// Create a new libp2p routing instance
    ///
    /// # Arguments
    ///
    /// * `swarm` - The libp2p swarm to use for routing operations
    ///
    /// # Examples
    ///
    /// \`\`\`ignore
    /// use helia_routers::libp2p_routing::Libp2pRouting;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// let swarm = create_swarm().await?;
    /// let routing = Libp2pRouting::new(Arc::new(Mutex::new(swarm)));
    /// \`\`\`
    pub fn new(swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>) -> Self {
        let routing = Self {
            swarm: swarm.clone(),
            query_manager: Arc::new(Mutex::new(QueryManager::new())),
            query_timeout: Duration::from_secs(30),
            event_loop_running: Arc::new(Mutex::new(false)),
        };

        // Start the event loop
        routing.start_event_loop();

        routing
    }

    /// Set the query timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }

    /// Start the background event loop to handle swarm events
    fn start_event_loop(&self) {
        let swarm = self.swarm.clone();
        let query_manager = self.query_manager.clone();
        let event_loop_running = self.event_loop_running.clone();

        tokio::spawn(async move {
            {
                let mut running = event_loop_running.lock().await;
                if *running {
                    // Event loop already running
                    return;
                }
                *running = true;
            }

            debug!("Starting libp2p routing event loop");

            loop {
                let event = {
                    let mut swarm_guard = swarm.lock().await;
                    swarm_guard.select_next_some().await
                };

                trace!("Received swarm event: {:?}", event);

                match event {
                    SwarmEvent::Behaviour(behaviour_event) => {
                        // Handle Kademlia events
                        if let HeliaBehaviourEvent::Kademlia(kad_event) = behaviour_event {
                            match kad_event {
                                kad::Event::OutboundQueryProgressed { id, result, .. } => {
                                    let mut manager = query_manager.lock().await;
                                    manager.handle_kad_event(id, result);
                                }
                                _ => {
                                    trace!("Unhandled Kademlia event: {:?}", kad_event);
                                }
                            }
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        debug!("Connection established with peer: {}", peer_id);
                    }
                    SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                        debug!("Connection closed with peer {}: {:?}", peer_id, cause);
                    }
                    _ => {
                        trace!("Unhandled swarm event");
                    }
                }
            }
        });
    }
}

#[async_trait]
impl helia_interface::Routing for Libp2pRouting {
    async fn find_providers(
        &self,
        cid: &Cid,
        _options: Option<FindProvidersOptions>,
    ) -> Result<AwaitIterable<Provider>, HeliaError> {
        debug!("Finding providers for CID: {}", cid);

        // Convert CID to Kademlia record key
        let multihash = cid.hash().to_bytes();
        let record_key = RecordKey::new(&multihash);

        // Register query and get receiver
        let (query_id, mut rx) = {
            let mut swarm = self.swarm.lock().await;
            let query_id = swarm.behaviour_mut().kademlia.get_providers(record_key);
            
            let mut manager = self.query_manager.lock().await;
            let rx = manager.register_provider_query(query_id);
            
            (query_id, rx)
        };

        debug!("Started provider query with ID: {:?}", query_id);

        // Create a stream that yields providers as they arrive
        let timeout = self.query_timeout;
        let stream = async_stream::stream! {
            let timeout_future = tokio::time::sleep(timeout);
            tokio::pin!(timeout_future);

            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Some(QueryResultType::Provider(provider)) => {
                                trace!("Yielding provider: {:?}", provider.peer_info.id);
                                yield provider;
                            }
                            Some(QueryResultType::Complete) => {
                                debug!("Provider query completed");
                                break;
                            }
                            Some(QueryResultType::Error(e)) => {
                                warn!("Provider query error: {}", e);
                                break;
                            }
                            None => {
                                debug!("Provider query channel closed");
                                break;
                            }
                            _ => {
                                warn!("Unexpected result type for provider query");
                            }
                        }
                    }
                    _ = &mut timeout_future => {
                        warn!("Provider query timed out after {:?}", timeout);
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn provide(&self, cid: &Cid, _options: Option<ProvideOptions>) -> Result<(), HeliaError> {
        debug!("Announcing provider for CID: {}", cid);

        // Convert CID to Kademlia record key
        let multihash = cid.hash().to_bytes();
        let record_key = RecordKey::new(&multihash);

        // Start providing this CID
        let result = {
            let mut swarm = self.swarm.lock().await;
            swarm
                .behaviour_mut()
                .kademlia
                .start_providing(record_key)
        };

        match result {
            Ok(query_id) => {
                debug!("Started provider announcement with query ID: {:?}", query_id);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to start provider announcement: {:?}", e);
                Err(HeliaError::routing(format!(
                    "Failed to announce provider: {:?}",
                    e
                )))
            }
        }
    }

    async fn find_peers(
        &self,
        peer_id: &PeerId,
        _options: Option<FindPeersOptions>,
    ) -> Result<AwaitIterable<PeerInfo>, HeliaError> {
        debug!("Finding peer: {}", peer_id);

        // Check if we're already connected to the peer
        let swarm = self.swarm.lock().await;
        let connected_peers = swarm.connected_peers().collect::<Vec<_>>();

        if connected_peers.contains(&peer_id) {
            // Return peer info from connected peer
            let peer_info = PeerInfo {
                id: *peer_id,
                multiaddrs: vec![], // TODO: Get actual addresses from swarm
                protocols: vec![],  // TODO: Get actual protocols
            };

            drop(swarm);
            return Ok(Box::pin(stream::iter(vec![peer_info])));
        }

        drop(swarm);

        // Register query and get receiver
        let (query_id, mut rx) = {
            let mut swarm = self.swarm.lock().await;
            let query_id = swarm.behaviour_mut().kademlia.get_closest_peers(*peer_id);
            
            let mut manager = self.query_manager.lock().await;
            let rx = manager.register_peer_query(query_id);
            
            (query_id, rx)
        };

        debug!("Started peer query with ID: {:?}", query_id);

        // Create a stream that yields peers as they arrive
        let timeout = self.query_timeout;
        let stream = async_stream::stream! {
            let timeout_future = tokio::time::sleep(timeout);
            tokio::pin!(timeout_future);

            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Some(QueryResultType::Peer(peer_info)) => {
                                trace!("Yielding peer: {:?}", peer_info.id);
                                yield peer_info;
                            }
                            Some(QueryResultType::Complete) => {
                                debug!("Peer query completed");
                                break;
                            }
                            Some(QueryResultType::Error(e)) => {
                                warn!("Peer query error: {}", e);
                                break;
                            }
                            None => {
                                debug!("Peer query channel closed");
                                break;
                            }
                            _ => {
                                warn!("Unexpected result type for peer query");
                            }
                        }
                    }
                    _ = &mut timeout_future => {
                        warn!("Peer query timed out after {:?}", timeout);
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn get(
        &self,
        key: &[u8],
        _options: Option<GetOptions>,
    ) -> Result<Option<RoutingRecord>, HeliaError> {
        debug!("Getting DHT record for key: {:?}", key);

        let record_key = RecordKey::new(&key);

        // Register query and get receiver
        let (query_id, mut rx) = {
            let mut swarm = self.swarm.lock().await;
            let query_id = swarm.behaviour_mut().kademlia.get_record(record_key);
            
            let mut manager = self.query_manager.lock().await;
            let rx = manager.register_record_query(query_id);
            
            (query_id, rx)
        };

        debug!("Started record get query with ID: {:?}", query_id);

        // Wait for the result with timeout
        let timeout = tokio::time::sleep(self.query_timeout);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Some(QueryResultType::Record(record)) => {
                            debug!("Retrieved DHT record");
                            return Ok(Some(record));
                        }
                        Some(QueryResultType::Error(e)) => {
                            warn!("Record query error: {}", e);
                            return Ok(None);
                        }
                        Some(QueryResultType::Complete) | None => {
                            debug!("Record query completed without result");
                            return Ok(None);
                        }
                        _ => {
                            warn!("Unexpected result type for record query");
                        }
                    }
                }
                _ = &mut timeout => {
                    warn!("Record query timed out after {:?}", self.query_timeout);
                    return Ok(None);
                }
            }
        }
    }

    async fn put(
        &self,
        key: &[u8],
        value: &[u8],
        _options: Option<PutOptions>,
    ) -> Result<(), HeliaError> {
        debug!("Putting DHT record for key: {:?}", key);

        let record_key = RecordKey::new(&key);
        let record = kad::Record {
            key: record_key,
            value: value.to_vec(),
            publisher: None,
            expires: None,
        };

        // Put the record into the DHT
        let result = {
            let mut swarm = self.swarm.lock().await;
            swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One)
        };

        match result {
            Ok(query_id) => {
                debug!("Started record put query with ID: {:?}", query_id);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to put DHT record: {:?}", e);
                Err(HeliaError::routing(format!("Failed to put DHT record: {:?}", e)))
            }
        }
    }
}

/// Factory function to create a libp2p routing instance
///
/// This function creates a Routing implementation that uses the provided
/// libp2p swarm for peer and content discovery operations.
///
/// # Arguments
///
/// * `swarm` - The libp2p swarm to use for routing
///
/// # Returns
///
/// A boxed Routing trait object
///
/// # Examples
///
/// \`\`\`ignore
/// use helia_routers::libp2p_routing::libp2p_routing;
/// use std::sync::Arc;
/// use tokio::sync::Mutex;
///
/// let swarm = create_swarm().await?;
/// let routing = libp2p_routing(Arc::new(Mutex::new(swarm)));
///
/// // Use with Helia
/// let helia = HeliaImpl::new(HeliaConfig {
///     // ...
/// }).await?;
/// \`\`\`
pub fn libp2p_routing(swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>) -> Box<dyn helia_interface::Routing> {
    Box::new(Libp2pRouting::new(swarm))
}

#[cfg(test)]
mod tests {
    use super::*;
    use helia_utils::create_swarm;

    #[tokio::test]
    async fn test_libp2p_routing_creation() {
        let swarm = create_swarm().await.expect("Failed to create swarm");
        let routing = Libp2pRouting::new(Arc::new(Mutex::new(swarm)));
        assert_eq!(routing.query_timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_libp2p_routing_with_timeout() {
        let swarm = create_swarm().await.expect("Failed to create swarm");
        let routing = Libp2pRouting::new(Arc::new(Mutex::new(swarm)))
            .with_timeout(Duration::from_secs(60));
        assert_eq!(routing.query_timeout, Duration::from_secs(60));
    }
}
