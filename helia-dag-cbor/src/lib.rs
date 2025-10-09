//! # Helia DAG-CBOR
//!
//! DAG-CBOR support for Helia, providing CBOR serialization and content addressing
//! for structured data compatible with IPFS DAG-CBOR specification.
//!
//! ## Example
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_cbor::{DagCbor, DagCborInterface};
//! use serde::{Deserialize, Serialize};
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct MyData {
//!     hello: String,
//!     numbers: Vec<i32>,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag = DagCbor::new(Arc::new(helia));
//!     
//!     let data = MyData {
//!         hello: "world".to_string(),
//!         numbers: vec![1, 2, 3],
//!     };
//!     
//!     // Add structured data
//!     let cid = dag.add(&data, None).await?;
//!     
//!     // Retrieve structured data
//!     let retrieved: MyData = dag.get(&cid, None).await?;
//!     assert_eq!(data, retrieved);
//!     
//!     Ok(())
//! }
//! ```

mod errors;
mod dag_cbor;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use cid::Cid;
use serde::{Deserialize, Serialize};

use helia_interface::AbortOptions;

pub use errors::*;
pub use dag_cbor::*;

/// Options for adding CBOR data
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    /// Whether to pin the data after adding
    pub pin: bool,
    /// Optional abort signal
    pub abort: Option<AbortOptions>,
}

/// Options for getting CBOR data
#[derive(Debug, Clone, Default)]
pub struct GetOptions {
    /// Optional abort signal
    pub abort: Option<AbortOptions>,
}

/// DAG-CBOR interface for adding and retrieving CBOR-encoded data
#[async_trait]
pub trait DagCborInterface {
    /// Add a CBOR-serializable object to the DAG
    ///
    /// # Arguments
    /// * `obj` - The object to serialize and add
    /// * `options` - Optional configuration for the add operation
    ///
    /// # Returns
    /// A CID identifying the stored CBOR data
    async fn add<T>(&self, obj: &T, options: Option<AddOptions>) -> Result<Cid, DagCborError>
    where
        T: Serialize + Send + Sync;

    /// Get a CBOR object from the DAG by CID
    ///
    /// # Arguments
    /// * `cid` - The CID of the CBOR data to retrieve
    /// * `options` - Optional configuration for the get operation
    ///
    /// # Returns
    /// The deserialized object
    async fn get<T>(&self, cid: &Cid, options: Option<GetOptions>) -> Result<T, DagCborError>
    where
        T: for<'de> Deserialize<'de> + Send;
}
