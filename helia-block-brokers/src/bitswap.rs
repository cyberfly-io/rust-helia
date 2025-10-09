//! Bitswap BlockBroker implementation
//!
//! This module provides a BlockBroker implementation that wraps the Bitswap coordinator.

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use helia_bitswap::Bitswap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{BlockAnnounceOptions, BlockBroker, BlockRetrievalOptions, BrokerStats, Result};

/// Bitswap BlockBroker wrapper
pub struct BitswapBroker {
    bitswap: Arc<Bitswap>,
    stats: Arc<tokio::sync::Mutex<BrokerStats>>,
}

impl BitswapBroker {
    /// Create a new Bitswap broker
    pub fn new(bitswap: Arc<Bitswap>) -> Self {
        Self {
            bitswap,
            stats: Arc::new(tokio::sync::Mutex::new(BrokerStats::default())),
        }
    }
}

#[async_trait]
impl BlockBroker for BitswapBroker {
    async fn retrieve(&self, cid: Cid, options: BlockRetrievalOptions) -> Result<Bytes> {
        let start = Instant::now();
        let mut stats = self.stats.lock().await;
        stats.requests_made += 1;
        drop(stats);

        // Use Bitswap want() API with timeout from options
        let want_options = helia_bitswap::WantOptions {
            timeout: options.timeout,
            priority: options.priority.unwrap_or(0),
            accept_block_presence: true,
            peer: None,
        };

        match self.bitswap.want(&cid, want_options).await {
            Ok(data) => {
                let elapsed = start.elapsed();
                let mut stats = self.stats.lock().await;
                stats.successful_requests += 1;

                // Update average response time
                let total_time = stats.avg_response_time.as_millis()
                    * stats.successful_requests as u128
                    + elapsed.as_millis();
                stats.avg_response_time = Duration::from_millis(
                    (total_time / (stats.successful_requests + 1) as u128) as u64,
                );
                stats.last_seen = Instant::now();

                Ok(data)
            }
            Err(e) => {
                let mut stats = self.stats.lock().await;
                stats.failed_requests += 1;
                Err(e)
            }
        }
    }

    async fn announce(&self, cid: Cid, data: Bytes, options: BlockAnnounceOptions) -> Result<()> {
        let notify_options = helia_bitswap::NotifyOptions {
            broadcast: options.broadcast,
        };

        self.bitswap
            .notify_new_blocks(vec![(cid, data)], notify_options)
            .await
    }

    async fn start(&self) -> Result<()> {
        self.bitswap.start().await
    }

    async fn stop(&self) -> Result<()> {
        self.bitswap.stop().await
    }

    fn get_stats(&self) -> BrokerStats {
        // This is synchronous, so we return a cloned version
        // In practice, you might want to use try_lock() or return a Future
        self.stats.try_lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn name(&self) -> &str {
        "bitswap"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helia_bitswap::BitswapConfig;
    use helia_utils::{BlockstoreConfig, SledBlockstore};

    #[tokio::test]
    async fn test_bitswap_broker_creation() {
        let blockstore = Arc::new(SledBlockstore::new(BlockstoreConfig::default()).unwrap());
        let config = BitswapConfig::default();
        let bitswap = Arc::new(Bitswap::new(blockstore, config).await.unwrap());

        let broker = BitswapBroker::new(bitswap);
        assert_eq!(broker.name(), "bitswap");

        let stats = broker.get_stats();
        assert_eq!(stats.requests_made, 0);
        assert_eq!(stats.successful_requests, 0);
    }
}
