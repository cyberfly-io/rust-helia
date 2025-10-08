//! Bitswap protocol implementation
//! 
//! This is a Rust implementation of the Bitswap protocol based on the
//! TypeScript @helia/bitswap package.
//!
//! Bitswap is a data exchange protocol used in IPFS for requesting and
//! providing blocks of data between peers.

// Core modules (TypeScript-based architecture)
pub mod constants;
pub mod pb;
pub mod utils;
pub mod network_new;
pub mod wantlist_new;
pub mod peer_want_lists;

// Session module (to be rewritten)
pub mod session;

// Re-exports
pub use constants::*;
pub use pb::{WantType, BlockPresenceType};
pub use utils::*;

// Architecture exports
pub use network_new::{Network, NetworkInit, NetworkEvent, BitswapMessageEvent};
pub use wantlist_new::{WantList, WantListEntry, WantResult};
pub use peer_want_lists::{PeerWantLists, PeerWantListsStats};

// Session exports (temporary until rewrite)
pub use session::*;

pub type Result<T> = std::result::Result<T, helia_interface::HeliaError>;
