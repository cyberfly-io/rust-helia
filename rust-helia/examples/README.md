# Helia Rust Examples

This directory contains practical examples demonstrating how to use Helia Rust for various IPFS operations.

## Running Examples

To run any example, use:

```bash
cargo run --example <example_name>
```

For example:

```bash
cargo run --example 01_basic_node
```

## Available Examples

### 01. Basic Node (`01_basic_node.rs`)

**Demonstrates:**
- Creating a Helia node with default configuration
- Starting and stopping the node
- Accessing node components (blockstore, datastore, pins, routing)

**Run:**
```bash
cargo run --example 01_basic_node
```

### 02. Block Storage (`02_block_storage.rs`)

**Demonstrates:**
- Storing raw blocks of data
- Retrieving blocks by CID
- Checking if blocks exist
- Deleting blocks
- Batch operations with multiple blocks

**Run:**
```bash
cargo run --example 02_block_storage
```

### 03. UnixFS Files (`03_unixfs_files.rs`)

**Demonstrates:**
- Adding files to IPFS using UnixFS
- Reading file content back
- Creating directories
- Adding files to directories
- Listing directory contents
- Getting file statistics
- Chunking large files

**Run:**
```bash
cargo run --example 03_unixfs_files
```

### 04. DAG-CBOR (`04_dag_cbor.rs`)

**Demonstrates:**
- Encoding structured data with DAG-CBOR
- Storing custom Rust types
- Retrieving and decoding data
- Working with nested structures
- Complex data types (structs, vectors, hashmaps)

**Run:**
```bash
cargo run --example 04_dag_cbor
```

### 05. CAR Files (`05_car_files.rs`)

**Demonstrates:**
- Exporting content to CAR (Content Addressable aRchive) files
- Importing content from CAR files
- Round-trip CAR operations
- Exporting directory structures
- Verifying imported content

**Run:**
```bash
cargo run --example 05_car_files
```

**Note:** This example creates CAR files in your system's temp directory.

### 06. Pinning (`06_pinning.rs`)

**Demonstrates:**
- Pinning content to prevent garbage collection
- Checking pin status
- Listing all pinned content
- Unpinning content
- Pinning directories and their contents

**Run:**
```bash
cargo run --example 06_pinning
```

### 07. Custom Configuration (`07_custom_config.rs`)

**Demonstrates:**
- Configuring custom storage paths for blockstore and datastore
- Setting up logging configuration
- Creating custom libp2p keypairs and identities
- Building a complete custom Helia configuration
- Using the configured node

**Run:**
```bash
cargo run --example 07_custom_config
```

**Note:** This example creates a custom storage directory in your system's temp directory.

### 08. JSON Codec (`08_json_codec.rs`)

**Demonstrates:**
- Storing JSON data in IPFS
- Retrieving JSON data by CID
- Working with structured JSON objects
- Serialization and deserialization with serde
- Handling different JSON data types (objects, arrays, raw values)

**Run:**
```bash
cargo run --example 08_json_codec
```

### 09. P2P Content Sharing (`09_p2p_content_sharing.rs`)

**Demonstrates:**
- Peer-to-peer content sharing between two nodes
- mDNS peer discovery on local network
- One node storing content and sharing the CID
- Another node retrieving content using the CID
- Real-world P2P workflow

**Run in Terminal 1 (Store):**
```bash
cargo run --example 09_p2p_content_sharing -- store "Hello from Node 1!"
```

**Run in Terminal 2 (Retrieve):**
```bash
cargo run --example 09_p2p_content_sharing -- get <CID-from-terminal-1>
```

**Example Workflow:**
1. Start the store node in terminal 1 with content
2. Copy the CID from the output
3. Start the retrieve node in terminal 2 with the CID
4. Watch as the second node discovers the first and retrieves the content!

**Note:** Both nodes must be on the same local network for mDNS discovery to work.

## Example Output

Each example includes detailed console output showing:
- Step-by-step progress
- CIDs of created content
- Retrieved data verification
- Statistics and metadata

## Example Structure

All examples follow a similar pattern:

1. **Setup**: Create and configure a Helia node
2. **Operations**: Perform specific IPFS operations
3. **Verification**: Verify the results
4. **Cleanup**: Stop the node and clean up resources

## Dependencies

All examples use:
- `tokio` - Async runtime
- `bytes` - Efficient byte buffers
- `cid` - Content Identifier handling
- `helia` and related crates - Core functionality

## Common Patterns

### Error Handling

All examples use `Result<(), Box<dyn std::error::Error>>` for simple error propagation:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}
```

### Arc Usage

Examples that need to share the Helia instance wrap it in `Arc`:

```rust
let helia = Arc::new(create_helia(None).await?);
let fs = UnixFS::new(helia.clone());
```

### Starting and Stopping

Always start the node before use and stop it when done:

```rust
helia.start().await?;
// ... use the node ...
helia.stop().await?;
```

## Next Steps

After running these examples:

1. Read the [Usage Guide](../USAGE.md) for more detailed documentation
2. Check the [API Reference](../API_REFERENCE.md) for complete API documentation
3. Explore the source code in the `helia-*` crates to understand implementations
4. Try modifying the examples to experiment with different configurations

## Troubleshooting

### Permission Errors

If you get permission errors, ensure you have write access to:
- The current directory (for default storage)
- The system temp directory (for examples that use it)

### Port Already in Use

If you get "address already in use" errors, a previous Helia node might still be running. Wait a moment and try again, or kill any lingering processes.

### Missing Dependencies

If you get compilation errors, ensure you've run:

```bash
cargo build
```

from the workspace root before running examples.

## Contributing Examples

Have an idea for a useful example? Contributions are welcome! Please:

1. Follow the existing example structure
2. Include detailed comments and console output
3. Ensure the example runs successfully
4. Update this README with your example

## License

These examples are part of the Helia Rust project and are dual-licensed under MIT and Apache 2.0.
