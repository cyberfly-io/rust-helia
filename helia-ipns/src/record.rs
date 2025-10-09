//! IPNS record types and validation

use crate::errors::IpnsError;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use libp2p_identity::Keypair;

/// IPNS record containing published content
///
/// This wraps the underlying `ipns` crate record with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpnsRecord {
    /// The value this name points to (can be /ipfs/<cid> or /ipns/<key>)
    pub value: String,

    /// Sequence number (for ordering, higher is newer)
    pub sequence: u64,

    /// Validity period (RFC3339 timestamp)
    pub validity: String,

    /// Time to live in nanoseconds
    pub ttl: u64,

    /// Public key that signed this record (serialized)
    pub public_key: Vec<u8>,

    /// Signature data
    pub signature: Vec<u8>,

    /// Signature V2 data (optional, for v2 compatibility)
    pub signature_v2: Option<Vec<u8>>,
}

impl IpnsRecord {
    /// Check if the record has expired based on its validity period
    pub fn is_expired(&self) -> bool {
        // Parse the validity timestamp
        if let Ok(validity_time) = chrono::DateTime::parse_from_rfc3339(&self.validity) {
            let now = chrono::Utc::now();
            now > validity_time
        } else {
            // If we can't parse, consider it expired to be safe
            true
        }
    }

    /// Get the TTL in milliseconds
    pub fn ttl_ms(&self) -> u64 {
        self.ttl / 1_000_000
    }

    /// Get the validity as a SystemTime
    pub fn validity_time(&self) -> Result<SystemTime, IpnsError> {
        chrono::DateTime::parse_from_rfc3339(&self.validity)
            .map(|dt| UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64))
            .map_err(|e| IpnsError::InvalidRecord(format!("Invalid validity timestamp: {}", e)))
    }
}

/// CBOR data structure for IPNS records
/// Contains the fields that are signed in V2 signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpnsCborData {
    #[serde(rename = "Value")]
    pub value: Vec<u8>,
    
    #[serde(rename = "Validity")]
    pub validity: Vec<u8>,
    
    #[serde(rename = "ValidityType")]
    pub validity_type: u64,
    
    #[serde(rename = "Sequence")]
    pub sequence: u64,
    
    #[serde(rename = "TTL")]
    pub ttl: u64,
}

/// Encode IPNS record data as DAG-CBOR
///
/// Creates a CBOR document with fields sorted alphabetically:
/// Sequence, TTL, Validity, ValidityType, Value
pub fn encode_cbor_data(record: &IpnsRecord) -> Result<Vec<u8>, IpnsError> {
    let cbor_data = IpnsCborData {
        value: record.value.as_bytes().to_vec(),
        validity: record.validity.as_bytes().to_vec(),
        validity_type: 0, // EOL (expiration time)
        sequence: record.sequence,
        ttl: record.ttl,
    };
    
    // Use serde_ipld_dagcbor for deterministic encoding
    serde_ipld_dagcbor::to_vec(&cbor_data)
        .map_err(|e| IpnsError::MarshalingError(format!("Failed to encode CBOR: {}", e)))
}

/// Decode DAG-CBOR data from bytes
pub fn decode_cbor_data(bytes: &[u8]) -> Result<IpnsCborData, IpnsError> {
    serde_ipld_dagcbor::from_slice(bytes)
        .map_err(|e| IpnsError::MarshalingError(format!("Failed to decode CBOR: {}", e)))
}

/// Create signature data for V2 IPNS signature
///
/// According to IPNS spec, V2 signature data is:
/// "ipns-signature:" prefix + raw CBOR bytes from record data
///
/// The CBOR data contains: Sequence, TTL, Validity, ValidityType, Value
/// sorted alphabetically by key name
fn create_signature_data_v2(record: &IpnsRecord) -> Result<Vec<u8>, IpnsError> {
    let mut data = Vec::new();
    
    // Add the "ipns-signature:" prefix (hex: 69706e732d7369676e61747572653a)
    data.extend_from_slice(b"ipns-signature:");
    
    // Encode the record data as DAG-CBOR
    let cbor_bytes = encode_cbor_data(record)?;
    data.extend_from_slice(&cbor_bytes);
    
    Ok(data)
}

/// Create signature data for V1 IPNS signature (legacy)
///
/// According to IPNS spec, V1 signature data is:
/// value + validity + string(validityType)
fn create_signature_data_v1(record: &IpnsRecord) -> Result<Vec<u8>, IpnsError> {
    let mut data = Vec::new();
    
    // Concatenate: value + validity + validityType
    data.extend_from_slice(record.value.as_bytes());
    data.extend_from_slice(record.validity.as_bytes());
    // ValidityType 0 means EOL (expiration time)
    data.extend_from_slice(b"0");
    
    Ok(data)
}

/// Sign an IPNS record with the given keypair
///
/// Returns (signature_v1, signature_v2) tuple
///
/// # Arguments
/// * `keypair` - The keypair to sign with
/// * `record` - The record to sign
///
/// # Returns
/// A tuple of (V1 signature, V2 signature) as byte vectors
pub fn sign_record(
    keypair: &Keypair,
    record: &IpnsRecord,
) -> Result<(Vec<u8>, Vec<u8>), IpnsError> {
    // Create V2 signature (modern, required)
    let sig_data_v2 = create_signature_data_v2(record)?;
    let signature_v2 = keypair
        .sign(&sig_data_v2)
        .map_err(|e| IpnsError::SigningFailed(format!("Failed to sign V2 data: {}", e)))?;
    
    // Create V1 signature (legacy, for backward compatibility)
    let sig_data_v1 = create_signature_data_v1(record)?;
    let signature_v1 = keypair
        .sign(&sig_data_v1)
        .map_err(|e| IpnsError::SigningFailed(format!("Failed to sign V1 data: {}", e)))?;
    
    Ok((signature_v1, signature_v2))
}

/// Verify the signature of an IPNS record
///
/// # Arguments
/// * `record` - The record to verify
/// * `routing_key` - Optional routing key to verify against (if None, will derive from public key)
///
/// # Returns
/// Ok(()) if signature is valid, Err otherwise
pub fn verify_signature(
    record: &IpnsRecord,
    routing_key: Option<&[u8]>,
) -> Result<(), IpnsError> {
    use libp2p_identity::PublicKey;
    use crate::keys::routing_key_from_public_key;
    
    // Decode the public key from the record
    let public_key = PublicKey::try_decode_protobuf(&record.public_key)
        .map_err(|e| IpnsError::Identity(format!("Failed to decode public key: {}", e)))?;
    
    // If routing key is provided, verify it matches the public key
    if let Some(expected_key) = routing_key {
        let derived_key = routing_key_from_public_key(&public_key);
        if expected_key != derived_key.as_slice() {
            return Err(IpnsError::ValidationFailed(
                "Routing key does not match public key".to_string()
            ));
        }
    }
    
    // Verify V2 signature (required)
    if let Some(sig_v2) = &record.signature_v2 {
        let sig_data_v2 = create_signature_data_v2(record)?;
        
        if !public_key.verify(&sig_data_v2, sig_v2) {
            return Err(IpnsError::ValidationFailed(
                "Invalid V2 signature".to_string()
            ));
        }
    } else {
        return Err(IpnsError::ValidationFailed(
            "Missing V2 signature".to_string()
        ));
    }
    
    // V1 signature verification is optional (for backward compatibility)
    // If present, verify it
    if !record.signature.is_empty() {
        let sig_data_v1 = create_signature_data_v1(record)?;
        
        if !public_key.verify(&sig_data_v1, &record.signature) {
            return Err(IpnsError::ValidationFailed(
                "Invalid V1 signature".to_string()
            ));
        }
    }
    
    Ok(())
}

/// Unmarshal an IPNS record from bytes
///
/// For now uses JSON, but should eventually use protobuf
pub fn unmarshal_record(bytes: &[u8]) -> Result<IpnsRecord, IpnsError> {
    serde_json::from_slice(bytes)
        .map_err(|e| IpnsError::MarshalingError(format!("Failed to unmarshal record: {}", e)))
}

/// Marshal an IPNS record to protobuf bytes
pub fn marshal_record_protobuf(record: &IpnsRecord) -> Result<Vec<u8>, IpnsError> {
    use prost::Message;
    use crate::protobuf::IpnsEntry;
    
    // Encode the record data as DAG-CBOR for the data field
    let cbor_data = encode_cbor_data(record)?;
    
    // Create the protobuf entry
    let entry = IpnsEntry {
        value: record.value.as_bytes().to_vec(),
        signature_v1: record.signature.clone(),
        validity_type: 0, // EOL
        validity: record.validity.as_bytes().to_vec(),
        sequence: record.sequence,
        ttl: record.ttl,
        pub_key: record.public_key.clone(),
        signature_v2: record.signature_v2.clone().unwrap_or_default(),
        data: cbor_data,
    };
    
    // Encode to protobuf bytes
    let mut buf = Vec::new();
    entry.encode(&mut buf)
        .map_err(|e| IpnsError::MarshalingError(format!("Failed to encode protobuf: {}", e)))?;
    
    Ok(buf)
}

/// Unmarshal an IPNS record from protobuf bytes
pub fn unmarshal_record_protobuf(bytes: &[u8]) -> Result<IpnsRecord, IpnsError> {
    use prost::Message;
    use crate::protobuf::IpnsEntry;
    
    // Decode the protobuf entry
    let entry = IpnsEntry::decode(bytes)
        .map_err(|e| IpnsError::MarshalingError(format!("Failed to decode protobuf: {}", e)))?;
    
    // Decode the CBOR data
    let cbor_data = decode_cbor_data(&entry.data)?;
    
    // Create the IpnsRecord from the entry
    // Use CBOR data as the source of truth (V2 standard)
    Ok(IpnsRecord {
        value: String::from_utf8(cbor_data.value)
            .map_err(|e| IpnsError::InvalidRecord(format!("Invalid UTF-8 in value: {}", e)))?,
        sequence: cbor_data.sequence,
        validity: String::from_utf8(cbor_data.validity)
            .map_err(|e| IpnsError::InvalidRecord(format!("Invalid UTF-8 in validity: {}", e)))?,
        ttl: cbor_data.ttl,
        public_key: entry.pub_key,
        signature: entry.signature_v1,
        signature_v2: if entry.signature_v2.is_empty() { None } else { Some(entry.signature_v2) },
    })
}

/// Validate an IPNS record
///
/// Checks:
/// - Record format (can be unmarshaled)
/// - Signature verification
/// - Validity period (not expired)
/// - Routing key matches public key
pub fn validate_ipns_record(routing_key: &[u8], record: &[u8]) -> Result<(), IpnsError> {
    // Basic checks
    if record.is_empty() {
        return Err(IpnsError::InvalidRecord("Empty record".to_string()));
    }

    if routing_key.is_empty() {
        return Err(IpnsError::InvalidKey("Empty routing key".to_string()));
    }

    // 1. Unmarshal the record
    let ipns_record = unmarshal_record(record)?;

    // 2. Verify the signature
    verify_signature(&ipns_record, Some(routing_key))?;

    // 3. Check if the record has expired
    if ipns_record.is_expired() {
        return Err(IpnsError::RecordExpired {
            validity: ipns_record.validity.clone(),
        });
    }

    // 4. Verify record format (value should be a valid path)
    if ipns_record.value.is_empty() {
        return Err(IpnsError::InvalidRecord("Empty value".to_string()));
    }

    if !ipns_record.value.starts_with("/ipfs/") && !ipns_record.value.starts_with("/ipns/") {
        return Err(IpnsError::InvalidRecord(
            format!("Invalid value path: {}", ipns_record.value)
        ));
    }

    Ok(())
}

/// Select the best record from a list of records
///
/// Uses sequence number and validity to determine which record is best
/// Returns the index of the best valid record
pub fn select_best_record(routing_key: &[u8], records: &[Vec<u8>]) -> Result<usize, IpnsError> {
    if records.is_empty() {
        return Err(IpnsError::NotFound("No records provided".to_string()));
    }

    if records.len() == 1 {
        // Still validate the single record
        validate_ipns_record(routing_key, &records[0])?;
        return Ok(0);
    }

    // Parse and validate all records, keeping track of valid ones
    let mut valid_records: Vec<(usize, IpnsRecord)> = Vec::new();
    
    for (idx, record_bytes) in records.iter().enumerate() {
        // Try to validate this record
        if validate_ipns_record(routing_key, record_bytes).is_ok() {
            if let Ok(record) = unmarshal_record(record_bytes) {
                valid_records.push((idx, record));
            }
        }
    }

    if valid_records.is_empty() {
        return Err(IpnsError::ValidationFailed(
            "No valid records found".to_string()
        ));
    }

    // Find the record with the highest sequence number
    let (best_idx, _) = valid_records
        .into_iter()
        .max_by_key(|(_, record)| record.sequence)
        .unwrap(); // Safe because we checked valid_records is not empty

    Ok(best_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_expiry() {
        // Create a record that expires in the future
        let future = chrono::Utc::now() + chrono::Duration::hours(24);
        let record = IpnsRecord {
            value: "/ipfs/QmTest".to_string(),
            sequence: 1,
            validity: future.to_rfc3339(),
            ttl: 300_000_000_000, // 5 minutes
            public_key: vec![],
            signature: vec![],
            signature_v2: None,
        };

        assert!(!record.is_expired());

        // Create a record that has already expired
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

    #[test]
    fn test_ttl_conversion() {
        let record = IpnsRecord {
            value: "/ipfs/QmTest".to_string(),
            sequence: 1,
            validity: chrono::Utc::now().to_rfc3339(),
            ttl: 300_000_000_000, // 5 minutes in nanoseconds
            public_key: vec![],
            signature: vec![],
            signature_v2: None,
        };

        assert_eq!(record.ttl_ms(), 300_000); // 5 minutes in milliseconds
    }
}
