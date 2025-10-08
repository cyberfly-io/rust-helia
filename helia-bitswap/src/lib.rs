//! Bitswap protocol implementation

pub mod message;
pub mod network;
pub mod peer_manager;
pub mod session;
pub mod stats;
pub mod wantlist;

pub use message::*;
pub use network::*;
pub use peer_manager::*;
pub use session::*;
pub use stats::*;
pub use wantlist::*;

pub type Result<T> = std::result::Result<T, helia_interface::HeliaError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WantType {
    Block,
    Have,
}
