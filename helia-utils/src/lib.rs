//! # Helia Utils
//!
//! Shared utilities and implementations for the Helia IPFS implementation.
//!
//! This crate provides concrete implementations of the traits defined in `helia-interface`,
//! including the main `Helia` struct, blockstore implementations, and utility functions.

pub mod blockstore;
pub mod blockstore_with_bitswap;
pub mod datastore;
pub mod helia;
pub mod libp2p_behaviour;
pub mod logger;
pub mod metrics;

#[cfg(test)]
mod blockstore_tests;

#[cfg(test)]
mod pins_tests;

use std::sync::Arc;

pub use blockstore::SledBlockstore;
pub use blockstore_with_bitswap::BlockstoreWithBitswap;
pub use datastore::SledDatastore;
pub use helia::{DummyRouting, HeliaImpl, SimplePins};
pub use libp2p_behaviour::{create_swarm, create_swarm_with_keypair, HeliaBehaviour};
pub use logger::TracingLogger;
pub use metrics::SimpleMetrics;

use libp2p::Swarm;
use tokio::sync::Mutex;

// Re-export interface types for convenience
pub use helia_interface::*;

/// Configuration for creating a new Helia node
pub struct HeliaConfig {
    /// The libp2p swarm instance (wrapped in Arc<Mutex<>> for thread safety)
    pub libp2p: Option<Arc<Mutex<Swarm<HeliaBehaviour>>>>,
    /// Datastore configuration
    pub datastore: DatastoreConfig,
    /// Blockstore configuration
    pub blockstore: BlockstoreConfig,
    /// DNS resolver configuration
    pub dns: Option<trust_dns_resolver::TokioAsyncResolver>,
    /// Logger configuration
    pub logger: LoggerConfig,
    /// Metrics configuration
    pub metrics: Option<Arc<dyn Metrics>>,
}

impl std::fmt::Debug for HeliaConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeliaConfig")
            .field("libp2p", &self.libp2p.as_ref().map(|_| "Some(Swarm)"))
            .field("datastore", &self.datastore)
            .field("blockstore", &self.blockstore)
            .field("dns", &self.dns.as_ref().map(|_| "Some(resolver)"))
            .field("logger", &self.logger)
            .field("metrics", &self.metrics.as_ref().map(|_| "Some(metrics)"))
            .finish()
    }
}

impl Default for HeliaConfig {
    fn default() -> Self {
        Self {
            libp2p: None,
            datastore: DatastoreConfig::default(),
            blockstore: BlockstoreConfig::default(),
            dns: None,
            logger: LoggerConfig::default(),
            metrics: None,
        }
    }
}

/// Configuration for the datastore
#[derive(Debug, Clone)]
pub struct DatastoreConfig {
    /// Path to the datastore directory
    pub path: Option<std::path::PathBuf>,
    /// Whether to create the datastore if it doesn't exist
    pub create_if_missing: bool,
}

impl Default for DatastoreConfig {
    fn default() -> Self {
        Self {
            path: None,
            create_if_missing: true,
        }
    }
}

/// Configuration for the blockstore
#[derive(Debug, Clone)]
pub struct BlockstoreConfig {
    /// Path to the blockstore directory
    pub path: Option<std::path::PathBuf>,
    /// Whether to create the blockstore if it doesn't exist
    pub create_if_missing: bool,
}

impl Default for BlockstoreConfig {
    fn default() -> Self {
        Self {
            path: None,
            create_if_missing: true,
        }
    }
}

/// Configuration for the logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Log level
    pub level: tracing::Level,
    /// Whether to include timestamps
    pub include_timestamps: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: tracing::Level::INFO,
            include_timestamps: true,
        }
    }
}
