//! Error types for DAG-CBOR operations

use thiserror::Error;
use helia_interface::HeliaError;

/// Errors that can occur during DAG-CBOR operations
#[derive(Error, Debug)]
pub enum DagCborError {
    /// Error from the underlying Helia instance
    #[error("Helia error: {0}")]
    Helia(#[from] HeliaError),

    /// CBOR serialization/deserialization error
    #[error("CBOR error: {0}")]
    Cbor(#[from] serde_cbor::Error),

    /// Invalid codec error - CID does not correspond to CBOR data
    #[error("Invalid codec: expected DAG-CBOR but got codec {codec}")]
    InvalidCodec { codec: u64 },

    /// Generic error for other issues
    #[error("DAG-CBOR error: {message}")]
    Other { message: String },
}

impl DagCborError {
    /// Create a new invalid codec error
    pub fn invalid_codec(codec: u64) -> Self {
        DagCborError::InvalidCodec { codec }
    }

    /// Create a new generic error
    pub fn other(message: impl Into<String>) -> Self {
        DagCborError::Other {
            message: message.into(),
        }
    }
}