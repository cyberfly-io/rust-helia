//! IPNS routing interface and implementations

use crate::errors::IpnsError;
use crate::local_store::RecordMetadata;
use async_trait::async_trait;
use libp2p::kad::{Behaviour as Kademlia, store::MemoryStore, Mode, Record as KadRecord, RecordKey, Quorum};
use libp2p::swarm::{Swarm, NetworkBehaviour};
use libp2p::{identity, noise, tcp, yamux, PeerId, Multiaddr};
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    async fn get(
        &self,
        routing_key: &[u8],
        options: GetOptions,
    ) -> Result<Vec<u8>, IpnsError>;

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

    async fn get(
        &self,
        _routing_key: &[u8],
        _options: GetOptions,
    ) -> Result<Vec<u8>, IpnsError> {
        Err(IpnsError::NotFound("Local router doesn't serve records".to_string()))
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
        tracing::info!("Creating DHT router with provided libp2p swarm, peer ID: {}", peer_id);
        
        Self {
            swarm: Arc::new(Mutex::new(swarm)),
            peer_id,
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

        // Put the record into the DHT
        let mut swarm = self.swarm.lock().await;
        let query_id = swarm.behaviour_mut().put_record(record, Quorum::One)
            .map_err(|e| IpnsError::Other(format!("Failed to put record: {}", e)))?;

        tracing::debug!("DHT put initiated with query ID: {:?}", query_id);

        // Release the lock
        drop(swarm);

        // In a real implementation, we'd wait for the query to complete
        // For now, we just return success after initiating the put
        Ok(())
    }

    async fn get(
        &self,
        routing_key: &[u8],
        _options: GetOptions,
    ) -> Result<Vec<u8>, IpnsError> {
        tracing::debug!("DHT router: resolving record from DHT");

        // Convert routing key to RecordKey
        let record_key = RecordKey::new(&routing_key);
        
        // Query the DHT for the record
        let mut swarm = self.swarm.lock().await;
        let query_id = swarm.behaviour_mut().get_record(record_key.clone());

        tracing::debug!("DHT get initiated with query ID: {:?}", query_id);

        // Release the lock
        drop(swarm);

        // In a real implementation, we'd wait for the query to complete
        // and return the found record. For now, we return NotFound.
        // This will be improved with proper event handling.
        Err(IpnsError::NotFound(
            "DHT router: async query initiated, event handling needed".to_string()
        ))
    }

    fn name(&self) -> &str {
        "dht"
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

    async fn get(
        &self,
        _routing_key: &[u8],
        _options: GetOptions,
    ) -> Result<Vec<u8>, IpnsError> {
        // Stub: would query via HTTP
        tracing::debug!("HTTP router ({}): get called (stub)", self.endpoint);
        Err(IpnsError::NotFound("HTTP router is a stub".to_string()))
    }

    fn name(&self) -> &str {
        "http"
    }
}
