//! Error types for DAG-JSON operations

use helia_interface::HeliaError;
use thiserror::Error;

/// Errors that can occur during DAG-JSON operations
#[derive(Error, Debug)]
pub enum DagJsonError {
    /// Error from the underlying Helia instance
    #[error("Helia error: {0}")]
    Helia(#[from] HeliaError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid codec error - CID does not correspond to JSON data
    #[error("Invalid codec: expected DAG-JSON but got codec {codec}")]
    InvalidCodec { codec: u64 },

    /// Generic error for other issues
    #[error("DAG-JSON error: {message}")]
    Other { message: String },
}

impl DagJsonError {
    /// Create a new invalid codec error
    pub fn invalid_codec(codec: u64) -> Self {
        DagJsonError::InvalidCodec { codec }
    }

    /// Create a new generic error
    pub fn other(message: impl Into<String>) -> Self {
        DagJsonError::Other {
            message: message.into(),
        }
    }
}
