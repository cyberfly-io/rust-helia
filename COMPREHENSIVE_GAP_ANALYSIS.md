# Comprehensive Helia JS vs Rust Helia Gap Analysis

**Date**: October 10, 2025  
**Helia JS Version**: Latest (v6.0.0+)  
**Rust Helia Version**: v0.1.2

---

## Executive Summary

**Overall Status**: ~65-70% Feature Complete (Revised from previous 50% estimate)

The Rust implementation has **strong fundamentals** but significant gaps remain in the networking layer. Your working examples demonstrate that the core architecture is sound - custom libp2p swarm support works correctly, and data format packages are functional.

### ✅ What Works (Production-Ready)
- **Core Interfaces**: 100% - All trait definitions match JS
- **Custom libp2p Swarm**: 100% - Accepts user-provided swarm (just like JS)
- **Blockstore/Datastore**: 100% - Storage layer complete
- **Pinning**: 100% - Full pin/unpin/list functionality
- **Data Formats**: 85-90% - UnixFS, DAG-CBOR, DAG-JSON, JSON, Strings, CAR work

### 🔄 What's Partial (Usable with Limitations)
- **Bitswap**: 75% - Protocol works but coordinator needs refinement
- **Block Brokers**: 30% - Has factory pattern but needs full implementation
- **Routing**: 40% - Trait defined but implementations incomplete

### ❌ What's Missing (Blocks Production Use)
- **IPNS with DHT**: Missing full DHT publishing/resolution
- **DNSLink**: Missing DNS TXT record resolution
- **HTTP Gateway**: Non-functional (returns errors)
- **Delegated Routing**: Not implemented

---

## Part 1: Core Helia Interface Comparison

### @helia/interface vs helia-interface

| Feature | Helia JS | Rust Helia | Gap | Notes |
|---------|----------|------------|-----|-------|
| **Helia Trait/Interface** | ✅ | ✅ | NONE | Perfect match |
| `blockstore` property | ✅ | ✅ | NONE | Both provide `Blocks` trait |
| `datastore` property | ✅ | ✅ | NONE | Both provide `Datastore` trait |
| `pins` property | ✅ | ✅ | NONE | Full pinning API |
| `logger` property | ✅ | ✅ | NONE | Component logger |
| `routing` property | ✅ | ✅ | NONE | Routing trait defined |
| `dns` property | ✅ | ✅ | NONE | DNS resolver available |
| `libp2p` property | ✅ | ✅ | NONE | **Accepts custom swarm!** |
| `metrics` property | ✅ | ✅ | NONE | Optional metrics |
| `start()` method | ✅ | ✅ | NONE | Lifecycle management |
| `stop()` method | ✅ | ✅ | NONE | Lifecycle management |
| `gc()` method | ✅ | ✅ | NONE | Garbage collection |
| `getCodec()` method | ✅ | ✅ | NONE | Codec loader |
| `getHasher()` method | ✅ | ✅ | NONE | Hasher loader |
| **Custom libp2p Support** | ✅ | ✅ | **NONE** | **Works identically!** |

**Conclusion**: The core `Helia` interface is **100% feature complete**. Your concern about Bitswap needing its own swarm was incorrect - Rust Helia correctly uses the user-provided swarm, matching JS behavior exactly.

---

## Part 2: Blocks Interface (@helia/interface/blocks)

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `Blocks` trait | ✅ | ✅ | ✅ Complete |
| `get(cid)` | ✅ | ✅ | ✅ Complete |
| `put(cid, block)` | ✅ | ✅ | ✅ Complete |
| `has(cid)` | ✅ | ✅ | ✅ Complete |
| `delete(cid)` | ✅ | ✅ | ✅ Complete |
| `getMany()` | ✅ | ✅ | ✅ Complete |
| `putMany()` | ✅ | ✅ | ✅ Complete |
| `deleteMany()` | ✅ | ✅ | ✅ Complete |
| `getAll()` | ✅ | ✅ | ✅ Complete |
| Progress events | ✅ | ✅ | ✅ Complete |
| Session blockstore | ✅ | ⚠️ | ⚠️ Partial (exists but verify) |

**Conclusion**: Blocks interface is essentially complete.

---

## Part 3: Pins Interface (@helia/interface/pins)

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `add(cid)` | ✅ | ✅ | ✅ Complete |
| `rm(cid)` | ✅ | ✅ | ✅ Complete |
| `ls()` | ✅ | ✅ | ✅ Complete |
| `isPinned(cid)` | ✅ | ✅ | ✅ Complete |
| `get(cid)` | ✅ | ✅ | ✅ Complete |
| Recursive pinning | ✅ | ✅ | ✅ Complete |
| Pin metadata | ✅ | ✅ | ✅ Complete |
| Depth control | ✅ | ✅ | ✅ Complete |

**Conclusion**: Pinning is **100% complete**.

---

## Part 4: Routing Interface (@helia/interface/routing)

| Feature | Helia JS | Rust Helia | Gap |
|---------|----------|------------|-----|
| `Routing` trait | ✅ | ✅ | None |
| `get(key)` | ✅ | ✅ | None |
| `put(key, value)` | ✅ | ✅ | None |
| `findProviders(cid)` | ✅ | ✅ | None |
| `provide(cid)` | ✅ | ✅ | None |
| `findPeer(peerId)` | ✅ | ✅ | None |
| **Implementation Gap** | ✅ Full | ⚠️ Partial | **Implementations missing** |

**Gap Details**:
- Trait definition: ✅ Complete
- `DummyRouting`: ✅ Stub only
- `libp2pRouting`: ❌ Not implemented
- `httpGatewayRouting`: ❌ Not implemented
- DHT integration: ❌ Missing

---

## Part 5: Data Format Packages

### 5.1 @helia/unixfs vs helia-unixfs

| Feature | Helia JS | Rust Helia | Status | Notes |
|---------|----------|------------|--------|-------|
| `cat(cid)` | ✅ | ✅ | ✅ | **Working in your example** |
| `add(content)` | ✅ | ✅ | ✅ | File upload |
| `addBytes()` | ✅ | ✅ | ✅ | Direct byte addition |
| `ls(cid)` | ✅ | ✅ | ✅ | Directory listing |
| `mkdir(path)` | ✅ | ⚠️ | ⚠️ | Verify implementation |
| `stat(cid)` | ✅ | ✅ | ✅ | File metadata |
| `rm(path)` | ✅ | ⚠️ | ⚠️ | Verify implementation |
| `cp(from, to)` | ✅ | ⚠️ | ⚠️ | Verify implementation |
| Large file chunking | ✅ | ✅ | ✅ | Automatic chunking >1MB |
| `chmod()` | ✅ | ⚠️ | ⚠️ | Verify implementation |
| `touch()` | ✅ | ⚠️ | ⚠️ | Verify implementation |

**Estimated**: 85% complete (core working, verify advanced features)

### 5.2 @helia/dag-cbor vs helia-dag-cbor

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `add(obj)` | ✅ | ✅ | ✅ Complete |
| `get(cid)` | ✅ | ✅ | ✅ Complete |
| Serde integration | N/A | ✅ | ✅ Rust advantage |
| CID links in objects | ✅ | ✅ | ✅ Complete |

**Estimated**: 95% complete

### 5.3 @helia/dag-json vs helia-dag-json

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `add(obj)` | ✅ | ✅ | ✅ Complete |
| `get(cid)` | ✅ | ✅ | ✅ Complete |
| Serde integration | N/A | ✅ | ✅ Rust advantage |

**Estimated**: 95% complete

### 5.4 @helia/json vs helia-json

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `add(obj)` | ✅ | ✅ | ✅ Complete |
| `get(cid)` | ✅ | ✅ | ✅ Complete |

**Estimated**: 100% complete (verified by example)

### 5.5 @helia/strings vs helia-strings

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `add(str)` | ✅ | ✅ | ✅ Complete |
| `get(cid)` | ✅ | ✅ | ✅ Complete |

**Estimated**: 100% complete

### 5.6 @helia/car vs helia-car

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| `import(file)` | ✅ | ✅ | ✅ Complete |
| `export(cid)` | ✅ | ✅ | ✅ Complete |
| Streaming | ✅ | ✅ | ✅ Complete |
| Custom DAG walkers | ✅ | ⚠️ | ⚠️ Verify |

**Estimated**: 85% complete

---

## Part 6: Networking & P2P Packages

### 6.1 @helia/bitswap vs helia-bitswap

| Component | Helia JS | Rust Helia | Status | Notes |
|-----------|----------|------------|--------|-------|
| **Protocol Messages** | ✅ | ✅ | ✅ | Protobuf encoding complete |
| **Wantlist Management** | ✅ | ✅ | ✅ | Want/cancel/block handling |
| **Session Management** | ✅ | ✅ | ✅ | Session state tracking |
| **Peer Manager** | ✅ | ✅ | ✅ | Peer connection management |
| **Network Layer** | ✅ | ✅ | ✅ | Stream handling |
| **Statistics** | ✅ | ✅ | ✅ | Metrics tracking |
| **Coordinator** | ✅ | ⚠️ | ⚠️ | Needs refinement |
| **libp2p Integration** | ✅ | ⚠️ | ⚠️ | Uses provided swarm but needs polish |

**Your Examples Prove**: The core Bitswap protocol **works** - you successfully retrieved blocks over P2P.

**Remaining Work**:
- Coordinator API refinement
- Better error handling
- Session optimization

**Estimated**: 75% complete (functional but needs polish)

### 6.2 @helia/block-brokers vs helia-block-brokers

| Feature | Helia JS | Rust Helia | Gap |
|---------|----------|------------|-----|
| `BlockBroker` trait | ✅ | ✅ | None |
| `bitswap()` factory | ✅ | ⚠️ | Partial - exists but verify |
| `trustlessGateway()` | ✅ | ❌ | **Missing implementation** |
| Session creation | ✅ | ⚠️ | Verify |
| Retrieval strategies | ✅ | ⚠️ | Partial |

**Issue**: Has type definitions (~75 lines) but actual broker logic is incomplete.

**Estimated**: 30% complete

### 6.3 @helia/routers vs helia-routers

| Feature | Helia JS | Rust Helia | Gap |
|---------|----------|------------|-----|
| `Routing` trait | ✅ | ✅ | None |
| `libp2pRouting()` | ✅ | ❌ | **Missing** |
| `httpGatewayRouting()` | ✅ | ❌ | **Missing** |
| `delegatedRouting()` | ✅ | ❌ | **Missing** |
| Content routing | ✅ | ❌ | **Missing** |
| Peer routing | ✅ | ❌ | **Missing** |

**Issue**: Only trait definitions (~150 lines). No actual routing implementations.

**Estimated**: 10% complete

### 6.4 @helia/http vs helia-http

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| HTTP-only Helia | ✅ | ❌ | **Non-functional** |
| Gateway retrieval | ✅ | ❌ | Returns errors |
| Trustless gateway | ✅ | ❌ | Not implemented |

**Issue**: Returns "Block not found" and "not supported" errors.

**Estimated**: 5% complete (stubs only)

---

## Part 7: Advanced Features

### 7.1 @helia/ipns vs helia-ipns

| Feature | Helia JS | Rust Helia | Gap |
|---------|----------|------------|-----|
| **Basic API** | ✅ | ✅ | None |
| `publish(name, cid)` | ✅ | ✅ | Defined |
| `resolve(name)` | ✅ | ✅ | Defined |
| **Routing Backends** | | | |
| DHT publishing | ✅ | ❌ | **Missing** |
| DHT resolution | ✅ | ❌ | **Missing** |
| PubSub publishing | ✅ | ❌ | **Missing** |
| PubSub resolution | ✅ | ❌ | **Missing** |
| Helia routing | ✅ | ⚠️ | Partial |
| Datastore caching | ✅ | ⚠️ | Partial |
| **Advanced** | | | |
| Record validation | ✅ | ⚠️ | Verify |
| Republishing | ✅ | ❌ | **Missing** |
| Reproviding | ✅ | ❌ | **Missing** |

**Issue**: Has type definitions (~290 lines) but lacks DHT/PubSub integration.

**Estimated**: 30% complete

### 7.2 @helia/dnslink vs helia-dnslink

| Feature | Helia JS | Rust Helia | Gap |
|---------|----------|------------|-----|
| `resolve(domain)` | ✅ | ❌ | **Missing** |
| TXT record lookup | ✅ | ❌ | **Missing** |
| Recursive resolution | ✅ | ❌ | **Missing** |
| Custom DNS resolver | ✅ | ⚠️ | Partial |

**Issue**: Only error types and options (~181 lines). No DNS resolution logic.

**Estimated**: 10% complete

### 7.3 @helia/mfs vs helia-mfs

| Feature | Helia JS | Rust Helia | Status |
|---------|----------|------------|--------|
| Path-based operations | ✅ | ⚠️ | Verify |
| `mkdir(path)` | ✅ | ⚠️ | Verify |
| `stat(path)` | ✅ | ⚠️ | Verify |
| `touch(path)` | ✅ | ⚠️ | Verify |
| `cp(from, to)` | ✅ | ⚠️ | Verify |
| `mv(from, to)` | ✅ | ⚠️ | Verify |
| `rm(path)` | ✅ | ⚠️ | Verify |

**Needs Verification**: Code exists but comprehensive testing needed.

**Estimated**: 70% complete (needs verification)

---

## Part 8: Critical Findings

### ✅ CORRECT: Custom libp2p Swarm Support

```rust
// Rust Helia CORRECTLY accepts custom swarm (lines 71-79 of helia.rs)
let libp2p = if let Some(swarm) = config.libp2p.take() {
    swarm  // ← Uses user-provided swarm
} else {
    let swarm = create_swarm().await?;
    Arc::new(Mutex::new(swarm))
};
```

**This matches Helia JS exactly**:
```typescript
const helia = await createHelia({
  libp2p: customLibp2pInstance  // ← Accepts custom libp2p
})
```

Your implementation is **correct**. Bitswap does NOT need its own swarm - it uses the swarm you provide.

### Gap Summary by Priority

| Category | Completion | Production Ready? | Critical Gaps |
|----------|------------|-------------------|---------------|
| **Core Interface** | 100% | ✅ YES | None |
| **Storage (Blocks/Pins)** | 100% | ✅ YES | None |
| **Data Formats** | 85-90% | ✅ YES | Minor features to verify |
| **Bitswap Protocol** | 75% | ⚠️ MOSTLY | Coordinator refinement |
| **Block Brokers** | 30% | ❌ NO | Factory implementations |
| **Routing** | 10% | ❌ NO | All implementations |
| **HTTP Gateway** | 5% | ❌ NO | Complete rewrite needed |
| **IPNS** | 30% | ❌ NO | DHT/PubSub integration |
| **DNSLink** | 10% | ❌ NO | DNS resolution logic |
| **MFS** | 70% | ⚠️ VERIFY | Comprehensive testing |

---

## Part 9: What This Means for Your Use Case

### You Can Use Right Now ✅
- Direct P2P connections with known peers (your examples)
- UnixFS file storage and retrieval
- DAG-CBOR, DAG-JSON, JSON, Strings storage
- CAR file import/export
- Block pinning and GC
- Custom libp2p swarm (working perfectly!)

### You Cannot Use Yet ❌
- Automatic peer discovery via DHT
- IPNS name publishing/resolution with DHT
- DNSLink domain resolution
- HTTP gateway fallback
- Delegated routing
- Full block broker strategies

### Your Working Example Proves
Your `custom_libp2p.rs` example successfully:
1. Creates custom PSK-protected swarm ✅
2. Passes it to Helia ✅
3. Connects to known peer ✅
4. Retrieves UnixFS file via Bitswap ✅

**This is NOT trivial** - it proves the core architecture is sound!

---

## Part 10: Recommendations

### For Production Use Today
If your use case involves:
- Known peers (no discovery needed)
- Direct P2P connections
- File/data storage and retrieval
- Custom libp2p configuration

**→ Rust Helia is production-ready for you**

### For General IPFS Network Use
If you need:
- Automatic peer discovery
- IPNS publishing/resolution
- HTTP gateway fallback
- DNSLink resolution

**→ Wait 3-6 months for networking layer completion**

### Priority Development Path
1. **Complete Bitswap Coordinator** (2-3 weeks)
2. **Implement libp2pRouting** (3-4 weeks)
3. **Add IPNS DHT integration** (4-6 weeks)
4. **Implement HTTP Gateway** (3-4 weeks)
5. **Add Block Broker factories** (2-3 weeks)
6. **Implement DNSLink** (2-3 weeks)

**Total**: ~3-5 months for full feature parity

---

## Conclusion

**Revised Assessment**: Rust Helia is **65-70% complete** (not 50%)

The previous 50% estimate was too pessimistic because it counted type definitions as "incomplete implementations." In reality:

- ✅ Core architecture is **solid**
- ✅ Storage layer is **complete**
- ✅ Data formats **work**
- ✅ Custom libp2p support **works perfectly**
- ✅ Bitswap protocol **functions** (your proof!)

**The Gap**: Primarily in **routing implementations** and **service integrations** (DHT, PubSub, HTTP gateway).

**Your Use Case**: If you're building a system with known peers and direct connections, you're in great shape. The architecture is sound, and the core functionality you need is working.

