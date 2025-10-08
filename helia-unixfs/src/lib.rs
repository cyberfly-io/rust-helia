//! # Helia UnixFS
//!
//! UnixFS filesystem implementation for Helia, providing file and directory operations
//! compatible with IPFS UnixFS specification.
//!
//! ## Example
//!
//! ```rust
//! use std::sync::Arc;
//! use helia::create_helia;
//! use helia_unixfs::{UnixFS, UnixFSInterface};
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia(None).await?;
//!     let fs = UnixFS::new(Arc::new(helia));
//!     
//!     // Add a file
//!     let data = Bytes::from("hello world");
//!     let file_cid = fs.add_bytes(data, None).await?;
//!     
//!     // Read the file back
//!     let retrieved_data = fs.cat(&file_cid, None).await?;
//!     
//!     // Create a directory and add the file to it
//!     let dir_cid = fs.add_directory(None, None).await?;
//!     let updated_dir_cid = fs.cp(&file_cid, &dir_cid, "hello.txt", None).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod errors;
pub mod unixfs;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use serde::{Deserialize, Serialize};

use helia_interface::{Helia, AwaitIterable};

pub use errors::*;
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
    async fn add_bytes(&self, bytes: Bytes, options: Option<AddOptions>) -> Result<Cid, UnixFSError>;
    
    /// Add a file candidate
    async fn add_file(&self, file: FileCandidate, options: Option<AddOptions>) -> Result<Cid, UnixFSError>;
    
    /// Add a directory
    async fn add_directory(&self, dir: Option<DirectoryCandidate>, options: Option<AddOptions>) -> Result<Cid, UnixFSError>;
    
    /// Read file content
    async fn cat(&self, cid: &Cid, options: Option<CatOptions>) -> Result<Bytes, UnixFSError>;
    
    /// Copy content to a directory
    async fn cp(&self, source: &Cid, target: &Cid, name: &str, options: Option<CpOptions>) -> Result<Cid, UnixFSError>;
    
    /// List directory contents
    async fn ls(&self, cid: &Cid, options: Option<LsOptions>) -> Result<AwaitIterable<UnixFSEntry>, UnixFSError>;
    
    /// Create a directory in an existing directory
    async fn mkdir(&self, cid: &Cid, dirname: &str, options: Option<MkdirOptions>) -> Result<Cid, UnixFSError>;
    
    /// Remove content from a directory
    async fn rm(&self, cid: &Cid, path: &str, options: Option<RmOptions>) -> Result<Cid, UnixFSError>;
    
    /// Get file or directory statistics
    async fn stat(&self, cid: &Cid, options: Option<StatOptions>) -> Result<UnixFSStat, UnixFSError>;
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
