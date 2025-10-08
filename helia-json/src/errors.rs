//! Error types for JSON operations

use thiserror::Error;

/// Errors that can occur during JSON operations
#[derive(Error, Debug)]
pub enum JsonError {
    /// Error serializing object to JSON
    #[error("Failed to serialize object to JSON: {0}")]
    Serialization(String),

    /// Error deserializing JSON to object
    #[error("Failed to deserialize JSON to object: {0}")]
    Deserialization(String),

    /// Error storing JSON data
    #[error("Failed to store JSON data: {0}")]
    Storage(String),

    /// Error retrieving JSON data
    #[error("Failed to retrieve JSON data: {0}")]
    Retrieval(String),

    /// Invalid codec for JSON data
    #[error("Invalid codec - expected JSON codec (0x0200), got {actual:#x}")]
    InvalidCodec { expected: u64, actual: u64 },
}