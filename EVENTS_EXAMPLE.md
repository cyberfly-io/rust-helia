# Event System Example

This example demonstrates how to subscribe to and handle Helia events using the event broadcasting system.

## Overview

The Helia event system provides a way to monitor node lifecycle events:
- **Start**: Emitted when the node starts
- **Stop**: Emitted when the node stops  
- **GcStarted**: Emitted when garbage collection begins
- **GcCompleted**: Emitted when garbage collection completes

## Basic Usage

```rust
use helia_interface::{Helia, HeliaEvent};
use helia_utils::{HeliaConfig, HeliaImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Helia instance
    let config = HeliaConfig::default();
    let helia = HeliaImpl::new(config).await?;
    
    // Subscribe to events BEFORE starting the node
    let mut events_rx = helia.subscribe_events();
    
    // Spawn a task to handle events
    tokio::spawn(async move {
        while let Ok(event) = events_rx.recv().await {
            match event {
                HeliaEvent::Start => {
                    println!("🚀 Helia node started");
                }
                HeliaEvent::Stop => {
                    println!("🛑 Helia node stopped");
                }
                HeliaEvent::GcStarted => {
                    println!("🗑️  Garbage collection started");
                }
                HeliaEvent::GcCompleted => {
                    println!("✅ Garbage collection completed");
                }
            }
        }
        println!("Event listener stopped");
    });
    
    // Start the node (emits Start event)
    helia.start().await?;
    
    // Do some work...
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Run garbage collection (emits GcStarted and GcCompleted events)
    helia.gc(None).await?;
    
    // Stop the node (emits Stop event)
    helia.stop().await?;
    
    // Give the event handler time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(())
}
```

## Multiple Subscribers

You can have multiple subscribers listening to the same events:

```rust
use helia_interface::{Helia, HeliaEvent};
use helia_utils::{HeliaConfig, HeliaImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = HeliaConfig::default();
    let helia = HeliaImpl::new(config).await?;
    
    // First subscriber: logs events
    let mut logger_rx = helia.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = logger_rx.recv().await {
            println!("[Logger] Event: {:?}", event);
        }
    });
    
    // Second subscriber: tracks metrics
    let mut metrics_rx = helia.subscribe_events();
    tokio::spawn(async move {
        let mut start_count = 0;
        let mut stop_count = 0;
        let mut gc_count = 0;
        
        while let Ok(event) = metrics_rx.recv().await {
            match event {
                HeliaEvent::Start => start_count += 1,
                HeliaEvent::Stop => stop_count += 1,
                HeliaEvent::GcCompleted => gc_count += 1,
                _ => {}
            }
            println!("[Metrics] Starts: {}, Stops: {}, GCs: {}", 
                     start_count, stop_count, gc_count);
        }
    });
    
    // Third subscriber: saves to database (example)
    let mut db_rx = helia.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = db_rx.recv().await {
            // In a real app, you'd save to a database here
            println!("[Database] Saving event: {:?}", event);
        }
    });
    
    helia.start().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    helia.gc(None).await?;
    helia.stop().await?;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(())
}
```

## Filtering Events

You can filter for specific events you care about:

```rust
use helia_interface::{Helia, HeliaEvent};
use helia_utils::{HeliaConfig, HeliaImpl};

async fn watch_gc_events(helia: &HeliaImpl) {
    let mut events_rx = helia.subscribe_events();
    
    tokio::spawn(async move {
        while let Ok(event) = events_rx.recv().await {
            // Only handle GC events
            match event {
                HeliaEvent::GcStarted => {
                    println!("GC started at: {:?}", std::time::SystemTime::now());
                }
                HeliaEvent::GcCompleted => {
                    println!("GC completed at: {:?}", std::time::SystemTime::now());
                }
                _ => {} // Ignore other events
            }
        }
    });
}
```

## Error Handling

If all subscribers disconnect, the sender will return an error when trying to send events. This is handled gracefully in the implementation:

```rust
// Inside HeliaImpl::start()
// This will not panic if there are no subscribers
let _ = self.event_tx.send(HeliaEvent::Start);
```

If you want to be notified when the event channel closes:

```rust
use helia_interface::{Helia, HeliaEvent};
use tokio::sync::broadcast::error::RecvError;

async fn handle_events_with_errors(helia: &impl Helia) {
    let mut events_rx = helia.subscribe_events();
    
    loop {
        match events_rx.recv().await {
            Ok(event) => {
                println!("Event: {:?}", event);
            }
            Err(RecvError::Lagged(skipped)) => {
                // Too many events, some were skipped
                println!("Warning: Skipped {} events (buffer full)", skipped);
            }
            Err(RecvError::Closed) => {
                // Channel closed, sender dropped
                println!("Event channel closed");
                break;
            }
        }
    }
}
```

## Comparison with Helia.js

In Helia.js, events work like this:

```javascript
// Helia.js (TypeScript)
const helia = await createHelia();

helia.events.addEventListener('start', () => {
  console.log('Helia started');
});

helia.events.addEventListener('stop', () => {
  console.log('Helia stopped');
});

await helia.start();
```

In rust-helia, the equivalent is:

```rust
// rust-helia (Rust)
let helia = HeliaImpl::new(config).await?;

let mut events = helia.subscribe_events();
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match event {
            HeliaEvent::Start => println!("Helia started"),
            HeliaEvent::Stop => println!("Helia stopped"),
            _ => {}
        }
    }
});

helia.start().await?;
```

## Performance Notes

- The event channel has a buffer size of **100 events**
- If you don't consume events fast enough, old events will be dropped
- Creating a subscriber is very cheap (just cloning a sender)
- You can create subscribers at any time, even after the node has started
- Subscribers created after an event won't receive that event (events are not replayed)

## Integration with Structured Logging

You can integrate events with your logging system:

```rust
use helia_interface::{Helia, HeliaEvent};
use tracing::{info, warn};

async fn setup_event_logging(helia: &impl Helia) {
    let mut events_rx = helia.subscribe_events();
    
    tokio::spawn(async move {
        while let Ok(event) = events_rx.recv().await {
            match event {
                HeliaEvent::Start => {
                    info!(target: "helia", "Node started");
                }
                HeliaEvent::Stop => {
                    warn!(target: "helia", "Node stopped");
                }
                HeliaEvent::GcStarted => {
                    info!(target: "helia::gc", "Garbage collection started");
                }
                HeliaEvent::GcCompleted => {
                    info!(target: "helia::gc", "Garbage collection completed");
                }
            }
        }
    });
}
```

## Future Events

The event system is designed to be extensible. Future versions may add:
- `HeliaEvent::BlockAdded(Cid)` - When a block is stored
- `HeliaEvent::BlockRetrieved(Cid)` - When a block is fetched
- `HeliaEvent::PeerConnected(PeerId)` - When a peer connects
- `HeliaEvent::PeerDisconnected(PeerId)` - When a peer disconnects
- `HeliaEvent::GcDeleted(Cid)` - When a block is deleted during GC

## See Also

- [Helia Interface Documentation](../../helia-interface/src/lib.rs)
- [Tokio broadcast channel](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
- [Helia.js Events](https://github.com/ipfs/helia/blob/main/packages/interface/src/index.ts)
