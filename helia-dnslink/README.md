# helia-dnslink

DNSLink resolution for Helia - enables domain names to point to IPFS content using DNS TXT records.

## Features

- ✅ DNS-over-HTTPS resolution using Google DNS
- ✅ Full DNSLink specification support
- ✅ Handles `_dnslink.` subdomain per spec
- ✅ Supports multiple namespaces: `/ipfs/`, `/ipns/`, `/dnslink/`
- ✅ Recursive DNSLink resolution (up to 32 levels)
- ✅ CNAME following for delegated domains
- ✅ Path extraction from DNSLink values
- ✅ Configurable DNS caching
- ✅ TypeScript Helia API-compatible

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
helia-dnslink = "0.1.2"
```

## Usage

### Basic Example

```rust
use helia_dnslink::{dns_link, DnsLinkInit, DnsLinkResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNSLink resolver
    let dnslink = dns_link(DnsLinkInit::default())?;
    
    // Resolve a domain with DNSLink record
    let result = dnslink.resolve("ipfs.tech").await?;
    
    // Handle the result
    match result {
        DnsLinkResult::IPFS { cid, path, .. } => {
            println!("Resolved to CID: {}", cid);
            if !path.is_empty() {
                println!("With path: {}", path);
            }
        }
        DnsLinkResult::IPNS { peer_id, path, .. } => {
            println!("Resolved to Peer ID: {}", peer_id);
        }
        DnsLinkResult::Other { namespace, value, .. } => {
            println!("Custom namespace: {} -> {}", namespace, value);
        }
    }
    
    Ok(())
}
```

### With Options

```rust
use helia_dnslink::{dns_link, DnsLinkInit, ResolveOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dnslink = dns_link(DnsLinkInit::default())?;
    
    // Custom resolution options
    let options = ResolveOptions {
        nocache: true,              // Don't use DNS cache
        offline: false,             // Allow network queries
        max_recursive_depth: Some(10), // Limit recursion depth
    };
    
    let result = dnslink.resolve_with_options("example.com", options).await?;
    Ok(())
}
```

### Custom Configuration

```rust
use helia_dnslink::{dns_link, DnsLinkInit};

let dnslink = dns_link(DnsLinkInit {
    use_https: true,        // Use DNS-over-HTTPS
    cache_enabled: true,    // Enable DNS caching
})?;
```

## DNSLink Format

DNSLink uses DNS TXT records to point domains to IPFS content:

```dns
_dnslink.example.com.  60  IN  TXT  "dnslink=/ipfs/bafybei..."
```

Supported formats:
- `dnslink=/ipfs/<cid>` - Points to IPFS content
- `dnslink=/ipfs/<cid>/path/to/file` - With path component
- `dnslink=/ipns/<peer-id>` - Points to IPNS name
- `dnslink=/dnslink/other.com` - Recursive DNSLink

## Result Types

```rust
pub enum DnsLinkResult {
    IPFS {
        answer: TxtRecord,    // DNS TXT record
        namespace: String,    // "ipfs"
        cid: Cid,            // Resolved CID
        path: String,        // Optional path
    },
    IPNS {
        answer: TxtRecord,
        namespace: String,    // "ipns"
        peer_id: PeerId,     // Resolved peer ID
        path: String,
    },
    Other {
        answer: TxtRecord,
        namespace: String,    // Custom namespace
        value: String,       // Raw value
    },
}
```

## Error Handling

```rust
pub enum DnsLinkError {
    NotFound(String),           // No DNSLink record found
    InvalidFormat(String),      // Invalid DNSLink format
    RecursionLimit(u32),        // Recursion limit exceeded
    InvalidCid(String),         // Invalid CID
    InvalidNamespace(String),   // Unknown namespace
    InvalidPeerId(String),      // Invalid peer ID
    DnsResolutionFailed(String), // DNS query failed
    InvalidDomain(String),      // Invalid domain name
    OfflineMode,                // Offline mode enabled
}
```

## How It Works

1. **Query DNS**: Queries `_dnslink.{domain}` TXT records
2. **Fallback**: Falls back to bare domain if `_dnslink.` fails
3. **Parse Record**: Extracts `dnslink=/namespace/value` format
4. **Handle Namespace**:
   - `/ipfs/<cid>` → Parse CID and return
   - `/ipns/<peer-id>` → Parse peer ID and return
   - `/dnslink/<domain>` → Recursively resolve
5. **CNAME**: Follows CNAME records if no TXT found
6. **Recursion**: Supports up to 32 levels of recursion

## Real-World Examples

Successfully resolves:
- `ipfs.tech` → `bafybeibb7bijpaz4kp5qrde45ui66lrzeqdb6kjabyorafmfzc6v6cls7q`
- `docs.ipfs.tech` → `bafybeihc3gzbj642jgt4dkgxebvnzoww53oahwvfbpxbmiiotajrpx6uja`

## Testing

Run tests:
```bash
cargo test -p helia-dnslink
```

Run network tests:
```bash
cargo test -p helia-dnslink -- --ignored --nocapture
```

## Performance

- DNS resolution: ~0.4-0.8 seconds (with DNS-over-HTTPS)
- Caching: Configurable DNS cache for repeated queries
- Async: Non-blocking async/await operations

## Compatibility

- ✅ **DNSLink Spec**: Fully compliant with DNSLink specification
- ✅ **TypeScript Helia**: API-compatible factory function pattern
- ✅ **IPFS Ecosystem**: Works with standard IPFS infrastructure
- ✅ **DNS-over-HTTPS**: Secure DNS resolution

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.

## Resources

- [DNSLink Specification](https://dnslink.dev/)
- [IPFS DNSLink Documentation](https://docs.ipfs.tech/concepts/dnslink/)
- [TypeScript @helia/dnslink](https://github.com/ipfs/helia/tree/main/packages/dnslink)
