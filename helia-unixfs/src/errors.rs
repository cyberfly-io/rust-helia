//! UnixFS-specific error types

use thiserror::Error;
use cid::Cid;
use helia_interface::HeliaError;

/// Base error type for UnixFS operations
#[derive(Error, Debug)]
pub enum UnixFSError {
    #[error("Not a UnixFS file or directory: {cid}")]
    NotUnixFS { cid: Cid },

    #[error("Invalid protobuf node: {reason}")]
    InvalidPBNode { reason: String },

    #[error("File or directory already exists: {path}")]
    AlreadyExists { path: String },

    #[error("File or directory does not exist: {path}")]
    DoesNotExist { path: String },

    #[error("No content found")]
    NoContent,

    #[error("Not a file: {cid}")]
    NotAFile { cid: Cid },

    #[error("Not a directory: {cid}")]
    NotADirectory { cid: Cid },

    #[error("Invalid parameters: {reason}")]
    InvalidParameters { reason: String },

    #[error("Unsupported UnixFS type: {type_name}")]
    UnsupportedType { type_name: String },

    #[error("Helia error: {0}")]
    Helia(#[from] HeliaError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Protobuf error: {0}")]
    Protobuf(#[from] prost::DecodeError),

    #[error("Other error: {message}")]
    Other { message: String },
}

impl UnixFSError {
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other { message: message.into() }
    }

    pub fn invalid_parameters(reason: impl Into<String>) -> Self {
        Self::InvalidParameters { reason: reason.into() }
    }

    pub fn invalid_pb_node(reason: impl Into<String>) -> Self {
        Self::InvalidPBNode { reason: reason.into() }
    }

    pub fn not_unixfs(cid: Cid) -> Self {
        Self::NotUnixFS { cid }
    }

    pub fn not_a_file(cid: Cid) -> Self {
        Self::NotAFile { cid }
    }

    pub fn not_a_directory(cid: Cid) -> Self {
        Self::NotADirectory { cid }
    }

    pub fn already_exists(path: impl Into<String>) -> Self {
        Self::AlreadyExists { path: path.into() }
    }

    pub fn does_not_exist(path: impl Into<String>) -> Self {
        Self::DoesNotExist { path: path.into() }
    }

    pub fn unsupported_type(type_name: impl Into<String>) -> Self {
        Self::UnsupportedType { type_name: type_name.into() }
    }
}