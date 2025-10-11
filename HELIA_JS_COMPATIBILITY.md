# Helia.js Interface Compatibility

This document explains how rust-helia's interface compares to the official Helia.js interface and design decisions made for Rust.

## Helia.js Interface (v6.0.0+)

```typescript
export interface Helia<T extends Libp2p = Libp2p> {
  libp2p: T                          // Direct property
  blockstore: Blocks                 // Direct property
  datastore: Datastore               // Direct property
  pins: Pins                         // Direct property
  logger: ComponentLogger            // Direct property
  routing: Routing                   // Direct property
  dns: DNS                           // Direct property
  metrics?: Metrics                  // Optional property
  events: TypedEventEmitter<Events>  // Event emitter
  
  start(): Promise<void>
  stop(): Promise<void>
  gc(options?: GCOptions): Promise<void>
  getCodec: CodecLoader              // Function property
  getHasher: HasherLoader            // Function property
}
```

## rust-helia Interface Design

### Current Design

```rust
// Base trait (similar to Helia.js v4.0 when libp2p was removed)
pub trait Helia: Send + Sync {
    fn blockstore(&self) -> &dyn Blocks;
    fn datastore(&self) -> &dyn Datastore;
    fn pins(&self) -> &dyn Pins;
    fn logger(&self) -> &dyn ComponentLogger;
    fn routing(&self) -> &dyn Routing;
    fn dns(&self) -> &TokioAsyncResolver;
    fn metrics(&self) -> Option<&dyn Metrics>;
    
    async fn start(&self) -> Result<(), HeliaError>;
    async fn stop(&self) -> Result<(), HeliaError>;
    async fn gc(&self, options: Option<GcOptions>) -> Result<(), HeliaError>;
    async fn get_codec(&self, code: u64) -> Result<Box<dyn Codec>, HeliaError>;
    async fn get_hasher(&self, code: u64) -> Result<Box<dyn Hasher>, HeliaError>;
}

// Generic trait with libp2p (similar to Helia.js v6.0+)
pub trait HeliaWithLibp2p<T>: Helia
where
    T: libp2p::swarm::NetworkBehaviour + Send + 'static,
{
    fn libp2p(&self) -> Arc<Mutex<Swarm<T>>>;
}
```

## Key Differences & Rationale

### 1. **libp2p in Separate Trait**

**Helia.js:** Direct generic parameter `Helia<T extends Libp2p>`

**rust-helia:** Separate trait `HeliaWithLibp2p<T>`

**Rationale:**
- Rust doesn't support "property syntax" like TypeScript - everything is a method
- Generic parameters on traits prevent using them as trait objects (`Box<dyn Helia>`)
- The split design allows:
  - `Helia` trait objects for dynamic dispatch
  - `HeliaWithLibp2p<T>` for concrete implementations with static dispatch
  - Future support for non-libp2p implementations (e.g., `@helia/http` equivalent)

**History:** Helia.js made the same trade-off in v4.0.0 (removed libp2p from base interface) but added it back in v6.0.0 as a generic parameter since TypeScript handles generics differently.

### 2. **Methods Instead of Properties**

**Helia.js:** `helia.blockstore` (property access)

**rust-helia:** `helia.blockstore()` (method call)

**Rationale:**
- Rust doesn't have a "property" concept like TypeScript
- Methods provide:
  - Flexibility to return references (`&dyn Blocks`)
  - Ability to perform logic/validation if needed
  - Consistent API patterns with other Rust libraries

### 3. **Thread Safety**

**Helia.js:** No explicit thread safety (single-threaded JavaScript)

**rust-helia:** 
- `Arc<Mutex<Swarm<T>>>` for thread-safe libp2p access
- `Send + Sync` bounds on all traits

**Rationale:**
- Rust is multi-threaded by default
- IPFS operations often need concurrent access
- `Arc<Mutex<>>` provides safe shared ownership and mutation

### 4. **Events System** ✅ **IMPLEMENTED**

**Helia.js:** `events: TypedEventEmitter<HeliaEvents<T>>`

**rust-helia:** `fn subscribe_events(&self) -> HeliaEventReceiver`

**Rationale:**
- Uses `tokio::sync::broadcast` for efficient multi-subscriber event distribution
- Returns a receiver that can be cloned for multiple listeners
- Events include: `Start`, `Stop`, `GcStarted`, `GcCompleted`
- Buffer size of 100 events to prevent slow consumers from blocking

**Example:**
```rust
let mut events = helia.subscribe_events();
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        println!("Event: {:?}", event);
    }
});
```

See [EVENTS_EXAMPLE.md](./EVENTS_EXAMPLE.md) for detailed usage examples.

### 5. **Error Handling**

**Helia.js:** Throws exceptions / returns rejected Promises

**rust-helia:** Returns `Result<T, HeliaError>`

**Rationale:**
- Rust's `Result` type provides explicit error handling
- Follows Rust ecosystem conventions
- Allows `?` operator for ergonomic error propagation

## Migration Guide from Helia.js

### JavaScript/TypeScript → Rust

```javascript
// Helia.js
const helia = await createHelia();
const block = await helia.blockstore.get(cid);
console.log(helia.libp2p.getPeers());
```

```rust
// rust-helia (base trait)
let helia = HeliaImpl::new(config).await?;
let block = helia.blockstore().get(&cid, None).await?;

// rust-helia (with libp2p)
use helia_interface::HeliaWithLibp2p;
let swarm = helia.libp2p();
let mut swarm_guard = swarm.lock().await;
let peers = swarm_guard.connected_peers().collect::<Vec<_>>();
```

## Alignment Roadmap

### ✅ Completed
- [x] Base `Helia` trait with all core properties/methods
- [x] `HeliaWithLibp2p<T>` trait for libp2p access
- [x] Thread-safe implementation with Arc/Mutex
- [x] `Blocks`, `Pins`, `Routing` traits
- [x] Error handling with `HeliaError`
- [x] Events system with `subscribe_events()` method
- [x] Event types: Start, Stop, GcStarted, GcCompleted

### 🔄 In Progress
- [ ] Complete metrics implementation
- [ ] Full garbage collection implementation

### 📋 Future Work
- [ ] Non-libp2p implementation (like `@helia/http`)
- [ ] DAG walker registration system
- [ ] Progress event system (block-level events)
- [ ] Stream-based APIs where appropriate
- [ ] Additional event types (BlockAdded, PeerConnected, etc.)

## Why Not Match Exactly?

While we strive for API compatibility with Helia.js, some differences are **necessary** due to:

1. **Language Differences**: Rust's ownership, borrowing, and type system require different patterns
2. **Concurrency Model**: Rust's multi-threaded model vs JavaScript's single-threaded event loop
3. **Type System**: Rust's trait system vs TypeScript's structural typing
4. **Ergonomics**: Following Rust ecosystem conventions makes the library more familiar to Rust developers

## Philosophy

**rust-helia aims for *conceptual compatibility* rather than *syntactic compatibility*.**

Users familiar with Helia.js should find the same concepts and capabilities in rust-helia, even if the syntax differs. The goal is to feel "Rust-native" while maintaining the same mental model as Helia.js.

## References

- [Helia.js Interface Source](https://github.com/ipfs/helia/blob/main/packages/interface/src/index.ts)
- [Helia.js v4.0.0 Breaking Changes](https://github.com/ipfs/helia/blob/main/packages/interface/CHANGELOG.md#400-2024-01-24) - Removed libp2p from interface
- [Helia.js v6.0.0 Changes](https://github.com/ipfs/helia/blob/main/packages/interface/CHANGELOG.md#600-2025-10-09) - Updated interface design
- [Rust async-trait documentation](https://docs.rs/async-trait/)
- [Rust trait objects guide](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
