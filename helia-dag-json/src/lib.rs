//! # Helia DAG-JSON
//!
//! DAG-JSON support for Helia, providing JSON serialization with content addressing
//! for structured data compatible with the IPFS DAG-JSON specification.
//!
//! ## Overview
//!
//! This module provides JSON encoding and decoding for IPLD (InterPlanetary Linked Data) with
//! content addressing. DAG-JSON is useful when you need:
//!
//! - **Human-readable storage**: JSON format is easy to debug and inspect
//! - **Web compatibility**: JSON works seamlessly with browsers and web APIs
//! - **Interoperability**: Compatible with existing JSON-based systems
//! - **Text-based data**: Better for configuration files and metadata
//! - **IPFS compatibility**: Works with go-ipfs and js-ipfs DAG-JSON implementations
//!
//! ## Core Concepts
//!
//! ### Content Addressing
//! Each JSON object is serialized and stored with a unique Content Identifier (CID):
//! - CID is deterministic: same input always produces the same CID
//! - CID includes codec identifier (0x0129 for DAG-JSON)
//! - CID provides integrity verification (content tampering is detectable)
//!
//! ### DAG-JSON vs DAG-CBOR
//! | Feature | DAG-JSON | DAG-CBOR |
//! |---------|----------|----------|
//! | Format | Text-based | Binary |
//! | Size | Larger (verbose) | Smaller (compact) |
//! | Readability | High | Low |
//! | Parsing Speed | Slower | Faster |
//! | Use Case | Web, debugging | Performance, storage efficiency |
//!
//! Choose DAG-JSON when readability and web compatibility matter more than size.
//!
//! ## Usage Examples
//!
//! ### Example 1: Basic Object Storage
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_json::{DagJson, DagJsonInterface};
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
//!     let dag_json = DagJson::new(Arc::new(helia));
//!     
//!     let person = Person {
//!         name: "Alice".to_string(),
//!         age: 30,
//!         email: "alice@example.com".to_string(),
//!     };
//!     
//!     // Store the person object
//!     let cid = dag_json.add(&person, None).await?;
//!     println!("Stored person at: {}", cid);
//!     
//!     // Retrieve the person object
//!     let retrieved: Person = dag_json.get(&cid, None).await?;
//!     assert_eq!(person.name, retrieved.name);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 2: Nested Structures (Configuration Files)
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_json::{DagJson, DagJsonInterface};
//! use serde::{Deserialize, Serialize};
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct AppConfig {
//!     version: String,
//!     features: HashMap<String, bool>,
//!     endpoints: Vec<String>,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag_json = DagJson::new(Arc::new(helia));
//!     
//!     let mut features = HashMap::new();
//!     features.insert("authentication".to_string(), true);
//!     features.insert("caching".to_string(), false);
//!     
//!     let config = AppConfig {
//!         version: "1.0.0".to_string(),
//!         features,
//!         endpoints: vec![
//!             "https://api.example.com".to_string(),
//!             "https://backup.example.com".to_string(),
//!         ],
//!     };
//!     
//!     // Store configuration
//!     let cid = dag_json.add(&config, None).await?;
//!     
//!     // Later: retrieve and use configuration
//!     let loaded_config: AppConfig = dag_json.get(&cid, None).await?;
//!     println!("Loaded version: {}", loaded_config.version);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 3: Pinning Data
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_json::{DagJson, DagJsonInterface, AddOptions};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct ImportantData {
//!     id: u64,
//!     content: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag_json = DagJson::new(Arc::new(helia));
//!     
//!     let data = ImportantData {
//!         id: 12345,
//!         content: "Critical system data".to_string(),
//!     };
//!     
//!     // Pin to prevent garbage collection
//!     let options = AddOptions {
//!         pin: true,
//!         ..Default::default()
//!     };
//!     
//!     let cid = dag_json.add(&data, Some(options)).await?;
//!     println!("Pinned data at: {}", cid);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 4: Primitive Types and Collections
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_json::{DagJson, DagJsonInterface};
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag_json = DagJson::new(Arc::new(helia));
//!     
//!     // Store a string
//!     let text = "Hello, IPFS!".to_string();
//!     let text_cid = dag_json.add(&text, None).await?;
//!     
//!     // Store an array
//!     let numbers = vec![1, 2, 3, 4, 5];
//!     let array_cid = dag_json.add(&numbers, None).await?;
//!     
//!     // Store a map
//!     let mut metadata = HashMap::new();
//!     metadata.insert("author".to_string(), "Alice".to_string());
//!     metadata.insert("version".to_string(), "1.0".to_string());
//!     let map_cid = dag_json.add(&metadata, None).await?;
//!     
//!     // Retrieve them
//!     let text_back: String = dag_json.get(&text_cid, None).await?;
//!     let numbers_back: Vec<i32> = dag_json.get(&array_cid, None).await?;
//!     let metadata_back: HashMap<String, String> = dag_json.get(&map_cid, None).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 5: Thread-Safe Concurrent Operations
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_json::{DagJson, DagJsonInterface};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! struct Record {
//!     id: u32,
//!     data: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag_json = Arc::new(DagJson::new(Arc::new(helia)));
//!     
//!     let mut handles = vec![];
//!     
//!     // Spawn multiple tasks to store data concurrently
//!     for i in 0..10 {
//!         let dag = dag_json.clone();
//!         let handle = tokio::spawn(async move {
//!             let record = Record {
//!                 id: i,
//!                 data: format!("Record {}", i),
//!             };
//!             dag.add(&record, None).await
//!         });
//!         handles.push(handle);
//!     }
//!     
//!     // Wait for all tasks to complete
//!     for handle in handles {
//!         let cid = handle.await??;
//!         println!("Stored record at: {}", cid);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! ### Serialization Performance
//! | Object Size | Serialization Time | Notes |
//! |-------------|-------------------|-------|
//! | Small (<1KB) | 15-70µs | Simple objects, few fields |
//! | Medium (1-10KB) | 70-300µs | Nested structures, arrays |
//! | Large (>10KB) | 300µs+ | Complex graphs, many fields |
//!
//! ### Storage Overhead
//! - **JSON overhead**: ~30-50% larger than CBOR
//! - **vs JSON files**: ~5-10% overhead (CID metadata)
//! - **Readability**: High - can inspect with any JSON viewer
//!
//! ### Memory Usage
//! Objects are serialized in memory before storage:
//! - Small objects (<10KB): Minimal memory impact
//! - Large objects (>100KB): Consider streaming or chunking
//! - Very large data: Use UnixFS for better performance
//!
//! ## Error Handling
//!
//! The module provides typed errors for common failure scenarios:
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_dag_json::{DagJson, DagJsonInterface, DagJsonError};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyData { value: String }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let dag_json = DagJson::new(Arc::new(helia));
//!     
//!     let data = MyData { value: "test".to_string() };
//!     let cid = dag_json.add(&data, None).await?;
//!     
//!     // Handle potential errors
//!     match dag_json.get::<MyData>(&cid, None).await {
//!         Ok(retrieved) => println!("Data: {}", retrieved.value),
//!         Err(DagJsonError::InvalidCodec { codec }) => {
//!             eprintln!("Wrong codec: expected DAG-JSON, got {}", codec);
//!         }
//!         Err(DagJsonError::Json(e)) => {
//!             eprintln!("JSON parsing failed: {}", e);
//!         }
//!         Err(e) => eprintln!("Other error: {}", e),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Limitations
//!
//! ### Current Constraints
//! 1. **Object size**: Recommended <10MB per JSON object for optimal performance
//! 2. **Parsing overhead**: JSON parsing is slower than binary formats (CBOR)
//! 3. **Storage efficiency**: 30-50% larger than DAG-CBOR
//! 4. **Floating point**: Limited precision compared to native JSON
//!
//! ### When to Use DAG-CBOR Instead
//! Consider using `helia-dag-cbor` if you need:
//! - Maximum storage efficiency
//! - Faster serialization/deserialization
//! - Binary data support
//! - High-performance applications
//!
//! ### Future Enhancements
//! - Streaming JSON parsing for very large objects
//! - Custom serialization options
//! - Schema validation support
//!
//! ## Compatibility
//!
//! This implementation is compatible with:
//! - **IPFS DAG-JSON spec**: Follows the official DAG-JSON specification
//! - **go-ipfs**: Can read/write data from go-ipfs nodes
//! - **js-ipfs**: Compatible with JavaScript IPFS implementations
//! - **RFC 8259**: Follows JSON specification (RFC 8259)

mod dag_json;
mod errors;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use cid::Cid;
use serde::{Deserialize, Serialize};

use helia_interface::AbortOptions;

pub use dag_json::*;
pub use errors::*;

/// Options for adding JSON data
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    /// Whether to pin the data after adding
    pub pin: bool,
    /// Optional abort signal
    pub abort: Option<AbortOptions>,
}

/// Options for getting JSON data
#[derive(Debug, Clone, Default)]
pub struct GetOptions {
    /// Optional abort signal
    pub abort: Option<AbortOptions>,
}

/// DAG-JSON interface for adding and retrieving JSON-encoded data
#[async_trait]
pub trait DagJsonInterface {
    /// Add a JSON-serializable object to the DAG
    ///
    /// # Arguments
    /// * `obj` - The object to serialize and add
    /// * `options` - Optional configuration for the add operation
    ///
    /// # Returns
    /// A CID identifying the stored JSON data
    async fn add<T>(&self, obj: &T, options: Option<AddOptions>) -> Result<Cid, DagJsonError>
    where
        T: Serialize + Send + Sync;

    /// Get a JSON object from the DAG by CID
    ///
    /// # Arguments
    /// * `cid` - The CID of the JSON data to retrieve
    /// * `options` - Optional configuration for the get operation
    ///
    /// # Returns
    /// The deserialized object
    async fn get<T>(&self, cid: &Cid, options: Option<GetOptions>) -> Result<T, DagJsonError>
    where
        T: for<'de> Deserialize<'de> + Send;
}
