# helia-unixfs

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust implementation of the UnixFS filesystem for IPFS/Helia.

## Overview

This crate provides a filesystem abstraction over IPFS, enabling file and directory operations with content-addressed storage. It implements the UnixFS specification, allowing you to create, read, update, and delete files and directories on IPFS.

## Features

- **File Operations**: Add files with metadata (mode, mtime), read file contents, partial reads with offset/length
- **Directory Operations**: Create directories, list contents, add/remove entries
- **Content Addressing**: All operations return CIDs (Content Identifiers)
- **Pinning Support**: Optional pinning of content for persistence
- **Metadata Support**: Unix-style permissions and modification times
- **Streaming APIs**: Efficient streaming of directory listings

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
helia-unixfs = "0.1.0"
```

## Usage

### Basic File Operations

```rust
use helia_unixfs::{UnixFS, UnixFSInterface};
use bytes::Bytes;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let helia = helia::create_helia_default().await?;
    let fs = UnixFS::new(Arc::new(helia));

    // Add a file
    let data = Bytes::from("Hello, IPFS!");
    let cid = fs.add_bytes(data, None).await?;
    println!("Added file: {}", cid);

    // Read the file
    let content = fs.cat(&cid, None).await?;
    println!("File content: {:?}", content);

    Ok(())
}
```

### File with Metadata

```rust
use helia_unixfs::{FileCandidate, AddOptions};

// Create a file with permissions and metadata
let file = FileCandidate {
    path: "document.txt".to_string(),
    content: Bytes::from("Important data"),
    mode: Some(0o644),  // rw-r--r--
    mtime: None,         // Uses current time
};

let cid = fs.add_file(file, Some(AddOptions {
    pin: true,  // Pin the content
    ..Default::default()
})).await?;
```

### Directory Operations

```rust
use helia_unixfs::{DirectoryCandidate, MkdirOptions};

// Create a directory
let dir_cid = fs.add_directory(None, None).await?;

// Create a subdirectory
let updated_dir_cid = fs.mkdir(
    &dir_cid, 
    "documents", 
    Some(MkdirOptions {
        mode: Some(0o755),
        ..Default::default()
    })
).await?;

// Add a file to the directory
let file_cid = fs.add_bytes(Bytes::from("test"), None).await?;
let dir_with_file = fs.cp(&file_cid, &dir_cid, "test.txt", None).await?;

// List directory contents
let mut entries = fs.ls(&dir_with_file, None).await?;
while let Some(entry) = entries.next().await {
    println!("Entry: {} ({})", entry.name, entry.cid);
}
```

### Partial File Reads

```rust
use helia_unixfs::CatOptions;

// Read from offset 10, length 20
let options = CatOptions {
    offset: Some(10),
    length: Some(20),
};
let partial_data = fs.cat(&cid, Some(options)).await?;
```

### Remove Files from Directory

```rust
// Remove a file from a directory
let updated_dir_cid = fs.rm(&dir_cid, "test.txt", None).await?;
```

### Get File/Directory Statistics

```rust
use helia_unixfs::UnixFSStat;

let stat = fs.stat(&cid, None).await?;
match stat {
    UnixFSStat::File(file_stat) => {
        println!("File size: {} bytes", file_stat.size);
        println!("Mode: {:?}", file_stat.mode);
    }
    UnixFSStat::Directory(dir_stat) => {
        println!("Directory entries: {}", dir_stat.entries);
    }
}
```

## Architecture

The crate is structured as follows:

- **UnixFSInterface**: Trait defining all filesystem operations
- **UnixFS**: Main implementation of the filesystem
- **UnixFSData**: Protobuf-like structure for file/directory metadata
- **DirectoryNode**: Directory structure with links to children
- **Error Types**: Comprehensive error handling with `UnixFSError`

## API Reference

### Core Types

- `UnixFS`: Main filesystem implementation
- `UnixFSInterface`: Trait for filesystem operations
- `UnixFSType`: Enum for file system entry types (File, Directory, Symlink, Raw)
- `UnixFSStat`: Statistics for files and directories
- `UnixFSEntry`: Directory entry with name, CID, size, and type

### Options

- `AddOptions`: Options for adding content (pinning, chunking)
- `CatOptions`: Options for reading files (offset, length)
- `LsOptions`: Options for listing directories
- `CpOptions`: Options for copying files
- `MkdirOptions`: Options for creating directories (mode, mtime)
- `RmOptions`: Options for removing entries
- `StatOptions`: Options for getting statistics

## Current Status

✅ **Implemented**:
- File add/read operations
- Directory creation and management
- Content-addressed storage
- Metadata support (mode, mtime)
- Pinning support
- Partial file reads
- Directory listing with streaming
- File/directory statistics

⚠️ **Limitations**:
- Uses JSON serialization instead of protobuf (for simplicity)
- No chunking for large files
- Basic content addressing (not compatible with js-ipfs/go-ipfs)
- No symlink support yet
- No file updates (immutable by design)

🔄 **Future Enhancements**:
- Proper UnixFS protobuf encoding
- Large file chunking support
- Compatible CID generation with other IPFS implementations
- Symlink support
- More sophisticated directory structures
- Performance optimizations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Resources

- [UnixFS Specification](https://github.com/ipfs/specs/blob/master/UNIXFS.md)
- [Helia Documentation](https://helia.io)
- [IPFS Documentation](https://docs.ipfs.tech)
