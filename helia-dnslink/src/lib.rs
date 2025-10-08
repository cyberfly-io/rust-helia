//! DNSLink resolution for Helia
//! 
//! DNSLink allows domain names to point to IPFS content using DNS TXT records.

use std::sync::Arc;
use async_trait::async_trait;
use cid::Cid;
use helia_interface::Helia;

/// Errors that can occur during DNSLink operations
#[derive(Debug, thiserror::Error)]
pub enum DnsLinkError {
    #[error("Invalid domain: {0}")]
    InvalidDomain(String),
    
    #[error("No DNSLink record found for: {0}")]
    NotFound(String),
    
    #[error("Invalid DNSLink format: {0}")]
    InvalidFormat(String),
    
    #[error("DNS resolution failed: {0}")]
    ResolutionFailed(String),
    
    #[error("Invalid CID in DNSLink: {0}")]
    InvalidCid(String),
}

/// Result of resolving a DNSLink
#[derive(Debug, Clone)]
pub struct DnsLinkResult {
    /// The resolved CID
    pub cid: Cid,
    
    /// Optional path component
    pub path: Option<String>,
    
    /// The TXT record value
    pub data: String,
}

/// Options for DNSLink resolution
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    /// Follow recursive DNSLink records
    pub recursive: bool,
    
    /// Maximum recursion depth
    pub max_depth: Option<u32>,
    
    /// Timeout in seconds
    pub timeout: Option<u64>,
}

/// The main DNSLink interface
#[async_trait]
pub trait DnsLinkInterface: Send + Sync {
    /// Resolve a domain to its DNSLink CID
    async fn resolve(&self, domain: &str, options: Option<ResolveOptions>) -> Result<DnsLinkResult, DnsLinkError>;
}

/// Default DNSLink implementation
pub struct DnsLink {
    _helia: Arc<dyn Helia>,
}

impl DnsLink {
    /// Create a new DNSLink instance
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self {
            _helia: helia,
        }
    }
    
    fn parse_dnslink(&self, txt_record: &str) -> Result<(Cid, Option<String>), DnsLinkError> {
        // DNSLink format: dnslink=/ipfs/<cid> or dnslink=/ipfs/<cid>/path
        let value = if txt_record.starts_with("dnslink=") {
            &txt_record[8..]
        } else {
            txt_record
        };
        
        if !value.starts_with("/ipfs/") {
            return Err(DnsLinkError::InvalidFormat(format!(
                "Expected /ipfs/ prefix, got: {}",
                value
            )));
        }
        
        let without_prefix = &value[6..]; // Remove "/ipfs/"
        let parts: Vec<&str> = without_prefix.splitn(2, '/').collect();
        
        let cid = Cid::try_from(parts[0])
            .map_err(|e| DnsLinkError::InvalidCid(format!("Invalid CID: {}", e)))?;
        
        let path = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };
        
        Ok((cid, path))
    }
}

#[async_trait]
impl DnsLinkInterface for DnsLink {
    async fn resolve(&self, domain: &str, _options: Option<ResolveOptions>) -> Result<DnsLinkResult, DnsLinkError> {
        // Construct the DNSLink domain (_dnslink.<domain>)
        let dnslink_domain = if domain.starts_with("_dnslink.") {
            domain.to_string()
        } else {
            format!("_dnslink.{}", domain)
        };
        
        // In a full implementation, we would:
        // 1. Query DNS TXT records for _dnslink.<domain>
        // 2. Parse the dnslink=/ipfs/<cid> format
        // 3. Handle recursive resolution if needed
        
        // For now, return a simulated result for testing
        // In production, use trust-dns-resolver to actually query DNS
        
        Err(DnsLinkError::NotFound(domain.to_string()))
    }
}

/// Create a DNSLink instance
pub fn dnslink(helia: Arc<dyn Helia>) -> impl DnsLinkInterface {
    DnsLink::new(helia)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_parse_dnslink() {
        let hash = multihash::Multihash::<64>::wrap(0x12, &[0u8; 32]).unwrap();
        let cid = Cid::new_v1(0x55, hash);
        let txt_record = format!("dnslink=/ipfs/{}", cid);
        
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let dnslink = DnsLink::new(helia);
        
        let result = dnslink.parse_dnslink(&txt_record).unwrap();
        assert_eq!(result.0, cid);
        assert_eq!(result.1, None);
    }
    
    #[tokio::test]
    async fn test_parse_dnslink_with_path() {
        let hash = multihash::Multihash::<64>::wrap(0x12, &[0u8; 32]).unwrap();
        let cid = Cid::new_v1(0x55, hash);
        let txt_record = format!("dnslink=/ipfs/{}/path/to/file", cid);
        
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let dnslink = DnsLink::new(helia);
        
        let result = dnslink.parse_dnslink(&txt_record).unwrap();
        assert_eq!(result.0, cid);
        assert_eq!(result.1, Some("path/to/file".to_string()));
    }
    
    #[tokio::test]
    async fn test_parse_invalid_dnslink() {
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let dnslink = DnsLink::new(helia);
        
        let result = dnslink.parse_dnslink("invalid-format");
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_dnslink_creation() {
        let helia = Arc::new(helia::create_helia_default().await.unwrap());
        let _dnslink = DnsLink::new(helia);
        // Just verify we can create an instance
    }
}
