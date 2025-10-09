# Custom Libp2p Configuration Example

This example demonstrates how to retrieve JSON data from a specific remote peer using custom libp2p configuration with a swarm key.

## Overview

The example shows:
- Parsing and validating swarm keys (PSK format)
- Parsing multiaddrs and extracting peer IDs
- Creating a Helia instance with JSON support
- Attempting to retrieve JSON content by CID
- Proper error handling and troubleshooting guides

## Running the Example

```bash
cargo run --package helia-json --example custom_libp2p
```

## Configuration

The example is pre-configured with:

- **Remote Node**: `node.cyberfly.io:31001`
- **Peer ID**: `12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw`
- **Swarm Key**: Private network key (32 bytes)
- **Target CID**: `bagaaiera7ggi35jy6tuckbxctbkjuozkcxd33kvfuoc2jc4hp5sxogyez73a`

## Current Behavior

The example currently:
1. ✅ Successfully parses all configuration
2. ✅ Creates a Helia node with default config
3. ✅ Attempts to retrieve JSON via Bitswap
4. ❌ Fails with "No connected peers" (expected)

## Why It Fails (and How to Fix)

### Issue
The example uses the default Helia configuration which:
- Does not include the swarm key for the private network
- Does not automatically connect to the specified peer
- Cannot access content on private networks without proper PSK configuration

### Solution: Implement Custom Transport

To make this work in production, you need to:

#### 1. Add PSK Support Dependencies

```toml
[dependencies]
libp2p = { version = "0.53", features = ["tcp", "noise", "yamux", "pnet"] }
pnet = "0.34"  # For Pre-Shared Key support
```

#### 2. Create PSK-Protected Transport

```rust
use libp2p::pnet::{PnetConfig, PreSharedKey};
use libp2p::tcp::tokio::Transport as TcpTransport;

// Parse swarm key
let swarm_key_bytes = parse_swarm_key(SWARM_KEY)?;
let psk = PreSharedKey::from_bytes(&swarm_key_bytes)?;

// Create PSK config
let psk_config = PnetConfig::new(psk);

// Wrap transport with PSK
let transport = TcpTransport::default()
    .and_then(move |socket, _| psk_config.upgrade_inbound(socket))
    // Add noise, yamux, etc.
```

#### 3. Configure Helia with Custom Transport

```rust
let helia = HeliaBuilder::new()
    .with_libp2p_transport(custom_transport)
    .with_peer_id(local_peer_id)
    .build()
    .await?;
```

#### 4. Add and Connect to Peer

```rust
// Add peer to address book
helia.swarm().add_address(peer_id, multiaddr.clone());

// Dial the peer
helia.swarm().dial(multiaddr)?;

// Wait for connection
// ... wait for ConnectionEstablished event
```

#### 5. Retrieve JSON Content

Once connected, the JSON retrieval will work:

```rust
let json_store = Json::new(helia);
let data: serde_json::Value = json_store.get(&cid, None).await?;
println!("Retrieved: {}", serde_json::to_string_pretty(&data)?);
```

## Understanding Swarm Keys

Swarm keys enable private IPFS networks by:

- **Encrypting all connections** between peers
- **Authenticating peers** before allowing communication
- **Isolating content** to only peers with the correct key

### Format

```
/key/swarm/psk/1.0.0/
/base16/
<64 hex characters = 32 bytes>
```

The example includes a parser that:
- Validates the PSK format
- Extracts the hex portion
- Converts to 32 bytes
- Verifies length

## Troubleshooting

### "No connected peers to request block from"

**Cause**: The node is not connected to any peers that have the content.

**Solutions**:
1. Implement custom PSK transport (see above)
2. Verify the remote peer is online
3. Check network connectivity
4. Ensure swarm key matches

### "Network error"

**Cause**: Bitswap cannot retrieve the content.

**Solutions**:
1. Verify the CID exists on the remote peer
2. Check that content is pinned on remote
3. Ensure Bitswap is properly configured
4. Add retry logic with timeouts

### "Invalid swarm key length"

**Cause**: Swarm key is not exactly 32 bytes.

**Solutions**:
1. Verify the hex string is 64 characters
2. Check for whitespace or formatting issues
3. Use the provided parser function

## Example Output

### Successful Run (with proper PSK implementation):

```
🚀 Helia JSON - Custom libp2p Configuration Example
===================================================

📝 Step 1: Parsing swarm key...
✅ Swarm key parsed successfully
   Key length: 32 bytes

🌐 Step 2: Parsing remote peer address...
✅ Multiaddr parsed: /dns4/node.cyberfly.io/tcp/31001/p2p/12D3...
✅ Peer ID: 12D3KooWA8mwP9wGUc65abVDMuYccaAMAkXhKUqpwKUZSN5McDrw

🎯 Step 3: Parsing target CID...
✅ CID parsed successfully
   CID: bagaaiera7ggi35jy6tuckbxctbkjuozkcxd33kvfuoc2jc4hp5sxogyez73a
   Codec: 0x200 (dag-cbor)
   Version: V1

⚙️  Step 4: Creating Helia instance...
✅ Helia node created with PSK transport

🔗 Step 5: Connecting to remote peer...
✅ Connected to peer!

📄 Step 6: Retrieving JSON content...
🎉 SUCCESS! JSON content retrieved!

📋 JSON Content:
{
  "example": "data",
  "nested": {
    "values": [1, 2, 3]
  }
}
```

## Security Considerations

When implementing PSK support:

1. **Keep swarm keys secret** - Never commit to public repositories
2. **Rotate keys regularly** - Update network-wide when compromised
3. **Use environment variables** - Load keys from secure storage
4. **Validate peer identities** - Ensure peers are authorized
5. **Monitor connections** - Log and audit peer connections

## Next Steps

1. Implement the custom transport with PSK
2. Add automatic peer discovery
3. Implement reconnection logic
4. Add content caching
5. Create a production-ready configuration

## References

- [libp2p PSK Documentation](https://docs.libp2p.io/concepts/security/security/)
- [IPFS Private Networks](https://github.com/ipfs/go-ipfs/blob/master/docs/experimental-features.md#private-networks)
- [Helia Documentation](https://github.com/ipfs/helia)
- [rust-libp2p Examples](https://github.com/libp2p/rust-libp2p/tree/master/examples)

## License

See the workspace LICENSE file.
