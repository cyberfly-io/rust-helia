//! # Helia JSON
//!
//! Simple JSON serialization for Helia, providing straightforward JSON storage
//! with content addressing for structured data in IPFS.
//!
//! ## Overview
//!
//! This module provides a simplified JSON interface for Helia, focusing on ease of use
//! for basic JSON operations. It's ideal when you need:
//!
//! - **Simple JSON storage**: Direct JSON-to-CID mapping
//! - **Quick prototyping**: Minimal setup for JSON operations
//! - **Familiar API**: Standard JSON serialization patterns
//! - **IPFS compatibility**: Works with IPFS ecosystem
//!
//! ## When to Use JSON vs DAG-JSON
//!
//! ### Use `helia-json` when:
//! - You need simple, straightforward JSON storage
//! - Working with standard JSON objects without IPLD features
//! - Building quick prototypes or simple applications
//! - Prefer minimal API surface
//!
//! ### Use `helia-dag-json` when:
//! - You need full IPLD (InterPlanetary Linked Data) support
//! - Working with linked data structures (CID links)
//! - Building complex DAG (Directed Acyclic Graph) applications
//! - Need compatibility with go-ipfs/js-ipfs DAG-JSON implementations
//! - Require advanced IPLD features
//!
//! ## Core Concepts
//!
//! ### Content Addressing
//! Each JSON object is stored with a unique Content Identifier (CID):
//! - CID is deterministic based on content
//! - CID includes codec identifier (0x0200 for JSON)
//! - CID provides integrity verification
//!
//! ### JSON Codec
//! This module uses the JSON codec (0x0200) from the multicodec table, providing
//! a simple JSON serialization format for IPFS storage.
//!
//! ## Usage Examples
//!
//! ### Example 1: Basic Object Storage
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_json::{Json, JsonInterface};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let json = Json::new(Arc::new(helia));
//!     
//!     let user = User {
//!         name: "Alice".to_string(),
//!         age: 30,
//!     };
//!     
//!     // Store JSON object
//!     let cid = json.add(&user, None).await?;
//!     println!("Stored at: {}", cid);
//!     
//!     // Retrieve JSON object
//!     let retrieved: User = json.get(&cid, None).await?;
//!     assert_eq!(user.name, retrieved.name);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 2: Collections and HashMaps
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_json::{Json, JsonInterface};
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let json = Json::new(Arc::new(helia));
//!     
//!     let mut config = HashMap::new();
//!     config.insert("api_key".to_string(), "secret123".to_string());
//!     config.insert("endpoint".to_string(), "https://api.example.com".to_string());
//!     
//!     let cid = json.add(&config, None).await?;
//!     
//!     let loaded: HashMap<String, String> = json.get(&cid, None).await?;
//!     println!("API Key: {}", loaded.get("api_key").unwrap());
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 3: Pinning for Persistence
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_json::{Json, JsonInterface, AddOptions};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Document {
//!     title: String,
//!     content: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let json = Json::new(Arc::new(helia));
//!     
//!     let doc = Document {
//!         title: "Important Note".to_string(),
//!         content: "Remember to pin!".to_string(),
//!     };
//!     
//!     // Pin to prevent garbage collection
//!     let options = AddOptions {
//!         pin: true,
//!         ..Default::default()
//!     };
//!     
//!     let cid = json.add(&doc, Some(options)).await?;
//!     println!("Pinned document at: {}", cid);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Example 4: Thread-Safe Operations
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_json::{Json, JsonInterface};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize, Clone)]
//! struct Message {
//!     id: u32,
//!     text: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let json = Arc::new(Json::new(Arc::new(helia)));
//!     
//!     let mut handles = vec![];
//!     
//!     for i in 0..5 {
//!         let json_clone = json.clone();
//!         let handle = tokio::spawn(async move {
//!             let msg = Message {
//!                 id: i,
//!                 text: format!("Message {}", i),
//!             };
//!             json_clone.add(&msg, None).await
//!         });
//!         handles.push(handle);
//!     }
//!     
//!     for handle in handles {
//!         let cid = handle.await??;
//!         println!("Stored message at: {}", cid);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! ### Serialization
//! - **Small objects (<1KB)**: ~15-60µs
//! - **Medium objects (1-10KB)**: ~60-250µs
//! - **Large objects (>10KB)**: 250µs+
//!
//! ### Storage
//! - **JSON overhead**: Similar to DAG-JSON (~30-50% larger than CBOR)
//! - **Recommended size**: <10MB per object
//! - **Format**: Human-readable text
//!
//! ## Error Handling
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_json::{Json, JsonInterface, JsonError};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Data { value: String }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let helia = create_helia_default().await?;
//!     let json = Json::new(Arc::new(helia));
//!     
//!     let data = Data { value: "test".to_string() };
//!     let cid = json.add(&data, None).await?;
//!     
//!     match json.get::<Data>(&cid, None).await {
//!         Ok(retrieved) => println!("Success: {}", retrieved.value),
//!         Err(JsonError::InvalidCodec { expected, actual }) => {
//!             eprintln!("Wrong codec: expected {}, got {}", expected, actual);
//!         }
//!         Err(JsonError::Serialization(e)) => {
//!             eprintln!("Serialization error: {}", e);
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Comparison: JSON vs DAG-JSON vs DAG-CBOR
//!
//! | Feature | JSON | DAG-JSON | DAG-CBOR |
//! |---------|------|----------|----------|
//! | Simplicity | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
//! | IPLD Support | ❌ | ✅ | ✅ |
//! | CID Links | ❌ | ✅ | ✅ |
//! | Format | Text | Text | Binary |
//! | Size | Medium | Medium | Small |
//! | Speed | Medium | Medium | Fast |
//! | Use Case | Simple | IPLD/DAG | Performance |
//!
//! ## Limitations
//!
//! ### Current Constraints
//! 1. **No IPLD features**: No support for CID links within objects
//! 2. **Object size**: Recommended <10MB per object
//! 3. **No DAG operations**: Not suitable for complex graph structures
//!
//! ### When to Use Alternatives
//! - **Need IPLD/CID links**: Use `helia-dag-json`
//! - **Need performance**: Use `helia-dag-cbor`
//! - **Need files**: Use `helia-unixfs`
//!
//! ## Compatibility
//!
//! - **JSON codec**: 0x0200 (multicodec table)
//! - **Serialization**: Standard serde_json
//! - **IPFS compatible**: Works with IPFS ecosystem

pub mod errors;
pub mod json;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use helia_interface::{AbortOptions, Helia};

pub use errors::*;
pub use json::*;

/// Options for adding JSON data
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    /// Optional abort signal
    pub abort_signal: Option<AbortOptions>,
    /// Whether to pin the added data
    pub pin: bool,
}

/// Options for getting JSON data
#[derive(Debug, Clone, Default)]
pub struct GetOptions {
    /// Optional abort signal
    pub abort_signal: Option<AbortOptions>,
}

/// Create a JSON instance for use with Helia
pub fn json(helia: Arc<dyn Helia>) -> Json {
    Json::new(helia)
}
