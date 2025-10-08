# Helia Rust Implementation

This is a Rust implementation of [Helia](https://github.com/ipfs/helia), maintaining the same API and folder structure as the original TypeScript implementation.

## Status

🚧 **Work in Progress** - This is an initial migration with basic infrastructure in place.

### Completed ✅
- **Core Infrastructure**: Workspace setup with all packages mirroring TypeScript structure
- **Interface Definitions**: Complete Rust trait definitions matching TypeScript interfaces
- **Core Types**: Error handling, progress tracking, CID support with serde serialization
- **Basic Implementation**: Simple blockstore, datastore, and Helia node implementation
- **Testing Framework**: Basic test structure with passing tests

### Package Structure

The implementation maintains the same package structure as the TypeScript version:

```
rust/
├── helia/                 # Main implementation (equivalent to packages/helia)
├── helia-interface/       # API traits and types (equivalent to packages/interface)
├── helia-utils/           # Shared utilities (equivalent to packages/utils)
├── helia-bitswap/         # Bitswap protocol implementation
├── helia-block-brokers/   # Block broker implementations
├── helia-car/             # CAR file format support
├── helia-dag-cbor/        # DAG-CBOR codec
├── helia-dag-json/        # DAG-JSON codec
├── helia-dnslink/         # DNSLink resolution
├── helia-http/            # HTTP transport
├── helia-interop/         # Interoperability utilities
├── helia-ipns/            # IPNS support
├── helia-json/            # JSON utilities
├── helia-mfs/             # Mutable File System
├── helia-routers/         # Content routing
├── helia-strings/         # String utilities
└── helia-unixfs/          # UnixFS implementation
```

### Key Features

- **Async/Await**: Full async support using Tokio
- **Type Safety**: Strong typing with Rust's type system
- **Memory Safety**: Zero-cost abstractions with memory safety guarantees
- **API Compatibility**: Same function signatures and behavior as TypeScript version
- **Serde Support**: JSON serialization/deserialization for all data types
- **libp2p Integration**: Built on rust-libp2p for networking
- **Modular Design**: Each package is independently compilable

## Usage

```rust
use helia::create_helia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Helia node with default configuration
    let helia = create_helia(None).await?;
    
    // Start the node
    helia.start().await?;
    
    // Use the node...
    
    // Stop the node
    helia.stop().await?;
    
    Ok(())
}
```

## Building

```bash
cd rust
cargo build
```

## Testing

```bash
cd rust
cargo test
```

## Roadmap

### Phase 1: Foundation (Completed ✅)
- [x] Basic workspace setup
- [x] Core trait definitions
- [x] Simple implementations
- [x] Compilation and basic tests

### Phase 2: Core Functionality
- [ ] Complete blockstore implementations
- [ ] Network layer integration
- [ ] Content routing
- [ ] Block exchange protocols

### Phase 3: Data Formats
- [ ] UnixFS implementation
- [ ] CAR file support
- [ ] DAG-CBOR/JSON codecs
- [ ] IPNS resolution

### Phase 4: Advanced Features
- [ ] MFS (Mutable File System)
- [ ] Advanced routing strategies
- [ ] Performance optimizations
- [ ] Full API compatibility

## Contributing

This is an initial migration effort. The foundational work is complete, and the project is ready for incremental development of individual packages.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Relationship to TypeScript Implementation

This Rust implementation aims to be:
- **API Compatible**: Same public interfaces and behavior
- **Interoperable**: Can work with existing IPFS networks and TypeScript nodes
- **Performant**: Leveraging Rust's zero-cost abstractions
- **Safe**: Memory safety without garbage collection overhead# rust-helia
