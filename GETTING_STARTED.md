# Getting Started with Helia Rust

Welcome to Helia Rust! This guide will help you get started with using Helia for decentralized file storage and content addressing using IPFS.

## What is Helia?

Helia is a lean, modular, and modern implementation of IPFS (InterPlanetary File System) for building applications on the distributed web. Helia Rust brings this powerful technology to the Rust ecosystem with:

- 🦀 **Type Safety**: Leverage Rust's strong type system
- ⚡ **Performance**: Near-native speed with zero-cost abstractions
- 🔒 **Memory Safety**: Guaranteed memory safety without garbage collection
- 🌐 **Interoperability**: Compatible with other IPFS implementations

## Quick Start (5 Minutes)

### Prerequisites

- Rust 1.70 or higher
- Basic understanding of async Rust (using Tokio)

### Installation

Clone the repository:

```bash
git clone https://github.com/cyberfly-io/rust-helia.git
cd rust-helia
```

### Your First Helia Program

Create a new file `my_first_helia.rs`:

```rust
use helia::create_helia;
use bytes::Bytes;
use helia_interface::Blocks;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Helia node
    let helia = create_helia(None).await?;
    helia.start().await?;
    
    // Store some data
    let data = Bytes::from("Hello, decentralized world!");
    let cid = helia.blockstore().put(data, None).await?;
    
    println!("Stored content with CID: {}", cid);
    
    // Retrieve the data
    let retrieved = helia.blockstore().get(&cid, None).await?;
    let text = String::from_utf8(retrieved.to_vec())?;
    
    println!("Retrieved: {}", text);
    
    helia.stop().await?;
    Ok(())
}
```

Run it:

```bash
cargo run --bin my_first_helia
```

## Core Concepts

### Content Identifiers (CIDs)

CIDs are self-describing content addresses that uniquely identify content regardless of where it's stored.

```rust
use cid::Cid;
use std::str::FromStr;

// Parse a CID
let cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;

// Every piece of content has a unique CID based on its content
```

### Blockstore

The blockstore is where raw blocks of data are stored. It's the foundation of IPFS storage.

```rust
// Store a block
let cid = helia.blockstore().put(data, None).await?;

// Retrieve a block
let data = helia.blockstore().get(&cid, None).await?;

// Check if a block exists
let exists = helia.blockstore().has(&cid, None).await?;
```

### UnixFS

UnixFS provides file system operations, making it easy to work with files and directories.

```rust
use helia_unixfs::{UnixFS, UnixFSInterface};
use std::sync::Arc;

let fs = UnixFS::new(Arc::new(helia));

// Add a file
let cid = fs.add_bytes(Bytes::from("file content"), None).await?;

// Read a file
let content = fs.cat(&cid, None).await?;
```

### Pinning

Pinning protects content from garbage collection, ensuring it stays available.

```rust
use helia_interface::Pins;

// Pin content
helia.pins().add(&cid, None).await?;

// Check if pinned
let is_pinned = helia.pins().is_pinned(&cid, None).await?;
```

## Common Use Cases

### 1. Storing and Retrieving Text

```rust
use helia::create_helia;
use helia_interface::Blocks;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = create_helia(None).await?;
    helia.start().await?;
    
    // Store text
    let text = "Hello, IPFS!";
    let cid = helia.blockstore().put(Bytes::from(text), None).await?;
    println!("CID: {}", cid);
    
    // Retrieve text
    let data = helia.blockstore().get(&cid, None).await?;
    let retrieved_text = String::from_utf8(data.to_vec())?;
    println!("Retrieved: {}", retrieved_text);
    
    helia.stop().await?;
    Ok(())
}
```

### 2. Working with Files

```rust
use helia::create_helia;
use helia_unixfs::{UnixFS, UnixFSInterface};
use bytes::Bytes;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;
    
    let fs = UnixFS::new(helia.clone());
    
    // Add a file
    let content = Bytes::from("File content goes here");
    let file_cid = fs.add_bytes(content, None).await?;
    
    // Create a directory
    let dir_cid = fs.add_directory(None, None).await?;
    
    // Add file to directory
    let dir_cid = fs.cp(&file_cid, &dir_cid, "myfile.txt", None).await?;
    
    // List directory
    let entries = fs.ls(&dir_cid, None).await?;
    for entry in entries {
        println!("{}: {}", entry.name, entry.cid);
    }
    
    helia.stop().await?;
    Ok(())
}
```

### 3. Structured Data with DAG-CBOR

```rust
use helia::create_helia;
use helia_dag_cbor::{DagCbor, DagCborInterface};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    email: String,
    age: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;
    
    let dag = DagCbor::new(helia.clone());
    
    // Store structured data
    let user = User {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };
    
    let cid = dag.add(&user, None).await?;
    println!("User stored with CID: {}", cid);
    
    // Retrieve structured data
    let retrieved_user: User = dag.get(&cid, None).await?;
    println!("Retrieved user: {:?}", retrieved_user);
    
    helia.stop().await?;
    Ok(())
}
```

### 4. Importing and Exporting CAR Files

```rust
use helia::create_helia;
use helia_car::{import_car, export_car};
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = Arc::new(create_helia(None).await?);
    helia.start().await?;
    
    // Import from CAR file
    let roots = import_car(helia.clone(), Path::new("data.car"), None).await?;
    println!("Imported {} root blocks", roots.len());
    
    // Export to CAR file
    export_car(helia.clone(), &roots[0], Path::new("export.car"), None).await?;
    println!("Exported to export.car");
    
    helia.stop().await?;
    Ok(())
}
```

## Configuration

### Custom Storage Paths

```rust
use helia::create_helia;
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
    
    // Your code here
    
    helia.stop().await?;
    Ok(())
}
```

### Custom libp2p Identity

```rust
use helia::create_helia;
use helia_utils::{HeliaConfig, create_swarm_with_keypair};
use libp2p::identity::Keypair;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a keypair
    let keypair = Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();
    println!("Peer ID: {}", peer_id);
    
    // Create swarm with keypair
    let swarm = create_swarm_with_keypair(keypair).await?;
    
    let config = HeliaConfig {
        libp2p: Some(Arc::new(Mutex::new(swarm))),
        ..Default::default()
    };
    
    let helia = create_helia(Some(config)).await?;
    helia.start().await?;
    
    // Your code here
    
    helia.stop().await?;
    Ok(())
}
```

## Error Handling

```rust
use helia::create_helia;
use helia_interface::{HeliaError, HeliaErrorType, Blocks};
use bytes::Bytes;
use cid::Cid;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let helia = match create_helia(None).await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to create Helia: {}", e);
            return;
        }
    };
    
    helia.start().await.expect("Failed to start node");
    
    let cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")
        .expect("Invalid CID");
    
    match helia.blockstore().get(&cid, None).await {
        Ok(data) => {
            println!("Retrieved {} bytes", data.len());
        }
        Err(e) => {
            match e.error_type() {
                HeliaErrorType::NotFound => {
                    println!("Block not found");
                }
                HeliaErrorType::Network => {
                    eprintln!("Network error: {}", e);
                }
                _ => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
    
    helia.stop().await.expect("Failed to stop node");
}
```

## Best Practices

### 1. Use Arc for Sharing

When sharing a Helia instance across different components:

```rust
use std::sync::Arc;

let helia = Arc::new(create_helia(None).await?);
let fs1 = UnixFS::new(helia.clone());
let fs2 = UnixFS::new(helia.clone());
```

### 2. Always Start and Stop

```rust
let helia = create_helia(None).await?;
helia.start().await?;  // Always start

// Your code here

helia.stop().await?;   // Always stop when done
```

### 3. Pin Important Content

```rust
// Pin content you want to keep
helia.pins().add(&cid, None).await?;

// Unpin when no longer needed
helia.pins().rm(&cid, None).await?;
```

### 4. Use Batch Operations

```rust
// Instead of this:
for block in blocks {
    helia.blockstore().put(block, None).await?;
}

// Do this:
let cids = helia.blockstore().put_many(blocks, None).await?;
```

### 5. Handle Errors Specifically

```rust
match operation().await {
    Ok(result) => { /* handle success */ },
    Err(e) => match e.error_type() {
        HeliaErrorType::NotFound => { /* handle not found */ },
        HeliaErrorType::Network => { /* handle network error */ },
        _ => { /* handle other errors */ },
    }
}
```

## Next Steps

Now that you understand the basics, explore:

1. **[Usage Guide](USAGE.md)** - Detailed documentation with more examples
2. **[Examples Directory](examples/)** - Complete working examples
3. **[API Reference](API_REFERENCE.md)** - Comprehensive API documentation
4. **[IPFS Documentation](https://docs.ipfs.io/)** - Learn more about IPFS concepts

## Getting Help

- 📖 Read the [documentation](https://github.com/cyberfly-io/rust-helia)
- 💬 Join [discussions](https://github.com/cyberfly-io/rust-helia/discussions)
- 🐛 Report [issues](https://github.com/cyberfly-io/rust-helia/issues)
- 🌐 Visit [IPFS community](https://ipfs.io/community)

## Example Projects

Check out these example projects to see Helia Rust in action:

- **File Sharing App**: Build a decentralized file sharing application
- **Content Archive**: Create a permanent web archive
- **Data Sync**: Synchronize data across distributed systems
- **NFT Storage**: Store NFT metadata on IPFS

## Common Patterns

### Progress Tracking

```rust
use helia_interface::{ProgressOptions, ProgressEvent};

let options = ProgressOptions {
    on_progress: Some(Box::new(|event: ProgressEvent<_>| {
        println!("Progress: {:?}", event.event_type);
    })),
};

let cid = helia.blockstore().put(data, Some(options)).await?;
```

### Working with Streams

```rust
use futures::StreamExt;

let mut stream = helia.pins().ls(None).await?;
while let Some(cid) = stream.next().await {
    println!("Pinned: {}", cid);
}
```

## Troubleshooting

### "Failed to create node"

- Check that storage paths are writable
- Ensure no other Helia instance is using the same storage

### "Block not found"

- Verify the CID is correct
- Check if the content is pinned
- Ensure the node has started

### "Network error"

- Check your internet connection
- Verify firewall settings allow libp2p connections

## Performance Tips

1. **Use batch operations** when storing multiple blocks
2. **Configure appropriate chunk sizes** for large files
3. **Pin frequently accessed content** to avoid re-fetching
4. **Use CAR files** for bulk import/export operations
5. **Configure custom storage paths** on fast drives

## Security Considerations

1. **Verify CIDs** before trusting content
2. **Validate data** after retrieval
3. **Use pinning** to ensure content availability
4. **Be aware** that public IPFS content is visible to all peers
5. **Consider encryption** for sensitive data before storing

---

Ready to build decentralized applications with Helia Rust? Start exploring the [examples](examples/) and [API documentation](API_REFERENCE.md)!

For questions or contributions, visit our [GitHub repository](https://github.com/cyberfly-io/rust-helia).
