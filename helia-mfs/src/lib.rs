//! MFS (Mutable File System) implementation for Helia

use std::sync::Arc;
use async_trait::async_trait;
use cid::Cid;
use bytes::Bytes;

use helia_interface::Helia;
use helia_unixfs::{create_unixfs, UnixFSInterface, UnixFSEntry};

/// Error types for MFS operations
#[derive(Debug, thiserror::Error)]
pub enum MfsError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("UnixFS error: {0}")]
    UnixFs(String),
}

/// The MFS interface
#[async_trait]
pub trait MfsInterface: Send + Sync {
    async fn mkdir(&self, path: &str) -> Result<(), MfsError>;
    async fn write_bytes(&self, bytes: &[u8], path: &str) -> Result<(), MfsError>;
    async fn ls(&self, path: Option<&str>) -> Result<Vec<UnixFSEntry>, MfsError>;
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
            let cid = self.unixfs.add_directory(None, None).await
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
        
        Ok(path.trim_start_matches('/').split('/').map(|s| s.to_string()).collect())
    }
}

#[async_trait]
impl MfsInterface for DefaultMfs {
    async fn mkdir(&self, path: &str) -> Result<(), MfsError> {
        let segments = self.parse_path(path)?;
        
        if segments.is_empty() {
            return Err(MfsError::InvalidPath("Root directory already exists".to_string()));
        }
        
        let dir_cid = self.unixfs.add_directory(None, None).await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;
        
        let root_cid = self.get_root_cid().await?;
        let dirname = segments.last().unwrap();
        
        let new_root = self.unixfs.cp(&dir_cid, &root_cid, dirname, None).await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;
        
        let mut root = self.root_cid.write().await;
        *root = Some(new_root);
        
        Ok(())
    }
    
    async fn write_bytes(&self, bytes: &[u8], path: &str) -> Result<(), MfsError> {
        let segments = self.parse_path(path)?;
        
        if segments.is_empty() {
            return Err(MfsError::InvalidPath("Cannot write to root".to_string()));
        }
        
        let file_cid = self.unixfs.add_bytes(Bytes::from(bytes.to_vec()), None).await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;
        
        let root_cid = self.get_root_cid().await?;
        let filename = segments.last().unwrap();
        
        let new_root = self.unixfs.cp(&file_cid, &root_cid, filename, None).await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;
        
        let mut root = self.root_cid.write().await;
        *root = Some(new_root);
        
        Ok(())
    }
    
    async fn ls(&self, path: Option<&str>) -> Result<Vec<UnixFSEntry>, MfsError> {
        let path = path.unwrap_or("/");
        let _segments = self.parse_path(path)?;
        
        let root_cid = self.get_root_cid().await?;
        let _entries_iter = self.unixfs.ls(&root_cid, None).await
            .map_err(|e| MfsError::UnixFs(e.to_string()))?;
        
        // For now, return empty list
        Ok(vec![])
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
        Arc::new(create_helia_default().await.expect("Failed to create Helia"))
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
        let result = fs.write_bytes(data, "/test-file.txt").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ls_root() {
        let helia = create_test_helia().await;
        let fs = mfs(helia);
        let result = fs.ls(None).await;
        assert!(result.is_ok());
    }
}