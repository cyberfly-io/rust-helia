//! Block broker abstractions for Helia
//!
//! This crate provides abstractions and implementations for coordinating block retrieval
//! from multiple sources in IPFS/Helia. Block brokers are components that abstract away
//! the details of how and where blocks are retrieved, allowing Helia to work with multiple
//! content retrieval strategies seamlessly.
//!
//! # Overview
//!
//! The `BlockBroker` trait defines a common interface for different block retrieval strategies:
//!
//! - **Bitswap Brokers**: Retrieve blocks via the Bitswap protocol over libp2p
//! - **Gateway Brokers**: Retrieve blocks from HTTP trustless gateways  
//! - **Composite Brokers**: Coordinate multiple brokers with fallback strategies
//! - **Custom Brokers**: Implement your own retrieval logic
//!
//! Each broker implementation handles the specifics of communicating with its source,
//! while exposing a consistent async interface to Helia.
//!
//! # Core Concepts
//!
//! ## Block Retrieval
//!
//! The primary operation is retrieving a block by its CID:
//!
//! ```rust,ignore
//! use helia_block_brokers::{BlockBroker, BlockRetrievalOptions, bitswap_broker};
//! use helia_bitswap::Bitswap;
//! use cid::Cid;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let bitswap: Arc<Bitswap> = todo!("Create Bitswap instance");
//! // Create a bitswap broker
//! let broker = bitswap_broker(bitswap);
//!
//! // Retrieve a block with options
//! let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
//! let options = BlockRetrievalOptions {
//!     timeout: Some(Duration::from_secs(30)),
//!     priority: Some(1),
//!     max_providers: Some(10),
//!     use_cache: true,
//! };
//!
//! let block = broker.retrieve(cid, options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Block Announcement
//!
//! Brokers can also announce that new blocks are available:
//!
//! ```rust,ignore
//! use helia_block_brokers::{BlockBroker, BlockAnnounceOptions, bitswap_broker};
//! use helia_bitswap::Bitswap;
//! use bytes::Bytes;
//! use cid::Cid;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let bitswap: Arc<Bitswap> = todo!("Create Bitswap instance");
//! let broker = bitswap_broker(bitswap);
//!
//! let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
//! let data = Bytes::from("hello world");
//!
//! let options = BlockAnnounceOptions {
//!     broadcast: true,
//!     providers: vec![],
//!     ttl: None,
//! };
//!
//! broker.announce(cid, data, options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Statistics and Monitoring
//!
//! Each broker tracks statistics about its performance:
//!
//! ```rust,ignore
//! use helia_block_brokers::{BlockBroker, bitswap_broker};
//! use helia_bitswap::Bitswap;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! # let bitswap: Arc<Bitswap> = todo!("Create Bitswap instance");
//! let broker = bitswap_broker(bitswap);
//!
//! // Get broker statistics
//! let stats = broker.get_stats();
//! println!("Requests: {}", stats.requests_made);
//! println!("Success rate: {:.2}%",
//!     (stats.successful_requests as f64 / stats.requests_made as f64) * 100.0
//! );
//! println!("Avg response time: {:?}", stats.avg_response_time);
//! # }
//! ```
//!
//! # Choosing a Broker Strategy
//!
//! Different broker types have different characteristics:
//!
//! | Broker Type | Best For | Pros | Cons |
//! |-------------|----------|------|------|
//! | **Bitswap** | P2P networks with many peers | - Fast with many peers<br>- Decentralized<br>- Efficient for popular content | - Requires libp2p setup<br>- Slower with few peers |
//! | **Gateway** | Public content retrieval | - No P2P setup needed<br>- Simple HTTP requests<br>- Works behind firewalls | - Relies on gateway availability<br>- Potentially slower<br>- Less decentralized |
//! | **Composite** | Production applications | - Automatic fallback<br>- Best of both worlds<br>- Resilient | - More complex<br>- Higher resource usage |
//!
//! # Using the Trustless Gateway Broker
//!
//! The trustless gateway broker fetches content via HTTP gateways:
//!
//! ```rust,no_run
//! use helia_block_brokers::{BlockBroker, BlockRetrievalOptions};
//! use helia_block_brokers::trustless_gateway::{trustless_gateway, TrustlessGatewayInit};
//! use url::Url;
//! use cid::Cid;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use default public gateways
//! let gateway = trustless_gateway(TrustlessGatewayInit::default());
//!
//! // Or configure custom gateways
//! let custom = trustless_gateway(TrustlessGatewayInit {
//!     gateways: vec![
//!         Url::parse("https://ipfs.io")?,
//!         Url::parse("https://dweb.link")?,
//!     ],
//!     max_retries: 3,
//!     timeout_ms: 30000,
//!     allow_insecure: false,
//!     allow_redirects: true,
//! });
//!
//! // Retrieve a block
//! let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
//! let block = custom.retrieve(cid, BlockRetrievalOptions::default()).await?;
//! # Ok(())
//! # }
//! ```
//!
//! The trustless gateway broker includes:
//!
//! - Automatic retry with exponential backoff
//! - Gateway reliability scoring and prioritization
//! - Failover between multiple gateways
//! - Request timeout and cancellation
//! - Detailed statistics per gateway
//!
//! # Implementing Custom Brokers
//!
//! You can implement custom block retrieval strategies:
//!
//! ```rust
//! use helia_block_brokers::{BlockBroker, BlockRetrievalOptions, BlockAnnounceOptions, BrokerStats};
//! use bytes::Bytes;
//! use cid::Cid;
//! use helia_interface::HeliaError;
//! use std::time::Instant;
//!
//! struct CacheBroker {
//!     cache: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Cid, Bytes>>>,
//! }
//!
//! #[async_trait::async_trait]
//! impl BlockBroker for CacheBroker {
//!     async fn retrieve(&self, cid: Cid, _options: BlockRetrievalOptions)
//!         -> Result<Bytes, HeliaError>
//!     {
//!         let cache = self.cache.read().await;
//!         cache.get(&cid)
//!             .cloned()
//!             .ok_or_else(|| HeliaError::NotFound("Block not in cache".to_string()))
//!     }
//!
//!     async fn announce(&self, cid: Cid, data: Bytes, _options: BlockAnnounceOptions)
//!         -> Result<(), HeliaError>
//!     {
//!         let mut cache = self.cache.write().await;
//!         cache.insert(cid, data);
//!         Ok(())
//!     }
//!
//!     async fn start(&self) -> Result<(), HeliaError> {
//!         Ok(())
//!     }
//!
//!     async fn stop(&self) -> Result<(), HeliaError> {
//!         Ok(())
//!     }
//!
//!     fn get_stats(&self) -> BrokerStats {
//!         BrokerStats::default()
//!     }
//!
//!     fn name(&self) -> &str {
//!         "cache"
//!     }
//! }
//! ```
//!
//! # Error Handling
//!
//! All broker operations return `Result<T, HeliaError>`. Common error scenarios include:
//!
//! - **Timeout**: The operation exceeded the configured timeout
//! - **NotFound**: The requested CID was not found by any provider
//! - **NetworkError**: Network communication failed
//! - **InvalidData**: Retrieved data failed verification
//!
//! Example error handling:
//!
//! ```rust,ignore
//! use helia_block_brokers::{BlockBroker, BlockRetrievalOptions, bitswap_broker};
//! use helia_bitswap::Bitswap;
//! use helia_interface::HeliaError;
//! use cid::Cid;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let bitswap: Arc<Bitswap> = todo!("Create Bitswap instance");
//! let broker = bitswap_broker(bitswap);
//! let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
//!
//! match broker.retrieve(cid, BlockRetrievalOptions::default()).await {
//!     Ok(block) => {
//!         println!("Retrieved {} bytes", block.len());
//!     }
//!     Err(HeliaError::Timeout) => {
//!         eprintln!("Request timed out, retrying with different broker...");
//!     }
//!     Err(HeliaError::NotFound(msg)) => {
//!         eprintln!("Block not found on network: {}", msg);
//!     }
//!     Err(e) => {
//!         eprintln!("Unexpected error: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! ## Bitswap Broker
//!
//! - Performance scales with number of connected peers
//! - Benefits from DHT for content routing
//! - Best for applications with established P2P networks
//! - Consider connection limits and bandwidth
//!
//! ## Gateway Broker
//!
//! - Performance depends on gateway availability and location
//! - Benefits from multiple gateway URLs for fallback
//! - Gateway reliability scoring helps avoid slow gateways
//! - Consider rate limiting from public gateways
//!
//! ## General Tips
//!
//! - Use appropriate timeouts (typically 10-30 seconds)
//! - Set realistic `max_providers` to balance speed vs resource usage
//! - Enable caching when appropriate
//! - Monitor broker statistics to identify performance issues
//! - Consider implementing composite brokers with fallback logic
//!
//! # Thread Safety
//!
//! All broker implementations are `Send + Sync` and can be safely shared across
//! async tasks. Internal state is protected with appropriate synchronization primitives.
//!
//! # Examples
//!
//! See the `examples/` directory for complete working examples of:
//!
//! - Basic bitswap broker usage
//! - Trustless gateway configuration
//! - Composite broker with fallback
//! - Custom broker implementation
//! - Statistics monitoring and logging

pub mod bitswap;
pub mod trustless_gateway;

use bytes::Bytes;
use cid::Cid;
use helia_interface::HeliaError;
use std::time::{Duration, Instant};

// Re-export key types and functions
pub use bitswap::{bitswap_broker, BitswapBroker};
pub use trustless_gateway::{trustless_gateway, TrustlessGateway, TrustlessGatewayInit};

pub type Result<T> = std::result::Result<T, HeliaError>;

/// Options for retrieving blocks
#[derive(Debug, Clone, Default)]
pub struct BlockRetrievalOptions {
    /// Timeout for the operation
    pub timeout: Option<Duration>,
    /// Priority hint
    pub priority: Option<i32>,
    /// Maximum number of providers to try
    pub max_providers: Option<usize>,
    /// Whether to use cache
    pub use_cache: bool,
}

/// Options for announcing blocks
#[derive(Debug, Clone, Default)]
pub struct BlockAnnounceOptions {
    /// Whether to broadcast to all peers
    pub broadcast: bool,
    /// Specific providers to announce to
    pub providers: Vec<String>,
    /// Time to live for the announcement
    pub ttl: Option<Duration>,
}

/// Types of block providers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderType {
    Bitswap,
    Gateway,
}

/// Broker statistics
#[derive(Debug, Clone)]
pub struct BrokerStats {
    pub requests_made: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: Duration,
    pub last_seen: Instant,
}

impl Default for BrokerStats {
    fn default() -> Self {
        Self {
            requests_made: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_secs(0),
            last_seen: Instant::now(),
        }
    }
}

/// Main block broker trait
#[async_trait::async_trait]
pub trait BlockBroker: Send + Sync {
    async fn retrieve(&self, cid: Cid, options: BlockRetrievalOptions) -> Result<Bytes>;
    async fn announce(&self, cid: Cid, data: Bytes, options: BlockAnnounceOptions) -> Result<()>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    fn get_stats(&self) -> BrokerStats;
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_retrieval_options_default() {
        let options = BlockRetrievalOptions::default();
        assert!(options.timeout.is_none());
        assert!(options.priority.is_none());
        assert!(options.max_providers.is_none());
        assert!(!options.use_cache);
    }

    #[test]
    fn test_block_retrieval_options_custom() {
        let options = BlockRetrievalOptions {
            timeout: Some(Duration::from_secs(30)),
            priority: Some(5),
            max_providers: Some(10),
            use_cache: true,
        };
        assert_eq!(options.timeout, Some(Duration::from_secs(30)));
        assert_eq!(options.priority, Some(5));
        assert_eq!(options.max_providers, Some(10));
        assert!(options.use_cache);
    }

    #[test]
    fn test_block_announce_options_default() {
        let options = BlockAnnounceOptions::default();
        assert!(!options.broadcast);
        assert!(options.providers.is_empty());
        assert!(options.ttl.is_none());
    }

    #[test]
    fn test_block_announce_options_custom() {
        let providers = vec!["peer1".to_string(), "peer2".to_string()];
        let options = BlockAnnounceOptions {
            broadcast: true,
            providers: providers.clone(),
            ttl: Some(Duration::from_secs(3600)),
        };
        assert!(options.broadcast);
        assert_eq!(options.providers, providers);
        assert_eq!(options.ttl, Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_provider_type_equality() {
        assert_eq!(ProviderType::Bitswap, ProviderType::Bitswap);
        assert_eq!(ProviderType::Gateway, ProviderType::Gateway);
        assert_ne!(ProviderType::Bitswap, ProviderType::Gateway);
    }

    #[test]
    fn test_provider_type_clone() {
        let provider = ProviderType::Bitswap;
        let cloned = provider.clone();
        assert_eq!(provider, cloned);
    }

    #[test]
    fn test_broker_stats_default() {
        let stats = BrokerStats::default();
        assert_eq!(stats.requests_made, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.avg_response_time, Duration::from_secs(0));
        // last_seen should be recent
        assert!(stats.last_seen.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_broker_stats_clone() {
        let stats = BrokerStats {
            requests_made: 10,
            successful_requests: 8,
            failed_requests: 2,
            avg_response_time: Duration::from_millis(500),
            last_seen: Instant::now(),
        };
        let cloned = stats.clone();
        assert_eq!(cloned.requests_made, 10);
        assert_eq!(cloned.successful_requests, 8);
        assert_eq!(cloned.failed_requests, 2);
        assert_eq!(cloned.avg_response_time, Duration::from_millis(500));
    }

    #[test]
    fn test_broker_stats_success_rate() {
        let stats = BrokerStats {
            requests_made: 100,
            successful_requests: 85,
            failed_requests: 15,
            avg_response_time: Duration::from_millis(250),
            last_seen: Instant::now(),
        };
        let success_rate = stats.successful_requests as f64 / stats.requests_made as f64;
        assert_eq!(success_rate, 0.85);
    }

    #[test]
    fn test_block_retrieval_options_timeout_boundary() {
        // Test zero timeout
        let opts = BlockRetrievalOptions {
            timeout: Some(Duration::from_secs(0)),
            ..Default::default()
        };
        assert_eq!(opts.timeout, Some(Duration::from_secs(0)));

        // Test very large timeout
        let opts = BlockRetrievalOptions {
            timeout: Some(Duration::from_secs(86400)), // 24 hours
            ..Default::default()
        };
        assert_eq!(opts.timeout, Some(Duration::from_secs(86400)));
    }

    #[test]
    fn test_block_retrieval_options_priority_boundary() {
        // Test negative priority
        let opts = BlockRetrievalOptions {
            priority: Some(-100),
            ..Default::default()
        };
        assert_eq!(opts.priority, Some(-100));

        // Test positive priority
        let opts = BlockRetrievalOptions {
            priority: Some(100),
            ..Default::default()
        };
        assert_eq!(opts.priority, Some(100));
    }

    #[test]
    fn test_block_retrieval_options_max_providers_zero() {
        let opts = BlockRetrievalOptions {
            max_providers: Some(0),
            ..Default::default()
        };
        assert_eq!(opts.max_providers, Some(0));
    }

    #[test]
    fn test_block_announce_options_empty_providers() {
        let opts = BlockAnnounceOptions {
            broadcast: true,
            providers: vec![],
            ttl: Some(Duration::from_secs(60)),
        };
        assert!(opts.providers.is_empty());
        assert!(opts.broadcast);
    }

    #[test]
    fn test_block_announce_options_many_providers() {
        let providers: Vec<String> = (0..100).map(|i| format!("peer{}", i)).collect();
        let opts = BlockAnnounceOptions {
            broadcast: false,
            providers: providers.clone(),
            ttl: None,
        };
        assert_eq!(opts.providers.len(), 100);
        assert_eq!(opts.providers[0], "peer0");
        assert_eq!(opts.providers[99], "peer99");
    }

    #[test]
    fn test_broker_stats_zero_requests() {
        let stats = BrokerStats {
            requests_made: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_secs(0),
            last_seen: Instant::now(),
        };
        // Should not panic when calculating success rate with zero requests
        let success_rate = if stats.requests_made > 0 {
            stats.successful_requests as f64 / stats.requests_made as f64
        } else {
            0.0
        };
        assert_eq!(success_rate, 0.0);
    }

    #[test]
    fn test_broker_stats_all_failures() {
        let stats = BrokerStats {
            requests_made: 50,
            successful_requests: 0,
            failed_requests: 50,
            avg_response_time: Duration::from_secs(0),
            last_seen: Instant::now(),
        };
        let success_rate = stats.successful_requests as f64 / stats.requests_made as f64;
        assert_eq!(success_rate, 0.0);
    }

    #[test]
    fn test_broker_stats_all_successes() {
        let stats = BrokerStats {
            requests_made: 50,
            successful_requests: 50,
            failed_requests: 0,
            avg_response_time: Duration::from_millis(100),
            last_seen: Instant::now(),
        };
        let success_rate = stats.successful_requests as f64 / stats.requests_made as f64;
        assert_eq!(success_rate, 1.0);
    }

    #[test]
    fn test_block_retrieval_options_builder_pattern() {
        // Simulate builder-like pattern
        let mut opts = BlockRetrievalOptions::default();
        opts.timeout = Some(Duration::from_secs(15));
        opts.priority = Some(3);
        opts.use_cache = true;

        assert_eq!(opts.timeout, Some(Duration::from_secs(15)));
        assert_eq!(opts.priority, Some(3));
        assert!(opts.use_cache);
        assert!(opts.max_providers.is_none());
    }

    #[test]
    fn test_block_announce_options_ttl_zero() {
        let opts = BlockAnnounceOptions {
            broadcast: false,
            providers: vec![],
            ttl: Some(Duration::from_secs(0)),
        };
        assert_eq!(opts.ttl, Some(Duration::from_secs(0)));
    }

    #[test]
    fn test_provider_type_debug() {
        let bitswap = ProviderType::Bitswap;
        let gateway = ProviderType::Gateway;
        let bitswap_debug = format!("{:?}", bitswap);
        let gateway_debug = format!("{:?}", gateway);
        assert!(bitswap_debug.contains("Bitswap"));
        assert!(gateway_debug.contains("Gateway"));
    }
}
