//! Bitswap protocol constants
//! Based on @helia/bitswap TypeScript implementation

/// Bitswap protocol version 1.2.0
pub const BITSWAP_120: &str = "/ipfs/bitswap/1.2.0";

/// Bitswap protocol version 1.1.0
pub const BITSWAP_110: &str = "/ipfs/bitswap/1.1.0";

/// Bitswap protocol version 1.0.0
pub const BITSWAP_100: &str = "/ipfs/bitswap/1.0.0";

/// All supported Bitswap protocol versions
pub const BITSWAP_PROTOCOLS: &[&str] = &[BITSWAP_120, BITSWAP_110, BITSWAP_100];

/// Default delay before sending queued messages (milliseconds)
pub const DEFAULT_MESSAGE_SEND_DELAY: u64 = 20;

/// Default want timeout (milliseconds)
pub const DEFAULT_WANT_TIMEOUT: u64 = 30_000;

/// Default priority for want requests
pub const DEFAULT_PRIORITY: i32 = 1;

/// Default maximum number of inbound streams
pub const DEFAULT_MAX_INBOUND_STREAMS: usize = 32;

/// Default maximum number of outbound streams
pub const DEFAULT_MAX_OUTBOUND_STREAMS: usize = 64;

/// Default timeout for receiving messages (milliseconds)
pub const DEFAULT_MESSAGE_RECEIVE_TIMEOUT: u64 = 10_000;

/// Default maximum incoming message size (bytes)
pub const DEFAULT_MAX_INCOMING_MESSAGE_SIZE: usize = 4 * 1024 * 1024; // 4MB

/// Default maximum outgoing message size (bytes)
pub const DEFAULT_MAX_OUTGOING_MESSAGE_SIZE: usize = 4 * 1024 * 1024; // 4MB

/// Maximum block size (bytes)
pub const MAX_BLOCK_SIZE: usize = 2 * 1024 * 1024; // 2MB

/// Default number of concurrent message sends
pub const DEFAULT_MESSAGE_SEND_CONCURRENCY: usize = 32;

/// Default maximum providers per request
pub const DEFAULT_MAX_PROVIDERS_PER_REQUEST: usize = 3;

/// Default whether to run on transient (limited) connections
pub const DEFAULT_RUN_ON_TRANSIENT_CONNECTIONS: bool = false;

/// Default session maximum providers
pub const DEFAULT_SESSION_MAX_PROVIDERS: usize = 5;

/// Default session query concurrency
pub const DEFAULT_SESSION_QUERY_CONCURRENCY: usize = 5;

/// Default session minimum providers
pub const DEFAULT_SESSION_MIN_PROVIDERS: usize = 2;
