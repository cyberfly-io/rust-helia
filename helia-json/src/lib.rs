//! # Helia JSON
//!
//! JSON support for Helia, providing simple JSON serialization and storage
//! for structured data compatible with IPFS JSON specification.
//!
//! ## Example
//!
//! ```no_run
//! use rust_helia::create_helia_default;
//! use helia_json::{Json, JsonInterface};
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
//!     let json = Json::new(Arc::new(helia));
//!     
//!     let data = MyData {
//!         hello: "world".to_string(),
//!         numbers: vec![1, 2, 3],
//!     };
//!     
//!     // Add JSON data
//!     let cid = json.add(&data, None).await?;
//!     
//!     // Retrieve JSON data
//!     let retrieved: MyData = json.get(&cid, None).await?;
//!     assert_eq!(data, retrieved);
//!     
//!     Ok(())
//! }
//! ```

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
