# Helia Rust Usage Guide

This guide provides comprehensive examples and documentation for using Helia Rust in your projects.

## Table of Contents

- [Getting Started](#getting-started)
- [Core Concepts](#core-concepts)
- [Creating a Helia Node](#creating-a-helia-node)
- [Working with Blocks](#working-with-blocks)
- [UnixFS Operations](#unixfs-operations)
- [DAG Codecs](#dag-codecs)
- [CAR Files](#car-files)
- [Pinning](#pinning)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Advanced Usage](#advanced-usage)

## Getting Started

### Adding Dependencies

Add Helia to your `Cargo.toml`:

```toml
[dependencies]
helia = { path = "./helia" }
helia-interface = { path = "./helia-interface" }
helia-unixfs = { path = "./helia-unixfs" }
helia-dag-cbor = { path = "./helia-dag-cbor" }
helia-dag-json = { path = "./helia-dag-json" }
helia-car = { path = "./helia-car" }

# Core dependencies
tokio = { version = "1", features = ["full"] }
bytes = "1.5"
cid = "0.11"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Setup

```rust
use rust_helia::create_helia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    println!("Helia node created successfully!");
    Ok(())
}
```

## Core Concepts

### Helia Trait

The `Helia` trait is the main interface for interacting with a Helia node:

```rust
pub trait Helia {
    fn blockstore(&self) -> &dyn Blocks;
    fn datastore(&self) -> &dyn Datastore;
    fn pins(&self) -> &dyn Pins;
    fn logger(&self) -> &dyn ComponentLogger;
    fn routing(&self) -> &dyn Routing;
    fn dns(&self) -> &TokioAsyncResolver;
    fn metrics(&self) -> Option<&dyn Metrics>;
    
    async fn start(&self) -> Result<(), HeliaError>;
    async fn stop(&self) -> Result<(), HeliaError>;
}
```

### Content Identifiers (CIDs)

CIDs are self-describing content-addressed identifiers used throughout IPFS:

```rust
use cid::Cid;
use std::str::FromStr;

// Parse a CID from string
let cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;

// Get CID properties
println!("Version: {:?}", cid.version());
println!("Codec: {:?}", cid.codec());
```

## Creating a Helia Node

### Default Configuration

```rust
use rust_helia::create_helia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create with default configuration
    let helia = create_helia(None).await?;
    
    // Start the node
    helia.start().await?;
    
    println!("Node is running!");
    
    // Stop when done
    helia.stop().await?;
    
    Ok(())
}
```

### Custom Configuration

```rust
use rust_helia::create_helia;
use helia_utils::{HeliaConfig, BlockstoreConfig, DatastoreConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = HeliaConfig {
        blockstore: BlockstoreConfig {
            path: Some(PathBuf::from("./my-blocks")),
            ..Default::default()
        },
        datastore: DatastoreConfig {
            path: Some(PathBuf::from("./my-data")),
            ..Default::default()
        },
        ..Default::default()
    };
    
    let helia = create_helia(Some(config)).await?;
    helia.start().await?;
    
    Ok(())
}
```

## Working with Blocks

The blockstore is the low-level storage layer for raw blocks of data.

### Storing Blocks

```rust
use rust_helia::create_helia;
use helia_interface::Blocks;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Create some data
    let data = Bytes::from("Hello, IPFS!");
    
    // Store the block
    let cid = helia.blockstore().put(data, None).await?;
    println!("Stored block with CID: {}", cid);
    
    Ok(())
}
```

### Retrieving Blocks

```rust
use rust_helia::create_helia;
use helia_interface::Blocks;
use cid::Cid;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    let cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;
    
    // Retrieve the block
    let data = helia.blockstore().get(&cid, None).await?;
    println!("Retrieved {} bytes", data.len());
    
    Ok(())
}
```

### Checking Block Existence

```rust
use rust_helia::create_helia;
use helia_interface::Blocks;
use cid::Cid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Check if a block exists
    let exists = helia.blockstore().has(&cid, None).await?;
    
    if exists {
        println!("Block found!");
    } else {
        println!("Block not found");
    }
    
    Ok(())
}
```

### Deleting Blocks

```rust
use rust_helia::create_helia;
use helia_interface::Blocks;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Delete a block
    helia.blockstore().delete(&cid, None).await?;
    println!("Block deleted");
    
    Ok(())
}
```

### Batch Operations

```rust
use rust_helia::create_helia;
use helia_interface::{Blocks, InputPair};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Store multiple blocks at once
    let blocks = vec![
        InputPair {
            cid: None,
            block: Bytes::from("Block 1"),
        },
        InputPair {
            cid: None,
            block: Bytes::from("Block 2"),
        },
        InputPair {
            cid: None,
            block: Bytes::from("Block 3"),
        },
    ];
    
    let cids = helia.blockstore().put_many(blocks, None).await?;
    println!("Stored {} blocks", cids.len());
    
    Ok(())
}
```

## UnixFS Operations

UnixFS provides file and directory operations compatible with IPFS.

### Adding Files

```rust
use rust_helia::create_helia;
use helia_unixfs::{UnixFS, UnixFSInterface};
use bytes::Bytes;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let fs = UnixFS::new(helia);
    
    // Add a file from bytes
    let content = Bytes::from("Hello, UnixFS!");
    let cid = fs.add_bytes(content, None).await?;
    println!("File CID: {}", cid);
    
    Ok(())
}
```

### Reading Files

```rust
use helia_unixfs::{UnixFS, UnixFSInterface};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let fs = UnixFS::new(helia);
    
    // Read file content
    let data = fs.cat(&cid, None).await?;
    let text = String::from_utf8(data.to_vec())?;
    println!("Content: {}", text);
    
    Ok(())
}
```

### Working with Directories

```rust
use helia_unixfs::{UnixFS, UnixFSInterface};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let fs = UnixFS::new(helia);
    
    // Create an empty directory
    let dir_cid = fs.add_directory(None, None).await?;
    println!("Directory CID: {}", dir_cid);
    
    // Add a file
    let file_data = Bytes::from("File content");
    let file_cid = fs.add_bytes(file_data, None).await?;
    
    // Copy file into directory
    let updated_dir = fs.cp(&file_cid, &dir_cid, "myfile.txt", None).await?;
    println!("Updated directory CID: {}", updated_dir);
    
    // List directory contents
    let entries = fs.ls(&updated_dir, None).await?;
    for entry in entries {
        println!("- {} ({})", entry.name, entry.cid);
    }
    
    Ok(())
}
```

### File Statistics

```rust
use helia_unixfs::{UnixFS, UnixFSInterface};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let fs = UnixFS::new(helia);
    
    // Get file/directory stats
    let stats = fs.stat(&cid, None).await?;
    println!("Type: {:?}", stats.file_type);
    println!("Size: {} bytes", stats.file_size);
    println!("Blocks: {}", stats.blocks);
    
    Ok(())
}
```

### Chunking Large Files

```rust
use helia_unixfs::{UnixFS, UnixFSInterface, AddBytesOptions};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let fs = UnixFS::new(helia);
    
    // Large file content
    let large_content = Bytes::from(vec![0u8; 1024 * 1024]); // 1 MB
    
    // Configure chunking
    let options = AddBytesOptions {
        chunk_size: Some(256 * 1024), // 256 KB chunks
        ..Default::default()
    };
    
    let cid = fs.add_bytes(large_content, Some(options)).await?;
    println!("Large file added: {}", cid);
    
    Ok(())
}
```

## DAG Codecs

Work with structured data using DAG codecs.

### DAG-CBOR

DAG-CBOR is ideal for encoding structured data with links to other content.

```rust
use rust_helia::create_helia;
use helia_dag_cbor::{DagCbor, DagCborInterface};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct BlogPost {
    title: String,
    author: String,
    content: String,
    tags: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let dag = DagCbor::new(helia);
    
    // Create structured data
    let post = BlogPost {
        title: "Hello IPFS".to_string(),
        author: "Alice".to_string(),
        content: "This is my first post on IPFS!".to_string(),
        tags: vec!["ipfs".to_string(), "web3".to_string()],
    };
    
    // Store it
    let cid = dag.add(&post, None).await?;
    println!("Blog post CID: {}", cid);
    
    // Retrieve and decode
    let retrieved: BlogPost = dag.get(&cid, None).await?;
    println!("Retrieved: {}", retrieved.title);
    
    Ok(())
}
```

### DAG-JSON

DAG-JSON is similar to DAG-CBOR but uses JSON encoding.

```rust
use rust_helia::create_helia;
use helia_dag_json::{DagJson, DagJsonInterface};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    version: String,
    settings: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let dag = DagJson::new(helia);
    
    let config = Config {
        version: "1.0.0".to_string(),
        settings: HashMap::new(),
    };
    
    let cid = dag.add(&config, None).await?;
    let retrieved: Config = dag.get(&cid, None).await?;
    
    Ok(())
}
```

### Regular JSON

For simple JSON without IPLD links:

```rust
use rust_helia::create_helia;
use helia_json::{Json, JsonInterface};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let json_codec = Json::new(helia);
    
    let data = json!({
        "name": "Alice",
        "age": 30,
        "city": "Wonderland"
    });
    
    let cid = json_codec.add(&data, None).await?;
    let retrieved = json_codec.get(&cid, None).await?;
    
    println!("Retrieved: {}", retrieved);
    
    Ok(())
}
```

## CAR Files

CAR (Content Addressable aRchive) files package IPLD data for transport and storage.

### Exporting to CAR

```rust
use rust_helia::create_helia;
use helia_car::export_car;
use helia_unixfs::{UnixFS, UnixFSInterface};
use std::path::Path;
use std::sync::Arc;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    let fs = UnixFS::new(helia.clone());
    
    // Add some content
    let content = Bytes::from("Hello, CAR files!");
    let cid = fs.add_bytes(content, None).await?;
    
    // Export to CAR file
    let output_path = Path::new("export.car");
    export_car(helia, &cid, output_path, None).await?;
    
    println!("Exported to {}", output_path.display());
    
    Ok(())
}
```

### Importing from CAR

```rust
use rust_helia::create_helia;
use helia_car::import_car;
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    
    // Import CAR file
    let input_path = Path::new("import.car");
    let roots = import_car(helia.clone(), input_path, None).await?;
    
    println!("Imported {} root blocks:", roots.len());
    for root in &roots {
        println!("  {}", root);
    }
    
    Ok(())
}
```

## Pinning

Pinning protects content from garbage collection.

### Pin Content

```rust
use rust_helia::create_helia;
use helia_interface::Pins;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Pin a CID
    helia.pins().add(&cid, None).await?;
    println!("Content pinned");
    
    Ok(())
}
```

### Check if Pinned

```rust
use rust_helia::create_helia;
use helia_interface::Pins;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    let is_pinned = helia.pins().is_pinned(&cid, None).await?;
    println!("Is pinned: {}", is_pinned);
    
    Ok(())
}
```

### Unpin Content

```rust
use rust_helia::create_helia;
use helia_interface::Pins;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    helia.pins().rm(&cid, None).await?;
    println!("Content unpinned");
    
    Ok(())
}
```

### List All Pins

```rust
use rust_helia::create_helia;
use helia_interface::Pins;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    let mut pins = helia.pins().ls(None).await?;
    
    while let Some(cid) = pins.next().await {
        println!("Pinned: {}", cid);
    }
    
    Ok(())
}
```

## Configuration

### Blockstore Configuration

```rust
use helia_utils::BlockstoreConfig;
use std::path::PathBuf;

let blockstore_config = BlockstoreConfig {
    path: Some(PathBuf::from("./blocks")),
    // Additional configuration options
    ..Default::default()
};
```

### Datastore Configuration

```rust
use helia_utils::DatastoreConfig;
use std::path::PathBuf;

let datastore_config = DatastoreConfig {
    path: Some(PathBuf::from("./data")),
    ..Default::default()
};
```

### libp2p Configuration

```rust
use helia_utils::create_swarm_with_keypair;
use libp2p::identity::Keypair;

// Create with custom keypair
let keypair = Keypair::generate_ed25519();
let swarm = create_swarm_with_keypair(keypair).await?;
```

## Error Handling

Helia uses a comprehensive error type system:

```rust
use rust_helia::create_helia;
use helia_interface::{HeliaError, HeliaErrorType};

#[tokio::main]
async fn main() {
    match create_helia(None).await {
        Ok(helia) => {
            println!("Node created successfully");
        }
        Err(e) => {
            match e.error_type() {
                HeliaErrorType::NotFound => {
                    eprintln!("Content not found: {}", e);
                }
                HeliaErrorType::Network => {
                    eprintln!("Network error: {}", e);
                }
                HeliaErrorType::InvalidInput => {
                    eprintln!("Invalid input: {}", e);
                }
                _ => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}
```

## Advanced Usage

### Progress Tracking

```rust
use rust_helia::create_helia;
use helia_interface::{Blocks, ProgressOptions, ProgressEvent};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    let data = Bytes::from("Hello with progress!");
    
    let options = ProgressOptions {
        on_progress: Some(Box::new(|event: ProgressEvent<_>| {
            println!("Progress: {:?}", event.event_type);
        })),
    };
    
    let cid = helia.blockstore().put(data, Some(options)).await?;
    
    Ok(())
}
```

### Custom Codecs

```rust
use helia_interface::{CodecLoader, Codec};
use async_trait::async_trait;

struct MyCodecLoader;

#[async_trait]
impl CodecLoader for MyCodecLoader {
    async fn load(&self, code: u64) -> Result<Box<dyn Codec>, HeliaError> {
        // Load codec implementation
        todo!()
    }
}
```

### Working with Streams

```rust
use futures::StreamExt;
use helia_interface::Pins;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    
    // Stream all pins
    let mut pin_stream = helia.pins().ls(None).await?;
    
    while let Some(cid) = pin_stream.next().await {
        println!("Pin: {}", cid);
    }
    
    Ok(())
}
```

## Best Practices

1. **Use Arc for sharing**: Wrap Helia instances in `Arc` when sharing across threads or async tasks
2. **Handle errors gracefully**: Always match on specific error types for better error handling
3. **Pin important content**: Use pinning to prevent garbage collection of important data
4. **Batch operations**: Use `put_many` for better performance when storing multiple blocks
5. **Configure paths**: Specify custom paths for blockstore and datastore in production
6. **Close resources**: Call `stop()` when done with a Helia node

## Troubleshooting

### Node Won't Start

```rust
// Check if already started
if let Err(e) = helia.start().await {
    eprintln!("Failed to start: {}", e);
    // Node might already be running
}
```

### Block Not Found

```rust
// Verify block exists before retrieving
if helia.blockstore().has(&cid, None).await? {
    let data = helia.blockstore().get(&cid, None).await?;
} else {
    println!("Block not found locally");
}
```

### Performance Issues

- Use batch operations for multiple blocks
- Configure appropriate chunk sizes for large files
- Consider using CAR files for bulk imports/exports

## Next Steps

- Explore the [API Reference](API_REFERENCE.md) for detailed trait documentation
- Check out the [examples/](examples/) directory for complete working examples
- Read the [TypeScript Helia documentation](https://helia.io) for conceptual understanding
- Join the [IPFS community](https://ipfs.io/community) for support

---

For more information, visit the [Helia Rust repository](https://github.com/cyberfly-io/rust-helia).
