//! # Helia UnixFS
//!
//! A Rust implementation of the IPFS UnixFS filesystem, providing file and directory
//! operations with content-addressed storage.
//!
//! ## Overview
//!
//! UnixFS is a protobuf-based format for representing files and directories on IPFS.
//! This crate provides a high-level interface for:
//!
//! - **File Operations**: Store and retrieve files with automatic chunking for large files
//! - **Directory Operations**: Create, modify, and traverse directory structures
//! - **Metadata Support**: Unix-style permissions (mode) and modification times (mtime)
//! - **Content Addressing**: All operations return CIDs (Content Identifiers)
//! - **Efficient Chunking**: Automatic chunking for files >1MB with configurable chunk size
//!
//! ## Core Concepts
//!
//! ### Content Addressing
//! Every file and directory is identified by a CID, ensuring:
//! - **Immutability**: Content cannot be changed without changing the CID
//! - **Deduplication**: Identical content has the same CID
//! - **Verification**: Content can be verified against its CID
//!
//! ### DAG-PB vs Raw Blocks
//! - **Small files (<256KB)**: Can be stored as either DAG-PB or raw blocks
//! - **Large files (>256KB)**: Automatically chunked and stored as DAG-PB with links
//! - **Directories**: Always stored as DAG-PB with links to entries
//!
//! ### Chunking Strategy
//! Large files are split into chunks for efficient storage and retrieval:
//! - **Default chunk size**: 262,144 bytes (256KB)
//! - **Configurable**: Set `chunk_size` in `AddOptions`
//! - **Merkle DAG**: Chunks are organized in a balanced tree structure
//!
//! ## Usage Examples
//!
//! ### Basic File Operations
//!
//! ```no_run
//! use std::sync::Arc;
//! use rust_helia::create_helia_default;
//! use helia_unixfs::{UnixFS, UnixFSInterface, AddOptions};
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Helia node
//!     let helia = create_helia_default().await?;
//!     let fs = UnixFS::new(Arc::new(helia));
//!     
//!     // Add a small file
//!     let data = Bytes::from("Hello, IPFS!");
//!     let cid = fs.add_bytes(data, None).await?;
//!     println!("File CID: {}", cid);
//!     
//!     // Read the file back
//!     let content = fs.cat(&cid, None).await?;
//!     println!("Content: {:?}", content);
//!     
//!     // Add with options
//!     let data2 = Bytes::from("Important data");
//!     let cid2 = fs.add_bytes(data2, Some(AddOptions {
//!         pin: true,  // Pin for persistence
//!         raw_leaves: true,  // Use raw blocks for leaves
//!         ..Default::default()
//!     })).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### File with Metadata
//!
//! ```no_run
//! # use helia_unixfs::{FileCandidate, UnixFSTime, AddOptions};
//! # use bytes::Bytes;
//! # async fn example(fs: impl helia_unixfs::UnixFSInterface) -> Result<(), Box<dyn std::error::Error>> {
//! // Create file with Unix permissions and timestamp
//! let file = FileCandidate {
//!     path: "document.txt".to_string(),
//!     content: Bytes::from("Important document"),
//!     mode: Some(0o644),  // rw-r--r--
//!     mtime: Some(UnixFSTime::now()),
//! };
//!
//! let cid = fs.add_file(file, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Directory Operations
//!
//! ```no_run
//! # async fn example(fs: impl helia_unixfs::UnixFSInterface) -> Result<(), Box<dyn std::error::Error>> {
//! # use bytes::Bytes;
//! // Create an empty directory
//! let dir_cid = fs.add_directory(None, None).await?;
//!
//! // Add a file to the directory
//! let file_data = Bytes::from("README content");
//! let file_cid = fs.add_bytes(file_data, None).await?;
//! let updated_dir = fs.cp(&file_cid, &dir_cid, "README.md", None).await?;
//!
//! // Create a subdirectory
//! use helia_unixfs::MkdirOptions;
//! let dir_with_subdir = fs.mkdir(
//!     &updated_dir,
//!     "docs",
//!     Some(MkdirOptions {
//!         mode: Some(0o755),  // rwxr-xr-x
//!         ..Default::default()
//!     })
//! ).await?;
//!
//! // List directory contents
//! let entries = fs.ls(&dir_with_subdir, None).await?;
//! // Iterate through entries...
//! # Ok(())
//! # }
//! ```
//!
//! ### Large File Handling
//!
//! ```no_run
//! # async fn example(fs: impl helia_unixfs::UnixFSInterface) -> Result<(), Box<dyn std::error::Error>> {
//! # use bytes::Bytes;
//! # use helia_unixfs::AddOptions;
//! // Large files are automatically chunked
//! let large_data = Bytes::from(vec![0u8; 5_000_000]); // 5MB
//!
//! let cid = fs.add_bytes(large_data, Some(AddOptions {
//!     chunk_size: Some(524_288), // 512KB chunks
//!     ..Default::default()
//! })).await?;
//!
//! // Read with offset and length for efficient partial reads
//! use helia_unixfs::CatOptions;
//! let partial = fs.cat(&cid, Some(CatOptions {
//!     offset: Some(1_000_000),  // Start at 1MB
//!     length: Some(100_000),     // Read 100KB
//! })).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Working with Statistics
//!
//! ```no_run
//! # use cid::Cid;
//! # async fn example(fs: impl helia_unixfs::UnixFSInterface, cid: &Cid) -> Result<(), Box<dyn std::error::Error>> {
//! use helia_unixfs::{UnixFSStat, FileStat, DirectoryStat};
//!
//! let stats = fs.stat(cid, None).await?;
//!
//! match stats {
//!     UnixFSStat::File(file_stats) => {
//!         println!("File size: {} bytes", file_stats.size);
//!         println!("Blocks: {}", file_stats.blocks);
//!         if let Some(mode) = file_stats.mode {
//!             println!("Mode: {:o}", mode);
//!         }
//!     }
//!     UnixFSStat::Directory(dir_stats) => {
//!         println!("Directory with {} entries", dir_stats.entries);
//!         println!("Total size: {} bytes", dir_stats.size);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! ### File Size Guidelines
//! - **< 256KB**: Single block, fast add/retrieve
//! - **256KB - 1MB**: Single block with DAG-PB wrapper
//! - **> 1MB**: Automatically chunked into 256KB blocks
//! - **Very large (>100MB)**: Efficient streaming with balanced Merkle tree
//!
//! ### Memory Usage
//! - **Small files**: Loaded entirely into memory
//! - **Large files**: Chunked streaming, constant memory usage
//! - **Directories**: Efficient lazy evaluation of entries
//!
//! ### Operation Complexity
//! - **add_bytes()**: O(n) where n = file size
//! - **cat()**: O(n) where n = bytes read
//! - **ls()**: O(m) where m = number of entries
//! - **cp()**: O(m) where m = directory size
//! - **stat()**: O(1) - constant time
//!
//! ## Thread Safety
//!
//! All UnixFS operations are thread-safe:
//! - Uses `Arc<dyn Helia>` for shared access
//! - All methods use `&self` (immutable borrow)
//! - Safe to share `UnixFS` instance across threads
//! - Concurrent operations are supported
//!
//! ```no_run
//! # use std::sync::Arc;
//! # use rust_helia::create_helia_default;
//! # use helia_unixfs::UnixFS;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let helia = create_helia_default().await?;
//! let fs = Arc::new(UnixFS::new(Arc::new(helia)));
//!
//! // Clone and use in multiple tasks
//! let fs1 = Arc::clone(&fs);
//! let fs2 = Arc::clone(&fs);
//!
//! tokio::spawn(async move {
//!     // Use fs1 in this task
//! });
//!
//! tokio::spawn(async move {
//!     // Use fs2 in this task
//! });
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All operations return `Result<T, UnixFSError>`:
//!
//! ```no_run
//! # use helia_unixfs::{UnixFS, UnixFSInterface, UnixFSError};
//! # async fn example(fs: impl UnixFSInterface, cid: &cid::Cid) -> Result<(), Box<dyn std::error::Error>> {
//! match fs.cat(cid, None).await {
//!     Ok(data) => println!("Read {} bytes", data.len()),
//!     Err(UnixFSError::NotAFile { cid }) => {
//!         println!("Not a file: {}", cid);
//!     }
//!     Err(UnixFSError::NotUnixFS { cid }) => {
//!         println!("Not a UnixFS node: {}", cid);
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Limitations
//!
//! ### Current Limitations
//! - **Symlinks**: Not yet implemented (returns error)
//! - **HAMTs**: Large directories (>10,000 entries) not optimized
//! - **Inline CIDs**: Very small files not inlined in parent blocks
//! - **Trickle DAG**: Only uses balanced DAG structure
//!
//! ### Future Enhancements
//! - Support for UnixFS v2 features
//! - HAMT-sharded directories for very large directories
//! - Trickle DAG option for better streaming
//! - More compression options
//!
//! ## Compatibility
//!
//! This implementation is compatible with:
//! - **go-ipfs/Kubo**: Full compatibility with standard IPFS nodes
//! - **js-ipfs**: Compatible with JavaScript IPFS implementations
//! - **@helia/unixfs**: Compatible with TypeScript Helia implementation
//!
//! ## Examples Directory
//!
//! See the `examples/` directory for more usage examples:
//! - `01_simple_file.rs` - Basic file operations
//! - `02_large_file.rs` - Chunked file handling
//! - `03_directories.rs` - Directory operations
//! - `04_metadata.rs` - Working with permissions and times

pub mod chunker;
pub mod dag_pb;
pub mod errors;
mod pb;
pub mod unixfs;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use serde::{Deserialize, Serialize};

use helia_interface::{AwaitIterable, Helia};

pub use chunker::*;
pub use dag_pb::*;
pub use errors::*;
pub use pb::*;
pub use unixfs::*;

/// File statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileStat {
    pub cid: Cid,
    pub size: u64,
    pub blocks: u64,
    pub type_: UnixFSType,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
}

/// Directory statistics  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryStat {
    pub cid: Cid,
    pub size: u64,
    pub blocks: u64,
    pub type_: UnixFSType,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
    pub entries: u64,
}

/// UnixFS entry types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnixFSType {
    File,
    Directory,
    Symlink,
    Raw,
}

/// UnixFS timestamp
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnixFSTime {
    pub seconds: u64,
    pub nanoseconds: Option<u32>,
}

impl UnixFSTime {
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            seconds: now.as_secs(),
            nanoseconds: Some(now.subsec_nanos()),
        }
    }
}

/// UnixFS directory entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnixFSEntry {
    pub name: String,
    pub cid: Cid,
    pub size: u64,
    pub type_: UnixFSType,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
}

/// File candidate for adding to UnixFS
#[derive(Debug, Clone)]
pub struct FileCandidate {
    pub path: String,
    pub content: Bytes,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
}

/// Directory candidate for adding to UnixFS
#[derive(Debug, Clone)]
pub struct DirectoryCandidate {
    pub path: String,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
}

/// Options for adding content
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    pub pin: bool,
    pub chunk_size: Option<usize>,
    pub raw_leaves: bool,
    pub wrap_with_directory: bool,
}

/// Options for reading content
#[derive(Debug, Clone, Default)]
pub struct CatOptions {
    pub offset: Option<u64>,
    pub length: Option<u64>,
}

/// Options for listing directory contents
#[derive(Debug, Clone, Default)]
pub struct LsOptions {
    pub recursive: bool,
}

/// Options for copying content
#[derive(Debug, Clone, Default)]
pub struct CpOptions {
    pub create_path: bool,
}

/// Options for making directories
#[derive(Debug, Clone, Default)]
pub struct MkdirOptions {
    pub parents: bool,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
}

/// Options for removing content
#[derive(Debug, Clone, Default)]
pub struct RmOptions {
    pub recursive: bool,
}

/// Options for file/directory statistics
#[derive(Debug, Clone, Default)]
pub struct StatOptions {
    pub with_local: bool,
}

/// Main UnixFS interface trait
#[async_trait]
pub trait UnixFSInterface: Send + Sync {
    /// Add bytes as a file
    async fn add_bytes(
        &self,
        bytes: Bytes,
        options: Option<AddOptions>,
    ) -> Result<Cid, UnixFSError>;

    /// Add a file candidate
    async fn add_file(
        &self,
        file: FileCandidate,
        options: Option<AddOptions>,
    ) -> Result<Cid, UnixFSError>;

    /// Add a directory
    async fn add_directory(
        &self,
        dir: Option<DirectoryCandidate>,
        options: Option<AddOptions>,
    ) -> Result<Cid, UnixFSError>;

    /// Read file content
    async fn cat(&self, cid: &Cid, options: Option<CatOptions>) -> Result<Bytes, UnixFSError>;

    /// Copy content to a directory
    async fn cp(
        &self,
        source: &Cid,
        target: &Cid,
        name: &str,
        options: Option<CpOptions>,
    ) -> Result<Cid, UnixFSError>;

    /// List directory contents
    async fn ls(
        &self,
        cid: &Cid,
        options: Option<LsOptions>,
    ) -> Result<AwaitIterable<UnixFSEntry>, UnixFSError>;

    /// Create a directory in an existing directory
    async fn mkdir(
        &self,
        cid: &Cid,
        dirname: &str,
        options: Option<MkdirOptions>,
    ) -> Result<Cid, UnixFSError>;

    /// Remove content from a directory
    async fn rm(
        &self,
        cid: &Cid,
        path: &str,
        options: Option<RmOptions>,
    ) -> Result<Cid, UnixFSError>;

    /// Get file or directory statistics
    async fn stat(
        &self,
        cid: &Cid,
        options: Option<StatOptions>,
    ) -> Result<UnixFSStat, UnixFSError>;
}

/// Union type for file and directory statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnixFSStat {
    File(FileStat),
    Directory(DirectoryStat),
}

/// Create a UnixFS instance from a Helia node
pub fn create_unixfs(helia: Arc<dyn Helia>) -> impl UnixFSInterface {
    UnixFS::new(helia)
}
