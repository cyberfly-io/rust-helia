//! Protobuf types for IPNS records

// Include the generated protobuf code
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/ipns.rs"));
}

pub use proto::*;
