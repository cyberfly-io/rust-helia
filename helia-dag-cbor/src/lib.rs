//! # Helia DAG-CBOR
//!
//! CBOR (Concise Binary Object Representation) codec for Helia, providing efficient
//! serialization and content addressing for structured data in IPFS.
//!
//! ## Overview
//!
//! DAG-CBOR is a binary format for encoding structured data (objects, arrays, primitives)
//! with content addressing. It's ideal for:
//! - **Structured data storage**: Store complex objects with nested structures
//! - **Efficient serialization**: Binary format is more compact than JSON
//! - **Interoperability**: Works with other IPFS implementations (go-ipfs, js-ipfs)
//! - **Deterministic CIDs**: Same data always produces the same CID
//!
//! ## Core Concepts
//!
//! ### Content Addressing
//! Every piece of CBOR data is identified by a unique **CID** (Content Identifier)
//! derived from the serialized content. This ensures:
//! - Data integrity (tampering changes the CID)
//! - Deduplication (identical data shares the same CID)
//! - Verifiable links between data structures
//!
//! ### CBOR vs JSON
//! - **Binary format**: More compact than text-based JSON
//! - **Faster**: Quicker to serialize/deserialize
//! - **Type-safe**: Preserves numeric types, binary data
//! - **Deterministic**: Canonical encoding ensures reproducible CIDs
//!
//! ## Usage Examples
//!
//! ### Basic Object Storage
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_cbor::{DagCbor, DagCborInterface};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct Person {
//!     name: String,
//!     age: u32,
//!     email: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag = DagCbor::new(Arc::new(helia));
//!     
//!     let person = Person {
//!         name: "Alice".to_string(),
//!         age: 30,
//!         email: "alice@example.com".to_string(),
//!     };
//!     
//!     // Add object
//!     let cid = dag.add(&person, None).await?;
//!     println!("Stored person with CID: {}", cid);
//!     
//!     // Retrieve object
//!     let retrieved: Person = dag.get(&cid, None).await?;
//!     assert_eq!(person, retrieved);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Nested Structures
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_dag_cbor::{DagCbor, DagCborInterface, AddOptions};
//! # use serde::{Deserialize, Serialize};
//! # use std::sync::Arc;
//! # use std::collections::HashMap;
//! #
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Organization {
//!     name: String,
//!     departments: Vec<Department>,
//!     metadata: HashMap<String, String>,
//! }
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Department {
//!     name: String,
//!     employee_count: u32,
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = create_helia_default().await?;
//! # let dag = DagCbor::new(Arc::new(helia));
//! #
//! let org = Organization {
//!     name: "Acme Corp".to_string(),
//!     departments: vec![
//!         Department {
//!             name: "Engineering".to_string(),
//!             employee_count: 50,
//!         },
//!         Department {
//!             name: "Sales".to_string(),
//!             employee_count: 30,
//!         },
//!     ],
//!     metadata: HashMap::from([
//!         ("founded".to_string(), "2020".to_string()),
//!         ("location".to_string(), "San Francisco".to_string()),
//!     ]),
//! };
//!
//! let cid = dag.add(&org, None).await?;
//! let retrieved: Organization = dag.get(&cid, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Pinning Data
//!
//! Pin important data to prevent garbage collection:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_dag_cbor::{DagCbor, DagCborInterface, AddOptions};
//! # use serde::{Deserialize, Serialize};
//! # use std::sync::Arc;
//! #
//! # #[derive(Serialize, Deserialize)]
//! # struct Config { setting: String }
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = create_helia_default().await?;
//! # let dag = DagCbor::new(Arc::new(helia));
//! #
//! let config = Config {
//!     setting: "important value".to_string(),
//! };
//!
//! // Pin the configuration
//! let options = AddOptions {
//!     pin: true,
//!     ..Default::default()
//! };
//!
//! let cid = dag.add(&config, Some(options)).await?;
//! // Data is now pinned and won't be garbage collected
//! # Ok(())
//! # }
//! ```
//!
//! ### Primitive Types
//!
//! DAG-CBOR supports all JSON-compatible types:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_dag_cbor::{DagCbor, DagCborInterface};
//! # use std::sync::Arc;
//! # use std::collections::HashMap;
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let helia = create_helia_default().await?;
//! # let dag = DagCbor::new(Arc::new(helia));
//! #
//! // Strings
//! let text_cid = dag.add(&"Hello, IPFS!".to_string(), None).await?;
//!
//! // Numbers
//! let number_cid = dag.add(&42i32, None).await?;
//!
//! // Arrays
//! let array_cid = dag.add(&vec![1, 2, 3], None).await?;
//!
//! // Maps
//! let map = HashMap::from([
//!     ("key1".to_string(), "value1".to_string()),
//!     ("key2".to_string(), "value2".to_string()),
//! ]);
//! let map_cid = dag.add(&map, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Thread Safety
//!
//! DagCbor is thread-safe and can be shared across tasks:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_dag_cbor::{DagCbor, DagCborInterface};
//! # use std::sync::Arc;
//! # use serde::{Serialize, Deserialize};
//! #
//! # #[derive(Serialize, Deserialize)]
//! # struct Data { value: i32 }
//! #
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let helia = create_helia_default().await?;
//! let dag = Arc::new(DagCbor::new(Arc::new(helia)));
//!
//! // Clone and use in multiple tasks
//! let dag1 = Arc::clone(&dag);
//! let dag2 = Arc::clone(&dag);
//!
//! let handle1 = tokio::spawn(async move {
//!     dag1.add(&Data { value: 1 }, None).await
//! });
//!
//! let handle2 = tokio::spawn(async move {
//!     dag2.add(&Data { value: 2 }, None).await
//! });
//!
//! let (cid1, cid2) = tokio::join!(handle1, handle2);
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! ### Serialization
//! - **Small objects (<1KB)**: ~10-50µs
//! - **Medium objects (1-10KB)**: ~50-200µs
//! - **Large objects (>10KB)**: Linear with size
//!
//! ### Storage
//! - **CBOR overhead**: ~5-15% compared to raw binary
//! - **vs JSON**: 20-40% smaller on average
//! - **Deterministic**: Same input always produces same CID
//!
//! ### Memory Usage
//! - Objects are serialized in memory before storage
//! - Large objects (>1MB) should be chunked or split
//! - Consider using UnixFS for very large binary data
//!
//! ## Error Handling
//!
//! All operations return `Result<T, DagCborError>`:
//!
//! ```no_run
//! # use rust_helia::create_helia_default;
//! # use helia_dag_cbor::{DagCbor, DagCborInterface, DagCborError};
//! # use std::sync::Arc;
//! # use serde::{Serialize, Deserialize};
//! #
//! # #[derive(Serialize, Deserialize)]
//! # struct MyData { value: String }
//! #
//! # async fn example(dag: DagCbor, cid: &cid::Cid) -> Result<(), Box<dyn std::error::Error>> {
//! match dag.get::<MyData>(cid, None).await {
//!     Ok(data) => println!("Retrieved: {:?}", data.value),
//!     Err(DagCborError::InvalidCodec { codec }) => {
//!         eprintln!("Invalid codec: expected DAG-CBOR, got {}", codec);
//!     }
//!     Err(DagCborError::Cbor(e)) => {
//!         eprintln!("CBOR error: {}", e);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Limitations
//!
//! ### Current Constraints
//! - **Object size**: Recommended <10MB per object
//! - **Nested depth**: Very deep nesting (>100 levels) may impact performance
//! - **Binary data**: Consider UnixFS for large binary files
//!
//! ### Future Enhancements
//! - Streaming serialization for large objects
//! - Custom codecs support
//! - Advanced CID generation options
//!
//! ## Compatibility
//!
//! This implementation is compatible with:
//! - **IPFS Specification**: Follows DAG-CBOR spec
//! - **Other implementations**: Interoperable with go-ipfs, js-ipfs
//! - **CBOR Standard**: RFC 8949 compliant
//!
//! ## See Also
//!
//! - [`DagCborInterface`] - Main trait for DAG-CBOR operations
//! - [`DagCbor`] - Implementation struct
//! - [`AddOptions`] - Configuration for add operations
//! - [`GetOptions`] - Configuration for get operations
//! - [`DagCborError`] - Error types

mod dag_cbor;
mod errors;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use cid::Cid;
use serde::{Deserialize, Serialize};

use helia_interface::AbortOptions;

pub use dag_cbor::*;
pub use errors::*;

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
