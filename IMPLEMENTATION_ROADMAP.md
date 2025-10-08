# Implementation Roadmap: Matching TypeScript Helia API

## Overview

To achieve full feature parity with TypeScript Helia while matching the exact API structure, we need to:

1. **Restructure all packages to use factory functions** (like TypeScript)
2. **Implement missing networking functionality** (HTTP clients, DNS, routing)
3. **Add DHT/PubSub integration** for IPNS
4. **Complete session management** for block brokers
5. **Create comprehensive tests** matching TypeScript test patterns

## Estimated Timeline: 6-8 Months

### Phase 1: Foundation & HTTP Infrastructure (2 months)

**Week 1-2: HTTP Gateway & CAR Support**
- [ ] Implement CAR file parser/writer with full spec support
- [ ] Create HTTP client with retry logic, timeouts
- [ ] Add trustless gateway protocol implementation
- [ ] Session management for gateway requests

**Week 3-4: Trustless Gateway Block Broker**
- [ ] Complete TrustlessGateway implementation
- [ ] Reliability tracking (error rates per gateway)
- [ ] Gateway rotation and fallback logic
- [ ] Integration with helia-car for CAR fetching

**Week 5-6: HTTP Routing**
- [ ] Delegated HTTP Routing V1 API client
- [ ] HTTP Gateway Routing implementation  
- [ ] Provider lookup via HTTP
- [ ] IPNS resolution via HTTP gateways

**Week 7-8: DNS Resolution**
- [ ] DNS-over-HTTPS client
- [ ] TXT record parsing for DNSLink
- [ ] CNAME following and recursive resolution
- [ ] Caching layer for DNS responses

### Phase 2: Networking & Routing (2 months)

**Week 9-10: Libp2p Routing Integration**
- [ ] libp2pRouting implementation
- [ ] DHT integration for content routing
- [ ] Provider queries via Kademlia DHT
- [ ] Peer routing and discovery

**Week 11-12: IPNS with DHT**
- [ ] IPNS record publishing to DHT
- [ ] IPNS resolution via DHT
- [ ] Record signing and verification
- [ ] TTL and expiration handling

**Week 13-14: IPNS with PubSub**
- [ ] PubSub routing implementation
- [ ] Topic subscription for IPNS updates
- [ ] Real-time IPNS record propagation
- [ ] Multi-router strategy (DHT + PubSub + Datastore)

**Week 15-16: Complete Bitswap**
- [ ] Finish session coordinator
- [ ] libp2p protocol handler
- [ ] Want-list broadcasting
- [ ] Block exchange optimization

### Phase 3: API Restructuring (1 month)

**Week 17-18: Factory Function Pattern**
- [ ] Restructure helia (main) to factory pattern
- [ ] Implement create_helia() matching TS signature
- [ ] Add helia_defaults() and libp2p_defaults()
- [ ] BlockBroker and Router as trait objects

**Week 19-20: Package-level Factories**
- [ ] Convert all data format packages to factories
- [ ] Ensure unixfs(helia), dag_cbor(helia) pattern
- [ ] Update exports in all package lib.rs files
- [ ] Create builder patterns for complex inits

### Phase 4: Integration & Testing (2 months)

**Week 21-24: Integration Testing**
- [ ] End-to-end tests with real IPFS network
- [ ] Gateway fetching tests with public gateways
- [ ] DHT publishing/resolution tests
- [ ] IPNS lifecycle tests
- [ ] DNSLink resolution tests

**Week 25-26: Interoperability Testing**
- [ ] Test against js-helia nodes
- [ ] Test against Kubo nodes  
- [ ] Cross-implementation content exchange
- [ ] Verify CAR file compatibility

**Week 27-28: Performance & Optimization**
- [ ] Benchmark against TypeScript version
- [ ] Optimize hot paths
- [ ] Memory usage profiling
- [ ] Connection pooling for HTTP clients

**Week 29-32: Documentation & Examples**
- [ ] Complete API documentation
- [ ] Migration guide from TypeScript
- [ ] Usage examples for all packages
- [ ] Update HONEST_STATUS.md to 95%+ complete

## Critical Path Dependencies

```
Foundation:
  CAR Support → TrustlessGateway → HTTP Block Broker
  DNS Resolution → DNSLink

Networking:
  Libp2p Integration → DHT Routing → IPNS DHT
  Bitswap Completion → Full P2P Support

API:
  Factory Pattern → All Packages Restructured
  Integration Tests → Verify Compatibility
```

## Minimum Viable Implementation (MVP) - 3 Months

If we need faster delivery, focus on:

1. **HTTP-only Helia** (like @helia/http)
   - Trustless Gateway block broker ✓
   - HTTP Gateway routing ✓
   - Delegated HTTP routing ✓
   - DNSLink resolution ✓
   - Factory function API ✓

2. **Data Format Support**
   - UnixFS complete ✓
   - DAG-CBOR/JSON/JSON ✓
   - Factory pattern APIs ✓

3. **Skip for MVP:**
   - Full libp2p DHT integration
   - PubSub routing
   - Bitswap completion
   - Complex session management

This gives you a **functional, HTTP-based IPFS client** in 3 months that:
- Fetches content from gateways
- Resolves DNSLink domains
- Works with UnixFS files
- Matches TypeScript API exactly

Then add P2P features in subsequent releases.

## Quick Start: Implementing First Factory Function

Let me show you how to implement the first complete piece - **trustless_gateway** factory:

### 1. Update helia-block-brokers Cargo.toml

```toml
[dependencies]
helia-interface = { version = "0.1.2", path = "../helia-interface" }
helia-car = { version = "0.1.0", path = "../helia-car" }
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

### 2. Implement TrustlessGateway

See TYPESCRIPT_API_MATCHING.md for full structure. Key files:

- `src/trustless_gateway/mod.rs` - Factory function
- `src/trustless_gateway/gateway.rs` - HTTP client logic
- `src/trustless_gateway/session.rs` - Session tracking
- `src/lib.rs` - Public exports

This is the pattern to follow for all implementations.

## Next Steps

1. **Review TYPESCRIPT_API_MATCHING.md** - Understand factory pattern
2. **Choose timeline** - MVP (3 months) or Full (6-8 months)
3. **Start with HTTP infrastructure** - Foundation for everything
4. **Implement incrementally** - Test each piece thoroughly
5. **Match TypeScript API exactly** - Users should feel at home

Would you like me to:
- Implement the MVP first (HTTP-only, 3 months)?
- Start with full implementation (all features, 6-8 months)?
- Focus on specific package first?
