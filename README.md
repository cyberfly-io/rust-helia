# Helia Rust Implementation

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A Rust implementation of [Helia](https://github.com/ipfs/helia), the lightweight, modular, and modern IPFS implementation. This project maintains API compatibility with the original TypeScript implementation while leveraging Rust's performance, safety, and concurrency features.

## ✨ Features

- 🦀 **Pure Rust**: Built from the ground up in Rust with zero runtime dependencies
- ⚡ **High Performance**: Leverages Rust's zero-cost abstractions and efficient memory management
- 🔒 **Memory Safe**: Guaranteed memory safety without garbage collection
- 🌐 **libp2p Integration**: Built on rust-libp2p for robust peer-to-peer networking
- 🔄 **Async/Await**: Fully asynchronous using Tokio runtime
- 📦 **Modular Design**: Each component is independently usable
- 🎯 **Type Safe**: Strong typing with comprehensive error handling
- 🔌 **API Compatible**: Familiar API for TypeScript Helia users
- 📊 **Serialization**: Full serde support for all data structures

## 🚀 Quick Start

Add Helia to your `Cargo.toml`:

```toml
[dependencies]
helia = { path = "./helia" }
helia-unixfs = { path = "./helia-unixfs" }
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use rust_helia::create_helia;
use helia_unixfs::UnixFS;
use bytes::Bytes;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Helia node
    let helia = create_helia(None).await?;
    
    // Create a UnixFS instance
    let fs = UnixFS::new(Arc::new(helia));
    
    // Add a file
    let content = Bytes::from("Hello, IPFS!");
    let cid = fs.add_bytes(content, None).await?;
    println!("Added file with CID: {}", cid);
    
    // Read the file back
    let data = fs.cat(&cid, None).await?;
    println!("Retrieved: {}", String::from_utf8(data.to_vec())?);
    
    Ok(())
}
```

## 📦 Status

🚧 **Work in Progress** - Core functionality is implemented and usable.

### Completed ✅
- **Core Infrastructure**: Workspace setup with all packages mirroring TypeScript structure
- **Interface Definitions**: Complete Rust trait definitions matching TypeScript interfaces
- **Core Types**: Error handling, progress tracking, CID support with serde serialization
- **Blockstore & Datastore**: Persistent storage using sled database
- **UnixFS Support**: File and directory operations
- **DAG Codecs**: DAG-CBOR, DAG-JSON, and JSON codec support
- **CAR Files**: Import and export CAR (Content Addressable aRchive) files
- **Testing Framework**: Comprehensive test coverage

### Package Structure

The implementation maintains the same modular package structure as the TypeScript version:

| Package | Description | Status |
|---------|-------------|--------|
| `helia` | Main entry point and node creation | ✅ Complete |
| `helia-interface` | Core traits and type definitions | ✅ Complete |
| `helia-utils` | Shared utilities and implementations | ✅ Complete |
| `helia-unixfs` | UnixFS file system operations | ✅ Complete |
| `helia-dag-cbor` | DAG-CBOR codec support | ✅ Complete |
| `helia-dag-json` | DAG-JSON codec support | ✅ Complete |
| `helia-json` | JSON utilities | ✅ Complete |
| `helia-car` | CAR file import/export | ✅ Complete |
| `helia-bitswap` | Bitswap protocol implementation | 🚧 In Progress |
| `helia-block-brokers` | Block broker implementations | 🚧 In Progress |
| `helia-dnslink` | DNSLink resolution | 🚧 In Progress |
| `helia-http` | HTTP transport | 🚧 In Progress |
| `helia-interop` | Interoperability utilities | 🚧 In Progress |
| `helia-ipns` | IPNS support | 🚧 In Progress |
| `helia-mfs` | Mutable File System | 🚧 In Progress |
| `helia-routers` | Content routing | 🚧 In Progress |
| `helia-strings` | String utilities | 🚧 In Progress |

## 📚 Documentation

- **[Usage Guide](USAGE.md)**: Comprehensive guide with examples for all major features
- **[API Reference](API_REFERENCE.md)**: Detailed API documentation
- **[Examples](examples/)**: Working code examples for common use cases

## 🔧 Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Clone and Build

```bash
git clone https://github.com/cyberfly-io/rust-helia.git
cd rust-helia
cargo build --release
```

### Run Tests

```bash
cargo test
```

## 💡 Examples

### Working with Blocks

```rust
use rust_helia::create_helia;
use helia_interface::Blocks;
use bytes::Bytes;
use cid::Cid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Store a block
    let data = Bytes::from("Hello, blocks!");
    let cid = helia.blockstore().put(data.clone(), None).await?;
    
    // Retrieve the block
    let retrieved = helia.blockstore().get(&cid, None).await?;
    assert_eq!(data, retrieved);
    
    // Check if block exists
    let exists = helia.blockstore().has(&cid, None).await?;
    println!("Block exists: {}", exists);
    
    Ok(())
}
```

### Working with DAG-CBOR

```rust
use rust_helia::create_helia;
use helia_dag_cbor::{DagCbor, DagCborInterface};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    let dag = DagCbor::new(Arc::new(helia));
    
    // Store structured data
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    
    let cid = dag.add(&person, None).await?;
    
    // Retrieve and decode
    let retrieved: Person = dag.get(&cid, None).await?;
    assert_eq!(person, retrieved);
    
    Ok(())
}
```

### Working with CAR Files

```rust
use rust_helia::create_helia;
use helia_car::{import_car, export_car};
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    
    // Import from CAR file
    let path = Path::new("example.car");
    let roots = import_car(helia.clone(), path, None).await?;
    println!("Imported {} root blocks", roots.len());
    
    // Export to CAR file
    let output = Path::new("exported.car");
    export_car(helia, &roots[0], output, None).await?;
    
    Ok(())
}
```

See the [examples/](examples/) directory for more detailed examples.

## 🏗️ Architecture

Helia Rust follows a modular architecture:

```
┌─────────────────────────────────────┐
│         Application Layer           │
│  (UnixFS, DAG Codecs, CAR, etc.)   │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│         Helia Core (helia)          │
│  - Node Management                  │
│  - Configuration                    │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Interface Layer (traits)       │
│  - Blocks, Pins, Routing            │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      Implementation (helia-utils)   │
│  - Blockstore (sled)                │
│  - Datastore (sled)                 │
│  - libp2p Integration               │
└─────────────────────────────────────┘
```

## 🛣️ Roadmap

### Phase 1: Foundation ✅
- [x] Basic workspace setup
- [x] Core trait definitions
- [x] Simple implementations
- [x] Compilation and basic tests

### Phase 2: Core Functionality ✅
- [x] Complete blockstore implementations
- [x] UnixFS support
- [x] DAG codec support
- [x] CAR file operations

### Phase 3: Network Layer 🚧
- [ ] Complete bitswap implementation
- [ ] DHT integration
- [ ] Content routing
- [ ] Block exchange protocols

### Phase 4: Advanced Features 📋
- [ ] IPNS support
- [ ] Mutable File System (MFS)
- [ ] HTTP gateway
- [ ] DNSLink resolution

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run clippy (`cargo clippy -- -D warnings`)
6. Format code (`cargo fmt`)
7. Commit your changes (`git commit -m 'Add some amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## 📄 License

This project is dual-licensed under MIT and Apache 2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## 🙏 Acknowledgments

- [Helia](https://github.com/ipfs/helia) - The original TypeScript implementation
- [IPFS](https://ipfs.io/) - The InterPlanetary File System
- [rust-libp2p](https://github.com/libp2p/rust-libp2p) - The libp2p networking stack
- [rust-cid](https://github.com/multiformats/rust-cid) - Content Identifier implementation

## 📞 Support

- 📖 [Documentation](https://docs.rs/helia)
- 💬 [Discussions](https://github.com/cyberfly-io/rust-helia/discussions)
- 🐛 [Issue Tracker](https://github.com/cyberfly-io/rust-helia/issues)

## 🔗 Related Projects

- [go-ipfs](https://github.com/ipfs/go-ipfs) - Go implementation of IPFS
- [js-ipfs](https://github.com/ipfs/js-ipfs) - JavaScript implementation of IPFS
- [helia](https://github.com/ipfs/helia) - Modern TypeScript IPFS implementation

---

Made with ❤️ by the Helia Rust community

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
