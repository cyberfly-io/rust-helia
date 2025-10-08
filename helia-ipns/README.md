# helia-ipns

> IPNS (InterPlanetary Name System) implementation for Helia in Rust

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

IPNS is the InterPlanetary Name System - a way to create mutable pointers to content-addressed data in IPFS. Think of it as DNS for IPFS, allowing you to create human-friendly names that point to CIDs, and update those pointers over time without changing the name.

## Features

- 🔐 **Key Management**: Built-in keychain with Ed25519, RSA, and secp256k1 support
- 📝 **Publish & Resolve**: Create and resolve IPNS records with full validation
- 🔄 **Auto-Republish**: Background task automatically refreshes expiring records
- 💾 **Local Caching**: Fast resolution with TTL-based caching
- 🌐 **Offline Mode**: Work without network connectivity using local storage
- 🔌 **Pluggable Routing**: Support for multiple routers (DHT, HTTP, custom)
- ⚡ **Concurrent Operations**: Parallel router queries and publishing
- 🛡️ **Type-Safe**: Comprehensive error handling and type safety

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
helia-ipns = "0.1"
cid = "0.11"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

### Basic Publish and Resolve

```rust
use helia_ipns::{ipns, IpnsInit, PublishOptions, ResolveOptions};
use cid::Cid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an IPNS instance
    let name = ipns(IpnsInit::default())?;
    
    // Parse a CID
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()?;
    
    // Publish the CID under a key name
    let mut pub_options = PublishOptions::default();
    pub_options.offline = true; // Use offline mode for this example
    
    let result = name.publish("my-key", &cid, pub_options).await?;
    println!("Published! Public key: {:?}", result.public_key);
    
    // Resolve the record back
    let mut res_options = ResolveOptions::default();
    res_options.offline = true;
    
    let resolved = name.resolve(&result.public_key, res_options).await?;
    println!("Resolved CID: {}", resolved.cid);
    println!("Path: {}", resolved.path);
    
    Ok(())
}
```

### With Network Routing

```rust
use helia_ipns::{ipns, IpnsInit, PublishOptions, ResolveOptions};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure with routers for network publishing
    let mut init = IpnsInit::default();
    // init.routers.push(Arc::new(MyDhtRouter::new()));
    // init.routers.push(Arc::new(MyHttpRouter::new("https://ipfs.io")));
    
    let name = ipns(init)?;
    
    let cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
    
    // Publish with network distribution
    let pub_options = PublishOptions {
        offline: false,        // Enable network publishing
        lifetime: None,        // Use default 48-hour lifetime
        ttl: None,             // Use default 5-minute TTL
    };
    
    let result = name.publish("my-key", &cid, pub_options).await?;
    
    // Resolve with network queries
    let res_options = ResolveOptions {
        offline: false,        // Query network if needed
        nocache: false,        // Use cache when available
    };
    
    let resolved = name.resolve(&result.public_key, res_options).await?;
    println!("Resolved: {}", resolved.cid);
    
    Ok(())
}
```

### Auto-Republishing

```rust
use helia_ipns::{ipns, IpnsInit};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut init = IpnsInit::default();
    init.enable_republish = true;                           // Enable auto-republish
    init.republish_interval = Some(Duration::from_secs(3600)); // Check every hour
    init.republish_concurrency = Some(5);                   // Republish 5 records at a time
    
    let name = ipns(init)?;
    
    // Start the service (starts republish task)
    name.start().await?;
    
    // ... publish records ...
    
    // Records will be automatically republished before they expire
    
    // Stop the service when done
    name.stop().await?;
    
    Ok(())
}
```

### Updating Records

```rust
// Publish initial version
let cid1 = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
let result1 = name.publish("my-key", &cid1, pub_options.clone()).await?;
println!("Sequence: {}", result1.record.sequence); // 1

// Update with new CID - sequence auto-increments
let cid2 = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq".parse()?;
let result2 = name.publish("my-key", &cid2, pub_options).await?;
println!("Sequence: {}", result2.record.sequence); // 2
```

### Key Management

```rust
use libp2p_identity::{Keypair, ed25519};

// Keys are automatically created on first use
let result = name.publish("my-key", &cid, pub_options).await?;

// You can also work with peer IDs
use libp2p_identity::PeerId;
let peer_id = PeerId::random();
let resolved = name.resolve_peer_id(&peer_id, res_options).await?;

// Export a public key for sharing
// (In a real implementation, you'd add an export method)
```

### Error Handling

```rust
use helia_ipns::IpnsError;

match name.resolve(key, options).await {
    Ok(result) => println!("Resolved: {}", result.cid),
    Err(IpnsError::NotFound(msg)) => println!("Record not found: {}", msg),
    Err(IpnsError::RecordExpired(msg)) => println!("Record expired: {}", msg),
    Err(IpnsError::InvalidKey(msg)) => println!("Invalid key: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Configuration Options

### IpnsInit

```rust
pub struct IpnsInit {
    /// Routers for publishing and resolving records
    pub routers: Vec<Arc<dyn IpnsRouting>>,
    
    /// How often to check for records needing republishing
    /// Default: 1 hour
    pub republish_interval: Option<Duration>,
    
    /// How many records to republish concurrently
    /// Default: 5
    pub republish_concurrency: Option<usize>,
    
    /// Enable automatic republishing of records
    /// Default: true
    pub enable_republish: bool,
}
```

### PublishOptions

```rust
pub struct PublishOptions {
    /// Work offline (don't publish to routers)
    /// Default: false
    pub offline: bool,
    
    /// Record lifetime in milliseconds
    /// Default: 48 hours (172,800,000 ms)
    pub lifetime: Option<u64>,
    
    /// Time-to-live in nanoseconds
    /// Default: 5 minutes (300,000,000,000 ns)
    pub ttl: Option<u64>,
}
```

### ResolveOptions

```rust
pub struct ResolveOptions {
    /// Work offline (only check local cache)
    /// Default: false
    pub offline: bool,
    
    /// Skip cache and always query network
    /// Default: false
    pub nocache: bool,
}
```

## How IPNS Works

1. **Publishing**: 
   - Create or load a keypair by name
   - Increment sequence number if updating
   - Sign the CID with the private key
   - Store locally with metadata
   - Distribute to configured routers (DHT, HTTP, etc.)

2. **Resolving**:
   - Check local cache first (unless nocache=true)
   - Validate TTL and expiry
   - Query routers if cache miss or expired
   - Select best record if multiple found
   - Parse CID and path from record value

3. **Republishing**:
   - Background task runs periodically
   - Checks which records are approaching expiry
   - Increments sequence number
   - Updates validity timestamp
   - Re-signs and republishes to routers

## Record Format

IPNS records contain:
- **Value**: The content being pointed to (e.g., `/ipfs/Qm...`)
- **Sequence**: Monotonically increasing number for updates
- **Validity**: RFC3339 timestamp when record expires
- **TTL**: Time-to-live in nanoseconds
- **Public Key**: Protobuf-encoded public key
- **Signatures**: V1 and V2 signatures for verification

## Architecture

```
┌─────────────────┐
│   Application   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   IPNS API      │
│  (publish,      │
│   resolve)      │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    v         v
┌────────┐  ┌──────────┐
│Keychain│  │LocalStore│
└────────┘  └──────────┘
    │
    v
┌─────────────────┐
│  Routers        │
│  - DHT          │
│  - HTTP         │
│  - Custom       │
└─────────────────┘
    │
    v
┌─────────────────┐
│  IPFS Network   │
└─────────────────┘
```

## Testing

Run all tests:

```bash
cargo test -p helia-ipns
```

Run with output:

```bash
cargo test -p helia-ipns -- --nocapture
```

## Performance Considerations

- **Local Cache**: First resolve is slow (network query), subsequent resolves are fast (cache hit)
- **TTL**: Shorter TTL = more network queries, longer TTL = stale data risk
- **Republish**: More frequent = higher network usage, less frequent = expiry risk
- **Concurrency**: Higher = faster bulk operations, but more resource usage

## Security

- Private keys are stored in-memory (no persistence by default)
- Records are signed with Ed25519 by default (fast and secure)
- Record validation prevents tampering
- Sequence numbers prevent replay attacks
- Expiry timestamps prevent indefinite validity

## Limitations

- Currently uses JSON for marshaling (protobuf planned)
- Router implementations are stubs (DHT and HTTP need implementation)
- No DNSLink integration yet (planned)
- No recursive resolution yet (planned)

## Roadmap

- [ ] Implement protobuf marshaling for records
- [ ] Implement DHT router
- [ ] Implement HTTP router
- [ ] Add DNSLink resolution
- [ ] Add recursive IPNS resolution
- [ ] Add key persistence
- [ ] Add record validation (signature verification)
- [ ] Performance optimizations

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Resources

- [IPNS Specification](https://specs.ipfs.tech/ipns/ipns-record/)
- [TypeScript Helia IPNS](https://github.com/ipfs/helia-ipns)
- [IPFS Documentation](https://docs.ipfs.tech/)
- [libp2p](https://libp2p.io/)
