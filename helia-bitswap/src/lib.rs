//! Bitswap protocol implementation
//!
//! This is a Rust implementation of the Bitswap protocol based on the
//! TypeScript @helia/bitswap package.
//!
//! Bitswap is a data exchange protocol used in IPFS for requesting and
//! providing blocks of data between peers.

// Core modules (TypeScript-based architecture)
pub mod behaviour;
pub mod constants;
pub mod coordinator;
pub mod network_new;
pub mod pb;
pub mod peer_want_lists;
pub mod stream;
pub mod utils;
pub mod wantlist_new;

// Session module (to be rewritten)
pub mod session;

// Re-exports
pub use constants::*;
pub use pb::{BlockPresenceType, WantType};
pub use utils::*;

// Architecture exports
pub use behaviour::{BitswapBehaviour, BitswapEvent};
pub use coordinator::{Bitswap, BitswapConfig, BitswapStats, NotifyOptions, WantOptions};
pub use network_new::{BitswapMessageEvent, Network, NetworkEvent, NetworkInit};
pub use peer_want_lists::{PeerWantLists, PeerWantListsStats};
pub use wantlist_new::{WantList, WantListEntry, WantResult};

// Session exports (temporary until rewrite)
pub use session::*;

pub type Result<T> = std::result::Result<T, helia_interface::HeliaError>;
