//! IPNS routing interface and implementations

use crate::errors::IpnsError;
use crate::local_store::RecordMetadata;
use async_trait::async_trait;
use std::fmt;

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

/// DHT router (stub implementation)
#[derive(Debug)]
pub struct DhtRouter {
    // Would contain DHT client
}

impl DhtRouter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl IpnsRouting for DhtRouter {
    async fn put(
        &self,
        _routing_key: &[u8],
        _marshaled_record: &[u8],
        _options: PutOptions,
    ) -> Result<(), IpnsError> {
        // Stub: would publish to DHT
        tracing::debug!("DHT router: put called (stub)");
        Ok(())
    }

    async fn get(
        &self,
        _routing_key: &[u8],
        _options: GetOptions,
    ) -> Result<Vec<u8>, IpnsError> {
        // Stub: would query DHT
        tracing::debug!("DHT router: get called (stub)");
        Err(IpnsError::NotFound("DHT router is a stub".to_string()))
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
