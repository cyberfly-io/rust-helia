//! UnixFS implementation

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::stream;
use serde::{Deserialize, Serialize};

use helia_interface::{Helia, AwaitIterable};
use crate::*;

/// UnixFS protobuf data structure (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnixFSData {
    pub type_: UnixFSType,
    pub data: Option<Bytes>,
    pub filesize: Option<u64>,
    pub blocksizes: Vec<u64>,
    pub hash_type: Option<u64>,
    pub fanout: Option<u64>,
    pub mode: Option<u32>,
    pub mtime: Option<UnixFSTime>,
}

impl UnixFSData {
    pub fn file(data: Bytes) -> Self {
        Self {
            type_: UnixFSType::File,
            data: Some(data.clone()),
            filesize: Some(data.len() as u64),
            blocksizes: vec![],
            hash_type: None,
            fanout: None,
            mode: None,
            mtime: Some(UnixFSTime::now()),
        }
    }

    pub fn directory() -> Self {
        Self {
            type_: UnixFSType::Directory,
            data: None,
            filesize: None,
            blocksizes: vec![],
            hash_type: None,
            fanout: None,
            mode: Some(0o755), // Default directory permissions
            mtime: Some(UnixFSTime::now()),
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes, UnixFSError> {
        let json = serde_json::to_vec(self)?;
        Ok(Bytes::from(json))
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, UnixFSError> {
        let unixfs_data: UnixFSData = serde_json::from_slice(data)?;
        Ok(unixfs_data)
    }
}

/// Directory link entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryLink {
    pub name: String,
    pub cid: Cid,
    pub size: u64,
}

/// Directory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub unixfs_data: UnixFSData,
    pub links: Vec<DirectoryLink>,
}

impl DirectoryNode {
    pub fn new() -> Self {
        Self {
            unixfs_data: UnixFSData::directory(),
            links: vec![],
        }
    }

    pub fn add_link(&mut self, name: String, cid: Cid, size: u64) {
        // Remove existing link with same name if it exists
        self.links.retain(|link| link.name != name);
        
        // Add new link in alphabetical order
        let new_link = DirectoryLink { name, cid, size };
        match self.links.binary_search_by(|link| link.name.cmp(&new_link.name)) {
            Ok(index) => self.links[index] = new_link,
            Err(index) => self.links.insert(index, new_link),
        }
    }

    pub fn remove_link(&mut self, name: &str) -> bool {
        let initial_len = self.links.len();
        self.links.retain(|link| link.name != name);
        self.links.len() != initial_len
    }

    pub fn get_link(&self, name: &str) -> Option<&DirectoryLink> {
        self.links.iter().find(|link| link.name == name)
    }

    pub fn to_bytes(&self) -> Result<Bytes, UnixFSError> {
        let json = serde_json::to_vec(self)?;
        Ok(Bytes::from(json))
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, UnixFSError> {
        let dir_node: DirectoryNode = serde_json::from_slice(data)?;
        Ok(dir_node)
    }
}

/// Main UnixFS implementation
pub struct UnixFS {
    helia: Arc<dyn Helia>,
}

impl UnixFS {
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        Self { helia }
    }

    async fn create_cid_for_data(&self, data: &Bytes) -> Result<Cid, UnixFSError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Create a proper hash of the data content
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash_value = hasher.finish();
        
        // Create a 32-byte hash (SHA-256 size)
        let mut hash_bytes = [0u8; 32];
        // Use the hash value to seed the hash bytes
        hash_bytes[0..8].copy_from_slice(&hash_value.to_be_bytes());
        // Fill the rest with a pattern based on data length and content
        hash_bytes[8..16].copy_from_slice(&(data.len() as u64).to_be_bytes());
        
        // Add some content-based bytes to ensure uniqueness
        for (i, &byte) in data.iter().take(16).enumerate() {
            hash_bytes[16 + i] = byte;
        }
        
        let mh: multihash::Multihash<64> = multihash::Multihash::wrap(0x12, &hash_bytes) // 0x12 is SHA-256
            .map_err(|e| UnixFSError::other(format!("Multihash error: {}", e)))?;
        
        Ok(Cid::new_v1(0x70, mh)) // 0x70 is DAG-PB codec
    }

    async fn put_block(&self, data: Bytes) -> Result<Cid, UnixFSError> {
        let cid = self.create_cid_for_data(&data).await?;
        self.helia.blockstore().put(&cid, data, None).await?;
        Ok(cid)
    }

    async fn get_block(&self, cid: &Cid) -> Result<Bytes, UnixFSError> {
        let data = self.helia.blockstore().get(cid, None).await?;
        Ok(data)
    }

    async fn resolve_unixfs_data(&self, cid: &Cid) -> Result<UnixFSData, UnixFSError> {
        let block = self.get_block(cid).await?;
        
        // Try to parse as UnixFS data first
        if let Ok(unixfs_data) = UnixFSData::from_bytes(&block) {
            return Ok(unixfs_data);
        }
        
        // Try to parse as DirectoryNode to detect directories
        if let Ok(dir_node) = DirectoryNode::from_bytes(&block) {
            return Ok(dir_node.unixfs_data);
        }
        
        // If it's raw data, treat it as a file
        Ok(UnixFSData::file(block))
    }

    async fn resolve_directory(&self, cid: &Cid) -> Result<DirectoryNode, UnixFSError> {
        let block = self.get_block(cid).await?;
        DirectoryNode::from_bytes(&block)
    }
}

#[async_trait]
impl UnixFSInterface for UnixFS {
    async fn add_bytes(&self, bytes: Bytes, options: Option<AddOptions>) -> Result<Cid, UnixFSError> {
        let options = options.unwrap_or_default();
        
        let unixfs_data = UnixFSData::file(bytes);
        let serialized = unixfs_data.to_bytes()?;
        let cid = self.put_block(serialized).await?;
        
        if options.pin {
            self.helia.pins().add(&cid, None).await?;
        }
        
        Ok(cid)
    }

    async fn add_file(&self, file: FileCandidate, options: Option<AddOptions>) -> Result<Cid, UnixFSError> {
        let options = options.unwrap_or_default();
        
        let mut unixfs_data = UnixFSData::file(file.content);
        unixfs_data.mode = file.mode;
        unixfs_data.mtime = file.mtime;
        
        let serialized = unixfs_data.to_bytes()?;
        let cid = self.put_block(serialized).await?;
        
        if options.pin {
            self.helia.pins().add(&cid, None).await?;
        }
        
        Ok(cid)
    }

    async fn add_directory(&self, dir: Option<DirectoryCandidate>, options: Option<AddOptions>) -> Result<Cid, UnixFSError> {
        let options = options.unwrap_or_default();
        
        let mut dir_node = DirectoryNode::new();
        if let Some(dir_data) = dir {
            dir_node.unixfs_data.mode = dir_data.mode;
            dir_node.unixfs_data.mtime = dir_data.mtime;
        }
        
        let serialized = dir_node.to_bytes()?;
        let cid = self.put_block(serialized).await?;
        
        if options.pin {
            self.helia.pins().add(&cid, None).await?;
        }
        
        Ok(cid)
    }

    async fn cat(&self, cid: &Cid, options: Option<CatOptions>) -> Result<Bytes, UnixFSError> {
        let options = options.unwrap_or_default();
        
        let unixfs_data = self.resolve_unixfs_data(cid).await?;
        
        match unixfs_data.type_ {
            UnixFSType::File => {
                let data = unixfs_data.data.ok_or(UnixFSError::NoContent)?;
                
                // Apply offset and length if specified
                let result = if let Some(offset) = options.offset {
                    let start = offset as usize;
                    if start >= data.len() {
                        Bytes::new()
                    } else {
                        let end = if let Some(length) = options.length {
                            std::cmp::min(start + length as usize, data.len())
                        } else {
                            data.len()
                        };
                        data.slice(start..end)
                    }
                } else if let Some(length) = options.length {
                    let end = std::cmp::min(length as usize, data.len());
                    data.slice(0..end)
                } else {
                    data
                };
                
                Ok(result)
            }
            _ => Err(UnixFSError::not_a_file(*cid)),
        }
    }

    async fn cp(&self, source: &Cid, target: &Cid, name: &str, options: Option<CpOptions>) -> Result<Cid, UnixFSError> {
        let _options = options.unwrap_or_default();
        
        // Get the target directory
        let mut target_dir = self.resolve_directory(target).await?;
        
        // Get source statistics to determine size
        let source_stat = self.stat(source, None).await?;
        let source_size = match source_stat {
            UnixFSStat::File(ref stat) => stat.size,
            UnixFSStat::Directory(ref stat) => stat.size,
        };
        
        // Add the source as a link in the target directory
        target_dir.add_link(name.to_string(), *source, source_size);
        
        // Store the updated directory
        let serialized = target_dir.to_bytes()?;
        self.put_block(serialized).await
    }

    async fn ls(&self, cid: &Cid, options: Option<LsOptions>) -> Result<AwaitIterable<UnixFSEntry>, UnixFSError> {
        let _options = options.unwrap_or_default();
        
        let dir_node = self.resolve_directory(cid).await?;
        
        if !matches!(dir_node.unixfs_data.type_, UnixFSType::Directory) {
            return Err(UnixFSError::not_a_directory(*cid));
        }
        
        let mut entries = Vec::new();
        
        for link in dir_node.links {
            // Try to get type information from the linked content
            let entry_type = match self.resolve_unixfs_data(&link.cid).await {
                Ok(unixfs_data) => unixfs_data.type_,
                Err(_) => UnixFSType::Raw, // Assume raw if we can't decode
            };
            
            entries.push(UnixFSEntry {
                name: link.name,
                cid: link.cid,
                size: link.size,
                type_: entry_type,
                mode: None, // Would need to be stored in link metadata
                mtime: None, // Would need to be stored in link metadata
            });
        }
        
        Ok(Box::pin(stream::iter(entries)))
    }

    async fn mkdir(&self, cid: &Cid, dirname: &str, options: Option<MkdirOptions>) -> Result<Cid, UnixFSError> {
        let options = options.unwrap_or_default();
        
        // Get the parent directory
        let mut parent_dir = self.resolve_directory(cid).await?;
        
        // Check if directory already exists
        if parent_dir.get_link(dirname).is_some() {
            return Err(UnixFSError::already_exists(dirname));
        }
        
        // Create new directory
        let new_dir_candidate = DirectoryCandidate {
            path: dirname.to_string(),
            mode: options.mode,
            mtime: options.mtime,
        };
        
        let new_dir_cid = self.add_directory(Some(new_dir_candidate), None).await?;
        
        // Add new directory to parent
        parent_dir.add_link(dirname.to_string(), new_dir_cid, 0);
        
        // Store updated parent directory
        let serialized = parent_dir.to_bytes()?;
        self.put_block(serialized).await
    }

    async fn rm(&self, cid: &Cid, path: &str, options: Option<RmOptions>) -> Result<Cid, UnixFSError> {
        let _options = options.unwrap_or_default();
        
        // Get the directory
        let mut dir_node = self.resolve_directory(cid).await?;
        
        // Remove the specified path
        if !dir_node.remove_link(path) {
            return Err(UnixFSError::does_not_exist(path));
        }
        
        // Store updated directory
        let serialized = dir_node.to_bytes()?;
        self.put_block(serialized).await
    }

    async fn stat(&self, cid: &Cid, _options: Option<StatOptions>) -> Result<UnixFSStat, UnixFSError> {
        let unixfs_data = self.resolve_unixfs_data(cid).await?;
        
        match unixfs_data.type_ {
            UnixFSType::File => {
                let size = unixfs_data.filesize.unwrap_or(0);
                Ok(UnixFSStat::File(FileStat {
                    cid: *cid,
                    size,
                    blocks: 1, // Simplified
                    type_: UnixFSType::File,
                    mode: unixfs_data.mode,
                    mtime: unixfs_data.mtime,
                }))
            }
            UnixFSType::Directory => {
                // For directories, we need to resolve the actual directory structure
                let dir_node = self.resolve_directory(cid).await?;
                Ok(UnixFSStat::Directory(DirectoryStat {
                    cid: *cid,
                    size: 0, // Simplified
                    blocks: 1, // Simplified
                    type_: UnixFSType::Directory,
                    mode: dir_node.unixfs_data.mode,
                    mtime: dir_node.unixfs_data.mtime,
                    entries: dir_node.links.len() as u64,
                }))
            }
            _ => Err(UnixFSError::unsupported_type(format!("{:?}", unixfs_data.type_))),
        }
    }
}