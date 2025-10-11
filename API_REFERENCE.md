# Rust Helia API Reference

Complete API documentation for all Rust Helia modules.

## Table of Contents

1. [Core Modules](#core-modules)
   - [helia](#helia)
   - [helia-interface](#helia-interface)
   - [helia-utils](#helia-utils)
2. [File System Modules](#file-system-modules)
   - [helia-unixfs](#helia-unixfs)
   - [helia-mfs](#helia-mfs)
3. [Data Format Modules](#data-format-modules)
   - [helia-dag-cbor](#helia-dag-cbor)
   - [helia-dag-json](#helia-dag-json)
   - [helia-json](#helia-json)
   - [helia-car](#helia-car)
4. [Network Modules](#network-modules)
   - [helia-bitswap](#helia-bitswap)
   - [helia-http](#helia-http)
   - [helia-block-brokers](#helia-block-brokers)
   - [helia-ipns](#helia-ipns)
   - [helia-dnslink](#helia-dnslink)
5. [Utility Modules](#utility-modules)
   - [helia-strings](#helia-strings)
   - [helia-routers](#helia-routers)

---

## Core Modules

### helia

**Description**: Core Helia instance creation and management.

#### Functions

##### `create_helia()`

Create a new Helia instance with default configuration.

```rust
pub async fn create_helia() -> Result<Arc<Helia>, HeliaError>
```

**Returns**: `Result<Arc<Helia>, HeliaError>`

**Example**:
```rust
use rust_helia::create_helia;

let helia = create_helia().await?;
```

##### `create_helia_with_config()`

Create a Helia instance with custom configuration.

```rust
pub async fn create_helia_with_config(
    config: HeliaConfig
) -> Result<Arc<Helia>, HeliaError>
```

**Parameters**:
- `config: HeliaConfig` - Custom configuration

**Example**:
```rust
let config = HeliaConfig {
    datastore: Some(custom_datastore),
    ..Default::default()
};
let helia = create_helia_with_config(config).await?;
```

#### Structs

##### `Helia`

Main Helia instance.

**Methods**:

###### `blocks()`

Access the blocks interface.

```rust
pub fn blocks(&self) -> &dyn Blocks
```

**Returns**: Reference to Blocks interface

**Example**:
```rust
let blocks = helia.blocks();
let cid = blocks.put(data, None).await?;
```

###### `pins()`

Access the pins interface.

```rust
pub fn pins(&self) -> &dyn Pins
```

**Returns**: Reference to Pins interface

**Example**:
```rust
let pins = helia.pins();
pins.add(&cid).await?;
```

###### `start()`

Start the Helia instance.

```rust
pub async fn start(&self) -> Result<(), HeliaError>
```

**Example**:
```rust
helia.start().await?;
```

###### `stop()`

Stop the Helia instance.

```rust
pub async fn stop(&self) -> Result<(), HeliaError>
```

**Example**:
```rust
helia.stop().await?;
```

###### `gc()`

Run garbage collection.

```rust
pub async fn gc(&self) -> Result<GcResult, HeliaError>
```

**Returns**: `GcResult` with statistics

**Example**:
```rust
let result = helia.gc().await?;
println!("Freed {} blocks", result.blocks_removed);
```

---

### helia-interface

**Description**: Core traits and types used across all modules.

#### Traits

##### `Blocks`

Block storage interface.

```rust
#[async_trait]
pub trait Blocks: Send + Sync {
    async fn get(&self, cid: &Cid, options: Option<GetOptions>) 
        -> Result<Bytes, HeliaError>;
    
    async fn put(&self, data: Bytes, options: Option<PutOptions>) 
        -> Result<Cid, HeliaError>;
    
    async fn has(&self, cid: &Cid, options: Option<HasOptions>) 
        -> Result<bool, HeliaError>;
    
    async fn delete(&self, cid: &Cid, options: Option<DeleteOptions>) 
        -> Result<(), HeliaError>;
    
    async fn delete_many(&self, cids: Vec<Cid>, options: Option<DeleteOptions>) 
        -> Result<Vec<Result<(), HeliaError>>, HeliaError>;
}
```

**Methods**:

###### `get()`

Retrieve block by CID.

**Parameters**:
- `cid: &Cid` - Content identifier
- `options: Option<GetOptions>` - Optional parameters

**Returns**: `Result<Bytes, HeliaError>` - Block data

**Errors**:
- `BlockNotFound` - Block not found
- `Network` - Network error

**Example**:
```rust
let blocks = helia.blocks();
let data = blocks.get(&cid, None).await?;
```

###### `put()`

Store a block.

**Parameters**:
- `data: Bytes` - Block data
- `options: Option<PutOptions>` - Optional parameters (codec, hasher)

**Returns**: `Result<Cid, HeliaError>` - Content identifier

**Example**:
```rust
use bytes::Bytes;

let data = Bytes::from("Hello IPFS!");
let cid = blocks.put(data, None).await?;
```

###### `has()`

Check if block exists.

**Parameters**:
- `cid: &Cid` - Content identifier
- `options: Option<HasOptions>` - Optional parameters

**Returns**: `Result<bool, HeliaError>` - True if block exists

**Example**:
```rust
if blocks.has(&cid, None).await? {
    println!("Block exists");
}
```

###### `delete()`

Delete a single block.

**Parameters**:
- `cid: &Cid` - Content identifier
- `options: Option<DeleteOptions>` - Optional parameters

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
blocks.delete(&cid, None).await?;
```

###### `delete_many()`

Delete multiple blocks.

**Parameters**:
- `cids: Vec<Cid>` - Content identifiers
- `options: Option<DeleteOptions>` - Optional parameters

**Returns**: `Result<Vec<Result<(), HeliaError>>, HeliaError>` - Results per CID

**Example**:
```rust
let results = blocks.delete_many(vec![cid1, cid2], None).await?;
```

##### `Pins`

Pin management interface.

```rust
#[async_trait]
pub trait Pins: Send + Sync {
    async fn add(&self, cid: &Cid) -> Result<(), HeliaError>;
    async fn remove(&self, cid: &Cid) -> Result<(), HeliaError>;
    async fn list(&self) -> Result<Vec<Cid>, HeliaError>;
    async fn is_pinned(&self, cid: &Cid) -> Result<bool, HeliaError>;
}
```

**Methods**:

###### `add()`

Pin content (prevent garbage collection).

**Parameters**:
- `cid: &Cid` - Content identifier

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
helia.pins().add(&cid).await?;
```

###### `remove()`

Unpin content.

**Parameters**:
- `cid: &Cid` - Content identifier

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
helia.pins().remove(&cid).await?;
```

###### `list()`

List all pinned CIDs.

**Returns**: `Result<Vec<Cid>, HeliaError>`

**Example**:
```rust
let pinned = helia.pins().list().await?;
for cid in pinned {
    println!("Pinned: {}", cid);
}
```

###### `is_pinned()`

Check if content is pinned.

**Parameters**:
- `cid: &Cid` - Content identifier

**Returns**: `Result<bool, HeliaError>`

**Example**:
```rust
if helia.pins().is_pinned(&cid).await? {
    println!("Content is pinned");
}
```

##### `Routing`

Content routing interface.

```rust
#[async_trait]
pub trait Routing: Send + Sync {
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<PeerId>, HeliaError>;
    async fn provide(&self, cid: &Cid) -> Result<(), HeliaError>;
}
```

#### Enums

##### `HeliaError`

All possible errors.

```rust
#[derive(Debug, Error)]
pub enum HeliaError {
    #[error("Block not found: {cid}")]
    BlockNotFound { cid: Cid },
    
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("Invalid CID: {cid}")]
    InvalidCid { cid: String },
    
    #[error("Timeout")]
    Timeout,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Encoding error: {0}")]
    Encoding(String),
    
    #[error("Not supported: {0}")]
    NotSupported(String),
}
```

---

### helia-utils

**Description**: Shared utilities and helpers.

#### Structs

##### `MemoryDatastore`

In-memory datastore implementation.

```rust
pub struct MemoryDatastore {
    // Private fields
}
```

**Methods**:

###### `new()`

Create a new in-memory datastore.

```rust
pub fn new() -> Self
```

**Example**:
```rust
use helia_utils::blockstore::MemoryDatastore;

let datastore = MemoryDatastore::new();
```

##### `Logger`

Logging utilities.

```rust
pub struct Logger;
```

**Methods**:

###### `init()`

Initialize logging.

```rust
pub fn init()
```

**Example**:
```rust
use helia_utils::logger::Logger;

Logger::init();
```

---

## File System Modules

### helia-unixfs

**Description**: Unix file system interface for IPFS.

#### Structs

##### `UnixFS`

UnixFS file operations.

```rust
pub struct UnixFS {
    helia: Arc<dyn Blocks>,
}
```

**Methods**:

###### `new()`

Create UnixFS instance.

```rust
pub fn new(helia: Arc<dyn Blocks>) -> Self
```

**Parameters**:
- `helia: Arc<dyn Blocks>` - Helia instance

**Example**:
```rust
use helia_unixfs::UnixFS;

let fs = UnixFS::new(helia);
```

###### `add_bytes()`

Add bytes as a file.

```rust
pub async fn add_bytes(&self, content: &[u8]) 
    -> Result<Cid, HeliaError>
```

**Parameters**:
- `content: &[u8]` - File content

**Returns**: `Result<Cid, HeliaError>` - Root CID

**Example**:
```rust
let cid = fs.add_bytes(b"Hello IPFS!").await?;
```

###### `add_file()`

Add file from path.

```rust
pub async fn add_file(&self, path: &Path) 
    -> Result<Cid, HeliaError>
```

**Parameters**:
- `path: &Path` - File path

**Returns**: `Result<Cid, HeliaError>` - Root CID

**Example**:
```rust
use std::path::Path;

let cid = fs.add_file(Path::new("file.txt")).await?;
```

###### `add_directory()`

Add directory recursively.

```rust
pub async fn add_directory(&self, path: &Path) 
    -> Result<Cid, HeliaError>
```

**Parameters**:
- `path: &Path` - Directory path

**Returns**: `Result<Cid, HeliaError>` - Root CID

**Example**:
```rust
let cid = fs.add_directory(Path::new("./docs")).await?;
```

###### `cat()`

Read file contents.

```rust
pub async fn cat(&self, cid: &Cid) 
    -> Result<Vec<u8>, HeliaError>
```

**Parameters**:
- `cid: &Cid` - File CID

**Returns**: `Result<Vec<u8>, HeliaError>` - File contents

**Example**:
```rust
let content = fs.cat(&cid).await?;
println!("{}", String::from_utf8_lossy(&content));
```

###### `ls()`

List directory contents.

```rust
pub async fn ls(&self, cid: &Cid) 
    -> Result<Vec<UnixFSEntry>, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Directory CID

**Returns**: `Result<Vec<UnixFSEntry>, HeliaError>` - Directory entries

**Example**:
```rust
let entries = fs.ls(&cid).await?;
for entry in entries {
    println!("{} - {}", entry.name, entry.cid);
}
```

###### `stat()`

Get file/directory metadata.

```rust
pub async fn stat(&self, cid: &Cid) 
    -> Result<UnixFSStat, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content CID

**Returns**: `Result<UnixFSStat, HeliaError>` - Metadata

**Example**:
```rust
let stat = fs.stat(&cid).await?;
println!("Size: {} bytes", stat.size);
println!("Type: {:?}", stat.type_);
```

#### Structs

##### `UnixFSEntry`

Directory entry.

```rust
pub struct UnixFSEntry {
    pub name: String,
    pub cid: Cid,
    pub size: u64,
    pub type_: UnixFSType,
}
```

##### `UnixFSStat`

File/directory metadata.

```rust
pub struct UnixFSStat {
    pub cid: Cid,
    pub size: u64,
    pub type_: UnixFSType,
    pub blocks: u64,
}
```

#### Enums

##### `UnixFSType`

Entry type.

```rust
pub enum UnixFSType {
    File,
    Directory,
    Raw,
}
```

---

### helia-mfs

**Description**: Mutable File System - Unix-like interface for IPFS.

#### Structs

##### `MFS`

Mutable file system operations.

```rust
pub struct MFS {
    helia: Arc<dyn Blocks>,
}
```

**Methods**:

###### `new()`

Create MFS instance.

```rust
pub fn new(helia: Arc<dyn Blocks>) -> Self
```

**Example**:
```rust
use helia_mfs::MFS;

let mfs = MFS::new(helia);
```

###### `write()`

Write file contents.

```rust
pub async fn write(&self, path: &str, content: &[u8]) 
    -> Result<(), HeliaError>
```

**Parameters**:
- `path: &str` - File path (e.g., "/docs/file.txt")
- `content: &[u8]` - File content

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
mfs.write("/docs/readme.txt", b"Hello MFS!").await?;
```

###### `read()`

Read file contents.

```rust
pub async fn read(&self, path: &str) 
    -> Result<Vec<u8>, HeliaError>
```

**Parameters**:
- `path: &str` - File path

**Returns**: `Result<Vec<u8>, HeliaError>` - File contents

**Example**:
```rust
let content = mfs.read("/docs/readme.txt").await?;
```

###### `cat()`

Alias for `read()`.

```rust
pub async fn cat(&self, path: &str) 
    -> Result<Vec<u8>, HeliaError>
```

###### `mkdir()`

Create directory.

```rust
pub async fn mkdir(&self, path: &str) 
    -> Result<(), HeliaError>
```

**Parameters**:
- `path: &str` - Directory path

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
mfs.mkdir("/docs").await?;
mfs.mkdir("/docs/api").await?;
```

###### `ls()`

List directory contents.

```rust
pub async fn ls(&self, path: &str) 
    -> Result<Vec<MFSEntry>, HeliaError>
```

**Parameters**:
- `path: &str` - Directory path

**Returns**: `Result<Vec<MFSEntry>, HeliaError>` - Directory entries

**Example**:
```rust
let entries = mfs.ls("/docs").await?;
for entry in entries {
    println!("{} - {}", entry.name, entry.size);
}
```

###### `rm()`

Remove file or directory.

```rust
pub async fn rm(&self, path: &str, recursive: bool) 
    -> Result<(), HeliaError>
```

**Parameters**:
- `path: &str` - Path to remove
- `recursive: bool` - Remove recursively if directory

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
mfs.rm("/docs/old.txt", false).await?;
mfs.rm("/old_docs", true).await?; // recursive
```

###### `stat()`

Get file/directory metadata.

```rust
pub async fn stat(&self, path: &str) 
    -> Result<MFSStat, HeliaError>
```

**Parameters**:
- `path: &str` - Path

**Returns**: `Result<MFSStat, HeliaError>` - Metadata

**Example**:
```rust
let stat = mfs.stat("/docs/readme.txt").await?;
println!("CID: {}", stat.cid);
println!("Size: {} bytes", stat.size);
```

###### `mv()`

Move/rename file or directory.

```rust
pub async fn mv(&self, from: &str, to: &str) 
    -> Result<(), HeliaError>
```

**Parameters**:
- `from: &str` - Source path
- `to: &str` - Destination path

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
mfs.mv("/docs/old.txt", "/docs/new.txt").await?;
```

###### `cp()`

Copy file or directory.

```rust
pub async fn cp(&self, from: &str, to: &str) 
    -> Result<(), HeliaError>
```

**Parameters**:
- `from: &str` - Source path
- `to: &str` - Destination path

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
mfs.cp("/docs/file.txt", "/backup/file.txt").await?;
```

#### Structs

##### `MFSEntry`

Directory entry.

```rust
pub struct MFSEntry {
    pub name: String,
    pub type_: MFSType,
    pub size: u64,
    pub cid: Cid,
}
```

##### `MFSStat`

File/directory metadata.

```rust
pub struct MFSStat {
    pub cid: Cid,
    pub size: u64,
    pub type_: MFSType,
    pub blocks: u64,
}
```

#### Enums

##### `MFSType`

Entry type.

```rust
pub enum MFSType {
    File,
    Directory,
}
```

---

## Data Format Modules

### helia-dag-cbor

**Description**: DAG-CBOR encoding/decoding for structured data.

#### Structs

##### `DagCbor`

DAG-CBOR operations.

```rust
pub struct DagCbor {
    helia: Arc<dyn Blocks>,
}
```

**Methods**:

###### `new()`

Create DagCbor instance.

```rust
pub fn new(helia: Arc<dyn Blocks>) -> Self
```

**Example**:
```rust
use helia_dag_cbor::DagCbor;

let dag_cbor = DagCbor::new(helia);
```

###### `put()`

Store CBOR-encoded data.

```rust
pub async fn put<T: Serialize>(
    &self, 
    value: &T, 
    options: Option<PutOptions>
) -> Result<Cid, HeliaError>
```

**Parameters**:
- `value: &T` - Data to encode (must implement `Serialize`)
- `options: Option<PutOptions>` - Optional parameters

**Returns**: `Result<Cid, HeliaError>` - Content identifier

**Example**:
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

let person = Person {
    name: "Alice".to_string(),
    age: 30,
};

let cid = dag_cbor.put(&person, None).await?;
```

###### `get()`

Retrieve and decode CBOR data.

```rust
pub async fn get<T: DeserializeOwned>(
    &self, 
    cid: &Cid, 
    options: Option<GetOptions>
) -> Result<T, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content identifier
- `options: Option<GetOptions>` - Optional parameters

**Returns**: `Result<T, HeliaError>` - Decoded data

**Example**:
```rust
let person: Person = dag_cbor.get(&cid, None).await?;
println!("Name: {}, Age: {}", person.name, person.age);
```

---

### helia-dag-json

**Description**: DAG-JSON encoding/decoding for structured data.

#### Structs

##### `DagJson`

DAG-JSON operations.

```rust
pub struct DagJson {
    helia: Arc<dyn Blocks>,
}
```

**Methods**:

###### `new()`

Create DagJson instance.

```rust
pub fn new(helia: Arc<dyn Blocks>) -> Self
```

**Example**:
```rust
use helia_dag_json::DagJson;

let dag_json = DagJson::new(helia);
```

###### `put()`

Store JSON-encoded data.

```rust
pub async fn put<T: Serialize>(
    &self, 
    value: &T, 
    options: Option<PutOptions>
) -> Result<Cid, HeliaError>
```

**Parameters**:
- `value: &T` - Data to encode
- `options: Option<PutOptions>` - Optional parameters

**Returns**: `Result<Cid, HeliaError>` - Content identifier

**Example**:
```rust
#[derive(Serialize)]
struct Config {
    api_key: String,
    timeout: u64,
}

let config = Config {
    api_key: "abc123".to_string(),
    timeout: 30,
};

let cid = dag_json.put(&config, None).await?;
```

###### `get()`

Retrieve and decode JSON data.

```rust
pub async fn get<T: DeserializeOwned>(
    &self, 
    cid: &Cid, 
    options: Option<GetOptions>
) -> Result<T, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content identifier
- `options: Option<GetOptions>` - Optional parameters

**Returns**: `Result<T, HeliaError>` - Decoded data

**Example**:
```rust
let config: Config = dag_json.get(&cid, None).await?;
```

---

### helia-json

**Description**: Simple JSON encoding/decoding.

#### Structs

##### `Json`

JSON operations.

```rust
pub struct Json {
    helia: Arc<dyn Blocks>,
}
```

**Methods**: Similar to DAG-JSON but simpler (no IPLD links).

---

### helia-car

**Description**: Content Archive (CAR) file operations.

#### Functions

##### `import_car()`

Import CAR file.

```rust
pub async fn import_car(
    helia: &impl Blocks, 
    path: &str
) -> Result<Cid, HeliaError>
```

**Parameters**:
- `helia: &impl Blocks` - Helia instance
- `path: &str` - CAR file path

**Returns**: `Result<Cid, HeliaError>` - Root CID

**Example**:
```rust
use helia_car::import_car;

let root = import_car(&helia, "archive.car").await?;
println!("Imported root: {}", root);
```

##### `export_car()`

Export to CAR file.

```rust
pub async fn export_car(
    helia: &impl Blocks, 
    root: &Cid, 
    path: &str
) -> Result<(), HeliaError>
```

**Parameters**:
- `helia: &impl Blocks` - Helia instance
- `root: &Cid` - Root CID to export
- `path: &str` - Output CAR file path

**Returns**: `Result<(), HeliaError>`

**Example**:
```rust
use helia_car::export_car;

export_car(&helia, &root_cid, "output.car").await?;
```

---

## Network Modules

### helia-http

**Description**: Pure HTTP gateway client (no P2P networking).

#### Functions

##### `create_helia_http()`

Create HTTP-only Helia instance.

```rust
pub async fn create_helia_http() -> Result<Arc<HeliaHttp>, HeliaError>
```

**Returns**: `Result<Arc<HeliaHttp>, HeliaError>`

**Example**:
```rust
use helia_http::create_helia_http;

let helia = create_helia_http().await?;
```

#### Structs

##### `HeliaHttp`

HTTP-only Helia client.

**Methods**:

###### `with_config()`

Create with custom gateway configuration.

```rust
pub async fn with_config(config: GatewayConfig) 
    -> Result<Arc<Self>, HeliaError>
```

**Parameters**:
- `config: GatewayConfig` - Gateway configuration

**Returns**: `Result<Arc<Self>, HeliaError>`

**Example**:
```rust
use helia_http::{HeliaHttp, GatewayConfig};

let config = GatewayConfig {
    gateways: vec![
        "https://ipfs.io".to_string(),
        "https://dweb.link".to_string(),
    ],
    timeout_secs: 60,
    max_retries: 5,
};

let helia = HeliaHttp::with_config(config).await?;
```

##### `GatewayConfig`

Gateway configuration.

```rust
pub struct GatewayConfig {
    pub gateways: Vec<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
}
```

**Default**:
```rust
GatewayConfig {
    gateways: vec![
        "https://ipfs.io".to_string(),
        "https://dweb.link".to_string(),
    ],
    timeout_secs: 30,
    max_retries: 3,
}
```

---

### helia-block-brokers

**Description**: Trustless gateway block retrieval.

#### Structs

##### `TrustlessGateway`

Trustless gateway client.

```rust
pub struct TrustlessGateway {
    // Private fields
}
```

**Methods**:

###### `new()`

Create trustless gateway client.

```rust
pub fn new(gateway_url: &str) -> Self
```

**Parameters**:
- `gateway_url: &str` - Gateway URL

**Example**:
```rust
use helia_block_brokers::TrustlessGateway;

let gateway = TrustlessGateway::new("https://trustless-gateway.link");
```

###### `fetch_block()`

Fetch and verify block.

```rust
pub async fn fetch_block(&self, cid: &Cid) 
    -> Result<Bytes, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content identifier

**Returns**: `Result<Bytes, HeliaError>` - Verified block data

**Example**:
```rust
let block = gateway.fetch_block(&cid).await?;
```

---

### helia-dnslink

**Description**: DNSLink resolution for human-readable addresses.

#### Functions

##### `dns_link()`

Create DNSLink resolver.

```rust
pub fn dns_link() -> DnsLink
```

**Returns**: `DnsLink` resolver instance

**Example**:
```rust
use helia_dnslink::dns_link;

let dnslink = dns_link();
```

#### Structs

##### `DnsLink`

DNSLink resolver.

**Methods**:

###### `resolve()`

Resolve domain to IPFS path.

```rust
pub async fn resolve(&self, domain: &str) 
    -> Result<String, HeliaError>
```

**Parameters**:
- `domain: &str` - Domain name

**Returns**: `Result<String, HeliaError>` - IPFS path

**Example**:
```rust
let path = dnslink.resolve("ipfs.tech").await?;
println!("Resolved to: {}", path); // /ipfs/Qm...
```

###### `resolve_recursive()`

Resolve recursively (follow CNAMEs).

```rust
pub async fn resolve_recursive(&self, domain: &str, depth: u32) 
    -> Result<String, HeliaError>
```

**Parameters**:
- `domain: &str` - Domain name
- `depth: u32` - Maximum recursion depth

**Returns**: `Result<String, HeliaError>` - Final IPFS path

**Example**:
```rust
let path = dnslink.resolve_recursive("example.com", 32).await?;
```

---

### helia-bitswap

**Description**: Bitswap protocol for P2P block exchange.

#### Structs

##### `Bitswap`

Bitswap client.

**Methods**:

###### `new()`

Create Bitswap client.

```rust
pub fn new(network: Network) -> Self
```

###### `want()`

Request block from peers.

```rust
pub async fn want(&self, cid: &Cid) 
    -> Result<Bytes, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content identifier

**Returns**: `Result<Bytes, HeliaError>` - Block data

---

### helia-ipns

**Description**: InterPlanetary Name System - mutable pointers.

#### Structs

##### `IPNS`

IPNS operations.

**Methods**:

###### `publish()`

Publish IPNS record.

```rust
pub async fn publish(&self, cid: &Cid) 
    -> Result<IpnsName, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content to publish

**Returns**: `Result<IpnsName, HeliaError>` - IPNS name

###### `resolve()`

Resolve IPNS name to CID.

```rust
pub async fn resolve(&self, name: &IpnsName) 
    -> Result<Cid, HeliaError>
```

**Parameters**:
- `name: &IpnsName` - IPNS name

**Returns**: `Result<Cid, HeliaError>` - Current CID

---

## Utility Modules

### helia-strings

**Description**: String encoding/decoding utilities.

#### Structs

##### `Strings`

String operations.

```rust
pub struct Strings {
    helia: Arc<dyn Blocks>,
}
```

**Methods**:

###### `new()`

Create Strings instance.

```rust
pub fn new(helia: Arc<dyn Blocks>) -> Self
```

###### `put()`

Store string.

```rust
pub async fn put(&self, text: &str) 
    -> Result<Cid, HeliaError>
```

**Parameters**:
- `text: &str` - String to store

**Returns**: `Result<Cid, HeliaError>` - Content identifier

**Example**:
```rust
use helia_strings::Strings;

let strings = Strings::new(helia);
let cid = strings.put("Hello, world!").await?;
```

###### `get()`

Retrieve string.

```rust
pub async fn get(&self, cid: &Cid) 
    -> Result<String, HeliaError>
```

**Parameters**:
- `cid: &Cid` - Content identifier

**Returns**: `Result<String, HeliaError>` - String content

**Example**:
```rust
let text = strings.get(&cid).await?;
println!("{}", text);
```

---

### helia-routers

**Description**: Content routing (DHT, delegated routing).

#### Structs

##### `DHTRouter`

DHT-based content routing.

**Methods**:

###### `find_providers()`

Find providers for content.

```rust
pub async fn find_providers(&self, cid: &Cid) 
    -> Result<Vec<PeerId>, HeliaError>
```

---

## Common Patterns

### Pattern 1: Store and Retrieve

```rust
// Store
let cid = fs.add_bytes(b"data").await?;

// Retrieve
let data = fs.cat(&cid).await?;
```

### Pattern 2: Error Handling

```rust
use helia_interface::errors::HeliaError;

match fs.cat(&cid).await {
    Ok(data) => println!("Got {} bytes", data.len()),
    Err(HeliaError::BlockNotFound { cid }) => {
        eprintln!("Not found: {}", cid);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Pattern 3: Concurrent Operations

```rust
use tokio::try_join;

let (cid1, cid2) = try_join!(
    fs.add_bytes(b"file 1"),
    fs.add_bytes(b"file 2")
)?;
```

### Pattern 4: Pinning

```rust
// Pin to prevent GC
helia.pins().add(&cid).await?;

// Do work...

// Unpin when done
helia.pins().remove(&cid).await?;
```

### Pattern 5: Structured Data

```rust
use helia_dag_cbor::DagCbor;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Data {
    field: String,
}

let dag = DagCbor::new(helia);
let cid = dag.put(&data, None).await?;
let retrieved: Data = dag.get(&cid, None).await?;
```

---

## Type Reference

### Common Types

```rust
// From cid crate
pub use cid::Cid;

// From bytes crate  
pub use bytes::Bytes;

// From libp2p
pub use libp2p::PeerId;

// Standard library
pub use std::sync::Arc;
pub use std::path::Path;
```

### Result Types

```rust
pub type Result<T> = std::result::Result<T, HeliaError>;
```

---

## Feature Flags

### helia-http

```toml
[dependencies.helia-http]
version = "0.1"
default-features = false
features = ["trustless-gateway"]
```

### helia-car

```toml
[dependencies.helia-car]
version = "0.1"
features = ["stream-import", "stream-export"]
```

---

## See Also

- **[User Guide](USER_GUIDE.md)** - Usage examples and tutorials
- **[Getting Started](GETTING_STARTED.md)** - Quick start guide
- **[Architecture](ARCHITECTURE.md)** - System design
- **[Examples](examples/)** - Working code examples

---

**API Version**: 0.1.0  
**Last Updated**: October 11, 2025  
**Status**: Production-ready
