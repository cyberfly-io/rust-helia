//! Mutable File System (MFS) for Helia
//!
//! This module provides a mutable file system layer on top of UnixFS, allowing
//! users to work with IPFS content using familiar file system operations like
//! mkdir, write, and ls.

mod path;
mod operations;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::StreamExt;
use helia_interface::{AwaitIterable, Helia};
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

    fn parse_path(&self, path: &str) -> Result<Vec<String>, MfsError> {
        if !path.starts_with('/') {
            return Err(MfsError::InvalidPath("Path must start with /".to_string()));
        }

        if path == "/" {
            return Ok(vec![]);
        }

        Ok(path
            .trim_start_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect())
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
            // At root level - add file directly to root
            let root_cid = self.get_root_cid().await?;
            return self
                .unixfs
                .cp(&file_cid, &root_cid, filename, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()));
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
        
        // Add file to target directory
        let mut updated_cid = self
            .unixfs
            .cp(&file_cid, &target_dir_cid, filename, None)
            .await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;

        // Update each parent directory from bottom to top
        for i in (0..path_segments.len()).rev() {
            let parent_cid = dir_cids[i];
            let dir_name = &path_segments[i];
            
            updated_cid = self
                .unixfs
                .cp(&updated_cid, &parent_cid, dir_name, None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;
        }

        Ok(updated_cid)
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

                // Add to current directory
                current_cid = self
                    .unixfs
                    .cp(&new_dir_cid, &current_cid, segment, None)
                    .await
                    .map_err(|e| MfsError::UnixFs(e.to_string()))?;
                
                dir_cids.push(current_cid);
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
                
                updated_cid = self
                    .unixfs
                    .cp(&updated_cid, &parent_cid, dir_name, None)
                    .await
                    .map_err(|e| MfsError::UnixFs(e.to_string()))?;
                
                dir_cids[i] = updated_cid;
            }
            
            // Update the first level directory in root
            let new_root = self
                .unixfs
                .cp(&dir_cids[1], &root_cid, segments[0], None)
                .await
                .map_err(|e| MfsError::UnixFs(e.to_string()))?;
            
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

    async fn cp(&self, _from: &str, _to: &str) -> Result<(), MfsError> {
        Err(MfsError::InvalidPath(
            "Copy operation not yet implemented".to_string(),
        ))
    }

    async fn mv(&self, _from: &str, _to: &str) -> Result<(), MfsError> {
        Err(MfsError::InvalidPath(
            "Move operation not yet implemented".to_string(),
        ))
    }

    async fn rm(&self, _path: &str, _recursive: bool) -> Result<(), MfsError> {
        Err(MfsError::InvalidPath(
            "Remove operation not yet implemented".to_string(),
        ))
    }

    async fn root_cid(&self) -> Option<Cid> {
        *self.root_cid.read().await
    }

    async fn flush(&self) -> Result<Cid, MfsError> {
        self.get_root_cid().await
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
}
