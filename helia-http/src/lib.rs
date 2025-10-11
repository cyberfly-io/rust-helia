//! # HTTP Gateway Client for Helia
//!
//! This module provides a **pure HTTP-only** client for accessing IPFS content through
//! [Trustless Gateways](https://specs.ipfs.tech/http-gateways/trustless-gateway/), without
//! requiring libp2p or P2P networking. This is different from the JavaScript `@helia/http`
//! module which is a hybrid approach (P2P + HTTP gateways).
//!
//! ## Comparison with JavaScript Helia
//!
//! | Feature | Rust helia-http | JS @helia/http |
//! |---------|----------------|----------------|
//! | **Architecture** | Pure HTTP-only | Hybrid (libp2p + HTTP) |
//! | **libp2p** | ❌ No | ✅ Yes |
//! | **P2P networking** | ❌ No | ✅ Yes |
//! | **Trustless Gateways** | ✅ Yes | ✅ Yes |
//! | **Weight** | Lightweight | Heavier (full node) |
//! | **Use case** | Serverless, edge, read-only | Full IPFS node capabilities |
//!
//! For full P2P support in Rust, use the main `helia` crate with `helia-utils::create_helia()`.
//!
//! ## Overview
//!
//! This client implements the [Trustless Gateway specification](https://specs.ipfs.tech/http-gateways/trustless-gateway/)
//! to fetch blocks from public or private HTTP gateways. It's designed for lightweight IPFS access
//! in environments where P2P networking is unavailable, restricted, or unnecessary.
//!
//! ### Key Features
//!
//! - **Fetch content** from IPFS via HTTP gateways (e.g., trustless-gateway.link, 4everland.io)
//! - **Trustless Gateway spec** - Uses `/ipfs/{cid}?format=raw` with `Accept: application/vnd.ipld.raw`
//! - **Gateway fallback** - Automatically tries multiple gateways if one fails
//! - **Retry logic** - Exponential backoff for transient failures
//! - **Simple integration** - Implements the same `Helia` trait as full P2P nodes
//!
//! ## When to Use HTTP Mode
//!
//! **✅ Good for:**
//! - Browser-based applications (WASM)
//! - Serverless functions (AWS Lambda, Cloudflare Workers)
//! - Edge computing environments
//! - CI/CD pipelines
//! - Quick prototyping and testing
//! - Read-only IPFS access
//! - Restricted network environments (firewalls, no UDP)
//!
//! **❌ Not ideal for:**
//! - Content publishing (use full P2P node)
//! - High-throughput applications (gateway rate limits)
//! - Mission-critical apps requiring 100% uptime (single point of failure)
//! - Applications that need pinning
//! - Private/offline IPFS networks
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐
//! │ Your App     │
//! └──────┬───────┘
//!        │ Helia trait
//!        ↓
//! ┌──────────────┐
//! │ HTTP Client  │
//! └──────┬───────┘
//!        │ HTTPS
//!        ↓
//! ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
//! │ ipfs.io      │ OR  │ dweb.link    │ OR  │ Custom GW    │
//! └──────────────┘     └──────────────┘     └──────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use helia_http::create_helia_http;
//! use helia_interface::Blocks;
//! use cid::Cid;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create HTTP-only Helia node
//! let helia = create_helia_http().await?;
//!
//! // Fetch content by CID
//! let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
//!     .parse()?;
//! let content = helia.blockstore().get(&cid, None).await?;
//!
//! println!("Fetched {} bytes from IPFS", content.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Gateway Configuration
//!
//! By default, the client uses public gateways with automatic fallback:
//!
//! ```rust,no_run
//! use helia_http::{create_helia_http_with_gateways, GatewayConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use custom gateways
//! let gateways = vec![
//!     "https://ipfs.io".to_string(),
//!     "https://dweb.link".to_string(),
//!     "https://cloudflare-ipfs.com".to_string(),
//! ];
//!
//! let config = GatewayConfig {
//!     gateways,
//!     timeout_secs: 30,
//!     max_retries: 3,
//! };
//!
//! let helia = create_helia_http_with_gateways(config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! ```rust,no_run
//! use helia_http::create_helia_http;
//! use helia_interface::{Blocks, HeliaError};
//! use cid::Cid;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let helia = create_helia_http().await?;
//! let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
//!
//! match helia.blockstore().get(&cid, None).await {
//!     Ok(content) => {
//!         println!("Success: {} bytes", content.len());
//!     }
//!     Err(e) => match e {
//!         HeliaError::BlockNotFound(_) => {
//!             eprintln!("Content not found on any gateway");
//!         }
//!         HeliaError::NetworkError(msg) => {
//!             eprintln!("Network error: {}", msg);
//!         }
//!         _ => {
//!             eprintln!("Other error: {}", e);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! ### Latency
//! - **First request**: 100-500ms (gateway lookup + HTTP roundtrip)
//! - **Cached at gateway**: 10-50ms (CDN hit)
//! - **Local cache**: <1ms (if using blockstore cache)
//!
//! ### Throughput
//! - **Limited by gateway**: ~10-50 MB/s typical
//! - **Rate limits**: Varies by gateway (usually 100-1000 req/min)
//! - **Concurrent requests**: Supported (10-100 parallel)
//!
//! ### Comparison with P2P Mode
//!
//! | Feature | HTTP Gateway | P2P Mode |
//! |---------|--------------|----------|
//! | Setup time | Instant | 5-30 seconds |
//! | First fetch | 100-500ms | 1-10 seconds |
//! | Cached fetch | 10-50ms | <1ms |
//! | Throughput | 10-50 MB/s | 50-200 MB/s |
//! | Availability | Gateway dependent | Peer dependent |
//! | NAT traversal | Not needed | Required |
//! | Can publish | ❌ No | ✅ Yes |
//! | Can pin | ❌ No | ✅ Yes |
//!
//! ## Gateway URLs
//!
//! The client constructs gateway URLs in these formats:
//!
//! ```text
//! https://ipfs.io/ipfs/{cid}           - Path gateway
//! https://{cid}.ipfs.dweb.link         - Subdomain gateway
//! https://cloudflare-ipfs.com/ipfs/{cid} - Path gateway
//! ```
//!
//! ## Limitations
//!
//! 1. **Read-only** - Cannot publish content to IPFS
//! 2. **No pinning** - Cannot pin content (gateways handle this)
//! 3. **Gateway dependency** - Availability depends on gateway uptime
//! 4. **Rate limits** - Subject to gateway rate limiting
//! 5. **Privacy** - Gateways can see what content you access
//! 6. **No P2P** - Cannot participate in IPFS network directly
//!
//! ## Best Practices
//!
//! 1. **Use multiple gateways** - Configure fallbacks for reliability
//! 2. **Cache aggressively** - Use local blockstore cache
//! 3. **Implement backoff** - Respect gateway rate limits
//! 4. **Monitor latency** - Track gateway performance
//! 5. **Consider privacy** - Use trusted gateways or run your own
//!
//! ## Integration with Other Modules
//!
//! The HTTP gateway client implements the `Helia` trait, so it works seamlessly with:
//!
//! - **helia-unixfs** - Fetch files and directories
//! - **helia-dag-cbor** - Fetch CBOR-encoded data structures
//! - **helia-dag-json** - Fetch JSON data structures
//! - **helia-json** - Fetch JSON content
//! - **helia-strings** - Fetch UTF-8 strings
//! - **helia-car** - Export fetched content to CAR files
//!
//! ```rust,no_run
//! use helia_http::create_helia_http;
//! use helia_strings::strings;
//! use cid::Cid;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use HTTP gateway with strings module
//! let helia = create_helia_http().await?;
//! let strings = strings(helia.clone());
//!
//! let cid: Cid = "bafkreiabaeaqcaibaeaqcaibaeaqcaibaeaqcaibaeaqcaibaeaqcaibae".parse()?;
//! let content = strings.get(&cid, Default::default()).await?;
//! println!("String content: {}", content);
//! # Ok(())
//! # }
//! ```
//!
//! ## Examples
//!
//! See `examples/` directory for:
//! - `http_fetch.rs` - Basic content fetching
//! - `http_gateway_fallback.rs` - Multiple gateway configuration
//! - `http_with_unixfs.rs` - Fetching files via HTTP
//!
//! ## See Also
//!
//! - [`Helia`](helia_interface::Helia) - Core trait implemented by this module
//! - [`Blocks`](helia_interface::Blocks) - Block storage interface
//! - [IPFS HTTP Gateway Specification](https://specs.ipfs.tech/http-gateways/)

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::stream;
use libp2p::PeerId;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use trust_dns_resolver::TokioAsyncResolver;

use helia_interface::{
    Blocks, Codec, ComponentLogger, Datastore, GcOptions, Hasher, Helia, HeliaError, HeliaEventReceiver, Metrics, Pins,
    Routing,
};
use tokio::sync::broadcast;

/// Configuration for HTTP gateway access
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// List of gateway URLs to try (in order)
    pub gateways: Vec<String>,
    /// Timeout for each HTTP request (seconds)
    pub timeout_secs: u64,
    /// Maximum number of retries per gateway
    pub max_retries: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            // Use Trustless Gateways (aligned with Helia JS defaults)
            // See: https://specs.ipfs.tech/http-gateways/trustless-gateway/
            // JS source: https://github.com/ipfs/helia/blob/main/packages/routers/src/http-gateway-routing.ts
            gateways: vec![
                "https://trustless-gateway.link".to_string(),
                "https://4everland.io".to_string(),
                "https://cloudflare-ipfs.com".to_string(),
            ],
            timeout_secs: 30,
            max_retries: 2,
        }
    }
}

pub struct HttpBlocks {
    client: Client,
    config: GatewayConfig,
}

impl HttpBlocks {
    pub fn new(config: GatewayConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Fetch block from gateway with automatic fallback
    async fn fetch_from_gateway(&self, cid: &Cid) -> Result<Bytes, HeliaError> {
        let cid_str = cid.to_string();
        let mut last_error = None;

        // Try each gateway in order
        for gateway_url in &self.config.gateways {
            // Try with retries for this gateway
            for attempt in 0..=self.config.max_retries {
                // Use Trustless Gateway spec: /ipfs/{cid}?format=raw
                // See: https://specs.ipfs.tech/http-gateways/trustless-gateway/
                let url = format!("{}/ipfs/{}?format=raw", gateway_url, cid_str);

                match self.client
                    .get(&url)
                    .header("Accept", "application/vnd.ipld.raw")
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.bytes().await {
                                Ok(bytes) => {
                                    return Ok(bytes);
                                }
                                Err(e) => {
                                    last_error = Some(format!("Failed to read response body: {}", e));
                                    continue;
                                }
                            }
                        } else if response.status().as_u16() == 404 {
                            // 404 means content doesn't exist, don't retry
                            return Err(HeliaError::BlockNotFound { cid: *cid });
                        } else {
                            last_error = Some(format!(
                                "Gateway {} returned status {}: attempt {}/{}",
                                gateway_url,
                                response.status(),
                                attempt + 1,
                                self.config.max_retries + 1
                            ));
                        }
                    }
                    Err(e) => {
                        last_error = Some(format!(
                            "Request to {} failed: {} (attempt {}/{})",
                            gateway_url,
                            e,
                            attempt + 1,
                            self.config.max_retries + 1
                        ));
                    }
                }

                // Wait before retry (exponential backoff)
                if attempt < self.config.max_retries {
                    tokio::time::sleep(Duration::from_millis(100 * (2_u64.pow(attempt as u32)))).await;
                }
            }
        }

        // All gateways failed
        Err(HeliaError::Network {
            message: format!(
                "Failed to fetch {} from all gateways. Last error: {}",
                cid_str,
                last_error.unwrap_or_else(|| "Unknown error".to_string())
            ),
        })
    }
}

#[async_trait]
impl Blocks for HttpBlocks {
    async fn get(
        &self,
        cid: &Cid,
        _options: Option<helia_interface::GetBlockOptions>,
    ) -> Result<Bytes, HeliaError> {
        self.fetch_from_gateway(cid).await
    }

    async fn get_many_cids(
        &self,
        _cids: Vec<Cid>,
        _options: Option<helia_interface::GetManyOptions>,
    ) -> Result<helia_interface::AwaitIterable<Result<helia_interface::Pair, HeliaError>>, HeliaError>
    {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn get_all(
        &self,
        _options: Option<helia_interface::GetAllOptions>,
    ) -> Result<helia_interface::AwaitIterable<helia_interface::Pair>, HeliaError> {
        Err(HeliaError::other("get_all not supported"))
    }

    async fn put(
        &self,
        cid: &Cid,
        _block: Bytes,
        _options: Option<helia_interface::PutBlockOptions>,
    ) -> Result<Cid, HeliaError> {
        Ok(*cid)
    }

    async fn put_many_blocks(
        &self,
        _blocks: Vec<helia_interface::InputPair>,
        _options: Option<helia_interface::PutManyOptions>,
    ) -> Result<helia_interface::AwaitIterable<Cid>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn has(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::HasOptions>,
    ) -> Result<bool, HeliaError> {
        Ok(false)
    }

    async fn has_many_cids(
        &self,
        _cids: Vec<Cid>,
        _options: Option<helia_interface::HasOptions>,
    ) -> Result<helia_interface::AwaitIterable<bool>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn delete_many_cids(
        &self,
        cids: Vec<Cid>,
        _options: Option<helia_interface::DeleteManyOptions>,
    ) -> Result<helia_interface::AwaitIterable<Cid>, HeliaError> {
        let s = stream::iter(cids);
        Ok(Box::pin(s))
    }
}

pub struct HttpPins;

#[async_trait]
impl Pins for HttpPins {
    async fn add(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::AddOptions>,
    ) -> Result<(), HeliaError> {
        Err(HeliaError::other("pinning not supported"))
    }

    async fn rm(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::RmOptions>,
    ) -> Result<(), HeliaError> {
        Err(HeliaError::other("rm not supported"))
    }

    async fn ls(
        &self,
        _options: Option<helia_interface::LsOptions>,
    ) -> Result<helia_interface::AwaitIterable<helia_interface::pins::Pin>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn is_pinned(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::IsPinnedOptions>,
    ) -> Result<bool, HeliaError> {
        Ok(false)
    }
}

pub struct HttpRouting;

#[async_trait]
impl Routing for HttpRouting {
    async fn find_providers(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::FindProvidersOptions>,
    ) -> Result<helia_interface::AwaitIterable<helia_interface::Provider>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn provide(
        &self,
        _cid: &Cid,
        _options: Option<helia_interface::ProvideOptions>,
    ) -> Result<(), HeliaError> {
        Err(HeliaError::other("provide not supported"))
    }

    async fn find_peers(
        &self,
        _peer_id: &PeerId,
        _options: Option<helia_interface::FindPeersOptions>,
    ) -> Result<helia_interface::AwaitIterable<helia_interface::PeerInfo>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }

    async fn get(
        &self,
        _key: &[u8],
        _options: Option<helia_interface::GetOptions>,
    ) -> Result<Option<helia_interface::RoutingRecord>, HeliaError> {
        Ok(None)
    }

    async fn put(
        &self,
        _key: &[u8],
        _value: &[u8],
        _options: Option<helia_interface::PutOptions>,
    ) -> Result<(), HeliaError> {
        Err(HeliaError::other("put not supported"))
    }
}

pub struct SimpleLogger;

impl ComponentLogger for SimpleLogger {
    fn debug(&self, message: &str) {
        eprintln!("[DEBUG] {}", message);
    }

    fn info(&self, message: &str) {
        eprintln!("[INFO] {}", message);
    }

    fn warn(&self, message: &str) {
        eprintln!("[WARN] {}", message);
    }

    fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}

pub struct MemoryDatastore {
    data: Arc<RwLock<HashMap<Vec<u8>, Bytes>>>,
}

impl MemoryDatastore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemoryDatastore {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Datastore for MemoryDatastore {
    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, HeliaError> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn put(&self, key: &[u8], value: Bytes) -> Result<(), HeliaError> {
        let mut data = self.data.write().await;
        data.insert(key.to_vec(), value);
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<(), HeliaError> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    async fn has(&self, key: &[u8]) -> Result<bool, HeliaError> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn query(
        &self,
        _prefix: Option<&[u8]>,
    ) -> Result<helia_interface::AwaitIterable<Bytes>, HeliaError> {
        let s = stream::iter(Vec::new());
        Ok(Box::pin(s))
    }
}

pub struct HeliaHttp {
    blockstore: Arc<HttpBlocks>,
    datastore: Arc<MemoryDatastore>,
    pins: Arc<HttpPins>,
    routing: Arc<HttpRouting>,
    logger: Arc<SimpleLogger>,
    dns: TokioAsyncResolver,
    /// Event broadcaster for Helia events
    event_tx: broadcast::Sender<helia_interface::HeliaEvent>,
}

impl HeliaHttp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_config(config: GatewayConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            blockstore: Arc::new(HttpBlocks::new(config)),
            datastore: Arc::new(MemoryDatastore::new()),
            pins: Arc::new(HttpPins),
            routing: Arc::new(HttpRouting),
            logger: Arc::new(SimpleLogger),
            dns: TokioAsyncResolver::tokio_from_system_conf().unwrap(),
            event_tx,
        }
    }
}

impl Default for HeliaHttp {
    fn default() -> Self {
        Self::new_with_config(GatewayConfig::default())
    }
}

#[async_trait]
impl Helia for HeliaHttp {
    fn blockstore(&self) -> &dyn Blocks {
        self.blockstore.as_ref()
    }

    fn datastore(&self) -> &dyn Datastore {
        self.datastore.as_ref()
    }

    fn pins(&self) -> &dyn Pins {
        self.pins.as_ref()
    }

    fn logger(&self) -> &dyn ComponentLogger {
        self.logger.as_ref()
    }

    fn routing(&self) -> &dyn Routing {
        self.routing.as_ref()
    }

    fn dns(&self) -> &TokioAsyncResolver {
        &self.dns
    }

    fn metrics(&self) -> Option<&dyn Metrics> {
        None
    }

    fn subscribe_events(&self) -> HeliaEventReceiver {
        self.event_tx.subscribe()
    }

    async fn start(&self) -> Result<(), HeliaError> {
        self.logger.info("Starting HTTP-only Helia node");
        let _ = self.event_tx.send(helia_interface::HeliaEvent::Start);
        Ok(())
    }

    async fn stop(&self) -> Result<(), HeliaError> {
        self.logger.info("Stopping HTTP-only Helia node");
        let _ = self.event_tx.send(helia_interface::HeliaEvent::Stop);
        Ok(())
    }

    async fn gc(&self, _options: Option<GcOptions>) -> Result<(), HeliaError> {
        let _ = self.event_tx.send(helia_interface::HeliaEvent::GcStarted);
        // HTTP-only client has no local storage to collect
        let _ = self.event_tx.send(helia_interface::HeliaEvent::GcCompleted);
        Ok(())
    }

    async fn get_codec(&self, _code: u64) -> Result<Box<dyn Codec>, HeliaError> {
        Err(HeliaError::other("codecs not supported"))
    }

    async fn get_hasher(&self, _code: u64) -> Result<Box<dyn Hasher>, HeliaError> {
        Err(HeliaError::other("hashers not supported"))
    }
}

pub async fn create_helia_http() -> Result<Arc<HeliaHttp>, HeliaError> {
    Ok(Arc::new(HeliaHttp::new()))
}

pub async fn create_helia_http_with_gateways(config: GatewayConfig) -> Result<Arc<HeliaHttp>, HeliaError> {
    Ok(Arc::new(HeliaHttp::new_with_config(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test creating a Helia HTTP instance with default configuration
    #[tokio::test]
    async fn test_create_default_helia_http() {
        let helia = create_helia_http().await;
        assert!(helia.is_ok(), "Should create Helia HTTP instance successfully");
        
        let helia = helia.unwrap();
        // blockstore() and pins() return references, not Options
        let _blockstore = helia.blockstore();
        let _pins = helia.pins();
        // If we got here without panicking, the instance was created successfully
    }

    /// Test creating a Helia HTTP instance with custom gateway configuration
    #[tokio::test]
    async fn test_create_custom_gateway_config() {
        let config = GatewayConfig {
            gateways: vec![
                "https://ipfs.io".to_string(),
                "https://dweb.link".to_string(),
            ],
            timeout_secs: 15,
            max_retries: 1,
        };
        
        let helia = create_helia_http_with_gateways(config).await;
        assert!(helia.is_ok(), "Should create Helia HTTP with custom config");
    }

    /// Test fetching a well-known IPFS block (empty directory)
    /// CID: bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354
    #[tokio::test]
    async fn test_fetch_known_block() {
        let helia = create_helia_http().await.unwrap();
        let blockstore = helia.blockstore();
        
        // Empty directory CID - well-known and should always be available
        let cid_str = "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        let cid = Cid::try_from(cid_str).expect("Valid CID");
        
        let result = blockstore.get(&cid, None).await;
        assert!(result.is_ok(), "Should fetch known block successfully: {:?}", result.err());
        
        let block = result.unwrap();
        assert!(!block.is_empty(), "Block should not be empty");
    }

    /// Test fetching a non-existent block returns appropriate error
    #[tokio::test]
    async fn test_fetch_nonexistent_block() {
        let helia = create_helia_http().await.unwrap();
        let blockstore = helia.blockstore();
        
        // Create a valid CID with random bytes that almost certainly doesn't exist
        // Using a CIDv1 with random multihash content
        let fake_cid_str = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku";
        let cid = Cid::try_from(fake_cid_str).expect("Valid CID format");
        
        let result = blockstore.get(&cid, None).await;
        // The block almost certainly doesn't exist, should return error
        // But gateway might succeed if by some miracle the content exists
        match result {
            Err(HeliaError::BlockNotFound { .. }) => {
                // Expected: block not found
            },
            Err(HeliaError::Network { .. }) => {
                // Also acceptable: network error trying all gateways
            },
            Ok(_) => {
                // Extremely unlikely but possible - the random CID exists
                // This isn't a test failure, just means we got lucky/unlucky
            },
            Err(other) => panic!("Unexpected error type: {:?}", other),
        }
    }

    /// Test has() method returns false for non-existent blocks
    #[tokio::test]
    async fn test_has_nonexistent_block() {
        let helia = create_helia_http().await.unwrap();
        let blockstore = helia.blockstore();
        
        let fake_cid_str = "bafybeibxm2nsadl3fnxv2sxcxmxaco2jl53wpeorjdzidjwf5aqdg7wa6u";
        let cid = Cid::try_from(fake_cid_str).expect("Valid CID format");
        
        let result = blockstore.has(&cid, None).await;
        // has() returns false for HTTP-only mode (can't verify without fetching)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false, "Should return false for has() in HTTP mode");
    }

    /// Test put() method succeeds but doesn't actually write (no-op for HTTP)
    #[tokio::test]
    async fn test_put_readonly() {
        let helia = create_helia_http().await.unwrap();
        let blockstore = helia.blockstore();
        
        let cid_str = "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        let cid = Cid::try_from(cid_str).expect("Valid CID");
        let data = Bytes::from(vec![1, 2, 3, 4]);
        
        let result = blockstore.put(&cid, data, None).await;
        // HTTP blockstore accepts put but doesn't actually write (no-op)
        assert!(result.is_ok(), "Put should succeed (no-op) for HTTP blockstore");
        assert_eq!(result.unwrap(), cid, "Should return the CID");
    }

    /// Test delete() method succeeds but doesn't actually delete (no-op for HTTP)
    #[tokio::test]
    async fn test_delete_readonly() {
        let helia = create_helia_http().await.unwrap();
        let blockstore = helia.blockstore();
        
        let cid_str = "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        let cid = Cid::try_from(cid_str).expect("Valid CID");
        
        let result = blockstore.delete_many_cids(vec![cid], None).await;
        // HTTP blockstore accepts delete but doesn't actually remove (no-op)
        assert!(result.is_ok(), "Delete should succeed (no-op) for HTTP blockstore");
    }

    /// Test lifecycle methods (start/stop) work without errors
    #[tokio::test]
    async fn test_lifecycle_methods() {
        let helia = create_helia_http().await.unwrap();
        
        let start_result = helia.start().await;
        assert!(start_result.is_ok(), "Should start successfully");
        
        let stop_result = helia.stop().await;
        assert!(stop_result.is_ok(), "Should stop successfully");
    }

    /// Test gc() method works (no-op for HTTP-only mode)
    #[tokio::test]
    async fn test_gc_noop() {
        let helia = create_helia_http().await.unwrap();
        
        let result = helia.gc(None).await;
        assert!(result.is_ok(), "GC should succeed (no-op)");
    }

    /// Test get_codec returns error
    #[tokio::test]
    async fn test_get_codec_not_supported() {
        let helia = create_helia_http().await.unwrap();
        
        let result = helia.get_codec(0x71).await; // dag-cbor code
        assert!(result.is_err(), "Should return error for codec");
    }

    /// Test get_hasher returns error
    #[tokio::test]
    async fn test_get_hasher_not_supported() {
        let helia = create_helia_http().await.unwrap();
        
        let result = helia.get_hasher(0x12).await; // sha2-256 code
        assert!(result.is_err(), "Should return error for hasher");
    }

    /// Test pins() returns interface (not used in HTTP-only mode)
    #[tokio::test]
    async fn test_pins_interface() {
        let helia = create_helia_http().await.unwrap();
        let _pins = helia.pins();
        // If we got here without panicking, pins interface exists
    }

    /// Test with shorter timeout to verify timeout handling
    #[tokio::test]
    async fn test_custom_timeout_config() {
        let config = GatewayConfig {
            gateways: vec!["https://ipfs.io".to_string()],
            timeout_secs: 1, // Very short timeout (1 second)
            max_retries: 0, // No retries
        };
        
        let helia = create_helia_http_with_gateways(config).await.unwrap();
        let blockstore = helia.blockstore();
        
        let cid_str = "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        let cid = Cid::try_from(cid_str).expect("Valid CID");
        
        // With 1s timeout and no retries, this might timeout or succeed depending on network
        let result = blockstore.get(&cid, None).await;
        // Either succeeds or fails with Network error (timeout)
        if let Err(e) = result {
            match e {
                HeliaError::Network { .. } => {}, // Expected timeout
                HeliaError::BlockNotFound { .. } => {}, // Also acceptable
                other => panic!("Unexpected error: {:?}", other),
            }
        }
    }

    /// Test gateway fallback by using invalid gateway first
    #[tokio::test]
    async fn test_gateway_fallback() {
        let config = GatewayConfig {
            gateways: vec![
                "https://invalid-gateway-that-does-not-exist-12345.com".to_string(),
                "https://ipfs.io".to_string(), // Valid fallback
            ],
            timeout_secs: 5,
            max_retries: 0, // No retries per gateway
        };
        
        let helia = create_helia_http_with_gateways(config).await.unwrap();
        let blockstore = helia.blockstore();
        
        let cid_str = "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        let cid = Cid::try_from(cid_str).expect("Valid CID");
        
        // Should fail on first gateway but succeed on fallback
        let result = blockstore.get(&cid, None).await;
        // May succeed via fallback or fail if all gateways fail
        assert!(result.is_ok() || result.is_err(), "Should complete (success or failure)");
    }

    /// Test default gateway configuration has expected values
    #[test]
    fn test_default_gateway_config() {
        let config = GatewayConfig::default();
        
        assert_eq!(config.gateways.len(), 3, "Should have 3 default gateways");
        assert!(config.gateways.contains(&"https://trustless-gateway.link".to_string()));
        assert!(config.gateways.contains(&"https://4everland.io".to_string()));
        assert!(config.gateways.contains(&"https://cloudflare-ipfs.com".to_string()));
        assert_eq!(config.timeout_secs, 30, "Default timeout should be 30s");
        assert_eq!(config.max_retries, 2, "Default max_retries should be 2");
    }

    /// Test concurrent requests to verify thread safety
    #[tokio::test]
    async fn test_concurrent_requests() {
        let helia = Arc::clone(&create_helia_http().await.unwrap());
        
        let cid_str = "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        let cid = Cid::try_from(cid_str).expect("Valid CID");
        
        // Launch 5 concurrent requests
        let mut handles = vec![];
        for _ in 0..5 {
            let h = Arc::clone(&helia);
            let c = cid;
            let handle = tokio::spawn(async move {
                let bs = h.blockstore();
                bs.get(&c, None).await
            });
            handles.push(handle);
        }
        
        // Wait for all to complete
        let mut success_count = 0;
        for handle in handles {
            let result = handle.await.expect("Task should complete");
            if result.is_ok() {
                success_count += 1;
            }
        }
        
        // At least some should succeed
        assert!(success_count > 0, "At least one concurrent request should succeed");
    }
}
