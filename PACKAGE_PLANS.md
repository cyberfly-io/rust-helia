# Detailed Package-by-Package Implementation Plans

## Package 1: helia-block-brokers

### Current State
- 75 lines of type definitions only
- Structs: `BlockRetrievalOptions`, `BlockAnnounceOptions`, `ProviderType`, `BrokerStats`
- No actual broker implementations

### Target API (TypeScript Pattern)
```rust
use helia_block_brokers::{trustless_gateway, bitswap, TrustlessGatewayInit};

let helia = create_helia(HeliaInit {
    block_brokers: vec![
        trustless_gateway(TrustlessGatewayInit {
            gateways: vec!["https://ipfs.io".into()],
            ..Default::default()
        }),
        bitswap(Default::default()),
    ],
    ..Default::default()
}).await?;
```

### File Structure
```
helia-block-brokers/
├── src/
│   ├── lib.rs                          // Public exports
│   ├── bitswap.rs                      // bitswap() factory
│   ├── types.rs                        // Shared types
│   └── trustless_gateway/
│       ├── mod.rs                      // trustless_gateway() factory
│       ├── gateway.rs                  // TrustlessGateway implementation
│       ├── session.rs                  // Session tracking
│       ├── reliability.rs              // Gateway reliability tracking
│       └── car_fetcher.rs              // CAR file fetching
└── Cargo.toml
```

### Implementation Details

#### 1. lib.rs (Public API)
```rust
// Export factory functions
pub use bitswap::bitswap;
pub use trustless_gateway::trustless_gateway;

// Export init types
pub use bitswap::BitswapInit;
pub use trustless_gateway::TrustlessGatewayInit;

// Export trait for block brokers
pub use helia_interface::BlockBroker;
```

#### 2. trustless_gateway/mod.rs (Factory Function)
```rust
use std::sync::Arc;
use helia_interface::BlockBroker;

#[derive(Debug, Clone)]
pub struct TrustlessGatewayInit {
    pub gateways: Vec<String>,
    pub allow_insecure: bool,
    pub allow_local: bool,
    pub timeout_ms: u64,
    pub max_retry_attempts: u32,
}

impl Default for TrustlessGatewayInit {
    fn default() -> Self {
        Self {
            gateways: vec![
                "https://trustless-gateway.link".into(),
                "https://ipfs.io".into(),
            ],
            allow_insecure: false,
            allow_local: false,
            timeout_ms: 30_000,
            max_retry_attempts: 3,
        }
    }
}

/// Create a trustless gateway block broker
pub fn trustless_gateway(init: TrustlessGatewayInit) -> Arc<dyn BlockBroker> {
    Arc::new(TrustlessGateway::new(init))
}
```

#### 3. trustless_gateway/gateway.rs (Core Implementation)
```rust
use async_trait::async_trait;
use helia_interface::{BlockBroker, HeliaError};
use cid::Cid;
use bytes::Bytes;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TrustlessGateway {
    client: Client,
    gateways: Vec<String>,
    reliability: Arc<RwLock<ReliabilityTracker>>,
    config: TrustlessGatewayInit,
}

impl TrustlessGateway {
    pub fn new(config: TrustlessGatewayInit) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(config.timeout_ms))
                .build()
                .unwrap(),
            gateways: config.gateways.clone(),
            reliability: Arc::new(RwLock::new(ReliabilityTracker::new())),
            config,
        }
    }

    async fn fetch_car_from_gateway(
        &self,
        gateway: &str,
        cid: &Cid,
    ) -> Result<Bytes, HeliaError> {
        // Build URL: {gateway}/ipfs/{cid}?format=car
        let url = format!("{}/ipfs/{}?format=car", gateway, cid);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.ipld.car")
            .send()
            .await
            .map_err(|e| HeliaError::network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(HeliaError::block_not_found(cid));
        }

        let body = response.bytes().await
            .map_err(|e| HeliaError::network(e.to_string()))?;

        Ok(body)
    }

    async fn extract_block_from_car(
        &self,
        car_bytes: Bytes,
        target_cid: &Cid,
    ) -> Result<Bytes, HeliaError> {
        // Parse CAR file and extract the specific block
        use helia_car::CarReader;
        
        let mut reader = CarReader::new(car_bytes.as_ref())?;
        
        while let Some((cid, block)) = reader.next_block().await? {
            if &cid == target_cid {
                return Ok(Bytes::from(block));
            }
        }
        
        Err(HeliaError::block_not_found(target_cid))
    }
}

#[async_trait]
impl BlockBroker for TrustlessGateway {
    async fn retrieve(
        &self,
        cid: &Cid,
        options: BlockRetrievalOptions,
    ) -> Result<Bytes, HeliaError> {
        // Get ordered list of gateways by reliability
        let gateways = self.reliability.read().await.get_ordered_gateways(&self.gateways);

        for gateway in gateways {
            for attempt in 0..self.config.max_retry_attempts {
                match self.fetch_car_from_gateway(&gateway, cid).await {
                    Ok(car_bytes) => {
                        match self.extract_block_from_car(car_bytes, cid).await {
                            Ok(block) => {
                                // Record success
                                self.reliability.write().await.record_success(&gateway);
                                return Ok(block);
                            }
                            Err(e) => {
                                // CAR parsing failed, try next gateway
                                self.reliability.write().await.record_failure(&gateway);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        if attempt == self.config.max_retry_attempts - 1 {
                            self.reliability.write().await.record_failure(&gateway);
                        }
                        // Retry with exponential backoff
                        tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempt))).await;
                    }
                }
            }
        }

        Err(HeliaError::block_not_found(cid))
    }

    async fn retrieve_many(
        &self,
        cids: Vec<Cid>,
        options: BlockRetrievalOptions,
    ) -> Vec<Result<Bytes, HeliaError>> {
        // Fetch blocks in parallel
        let futures = cids.iter().map(|cid| self.retrieve(cid, options.clone()));
        futures::future::join_all(futures).await
    }
}
```

#### 4. trustless_gateway/reliability.rs (Gateway Reliability)
```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ReliabilityTracker {
    stats: HashMap<String, GatewayStats>,
}

struct GatewayStats {
    success_count: u64,
    failure_count: u64,
    last_failure: Option<Instant>,
    avg_response_time: Duration,
}

impl ReliabilityTracker {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    pub fn record_success(&mut self, gateway: &str) {
        let stats = self.stats.entry(gateway.to_string()).or_insert_with(|| GatewayStats {
            success_count: 0,
            failure_count: 0,
            last_failure: None,
            avg_response_time: Duration::from_secs(1),
        });
        stats.success_count += 1;
    }

    pub fn record_failure(&mut self, gateway: &str) {
        let stats = self.stats.entry(gateway.to_string()).or_insert_with(|| GatewayStats {
            success_count: 0,
            failure_count: 0,
            last_failure: None,
            avg_response_time: Duration::from_secs(1),
        });
        stats.failure_count += 1;
        stats.last_failure = Some(Instant::now());
    }

    pub fn get_ordered_gateways(&self, gateways: &[String]) -> Vec<String> {
        let mut scored: Vec<_> = gateways.iter().map(|gw| {
            let score = self.calculate_score(gw);
            (gw.clone(), score)
        }).collect();

        // Sort by score (higher is better)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.into_iter().map(|(gw, _)| gw).collect()
    }

    fn calculate_score(&self, gateway: &str) -> f64 {
        let stats = self.stats.get(gateway);
        match stats {
            None => 1.0, // No data, neutral score
            Some(s) => {
                let total = (s.success_count + s.failure_count) as f64;
                if total == 0.0 {
                    return 1.0;
                }
                
                let success_rate = s.success_count as f64 / total;
                
                // Penalize recent failures
                let recency_penalty = if let Some(last_failure) = s.last_failure {
                    let age = Instant::now().duration_since(last_failure).as_secs() as f64;
                    if age < 60.0 {
                        0.5 // Recent failure, heavy penalty
                    } else if age < 300.0 {
                        0.8 // Somewhat recent
                    } else {
                        1.0 // Old failure, no penalty
                    }
                } else {
                    1.0
                };

                success_rate * recency_penalty
            }
        }
    }
}
```

#### 5. bitswap.rs (Bitswap Factory)
```rust
use std::sync::Arc;
use helia_interface::BlockBroker;
use helia_bitswap::BitswapBlockBroker;

#[derive(Debug, Clone, Default)]
pub struct BitswapInit {
    pub max_providers: usize,
    pub want_list_size: usize,
}

/// Create a bitswap block broker
pub fn bitswap(init: BitswapInit) -> Arc<dyn BlockBroker> {
    Arc::new(BitswapBlockBroker::new(init))
}
```

### Dependencies to Add
```toml
[dependencies]
helia-interface = { version = "0.1.2", path = "../helia-interface" }
helia-car = { path = "../helia-car" }
helia-bitswap = { path = "../helia-bitswap" }
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { workspace = true, features = ["full"] }
async-trait = { workspace = true }
cid = { workspace = true }
bytes = { workspace = true }
futures = "0.3"
url = "2.5"
serde = { version = "1.0", features = ["derive"] }
thiserror = { workspace = true }
```

### Testing Strategy
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_from_public_gateway() {
        let gateway = trustless_gateway(TrustlessGatewayInit {
            gateways: vec!["https://ipfs.io".into()],
            ..Default::default()
        });

        // Test CID for a small file
        let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
            .parse()
            .unwrap();

        let result = gateway.retrieve(&cid, Default::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reliability_tracking() {
        let tracker = ReliabilityTracker::new();
        // Test that reliability tracking works
    }
}
```

### Estimated Effort
- **Core implementation**: 2-3 days
- **Reliability tracking**: 1 day
- **CAR integration**: 1 day (depends on helia-car completion)
- **Testing**: 1-2 days
- **Total**: ~1 week

---

## Package 2: helia-routers

### Current State
- 150 lines of trait definitions only
- Traits: `Router`, types: `ProviderInfo`, `PeerInfo`, `RoutingError`
- No actual router implementations

### Target API
```rust
use helia_routers::{
    delegated_http_routing,
    http_gateway_routing,
    libp2p_routing,
};

let helia = create_helia(HeliaInit {
    routers: vec![
        delegated_http_routing("https://delegated-ipfs.dev", Default::default()),
        http_gateway_routing(Default::default()),
        libp2p_routing(libp2p),
    ],
    ..Default::default()
}).await?;
```

### File Structure
```
helia-routers/
├── src/
│   ├── lib.rs                          // Public exports
│   ├── delegated_http_routing.rs       // Delegated HTTP router
│   ├── http_gateway_routing.rs         // HTTP gateway router
│   ├── libp2p_routing.rs               // Libp2p DHT router
│   └── utils/
│       ├── mod.rs
│       └── delegated_http_routing_defaults.rs
└── Cargo.toml
```

### Implementation Details

#### 1. lib.rs (Public Exports)
```rust
pub use delegated_http_routing::{delegated_http_routing, DelegatedHTTPRoutingInit};
pub use http_gateway_routing::{http_gateway_routing, HTTPGatewayRoutingInit};
pub use libp2p_routing::{libp2p_routing, Libp2pRoutingInit};

pub use helia_interface::{Router, ProviderInfo, PeerInfo};
```

#### 2. delegated_http_routing.rs (Delegated Routing V1 API)
```rust
use async_trait::async_trait;
use helia_interface::{Router, ProviderInfo, HeliaError};
use cid::Cid;
use reqwest::Client;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DelegatedHTTPRoutingInit {
    pub endpoints: Vec<String>,
    pub timeout_ms: u64,
}

impl Default for DelegatedHTTPRoutingInit {
    fn default() -> Self {
        Self {
            endpoints: vec!["https://delegated-ipfs.dev".into()],
            timeout_ms: 30_000,
        }
    }
}

pub fn delegated_http_routing(
    url: &str,
    init: DelegatedHTTPRoutingInit,
) -> Arc<dyn Router> {
    let mut init = init;
    init.endpoints = vec![url.to_string()];
    Arc::new(DelegatedHTTPRouter::new(init))
}

pub struct DelegatedHTTPRouter {
    client: Client,
    endpoints: Vec<String>,
}

impl DelegatedHTTPRouter {
    pub fn new(init: DelegatedHTTPRoutingInit) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_millis(init.timeout_ms))
                .build()
                .unwrap(),
            endpoints: init.endpoints,
        }
    }

    async fn fetch_providers_from_endpoint(
        &self,
        endpoint: &str,
        cid: &Cid,
    ) -> Result<Vec<ProviderInfo>, HeliaError> {
        // Delegated Routing V1 API: GET /routing/v1/providers/{cid}
        let url = format!("{}/routing/v1/providers/{}", endpoint, cid);

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| HeliaError::network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(HeliaError::routing_failed("No providers found".into()));
        }

        #[derive(Deserialize)]
        struct ProviderResponse {
            #[serde(rename = "Providers")]
            providers: Vec<ProviderRecord>,
        }

        #[derive(Deserialize)]
        struct ProviderRecord {
            #[serde(rename = "Protocol")]
            protocol: String,
            #[serde(rename = "Schema")]
            schema: String,
            #[serde(rename = "ID")]
            id: Option<String>,
            #[serde(rename = "Addrs")]
            addrs: Option<Vec<String>>,
        }

        let data: ProviderResponse = response.json().await
            .map_err(|e| HeliaError::routing_failed(e.to_string()))?;

        let providers = data.providers.into_iter()
            .filter_map(|p| {
                Some(ProviderInfo {
                    protocol: p.protocol,
                    addrs: p.addrs.unwrap_or_default(),
                })
            })
            .collect();

        Ok(providers)
    }
}

#[async_trait]
impl Router for DelegatedHTTPRouter {
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<ProviderInfo>, HeliaError> {
        // Try each endpoint until one succeeds
        for endpoint in &self.endpoints {
            if let Ok(providers) = self.fetch_providers_from_endpoint(endpoint, cid).await {
                if !providers.is_empty() {
                    return Ok(providers);
                }
            }
        }

        Ok(vec![]) // Return empty if no providers found
    }

    async fn provide(&self, cid: &Cid) -> Result<(), HeliaError> {
        // Delegated routing doesn't support providing
        Err(HeliaError::not_supported("provide not supported"))
    }

    async fn get_record(&self, key: &[u8]) -> Result<Vec<u8>, HeliaError> {
        // GET /routing/v1/ipns/{key}
        let key_str = multibase::encode(multibase::Base::Base64Url, key);
        
        for endpoint in &self.endpoints {
            let url = format!("{}/routing/v1/ipns/{}", endpoint, key_str);
            
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.bytes().await {
                        return Ok(data.to_vec());
                    }
                }
            }
        }

        Err(HeliaError::not_found("Record not found"))
    }

    async fn put_record(&self, key: &[u8], value: &[u8]) -> Result<(), HeliaError> {
        // Delegated routing doesn't support putting records
        Err(HeliaError::not_supported("put_record not supported"))
    }
}
```

#### 3. http_gateway_routing.rs (Gateway-based Routing)
```rust
use async_trait::async_trait;
use helia_interface::{Router, ProviderInfo, HeliaError};
use cid::Cid;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct HTTPGatewayRoutingInit {
    pub gateways: Vec<String>,
}

impl Default for HTTPGatewayRoutingInit {
    fn default() -> Self {
        Self {
            gateways: vec![
                "https://ipfs.io".into(),
                "https://dweb.link".into(),
            ],
        }
    }
}

pub fn http_gateway_routing(init: HTTPGatewayRoutingInit) -> Arc<dyn Router> {
    Arc::new(HTTPGatewayRouter::new(init))
}

pub struct HTTPGatewayRouter {
    gateways: Vec<String>,
}

impl HTTPGatewayRouter {
    pub fn new(init: HTTPGatewayRoutingInit) -> Self {
        Self {
            gateways: init.gateways,
        }
    }
}

#[async_trait]
impl Router for HTTPGatewayRouter {
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<ProviderInfo>, HeliaError> {
        // Return gateway URLs as "providers"
        let providers = self.gateways.iter().map(|gw| {
            ProviderInfo {
                protocol: "transport-ipfs-gateway-http".into(),
                addrs: vec![format!("{}/ipfs/{}", gw, cid)],
            }
        }).collect();

        Ok(providers)
    }

    async fn provide(&self, _cid: &Cid) -> Result<(), HeliaError> {
        Err(HeliaError::not_supported("HTTP gateways cannot provide"))
    }

    async fn get_record(&self, _key: &[u8]) -> Result<Vec<u8>, HeliaError> {
        Err(HeliaError::not_supported("HTTP gateways cannot get records"))
    }

    async fn put_record(&self, _key: &[u8], _value: &[u8]) -> Result<(), HeliaError> {
        Err(HeliaError::not_supported("HTTP gateways cannot put records"))
    }
}
```

#### 4. libp2p_routing.rs (Libp2p DHT Integration)
```rust
use async_trait::async_trait;
use helia_interface::{Router, ProviderInfo, HeliaError};
use cid::Cid;
use libp2p::Swarm;
use std::sync::Arc;

pub struct Libp2pRoutingInit;

pub fn libp2p_routing(libp2p: Arc<Swarm<_>>) -> Arc<dyn Router> {
    Arc::new(Libp2pRouter::new(libp2p))
}

pub struct Libp2pRouter {
    libp2p: Arc<Swarm<_>>,
}

impl Libp2pRouter {
    pub fn new(libp2p: Arc<Swarm<_>>) -> Self {
        Self { libp2p }
    }
}

#[async_trait]
impl Router for Libp2pRouter {
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<ProviderInfo>, HeliaError> {
        // Use libp2p Kademlia DHT to find providers
        // This requires access to the DHT behavior
        todo!("Implement DHT provider lookup")
    }

    async fn provide(&self, cid: &Cid) -> Result<(), HeliaError> {
        // Announce to DHT that we provide this CID
        todo!("Implement DHT provide")
    }

    async fn get_record(&self, key: &[u8]) -> Result<Vec<u8>, HeliaError> {
        // Get record from DHT
        todo!("Implement DHT get")
    }

    async fn put_record(&self, key: &[u8], value: &[u8]) -> Result<(), HeliaError> {
        // Put record to DHT
        todo!("Implement DHT put")
    }
}
```

### Dependencies
```toml
[dependencies]
helia-interface = { version = "0.1.2", path = "../helia-interface" }
reqwest = { version = "0.12", features = ["json"] }
tokio = { workspace = true }
async-trait = { workspace = true }
cid = { workspace = true }
libp2p = { workspace = true }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
multibase = "0.9"
thiserror = { workspace = true }
```

### Estimated Effort
- **Delegated HTTP**: 2 days
- **HTTP Gateway**: 1 day
- **Libp2p (basic)**: 2-3 days
- **Testing**: 1-2 days
- **Total**: ~1 week

---

## Package 3: helia-dnslink

### Current State
- 181 lines of type definitions
- Types: `DnsLinkError`, `DnsLinkResult`, `ResolveOptions`
- No actual DNS resolution

### Target API
```rust
use helia_dnslink::dnslink;

let resolver = dnslink(helia.clone(), Default::default());
let result = resolver.resolve("example.com").await?;
println!("Resolved to: {}", result.cid);
```

### File Structure
```
helia-dnslink/
├── src/
│   ├── lib.rs                  // dnslink() factory
│   ├── dnslink.rs              // DnsLink struct
│   ├── resolver.rs             // DNS resolution logic
│   ├── parser.rs               // TXT record parsing
│   └── cache.rs                // DNS cache
└── Cargo.toml
```

### Implementation Summary
- DNS-over-HTTPS client (using Cloudflare/Google DNS)
- TXT record parsing for `_dnslink.example.com`
- CNAME chain following
- Recursive resolution
- Cache with TTL

### Estimated Effort: 3-4 days

---

## Package 4: helia-ipns

### Current State
- 290 lines with basic local storage
- Missing: DHT, PubSub, routing strategies
- Comment in code: "// In full implementation would: Sign, Publish DHT, PubSub"

### Target API
```rust
use helia_ipns::{ipns, IpnsOptions};
use helia_ipns::routing::{helia_routing, pubsub};

let name = ipns(helia.clone(), IpnsOptions {
    routers: vec![
        helia_routing(helia.routing()),
        pubsub(helia.clone()),
    ],
});

let result = name.publish("key-1", &cid).await?;
let resolved = name.resolve(&result.public_key).await?;
```

### Key Additions Needed
1. **src/routing/helia.rs** - Use helia.routing for IPNS
2. **src/routing/pubsub.rs** - PubSub integration
3. **Record signing** - Proper Ed25519 signatures
4. **DHT publishing** - Via helia.routing
5. **Caching** - With TTL support

### Estimated Effort: 4-5 days

---

## Package 5: helia-http

### Current State
- 282 lines of stubs returning errors
- All methods return `Err("not supported")`

### Target API
```rust
use helia_http::create_helia_http;

let helia = create_helia_http(Default::default()).await?;
let fs = unixfs(helia);
```

### Key Work
- Replace stub HttpBlocks with functional implementation
- Use trustless_gateway for block fetching
- Integrate with routers for provider discovery
- Lightweight libp2p setup

### Estimated Effort: 3-4 days

---

## Summary Timeline

| Package | Effort | Dependencies | Can Start |
|---------|--------|--------------|-----------|
| helia-car | 2-3 days | None | ✅ Immediately |
| helia-block-brokers | 1 week | helia-car | After CAR |
| helia-routers | 1 week | None | ✅ Immediately |
| helia-dnslink | 3-4 days | None | ✅ Immediately |
| helia-ipns | 4-5 days | helia-routers | After routers |
| helia-http | 3-4 days | block-brokers, routers | After both |
| helia (main) | 2-3 days | All above | Final integration |

**Total: 6-8 weeks for MVP (HTTP-only functionality)**

Would you like me to start implementing any specific package?
