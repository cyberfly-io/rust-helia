# Comparison: Rust helia-http vs JavaScript @helia/http

**Date:** October 2025

## Architecture Differences

### JavaScript @helia/http (Hybrid Approach)

The JavaScript version is **NOT** a pure HTTP-only client. Instead, it's a full Helia node with libp2p that **prefers** HTTP gateways for block fetching:

```typescript
// JS creates a full Helia node with libp2p
const helia = await createHeliaHTTP({
  blockBrokers: [
    trustlessGateway()  // Fetches blocks via HTTP
  ],
  routers: [
    libp2pRouting(libp2p),      // P2P routing (primary)
    httpGatewayRouting()         // HTTP gateway routing (fallback)
  ]
})
```

**Key characteristics:**
- ✅ Full libp2p node with P2P networking
- ✅ Can publish content
- ✅ Can pin content  
- ✅ Can participate in DHT
- ✅ Trustless Gateway support via block-brokers
- ❌ Heavier weight (full IPFS node)

### Rust helia-http (Pure HTTP-Only)

The Rust version is a **true HTTP-only client** without any P2P capabilities:

```rust
// Rust creates a lightweight HTTP-only client
let helia = create_helia_http().await?;
// No libp2p, no P2P networking, only HTTP gateway access
```

**Key characteristics:**
- ✅ Pure HTTP gateway client
- ✅ Trustless Gateway specification
- ✅ Lightweight (no P2P overhead)
- ✅ Perfect for serverless/edge
- ❌ Cannot publish content
- ❌ Cannot pin content
- ❌ Read-only access

## Feature Comparison

| Feature | Rust helia-http | JS @helia/http |
|---------|----------------|----------------|
| **libp2p** | ❌ No | ✅ Yes |
| **P2P networking** | ❌ No | ✅ Yes |
| **DHT participation** | ❌ No | ✅ Yes |
| **Trustless Gateways** | ✅ Yes | ✅ Yes |
| **Gateway fallback** | ✅ Yes | ✅ Yes |
| **Content publishing** | ❌ No | ✅ Yes |
| **Content pinning** | ❌ No | ✅ Yes |
| **Block validation** | ✅ Basic | ✅ Advanced (validateFn) |
| **Session management** | ❌ No | ✅ Yes |
| **Reliability tracking** | ❌ No | ✅ Yes (per gateway) |
| **Custom headers** | ❌ No | ✅ Yes (transformRequestInit) |
| **Weight** | 🟢 Light | 🟡 Heavy |
| **Startup time** | 🟢 Instant | 🟡 5-30 seconds |
| **Use case** | Read-only, edge | Full IPFS node |

## Implementation Details

### Gateway URLs

Both use the **Trustless Gateway specification**:

```
https://gateway.example.com/ipfs/{cid}?format=raw
Accept: application/vnd.ipld.raw
```

**Default gateways (both):**
- `https://trustless-gateway.link`
- `https://4everland.io` 
- (Rust also includes: `https://cloudflare-ipfs.com`)

### Error Handling

**JavaScript:**
```typescript
// Tries all gateways, collects errors
throw new AggregateError(errors, 'Failed to fetch from any gateway')
```

**Rust:**
```rust
// Tries all gateways, returns Network error
Err(HeliaError::Network { 
    message: "Failed to fetch from all gateways" 
})
```

### Retry Logic

Both implement exponential backoff:
- Initial delay: 100ms
- Max retries: 2 (default)
- Exponential: 100ms → 200ms → 400ms

## Use Case Recommendations

### Use Rust helia-http when:
- ✅ Serverless functions (AWS Lambda, Cloudflare Workers)
- ✅ Edge computing environments
- ✅ Browser WASM (lightweight)
- ✅ CI/CD pipelines
- ✅ Read-only IPFS access
- ✅ Minimal dependencies required
- ✅ Fast startup needed

### Use JS @helia/http when:
- ✅ Need full IPFS capabilities
- ✅ Want to publish content
- ✅ Need pinning
- ✅ Want P2P + HTTP hybrid
- ✅ Need DHT participation
- ✅ Existing Node.js/browser environment

### Use Rust helia (full P2P) when:
- ✅ Building IPFS infrastructure
- ✅ Long-running services
- ✅ High performance requirements
- ✅ Need full P2P capabilities
- ✅ Native Rust application

## API Compatibility

### Creating a Helia Instance

**JavaScript:**
```typescript
import { createHeliaHTTP } from '@helia/http'

const helia = await createHeliaHTTP({
  blockBrokers: [trustlessGateway()],
  routers: [httpGatewayRouting()]
})
```

**Rust:**
```rust
use helia_http::{create_helia_http, GatewayConfig};

let helia = create_helia_http().await?;

// Or with custom config
let config = GatewayConfig {
    gateways: vec![
        "https://trustless-gateway.link".to_string(),
    ],
    timeout_secs: 30,
    max_retries: 2,
};
let helia = create_helia_http_with_gateways(config).await?;
```

### Fetching Blocks

Both use the same `Helia` trait/interface:

**JavaScript:**
```typescript
const cid = CID.parse('bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi')
const block = await helia.blockstore.get(cid)
```

**Rust:**
```rust
use cid::Cid;

let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
let block = helia.blockstore().get(&cid, None).await?;
```

### Integration with Other Modules

Both work seamlessly with higher-level modules:

**JavaScript:**
```typescript
import { unixfs } from '@helia/unixfs'

const fs = unixfs(helia)
const content = await fs.cat(cid)
```

**Rust:**
```rust
use helia_unixfs::unixfs;

let fs = unixfs(helia.clone());
let content = fs.cat(&cid, Default::default()).await?;
```

## Performance Characteristics

### Startup Time
- **Rust helia-http:** <10ms (instant)
- **JS @helia/http:** 5-30 seconds (libp2p startup)

### First Block Fetch
- **Rust helia-http:** 100-500ms (gateway lookup + HTTP)
- **JS @helia/http:** 100-500ms (gateway) or 1-10s (P2P)

### Memory Usage
- **Rust helia-http:** ~5-10 MB
- **JS @helia/http:** ~50-100 MB (full node)

### Throughput
- **Rust helia-http:** 10-50 MB/s (gateway limited)
- **JS @helia/http:** 50-200 MB/s (P2P + gateways)

## Future Enhancements

### Potential Rust Additions
1. **Block validation function** - Add `validateFn` like JS
2. **Session management** - Batch operations with session state
3. **Reliability tracking** - Track gateway performance
4. **Custom headers** - Add `transform_request` callback
5. **Metrics collection** - Gateway stats like JS

### Maintaining Compatibility
- Keep Trustless Gateway spec alignment
- Match default gateway URLs
- Similar API patterns
- Compatible error handling

## Conclusion

The Rust `helia-http` module is a **pure HTTP-only client** designed for lightweight, read-only IPFS access, while the JavaScript `@helia/http` module is a **full IPFS node** that happens to prefer HTTP gateways for block fetching.

**They serve different purposes:**
- **Rust helia-http** = Lightweight HTTP client for edge/serverless
- **JS @helia/http** = Full IPFS node with HTTP gateway support
- **Rust helia** = Full P2P IPFS node (equivalent to JS @helia/http P2P capabilities)

Both implementations follow the **Trustless Gateway specification** and can interoperate with the same gateways and content.

---

**References:**
- [Trustless Gateway Specification](https://specs.ipfs.tech/http-gateways/trustless-gateway/)
- [Helia JS Source](https://github.com/ipfs/helia)
- [Helia JS HTTP Module](https://github.com/ipfs/helia/tree/main/packages/http)
- [Helia JS Block Brokers](https://github.com/ipfs/helia/tree/main/packages/block-brokers)
