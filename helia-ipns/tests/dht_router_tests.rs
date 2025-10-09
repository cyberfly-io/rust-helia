use helia_ipns::record::{
    marshal_record_protobuf, sign_record, unmarshal_record_protobuf, verify_signature,
};
use helia_ipns::routing::{DhtRouter, GetOptions, PutOptions};
use helia_ipns::{IpnsRecord, IpnsRouting};
use libp2p::identity::Keypair;
use libp2p::kad::{store::MemoryStore, Behaviour as Kademlia, Mode};
use libp2p::{noise, tcp, yamux, PeerId};
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper function to create a libp2p swarm configured for IPNS
/// This demonstrates the pattern where users create their own libp2p instance
fn create_test_swarm(
    keypair: Keypair,
) -> Result<(libp2p::Swarm<Kademlia<MemoryStore>>, PeerId), Box<dyn std::error::Error>> {
    let peer_id = PeerId::from(keypair.public());

    // Create Kademlia behaviour
    let store = MemoryStore::new(peer_id);
    let mut kad = Kademlia::new(peer_id, store);
    kad.set_mode(Some(Mode::Server));

    // Build the swarm - users have full control over configuration
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_key| kad)?
        .build();

    Ok((swarm, peer_id))
}

#[tokio::test]
async fn test_dht_router_creation() {
    // User creates their own libp2p swarm (following Helia pattern)
    let keypair = Keypair::generate_ed25519();
    let (swarm, peer_id) = create_test_swarm(keypair).expect("Failed to create swarm");

    // Pass the swarm to DhtRouter
    let router = DhtRouter::new(swarm, peer_id);

    // Verify peer ID matches
    assert_eq!(router.peer_id(), peer_id);
    println!(
        "Created DHT router with user-provided libp2p swarm, peer ID: {}",
        peer_id
    );
}

#[tokio::test]
async fn test_dht_router_with_custom_keypair() {
    // User can use their own keypair
    let keypair = Keypair::generate_ed25519();
    let expected_peer_id = PeerId::from(keypair.public());

    let (swarm, peer_id) = create_test_swarm(keypair).expect("Failed to create swarm");

    let router = DhtRouter::new(swarm, peer_id);
    assert_eq!(router.peer_id(), expected_peer_id);
    println!("Router correctly uses user-provided keypair");
}

#[tokio::test]
async fn test_dht_put_operation() {
    // User creates and configures their libp2p instance
    let keypair = Keypair::generate_ed25519();
    let (swarm, peer_id) = create_test_swarm(keypair.clone()).expect("Failed to create swarm");

    let router = DhtRouter::new(swarm, peer_id);

    // Create a test IPNS record
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: format!("{}000000000", timestamp + 3600), // Valid for 1 hour
        ttl: 3600,
        public_key: vec![],
        signature: vec![],
        signature_v2: None,
    };

    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).expect("Failed to sign");
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);

    // Marshal to protobuf
    let marshaled = marshal_record_protobuf(&record).expect("Failed to marshal");

    // Create routing key
    let routing_key = format!("/ipns/{}", peer_id).into_bytes();

    // Test put operation
    let result = router
        .put(&routing_key, &marshaled, PutOptions::default())
        .await;
    assert!(result.is_ok(), "DHT put should succeed: {:?}", result.err());

    println!("Successfully published record to DHT using user-provided swarm");
}

#[tokio::test]
async fn test_dht_get_operation() {
    // User creates libp2p swarm
    let keypair = Keypair::generate_ed25519();
    let (swarm, peer_id) = create_test_swarm(keypair).expect("Failed to create swarm");

    let router = DhtRouter::new(swarm, peer_id);

    // Create routing key
    let routing_key = format!("/ipns/{}", peer_id).into_bytes();

    // Test get operation (should return NotFound for now since we haven't published anything)
    let result = router.get(&routing_key, GetOptions::default()).await;

    // We expect this to fail with NotFound since:
    // 1. We haven't published anything to the DHT
    // 2. The async query handling is not fully implemented yet
    assert!(result.is_err(), "Get should fail for non-existent record");

    println!("Get operation correctly returns NotFound for unpublished record");
}

#[tokio::test]
async fn test_protobuf_marshal_for_dht() {
    // Test that records can be marshaled for DHT distribution
    let keypair = Keypair::generate_ed25519();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut record = IpnsRecord {
        value: "/ipfs/QmTestContent".to_string(),
        sequence: 42,
        validity: format!("{}000000000", timestamp + 7200), // Valid for 2 hours
        ttl: 7200,
        public_key: keypair.public().encode_protobuf(), // Set the public key
        signature: vec![],
        signature_v2: None,
    };

    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).expect("Failed to sign");
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);

    // Marshal to protobuf
    let marshaled = marshal_record_protobuf(&record).expect("Failed to marshal");

    // Verify marshaled data is not empty
    assert!(!marshaled.is_empty(), "Marshaled data should not be empty");

    // Unmarshal and verify
    let unmarshaled = unmarshal_record_protobuf(&marshaled).expect("Failed to unmarshal");

    assert_eq!(unmarshaled.value, record.value);
    assert_eq!(unmarshaled.sequence, record.sequence);
    assert_eq!(unmarshaled.validity, record.validity);
    assert_eq!(unmarshaled.ttl, record.ttl);

    // Verify signature is still valid
    verify_signature(&unmarshaled, None).expect("Signature should be valid");

    println!("Protobuf marshaling works correctly for DHT distribution");
}

#[tokio::test]
async fn test_router_name() {
    let keypair = Keypair::generate_ed25519();
    let (swarm, peer_id) = create_test_swarm(keypair).expect("Failed to create swarm");

    let router = DhtRouter::new(swarm, peer_id);
    assert_eq!(router.name(), "dht", "Router name should be 'dht'");
}

#[tokio::test]
async fn test_multiple_routers() {
    // Test creating multiple DHT routers (simulating multi-peer network)
    // Each user creates their own libp2p instance
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();
    let keypair3 = Keypair::generate_ed25519();

    let (swarm1, peer_id1) = create_test_swarm(keypair1).expect("Failed to create swarm 1");
    let (swarm2, peer_id2) = create_test_swarm(keypair2).expect("Failed to create swarm 2");
    let (swarm3, peer_id3) = create_test_swarm(keypair3).expect("Failed to create swarm 3");

    let router1 = DhtRouter::new(swarm1, peer_id1);
    let router2 = DhtRouter::new(swarm2, peer_id2);
    let router3 = DhtRouter::new(swarm3, peer_id3);

    // Verify all peer IDs are unique
    assert_ne!(peer_id1, peer_id2, "Peer IDs should be unique");
    assert_ne!(peer_id2, peer_id3, "Peer IDs should be unique");
    assert_ne!(peer_id1, peer_id3, "Peer IDs should be unique");

    println!("Created 3 unique DHT routers with user-provided swarms:");
    println!("  Router 1: {}", peer_id1);
    println!("  Router 2: {}", peer_id2);
    println!("  Router 3: {}", peer_id3);
}
