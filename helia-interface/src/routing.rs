//! Routing interface for peer and content discovery

use std::time::Duration;

use async_trait::async_trait;
use cid::Cid;
use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};

use crate::{AbortOptions, AwaitIterable, HeliaError, ProgressOptions};

/// Options for routing operations
#[derive(Debug, Clone)]
pub struct RoutingOptions {
    /// Abort options
    pub abort: AbortOptions,
    /// Progress options
    pub progress: ProgressOptions<()>,
    /// Whether to use the network for lookups (default: true)
    pub use_network: bool,
    /// Whether to use cached values (default: true)
    pub use_cache: bool,
    /// Whether to perform validation (default: true)
    pub validate: bool,
}

impl Default for RoutingOptions {
    fn default() -> Self {
        Self {
            abort: AbortOptions::default(),
            progress: ProgressOptions::default(),
            use_network: true,
            use_cache: true,
            validate: true,
        }
    }
}

/// Information about a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// The peer's ID
    pub id: PeerId,
    /// Known multiaddresses for the peer
    pub multiaddrs: Vec<Multiaddr>,
    /// Protocols supported by the peer
    pub protocols: Vec<String>,
}

/// A provider can supply content for a CID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Peer information
    #[serde(flatten)]
    pub peer_info: PeerInfo,
    /// Transport methods available for retrieving content
    pub transport_methods: Vec<TransportMethod>,
}

/// Methods available for content transport
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransportMethod {
    /// Content available over Bitswap protocol
    Bitswap,
    /// Content available over HTTP
    Http,
    /// Content available over libp2p streams
    Libp2pStream,
    /// Custom transport method
    Custom(String),
}

/// Options for finding providers
#[derive(Debug, Clone, Default)]
pub struct FindProvidersOptions {
    /// Routing options
    pub routing: RoutingOptions,
}

/// Options for providing content
#[derive(Debug, Clone, Default)]
pub struct ProvideOptions {
    /// Routing options
    pub routing: RoutingOptions,
}

/// Options for finding peers
#[derive(Debug, Clone, Default)]
pub struct FindPeersOptions {
    /// Routing options
    pub routing: RoutingOptions,
}

/// Options for getting peer info
#[derive(Debug, Clone, Default)]
pub struct GetOptions {
    /// Routing options
    pub routing: RoutingOptions,
}

/// Options for putting records
#[derive(Debug, Clone, Default)]
pub struct PutOptions {
    /// Routing options
    pub routing: RoutingOptions,
}

/// A routing record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecord {
    /// The record key
    pub key: Vec<u8>,
    /// The record value
    pub value: Vec<u8>,
    /// When the record was created
    pub time_received: Option<std::time::SystemTime>,
    /// TTL for the record
    pub ttl: Option<Duration>,
}

/// Routing interface for peer and content discovery
#[async_trait]
pub trait Routing: Send + Sync {
    /// Find providers for a given CID
    async fn find_providers(
        &self,
        cid: &Cid,
        options: Option<FindProvidersOptions>,
    ) -> Result<AwaitIterable<Provider>, HeliaError>;

    /// Announce that this node can provide content for a CID
    async fn provide(&self, cid: &Cid, options: Option<ProvideOptions>) -> Result<(), HeliaError>;

    /// Find peers in the routing system
    async fn find_peers(
        &self,
        peer_id: &PeerId,
        options: Option<FindPeersOptions>,
    ) -> Result<AwaitIterable<PeerInfo>, HeliaError>;

    /// Get a record from the routing system
    async fn get(
        &self,
        key: &[u8],
        options: Option<GetOptions>,
    ) -> Result<Option<RoutingRecord>, HeliaError>;

    /// Put a record into the routing system
    async fn put(
        &self,
        key: &[u8],
        value: &[u8],
        options: Option<PutOptions>,
    ) -> Result<(), HeliaError>;
}