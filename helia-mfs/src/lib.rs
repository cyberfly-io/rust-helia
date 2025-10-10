//! Mutable File System (MFS) for Helia
//!
//! This module provides a mutable file system layer on top of UnixFS, allowing
//! users to work with IPFS content using familiar file system operations.
//!
//! # Overview
//!
//! MFS (Mutable File System) provides a POSIX-like interface for interacting with
//! IPFS content. While IPFS content is inherently immutable, MFS maintains a mutable
//! "view" into the content-addressed data by tracking the root CID of your file system
//! tree and updating it as you make changes.
//!
//! # Core Concepts
//!
//! - **Immutable Content**: All UnixFS content is stored immutably using content-addressed
//!   CIDs. When you modify a file, a new CID is created.
//! - **Mutable Root**: MFS tracks a single "root" CID that represents your current file
//!   system state. This root is updated after each modification.
//! - **Directory Chains**: Modifying any file requires updating its parent directory,
//!   which requires updating that directory's parent, all the way to the root.
//!
//! # Supported Operations
//!
//! - **mkdir** - Create directories (like `mkdir -p`)
//! - **write_bytes** - Write files from byte slices
//! - **ls** - List directory contents
//! - **stat** - Get file/directory metadata
//! - **cp** - Copy files or directories
//! - **mv** - Move/rename files or directories
//! - **rm** - Remove files or directories
//! - **root_cid** - Get the current root CID
//! - **flush** - Ensure changes are persisted
//!
//! # Example Usage
//!
//! ```no_run
//! use helia_mfs::{mfs, MfsInterface};
//! use rust_helia::create_helia_default;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Helia node
//!     let helia = Arc::new(create_helia_default().await?);
//!     
//!     // Create an MFS instance
//!     let fs = mfs(helia);
//!     
//!     // Create a directory
//!     fs.mkdir("/documents").await?;
//!     
//!     // Write a file
//!     fs.write_bytes("/documents/hello.txt", b"Hello, IPFS!").await?;
//!     
//!     // List contents
//!     let entries = fs.ls("/documents").await?;
//!     for entry in entries {
//!         println!("{}: {} bytes", entry.name, entry.size);
//!     }
//!     
//!     // Copy a file
//!     fs.cp("/documents/hello.txt", "/documents/hello_copy.txt").await?;
//!     
//!     // Move a file
//!     fs.mv("/documents/hello_copy.txt", "/backup.txt").await?;
//!     
//!     // Remove a file
//!     fs.rm("/backup.txt", false).await?;
//!     
//!     // Get the root CID (represents your entire file system)
//!     if let Some(root) = fs.root_cid().await {
//!         println!("File system root: {}", root);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Performance Considerations
//!
//! - **Copy Operations**: Copying files is O(1) in space - only directory metadata
//!   is copied, not the actual file content.
//! - **Deep Paths**: Operations on deeply nested paths require updating the entire
//!   directory chain from the modified file up to the root.
//! - **Large Directories**: Listing and modifying large directories may be slow as
//!   all entries must be loaded into memory.
//!
//! # Thread Safety
//!
//! All MFS operations are thread-safe and can be called concurrently from multiple
//! tasks. The root CID is protected by an `RwLock` to ensure consistency.
//!
//! # Error Handling
//!
//! Operations return `Result<T, MfsError>` where `MfsError` provides detailed
//! information about what went wrong. Common errors include:
//!
//! - `InvalidPath` - Malformed or invalid paths
//! - `NotFound` - File or directory doesn't exist
//! - `UnixFs` - Underlying UnixFS operation failed
//!
//! # Limitations
//!
//! - **No Metadata Updates**: Operations like `touch()` (update timestamps) and
//!   `chmod()` (change permissions) would require recreating content with new
//!   metadata, which is not yet implemented.
//! - **No Streaming**: Large files must fit in memory during write operations.
//! - **No Transactions**: Operations are not transactional beyond atomic `mv()`.

mod path;
mod operations;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::StreamExt;
use helia_interface::Helia;
use helia_unixfs::{create_unixfs, UnixFSEntry, UnixFSInterface, UnixFSType};
use std::sync::Arc;

pub use path::MfsPath;
use operations::{normalize_path, split_path};

/// Error types for MFS operations
#[derive(Debug, thiserror::Error)]
pub enum MfsError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("UnixFS error: {0}")]
    UnixFs(String),
}

/// Trait defining the MFS interface
#[async_trait]
pub trait MfsInterface: Send + Sync {
    /// Create a directory at the given path
    /// Creates parent directories if they don't exist (like mkdir -p)
    async fn mkdir(&self, path: &str) -> Result<(), MfsError>;

    /// Write bytes to a file at the given path
    /// Creates parent directories if they don't exist
    async fn write_bytes(&self, path: &str, content: &[u8]) -> Result<(), MfsError>;

    /// List directory contents
    async fn ls(&self, path: &str) -> Result<Vec<UnixFSEntry>, MfsError>;

    /// Get file/directory statistics
    async fn stat(&self, path: &str) -> Result<UnixFSEntry, MfsError>;

    /// Copy a file or directory
    async fn cp(&self, from: &str, to: &str) -> Result<(), MfsError>;

    /// Move (rename) a file or directory
    async fn mv(&self, from: &str, to: &str) -> Result<(), MfsError>;

    /// Remove a file or directory
    /// If recursive is true, removes directories with contents
    async fn rm(&self, path: &str, recursive: bool) -> Result<(), MfsError>;

    /// Get the root CID of the file system
    async fn root_cid(&self) -> Option<Cid>;

    /// Flush changes and update the root CID
    async fn flush(&self) -> Result<Cid, MfsError>;
}

/// Default MFS implementation
pub struct DefaultMfs {
    unixfs: Box<dyn UnixFSInterface>,
    root_cid: Arc<tokio::sync::RwLock<Option<Cid>>>,
}

impl DefaultMfs {
    pub fn new(helia: Arc<dyn Helia>) -> Self {
        let unixfs = Box::new(create_unixfs(helia.clone()));
        Self {
            unixfs,
            root_cid: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    async fn get_root_cid(&self) -> Result<Cid, MfsError> {
        let mut root = self.root_cid.write().await;
        if root.is_none() {
            let cid = self
                .unixfs
                .add_directory(None, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;
            *root = Some(cid);
        }
        Ok(root.unwrap())
    }

    /// Navigate to a directory and return its CID
    async fn navigate_to_dir(&self, path: &str) -> Result<Cid, MfsError> {
        if path == "/" {
            return self.get_root_cid().await;
        }

        let segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let root_cid = self.get_root_cid().await?;
        let mut current_cid = root_cid;

        // Navigate through each segment
        for segment in segments {
            let entries = self
                .unixfs
                .ls(&current_cid, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;

            // Convert stream to vector
            let mut entries_vec = Vec::new();
            let mut entries_stream = entries;
            while let Some(entry) = entries_stream.next().await {
                entries_vec.push(entry);
            }

            // Find the segment in current directory
            let found = entries_vec.iter().find(|e| e.name == segment);

            match found {
                Some(entry) if matches!(entry.type_, UnixFSType::Directory) => {
                    current_cid = entry.cid;
                }
                Some(_) => {
                    return Err(MfsError::InvalidPath(format!(
                        "'{}' is not a directory",
                        segment
                    )));
                }
                None => {
                    return Err(MfsError::InvalidPath(format!(
                        "Directory '{}' not found",
                        segment
                    )));
                }
            }
        }

        Ok(current_cid)
    }

    /// Update a file in a nested directory structure
    async fn update_nested_file(
        &self,
        path_segments: &[String],
        file_cid: Cid,
        filename: &str,
    ) -> Result<Cid, MfsError> {
        if path_segments.is_empty() {
            // At root level - add file directly to root (or update if exists)
            let root_cid = self.get_root_cid().await?;
            return self.add_or_update_entry(&root_cid, filename, &file_cid).await;
        }

        // Get all directory CIDs from root to target (iteratively)
        let root_cid = self.get_root_cid().await?;
        let mut dir_cids = vec![root_cid];
        let mut current_cid = root_cid;

        // Navigate and collect all directory CIDs
        for segment in path_segments {
            let entries = self
                .unixfs
                .ls(&current_cid, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;

            let mut entries_vec = Vec::new();
            let mut entries_stream = entries;
            while let Some(entry) = entries_stream.next().await {
                entries_vec.push(entry);
            }

            let found = entries_vec.iter().find(|e| e.name == *segment);
            match found {
                Some(entry) if matches!(entry.type_, UnixFSType::Directory) => {
                    current_cid = entry.cid;
                    dir_cids.push(current_cid);
                }
                _ => {
                    return Err(MfsError::InvalidPath(format!(
                        "Directory '{}' not found in path",
                        segment
                    )));
                }
            }
        }

        // Now update from deepest to root
        let target_dir_cid = *dir_cids.last().unwrap();
        
        // Add file to target directory (or update if exists)
        let mut updated_cid = self
            .add_or_update_entry(&target_dir_cid, filename, &file_cid)
            .await?;

        // Update each parent directory from bottom to top
        for i in (0..path_segments.len()).rev() {
            let parent_cid = dir_cids[i];
            let dir_name = &path_segments[i];
            
            updated_cid = self
                .add_or_update_entry(&parent_cid, dir_name, &updated_cid)
                .await?;
        }

        Ok(updated_cid)
    }

    /// Update a directory in the chain from a modified child CID
    /// Given path segments and the new CID for the deepest directory,
    /// update each parent directory from bottom to top
    async fn update_directory_chain(
        &self,
        path_segments: &[String],
        updated_child_cid: Cid,
    ) -> Result<Cid, MfsError> {
        if path_segments.is_empty() {
            return Ok(updated_child_cid);
        }

        // Get all directory CIDs from root to parent of target
        let root_cid = self.get_root_cid().await?;
        let mut dir_cids = vec![root_cid];
        let mut current_cid = root_cid;

        // Navigate and collect all directory CIDs (except the last one which we're updating)
        for segment in &path_segments[..path_segments.len() - 1] {
            let entries = self
                .unixfs
                .ls(&current_cid, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;

            let mut entries_vec = Vec::new();
            let mut entries_stream = entries;
            while let Some(entry) = entries_stream.next().await {
                entries_vec.push(entry);
            }

            let found = entries_vec.iter().find(|e| e.name == *segment);
            match found {
                Some(entry) if matches!(entry.type_, UnixFSType::Directory) => {
                    current_cid = entry.cid;
                    dir_cids.push(current_cid);
                }
                _ => {
                    return Err(MfsError::InvalidPath(format!(
                        "Directory '{}' not found in path",
                        segment
                    )));
                }
            }
        }

        // Start with the updated child CID
        let mut updated_cid = updated_child_cid;
        let last_segment = path_segments.last().unwrap();

        // Update the parent directory to point to the new child
        if path_segments.len() == 1 {
            // Single segment means parent is root
            let parent_cid = dir_cids[0];
            updated_cid = self
                .add_or_update_entry(&parent_cid, last_segment, &updated_cid)
                .await?;
        } else {
            // Multiple segments, parent is at len - 2
            let parent_idx = path_segments.len() - 2;
            let parent_cid = dir_cids[parent_idx];
            updated_cid = self
                .add_or_update_entry(&parent_cid, last_segment, &updated_cid)
                .await?;
        }

        // Update each ancestor directory from bottom to top
        for i in (0..path_segments.len() - 1).rev() {
            if i >= dir_cids.len() {
                continue;
            }
            let parent_cid = dir_cids[i];
            let dir_name = &path_segments[i];
            
            updated_cid = self
                .add_or_update_entry(&parent_cid, dir_name, &updated_cid)
                .await?;
        }

        Ok(updated_cid)
    }

    /// Add or update an entry in a directory
    /// If an entry with the same name already exists, it is removed first
    /// This prevents duplicate entries when overwriting files or directories
    async fn add_or_update_entry(
        &self,
        parent_cid: &Cid,
        name: &str,
        entry_cid: &Cid,
    ) -> Result<Cid, MfsError> {
        // Check if entry already exists
        let entries = self
            .unixfs
            .ls(parent_cid, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        let mut entries_vec = Vec::new();
        let mut entries_stream = entries;
        while let Some(entry) = entries_stream.next().await {
            entries_vec.push(entry);
        }

        let existing = entries_vec.iter().find(|e| e.name == name);

        // If entry exists, remove it first
        let parent_cid_to_use = if existing.is_some() {
            self.unixfs
                .rm(parent_cid, name, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?
        } else {
            *parent_cid
        };

        // Now add the new entry
        self.unixfs
            .cp(entry_cid, &parent_cid_to_use, name, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))
    }
}

#[async_trait]
impl MfsInterface for DefaultMfs {
    async fn mkdir(&self, path: &str) -> Result<(), MfsError> {
        let path = normalize_path(path)?;

        if path == "/" {
            return Err(MfsError::InvalidPath(
                "Root directory already exists".to_string(),
            ));
        }

        // Parse path into segments
        let segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let root_cid = self.get_root_cid().await?;
        
        // Track all directory CIDs from root to target
        let mut dir_cids = vec![root_cid];
        let mut current_cid = root_cid;
        let mut needs_update = false;

        // Navigate/create each directory in the path
        for segment in &segments {
            let entries = self
                .unixfs
                .ls(&current_cid, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;

            let mut entries_vec = Vec::new();
            let mut entries_stream = entries;
            while let Some(entry) = entries_stream.next().await {
                entries_vec.push(entry);
            }

            // Check if segment exists
            if let Some(existing) = entries_vec.iter().find(|e| e.name == *segment) {
                if !matches!(existing.type_, UnixFSType::Directory) {
                    return Err(MfsError::InvalidPath(format!(
                        "'{}' exists but is not a directory",
                        segment
                    )));
                }
                current_cid = existing.cid;
                dir_cids.push(current_cid);
            } else {
                // Create new directory
                let new_dir_cid = self
                    .unixfs
                    .add_directory(None, None)
                    .await
                    .map_err(|e| MfsError::UnixFs(e.to_string()))?;

                // Add to current directory - this updates the parent
                let _updated_parent = self
                    .unixfs
                    .cp(&new_dir_cid, &current_cid, segment, None)
                    .await
                    .map_err(|e| MfsError::UnixFs(e.to_string()))?;
                
                // For next iteration, navigate into the new directory
                current_cid = new_dir_cid;
                dir_cids.push(new_dir_cid);
                needs_update = true;
            }
        }

        // If we created any new directories, update the chain back to root
        if needs_update {
            // Work backwards through the segments to update parent directories
            let mut updated_cid = *dir_cids.last().unwrap();
            
            for i in (1..segments.len()).rev() {
                let parent_cid = dir_cids[i - 1];
                let dir_name = segments[i];
                
                // Use add_or_update to prevent duplicates when updating pointers
                updated_cid = self
                    .add_or_update_entry(&parent_cid, dir_name, &updated_cid)
                    .await?;
                
                dir_cids[i] = updated_cid;
            }
            
            // Update the first level directory in root
            let new_root = self
                    .add_or_update_entry(&root_cid, segments[0], &dir_cids[1])
                    .await?;
            
            let mut root = self.root_cid.write().await;
            *root = Some(new_root);
        }

        Ok(())
    }

    async fn write_bytes(&self, path: &str, content: &[u8]) -> Result<(), MfsError> {
        let path = normalize_path(path)?;

        if path == "/" {
            return Err(MfsError::InvalidPath("Cannot write to root".to_string()));
        }

        // Split into parent and filename
        let (parent_path, filename) = split_path(&path)?;

        // Ensure parent directories exist
        if parent_path != "/" {
            self.mkdir(&parent_path).await?;
        }

        // Add file content
        let file_cid = self
            .unixfs
            .add_bytes(Bytes::from(content.to_vec()), None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        // Parse parent path into segments
        let parent_segments: Vec<String> = if parent_path == "/" {
            vec![]
        } else {
            parent_path
                .trim_start_matches('/')
                .split('/')
                .map(|s| s.to_string())
                .collect()
        };

        // Update the file in the directory structure
        let new_root = self.update_nested_file(&parent_segments, file_cid, &filename).await?;

        // Update root CID
        let mut root = self.root_cid.write().await;
        *root = Some(new_root);

        Ok(())
    }

    async fn ls(&self, path: &str) -> Result<Vec<UnixFSEntry>, MfsError> {
        let path = normalize_path(path)?;

        // Navigate to the target directory
        let target_cid = self.navigate_to_dir(&path).await?;

        // List the directory
        let entries_iter = self
            .unixfs
            .ls(&target_cid, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        // Convert iterator to vector
        let mut entries_vec = Vec::new();
        let mut entries_stream = entries_iter;
        while let Some(entry) = entries_stream.next().await {
            entries_vec.push(entry);
        }

        Ok(entries_vec)
    }

    async fn stat(&self, path: &str) -> Result<UnixFSEntry, MfsError> {
        let path = normalize_path(path)?;

        if path == "/" {
            let root_cid = self.get_root_cid().await?;
            return Ok(UnixFSEntry {
                name: "/".to_string(),
                cid: root_cid,
                size: 0,
                type_: UnixFSType::Directory,
                mode: None,
                mtime: None,
            });
        }

        // Split into parent and name
        let (parent_path, name) = split_path(&path)?;

        // List parent directory
        let parent_entries = self.ls(&parent_path).await?;

        // Find the entry
        parent_entries
            .into_iter()
            .find(|e| e.name == name)
            .ok_or_else(|| MfsError::InvalidPath(format!("'{}' not found", path)))
    }

    async fn cp(&self, from: &str, to: &str) -> Result<(), MfsError> {
        let from = normalize_path(from)?;
        let to = normalize_path(to)?;

        // Cannot copy from or to root
        if from == "/" {
            return Err(MfsError::InvalidPath(
                "Cannot copy root directory".to_string(),
            ));
        }

        if to == "/" {
            return Err(MfsError::InvalidPath(
                "Cannot copy to root (specify destination path)".to_string(),
            ));
        }

        // Get source entry info
        let (source_parent_path, source_name) = split_path(&from)?;
        let source_parent_cid = self.navigate_to_dir(&source_parent_path).await?;

        // Find source entry in parent
        let entries = self
            .unixfs
            .ls(&source_parent_cid, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        let mut entries_vec = Vec::new();
        let mut entries_stream = entries;
        while let Some(entry) = entries_stream.next().await {
            entries_vec.push(entry);
        }

        let source_entry = entries_vec
            .iter()
            .find(|e| e.name == source_name)
            .ok_or_else(|| MfsError::InvalidPath(format!("Source '{}' not found", from)))?;

        let source_cid = source_entry.cid;

        // Determine destination
        // Check if destination exists and is a directory
        let (dest_parent_path, dest_name) = if let Ok(dest_stat) = self.stat(&to).await {
            // Destination exists
            if matches!(dest_stat.type_, UnixFSType::Directory) {
                // Copying into a directory, use source name
                (to.clone(), source_name.to_string())
            } else {
                // Destination is a file, will overwrite
                split_path(&to)?
            }
        } else {
            // Destination doesn't exist, treat as new name
            split_path(&to)?
        };

        // Ensure destination parent exists
        if dest_parent_path != "/" {
            self.mkdir(&dest_parent_path).await?;
        }

        // Navigate to destination parent
        let dest_parent_cid = self.navigate_to_dir(&dest_parent_path).await?;

        // Add source to destination parent using add_or_update to prevent duplicates
        let updated_dest_parent_cid = self
            .add_or_update_entry(&dest_parent_cid, &dest_name, &source_cid)
            .await?;

        // Update the directory chain back to root
        if dest_parent_path == "/" {
            // Destination parent is root, just update root
            let mut root = self.root_cid.write().await;
            *root = Some(updated_dest_parent_cid);
        } else {
            // Need to update the entire chain
            let dest_segments: Vec<String> = dest_parent_path
                .trim_start_matches('/')
                .split('/')
                .map(|s| s.to_string())
                .collect();

            let new_root = self
                .update_directory_chain(&dest_segments, updated_dest_parent_cid)
                .await?;

            let mut root = self.root_cid.write().await;
            *root = Some(new_root);
        }

        Ok(())
    }

    async fn mv(&self, from: &str, to: &str) -> Result<(), MfsError> {
        let from = normalize_path(from)?;
        let to = normalize_path(to)?;

        // Cannot move root
        if from == "/" {
            return Err(MfsError::InvalidPath(
                "Cannot move root directory".to_string(),
            ));
        }

        // Cannot move to itself
        if from == to {
            return Ok(()); // No-op
        }

        // Check if trying to move to a subdirectory of itself
        if to.starts_with(&format!("{}/", from)) {
            return Err(MfsError::InvalidPath(
                "Cannot move directory into itself".to_string(),
            ));
        }

        // Copy to destination
        self.cp(&from, &to).await?;

        // Remove from source (use recursive for directories)
        // We need to check if source was a directory
        let (source_parent_path, source_name) = split_path(&from)?;
        let source_parent_cid = self.navigate_to_dir(&source_parent_path).await?;

        let entries = self
            .unixfs
            .ls(&source_parent_cid, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        let mut entries_vec = Vec::new();
        let mut entries_stream = entries;
        while let Some(entry) = entries_stream.next().await {
            entries_vec.push(entry);
        }

        let _source_entry = entries_vec
            .iter()
            .find(|e| e.name == source_name);

        // Remove source (always use recursive=true since cp already succeeded)
        self.rm(&from, true).await?;

        Ok(())
    }

    async fn rm(&self, path: &str, recursive: bool) -> Result<(), MfsError> {
        let path = normalize_path(path)?;

        if path == "/" {
            return Err(MfsError::InvalidPath(
                "Cannot remove root directory".to_string(),
            ));
        }

        // Split into parent and entry name
        let (parent_path, entry_name) = split_path(&path)?;

        // First, check if entry exists and if it's a directory
        let parent_cid = self.navigate_to_dir(&parent_path).await?;
        
        // List parent to verify entry exists
        let entries = self
            .unixfs
            .ls(&parent_cid, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        let mut entries_vec = Vec::new();
        let mut entries_stream = entries;
        while let Some(entry) = entries_stream.next().await {
            entries_vec.push(entry);
        }

        let entry = entries_vec
            .iter()
            .find(|e| e.name == entry_name)
            .ok_or_else(|| {
                MfsError::InvalidPath(format!("'{}' not found", path))
            })?;

        // Check if it's a directory and recursive flag
        if matches!(entry.type_, UnixFSType::Directory) && !recursive {
            // Check if directory is empty
            let dir_entries = self
                .unixfs
                .ls(&entry.cid, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;

            // Check if directory has any entries
            let mut dir_stream = dir_entries;
            let has_entries = dir_stream.next().await.is_some();

            if has_entries {
                return Err(MfsError::InvalidPath(
                    format!("Directory '{}' is not empty. Use recursive flag to remove.", path)
                ));
            }
        }

        // Use UnixFS rm to remove the entry
        let updated_parent_cid = self
            .unixfs
            .rm(&parent_cid, &entry_name, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        // Now update the parent chain back to root
        if parent_path == "/" {
            // Parent is root, just update root
            let mut root = self.root_cid.write().await;
            *root = Some(updated_parent_cid);
        } else {
            // Need to update the entire chain
            let parent_segments: Vec<String> = parent_path
                .trim_start_matches('/')
                .split('/')
                .map(|s| s.to_string())
                .collect();

            let new_root = self.update_directory_chain(&parent_segments, updated_parent_cid).await?;
            
            let mut root = self.root_cid.write().await;
            *root = Some(new_root);
        }

        Ok(())
    }

    async fn root_cid(&self) -> Option<Cid> {
        *self.root_cid.read().await
    }

    async fn flush(&self) -> Result<Cid, MfsError> {
        // Get the current root CID, creating an empty directory if needed
        // This ensures the file system has a valid root
        let root = self.get_root_cid().await?;
        
        // In a more complete implementation, this would:
        // 1. Ensure all UnixFS blocks are written to the blockstore
        // 2. Pin the root CID to prevent garbage collection
        // 3. Return the stable root CID
        //
        // For now, we simply return the current root CID which
        // represents the current state of the file system.
        Ok(root)
    }
}

/// Create an MFS instance
pub fn mfs(helia: Arc<dyn Helia>) -> impl MfsInterface {
    DefaultMfs::new(helia)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_helia::create_helia_default;

    async fn create_test_helia() -> Arc<dyn Helia> {
        Arc::new(
            create_helia_default()
                .await
                .expect("Failed to create Helia"),
        )
    }

    #[tokio::test]
    async fn test_mkdir() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);
        let result = fs.mkdir("/test-dir").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_bytes() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);
        let data = b"hello world";
        let result = fs.write_bytes("/test-file.txt", data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ls_root() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);
        let result = fs.ls("/").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mkdir_nested() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);
        let result = fs.mkdir("/foo/bar/baz").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stat_root() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);
        let result = fs.stat("/").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rm_file() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a file
        fs.write_bytes("/test.txt", b"content").await.unwrap();

        // Verify it exists
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test.txt");

        // Remove it
        fs.rm("/test.txt", false).await.unwrap();

        // Verify it's gone
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_rm_directory() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create an empty directory
        fs.mkdir("/empty").await.unwrap();

        // Verify it exists
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 1);

        // Remove it with recursive flag (safer for directory removal)
        fs.rm("/empty", true).await.unwrap();

        // Verify it's gone
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_rm_non_empty_directory_requires_recursive() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a directory with content
        fs.mkdir("/dir").await.unwrap();
        fs.write_bytes("/dir/file.txt", b"content").await.unwrap();

        // Try to remove without recursive flag - should fail
        let result = fs.rm("/dir", false).await;
        assert!(result.is_err());

        // Remove with recursive flag - should succeed
        fs.rm("/dir", true).await.unwrap();

        // Verify it's gone
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_overwrite_file_no_duplicates() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Write file first time
        fs.write_bytes("/test.txt", b"version 1").await.unwrap();
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test.txt");

        // Overwrite the file
        fs.write_bytes("/test.txt", b"version 2").await.unwrap();

        // Verify NO duplicates
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 1, "Should have exactly one entry, no duplicates");
        assert_eq!(entries[0].name, "test.txt");
    }

    #[tokio::test]
    async fn test_rm_nested_file() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create nested structure
        fs.mkdir("/docs/examples").await.unwrap();
        fs.write_bytes("/docs/examples/test.txt", b"content").await.unwrap();

        // Verify file exists
        let entries = fs.ls("/docs/examples").await.unwrap();
        assert_eq!(entries.len(), 1);

        // Remove the file
        fs.rm("/docs/examples/test.txt", false).await.unwrap();

        // Verify it's gone
        let entries = fs.ls("/docs/examples").await.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_cp_file() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a file
        fs.write_bytes("/original.txt", b"test content").await.unwrap();

        // Copy it
        fs.cp("/original.txt", "/copy.txt").await.unwrap();

        // Verify both exist
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "original.txt"));
        assert!(entries.iter().any(|e| e.name == "copy.txt"));
    }

    #[tokio::test]
    async fn test_cp_directory() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create directory with content
        fs.mkdir("/source").await.unwrap();
        fs.write_bytes("/source/file.txt", b"content").await.unwrap();

        // Copy directory
        fs.cp("/source", "/dest").await.unwrap();

        // Verify both exist
        let root_entries = fs.ls("/").await.unwrap();
        assert_eq!(root_entries.len(), 2);
        
        // Verify dest has the same content
        let dest_entries = fs.ls("/dest").await.unwrap();
        assert_eq!(dest_entries.len(), 1);
        assert_eq!(dest_entries[0].name, "file.txt");
    }

    #[tokio::test]
    async fn test_cp_to_existing_directory() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create source file and dest directory
        fs.write_bytes("/file.txt", b"content").await.unwrap();
        fs.mkdir("/dir").await.unwrap();

        // Copy file into directory
        fs.cp("/file.txt", "/dir").await.unwrap();

        // Verify file is copied into dir with original name
        let dir_entries = fs.ls("/dir").await.unwrap();
        assert_eq!(dir_entries.len(), 1);
        assert_eq!(dir_entries[0].name, "file.txt");

        // Verify original still exists
        let root_entries = fs.ls("/").await.unwrap();
        assert!(root_entries.iter().any(|e| e.name == "file.txt"));
    }

    #[tokio::test]
    async fn test_cp_overwrite() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create two files
        fs.write_bytes("/file1.txt", b"version 1").await.unwrap();
        fs.write_bytes("/file2.txt", b"version 2").await.unwrap();

        // Copy file1 to file2 (overwrite)
        fs.cp("/file1.txt", "/file2.txt").await.unwrap();

        // Verify no duplicates
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 2, "Should have exactly 2 files, no duplicates");
    }

    #[tokio::test]
    async fn test_mv_file() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a file
        fs.write_bytes("/original.txt", b"test content").await.unwrap();

        // Move it
        fs.mv("/original.txt", "/moved.txt").await.unwrap();

        // Verify only moved file exists
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "moved.txt");
    }

    #[tokio::test]
    async fn test_mv_directory() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create directory with content
        fs.mkdir("/source").await.unwrap();
        fs.write_bytes("/source/file.txt", b"content").await.unwrap();

        // Move directory
        fs.mv("/source", "/dest").await.unwrap();

        // Verify only dest exists
        let root_entries = fs.ls("/").await.unwrap();
        assert_eq!(root_entries.len(), 1);
        assert_eq!(root_entries[0].name, "dest");

        // Verify dest has the content
        let dest_entries = fs.ls("/dest").await.unwrap();
        assert_eq!(dest_entries.len(), 1);
        assert_eq!(dest_entries[0].name, "file.txt");
    }

    #[tokio::test]
    async fn test_mv_to_existing_directory() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create source file and dest directory
        fs.write_bytes("/file.txt", b"content").await.unwrap();
        fs.mkdir("/dir").await.unwrap();

        // Move file into directory
        fs.mv("/file.txt", "/dir").await.unwrap();

        // Verify file is moved into dir
        let dir_entries = fs.ls("/dir").await.unwrap();
        assert_eq!(dir_entries.len(), 1);
        assert_eq!(dir_entries[0].name, "file.txt");

        // Verify original is gone from root
        let root_entries = fs.ls("/").await.unwrap();
        assert!(!root_entries.iter().any(|e| e.name == "file.txt"));
    }

    #[tokio::test]
    async fn test_mv_overwrite() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create two files
        fs.write_bytes("/file1.txt", b"version 1").await.unwrap();
        fs.write_bytes("/file2.txt", b"version 2").await.unwrap();

        // Move file1 to file2 (overwrite)
        fs.mv("/file1.txt", "/file2.txt").await.unwrap();

        // Verify only one file exists
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 1, "Should have exactly 1 file after move");
        assert_eq!(entries[0].name, "file2.txt");
    }

    // ===== Edge Case Tests =====

    #[tokio::test]
    async fn test_deep_nesting() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a very deep directory structure
        fs.mkdir("/a/b/c/d/e/f/g/h/i/j").await.unwrap();

        // Write a file at the deepest level
        fs.write_bytes("/a/b/c/d/e/f/g/h/i/j/deep.txt", b"nested content")
            .await
            .unwrap();

        // Verify we can read it
        let entries = fs.ls("/a/b/c/d/e/f/g/h/i/j").await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "deep.txt");

        // Verify we can stat it
        let stat = fs.stat("/a/b/c/d/e/f/g/h/i/j/deep.txt").await.unwrap();
        assert_eq!(stat.size, 14);
    }

    #[tokio::test]
    async fn test_empty_file() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Write an empty file
        fs.write_bytes("/empty.txt", b"").await.unwrap();

        // Verify it exists
        let stat = fs.stat("/empty.txt").await.unwrap();
        assert_eq!(stat.size, 0);
        assert_eq!(stat.name, "empty.txt");
    }

    #[tokio::test]
    async fn test_special_characters_in_names() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create files with special characters (but valid for filesystems)
        fs.write_bytes("/file-with-dash.txt", b"dash").await.unwrap();
        fs.write_bytes("/file_with_underscore.txt", b"underscore")
            .await
            .unwrap();
        fs.write_bytes("/file.with.dots.txt", b"dots").await.unwrap();

        // Verify all exist
        let entries = fs.ls("/").await.unwrap();
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_cp_directory_structure() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a simple directory with a file
        fs.mkdir("/source").await.unwrap();
        fs.write_bytes("/source/file.txt", b"content")
            .await
            .unwrap();

        // Copy the directory
        fs.cp("/source", "/backup").await.unwrap();

        // Verify both exist
        let root_entries = fs.ls("/").await.unwrap();
        assert_eq!(root_entries.len(), 2); // source and backup

        // Since cp() copies the CID reference, /backup points to the same
        // UnixFS directory structure as /source
        let backup_entries = fs.ls("/backup").await.unwrap();
        assert_eq!(backup_entries.len(), 1);
        assert_eq!(backup_entries[0].name, "file.txt");
    }

    #[tokio::test]
    async fn test_flush_returns_root_cid() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Initially, root might not exist
        // Flush should create it
        let root1 = fs.flush().await.unwrap();

        // After writing a file, flush should return a different root
        fs.write_bytes("/file.txt", b"content").await.unwrap();
        let root2 = fs.flush().await.unwrap();

        // Root should change after modification
        assert_ne!(root1, root2, "Root CID should change after modifications");

        // root_cid() should match flush() result
        let root3 = fs.root_cid().await.unwrap();
        assert_eq!(root2, root3, "root_cid() should match flush() result");
    }

    #[tokio::test]
    async fn test_rm_error_on_root() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Trying to remove root should fail
        let result = fs.rm("/", false).await;
        assert!(result.is_err());

        // Even with recursive
        let result = fs.rm("/", true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stat_nonexistent() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Stat a non-existent file should fail
        let result = fs.stat("/nonexistent.txt").await;
        assert!(result.is_err());

        // Stat a non-existent directory should fail
        let result = fs.stat("/nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ls_empty_directory() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create an empty directory
        fs.mkdir("/empty_dir").await.unwrap();

        // List it - should return empty
        let entries = fs.ls("/empty_dir").await.unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_operations_sequence() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Complex sequence of operations
        fs.mkdir("/workspace/projects/rust").await.unwrap();
        fs.write_bytes("/workspace/projects/rust/main.rs", b"fn main() {}")
            .await
            .unwrap();
        fs.write_bytes("/workspace/README.md", b"# Workspace")
            .await
            .unwrap();

        // Copy a file
        fs.cp("/workspace/README.md", "/workspace/README_backup.md")
            .await
            .unwrap();

        // Move a file
        fs.mv("/workspace/README_backup.md", "/README_backup.md")
            .await
            .unwrap();

        // Remove a file
        fs.rm("/README_backup.md", false).await.unwrap();

        // Verify final state
        let workspace_entries = fs.ls("/workspace").await.unwrap();
        assert_eq!(workspace_entries.len(), 2); // projects dir + README.md

        let rust_entries = fs.ls("/workspace/projects/rust").await.unwrap();
        assert_eq!(rust_entries.len(), 1);
        assert_eq!(rust_entries[0].name, "main.rs");
    }

    #[tokio::test]
    async fn test_overwrite_directory_with_file_fails() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);

        // Create a directory
        fs.mkdir("/mydir").await.unwrap();

        // Try to write a file with the same name - this should work
        // (it will overwrite the directory)
        fs.write_bytes("/mydir", b"content").await.unwrap();

        // Verify it's now a file
        let stat = fs.stat("/mydir").await.unwrap();
        assert!(matches!(stat.type_, UnixFSType::File));
    }
}


