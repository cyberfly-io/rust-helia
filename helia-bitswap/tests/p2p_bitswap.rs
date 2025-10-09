//! P2P Bitswap integration test
//!
//! This test validates end-to-end block exchange between two Helia nodes using Bitswap.
//! 
//! Test Scenario:
//! - Node A (Provider): Stores blocks and announces them
//! - Node B (Requester): Connects to Node A and retrieves blocks
//! - Verification: Block data matches original

use bytes::Bytes;
use cid::Cid;
use helia_bitswap::{Bitswap, BitswapConfig};
use helia_interface::Blocks;
use helia_utils::{BlockstoreConfig, SledBlockstore};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, debug};

/// Helper to create a test CID from data
fn create_test_cid(data: &[u8]) -> Cid {
    use sha2::{Sha256, Digest};
    use multihash::Multihash;
    
    let hash = Sha256::digest(data);
    let mh = Multihash::wrap(0x12, &hash).expect("Failed to create multihash");
    Cid::new_v1(0x55, mh)
}

#[tokio::test]
#[ignore] // Ignore by default as it requires network setup
async fn test_p2p_block_exchange() {
    // Initialize tracing for debugging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    info!("=== Starting P2P Bitswap Test ===");

    // Test data
    let test_data = Bytes::from("Hello, Bitswap P2P World!");
    let cid = create_test_cid(&test_data);
    
    info!("Test CID: {}", cid);
    info!("Test data: {} bytes", test_data.len());

    // Create two separate blockstores
    let blockstore_a = Arc::new(
        SledBlockstore::new(BlockstoreConfig {
            path: Some("/tmp/helia-test-node-a".into()),
            ..Default::default()
        })
        .expect("Failed to create blockstore A")
    );
    
    let blockstore_b = Arc::new(
        SledBlockstore::new(BlockstoreConfig {
            path: Some("/tmp/helia-test-node-b".into()),
            ..Default::default()
        })
        .expect("Failed to create blockstore B")
    );

    info!("Created blockstores");

    // Create Bitswap instances
    let config_a = BitswapConfig::default();
    let bitswap_a = Arc::new(
        Bitswap::new(blockstore_a.clone(), config_a)
            .await
            .expect("Failed to create Bitswap A")
    );

    let config_b = BitswapConfig::default();
    let bitswap_b = Arc::new(
        Bitswap::new(blockstore_b.clone(), config_b)
            .await
            .expect("Failed to create Bitswap B")
    );

    info!("Created Bitswap instances");

    // Start both nodes
    bitswap_a.start().await.expect("Failed to start Bitswap A");
    bitswap_b.start().await.expect("Failed to start Bitswap B");
    
    info!("Started Bitswap nodes");

    // Give nodes time to initialize
    sleep(Duration::from_secs(1)).await;

    // Node A: Store the block
    info!("Node A: Storing block");
    blockstore_a
        .put(&cid, test_data.clone(), None)
        .await
        .expect("Failed to store block in A");
    
    debug!("Node A: Block stored successfully");

    // Node A: Announce the block
    info!("Node A: Announcing block");
    bitswap_a
        .notify_new_blocks(
            vec![(cid, test_data.clone())],
            Default::default()
        )
        .await
        .expect("Failed to announce block");
    
    debug!("Node A: Block announced");

    // Give time for announcement to propagate
    sleep(Duration::from_secs(1)).await;

    // Node B: Request the block
    info!("Node B: Requesting block from network");
    let want_options = helia_bitswap::WantOptions {
        timeout: Some(Duration::from_secs(30)),
        priority: 10,
        accept_block_presence: true,
        peer: None,
    };

    match bitswap_b.want(&cid, want_options).await {
        Ok(received_data) => {
            info!("Node B: Block received! {} bytes", received_data.len());
            
            // Verify data integrity
            assert_eq!(
                received_data, test_data,
                "Received data doesn't match original"
            );
            
            info!("✅ SUCCESS: Block data matches original!");
            
            // Verify it's now in Node B's blockstore
            let stored_data = blockstore_b
                .get(&cid, None)
                .await
                .expect("Block should be in Node B's blockstore");
            
            assert_eq!(stored_data, test_data, "Stored data doesn't match");
            info!("✅ SUCCESS: Block correctly stored in Node B's blockstore!");
        }
        Err(e) => {
            panic!("❌ FAILED: Node B could not retrieve block: {:?}", e);
        }
    }

    // Check statistics
    let stats_a = bitswap_a.stats().await;
    let stats_b = bitswap_b.stats().await;
    
    info!("Node A stats: sent={}, received={}", stats_a.blocks_sent, stats_a.blocks_received);
    info!("Node B stats: sent={}, received={}", stats_b.blocks_sent, stats_b.blocks_received);

    // Stop both nodes
    bitswap_a.stop().await.expect("Failed to stop Bitswap A");
    bitswap_b.stop().await.expect("Failed to stop Bitswap B");
    
    info!("Stopped Bitswap nodes");
    info!("=== P2P Bitswap Test Complete ===");
}

#[tokio::test]
async fn test_bitswap_local_retrieval() {
    // Test that Bitswap can retrieve blocks from local blockstore
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    info!("=== Testing Local Block Retrieval ===");

    let test_data = Bytes::from("Local test data");
    let cid = create_test_cid(&test_data);

    let blockstore = Arc::new(
        SledBlockstore::new(BlockstoreConfig::default())
            .expect("Failed to create blockstore")
    );

    // Store block directly in blockstore
    blockstore
        .put(&cid, test_data.clone(), None)
        .await
        .expect("Failed to store block");

    info!("Stored block locally with CID: {}", cid);

    // Create Bitswap instance
    let bitswap = Bitswap::new(blockstore.clone(), BitswapConfig::default())
        .await
        .expect("Failed to create Bitswap");

    bitswap.start().await.expect("Failed to start Bitswap");

    // Try to retrieve (should find in local blockstore first)
    let want_options = helia_bitswap::WantOptions {
        timeout: Some(Duration::from_secs(5)),
        priority: 0,
        accept_block_presence: true,
        peer: None,
    };

    match bitswap.want(&cid, want_options).await {
        Ok(retrieved_data) => {
            assert_eq!(retrieved_data, test_data);
            info!("✅ SUCCESS: Retrieved block from local blockstore");
        }
        Err(e) => {
            panic!("❌ FAILED: Could not retrieve local block: {:?}", e);
        }
    }

    bitswap.stop().await.expect("Failed to stop Bitswap");
    info!("=== Local Retrieval Test Complete ===");
}

#[tokio::test]
async fn test_bitswap_missing_block_timeout() {
    // Test that requesting a non-existent block times out appropriately
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    info!("=== Testing Missing Block Timeout ===");

    let test_data = Bytes::from("This block doesn't exist");
    let cid = create_test_cid(&test_data);

    let blockstore = Arc::new(
        SledBlockstore::new(BlockstoreConfig::default())
            .expect("Failed to create blockstore")
    );

    let bitswap = Bitswap::new(blockstore, BitswapConfig::default())
        .await
        .expect("Failed to create Bitswap");

    bitswap.start().await.expect("Failed to start Bitswap");

    info!("Requesting non-existent block: {}", cid);

    // Request a block that doesn't exist with short timeout
    let want_options = helia_bitswap::WantOptions {
        timeout: Some(Duration::from_secs(5)),
        priority: 0,
        accept_block_presence: true,
        peer: None,
    };

    let start = std::time::Instant::now();
    
    match bitswap.want(&cid, want_options).await {
        Ok(_) => {
            panic!("❌ FAILED: Should not have found non-existent block");
        }
        Err(e) => {
            let elapsed = start.elapsed();
            info!("Request timed out after {:?} with error: {:?}", elapsed, e);
            
            // Should timeout around 5 seconds (allow some margin)
            assert!(
                elapsed >= Duration::from_secs(4) && elapsed <= Duration::from_secs(35),
                "Timeout should be around 5-30 seconds (WantList default), was {:?}",
                elapsed
            );
            
            info!("✅ SUCCESS: Timeout behavior correct");
        }
    }

    bitswap.stop().await.expect("Failed to stop Bitswap");
    info!("=== Timeout Test Complete ===");
}
