//! Error types for Helia operations

use thiserror::Error;

/// Main error type for Helia operations
#[derive(Error, Debug)]
pub enum HeliaError {
    /// Libp2p related errors (using string representation since libp2p::Error is complex)
    #[error("Libp2p error: {0}")]
    Libp2p(String),

    /// CID parsing or validation errors
    #[error("CID error: {0}")]
    Cid(#[from] cid::Error),

    /// Multihash errors
    #[error("Multihash error: {0}")]
    Multihash(#[from] multihash::Error),

    /// Multiaddr parsing errors  
    #[error("Multiaddr error: {0}")]
    Multiaddr(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Block not found
    #[error("Block not found: {cid}")]
    BlockNotFound { cid: cid::Cid },

    /// Peer not found
    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: libp2p::PeerId },

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Abort error
    #[error("Operation was aborted")]
    Aborted,

    /// Invalid input
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Node not started
    #[error("Node not started")]
    NodeNotStarted,

    /// Node already started  
    #[error("Node already started")]
    NodeAlreadyStarted,

    /// Codec not found
    #[error("Codec not found: {code}")]
    CodecNotFound { code: u64 },

    /// Hasher not found
    #[error("Hasher not found: {code}")]
    HasherNotFound { code: u64 },

    /// Pin not found
    #[error("Pin not found: {cid}")]
    PinNotFound { cid: cid::Cid },

    /// Pin already exists
    #[error("Pin already exists: {cid}")]
    PinAlreadyExists { cid: cid::Cid },

    /// Datastore error
    #[error("Datastore error: {message}")]
    Datastore { message: String },

    /// Routing error
    #[error("Routing error: {message}")]
    Routing { message: String },

    /// Network error
    #[error("Network error: {message}")]
    Network { message: String },

    /// DNS resolution error
    #[error("DNS error: {0}")]
    Dns(#[from] trust_dns_resolver::error::ResolveError),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),

    /// Generic error with custom message
    #[error("Error: {message}")]
    Other { message: String },
}

impl HeliaError {
    /// Create a new custom error
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }

    /// Create a new datastore error
    pub fn datastore(message: impl Into<String>) -> Self {
        Self::Datastore {
            message: message.into(),
        }
    }

    /// Create a new routing error
    pub fn routing(message: impl Into<String>) -> Self {
        Self::Routing {
            message: message.into(),
        }
    }

    /// Create a new network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a new invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }
}
