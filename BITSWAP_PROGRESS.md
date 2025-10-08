# Bitswap Implementation Progress

## Overview

This document tracks the progress of the Bitswap protocol implementation, which is being rewritten to match the architecture of [@helia/bitswap](https://github.com/ipfs/helia/tree/main/packages/bitswap) from TypeScript.

**Status**: 🔄 In Progress (75% Complete)  
**Reference**: TypeScript implementation at https://github.com/ipfs/helia/tree/main/packages/bitswap

## Architecture

The Bitswap implementation consists of the following components:

```
helia-bitswap/
├── constants.rs          ✅ Complete - Protocol constants
├── pb.rs                ✅ Complete - Protocol Buffer definitions
├── utils.rs             ✅ Complete - Message utilities
├── network_new.rs       ✅ Complete - Network layer
├── wantlist_new.rs      ✅ Complete - WantList manager
├── peer_want_lists.rs   ✅ Complete - Peer wantlist tracking
├── session.rs           🔄 Legacy - Needs rewrite
└── lib.rs               ✅ Clean - Legacy modules removed

Removed legacy files (replaced by new architecture):
├── message.rs           ❌ REMOVED
├── network.rs           ❌ REMOVED
├── peer_manager.rs      ❌ REMOVED
├── stats.rs            ❌ REMOVED
└── wantlist.rs         ❌ REMOVED
```

## Completed Components (✅)

### 1. Constants Module (`constants.rs`)
**Status**: ✅ Complete  
**Lines**: ~50

**Features**:
- Protocol version strings (`BITSWAP_120`, `BITSWAP_110`, `BITSWAP_100`)
- Message size limits (4MB max incoming/outgoing, 2MB max block)
- Timeout configuration (10s receive timeout, 20ms send delay)
- Stream limits (32 inbound, 64 outbound)
- Concurrency settings (8 message send concurrency)

**Reference**: Matches TypeScript `src/constants.ts`

### 2. Protocol Buffer Definitions (`pb.rs`)
**Status**: ✅ Complete  
**Lines**: ~150

**Features**:
- Uses `prost` crate for Protocol Buffer encoding/decoding
- `WantType` enum: `WantBlock`, `WantHave`
- `BlockPresenceType` enum: `HaveBlock`, `DoNotHaveBlock`
- Message structures: `WantlistEntry`, `Wantlist`, `Block`, `BlockPresence`, `BitswapMessage`
- Helper methods: `encode_to_vec()`, `decode_from_bytes()`, `is_empty()`, `estimated_size()`

**Reference**: Matches TypeScript `src/pb/` directory

### 3. Message Utilities (`utils.rs`)
**Status**: ✅ Complete  
**Lines**: ~280

**Features**:
- `QueuedBitswapMessage`: HashMap-based incremental message builder
  - `add_want_block(cid, priority)` - Request full block
  - `add_want_have(cid, priority)` - Request block presence only
  - `add_cancel(cid)` - Cancel previous want
  - `add_block(cid, prefix, data)` - Add block data to message
  - `add_block_presence(cid, presence_type)` - Add block presence info
- `merge_messages(base, other)` - Combine two queued messages
- `split_message(message, max_size)` - Split large messages for wire protocol
- `cid_to_prefix(cid)` - Extract CID prefix (version, codec, hash info)

**Reference**: Matches TypeScript `src/utils.ts`

### 4. Network Layer (`network_new.rs`)
**Status**: ✅ Complete (Foundation)  
**Lines**: ~350

**Features**:
- `Network` struct with event-driven architecture
- Configuration via `NetworkInit` with sensible defaults
- Event types: `BitswapMessage`, `PeerConnected`, `PeerDisconnected`
- Methods:
  - `start()`, `stop()` - Lifecycle management
  - `send_message(peer, message)` - Send with automatic queue merging
  - `handle_incoming_stream(peer, data)` - Process received messages
  - `next_event()` - Async event stream
  - `find_providers(cid)`, `find_and_connect(cid)`, `connect_to(peer)` - Routing (stubs)
- Send queue with message merging per peer
- mpsc channel-based event system

**Reference**: Matches TypeScript `src/network.ts`

**Pending**:
- libp2p stream handling integration
- Actual protocol negotiation
- Integration with helia-utils libp2p behavior

### 5. WantList Manager (`wantlist_new.rs`)
**Status**: ✅ Complete (Foundation)  
**Lines**: ~400

**Features**:
- `WantList` manager for tracking desired blocks
- Methods:
  - `want_block(cid, priority)` - Request block from any peer
  - `want_session_block(cid, peer, priority)` - Request from specific peer
  - `received_block(cid)` - Notify when block arrives
  - `get_wantlist()` - Get current wants
- Automatic message sending to all connected peers
- Timeout handling (30s) for wants
- oneshot channels for async result notification
- Separate tracking for session wants (peer-specific) and global wants

**Reference**: Matches TypeScript `src/want-list.ts`

**Pending**:
- Integration with blockstore for storing received blocks
- Better error handling and retry logic
- Message batching optimization

### 6. Peer WantLists (`peer_want_lists.rs`)
**Status**: ✅ Complete  
**Lines**: ~370

**Features**:
- `PeerWantLists` manager for tracking what each peer wants from us
- Methods:
  - `add_peer(peer)`, `remove_peer(peer)` - Peer lifecycle
  - `add_want(peer, cid, priority, want_type, send_dont_have)` - Track peer wants
  - `remove_want(peer, cid)` - Remove want
  - `has_want(peer, cid)`, `wants_block(peer, cid)`, `wants_have(peer, cid)` - Queries
  - `get_peers_wanting(cid)`, `get_peers_wanting_block(cid)` - Reverse lookup
  - `received_block(cid)` - Get peers to notify
  - `create_block_messages(cid, data)` - Create messages for all interested peers
  - `create_dont_have_messages(cid)` - Create "don't have" messages
  - `stats()` - Get statistics (num peers, total wants)

**Reference**: Matches TypeScript `src/peer-want-lists/` directory

## In Progress Components (🔄)

### 7. Session Manager (`session.rs`)
**Status**: 🔄 Needs Rewrite  
**Current**: Legacy implementation exists but doesn't match architecture

**Planned Features** (based on TypeScript `src/session.ts`):
- Abstract session interface for block retrieval
- Provider discovery and querying
- Session-based want tracking separate from global wants
- Provider rotation on failures
- Session timeout management
- Methods:
  - `new(network, wantlist, providers)` - Create session
  - `retrieve(cid, options)` - Retrieve block via this session
  - `add_provider(peer)` - Add provider to session
  - `close()` - Clean up session

**Reference**: TypeScript `src/session.ts` with `AbstractSession`

### 8. Main Bitswap Coordinator
**Status**: 🔄 Not Started  
**File**: Will be added to `lib.rs` or new `bitswap.rs`

**Planned Features** (based on TypeScript `src/index.ts`):
- `Bitswap` struct coordinating all components
- Methods:
  - `new(blockstore, network, wantlist, peer_want_lists)` - Create coordinator
  - `start()`, `stop()` - Lifecycle
  - `want(cid, options)` - High-level block request
  - `notify(cid, block)` - Announce block availability
  - `get_wantlist()` - Get current wants
  - `get_peer_wantlist(peer)` - Get what peer wants from us

**Reference**: TypeScript `src/index.ts`

## Pending Components (⏳)

### 9. libp2p Integration
**Status**: ⏳ Planned  
**Location**: `helia-utils/src/libp2p_behaviour.rs` or new file

**Planned Features**:
- `BitswapBehaviour` implementing libp2p `NetworkBehaviour` trait
- Stream protocol negotiation (bitswap/1.2.0, bitswap/1.1.0, bitswap/1.0.0)
- Wire protocol handling (send/receive messages)
- Connection lifecycle integration
- Integration points:
  - Connect Network::handle_incoming_stream to libp2p streams
  - Connect Network::send_message to libp2p stream writing
  - Forward peer connect/disconnect events to Network

### 10. Example Update
**Status**: ⏳ Planned  
**File**: `examples/09_p2p_content_sharing.rs`

**Current**: Uses shared blockstore workaround at `/tmp/helia-p2p-demo`

**Planned Changes**:
- Remove shared blockstore workaround
- Use real Bitswap protocol:
  - Store: `helia.blockstore().put()` + `bitswap.notify()`
  - Retrieve: `bitswap.want(cid)`
- Demonstrate true P2P block exchange
- Better error handling and progress reporting

### 11. Testing
**Status**: ⏳ Planned

**Planned Tests**:
- Unit tests for each component:
  - Message encoding/decoding (pb.rs)
  - Message merging and splitting (utils.rs)
  - Want tracking (wantlist_new.rs)
  - Peer want tracking (peer_want_lists.rs)
- Integration tests:
  - Two-node block exchange
  - Multiple peer scenarios
  - Session-based retrieval
  - Provider rotation on failures
- Performance tests:
  - Large message handling
  - High peer count
  - Concurrent block requests

## Recent Updates

### ✅ Cleanup Complete (Latest)

**Removed Legacy Modules**:
- ❌ `message.rs` - Replaced by `pb.rs` and `utils.rs`
- ❌ `network.rs` - Replaced by `network_new.rs`
- ❌ `peer_manager.rs` - Replaced by `peer_want_lists.rs`
- ❌ `stats.rs` - Will be integrated into new architecture
- ❌ `wantlist.rs` - Replaced by `wantlist_new.rs`

**Benefits**:
- Cleaner codebase with single source of truth
- Reduced compilation warnings (38 → 19)
- Eliminated confusion between old and new implementations
- Smaller binary size

## Compilation Status

✅ **All modules compile successfully**

```bash
$ cargo check -p helia-bitswap
   Compiling helia-bitswap v0.1.2
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.24s
```

```bash
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.29s
```

Warnings: 19 warnings (unused code, will be addressed)  
Errors: 0

## Next Steps

1. **Session Manager Rewrite** (Priority: High)
   - Review TypeScript `session.ts` implementation
   - Create `session_new.rs` matching architecture
   - Implement provider discovery and rotation
   - Add tests

2. **Main Coordinator** (Priority: High)
   - Create main `Bitswap` struct
   - Implement high-level API (`want`, `notify`)
   - Wire up all components
   - Add lifecycle management

3. **libp2p Integration** (Priority: High)
   - Create `BitswapBehaviour` 
   - Implement `NetworkBehaviour` trait
   - Connect to Network layer
   - Test stream handling

4. **Example Update** (Priority: Medium)
   - Remove shared blockstore workaround
   - Use real Bitswap protocol
   - Add logging and error handling
   - Create detailed usage docs

5. **Testing** (Priority: Medium)
   - Write unit tests for all components
   - Create integration tests
   - Add performance benchmarks

6. **Cleanup** (Priority: Low)
   - ✅ Remove legacy files (message.rs, old network.rs, etc.) - COMPLETED
   - Address remaining compilation warnings
   - Update documentation

## Timeline Estimate

- **Session Manager**: 2-3 days
- **Main Coordinator**: 2-3 days  
- **libp2p Integration**: 3-5 days
- **Example Update**: 1 day
- **Testing**: 3-5 days
- ~~**Cleanup**: 1-2 days~~ ✅ COMPLETED

**Total**: 11-17 days remaining for complete implementation (was 12-19 days)

## References

- TypeScript Implementation: https://github.com/ipfs/helia/tree/main/packages/bitswap
- Bitswap Specification: https://github.com/ipfs/specs/blob/main/BITSWAP.md
- libp2p rust-libp2p: https://github.com/libp2p/rust-libp2p
- Protocol Buffers with prost: https://github.com/tokio-rs/prost

## Notes

- The new architecture uses `prost` for Protocol Buffers instead of custom encoding
- Event-driven design with tokio mpsc channels for better async integration
- HashMap-based message building for efficient incremental construction
- Separate tracking of global wants vs session wants for better provider management
- All new modules follow Rust best practices (Arc<RwLock<T>> for shared state, oneshot channels for results)
