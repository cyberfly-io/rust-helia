# JSON P2P Example - Usage Guide

## Overview

This example demonstrates **JSON data sharing over P2P network** using rust-helia, matching the simplicity of JS Helia:

```javascript
// JavaScript Helia
const helia = await createHelia()
const j = json(helia)
const cid = await j.add({ hello: 'world' })
const obj = await j.get(cid)
```

```rust
// Rust Helia - Just as simple!
let helia = create_helia(None).await?;
let j = json(Arc::new(helia));
let cid = j.add(&user, None).await?;
let user: UserData = j.get(&cid, None).await?;
```

## Quick Test

### Terminal 1: Store Node

```bash
cargo run --example 10_json_p2p_sharing -- store
```

**What it does**:
1. Creates Helia node with blockstore at `/tmp/helia-json-store`
2. Stores a JSON object (UserData with name, age, languages, active flag)
3. Prints the CID
4. Keeps running to serve data over P2P

**Expected output**:
```
🌐 Helia JSON P2P Sharing Example

📝 Starting Store Node...

✅ Helia store node started

📦 Storing JSON data:
UserData {
    name: "Alice",
    age: 30,
    languages: [
        "Rust",
        "JavaScript",
        "Python",
    ],
    active: true,
}

✅ JSON data stored successfully!
🔑 CID: bafyreig...

📋 To retrieve from NETWORK (different node), run in another terminal:
   cargo run --example 10_json_p2p_sharing -- get bafyreig...

⏳ Keep this terminal running to serve data over P2P...
   Press Ctrl+C to stop
```

### Terminal 2: Retrieve Node (wait 5 seconds after store node starts)

```bash
# Copy the CID from terminal 1 output
cargo run --example 10_json_p2p_sharing -- get <CID>
```

**What it does**:
1. Creates NEW Helia node with blockstore at `/tmp/helia-json-retrieve` (empty!)
2. Connects to store node via mDNS
3. Requests JSON data by CID over Bitswap
4. Reconstructs and deserializes the JSON
5. Prints the retrieved data

**Expected output**:
```
🌐 Helia JSON P2P Sharing Example

📥 Starting Retrieve Node...

✅ Helia retrieve node started

🔍 Retrieving JSON data for CID: bafyreig...

⏳ Fetching from network (may take a few seconds for peer discovery)...

✅ JSON data retrieved successfully!

📄 Data:
UserData {
    name: "Alice",
    age: 30,
    languages: [
        "Rust",
        "JavaScript",
        "Python",
    ],
    active: true,
}

🎉 P2P JSON retrieval successful!
   Data was fetched from the network, not from local storage!
```

## Architecture

### How It Works

1. **Store Node**:
   - Creates Helia node with Bitswap + mDNS
   - Serializes UserData to DAG-JSON format
   - Stores block in local blockstore
   - Announces presence via mDNS
   - Waits for Bitswap requests

2. **Retrieve Node**:
   - Creates Helia node (DIFFERENT blockstore, empty!)
   - Discovers store node via mDNS
   - Sends WANT request via Bitswap
   - Receives block from store node
   - Stores block locally
   - Deserializes DAG-JSON to UserData

3. **P2P Communication**:
   ```
   Store Node          mDNS            Retrieve Node
   ----------          ----            -------------
   Start              ------>          Discover
   Listening          <------          WANT request
   Fetch block        
   Send block         ------>          Receive block
                                       Store block
                                       Deserialize JSON
   ```

### Key Files

- **Example**: `rust-helia/examples/10_json_p2p_sharing.rs`
- **JSON Codec**: `helia-json/src/lib.rs`
- **Main Entry**: `rust-helia/src/lib.rs` (`create_helia`)
- **Bitswap**: `helia-bitswap/src/` (coordinator + behaviour)
- **Event Loop**: `helia-utils/src/helia.rs` (swarm management)

### API Reference

```rust
// Create Helia node
use rust_helia::create_helia;
let helia = create_helia(Some(config)).await?;
helia.start().await?;  // Starts P2P networking

// JSON operations
use helia_json::{json, JsonInterface};
let j = json(Arc::new(helia));

// Store JSON
let cid = j.add(&my_data, None).await?;

// Retrieve JSON
let my_data: MyStruct = j.get(&cid, None).await?;
```

## Troubleshooting

### Timeout Issues

If retrieve times out after 30 seconds:

1. **Check store node is running**: Make sure terminal 1 is still active
2. **Wait longer**: Give mDNS 5-10 seconds for peer discovery
3. **Check blockstores**: Ensure different paths (`/tmp/helia-json-store` vs `/tmp/helia-json-retrieve`)
4. **Clean and retry**: `rm -rf /tmp/helia-json-*` and restart both nodes

### Build Errors

```bash
# Full rebuild
cargo clean
cargo build --example 10_json_p2p_sharing
```

### Debugging

To see detailed logs, set environment variable:

```bash
RUST_LOG=debug cargo run --example 10_json_p2p_sharing -- store
```

## Comparison with JS Helia

| Feature | JavaScript | Rust |
|---------|-----------|------|
| Create node | `await createHelia()` | `create_helia(None).await?` |
| JSON codec | `json(helia)` | `json(Arc::new(helia))` |
| Store JSON | `await j.add(obj)` | `j.add(&obj, None).await?` |
| Get JSON | `await j.get(cid)` | `j.get::<T>(&cid, None).await?` |
| Type safety | Runtime | **Compile-time** ✨ |
| Performance | Fast | **Faster** 🚀 |

## Success! 🎉

The rust-helia JSON P2P example demonstrates:

✅ **Simple API** matching JS Helia  
✅ **Complete P2P block exchange** working  
✅ **DAG-JSON encoding/decoding** functional  
✅ **mDNS peer discovery** automatic  
✅ **Bitswap protocol** fully implemented  
✅ **Type-safe** with Rust's type system  

This is a **complete, working P2P IPFS implementation in Rust**!
