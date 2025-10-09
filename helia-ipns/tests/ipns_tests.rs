//! Tests for IPNS functionality

use helia_ipns::*;
use cid::Cid;
use std::sync::Arc;

#[tokio::test]
async fn test_ipns_factory() {
    let init = IpnsInit::default();
    let name = ipns(init);
    assert!(name.is_ok());
}

#[tokio::test]
async fn test_ipns_with_custom_routers() {
    let init = IpnsInit {
        routers: vec![],
        republish_interval: Some(std::time::Duration::from_secs(3600)),
        republish_concurrency: Some(5),
        enable_republish: false,
    };
    
    let name = ipns(init).unwrap();
    assert_eq!(name.routers().len(), 0);
}

#[tokio::test]
async fn test_publish_basic() {
    let name = ipns(IpnsInit::default()).unwrap();
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    // Publish in offline mode (no routers needed)
    let mut options = PublishOptions::default();
    options.offline = true;
    
    let result = name.publish("test-key", &cid, options).await;
    assert!(result.is_ok());
    
    let publish_result = result.unwrap();
    assert_eq!(publish_result.record.sequence, 1);
    assert!(!publish_result.public_key.is_empty());
    assert!(publish_result.record.value.contains("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"));
}

#[tokio::test]
async fn test_resolve_published_record() {
    let name = ipns(IpnsInit::default()).unwrap();
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    // Publish in offline mode
    let mut pub_options = PublishOptions::default();
    pub_options.offline = true;
    
    let publish_result = name.publish("test-key-resolve", &cid, pub_options).await.unwrap();
    
    // Resolve using the public key
    let mut res_options = ResolveOptions::default();
    res_options.offline = true;
    
    let resolve_result = name.resolve(&publish_result.public_key, res_options).await;
    assert!(resolve_result.is_ok());
    
    let resolved = resolve_result.unwrap();
    assert_eq!(resolved.cid.to_string(), cid.to_string());
    assert_eq!(resolved.path, "");
}

#[tokio::test]
async fn test_local_store() {
    let store = LocalStore::new();
    assert!(store.is_empty());
    
    let routing_key = b"test-routing-key";
    let record = b"test-record-data".to_vec();
    let metadata = RecordMetadata::new("my-key".to_string(), 48 * 60 * 60 * 1000);
    
    store.put(routing_key, record.clone(), Some(metadata)).unwrap();
    assert!(!store.is_empty());
    assert!(store.has(routing_key));
    
    let stored = store.get(routing_key).unwrap();
    assert_eq!(stored.record, record);
}

#[tokio::test]
async fn test_record_expiry() {
    // Create a record that hasn't expired yet
    let future = chrono::Utc::now() + chrono::Duration::hours(24);
    let record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: future.to_rfc3339(),
        ttl: 300_000_000_000, // 5 minutes in nanoseconds
        public_key: vec![],
        signature: vec![],
        signature_v2: None,
    };
    
    assert!(!record.is_expired());
    assert_eq!(record.ttl_ms(), 300_000); // 5 minutes in milliseconds
    
    // Create a record that has expired
    let past = chrono::Utc::now() - chrono::Duration::hours(24);
    let expired_record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: past.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: vec![],
        signature: vec![],
        signature_v2: None,
    };
    
    assert!(expired_record.is_expired());
}

#[tokio::test]
async fn test_error_types() {
    let err = IpnsError::NotFound("test".to_string());
    assert!(err.to_string().contains("not found"));
    
    let err = IpnsError::RecordExpired {
        validity: "2024-01-01T00:00:00Z".to_string(),
    };
    assert!(err.to_string().contains("expired"));
    
    let err = IpnsError::records_failed_validation(3);
    assert!(err.to_string().contains("3"));
    assert!(err.to_string().contains("records")); // plural
    
    let err = IpnsError::records_failed_validation(1);
    assert!(err.to_string().contains("record")); // singular
}

#[tokio::test]
async fn test_publish_options_defaults() {
    let options = PublishOptions::default();
    assert_eq!(options.lifetime, Some(DEFAULT_LIFETIME_MS));
    assert!(!options.offline);
    assert_eq!(options.ttl, Some(DEFAULT_TTL_NS / 1_000_000));
    assert!(options.v1_compatible);
}

#[tokio::test]
async fn test_resolve_options_defaults() {
    let options = ResolveOptions::default();
    assert!(!options.offline);
    assert!(!options.nocache);
    assert!(options.max_depth.is_none());
    assert!(options.timeout.is_none());
}

#[tokio::test]
async fn test_publish_increments_sequence() {
    let name = ipns(IpnsInit::default()).unwrap();
    let cid1: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    let cid2: Cid = "bafybeihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
        .parse()
        .unwrap();
    
    let mut options = PublishOptions::default();
    options.offline = true;
    
    // First publish - sequence should be 1
    let result1 = name.publish("test-seq-key", &cid1, options.clone()).await.unwrap();
    assert_eq!(result1.record.sequence, 1);
    
    // Second publish - sequence should be 2
    let result2 = name.publish("test-seq-key", &cid2, options).await.unwrap();
    assert_eq!(result2.record.sequence, 2);
}

#[tokio::test]
async fn test_unpublish() {
    let name = ipns(IpnsInit::default()).unwrap();
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    let mut options = PublishOptions::default();
    options.offline = true;
    
    // Publish
    let result = name.publish("test-unpublish-key", &cid, options.clone()).await.unwrap();
    
    // Verify it exists
    let mut res_options = ResolveOptions::default();
    res_options.offline = true;
    let resolve_result = name.resolve(&result.public_key, res_options.clone()).await;
    assert!(resolve_result.is_ok());
    
    // Unpublish
    let unpublish_result = name.unpublish("test-unpublish-key").await;
    assert!(unpublish_result.is_ok());
    
    // Verify it's gone
    let resolve_result = name.resolve(&result.public_key, res_options).await;
    assert!(resolve_result.is_err());
}

#[tokio::test]
async fn test_start_stop() {
    let name = ipns(IpnsInit::default()).unwrap();
    
    assert!(name.start().await.is_ok());
    assert!(name.stop().await.is_ok());
}

#[tokio::test]
async fn test_resolve_not_found() {
    let name = ipns(IpnsInit::default()).unwrap();
    let fake_key = b"fake-nonexistent-key";
    
    let mut options = ResolveOptions::default();
    options.offline = true;
    
    let result = name.resolve(fake_key, options).await;
    assert!(result.is_err());
    
    // The error could be either NotFound or InvalidKey depending on what fake_key is
    if let Err(e) = result {
        assert!(matches!(e, IpnsError::NotFound(_) | IpnsError::InvalidKey(_)));
    }
}

#[tokio::test]
async fn test_nocache_option() {
    let name = ipns(IpnsInit::default()).unwrap();
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    let mut pub_options = PublishOptions::default();
    pub_options.offline = true;
    
    let publish_result = name.publish("test-nocache-key", &cid, pub_options).await.unwrap();
    
    // First resolve (should cache)
    let mut res_options = ResolveOptions::default();
    res_options.offline = true;
    let result1 = name.resolve(&publish_result.public_key, res_options.clone()).await;
    assert!(result1.is_ok());
    
    // Resolve with nocache - in offline mode, should still read from local store
    // but won't update the cache timestamp
    res_options.nocache = true;
    let result2 = name.resolve(&publish_result.public_key, res_options).await;
    assert!(result2.is_ok()); // Should work since offline=true forces local store check
}

#[tokio::test]
async fn test_republish_start_stop() {
    // Test that republish task starts and stops with service
    let mut init = IpnsInit::default();
    init.enable_republish = true;
    init.republish_interval = Some(std::time::Duration::from_millis(100)); // Fast interval for testing
    
    let name = ipns(init).unwrap();
    
    // Start the service (should start republish task)
    let start_result = name.start().await;
    assert!(start_result.is_ok());
    
    // Wait a bit to ensure task is running
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    // Stop the service (should stop republish task)
    let stop_result = name.stop().await;
    assert!(stop_result.is_ok());
    
    // Wait a bit more to ensure clean shutdown
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_republish_disabled() {
    // Test that republish can be disabled
    let mut init = IpnsInit::default();
    init.enable_republish = false;
    
    let name = ipns(init).unwrap();
    
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    let mut pub_options = PublishOptions::default();
    pub_options.offline = true;
    
    // Publish a record
    let publish_result = name.publish("test-no-republish", &cid, pub_options).await;
    assert!(publish_result.is_ok());
    
    // Start service (should not start republish task since disabled)
    let start_result = name.start().await;
    assert!(start_result.is_ok());
    
    // Wait a bit
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Stop service
    let stop_result = name.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_multiple_start_stop() {
    // Test that multiple start/stop calls don't cause issues
    let name = ipns(IpnsInit::default()).unwrap();
    
    // Start multiple times
    assert!(name.start().await.is_ok());
    assert!(name.start().await.is_ok()); // Should be idempotent
    assert!(name.start().await.is_ok());
    
    // Stop multiple times  
    assert!(name.stop().await.is_ok());
    assert!(name.stop().await.is_ok()); // Should be idempotent
    assert!(name.stop().await.is_ok());
}

// ============ Signature Tests ============

#[tokio::test]
async fn test_signature_generation() {
    // Test that signatures are generated and not empty
    let name = ipns(IpnsInit::default()).unwrap();
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    let mut options = PublishOptions::default();
    options.offline = true;
    
    let result = name.publish("test-key", &cid, options).await.unwrap();
    
    // Check that signatures are not empty
    assert!(!result.record.signature.is_empty(), "V1 signature should not be empty");
    assert!(result.record.signature_v2.is_some(), "V2 signature should exist");
    assert!(!result.record.signature_v2.unwrap().is_empty(), "V2 signature should not be empty");
}

#[tokio::test]
async fn test_signature_verification_valid() {
    use helia_ipns::record::{IpnsRecord, verify_signature};
    use libp2p_identity::Keypair;
    use helia_ipns::record::sign_record;
    
    // Create a test keypair
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    
    // Create a test record
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: validity.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Verify should succeed
    let result = verify_signature(&record, None);
    assert!(result.is_ok(), "Valid signature should verify successfully");
}

#[tokio::test]
async fn test_signature_verification_invalid() {
    use helia_ipns::record::{IpnsRecord, verify_signature};
    use libp2p_identity::Keypair;
    use helia_ipns::record::sign_record;
    
    // Create a test keypair
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    
    // Create a test record
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: validity.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Tamper with the record value
    record.value = "/ipfs/QmTamperedValue".to_string();
    
    // Verification should fail
    let result = verify_signature(&record, None);
    assert!(result.is_err(), "Tampered record should fail verification");
}

#[tokio::test]
async fn test_signature_verification_wrong_key() {
    use helia_ipns::record::{IpnsRecord, verify_signature};
    use libp2p_identity::Keypair;
    use helia_ipns::record::sign_record;
    use helia_ipns::keys::routing_key_from_public_key;
    
    // Create two different keypairs
    let keypair1 = Keypair::generate_ed25519();
    let keypair2 = Keypair::generate_ed25519();
    
    let public_key1 = keypair1.public();
    let public_key2 = keypair2.public();
    
    // Create a record signed by keypair1
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: validity.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: public_key1.encode_protobuf(),
        signature: vec![],
        signature_v2: None,
    };
    
    // Sign with keypair1
    let (sig_v1, sig_v2) = sign_record(&keypair1, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Try to verify with routing key from keypair2
    let wrong_routing_key = routing_key_from_public_key(&public_key2);
    let result = verify_signature(&record, Some(&wrong_routing_key));
    
    // Should fail because routing key doesn't match
    assert!(result.is_err(), "Wrong routing key should fail verification");
}

#[tokio::test]
async fn test_publish_creates_valid_signatures() {
    // Test that publish creates records that can be verified
    let name = ipns(IpnsInit::default()).unwrap();
    let cid: Cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        .parse()
        .unwrap();
    
    let mut options = PublishOptions::default();
    options.offline = true;
    
    let result = name.publish("test-key", &cid, options).await.unwrap();
    
    // Verify the published record
    use helia_ipns::record::verify_signature;
    let verify_result = verify_signature(&result.record, None);
    assert!(verify_result.is_ok(), "Published record should have valid signature");
}

#[tokio::test]
async fn test_validation_with_signatures() {
    use helia_ipns::record::{validate_ipns_record, IpnsRecord};
    use libp2p_identity::Keypair;
    use helia_ipns::record::sign_record;
    use helia_ipns::keys::routing_key_from_public_key;
    
    // Create a test keypair
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    let routing_key = routing_key_from_public_key(&public_key);
    
    // Create a valid record
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: validity.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Marshal the record
    let record_bytes = serde_json::to_vec(&record).unwrap();
    
    // Validation should succeed
    let result = validate_ipns_record(&routing_key, &record_bytes);
    assert!(result.is_ok(), "Valid record should pass validation");
}

#[tokio::test]
async fn test_validation_rejects_expired() {
    use helia_ipns::record::{validate_ipns_record, IpnsRecord};
    use libp2p_identity::Keypair;
    use helia_ipns::record::sign_record;
    use helia_ipns::keys::routing_key_from_public_key;
    
    // Create a test keypair
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    let routing_key = routing_key_from_public_key(&public_key);
    
    // Create an expired record
    let validity = chrono::Utc::now() - chrono::Duration::hours(1);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest".to_string(),
        sequence: 1,
        validity: validity.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Marshal the record
    let record_bytes = serde_json::to_vec(&record).unwrap();
    
    // Validation should fail due to expiry
    let result = validate_ipns_record(&routing_key, &record_bytes);
    assert!(result.is_err(), "Expired record should fail validation");
}

#[tokio::test]
async fn test_select_best_record() {
    use helia_ipns::record::{select_best_record, IpnsRecord};
    use libp2p_identity::Keypair;
    use helia_ipns::record::sign_record;
    use helia_ipns::keys::routing_key_from_public_key;
    
    // Create a test keypair
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    let routing_key = routing_key_from_public_key(&public_key);
    
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    
    // Create three records with different sequence numbers
    let mut records_bytes = Vec::new();
    for seq in [1, 3, 2] {
        let mut record = IpnsRecord {
            value: format!("/ipfs/QmTest{}", seq),
            sequence: seq,
            validity: validity.to_rfc3339(),
            ttl: 300_000_000_000,
            public_key: public_key_bytes.clone(),
            signature: vec![],
            signature_v2: None,
        };
        
        let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
        record.signature = sig_v1;
        record.signature_v2 = Some(sig_v2);
        
        records_bytes.push(serde_json::to_vec(&record).unwrap());
    }
    
    // Select best should return index 1 (sequence 3)
    let best_idx = select_best_record(&routing_key, &records_bytes).unwrap();
    assert_eq!(best_idx, 1, "Should select record with highest sequence number");
}

// ============ Protobuf Tests ============

#[tokio::test]
async fn test_protobuf_marshal_unmarshal_roundtrip() {
    use helia_ipns::record::{IpnsRecord, marshal_record_protobuf, unmarshal_record_protobuf, sign_record};
    use libp2p_identity::Keypair;
    
    // Create a test keypair and record
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest123".to_string(),
        sequence: 42,
        validity: validity.to_rfc3339(),
        ttl: 300_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    // Sign the record
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Marshal to protobuf
    let protobuf_bytes = marshal_record_protobuf(&record).unwrap();
    
    // Unmarshal from protobuf
    let unmarshaled = unmarshal_record_protobuf(&protobuf_bytes).unwrap();
    
    // Verify all fields match
    assert_eq!(record.value, unmarshaled.value);
    assert_eq!(record.sequence, unmarshaled.sequence);
    assert_eq!(record.validity, unmarshaled.validity);
    assert_eq!(record.ttl, unmarshaled.ttl);
    assert_eq!(record.public_key, unmarshaled.public_key);
    assert_eq!(record.signature, unmarshaled.signature);
    assert_eq!(record.signature_v2, unmarshaled.signature_v2);
}

#[tokio::test]
async fn test_protobuf_signature_verification() {
    use helia_ipns::record::{IpnsRecord, marshal_record_protobuf, unmarshal_record_protobuf, sign_record, verify_signature};
    use libp2p_identity::Keypair;
    
    // Create and sign a record
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    
    let validity = chrono::Utc::now() + chrono::Duration::hours(24);
    let mut record = IpnsRecord {
        value: "/ipfs/QmTest456".to_string(),
        sequence: 100,
        validity: validity.to_rfc3339(),
        ttl: 600_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Marshal, unmarshal, and verify
    let protobuf_bytes = marshal_record_protobuf(&record).unwrap();
    let unmarshaled = unmarshal_record_protobuf(&protobuf_bytes).unwrap();
    
    // Signature should still be valid after roundtrip
    let result = verify_signature(&unmarshaled, None);
    assert!(result.is_ok(), "Signature should be valid after protobuf roundtrip");
}

#[tokio::test]
async fn test_protobuf_with_dag_cbor() {
    use helia_ipns::record::{IpnsRecord, marshal_record_protobuf, unmarshal_record_protobuf, encode_cbor_data, decode_cbor_data, sign_record};
    use libp2p_identity::Keypair;
    
    // Create a record
    let keypair = Keypair::generate_ed25519();
    let public_key = keypair.public();
    let public_key_bytes = public_key.encode_protobuf();
    
    let validity = chrono::Utc::now() + chrono::Duration::hours(48);
    let mut record = IpnsRecord {
        value: "/ipfs/Qm789Complex".to_string(),
        sequence: 999,
        validity: validity.to_rfc3339(),
        ttl: 900_000_000_000,
        public_key: public_key_bytes,
        signature: vec![],
        signature_v2: None,
    };
    
    let (sig_v1, sig_v2) = sign_record(&keypair, &record).unwrap();
    record.signature = sig_v1;
    record.signature_v2 = Some(sig_v2);
    
    // Encode CBOR data directly
    let cbor_bytes = encode_cbor_data(&record).unwrap();
    let decoded_cbor = decode_cbor_data(&cbor_bytes).unwrap();
    
    // Verify CBOR encoding/decoding
    assert_eq!(record.value.as_bytes(), decoded_cbor.value.as_slice());
    assert_eq!(record.sequence, decoded_cbor.sequence);
    assert_eq!(record.ttl, decoded_cbor.ttl);
    
    // Now test full protobuf roundtrip
    let protobuf_bytes = marshal_record_protobuf(&record).unwrap();
    let unmarshaled = unmarshal_record_protobuf(&protobuf_bytes).unwrap();
    
    // Everything should match
    assert_eq!(record.value, unmarshaled.value);
    assert_eq!(record.sequence, unmarshaled.sequence);
}
