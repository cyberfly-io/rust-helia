//! IPNS routing interface and implementations

use crate::errors::IpnsError;
use crate::local_store::RecordMetadata;
use async_trait::async_trait;
use libp2p::kad::{
    store::MemoryStore, Behaviour as Kademlia, Mode, QueryId, Quorum, Record as KadRecord,
    RecordKey,
};
use libp2p::swarm::{NetworkBehaviour, Swarm};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, PeerId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// Progress/event tracking for routing operations
#[derive(Debug, Clone)]
pub enum RoutingEvent {
    /// Starting a put operation
    PutStart,
    /// Put operation succeeded
    PutSuccess,
    /// Put operation failed
    PutError(String),
    /// Starting a get operation
    GetStart,
    /// Get operation succeeded
    GetSuccess,
    /// Get operation failed
    GetError(String),
}

/// Result type for DHT query operations
#[derive(Debug, Clone)]
enum DhtQueryResult {
    /// Put operation completed
    PutComplete,
    /// Get operation completed with record data
    GetComplete(Vec<u8>),
    /// Query failed
    Error(String),
}

/// Manager for tracking ongoing DHT queries
struct DhtQueryManager {
    /// Mapping from QueryId to result sender
    pending: HashMap<QueryId, mpsc::UnboundedSender<DhtQueryResult>>,
}

impl DhtQueryManager {
    fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Register a new query and return the receiver for results
    fn register_query(&mut self, query_id: QueryId) -> mpsc::UnboundedReceiver<DhtQueryResult> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.pending.insert(query_id, tx);
        rx
    }

    /// Send a result for a query
    fn complete_query(&mut self, query_id: &QueryId, result: DhtQueryResult) {
        if let Some(tx) = self.pending.remove(query_id) {
            let _ = tx.send(result);
        }
    }

    /// Check if a query is pending
    fn has_query(&self, query_id: &QueryId) -> bool {
        self.pending.contains_key(query_id)
    }
}

/// Options for putting IPNS records
#[derive(Debug, Clone, Default)]
pub struct PutOptions {
    /// Metadata about the record being published
    pub metadata: Option<RecordMetadata>,
}

/// Options for getting IPNS records
#[derive(Debug, Clone, Default)]
pub struct GetOptions {
    /// Whether to validate the record (default: true)
    pub validate: bool,
}

/// IPNS routing trait
///
/// Implementations provide different strategies for publishing and resolving
/// IPNS records (e.g., DHT, HTTP, PubSub)
#[async_trait]
pub trait IpnsRouting: Send + Sync + fmt::Debug {
    /// Put an IPNS record to the routing system
    ///
    /// # Arguments
    /// * `routing_key` - The routing key (derived from public key multihash)
    /// * `marshaled_record` - The serialized IPNS record
    /// * `options` - Publishing options including metadata
    async fn put(
        &self,
        routing_key: &[u8],
        marshaled_record: &[u8],
        options: PutOptions,
    ) -> Result<(), IpnsError>;

    /// Get an IPNS record from the routing system
    ///
    /// # Arguments
    /// * `routing_key` - The routing key to look up
    /// * `options` - Resolution options (validation, etc.)
    async fn get(&self, routing_key: &[u8], options: GetOptions) -> Result<Vec<u8>, IpnsError>;

    /// Get a human-readable name for this router (for debugging)
    fn name(&self) -> &str;
}

/// Local-only router (stores records only in local datastore)
#[derive(Debug)]
pub struct LocalRouter {
    // This would normally contain a reference to the local store
    // For now, it's a stub
}

impl LocalRouter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl IpnsRouting for LocalRouter {
    async fn put(
        &self,
        _routing_key: &[u8],
        _marshaled_record: &[u8],
        _options: PutOptions,
    ) -> Result<(), IpnsError> {
        // This is handled by the local store directly
        Ok(())
    }

    async fn get(&self, _routing_key: &[u8], _options: GetOptions) -> Result<Vec<u8>, IpnsError> {
        Err(IpnsError::NotFound(
            "Local router doesn't serve records".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "local"
    }
}

/// DHT router for IPNS record distribution via libp2p Kademlia DHT
///
/// This router accepts a libp2p swarm instance, following the Helia pattern
/// where the user manages their own libp2p configuration.
pub struct DhtRouter {
    /// The libp2p swarm with Kademlia behaviour
    /// Users must provide a swarm configured with Kademlia
    swarm: Arc<Mutex<Swarm<Kademlia<MemoryStore>>>>,
    /// Local peer ID
    peer_id: PeerId,
    /// Query manager for tracking DHT operations
    query_manager: Arc<Mutex<DhtQueryManager>>,
}

impl fmt::Debug for DhtRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DhtRouter")
            .field("peer_id", &self.peer_id)
            .finish()
    }
}

impl DhtRouter {
    /// Create a new DHT router with an existing libp2p swarm
    ///
    /// This follows the Helia pattern where users manage their own libp2p instance.
    /// The swarm must be configured with a Kademlia behaviour.
    ///
    /// # Arguments
    /// * `swarm` - A libp2p swarm configured with Kademlia DHT
    /// * `peer_id` - The local peer ID
    ///
    /// # Example
    /// ```ignore
    /// // User creates and configures their own libp2p swarm
    /// let keypair = Keypair::generate_ed25519();
    /// let peer_id = PeerId::from(keypair.public());
    ///
    /// let store = MemoryStore::new(peer_id);
    /// let kad = Kademlia::new(peer_id, store);
    ///
    /// let swarm = SwarmBuilder::with_existing_identity(keypair)
    ///     .with_tokio()
    ///     .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default())?
    ///     .with_behaviour(|_| kad)?
    ///     .build();
    ///
    /// // Pass the swarm to DhtRouter
    /// let router = DhtRouter::new(swarm, peer_id);
    /// ```
    pub fn new(swarm: Swarm<Kademlia<MemoryStore>>, peer_id: PeerId) -> Self {
        tracing::info!(
            "Creating DHT router with provided libp2p swarm, peer ID: {}",
            peer_id
        );

        Self {
            swarm: Arc::new(Mutex::new(swarm)),
            peer_id,
            query_manager: Arc::new(Mutex::new(DhtQueryManager::new())),
        }
    }

    /// Get the local peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get a reference to the swarm for advanced operations
    ///
    /// This allows users to interact with the underlying libp2p swarm
    /// for operations not directly supported by the router
    pub fn swarm(&self) -> Arc<Mutex<Swarm<Kademlia<MemoryStore>>>> {
        Arc::clone(&self.swarm)
    }

    /// Create an IPNS routing key from a peer ID
    /// Format: /ipns/<peer-id>
    fn create_routing_key(peer_id: &PeerId) -> Vec<u8> {
        format!("/ipns/{}", peer_id).into_bytes()
    }
}

#[async_trait]
impl IpnsRouting for DhtRouter {
    async fn put(
        &self,
        routing_key: &[u8],
        marshaled_record: &[u8],
        _options: PutOptions,
    ) -> Result<(), IpnsError> {
        tracing::debug!("DHT router: publishing record to DHT");

        // Convert routing key to RecordKey
        let record_key = RecordKey::new(&routing_key);

        // Create a Kademlia record
        let record = KadRecord {
            key: record_key.clone(),
            value: marshaled_record.to_vec(),
            publisher: Some(self.peer_id),
            expires: None, // IPNS records handle their own expiry
        };

        // Register query and get receiver for results
        let mut query_manager = self.query_manager.lock().await;
        let query_id = {
            let mut swarm = self.swarm.lock().await;
            swarm
                .behaviour_mut()
                .put_record(record, Quorum::One)
                .map_err(|e| IpnsError::Other(format!("Failed to put record: {}", e)))?
        };
        
        let mut result_rx = query_manager.register_query(query_id);
        drop(query_manager);

        tracing::debug!("DHT put initiated with query ID: {:?}", query_id);

        // Wait for the query to complete with timeout
        let timeout = Duration::from_secs(30);
        match tokio::time::timeout(timeout, result_rx.recv()).await {
            Ok(Some(DhtQueryResult::PutComplete)) => {
                tracing::debug!("DHT put completed successfully");
                Ok(())
            }
            Ok(Some(DhtQueryResult::Error(e))) => {
                tracing::warn!("DHT put failed: {}", e);
                Err(IpnsError::PublishFailed(e))
            }
            Ok(None) => {
                tracing::warn!("DHT put channel closed unexpectedly");
                Err(IpnsError::PublishFailed(
                    "Query result channel closed".to_string(),
                ))
            }
            Err(_) => {
                tracing::warn!("DHT put timed out after {} seconds", timeout.as_secs());
                Err(IpnsError::Timeout)
            }
            _ => Err(IpnsError::Other("Unexpected query result".to_string())),
        }
    }

    async fn get(&self, routing_key: &[u8], _options: GetOptions) -> Result<Vec<u8>, IpnsError> {
        tracing::debug!("DHT router: resolving record from DHT");

        // Convert routing key to RecordKey
        let record_key = RecordKey::new(&routing_key);

        // Register query and get receiver for results
        let mut query_manager = self.query_manager.lock().await;
        let query_id = {
            let mut swarm = self.swarm.lock().await;
            swarm.behaviour_mut().get_record(record_key.clone())
        };
        
        let mut result_rx = query_manager.register_query(query_id);
        drop(query_manager);

        tracing::debug!("DHT get initiated with query ID: {:?}", query_id);

        // Wait for the query to complete with timeout
        let timeout = Duration::from_secs(30);
        match tokio::time::timeout(timeout, result_rx.recv()).await {
            Ok(Some(DhtQueryResult::GetComplete(data))) => {
                tracing::debug!("DHT get completed successfully, got {} bytes", data.len());
                Ok(data)
            }
            Ok(Some(DhtQueryResult::Error(e))) => {
                tracing::warn!("DHT get failed: {}", e);
                Err(IpnsError::NotFound(e))
            }
            Ok(None) => {
                tracing::warn!("DHT get channel closed unexpectedly");
                Err(IpnsError::NotFound(
                    "Query result channel closed".to_string(),
                ))
            }
            Err(_) => {
                tracing::warn!("DHT get timed out after {} seconds", timeout.as_secs());
                Err(IpnsError::Timeout)
            }
            _ => Err(IpnsError::NotFound("Unexpected query result".to_string())),
        }
    }

    fn name(&self) -> &str {
        "dht"
    }
}

impl DhtRouter {
    /// Handle Kademlia events and complete DHT queries
    ///
    /// This should be called from the swarm event loop when Kademlia events occur.
    /// It processes query results and notifies waiting operations.
    ///
    /// # Example
    /// ```ignore
    /// // In swarm event loop
    /// match event {
    ///     SwarmEvent::Behaviour(kad::Event::OutboundQueryProgressed { id, result, .. }) => {
    ///         dht_router.handle_kad_event(id, result).await;
    ///     }
    ///     _ => {}
    /// }
    /// ```
    pub async fn handle_kad_event(&self, query_id: QueryId, result: libp2p::kad::QueryResult) {
        let mut query_manager = self.query_manager.lock().await;

        // Check if this is one of our queries
        if !query_manager.has_query(&query_id) {
            return;
        }

        use libp2p::kad::QueryResult::*;

        match result {
            PutRecord(Ok(_)) => {
                tracing::debug!("DHT put query {:?} succeeded", query_id);
                query_manager.complete_query(&query_id, DhtQueryResult::PutComplete);
            }
            PutRecord(Err(e)) => {
                tracing::warn!("DHT put query {:?} failed: {:?}", query_id, e);
                query_manager.complete_query(
                    &query_id,
                    DhtQueryResult::Error(format!("Put failed: {:?}", e)),
                );
            }
            GetRecord(Ok(ok)) => {
                tracing::debug!("DHT get query {:?} succeeded", query_id);
                // Get the record value from the result
                match ok {
                    libp2p::kad::GetRecordOk::FoundRecord(record) => {
                        query_manager.complete_query(
                            &query_id,
                            DhtQueryResult::GetComplete(record.record.value.clone()),
                        );
                    }
                    libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {
                        query_manager.complete_query(
                            &query_id,
                            DhtQueryResult::Error("No additional records found".to_string()),
                        );
                    }
                }
            }
            GetRecord(Err(e)) => {
                tracing::warn!("DHT get query {:?} failed: {:?}", query_id, e);
                query_manager.complete_query(
                    &query_id,
                    DhtQueryResult::Error(format!("Get failed: {:?}", e)),
                );
            }
            _ => {
                // Other query types are not tracked
                tracing::trace!("DHT query {:?} completed with untracked result type", query_id);
            }
        }
    }
}

/// HTTP router for delegated routing (stub implementation)
#[derive(Debug)]
pub struct HttpRouter {
    endpoint: String,
}

impl HttpRouter {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

#[async_trait]
impl IpnsRouting for HttpRouter {
    async fn put(
        &self,
        _routing_key: &[u8],
        _marshaled_record: &[u8],
        _options: PutOptions,
    ) -> Result<(), IpnsError> {
        // Stub: would publish via HTTP
        tracing::debug!("HTTP router ({}): put called (stub)", self.endpoint);
        Ok(())
    }

    async fn get(&self, _routing_key: &[u8], _options: GetOptions) -> Result<Vec<u8>, IpnsError> {
        // Stub: would query via HTTP
        tracing::debug!("HTTP router ({}): get called (stub)", self.endpoint);
        Err(IpnsError::NotFound("HTTP router is a stub".to_string()))
    }

    fn name(&self) -> &str {
        "http"
    }
}
