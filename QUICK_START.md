# Quick Start: Begin Implementation

## Start Here: Package Implementation Order

Based on dependencies and critical path, implement in this exact order:

### Phase 1: Foundation (Week 1-2)

#### Day 1-3: helia-car (CAR File Support)
**Why first:** Required by trustless gateway for fetching blocks

**Files to create:**
```
helia-car/src/
├── reader.rs      // Parse CAR files
├── writer.rs      // Generate CAR files  
└── lib.rs         // Public API
```

**Start command:**
```bash
cd helia-car/src
# Implementation in PACKAGE_PLANS.md
```

**Test with:**
```rust
let car_bytes = fetch_from_gateway("https://ipfs.io/ipfs/{cid}?format=car");
let mut reader = CarReader::new(&car_bytes)?;
while let Some((cid, block)) = reader.next_block()? {
    println!("Block: {}", cid);
}
```

#### Day 4-7: helia-block-brokers/trustless-gateway
**Why next:** Core functionality for HTTP-based fetching

**Files to create:**
```
helia-block-brokers/src/
├── lib.rs
├── trustless_gateway/
│   ├── mod.rs       // Factory function
│   ├── gateway.rs   // HTTP client
│   ├── reliability.rs
│   └── session.rs
└── bitswap.rs       // Factory (uses existing helia-bitswap)
```

**Start command:**
```bash
cd helia-block-brokers/src
mkdir trustless_gateway
# See detailed implementation in PACKAGE_PLANS.md
```

**Test with:**
```rust
let broker = trustless_gateway(TrustlessGatewayInit::default());
let block = broker.retrieve(&cid, Default::default()).await?;
assert_eq!(block.len() > 0, true);
```

### Phase 2: Routing (Week 2-3)

#### Day 8-10: helia-routers (All 3 implementations)

**Priority order:**
1. **delegated_http_routing** (easiest, most useful)
2. **http_gateway_routing** (simple)
3. **libp2p_routing** (can be basic stub initially)

**Files to create:**
```
helia-routers/src/
├── lib.rs
├── delegated_http_routing.rs   // HTTP API client
├── http_gateway_routing.rs     // Gateway-based routing
└── libp2p_routing.rs           // Libp2p wrapper
```

**Start with delegated_http_routing:**
```rust
// Test against real delegated routing API
let router = delegated_http_routing("https://delegated-ipfs.dev", Default::default());
let providers = router.find_providers(&cid).await?;
assert!(providers.len() > 0);
```

#### Day 11-13: helia-dnslink

**Files to create:**
```
helia-dnslink/src/
├── lib.rs          // dnslink() factory
├── dnslink.rs      // DnsLink struct
├── resolver.rs     // DNS-over-HTTPS
└── parser.rs       // TXT record parsing
```

**Test with:**
```rust
let resolver = dnslink(helia.clone(), Default::default());
let result = resolver.resolve("docs.ipfs.tech").await?;
println!("Resolved to: {}", result.cid);
```

### Phase 3: Enhanced Features (Week 3-4)

#### Day 14-18: helia-ipns (with routing)

**Files to enhance:**
```
helia-ipns/src/
├── lib.rs                    // ipns() factory
├── routing/
│   ├── mod.rs
│   ├── helia.rs             // Use helia.routing
│   └── pubsub.rs            // PubSub integration
└── signing.rs               // Proper signatures
```

**Test with:**
```rust
let name = ipns(helia.clone(), IpnsOptions {
    routers: vec![helia_routing(helia.routing())],
});
name.publish("key-1", &cid).await?;
```

#### Day 19-21: helia-http (Complete rewrite)

**Files to replace:**
```
helia-http/src/
├── lib.rs                // create_helia_http()
└── utils/
    ├── libp2p.rs        // Lightweight libp2p
    └── defaults.rs      // HTTP-specific defaults
```

**Test with:**
```rust
let helia = create_helia_http(Default::default()).await?;
let fs = unixfs(helia);
let content = fs.cat(&cid).await?;
```

### Phase 4: Integration (Week 4-5)

#### Day 22-24: helia (Main package restructuring)

**Files to update:**
```
helia/src/
├── lib.rs                        // create_helia() factory
└── utils/
    ├── helia_defaults.rs         // Default config
    └── libp2p_defaults.rs        // libp2p config
```

**Target API:**
```rust
use rust_helia::create_helia;
use helia_block_brokers::{trustless_gateway, bitswap};
use helia_routers::{delegated_http_routing, http_gateway_routing};

let helia = create_helia(HeliaInit {
    block_brokers: vec![
        trustless_gateway(Default::default()),
        bitswap(Default::default()),
    ],
    routers: vec![
        delegated_http_routing("https://delegated-ipfs.dev", Default::default()),
        http_gateway_routing(Default::default()),
    ],
    ..Default::default()
}).await?;
```

#### Day 25-28: Testing & Examples

**Create examples:**
```
examples/
├── 01_http_fetch.rs              // Basic HTTP fetching
├── 02_dnslink.rs                 // DNSLink resolution
├── 03_ipns.rs                    // IPNS operations
├── 04_custom_gateways.rs         // Custom configuration
└── 05_complete_usage.rs          // Full feature demo
```

---

## Step-by-Step: Start With TrustlessGateway

### 1. Update Cargo.toml

```bash
cd helia-block-brokers
```

Edit `Cargo.toml`:
```toml
[dependencies]
helia-interface = { version = "0.1.2", path = "../helia-interface" }
helia-car = { path = "../helia-car" }
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

### 2. Create File Structure

```bash
mkdir -p src/trustless_gateway
touch src/trustless_gateway/mod.rs
touch src/trustless_gateway/gateway.rs
touch src/trustless_gateway/reliability.rs
touch src/bitswap.rs
```

### 3. Implement lib.rs

```rust
// helia-block-brokers/src/lib.rs
pub mod trustless_gateway;
pub mod bitswap;

// Re-export factory functions
pub use trustless_gateway::trustless_gateway;
pub use bitswap::bitswap;

// Re-export types
pub use trustless_gateway::TrustlessGatewayInit;
pub use bitswap::BitswapInit;
```

### 4. Implement trustless_gateway/mod.rs

Copy implementation from PACKAGE_PLANS.md, section "trustless_gateway/mod.rs"

### 5. Implement trustless_gateway/gateway.rs

Copy implementation from PACKAGE_PLANS.md, section "trustless_gateway/gateway.rs"

### 6. Test It

```bash
cargo test --package helia-block-brokers
```

### 7. Try Real Fetching

```rust
// examples/test_trustless_gateway.rs
use helia_block_brokers::trustless_gateway;

#[tokio::main]
async fn main() {
    let broker = trustless_gateway(Default::default());
    
    // Well-known test CID
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    match broker.retrieve(&cid, Default::default()).await {
        Ok(block) => println!("✅ Fetched {} bytes", block.len()),
        Err(e) => println!("❌ Error: {}", e),
    }
}
```

Run:
```bash
cargo run --example test_trustless_gateway
```

---

## Checkpoint System

After completing each package, verify with these checkpoints:

### ✓ helia-car
- [ ] Can parse CAR v1 format
- [ ] Can extract specific block from CAR
- [ ] Handles invalid CAR gracefully
- [ ] Tests pass

### ✓ helia-block-brokers
- [ ] Fetches from public gateways
- [ ] Reliability tracking works
- [ ] Fallback to next gateway on failure
- [ ] Returns correct block data
- [ ] Tests pass

### ✓ helia-routers
- [ ] Delegated routing finds providers
- [ ] HTTP gateway routing returns URLs
- [ ] Libp2p routing (basic) compiles
- [ ] Tests pass

### ✓ helia-dnslink
- [ ] Resolves real domains (docs.ipfs.tech)
- [ ] Follows CNAME chains
- [ ] Caches results
- [ ] Tests pass

### ✓ helia-ipns
- [ ] Publishes to routing
- [ ] Resolves from routing
- [ ] Multiple routing strategies work
- [ ] Tests pass

### ✓ helia-http
- [ ] create_helia_http() works
- [ ] Can fetch content via HTTP
- [ ] Integrates with routers
- [ ] Tests pass

### ✓ helia (main)
- [ ] create_helia() factory works
- [ ] BlockBrokers integrate
- [ ] Routers integrate
- [ ] Matches TypeScript API
- [ ] All tests pass

---

## Development Commands

### Build everything
```bash
cargo build --workspace
```

### Test specific package
```bash
cargo test --package helia-block-brokers
```

### Run example
```bash
cargo run --example http_fetch
```

### Check compilation
```bash
cargo check --workspace
```

### Format code
```bash
cargo fmt --all
```

### Run clippy
```bash
cargo clippy --workspace -- -D warnings
```

---

## When You Get Stuck

### Issue: CAR parsing fails
**Solution:** Check CAR v1 spec at https://ipld.io/specs/transport/car/carv1/

### Issue: HTTP fetch times out
**Solution:** Increase timeout in TrustlessGatewayInit or try different gateway

### Issue: Gateway returns 404
**Solution:** Verify CID exists with: `curl https://ipfs.io/ipfs/{cid}?format=car -I`

### Issue: DNS resolution fails
**Solution:** Test DNS-over-HTTPS manually with: 
```bash
curl 'https://cloudflare-dns.com/dns-query?name=_dnslink.example.com&type=TXT'
```

### Issue: Tests fail with network errors
**Solution:** Some tests require internet. Use `#[ignore]` for CI or mock responses

---

## Next Action

**I recommend starting with helia-car** (2-3 days) as it's foundational and self-contained.

Would you like me to:
1. **Implement helia-car completely** (reader + writer)?
2. **Implement trustless_gateway** (full working implementation)?
3. **Create a specific package** (which one)?

Let me know and I'll start coding!
