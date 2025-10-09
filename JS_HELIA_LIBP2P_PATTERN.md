# JS Helia libp2p Integration Pattern

## Key Findings from JS Implementation

### 1. createHelia Accepts libp2p Instance or Config

**From `packages/utils/src/index.ts`**:
```typescript
export interface HeliaInit<T extends Libp2p = Libp2p> {
  /**
   * A libp2p node is required to perform network operations. Either a
   * pre-configured node or options to configure a node can be passed
   * here.
   *
   * If node options are passed, they will be merged with the default
   * config for the current platform. In this case all passed config
   * keys will replace those from the default config.
   *
   * The libp2p `start` option is not supported, instead please pass `start` in
   * the root of the HeliaInit object.
   */
  libp2p: T | Omit<Libp2pOptions<any>, 'start'>
  
  // ... other options
}
```

### 2. Helia Class Stores libp2p Reference

**From `packages/utils/src/index.ts`**:
```typescript
export class Helia<T extends Libp2p> implements HeliaInterface<T> {
  public libp2p: T
  public blockstore: BlockStorage
  public datastore: Datastore
  // ... other fields

  constructor (init: Omit<HeliaInit, 'start' | 'libp2p'> & { libp2p: T }) {
    this.libp2p = init.libp2p
    // ... other initialization
  }
}
```

### 3. Usage Examples

**Example 1: Pass Existing libp2p Node**
```typescript
import { createHelia } from 'helia'
import { createLibp2p } from 'libp2p'

const libp2p = await createLibp2p({
  // custom config
})

const helia = await createHelia({
  libp2p  // Pass the instance
})
```

**Example 2: Pass libp2p Options**
```typescript
const helia = await createHelia({
  libp2p: {
    addresses: {
      listen: ['/ip4/0.0.0.0/tcp/0']
    },
    transports: [...],
    services: {
      identify: identify(),
      // ... custom services
    }
  }
})
```

**Example 3: With Bitswap (from benchmarks)**
```typescript
const libp2p = await createLibp2p({
  // ... config
})

const helia = await createHelia({
  libp2p,
  blockBrokers: [
    bitswap()  // Bitswap uses libp2p from helia
  ],
  routers: [
    libp2pRouting(libp2p)
  ]
})
```

### 4. Bitswap Integration Pattern

**From `benchmarks/transports/src/runner/helia/get-helia.ts`**:
```typescript
export async function getHelia (): Promise<Helia<Libp2p<any>>> {
  const libp2p = await createLibp2p({
    // ... configuration
    services: {
      identify: identify()
    }
  })

  return createHelia({
    libp2p,
    blockBrokers: [
      bitswap()  // ← Bitswap gets libp2p from Helia
    ],
    routers: [
      libp2pRouting(libp2p)  // ← Routing uses libp2p
    ]
  })
}
```

### 5. Bitswap Uses Helia's libp2p

**Key Pattern**: Bitswap doesn't manage libp2p directly. Instead:
1. Helia stores the libp2p instance
2. Bitswap is created as a "block broker"
3. Bitswap receives the libp2p instance from Helia's components

**From block-brokers pattern**:
```typescript
// Block broker function receives components from Helia
export function bitswap() {
  return (components: any) => {
    // components.libp2p is available here
    // components.blockstore is available here
    return createBitswapBroker(components)
  }
}
```

## Applying This to Rust Implementation

### Current Rust Architecture

**Problem**: We have separate initialization:
```rust
// Create Helia
let helia = create_helia(config).await?;

// Create Bitswap separately
let bitswap = Bitswap::new(config)?;

// They don't know about each other's libp2p/swarm!
```

### Solution: Follow JS Pattern

**Option 1: Pass Swarm to Helia** (Recommended)
```rust
// Create swarm first
let swarm = create_swarm().await?;

// Pass swarm to Helia (like JS passes libp2p)
let helia = HeliaBuilder::new()
    .with_swarm(swarm)  // ← Store swarm reference
    .with_blockstore(blockstore)
    .with_datastore(datastore)
    .build()
    .await?;

// Bitswap gets swarm from Helia
// (via block brokers pattern)
```

**Option 2: Helia Creates and Manages Swarm**
```rust
// Helia creates swarm internally
let helia = HeliaBuilder::new()
    .with_libp2p_config(libp2p_config)  // ← Pass config
    .with_blockstore(blockstore)
    .build()
    .await?;

// Access swarm when needed
helia.swarm()  // Returns Arc<Mutex<Swarm<HeliaBehaviour>>>
```

**Option 3: Component-Based** (Most flexible)
```rust
// Create shared components
let components = Components {
    blockstore: Arc::new(blockstore),
    datastore: Arc::new(datastore),
    swarm: Arc::new(Mutex::new(swarm)),
};

// Helia uses components
let helia = Helia::new(components.clone()).await?;

// Bitswap uses same components
let bitswap = Bitswap::new(components.clone()).await?;

// Both share the same swarm!
```

## Recommended Architecture

### 1. HeliaInit Structure

```rust
pub struct HeliaInit {
    /// Either a pre-configured swarm or config to create one
    pub swarm: Either<Swarm<HeliaBehaviour>, SwarmConfig>,
    
    /// Blockstore for storing blocks
    pub blockstore: Arc<dyn Blocks>,
    
    /// Datastore for metadata
    pub datastore: Arc<dyn Datastore>,
    
    /// Block brokers (including Bitswap)
    pub block_brokers: Vec<Box<dyn BlockBroker>>,
    
    /// Start immediately or not
    pub start: bool,
}
```

### 2. Helia Structure

```rust
pub struct Helia {
    /// libp2p swarm (like JS stores libp2p)
    swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>,
    
    /// Blockstore
    blockstore: Arc<dyn Blocks>,
    
    /// Datastore  
    datastore: Arc<dyn Datastore>,
    
    /// Event loop handle
    event_loop: Option<JoinHandle<()>>,
}

impl Helia {
    pub fn swarm(&self) -> &Arc<Mutex<Swarm<HeliaBehaviour>>> {
        &self.swarm
    }
    
    pub async fn start(&mut self) -> Result<()> {
        // Start swarm event loop
        let swarm = self.swarm.clone();
        let handle = tokio::spawn(async move {
            run_swarm_event_loop(swarm).await;
        });
        self.event_loop = Some(handle);
        Ok(())
    }
}
```

### 3. Bitswap Integration

```rust
// Block broker pattern (like JS)
pub fn bitswap() -> Box<dyn BlockBrokerFactory> {
    Box::new(|components: &Components| -> Box<dyn BlockBroker> {
        Box::new(BitswapBroker::new(
            components.swarm.clone(),
            components.blockstore.clone(),
        ))
    })
}

pub struct BitswapBroker {
    swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>,
    blockstore: Arc<dyn Blocks>,
    coordinator: Arc<Bitswap>,
}

impl BlockBroker for BitswapBroker {
    async fn retrieve(&self, cid: &Cid) -> Result<Bytes> {
        // Use swarm to request block
        let mut swarm = self.swarm.lock().await;
        swarm.behaviour_mut()
            .bitswap
            .send_want_message(cid)
            .await
    }
}
```

### 4. Usage Pattern

```rust
// User code (similar to JS)
use helia::create_helia;
use helia_bitswap::bitswap;

let helia = create_helia(HeliaInit {
    swarm: Either::Right(SwarmConfig::default()),  // Or pass existing swarm
    blockstore: Arc::new(MemoryBlockstore::new()),
    datastore: Arc::new(MemoryDatastore::new()),
    block_brokers: vec![
        bitswap(),  // ← Bitswap broker
    ],
    start: true,
}).await?;

// Bitswap automatically integrated!
// Swarm event loop running
// P2P block exchange enabled
```

## Benefits of This Approach

1. **Matches JS Architecture**: Familiar to JS developers
2. **Single Swarm Instance**: No duplication
3. **Automatic Integration**: Block brokers connect automatically
4. **Flexible**: Can pass custom swarm or let Helia create it
5. **Event Loop Management**: Helia manages swarm lifecycle
6. **Clean Separation**: Bitswap is just a block broker plugin

## Implementation Steps

1. ✅ NetworkBehaviour implemented (done)
2. ⏸️ Add swarm field to Helia struct
3. ⏸️ Create HeliaInit with swarm option
4. ⏸️ Implement BlockBroker trait for Bitswap
5. ⏸️ Create swarm event loop in Helia::start()
6. ⏸️ Update create_helia() to accept swarm
7. ⏸️ Test P2P block exchange

## Next Action

Modify `helia-utils/src/helia.rs` to:
1. Add `swarm: Arc<Mutex<Swarm<HeliaBehaviour>>>` field
2. Accept swarm in initialization
3. Start swarm event loop
4. Connect Bitswap to swarm via event handling
