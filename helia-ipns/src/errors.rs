//! Error types for IPNS operations

use std::fmt;

/// Errors that can occur during IPNS operations
#[derive(Debug, thiserror::Error)]
pub enum IpnsError {
    /// IPNS record was not found
    #[error("IPNS record not found: {0}")]
    NotFound(String),

    /// Invalid IPNS record format or content
    #[error("Invalid IPNS record: {0}")]
    InvalidRecord(String),

    /// IPNS record has expired
    #[error("IPNS record expired (validity: {validity})")]
    RecordExpired { validity: String },

    /// Invalid key format or unsupported key type
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Record validation failed (signature, format, etc.)
    #[error("IPNS record validation failed: {0}")]
    ValidationFailed(String),

    /// Routing operation failed
    #[error("Routing failed: {0}")]
    RoutingFailed(String),

    /// Recursion limit exceeded during resolution
    #[error("Recursion limit exceeded (max: {0})")]
    RecursionLimit(u32),

    /// Invalid CID in IPNS record value
    #[error("Invalid CID in IPNS value: {0}")]
    InvalidCid(String),

    /// Invalid path in IPNS record
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Unsupported multibase prefix
    #[error("Unsupported multibase prefix: {0}")]
    UnsupportedMultibase(String),

    /// Unsupported multihash codec
    #[error("Unsupported multihash codec: {0}")]
    UnsupportedMultihash(String),

    /// Operation requires network but offline mode is enabled
    #[error("Offline mode enabled, cannot query network")]
    OfflineMode,

    /// Key not found in keychain
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Failed to marshal/unmarshal IPNS record
    #[error("Marshaling error: {0}")]
    MarshalingError(String),

    /// Failed to publish IPNS record
    #[error("Publish failed: {0}")]
    PublishFailed(String),

    /// Failed to resolve IPNS name
    #[error("Resolve failed: {0}")]
    ResolveFailed(String),

    /// Multiple records failed validation
    #[error("{count} IPNS {label} failed validation")]
    RecordsFailedValidation { count: usize, label: String },

    /// DNSLink error (wrapped)
    #[error("DNSLink error: {0}")]
    DnsLink(#[from] helia_dnslink::DnsLinkError),

    /// IPNS library error (wrapped)
    #[error("IPNS error: {0}")]
    IpnsLib(String),

    /// CID parsing error
    #[error("CID error: {0}")]
    Cid(#[from] cid::Error),

    /// Multihash error
    #[error("Multihash error: {0}")]
    Multihash(String),

    /// libp2p identity error
    #[error("Identity error: {0}")]
    Identity(String),

    /// Failed to sign IPNS record
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Operation timed out
    #[error("Operation timed out")]
    Timeout,

    /// General error
    #[error("{0}")]
    Other(String),
}

impl IpnsError {
    /// Create a RecordsFailedValidation error with proper label
    pub fn records_failed_validation(count: usize) -> Self {
        Self::RecordsFailedValidation {
            count,
            label: if count > 1 {
                "records".to_string()
            } else {
                "record".to_string()
            },
        }
    }
}
