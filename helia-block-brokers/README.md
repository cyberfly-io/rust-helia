# helia-block-brokers

Block broker abstractions for coordinating block retrieval from multiple sources in Helia.

## Overview

This crate provides the core traits and types for implementing block brokers - components that coordinate retrieving and announcing blocks across different protocols and sources.

## Features

- **BlockBroker Trait**: Core async trait for implementing different block retrieval strategies
- **Provider Types**: Support for Bitswap and Gateway providers
- **Statistics Tracking**: Built-in statistics for monitoring broker performance
- **Flexible Options**: Configurable timeout, priority, caching, and provider selection

## Usage

```rust
use helia_block_brokers::{BlockBroker, BrokerStats, BlockRetrievalOptions};
use bytes::Bytes;
use cid::Cid;

// Implement the BlockBroker trait for your custom broker
#[async_trait::async_trait]
impl BlockBroker for MyCustomBroker {
    async fn retrieve(&self, cid: Cid, options: BlockRetrievalOptions) -> Result<Bytes> {
        // Your implementation here
    }
    
    async fn announce(&self, cid: Cid, data: Bytes, options: BlockAnnounceOptions) -> Result<()> {
        // Your implementation here
    }
    
    async fn start(&self) -> Result<()> {
        // Start broker
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Stop broker
        Ok(())
    }
    
    fn get_stats(&self) -> BrokerStats {
        // Return statistics
        BrokerStats::default()
    }
    
    fn name(&self) -> &str {
        "my-custom-broker"
    }
}
```

## Architecture

The block broker abstraction allows for multiple implementation strategies:

- **Bitswap Brokers**: Retrieve blocks via the Bitswap protocol over libp2p
- **Gateway Brokers**: Retrieve blocks from HTTP trustless gateways
- **Composite Brokers**: Coordinate multiple brokers with fallback strategies
- **Custom Brokers**: Implement your own retrieval logic

## Status

This is a foundational package providing core abstractions. Concrete implementations will be added in future iterations.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
