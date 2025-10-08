# TypeScript API Matching Guide

This document shows how the Rust implementation should match the TypeScript Helia API structure.

## Core Pattern: Factory Functions

**TypeScript Pattern:**
```typescript
import { createHelia } from 'helia'
import { unixfs } from '@helia/unixfs'
import { bitswap, trustlessGateway } from '@helia/block-brokers'
import { delegatedHTTPRouting, httpGatewayRouting, libp2pRouting } from '@helia/routers'

const helia = await createHelia({
  blockBrokers: [
    trustlessGateway(),
    bitswap()
  ],
  routers: [
    delegatedHTTPRouting('https://delegated-ipfs.dev'),
    httpGatewayRouting(),
    libp2pRouting(libp2p)
  ]
})

const fs = unixfs(helia)
await fs.cat(cid)
```

**Rust Equivalent (Target):**
```rust
use helia::create_helia;
use helia_unixfs::unixfs;
use helia_block_brokers::{bitswap, trustless_gateway};
use helia_routers::{delegated_http_routing, http_gateway_routing, libp2p_routing};

let helia = create_helia(HeliaInit {
    block_brokers: vec![
        trustless_gateway(TrustlessGatewayInit::default()),
        bitswap(BitswapInit::default()),
    ],
    routers: vec![
        delegated_http_routing("https://delegated-ipfs.dev"),
        http_gateway_routing(HttpGatewayRoutingInit::default()),
        libp2p_routing(libp2p),
    ],
    ..Default::default()
}).await?;

let fs = unixfs(helia);
fs.cat(&cid).await?;
```

## Package Structure Mapping

### 1. helia (Main Package)

**TypeScript:**
- `packages/helia/src/index.ts` - Exports `createHelia`, `libp2pDefaults`, `heliaDefaults`
- `packages/helia/src/utils/helia-defaults.ts` - Default configuration
- `packages/helia/src/utils/libp2p-defaults.ts` - libp2p configuration

**Rust:**
```
helia/
├── src/
│   ├── lib.rs                    // pub fn create_helia(), pub fn helia_defaults()
│   └── utils/
│       ├── mod.rs
│       ├── helia_defaults.rs     // Default Helia configuration
│       └── libp2p_defaults.rs    // Default libp2p configuration
└── Cargo.toml
```

**Key Exports:**
```rust
// helia/src/lib.rs
pub async fn create_helia<T: Libp2p>(init: HeliaInit<T>) -> Result<Helia<T>, HeliaError>;
pub async fn helia_defaults<T: Libp2p>(init: Partial<HeliaInit<T>>) -> HeliaInit<T>;
pub fn libp2p_defaults(options: Libp2pDefaultsOptions) -> Libp2pOptions;
```

### 2. @helia/block-brokers → helia-block-brokers

**TypeScript:**
```typescript
// packages/block-brokers/src/index.ts
export { bitswap } from './bitswap.js'
export { trustlessGateway } from './trustless-gateway/index.js'

// Usage
import { bitswap, trustlessGateway } from '@helia/block-brokers'
const helia = await createHelia({
  blockBrokers: [
    trustlessGateway({ gateways: ['https://ipfs.io'] }),
    bitswap()
  ]
})
```

**Rust:**
```
helia-block-brokers/
├── src/
│   ├── lib.rs                           // Exports
│   ├── bitswap.rs                       // pub fn bitswap()
│   └── trustless_gateway/
│       ├── mod.rs                       // pub fn trustless_gateway()
│       ├── gateway.rs                   // TrustlessGateway struct
│       ├── session.rs                   // Session management
│       └── reliability.rs               // Reliability tracking
└── Cargo.toml
```

**Key Exports:**
```rust
// helia-block-brokers/src/lib.rs
pub use bitswap::bitswap;
pub use trustless_gateway::trustless_gateway;

// helia-block-brokers/src/bitswap.rs
pub fn bitswap(init: BitswapInit) -> BlockBroker {
    Box::new(BitswapBlockBroker::new(init))
}

// helia-block-brokers/src/trustless_gateway/mod.rs
pub fn trustless_gateway(init: TrustlessGatewayInit) -> BlockBroker {
    Box::new(TrustlessGateway::new(init))
}
```

### 3. @helia/routers → helia-routers

**TypeScript:**
```typescript
// packages/routers/src/index.ts
export { delegatedHTTPRouting } from './delegated-http-routing.js'
export { httpGatewayRouting } from './http-gateway-routing.js'
export { libp2pRouting } from './libp2p-routing.js'

// Usage
import { delegatedHTTPRouting, httpGatewayRouting } from '@helia/routers'
const helia = await createHelia({
  routers: [
    delegatedHTTPRouting('https://delegated-ipfs.dev'),
    httpGatewayRouting({ gateways: ['https://ipfs.io'] })
  ]
})
```

**Rust:**
```
helia-routers/
├── src/
│   ├── lib.rs                              // Exports
│   ├── delegated_http_routing.rs           // pub fn delegated_http_routing()
│   ├── http_gateway_routing.rs             // pub fn http_gateway_routing()
│   ├── libp2p_routing.rs                   // pub fn libp2p_routing()
│   └── utils/
│       └── delegated_http_routing_defaults.rs
└── Cargo.toml
```

**Key Exports:**
```rust
// helia-routers/src/lib.rs
pub use delegated_http_routing::delegated_http_routing;
pub use http_gateway_routing::http_gateway_routing;
pub use libp2p_routing::libp2p_routing;

// helia-routers/src/delegated_http_routing.rs
pub fn delegated_http_routing(url: &str, init: DelegatedHTTPRoutingInit) -> Router {
    Box::new(DelegatedHTTPRouter::new(url, init))
}

// helia-routers/src/http_gateway_routing.rs
pub fn http_gateway_routing(init: HTTPGatewayRouterInit) -> Router {
    Box::new(HTTPGatewayRouter::new(init))
}

// helia-routers/src/libp2p_routing.rs
pub fn libp2p_routing(libp2p: Arc<dyn Libp2p>) -> Router {
    Box::new(Libp2pRouter::new(libp2p))
}
```

### 4. @helia/ipns → helia-ipns

**TypeScript:**
```typescript
// packages/ipns/src/index.ts
export function ipns(helia: Helia, options?: IPNSOptions): IPNS

// packages/ipns/src/routing/index.ts
export { helia } from './helia.js'
export { pubsub } from './pubsub.js'

// Usage
import { ipns } from '@helia/ipns'
import { helia as heliaRouting, pubsub } from '@helia/ipns/routing'

const name = ipns(helia, {
  routers: [
    heliaRouting(helia.routing),
    pubsub(helia)
  ]
})

await name.publish('key-1', cid)
const result = await name.resolve(publicKey)
```

**Rust:**
```
helia-ipns/
├── src/
│   ├── lib.rs                    // pub fn ipns()
│   ├── ipns.rs                   // IPNS struct
│   ├── record.rs                 // IPNSRecord
│   └── routing/
│       ├── mod.rs                // Exports
│       ├── helia.rs              // pub fn helia()
│       ├── pubsub.rs             // pub fn pubsub()
│       └── datastore.rs          // Datastore routing
└── Cargo.toml
```

**Key Exports:**
```rust
// helia-ipns/src/lib.rs
pub fn ipns(helia: Arc<dyn Helia>, options: IpnsOptions) -> Ipns {
    Ipns::new(helia, options)
}

// helia-ipns/src/routing/mod.rs
pub use helia::helia as helia_routing;
pub use pubsub::pubsub;

// helia-ipns/src/routing/helia.rs
pub fn helia_routing(routing: Arc<dyn Routing>) -> Box<dyn IpnsRouting> {
    Box::new(HeliaRouting::new(routing))
}

// helia-ipns/src/routing/pubsub.rs
pub fn pubsub<T: Libp2p>(helia: Arc<Helia<T>>) -> Box<dyn IpnsRouting> {
    Box::new(PubsubRouting::new(helia.libp2p.pubsub()))
}
```

### 5. @helia/dnslink → helia-dnslink

**TypeScript:**
```typescript
// packages/dnslink/src/index.ts
export function dnslink(helia: Helia, options?: DNSLinkOptions): DNSLink

// Usage
import { dnslink } from '@helia/dnslink'

const resolver = dnslink(helia)
const result = await resolver.resolve('example.com')
```

**Rust:**
```
helia-dnslink/
├── src/
│   ├── lib.rs                    // pub fn dnslink()
│   ├── dnslink.rs                // DnsLink struct
│   ├── resolver.rs               // DNS resolution
│   └── parser.rs                 // TXT record parsing
└── Cargo.toml
```

**Key Exports:**
```rust
// helia-dnslink/src/lib.rs
pub fn dnslink(helia: Arc<dyn Helia>, options: DnsLinkOptions) -> DnsLink {
    DnsLink::new(helia, options)
}
```

### 6. @helia/http → helia-http

**TypeScript:**
```typescript
// packages/http/src/index.ts
export async function createHeliaHTTP(init?: Partial<HeliaHTTPInit>): Promise<Helia>

// Usage
import { createHeliaHTTP } from '@helia/http'
import { trustlessGateway } from '@helia/block-brokers'

const helia = await createHeliaHTTP({
  blockBrokers: [trustlessGateway()],
  routers: [delegatedHTTPRouting('https://delegated-ipfs.dev')]
})
```

**Rust:**
```
helia-http/
├── src/
│   ├── lib.rs                    // pub async fn create_helia_http()
│   └── utils/
│       ├── mod.rs
│       ├── libp2p.rs             // Lightweight libp2p
│       └── libp2p_defaults.rs    // HTTP-specific defaults
└── Cargo.toml
```

**Key Exports:**
```rust
// helia-http/src/lib.rs
pub async fn create_helia_http(init: HeliaHttpInit) -> Result<Helia<Libp2p<DefaultLibp2pHTTPServices>>, HeliaError> {
    let options = helia_defaults(init).await?;
    let helia = HeliaClass::new(options);
    
    if options.start != Some(false) {
        helia.start().await?;
    }
    
    Ok(helia)
}
```

### 7. Data Format Packages

**TypeScript Pattern:**
```typescript
import { unixfs } from '@helia/unixfs'
import { dagCbor } from '@helia/dag-cbor'
import { dagJson } from '@helia/dag-json'
import { json } from '@helia/json'
import { strings } from '@helia/strings'

const fs = unixfs(helia)
const cbor = dagCbor(helia)
const djson = dagJson(helia)
const j = json(helia)
const str = strings(helia)
```

**Rust Pattern:**
```rust
use helia_unixfs::unixfs;
use helia_dag_cbor::dag_cbor;
use helia_dag_json::dag_json;
use helia_json::json;
use helia_strings::strings;

let fs = unixfs(helia.clone());
let cbor = dag_cbor(helia.clone());
let djson = dag_json(helia.clone());
let j = json(helia.clone());
let str = strings(helia.clone());
```

## Key Principles

1. **Factory Functions**: Every package exports factory functions that take `helia` and return instances
2. **Options Pattern**: Factory functions accept options structs (can be Default)
3. **Arc<dyn Trait>**: Use trait objects with Arc for sharing Helia instance
4. **Async/Await**: Match async patterns from TypeScript
5. **Builder Pattern**: For complex initialization (like trustlessGateway with many options)

## Implementation Priority

1. ✅ **helia-interface** - Already complete
2. ✅ **helia-utils** - Already complete
3. 🔄 **helia** (main) - Needs restructuring to factory pattern
4. ⚠️ **helia-block-brokers** - Needs complete implementation
5. ⚠️ **helia-routers** - Needs complete implementation
6. ⚠️ **helia-ipns** - Needs DHT/PubSub + factory pattern
7. ⚠️ **helia-dnslink** - Needs DNS resolution + factory pattern
8. ⚠️ **helia-http** - Needs restructuring + implementation
9. 🔍 **Data formats** - Need verification they match factory pattern

## Example: Complete Usage Pattern

**TypeScript:**
```typescript
import { createHelia } from 'helia'
import { unixfs } from '@helia/unixfs'
import { bitswap, trustlessGateway } from '@helia/block-brokers'
import { delegatedHTTPRouting, httpGatewayRouting } from '@helia/routers'
import { ipns } from '@helia/ipns'
import { dnslink } from '@helia/dnslink'

const helia = await createHelia({
  blockBrokers: [
    trustlessGateway({
      gateways: ['https://ipfs.io', 'https://dweb.link']
    }),
    bitswap()
  ],
  routers: [
    delegatedHTTPRouting('https://delegated-ipfs.dev'),
    httpGatewayRouting()
  ]
})

const fs = unixfs(helia)
const name = ipns(helia)
const dns = dnslink(helia)

// Use them
const file = await fs.cat(cid)
const resolved = await name.resolve(publicKey)
const domain = await dns.resolve('example.com')
```

**Rust (Target):**
```rust
use helia::{create_helia, HeliaInit};
use helia_unixfs::unixfs;
use helia_block_brokers::{bitswap, trustless_gateway, TrustlessGatewayInit};
use helia_routers::{delegated_http_routing, http_gateway_routing};
use helia_ipns::ipns;
use helia_dnslink::dnslink;

let helia = create_helia(HeliaInit {
    block_brokers: vec![
        trustless_gateway(TrustlessGatewayInit {
            gateways: vec![
                "https://ipfs.io".to_string(),
                "https://dweb.link".to_string(),
            ],
            ..Default::default()
        }),
        bitswap(Default::default()),
    ],
    routers: vec![
        delegated_http_routing("https://delegated-ipfs.dev", Default::default()),
        http_gateway_routing(Default::default()),
    ],
    ..Default::default()
}).await?;

let fs = unixfs(helia.clone());
let name = ipns(helia.clone(), Default::default());
let dns = dnslink(helia.clone(), Default::default());

// Use them
let file = fs.cat(&cid).await?;
let resolved = name.resolve(&public_key).await?;
let domain = dns.resolve("example.com").await?;
```

This structure ensures:
- ✅ Same API surface as TypeScript
- ✅ Same import/usage patterns
- ✅ Same factory function approach
- ✅ Same composability
- ✅ Drop-in replacement for TypeScript users
