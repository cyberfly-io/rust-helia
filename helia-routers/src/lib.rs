//! Routing abstractions for Helia
//! 
//! Provides content routing (finding content) and peer routing (finding peers).

pub mod delegated_http_routing;
pub mod http_gateway_routing;

use std::sync::Arc;
use async_trait::async_trait;
use cid::Cid;
use libp2p::PeerId;
use helia_interface::Helia;

/// Errors that can occur during routing operations
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("Content not found: {0}")]
    ContentNotFound(Cid),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    
    #[error("Routing failed: {0}")]
    RoutingFailed(String),
    
    #[error("Timeout")]
    Timeout,
}

/// Information about a content provider
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Peer ID of the provider
    pub peer_id: PeerId,
    
    /// Addresses where the peer can be reached
    pub addrs: Vec<libp2p::Multiaddr>,
}

/// Information about a peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// The peer's ID
    pub peer_id: PeerId,
    
    /// Known addresses for the peer
    pub addrs: Vec<libp2p::Multiaddr>,
    
    /// Protocols supported by the peer
    pub protocols: Vec<String>,
}

/// Content routing interface
#[async_trait]
pub trait ContentRouting: Send + Sync {
    /// Find providers for a given CID
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<ProviderInfo>, RoutingError>;
    
    /// Announce that we are providing content
    async fn provide(&self, cid: &Cid) -> Result<(), RoutingError>;
}

/// Peer routing interface
#[async_trait]
pub trait PeerRouting: Send + Sync {
    /// Find information about a peer
    async fn find_peer(&self, peer_id: &PeerId) -> Result<PeerInfo, RoutingError>;
}

/// Combined routers struct
pub struct Routers {
    _helia: Arc<dyn Helia>,
}

impl Routers {
    /// Create a new Routers instance
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self {
            _helia: helia,
        }
    }
}

#[async_trait]
impl ContentRouting for Routers {
    async fn find_providers(&self, _cid: &Cid) -> Result<Vec<ProviderInfo>, RoutingError> {
        // In a full implementation, this would:
        // 1. Query the DHT for providers
        // 2. Query bitswap for connected peers
        // 3. Return combined results
        Ok(vec![])
    }
    
    async fn provide(&self, _cid: &Cid) -> Result<(), RoutingError> {
        // In a full implementation, this would:
        // 1. Announce to the DHT that we have this content
        // 2. Keep the announcement alive
        Ok(())
    }
}

#[async_trait]
impl PeerRouting for Routers {
    async fn find_peer(&self, peer_id: &PeerId) -> Result<PeerInfo, RoutingError> {
        // In a full implementation, this would:
        // 1. Check if we're already connected
        // 2. Query the DHT for peer information
        // 3. Return peer details
        Err(RoutingError::PeerNotFound(*peer_id))
    }
}

/// Create a Routers instance
pub fn routers(helia: Arc<dyn Helia>) -> Routers {
    Routers::new(helia)
}

// Tests have been moved to individual router module tests
