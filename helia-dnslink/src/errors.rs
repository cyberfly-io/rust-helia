/// Errors that can occur during DNSLink operations
#[derive(Debug, thiserror::Error)]
pub enum DnsLinkError {
    #[error("DNSLink not found for domain: {0}")]
    NotFound(String),

    #[error("Invalid DNSLink format: {0}")]
    InvalidFormat(String),

    #[error("Recursion limit exceeded (max: {0})")]
    RecursionLimit(u32),

    #[error("Invalid CID in DNSLink: {0}")]
    InvalidCid(String),

    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),

    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolutionFailed(String),

    #[error("Invalid domain name: {0}")]
    InvalidDomain(String),

    #[error("Offline mode enabled, cannot query network")]
    OfflineMode,
}
