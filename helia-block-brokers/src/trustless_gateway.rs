//! Trustless Gateway implementation for fetching content from IPFS HTTP gateways
//!
//! This module implements a block broker that fetches content from trustless IPFS gateways
//! using the CAR (Content Addressed aRchive) format. It includes reliability tracking,
//! retry logic, and automatic failover between gateways.
//!
//! # Example
//!
//! ```no_run
//! use helia_block_brokers::trustless_gateway::{trustless_gateway, TrustlessGatewayInit};
//! use url::Url;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a trustless gateway broker with default public gateways
//! let gateway = trustless_gateway(TrustlessGatewayInit::default());
//!
//! // Or specify custom gateways
//! let custom_gateway = trustless_gateway(TrustlessGatewayInit {
//!     gateways: vec![
//!         Url::parse("https://ipfs.io")?,
//!         Url::parse("https://dweb.link")?,
//!     ],
//!     max_retries: 3,
//!     timeout_ms: 30000,
//!     ..Default::default()
//! });
//! # Ok(())
//! # }
//! ```

use crate::{BlockBroker, BlockRetrievalOptions, BlockAnnounceOptions, BrokerStats, Result};
use bytes::Bytes;
use cid::Cid;
use helia_car::CarReader;
use helia_interface::HeliaError;
use reqwest::Client;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn, error};
use url::Url;

/// Default public IPFS gateways
const DEFAULT_GATEWAYS: &[&str] = &[
    "https://ipfs.io",
    "https://dweb.link",
    "https://cloudflare-ipfs.com",
];

/// Configuration for trustless gateway initialization
#[derive(Debug, Clone)]
pub struct TrustlessGatewayInit {
    /// List of gateway URLs to use
    pub gateways: Vec<Url>,
    
    /// Maximum number of retries per gateway
    pub max_retries: usize,
    
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Whether to allow HTTP (not just HTTPS)
    pub allow_insecure: bool,
    
    /// Whether to allow gateway redirects
    pub allow_redirects: bool,
}

impl Default for TrustlessGatewayInit {
    fn default() -> Self {
        Self {
            gateways: DEFAULT_GATEWAYS
                .iter()
                .filter_map(|url| Url::parse(url).ok())
                .collect(),
            max_retries: 3,
            timeout_ms: 30000, // 30 seconds
            allow_insecure: false,
            allow_redirects: true,
        }
    }
}

/// Statistics for a single gateway
#[derive(Debug, Clone)]
struct GatewayStats {
    /// Total requests to this gateway
    requests: u64,
    
    /// Successful requests
    successes: u64,
    
    /// Failed requests
    failures: u64,
    
    /// Average response time
    avg_response_time: Duration,
    
    /// Last successful request
    last_success: Option<Instant>,
    
    /// Last failure
    last_failure: Option<Instant>,
    
    /// Consecutive failures (used for backoff)
    consecutive_failures: u32,
}

impl Default for GatewayStats {
    fn default() -> Self {
        Self {
            requests: 0,
            successes: 0,
            failures: 0,
            avg_response_time: Duration::from_secs(0),
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
        }
    }
}

impl GatewayStats {
    /// Calculate reliability score (0.0 to 1.0)
    fn reliability_score(&self) -> f64 {
        if self.requests == 0 {
            return 0.5; // Neutral for untested gateways
        }
        
        let success_rate = self.successes as f64 / self.requests as f64;
        
        // Penalize recent failures
        let recency_penalty = if self.consecutive_failures > 0 {
            0.9_f64.powi(self.consecutive_failures as i32)
        } else {
            1.0
        };
        
        success_rate * recency_penalty
    }
    
    /// Record a successful request
    fn record_success(&mut self, response_time: Duration) {
        self.requests += 1;
        self.successes += 1;
        self.consecutive_failures = 0;
        self.last_success = Some(Instant::now());
        
        // Update moving average
        if self.avg_response_time.as_millis() == 0 {
            self.avg_response_time = response_time;
        } else {
            let avg_ms = self.avg_response_time.as_millis() as f64;
            let new_ms = response_time.as_millis() as f64;
            let updated_avg = (avg_ms * 0.8) + (new_ms * 0.2);
            self.avg_response_time = Duration::from_millis(updated_avg as u64);
        }
    }
    
    /// Record a failed request
    fn record_failure(&mut self) {
        self.requests += 1;
        self.failures += 1;
        self.consecutive_failures += 1;
        self.last_failure = Some(Instant::now());
    }
}

/// Trustless Gateway block broker
pub struct TrustlessGateway {
    /// HTTP client
    client: Client,
    
    /// Gateway URLs
    gateways: Vec<Url>,
    
    /// Configuration
    config: TrustlessGatewayInit,
    
    /// Statistics per gateway
    stats: Arc<RwLock<HashMap<String, GatewayStats>>>,
    
    /// Overall broker statistics
    broker_stats: Arc<RwLock<BrokerStats>>,
}

impl TrustlessGateway {
    /// Create a new trustless gateway broker
    pub fn new(init: TrustlessGatewayInit) -> Self {
        // Build HTTP client
        let client = Client::builder()
            .timeout(Duration::from_millis(init.timeout_ms))
            .redirect(if init.allow_redirects {
                reqwest::redirect::Policy::limited(5)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()
            .expect("Failed to create HTTP client");
        
        // Initialize stats for each gateway
        let mut stats_map = HashMap::new();
        for gateway in &init.gateways {
            stats_map.insert(gateway.to_string(), GatewayStats::default());
        }
        
        Self {
            client,
            gateways: init.gateways.clone(),
            config: init,
            stats: Arc::new(RwLock::new(stats_map)),
            broker_stats: Arc::new(RwLock::new(BrokerStats::default())),
        }
    }
    
    /// Get sorted gateways by reliability
    async fn sorted_gateways(&self) -> Vec<Url> {
        let stats = self.stats.read().await;
        let mut gateways_with_scores: Vec<(Url, f64)> = self.gateways
            .iter()
            .map(|url| {
                let score = stats
                    .get(&url.to_string())
                    .map(|s| s.reliability_score())
                    .unwrap_or(0.5);
                (url.clone(), score)
            })
            .collect();
        
        // Sort by score descending (best first)
        gateways_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        gateways_with_scores.into_iter().map(|(url, _)| url).collect()
    }
    
    /// Fetch a block from a specific gateway
    async fn fetch_from_gateway(&self, gateway: &Url, cid: &Cid) -> Result<Bytes> {
        let start = Instant::now();
        
        // Construct gateway URL: {gateway}/ipfs/{cid}?format=car
        let mut url = gateway.clone();
        url.set_path(&format!("/ipfs/{}", cid));
        url.set_query(Some("format=car"));
        
        debug!("Fetching {} from gateway: {}", cid, url);
        
        // Make HTTP request
        let response = self.client
            .get(url.clone())
            .header("Accept", "application/vnd.ipld.car")
            .send()
            .await
            .map_err(|e| {
                warn!("HTTP request failed for {}: {}", url, e);
                HeliaError::other(format!("Gateway request failed: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            warn!("Gateway returned error status {} for {}", status, url);
            return Err(HeliaError::other(format!("Gateway returned status: {}", status)));
        }
        
        // Read response body
        let car_bytes = response.bytes().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            HeliaError::other(format!("Failed to read CAR data: {}", e))
        })?;
        
        debug!("Received {} bytes from gateway", car_bytes.len());
        
        // Parse CAR file
        let cursor = Cursor::new(car_bytes.to_vec());
        let mut car_reader = CarReader::new(cursor);
        
        // Read header
        car_reader.read_header().await?;
        
        // Find the requested block
        let block_data = car_reader.find_block(cid).await?
            .ok_or_else(|| HeliaError::other("Block not found in CAR response"))?;
        
        let elapsed = start.elapsed();
        debug!("Successfully fetched {} in {:?}", cid, elapsed);
        
        // Record success
        let mut stats = self.stats.write().await;
        if let Some(gw_stats) = stats.get_mut(&gateway.to_string()) {
            gw_stats.record_success(elapsed);
        }
        
        Ok(block_data)
    }
}

#[async_trait::async_trait]
impl BlockBroker for TrustlessGateway {
    async fn retrieve(&self, cid: Cid, _options: BlockRetrievalOptions) -> Result<Bytes> {
        let mut last_error = None;
        
        // Try gateways in order of reliability
        for gateway in self.sorted_gateways().await {
            for attempt in 0..self.config.max_retries {
                match self.fetch_from_gateway(&gateway, &cid).await {
                    Ok(data) => {
                        // Update broker stats
                        let mut broker_stats = self.broker_stats.write().await;
                        broker_stats.requests_made += 1;
                        broker_stats.successful_requests += 1;
                        broker_stats.last_seen = Instant::now();
                        drop(broker_stats); // Release lock
                        
                        return Ok(data);
                    }
                    Err(e) => {
                        warn!(
                            "Attempt {}/{} failed for gateway {}: {}",
                            attempt + 1,
                            self.config.max_retries,
                            gateway,
                            e
                        );
                        
                        last_error = Some(e);
                        
                        // Record failure (don't hold lock across await)
                        {
                            let mut stats = self.stats.write().await;
                            if let Some(gw_stats) = stats.get_mut(&gateway.to_string()) {
                                gw_stats.record_failure();
                            }
                        } // Lock released here
                        
                        // Wait before retry (exponential backoff)
                        if attempt + 1 < self.config.max_retries {
                            let backoff = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                            tokio::time::sleep(backoff).await;
                        }
                    }
                }
            }
        }
        
        // All gateways failed
        let mut broker_stats = self.broker_stats.write().await;
        broker_stats.requests_made += 1;
        broker_stats.failed_requests += 1;
        
        Err(last_error.unwrap_or_else(|| HeliaError::other("All gateways failed")))
    }
    
    async fn announce(&self, _cid: Cid, _data: Bytes, _options: BlockAnnounceOptions) -> Result<()> {
        // Trustless gateways don't support announcements (read-only)
        Err(HeliaError::other("Trustless gateway does not support announcements"))
    }
    
    async fn start(&self) -> Result<()> {
        debug!("Trustless gateway started with {} gateways", self.gateways.len());
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        debug!("Trustless gateway stopped");
        Ok(())
    }
    
    fn get_stats(&self) -> BrokerStats {
        // Use try_read to avoid blocking in sync context
        self.broker_stats.try_read()
            .map(|stats| stats.clone())
            .unwrap_or_default()
    }
    
    fn name(&self) -> &str {
        "TrustlessGateway"
    }
}

/// Factory function to create a trustless gateway (matches TypeScript API)
///
/// # Example
///
/// ```no_run
/// use helia_block_brokers::trustless_gateway::{trustless_gateway, TrustlessGatewayInit};
///
/// let gateway = trustless_gateway(TrustlessGatewayInit::default());
/// ```
pub fn trustless_gateway(init: TrustlessGatewayInit) -> Arc<dyn BlockBroker> {
    Arc::new(TrustlessGateway::new(init))
}
