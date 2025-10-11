# Getting Started with Rust Helia

A step-by-step tutorial to get you up and running with Rust Helia in minutes!

## What is Helia?

Helia is a lean, modular, and modern implementation of IPFS (InterPlanetary File System) for building applications on the distributed web. Rust Helia brings this powerful technology to the Rust ecosystem with:

- 🦀 **Type Safety**: Leverage Rust's strong type system
- ⚡ **Performance**: Near-native speed with zero-cost abstractions
- 🔒 **Memory Safety**: Guaranteed memory safety without garbage collection
- 🌐 **Interoperability**: Compatible with other IPFS implementations

## Prerequisites

Before you begin, ensure you have:

- **Rust 1.70 or later** - Install from [rustup.rs](https://rustup.rs)
- Basic knowledge of Rust and async programming
- A code editor (VS Code, IntelliJ IDEA, etc.)

## Quick Start (5 Minutes)

### Step 1: Create a New Project

```bash
cargo new my-ipfs-app
cd my-ipfs-app
```

### Step 2: Add Dependencies

Edit `Cargo.toml` and add:

```toml
[dependencies]
rust-helia = "0.1.3"
helia-unixfs = "0.1.3"
tokio = { version = "1.35", features = ["full"] }
bytes = "1.5"
cid = "0.11"
```

### Step 3: Your First Helia Program

Create `src/main.rs`:

```rust
use rust_helia::create_helia;
use bytes::Bytes;
use helia_interface::Blocks;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Helia node
    let helia = create_helia(None).await?;
    helia.start().await?;
    
    // Store some data
    let data = Bytes::from("Hello, decentralized world!");
    let cid = helia.blockstore().put(data.clone(), None).await?;
    
    println!("Stored content with CID: {}", cid);
    
    // Retrieve the data
    let retrieved = helia.blockstore().get(&cid, None).await?;
    let text = String::from_utf8(retrieved.to_vec())?;
    
    println!("Retrieved: {}", text);
    
    Ok(())
}
```

### Step 4: Run Your Program

```bash
cargo run
```

🎉 **Congratulations!** You just stored and retrieved your first piece of content on IPFS!

---

## Tutorial 1: Working with Files

Let's learn how to store and retrieve files using UnixFS.

### Storing a File

```rust
use rust_helia::create_helia;
use helia_unixfs::UnixFS;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    let unixfs = UnixFS::new(helia);
    
    // Read and store file
    let content = fs::read("example.txt").await?;
    let cid = unixfs.add_bytes(&content).await?;
    
    println!("File stored with CID: {}", cid);
    Ok(())
}
```

### Retrieving a File

```rust
use rust_helia::create_helia;
use helia_unixfs::UnixFS;
use cid::Cid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    let unixfs = UnixFS::new(helia);
    
    let cid: Cid = "bafkreiexample...".parse()?;
    let content = unixfs.cat(&cid).await?;
    
    tokio::fs::write("retrieved.txt", content).await?;
    Ok(())
}
```

---

## Tutorial 2: HTTP Gateway Integration

Use Helia in HTTP-only mode to fetch content from IPFS gateways.

```rust
use helia_http::create_helia_http;
use helia_unixfs::UnixFS;
use cid::Cid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HTTP-only client (no P2P)
    let helia = create_helia_http().await?;
    let unixfs = UnixFS::new(helia);
    
    // Fetch from public gateways
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".parse()?;
    let content = unixfs.cat(&cid).await?;
    
    println!("Fetched {} bytes from gateways", content.len());
    Ok(())
}
```

---

## Tutorial 3: Working with JSON Data

```rust
use rust_helia::create_helia;
use helia_json::JSON;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    let json_api = JSON::new(helia);
    
    // Store JSON
    let data = json!({
        "name": "Alice",
        "age": 30
    });
    let cid = json_api.add(&data).await?;
    
    // Retrieve JSON
    let retrieved = json_api.get(&cid).await?;
    println!("Retrieved: {}", retrieved);
    
    Ok(())
}
```

---

## Tutorial 4: CAR File Import/Export

```rust
use rust_helia::create_helia;
use helia_car::{CarWriter, export_to_car};
use cid::Cid;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    let cid: Cid = "bafybeic...".parse()?;
    
    // Export to CAR file
    let file = File::create("export.car").await?;
    let mut writer = CarWriter::new(file);
    export_to_car(&helia, &cid, &mut writer).await?;
    
    println!("Exported to export.car");
    Ok(())
}
```

---

## Next Steps

- Read the [API Reference](API_REFERENCE.md) for detailed documentation
- Check the [User Guide](USER_GUIDE.md) for best practices
- Explore the [Architecture Guide](ARCHITECTURE.md)
- Browse the `examples/` directory for more complete applications

**Happy building with Rust Helia!** 🚀
