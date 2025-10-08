# helia-mfs

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust implementation of the Mutable File System (MFS) for IPFS/Helia.

## Overview

MFS (Mutable File System) provides a familiar filesystem-like interface over IPFS's immutable content-addressed storage. It maintains a mutable root pointer that allows you to modify your filesystem while still benefiting from IPFS's content addressing and deduplication.

Think of MFS as your "working directory" in IPFS - you can create, modify, and delete files and directories as you would with a traditional filesystem, while under the hood everything is still content-addressed and immutable.

## Features

- **Familiar API**: Unix-like filesystem operations (mkdir, ls, write, etc.)
- **Mutable Root**: Maintains a mutable root CID that updates as you modify the filesystem
- **Built on UnixFS**: Leverages the UnixFS implementation for underlying operations
- **Path-based**: Work with familiar `/path/to/file` style paths
- **Automatic Root Creation**: Creates root directory automatically when needed

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
helia-mfs = "0.1.0"
```

## Usage

### Basic Operations

```rust
use helia_mfs::{mfs, MfsInterface};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = helia::create_helia_default().await?;
    let fs = mfs(Arc::new(helia));

    // Create a directory
    fs.mkdir("/my-directory").await?;

    // Write a file
    let data = b"Hello, MFS!";
    fs.write_bytes(data, "/my-directory/hello.txt").await?;

    // List directory contents
    let entries = fs.ls(Some("/")).await?;
    for entry in entries {
        println!("Entry: {}", entry.name);
    }

    Ok(())
}
```

### Creating Directories

```rust
// Create a directory at root
fs.mkdir("/documents").await?;

// Create nested directories (parent must exist first)
fs.mkdir("/documents/2024").await?;
```

### Writing Files

```rust
// Write text data
let text = b"This is a test document";
fs.write_bytes(text, "/documents/test.txt").await?;

// Write binary data
let binary_data = vec![0x00, 0x01, 0x02, 0x03];
fs.write_bytes(&binary_data, "/documents/data.bin").await?;
```

### Listing Directory Contents

```rust
// List root directory
let entries = fs.ls(None).await?;
for entry in entries {
    println!("{} - Type: {:?}", entry.name, entry.type_);
}

// List specific directory
let entries = fs.ls(Some("/documents")).await?;
for entry in entries {
    println!("{} - CID: {}", entry.name, entry.cid);
}
```

## Architecture

MFS maintains a mutable root CID that gets updated whenever you make changes to the filesystem. Each operation:

1. Retrieves the current root CID
2. Performs the UnixFS operation
3. Updates the root CID with the new state

This approach provides:
- **Immutability**: All content is still content-addressed
- **History**: Old versions are still accessible via their CIDs
- **Efficiency**: Unchanged parts of the tree are reused

### Relationship with UnixFS

```
MFS (Mutable Layer)
    ↓
UnixFS (Filesystem Layer)
    ↓
Helia (IPFS Core)
    ↓
Blockstore (Storage Layer)
```

## API Reference

### MfsInterface Trait

```rust
pub trait MfsInterface: Send + Sync {
    /// Create a directory at the given path
    async fn mkdir(&self, path: &str) -> Result<(), MfsError>;
    
    /// Write bytes to a file at the given path
    async fn write_bytes(&self, bytes: &[u8], path: &str) -> Result<(), MfsError>;
    
    /// List entries in a directory (None for root)
    async fn ls(&self, path: Option<&str>) -> Result<Vec<UnixFSEntry>, MfsError>;
}
```

### Path Format

All paths must:
- Start with `/` (absolute paths)
- Use `/` as the path separator
- Cannot write to root (must have a filename)

Examples:
- ✅ `/file.txt`
- ✅ `/documents/report.pdf`
- ✅ `/data/2024/january/metrics.json`
- ❌ `file.txt` (no leading slash)
- ❌ `/` (cannot write to root)

### Error Types

```rust
pub enum MfsError {
    InvalidPath(String),  // Path format errors
    UnixFs(String),       // Underlying UnixFS errors
}
```

## Current Status

✅ **Implemented**:
- Directory creation
- File writing
- Directory listing
- Mutable root management
- Path parsing and validation
- Integration with UnixFS

⚠️ **Limitations**:
- No nested directory creation (must create parent first)
- No file reading (use UnixFS directly with CIDs)
- No file/directory deletion
- No file/directory moving or copying
- Simplified path handling (single-level only)
- No path resolution through symlinks

🔄 **Future Enhancements**:
- Recursive directory creation (`mkdir -p` equivalent)
- File reading operations
- File and directory deletion
- Move and copy operations
- Nested path traversal
- Symlink support
- Stat operations through paths
- Touch operations (update timestamps)
- Chmod operations (change permissions)

## Differences from JavaScript Implementation

| Feature | This Implementation | JS Helia |
|---------|-------------------|----------|
| Basic operations | ✅ mkdir, write, ls | ✅ Full API |
| Path resolution | ⚠️ Simplified | ✅ Complete |
| Nested paths | ❌ Single level | ✅ Full support |
| File operations | ⚠️ Write only | ✅ Read/write |
| Root management | ✅ Automatic | ✅ Configurable |

## Examples

### Complete Workflow

```rust
use helia_mfs::{mfs, MfsInterface};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Helia and MFS
    let helia = helia::create_helia_default().await?;
    let fs = mfs(Arc::new(helia));

    // Create directory structure
    fs.mkdir("/projects").await?;
    fs.mkdir("/documents").await?;

    // Write project file
    let code = b"fn main() { println!(\"Hello!\"); }";
    fs.write_bytes(code, "/projects/main.rs").await?;

    // Write documentation
    let docs = b"# Project Documentation\n\nThis is my project.";
    fs.write_bytes(docs, "/documents/README.md").await?;

    // List all files
    println!("Root contents:");
    for entry in fs.ls(None).await? {
        println!("  - {} ({:?})", entry.name, entry.type_);
    }

    Ok(())
}
```

## Testing

Run the test suite:

```bash
cargo test
```

Current tests:
- `test_mkdir`: Verify directory creation
- `test_write_bytes`: Verify file writing
- `test_ls_root`: Verify directory listing

## Contributing

Contributions are welcome! Areas for improvement:
- Nested path support
- Additional filesystem operations
- Better error messages
- Performance optimizations

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Resources

- [MFS Specification](https://github.com/ipfs/specs/blob/master/MFS.md)
- [UnixFS Documentation](https://github.com/ipfs/specs/blob/master/UNIXFS.md)
- [Helia Documentation](https://helia.io)
- [IPFS Documentation](https://docs.ipfs.tech)
