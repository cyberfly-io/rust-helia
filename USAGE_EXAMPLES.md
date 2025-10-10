# Rust Helia - Usage Examples

This document provides practical examples of using the newly implemented routing and block broker features.

---

## 🌐 Routing Examples

### HTTP Gateway Routing

HTTP gateway routing provides fallback content retrieval through public IPFS gateways.

#### Basic Usage with Default Gateways

```rust
use helia_routers::http_gateway_routing::{http_gateway_routing, HTTPGatewayRoutingInit};
use helia_interface::Routing;
use cid::Cid;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTTP gateway router with default gateways
    // Default: ipfs.io, dweb.link, cloudflare-ipfs.com
    let routing = http_gateway_routing(HTTPGatewayRoutingInit::default());

    // Find providers for a CID (returns gateways as synthetic providers)
    let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;
    let mut providers = routing.find_providers(&cid, None).await?;

    // Iterate through providers
    while let Some(provider) = providers.next().await {
        println!("Provider: {:?}", provider.peer_info.id);
        println!("Addresses: {:?}", provider.peer_info.multiaddrs);
        println!("Transport: {:?}", provider.transport_methods);
    }

    Ok(())
}
```

#### Custom Gateway Configuration

```rust
use helia_routers::http_gateway_routing::{http_gateway_routing, HTTPGatewayRoutingInit};
use url::Url;

// Configure custom gateways
let routing = http_gateway_routing(HTTPGatewayRoutingInit {
    gateways: vec![
        Url::parse("https://ipfs.io")?,
        Url::parse("https://dweb.link")?,
        Url::parse("https://my-custom-gateway.example")?,
    ],
});
```

#### Error Handling

```rust
use helia_interface::HeliaError;

// Note: These operations are NOT supported by HTTP gateway routing
// and will return errors

// Attempting to provide content (not supported)
match routing.provide(&cid, None).await {
    Err(HeliaError::OperationNotSupported(msg)) => {
        println!("Expected: {}", msg);
    }
    _ => panic!("Should return OperationNotSupported"),
}

// Attempting peer discovery (not supported)
let peer_id = libp2p::PeerId::random();
match routing.find_peers(&peer_id, None).await {
    Err(HeliaError::OperationNotSupported(msg)) => {
        println!("Expected: {}", msg);
    }
    _ => panic!("Should return OperationNotSupported"),
}

// Attempting DHT operations (not supported)
match routing.get(b"some-key", None).await {
    Err(HeliaError::OperationNotSupported(msg)) => {
        println!("Expected: {}", msg);
    }
    _ => panic!("Should return OperationNotSupported"),
}
```

---

### libp2p Routing (Skeleton)

libp2p routing provides DHT-based content and peer discovery.

#### Basic Setup

```rust
use helia_routers::libp2p_routing::libp2p_routing;
use helia_interface::Routing;
use libp2p::Swarm;
use std::sync::Arc;
use tokio::sync::Mutex;

// Create libp2p swarm with Kademlia DHT
// (This is a simplified example - actual setup would be more complex)
let swarm = create_libp2p_swarm().await?;
let swarm_arc = Arc::new(Mutex::new(swarm));

// Create routing instance
let routing = libp2p_routing(swarm_arc.clone());

// Use routing for DHT operations
let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;
let mut providers = routing.find_providers(&cid, None).await?;

// Note: Currently returns empty results until Kademlia integration is complete
```

#### Integration with Helia

```rust
use helia_routers::libp2p_routing::libp2p_routing;
use helia_utils::helia::{create_helia, HeliaInit};

// Create Helia instance with custom routing
let helia_init = HeliaInit {
    // ... other config
    ..Default::default()
};

let helia = create_helia(helia_init).await?;

// Get the swarm from Helia
let swarm_arc = helia.swarm(); // Assuming Helia exposes swarm

// Create routing
let routing = libp2p_routing(swarm_arc);
```

---

## 📦 Block Broker Examples

Block brokers provide different strategies for retrieving blocks.

### Bitswap Broker

```rust
use helia_block_brokers::{bitswap_broker, BitswapBroker};
use helia_bitswap::Bitswap;
use helia_interface::BlockBroker;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Bitswap instance
    let bitswap = Bitswap::new(blockstore, config).await?;
    
    // Create broker using factory function
    let broker = bitswap_broker(Arc::new(bitswap));

    // Retrieve a block
    let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;
    let block = broker.retrieve(&cid, Default::default()).await?;

    println!("Retrieved {} bytes", block.len());

    Ok(())
}
```

### Trustless Gateway Broker

```rust
use helia_block_brokers::{trustless_gateway, TrustlessGatewayInit};
use url::Url;

// Create trustless gateway broker with default settings
let broker = trustless_gateway(TrustlessGatewayInit::default());

// Or with custom configuration
let broker = trustless_gateway(TrustlessGatewayInit {
    gateways: vec![
        Url::parse("https://ipfs.io")?,
        Url::parse("https://dweb.link")?,
    ],
    max_retries: 5,
    timeout_ms: 60000,
    ..Default::default()
});

// Retrieve a block
let block = broker.retrieve(&cid, Default::default()).await?;
```

### Combining Multiple Brokers

```rust
use helia_block_brokers::{bitswap_broker, trustless_gateway};

// Create multiple brokers
let bitswap = bitswap_broker(Arc::new(bitswap_instance));
let gateway = trustless_gateway(TrustlessGatewayInit::default());

// Try Bitswap first, fallback to gateway
let block = match bitswap.retrieve(&cid, Default::default()).await {
    Ok(block) => {
        println!("Retrieved via Bitswap");
        block
    }
    Err(_) => {
        println!("Bitswap failed, trying gateway...");
        gateway.retrieve(&cid, Default::default()).await?
    }
};
```

---

## 🔄 Complete Integration Example

Here's a complete example showing how all pieces work together:

```rust
use helia_utils::helia::{create_helia, HeliaInit};
use helia_routers::http_gateway_routing::{http_gateway_routing, HTTPGatewayRoutingInit};
use helia_block_brokers::{bitswap_broker, trustless_gateway, TrustlessGatewayInit};
use helia_interface::{Helia, Routing, BlockBroker};
use cid::Cid;
use std::sync::Arc;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create Helia instance
    let helia = create_helia(HeliaInit::default()).await?;
    let helia_arc = Arc::new(helia);

    // 2. Set up routing strategies
    let http_routing = http_gateway_routing(HTTPGatewayRoutingInit::default());
    
    // 3. Set up block brokers
    let bitswap = helia_arc.bitswap(); // Assuming Helia exposes bitswap
    let bitswap_broker = bitswap_broker(bitswap);
    let gateway_broker = trustless_gateway(TrustlessGatewayInit::default());

    // 4. Find content
    let cid = Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;
    
    println!("Finding providers for CID: {}", cid);
    let mut providers = http_routing.find_providers(&cid, None).await?;
    
    let mut provider_count = 0;
    while let Some(provider) = providers.next().await {
        println!("  Provider: {:?}", provider.peer_info.id);
        provider_count += 1;
    }
    println!("Found {} providers", provider_count);

    // 5. Retrieve content
    println!("Retrieving block...");
    let block = match bitswap_broker.retrieve(&cid, Default::default()).await {
        Ok(block) => {
            println!("Retrieved via Bitswap: {} bytes", block.len());
            block
        }
        Err(e) => {
            println!("Bitswap failed ({}), trying gateway...", e);
            let block = gateway_broker.retrieve(&cid, Default::default()).await?;
            println!("Retrieved via Gateway: {} bytes", block.len());
            block
        }
    };

    // 6. Store in blockstore
    helia_arc.blockstore().put(&cid, &block).await?;
    println!("Stored block in blockstore");

    // 7. Pin the content
    helia_arc.pins().add(&cid).await?;
    println!("Pinned CID: {}", cid);

    Ok(())
}
```

---

## 🧪 Testing Examples

### Testing HTTP Gateway Routing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_gateway_find_providers() {
        let routing = http_gateway_routing(HTTPGatewayRoutingInit {
            gateways: vec![
                Url::parse("https://ipfs.io").unwrap(),
            ],
        });

        let cid = Cid::default();
        let mut providers = routing.find_providers(&cid, None).await.unwrap();
        
        let provider_vec: Vec<_> = providers.collect().await;
        assert!(!provider_vec.is_empty());
        
        let provider = &provider_vec[0];
        assert!(!provider.peer_info.multiaddrs.is_empty());
        assert_eq!(provider.transport_methods.len(), 1);
    }

    #[tokio::test]
    async fn test_http_gateway_unsupported_ops() {
        let routing = http_gateway_routing(HTTPGatewayRoutingInit::default());
        let cid = Cid::default();

        // These should all fail with OperationNotSupported
        assert!(routing.provide(&cid, None).await.is_err());
        assert!(routing.find_peers(&PeerId::random(), None).await.is_err());
        assert!(routing.get(b"key", None).await.is_err());
        assert!(routing.put(b"key", b"value", None).await.is_err());
    }
}
```

### Testing Block Brokers

```rust
#[tokio::test]
async fn test_broker_retrieval() {
    let broker = trustless_gateway(TrustlessGatewayInit {
        gateways: vec![
            Url::parse("https://ipfs.io").unwrap(),
        ],
        ..Default::default()
    });

    // Use a real CID for integration test
    let cid = Cid::try_from("bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku").unwrap();
    
    let block = broker.retrieve(&cid, Default::default()).await;
    assert!(block.is_ok());
}
```

---

## 📚 API Reference

### Routing Trait

```rust
pub trait Routing: Send + Sync {
    async fn find_providers(
        &self,
        cid: &Cid,
        options: Option<FindProvidersOptions>,
    ) -> Result<AwaitIterable<Provider>, HeliaError>;

    async fn provide(
        &self,
        cid: &Cid,
        options: Option<ProvideOptions>
    ) -> Result<(), HeliaError>;

    async fn find_peers(
        &self,
        peer_id: &PeerId,
        options: Option<FindPeersOptions>,
    ) -> Result<AwaitIterable<PeerInfo>, HeliaError>;

    async fn get(
        &self,
        key: &[u8],
        options: Option<GetOptions>,
    ) -> Result<Option<RoutingRecord>, HeliaError>;

    async fn put(
        &self,
        key: &[u8],
        value: &[u8],
        options: Option<PutOptions>,
    ) -> Result<(), HeliaError>;
}
```

### BlockBroker Trait

```rust
pub trait BlockBroker: Send + Sync {
    async fn retrieve(
        &self,
        cid: &Cid,
        options: RetrieveOptions,
    ) -> Result<Vec<u8>, HeliaError>;

    fn stats(&self) -> BrokerStats;
}
```

---

## 🎯 Next Steps

1. **Complete libp2p DHT Integration** - Enable actual provider queries
2. **Add IPNS Support** - Use routing for name resolution
3. **Implement DNSLink** - DNS-based content resolution
4. **Add More Tests** - Comprehensive integration tests
5. **Performance Tuning** - Optimize routing and retrieval

---

## 📖 Related Documentation

- [Implementation Progress Report](./IMPLEMENTATION_PROGRESS.md)
- [Comprehensive Gap Analysis](./COMPREHENSIVE_GAP_ANALYSIS.md)
- [Session Summary](./SESSION_SUMMARY.md)
