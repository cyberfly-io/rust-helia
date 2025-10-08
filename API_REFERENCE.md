# Helia Rust API Reference

This document provides detailed API documentation for all major traits, types, and functions in Helia Rust.

## Table of Contents

- [Core Traits](#core-traits)
  - [Helia](#helia)
  - [Blocks](#blocks)
  - [Pins](#pins)
  - [Routing](#routing)
- [UnixFS](#unixfs)
- [DAG Codecs](#dag-codecs)
- [CAR Operations](#car-operations)
- [Configuration](#configuration)
- [Error Handling](#error-handling)
- [Types](#types)

## Core Traits

### Helia

The main interface for interacting with a Helia IPFS node.

```rust
pub trait Helia: Send + Sync {
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

#### Methods

##### `blockstore() -> &dyn Blocks`

Returns a reference to the node's blockstore for raw block operations.

**Example:**
```rust
let blockstore = helia.blockstore();
```

##### `datastore() -> &dyn Datastore`

Returns a reference to the node's datastore for key-value storage.

**Example:**
```rust
let datastore = helia.datastore();
```

##### `pins() -> &dyn Pins`

Returns a reference to the pinning interface for content persistence.

**Example:**
```rust
let pins = helia.pins();
```

##### `logger() -> &dyn ComponentLogger`

Returns a reference to the logger for recording events and debugging.

##### `routing() -> &dyn Routing`

Returns a reference to the content routing interface.

##### `dns() -> &TokioAsyncResolver`

Returns a reference to the DNS resolver for name resolution.

##### `metrics() -> Option<&dyn Metrics>`

Returns an optional reference to the metrics collector if enabled.

##### `async start() -> Result<(), HeliaError>`

Starts the Helia node and its networking components.

**Example:**
```rust
helia.start().await?;
```

##### `async stop() -> Result<(), HeliaError>`

Stops the Helia node and cleans up resources.

**Example:**
```rust
helia.stop().await?;
```

### Blocks

Interface for storing and retrieving raw blocks of data.

```rust
#[async_trait]
pub trait Blocks: Send + Sync {
    async fn get(
        &self,
        cid: &Cid,
        options: Option<GetBlockOptions>,
    ) -> Result<Bytes, HeliaError>;

    async fn put(
        &self,
        block: Bytes,
        options: Option<PutBlockOptions>,
    ) -> Result<Cid, HeliaError>;

    async fn has(
        &self,
        cid: &Cid,
        options: Option<HasBlockOptions>,
    ) -> Result<bool, HeliaError>;

    async fn delete(
        &self,
        cid: &Cid,
        options: Option<DeleteBlockOptions>,
    ) -> Result<(), HeliaError>;

    async fn put_many(
        &self,
        blocks: Vec<InputPair>,
        options: Option<PutManyBlocksOptions>,
    ) -> Result<Vec<Cid>, HeliaError>;

    async fn get_many(
        &self,
        cids: Vec<Cid>,
        options: Option<GetManyBlocksOptions>,
    ) -> Result<AwaitIterable<Result<Pair, HeliaError>>, HeliaError>;

    async fn delete_many(
        &self,
        cids: Vec<Cid>,
        options: Option<DeleteManyBlocksOptions>,
    ) -> Result<AwaitIterable<Result<Cid, HeliaError>>, HeliaError>;
}
```

#### Methods

##### `async get(cid, options) -> Result<Bytes, HeliaError>`

Retrieves a block by its CID.

**Parameters:**
- `cid: &Cid` - The Content Identifier of the block to retrieve
- `options: Option<GetBlockOptions>` - Optional configuration for the operation

**Returns:** The block data as `Bytes`

**Example:**
```rust
let data = blockstore.get(&cid, None).await?;
```

##### `async put(block, options) -> Result<Cid, HeliaError>`

Stores a block and returns its CID.

**Parameters:**
- `block: Bytes` - The block data to store
- `options: Option<PutBlockOptions>` - Optional configuration including progress tracking

**Returns:** The CID of the stored block

**Example:**
```rust
let cid = blockstore.put(Bytes::from("data"), None).await?;
```

##### `async has(cid, options) -> Result<bool, HeliaError>`

Checks if a block exists in the blockstore.

**Parameters:**
- `cid: &Cid` - The Content Identifier to check
- `options: Option<HasBlockOptions>` - Optional configuration

**Returns:** `true` if the block exists, `false` otherwise

**Example:**
```rust
let exists = blockstore.has(&cid, None).await?;
```

##### `async delete(cid, options) -> Result<(), HeliaError>`

Deletes a block from the blockstore.

**Parameters:**
- `cid: &Cid` - The Content Identifier of the block to delete
- `options: Option<DeleteBlockOptions>` - Optional configuration

**Example:**
```rust
blockstore.delete(&cid, None).await?;
```

##### `async put_many(blocks, options) -> Result<Vec<Cid>, HeliaError>`

Stores multiple blocks efficiently in a batch operation.

**Parameters:**
- `blocks: Vec<InputPair>` - Vector of blocks to store
- `options: Option<PutManyBlocksOptions>` - Optional configuration

**Returns:** Vector of CIDs for the stored blocks

**Example:**
```rust
let blocks = vec![
    InputPair { cid: None, block: Bytes::from("block1") },
    InputPair { cid: None, block: Bytes::from("block2") },
];
let cids = blockstore.put_many(blocks, None).await?;
```

### Pins

Interface for pinning content to prevent garbage collection.

```rust
#[async_trait]
pub trait Pins: Send + Sync {
    async fn add(
        &self,
        cid: &Cid,
        options: Option<AddPinOptions>,
    ) -> Result<(), HeliaError>;

    async fn rm(
        &self,
        cid: &Cid,
        options: Option<RmPinOptions>,
    ) -> Result<(), HeliaError>;

    async fn ls(
        &self,
        options: Option<LsPinOptions>,
    ) -> Result<AwaitIterable<Cid>, HeliaError>;

    async fn is_pinned(
        &self,
        cid: &Cid,
        options: Option<IsPinnedOptions>,
    ) -> Result<bool, HeliaError>;
}
```

#### Methods

##### `async add(cid, options) -> Result<(), HeliaError>`

Pins content to protect it from garbage collection.

**Parameters:**
- `cid: &Cid` - The Content Identifier to pin
- `options: Option<AddPinOptions>` - Optional configuration including depth

**Example:**
```rust
pins.add(&cid, None).await?;
```

##### `async rm(cid, options) -> Result<(), HeliaError>`

Unpins content, allowing it to be garbage collected.

**Parameters:**
- `cid: &Cid` - The Content Identifier to unpin
- `options: Option<RmPinOptions>` - Optional configuration

**Example:**
```rust
pins.rm(&cid, None).await?;
```

##### `async ls(options) -> Result<AwaitIterable<Cid>, HeliaError>`

Lists all pinned content.

**Parameters:**
- `options: Option<LsPinOptions>` - Optional configuration

**Returns:** A stream of CIDs for all pinned content

**Example:**
```rust
use futures::StreamExt;

let mut stream = pins.ls(None).await?;
while let Some(cid) = stream.next().await {
    println!("Pinned: {}", cid);
}
```

##### `async is_pinned(cid, options) -> Result<bool, HeliaError>`

Checks if content is pinned.

**Parameters:**
- `cid: &Cid` - The Content Identifier to check
- `options: Option<IsPinnedOptions>` - Optional configuration

**Returns:** `true` if pinned, `false` otherwise

**Example:**
```rust
let pinned = pins.is_pinned(&cid, None).await?;
```

### Routing

Interface for content routing and peer discovery.

```rust
#[async_trait]
pub trait Routing: Send + Sync {
    async fn get(
        &self,
        key: &[u8],
    ) -> Result<Option<Bytes>, HeliaError>;

    async fn put(
        &self,
        key: Vec<u8>,
        value: Bytes,
    ) -> Result<(), HeliaError>;

    async fn provide(
        &self,
        cid: &Cid,
    ) -> Result<(), HeliaError>;

    async fn find_providers(
        &self,
        cid: &Cid,
        options: Option<FindProvidersOptions>,
    ) -> Result<AwaitIterable<PeerInfo>, HeliaError>;
}
```

## UnixFS

UnixFS provides file system operations compatible with IPFS.

### UnixFSInterface Trait

```rust
#[async_trait]
pub trait UnixFSInterface: Send + Sync {
    async fn add_bytes(
        &self,
        data: Bytes,
        options: Option<AddBytesOptions>,
    ) -> Result<Cid, UnixFSError>;

    async fn cat(
        &self,
        cid: &Cid,
        options: Option<CatOptions>,
    ) -> Result<Bytes, UnixFSError>;

    async fn ls(
        &self,
        cid: &Cid,
        options: Option<LsOptions>,
    ) -> Result<Vec<UnixFSEntry>, UnixFSError>;

    async fn stat(
        &self,
        cid: &Cid,
        options: Option<StatOptions>,
    ) -> Result<UnixFSStats, UnixFSError>;

    async fn add_directory(
        &self,
        entries: Option<Vec<(String, Cid)>>,
        options: Option<AddDirectoryOptions>,
    ) -> Result<Cid, UnixFSError>;

    async fn cp(
        &self,
        source: &Cid,
        target: &Cid,
        name: &str,
        options: Option<CpOptions>,
    ) -> Result<Cid, UnixFSError>;

    async fn mkdir(
        &self,
        cid: &Cid,
        dirname: &str,
        options: Option<MkdirOptions>,
    ) -> Result<Cid, UnixFSError>;

    async fn rm(
        &self,
        cid: &Cid,
        path: &str,
        options: Option<RmOptions>,
    ) -> Result<Cid, UnixFSError>;

    async fn chmod(
        &self,
        cid: &Cid,
        mode: u32,
        options: Option<ChmodOptions>,
    ) -> Result<Cid, UnixFSError>;

    async fn touch(
        &self,
        cid: &Cid,
        options: Option<TouchOptions>,
    ) -> Result<Cid, UnixFSError>;
}
```

### Key Methods

#### `async add_bytes(data, options) -> Result<Cid, UnixFSError>`

Adds file data to IPFS.

**Options:**
- `chunk_size: Option<usize>` - Size of chunks for large files
- `raw_leaves: bool` - Use raw leaves for file chunks
- `progress: ProgressOptions` - Progress tracking

**Example:**
```rust
let cid = fs.add_bytes(Bytes::from("hello"), None).await?;
```

#### `async cat(cid, options) -> Result<Bytes, UnixFSError>`

Reads file content from IPFS.

**Example:**
```rust
let data = fs.cat(&cid, None).await?;
```

#### `async ls(cid, options) -> Result<Vec<UnixFSEntry>, UnixFSError>`

Lists directory contents.

**Returns:** Vector of `UnixFSEntry` containing:
- `name: String` - Entry name
- `cid: Cid` - Entry CID
- `size: Option<u64>` - Entry size
- `type: FileType` - File or directory

**Example:**
```rust
let entries = fs.ls(&dir_cid, None).await?;
for entry in entries {
    println!("{}: {}", entry.name, entry.cid);
}
```

#### `async stat(cid, options) -> Result<UnixFSStats, UnixFSError>`

Gets file/directory statistics.

**Returns:** `UnixFSStats` containing:
- `file_type: FileType` - Type (file or directory)
- `file_size: u64` - Total size in bytes
- `blocks: usize` - Number of blocks
- `mode: Option<u32>` - Unix mode
- `mtime: Option<SystemTime>` - Modification time

**Example:**
```rust
let stats = fs.stat(&cid, None).await?;
println!("Size: {} bytes", stats.file_size);
```

## DAG Codecs

### DAG-CBOR

Interface for working with DAG-CBOR encoded data.

```rust
#[async_trait]
pub trait DagCborInterface<T>: Send + Sync
where
    T: Serialize + for<'de> Deserialize<'de> + Send,
{
    async fn add(
        &self,
        data: &T,
        options: Option<AddOptions>,
    ) -> Result<Cid, DagCborError>;

    async fn get(
        &self,
        cid: &Cid,
        options: Option<GetOptions>,
    ) -> Result<T, DagCborError>;
}
```

**Example:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyData {
    name: String,
    value: i32,
}

let dag = DagCbor::new(helia);
let data = MyData { name: "test".into(), value: 42 };
let cid = dag.add(&data, None).await?;
let retrieved: MyData = dag.get(&cid, None).await?;
```

### DAG-JSON

Similar to DAG-CBOR but uses JSON encoding.

```rust
#[async_trait]
pub trait DagJsonInterface<T>: Send + Sync
where
    T: Serialize + for<'de> Deserialize<'de> + Send,
{
    async fn add(
        &self,
        data: &T,
        options: Option<AddOptions>,
    ) -> Result<Cid, DagJsonError>;

    async fn get(
        &self,
        cid: &Cid,
        options: Option<GetOptions>,
    ) -> Result<T, DagJsonError>;
}
```

### JSON

Interface for regular JSON (without IPLD links).

```rust
#[async_trait]
pub trait JsonInterface: Send + Sync {
    async fn add(
        &self,
        data: &serde_json::Value,
        options: Option<AddOptions>,
    ) -> Result<Cid, JsonError>;

    async fn get(
        &self,
        cid: &Cid,
        options: Option<GetOptions>,
    ) -> Result<serde_json::Value, JsonError>;
}
```

## CAR Operations

Functions for working with CAR (Content Addressable aRchive) files.

### `async import_car(helia, path, options) -> Result<Vec<Cid>, CarError>`

Imports content from a CAR file.

**Parameters:**
- `helia: Arc<impl Helia>` - Helia node instance
- `path: &Path` - Path to CAR file
- `options: Option<ImportCarOptions>` - Optional configuration

**Returns:** Vector of root CIDs from the CAR file

**Example:**
```rust
use std::path::Path;

let roots = import_car(helia, Path::new("data.car"), None).await?;
```

### `async export_car(helia, cid, path, options) -> Result<(), CarError>`

Exports content to a CAR file.

**Parameters:**
- `helia: Arc<impl Helia>` - Helia node instance
- `cid: &Cid` - Root CID to export
- `path: &Path` - Output file path
- `options: Option<ExportCarOptions>` - Optional configuration

**Example:**
```rust
export_car(helia, &root_cid, Path::new("export.car"), None).await?;
```

## Configuration

### HeliaConfig

Main configuration structure for creating a Helia node.

```rust
pub struct HeliaConfig {
    pub blockstore: BlockstoreConfig,
    pub datastore: DatastoreConfig,
    pub logger: LoggerConfig,
    pub libp2p: Option<Arc<Mutex<Swarm<HeliaBehaviour>>>>,
    pub dns: Option<TokioAsyncResolver>,
    pub metrics: Option<Arc<dyn Metrics>>,
}
```

**Example:**
```rust
let config = HeliaConfig {
    blockstore: BlockstoreConfig {
        path: Some(PathBuf::from("./blocks")),
        ..Default::default()
    },
    datastore: DatastoreConfig {
        path: Some(PathBuf::from("./data")),
        ..Default::default()
    },
    ..Default::default()
};

let helia = create_helia(Some(config)).await?;
```

### BlockstoreConfig

Configuration for the blockstore.

```rust
pub struct BlockstoreConfig {
    pub path: Option<PathBuf>,
    // Additional configuration fields
}
```

### DatastoreConfig

Configuration for the datastore.

```rust
pub struct DatastoreConfig {
    pub path: Option<PathBuf>,
    // Additional configuration fields
}
```

### LoggerConfig

Configuration for logging.

```rust
pub struct LoggerConfig {
    pub enabled: bool,
    pub level: String,
}
```

## Error Handling

### HeliaError

Main error type for Helia operations.

```rust
pub struct HeliaError {
    error_type: HeliaErrorType,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}
```

#### Error Types

```rust
pub enum HeliaErrorType {
    NotFound,
    InvalidInput,
    Network,
    Storage,
    Timeout,
    Decode,
    Encode,
    Other,
}
```

#### Methods

```rust
impl HeliaError {
    pub fn error_type(&self) -> &HeliaErrorType;
    pub fn message(&self) -> &str;
    
    // Constructors
    pub fn not_found(msg: impl Into<String>) -> Self;
    pub fn invalid_input(msg: impl Into<String>) -> Self;
    pub fn network(msg: impl Into<String>) -> Self;
    pub fn storage(msg: impl Into<String>) -> Self;
    // ... more constructors
}
```

**Example:**
```rust
match operation().await {
    Ok(result) => println!("Success: {:?}", result),
    Err(e) => match e.error_type() {
        HeliaErrorType::NotFound => eprintln!("Content not found: {}", e),
        HeliaErrorType::Network => eprintln!("Network error: {}", e),
        _ => eprintln!("Error: {}", e),
    }
}
```

### Specialized Errors

- `UnixFSError` - Errors specific to UnixFS operations
- `DagCborError` - Errors for DAG-CBOR encoding/decoding
- `DagJsonError` - Errors for DAG-JSON encoding/decoding
- `JsonError` - Errors for JSON operations
- `CarError` - Errors for CAR file operations

## Types

### Cid (Content Identifier)

Content Identifiers from the `cid` crate.

```rust
use cid::Cid;
use std::str::FromStr;

let cid = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi")?;
```

### Bytes

Efficient byte buffer from the `bytes` crate.

```rust
use bytes::Bytes;

let data = Bytes::from("hello");
let data = Bytes::from(vec![1, 2, 3, 4]);
```

### Pair

A CID-block pair.

```rust
pub struct Pair {
    pub cid: Cid,
    pub block: Bytes,
}
```

### InputPair

Input for storing blocks.

```rust
pub struct InputPair {
    pub cid: Option<Cid>,
    pub block: Bytes,
}
```

### UnixFSEntry

Entry in a UnixFS directory listing.

```rust
pub struct UnixFSEntry {
    pub name: String,
    pub cid: Cid,
    pub size: Option<u64>,
    pub file_type: FileType,
}
```

### UnixFSStats

Statistics for a UnixFS entry.

```rust
pub struct UnixFSStats {
    pub file_type: FileType,
    pub file_size: u64,
    pub blocks: usize,
    pub mode: Option<u32>,
    pub mtime: Option<SystemTime>,
}
```

### FileType

Type of UnixFS entry.

```rust
pub enum FileType {
    File,
    Directory,
}
```

### ProgressOptions

Options for tracking operation progress.

```rust
pub struct ProgressOptions<T> {
    pub on_progress: Option<Box<dyn Fn(ProgressEvent<T>) + Send + Sync>>,
}
```

**Example:**
```rust
let options = ProgressOptions {
    on_progress: Some(Box::new(|event| {
        println!("Progress: {:?}", event.event_type);
    })),
};
```

## Async Patterns

### Streams

Many operations return async streams:

```rust
use futures::StreamExt;

let mut stream = pins.ls(None).await?;
while let Some(cid) = stream.next().await {
    println!("{}", cid);
}
```

### Arc for Sharing

Share Helia instances across threads:

```rust
use std::sync::Arc;

let helia = Arc::new(create_helia(None).await?);
let fs1 = UnixFS::new(helia.clone());
let fs2 = UnixFS::new(helia.clone());
```

## Best Practices

1. **Always start the node**: Call `start()` after creating a Helia instance
2. **Use Arc for sharing**: Wrap instances in `Arc` when sharing across components
3. **Handle errors specifically**: Match on error types for better error handling
4. **Pin important content**: Use pinning to prevent garbage collection
5. **Clean up**: Call `stop()` when done to release resources
6. **Use batch operations**: `put_many` is more efficient than multiple `put` calls

## See Also

- [Usage Guide](USAGE.md) - Comprehensive usage examples
- [Examples](examples/) - Working code examples
- [README](README.md) - Project overview
- [Helia TypeScript Docs](https://helia.io) - Original implementation

---

For more information and updates, visit the [Helia Rust repository](https://github.com/cyberfly-io/rust-helia).
