//! Constants for IPNS operations

/// Maximum recursion depth for resolving IPNS records
pub const MAX_RECURSIVE_DEPTH: u32 = 32;

/// Default lifetime for IPNS records (48 hours in milliseconds)
pub const DEFAULT_LIFETIME_MS: u64 = 48 * 60 * 60 * 1000;

/// Default TTL for IPNS records (5 minutes in nanoseconds)
pub const DEFAULT_TTL_NS: u64 = 5 * 60 * 1_000_000_000;

/// Default republish interval (1 hour in milliseconds)
pub const DEFAULT_REPUBLISH_INTERVAL_MS: u64 = 60 * 60 * 1000;

/// Default republish concurrency (how many records to republish at once)
pub const DEFAULT_REPUBLISH_CONCURRENCY: usize = 5;

/// DHT record expiry time (24 hours in milliseconds)
pub const DHT_EXPIRY_MS: u64 = 24 * 60 * 60 * 1000;

/// Threshold before expiry to trigger republish (4 hours in milliseconds)
pub const REPUBLISH_THRESHOLD_MS: u64 = 4 * 60 * 60 * 1000;

/// Identity multihash codec
pub const IDENTITY_CODEC: u64 = 0x00;

/// SHA256 multihash codec
pub const SHA256_CODEC: u64 = 0x12;

/// libp2p-key CID codec
pub const LIBP2P_KEY_CODEC: u64 = 0x72;
