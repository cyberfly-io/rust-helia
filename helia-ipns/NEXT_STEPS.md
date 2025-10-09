# IPNS Implementation - Next Steps

## Current Status ✅

**Completed Features**:
- ✅ IPNS record structure and validation
- ✅ V1 and V2 signature generation and verification
- ✅ Protobuf marshaling/unmarshaling with DAG-CBOR
- ✅ DHT router with user-provided libp2p pattern
- ✅ Local store for record caching
- ✅ Complete test coverage (41/41 tests passing)

**Test Results**: 100% pass rate
- Unit tests: 6/6 ✅
- DHT tests: 7/7 ✅
- Integration tests: 28/28 ✅

---

## Priority Roadmap

### 🔴 HIGH PRIORITY - Core Functionality

#### 1. DHT Event Loop Integration (3-5 days)
**Current Issue**: DHT `get()` operations don't return actual records - they initiate queries but don't process responses.

**Tasks**:
- [ ] Create DHT event handler loop
- [ ] Process `KademliaEvent::OutboundQueryProgressed` events
- [ ] Implement async channels for query results
- [ ] Handle `GetRecord` success/failure
- [ ] Handle `PutRecord` success/failure
- [ ] Add timeout handling for queries
- [ ] Update `get()` to wait for query completion
- [ ] Add comprehensive event loop tests

**Implementation Approach**:
```rust
// Spawn event loop task
pub async fn start_event_loop(&self) {
    let swarm = self.swarm.clone();
    tokio::spawn(async move {
        loop {
            let event = swarm.lock().await.select_next_some().await;
            match event {
                SwarmEvent::Behaviour(KademliaEvent::OutboundQueryProgressed { 
                    id, result, .. 
                }) => {
                    match result {
                        QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(record))) => {
                            // Send record to waiting get() call
                        }
                        // Handle other cases...
                    }
                }
            }
        }
    });
}
```

**Benefits**:
- Enables full DHT functionality
- Records can actually be resolved from the network
- Proper error handling for network failures

**Estimated Effort**: 3-5 days

---

#### 2. HTTP Router Implementation (2-3 days)
**Purpose**: Fallback routing method using IPFS HTTP gateways.

**Tasks**:
- [ ] Add `reqwest` dependency for HTTP client
- [ ] Implement `HttpRouter` struct (currently stub)
- [ ] Implement gateway publish API (POST /routing/v1/ipns/:name)
- [ ] Implement gateway resolve API (GET /routing/v1/ipns/:name)
- [ ] Handle HTTP errors gracefully
- [ ] Add retry logic for failed requests
- [ ] Support multiple gateway endpoints
- [ ] Add HTTP router tests

**Example Gateway Endpoints**:
- Public gateway: `https://ipfs.io/routing/v1/`
- Dweb.link: `https://dweb.link/routing/v1/`
- Custom gateway: User-provided URL

**Implementation**:
```rust
pub struct HttpRouter {
    client: reqwest::Client,
    gateways: Vec<String>,
}

impl IpnsRouting for HttpRouter {
    async fn put(&self, key: &[u8], record: &[u8], opts: PutOptions) -> Result<(), IpnsError> {
        for gateway in &self.gateways {
            let url = format!("{}/ipns/{}", gateway, hex::encode(key));
            match self.client.post(&url).body(record.to_vec()).send().await {
                Ok(response) if response.status().is_success() => return Ok(()),
                _ => continue, // Try next gateway
            }
        }
        Err(IpnsError::NotFound("All gateways failed".into()))
    }
}
```

**Benefits**:
- Works immediately without DHT setup
- Fallback when DHT is unavailable
- Easier for web-based applications

**Estimated Effort**: 2-3 days

---

#### 3. Connection Management & Bootstrap (2-3 days)
**Purpose**: Enable DHT to actually connect to the IPFS network.

**Tasks**:
- [ ] Add bootstrap node configuration
- [ ] Implement `connect_to_bootstrap_nodes()` helper
- [ ] Add default IPFS bootstrap nodes
- [ ] Support custom bootstrap nodes
- [ ] Implement DHT bootstrapping procedure
- [ ] Add peer discovery helpers
- [ ] Create connection health checks
- [ ] Add bootstrap tests

**Default Bootstrap Nodes**:
```rust
pub const DEFAULT_BOOTSTRAP_NODES: &[(&str, &str)] = &[
    ("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN", "/dnsaddr/bootstrap.libp2p.io"),
    ("QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa", "/dnsaddr/bootstrap.libp2p.io"),
    // ... more bootstrap nodes
];

impl DhtRouter {
    pub async fn bootstrap(&self, nodes: Option<Vec<(PeerId, Multiaddr)>>) -> Result<(), IpnsError> {
        let bootstrap_nodes = nodes.unwrap_or_else(|| self.default_bootstrap_nodes());
        
        let mut swarm = self.swarm.lock().await;
        for (peer_id, addr) in bootstrap_nodes {
            swarm.behaviour_mut().add_address(&peer_id, addr);
        }
        swarm.behaviour_mut().bootstrap()?;
        
        Ok(())
    }
}
```

**Benefits**:
- DHT can actually connect to IPFS network
- Records can be published and resolved globally
- Full IPNS functionality

**Estimated Effort**: 2-3 days

---

### 🟡 MEDIUM PRIORITY - Enhanced Functionality

#### 4. Record Republishing & TTL Management (2-3 days)
**Purpose**: Automatically republish records before they expire.

**Current State**: Basic republishing logic exists but needs enhancement.

**Tasks**:
- [ ] Implement smart republishing scheduler
- [ ] Calculate optimal republish intervals based on TTL
- [ ] Add jitter to prevent thundering herd
- [ ] Handle network failures gracefully
- [ ] Add republish metrics
- [ ] Create republishing tests
- [ ] Document republishing behavior

**Implementation**:
```rust
pub struct RepublishScheduler {
    records: HashMap<Vec<u8>, IpnsRecord>,
    router: Arc<dyn IpnsRouting>,
}

impl RepublishScheduler {
    pub async fn schedule_republish(&self, key: Vec<u8>, record: IpnsRecord) {
        let ttl = record.ttl;
        let interval = ttl * 2 / 3; // Republish at 66% of TTL
        let jitter = rand::random::<u64>() % (ttl / 10); // ±10% jitter
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(interval + jitter)).await;
                // Republish record
            }
        });
    }
}
```

**Benefits**:
- Records stay available without manual republishing
- Better user experience
- Consistent with IPFS behavior

**Estimated Effort**: 2-3 days

---

#### 5. Multi-Router Composition (1-2 days)
**Purpose**: Try multiple routers in parallel for faster resolution.

**Tasks**:
- [ ] Create `CompositeRouter` struct
- [ ] Implement parallel query to multiple routers
- [ ] Return first successful result
- [ ] Handle partial failures gracefully
- [ ] Add priority/weight system for routers
- [ ] Create composition tests

**Implementation**:
```rust
pub struct CompositeRouter {
    routers: Vec<(Arc<dyn IpnsRouting>, u8)>, // (router, priority)
}

impl IpnsRouting for CompositeRouter {
    async fn get(&self, key: &[u8], opts: GetOptions) -> Result<Vec<u8>, IpnsError> {
        // Sort by priority
        let mut routers = self.routers.clone();
        routers.sort_by_key(|(_, priority)| *priority);
        
        // Try all routers in parallel
        let futures: Vec<_> = routers.iter()
            .map(|(router, _)| router.get(key, opts.clone()))
            .collect();
        
        // Return first success
        futures::future::select_ok(futures).await
            .map(|(result, _)| result)
            .map_err(|_| IpnsError::NotFound("All routers failed".into()))
    }
}
```

**Example Usage**:
```rust
let composite = CompositeRouter::new()
    .add(dht_router, 1)      // Try DHT first
    .add(http_router, 2)     // Fallback to HTTP
    .add(local_router, 0);   // Check local cache first (highest priority)

ipns.publish_with_router(Arc::new(composite), ...);
```

**Benefits**:
- Faster resolution (first success wins)
- Better reliability (fallback options)
- Flexible routing strategies

**Estimated Effort**: 1-2 days

---

#### 6. Record Validation & Best Record Selection (1-2 days)
**Purpose**: Choose the best record when multiple versions exist.

**Current State**: Basic `select_best_record()` exists.

**Tasks**:
- [ ] Enhance validation rules
- [ ] Implement comprehensive comparison logic
- [ ] Handle clock skew gracefully
- [ ] Add malformed record handling
- [ ] Create validation tests
- [ ] Document validation rules

**Validation Rules**:
1. Signature must be valid
2. Record must not be expired
3. Higher sequence number wins
4. If same sequence, newer validity wins
5. If same validity, arbitrary tiebreaker (lexicographic)

**Benefits**:
- Prevents outdated records from being used
- Handles network partitions
- Consistent with IPNS spec

**Estimated Effort**: 1-2 days

---

### 🟢 LOW PRIORITY - Polish & Optimization

#### 7. Metrics & Observability (2-3 days)
**Tasks**:
- [ ] Add Prometheus metrics
- [ ] Track publish/resolve success rates
- [ ] Measure query latencies
- [ ] Count DHT peers
- [ ] Monitor record cache hit rates
- [ ] Add tracing spans
- [ ] Create metrics dashboard

**Metrics to Track**:
- `ipns_publish_total` (counter)
- `ipns_publish_duration_seconds` (histogram)
- `ipns_resolve_total` (counter)
- `ipns_resolve_duration_seconds` (histogram)
- `ipns_dht_peers_total` (gauge)
- `ipns_cache_hit_rate` (gauge)

**Estimated Effort**: 2-3 days

---

#### 8. Persistent Storage (2-3 days)
**Tasks**:
- [ ] Add RocksDB backend for Kademlia store
- [ ] Implement persistent local store
- [ ] Add database migrations
- [ ] Support configurable storage backends
- [ ] Add storage tests

**Benefits**:
- Records persist across restarts
- Faster cold starts
- Better offline support

**Estimated Effort**: 2-3 days

---

#### 9. Advanced DHT Features (3-4 days)
**Tasks**:
- [ ] Implement provider records
- [ ] Add accelerated DHT lookups
- [ ] Implement optimistic provides
- [ ] Add custom DHT protocols
- [ ] Create advanced DHT tests

**Estimated Effort**: 3-4 days

---

#### 10. Documentation & Examples (2-3 days)
**Tasks**:
- [ ] Create comprehensive API documentation
- [ ] Write usage guide
- [ ] Add code examples:
  - Simple publish/resolve
  - Custom router implementation
  - Multi-router setup
  - Bootstrap node configuration
- [ ] Create tutorial series
- [ ] Add architecture diagrams
- [ ] Document common pitfalls

**Estimated Effort**: 2-3 days

---

## Recommended Implementation Order

### Phase 1: Core Network Functionality (Week 1-2)
1. **DHT Event Loop Integration** ← Start here (most critical)
2. **Connection Management & Bootstrap**
3. **HTTP Router Implementation**

**Goal**: Enable full publish/resolve workflow over the network.

### Phase 2: Reliability & Performance (Week 3)
4. **Multi-Router Composition**
5. **Record Validation Enhancement**
6. **Record Republishing**

**Goal**: Make IPNS reliable and performant.

### Phase 3: Production Readiness (Week 4)
7. **Metrics & Observability**
8. **Persistent Storage**
9. **Documentation & Examples**

**Goal**: Production-ready IPNS implementation.

### Phase 4: Advanced Features (Future)
10. **Advanced DHT Features**
11. **PubSub Router** (alternative routing method)
12. **Custom Key Types** (beyond peer IDs)

---

## Quick Wins (Can Do Now)

These are small improvements that can be done quickly:

1. **Fix Module Visibility** (30 minutes)
   - Export routing module properly in lib.rs
   - Ensure all public types are accessible

2. **Add Unused Router Methods** (1 hour)
   - Remove unused function warnings
   - Mark intentionally unused items with `#[allow(dead_code)]`

3. **Clean Up Imports** (30 minutes)
   - Remove unused imports
   - Organize use statements

4. **Add More Inline Documentation** (1-2 hours)
   - Document all public functions
   - Add examples to doc comments

---

## Testing Strategy

For each new feature:

1. **Unit Tests**: Test individual components
2. **Integration Tests**: Test end-to-end workflows
3. **Network Tests**: Test with real DHT (optional, can be slow)
4. **Interop Tests**: Test with Go/JS IPNS (future)

**Target**: Maintain 100% test pass rate throughout.

---

## Dependencies to Add

```toml
# For HTTP router
reqwest = { version = "0.12", features = ["json"] }

# For metrics
prometheus = "0.13"

# For persistent storage
rocksdb = "0.22"

# For better error handling
color-eyre = "0.6"

# For better logging
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## Success Criteria

**Phase 1 Complete** when:
- ✅ DHT records can be published and resolved
- ✅ Bootstrap nodes can be connected
- ✅ HTTP gateway fallback works
- ✅ All tests pass

**Phase 2 Complete** when:
- ✅ Multi-router composition works
- ✅ Records are automatically republished
- ✅ Best record selection handles edge cases
- ✅ All tests pass

**Phase 3 Complete** when:
- ✅ Metrics are collected and exportable
- ✅ Records persist across restarts
- ✅ Documentation is comprehensive
- ✅ Examples demonstrate all features

---

## Next Immediate Step

**Start with: DHT Event Loop Integration**

This is the most critical missing piece. Once this is complete, the DHT router will be fully functional and records can actually be published/resolved over the network.

**Command to begin**:
```bash
# Create a new file for the event loop
touch helia-ipns/src/dht_event_loop.rs

# Add it to lib.rs
echo "mod dht_event_loop;" >> helia-ipns/src/lib.rs
```

Would you like me to start implementing the DHT event loop integration now?
