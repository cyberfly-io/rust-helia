//! Block broker abstractions for Helia
//!
//! This crate provides abstractions for coordinating block retrieval from multiple sources.

pub mod trustless_gateway;
pub mod bitswap;

use bytes::Bytes;
use cid::Cid;
use helia_interface::HeliaError;
use std::time::{Duration, Instant};

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
