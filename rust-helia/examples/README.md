# Rust-Helia Examples

This directory contains example code demonstrating how to use various components of the Rust-Helia library.

## Examples

### Provider Discovery Examples

#### 1. Basic Find Providers (`basic_find_providers.rs`)

The simplest example showing how to search for content providers.

**Usage:**
```bash
# Use default CID
cargo run --example basic_find_providers

# Or specify a CID
cargo run --example basic_find_providers QmYourCIDHere
```

**What it demonstrates:**
- Creating a libp2p swarm
- Initializing routing
- Finding providers for a CID
- Processing results

---

#### 2. Find Providers (`find_providers.rs`)

A more detailed example with logging and error handling.

**Usage:**
```bash
cargo run --example find_providers
```

**What it demonstrates:**
- Setting up tracing/logging
- Proper error handling
- Timeout management
- Result collection patterns

---

#### 3. Provider Workflow (`provider_workflow.rs`)

Complete workflow showing both provider announcement and discovery.

**Usage:**
```bash
cargo run --example provider_workflow
```

**What it demonstrates:**
- Creating multiple nodes
- Node A announcing it provides content
- Node B searching for that content
- DHT record propagation
- End-to-end provider discovery flow

---

## Current Implementation Status

### ✅ Working

- Creating libp2p swarms with Kademlia DHT
- Initiating provider queries
- Announcing provider records
- Peer discovery
- DHT record storage/retrieval

### ⏳ In Progress

- **Event handling for query results**: Currently, DHT queries are initiated successfully and return QueryId values, but the results aren't yet streamed back from the swarm event loop.

**What this means:**
- All DHT operations are functional at the protocol level
- Queries are properly sent to the DHT
- Event loop integration is needed to collect and return results

### Next Steps

To make the examples return actual provider results, implement event handling in `libp2p_routing.rs`:

1. Add query result collector using channels
2. Spawn task to poll swarm events
3. Filter for relevant query IDs
4. Stream results back to callers

See `LIBP2P_ROUTING_COMPLETE.md` for detailed implementation plan.

---

## Running Examples

All examples can be run with:

```bash
cargo run --example <example_name>
```

For verbose logging:

```bash
RUST_LOG=debug cargo run --example <example_name>
```

---

## Code Structure

Each example follows this pattern:

```rust
use helia_routers::libp2p_routing::libp2p_routing;
use helia_utils::create_swarm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create swarm
    let swarm = create_swarm().await?;
    let swarm = Arc::new(Mutex::new(swarm));
    
    // 2. Create routing
    let routing = libp2p_routing(swarm);
    
    // 3. Use routing methods
    let providers = routing.find_providers(&cid, None).await?;
    
    // 4. Process results
    // ...
}
```

---

## Dependencies

Examples use:
- `helia-routers` - Routing implementations
- `helia-utils` - Swarm creation utilities
- `helia-interface` - Core traits
- `tokio` - Async runtime
- `futures` - Stream processing
- `tracing` - Logging (optional)

---

## Additional Examples Needed

Future examples to add:
- [ ] Peer discovery example
- [ ] DHT record get/put example
- [ ] Custom routing configuration
- [ ] Integration with Helia blockstore
- [ ] HTTP gateway routing example
- [ ] Combined routing strategies

---

## Contributing

When adding new examples:

1. Use descriptive filenames
2. Add comprehensive comments
3. Include usage instructions in file header
4. Update this README
5. Keep examples focused on one concept
6. Add error handling
7. Test the example works

---

## Related Documentation

- [LIBP2P_ROUTING_COMPLETE.md](../LIBP2P_ROUTING_COMPLETE.md) - Detailed routing implementation guide
- [README.md](../README.md) - Main project documentation
- [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) - Overall project status
