//! # Helia
//!
//! An implementation of IPFS in Rust
//!
//! This crate provides a `create_helia` function that returns an object implementing
//! the [`Helia`] trait. Pass it to other modules like `helia-unixfs` to make files
//! available on the distributed web.
//!
//! ## Example
//!
//! ```rust
//! use helia::create_helia;
//! use cid::Cid;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia(None).await?;
//!     
//!     // Use helia with other modules
//!     // let fs = helia_unixfs::create_unixfs(helia);
//!     // fs.cat(&cid).await?;
//!     
//!     Ok(())
//! }
//! ```

use helia_utils::{HeliaConfig, HeliaImpl};

pub use helia_interface::*;
pub use helia_utils::{
    create_swarm, create_swarm_with_keypair, BlockstoreConfig, DatastoreConfig, LoggerConfig,
};

/// Create a new Helia node with the given configuration
///
/// If no configuration is provided, sensible defaults will be used.
pub async fn create_helia(config: Option<HeliaConfig>) -> Result<HeliaImpl, HeliaError> {
    let config = config.unwrap_or_default();
    let helia = HeliaImpl::new(config).await?;
    Ok(helia)
}

/// Create a new Helia node with default configuration
pub async fn create_helia_default() -> Result<HeliaImpl, HeliaError> {
    create_helia(None).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_helia_default() {
        let helia = create_helia_default().await;
        assert!(helia.is_ok());
    }

    #[tokio::test]
    async fn test_create_helia_with_config() {
        let config = HeliaConfig::default();
        let helia = create_helia(Some(config)).await;
        assert!(helia.is_ok());
    }
}
