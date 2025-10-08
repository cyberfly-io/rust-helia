//! IPNS record types and validation

use crate::errors::IpnsError;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

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

/// Validate an IPNS record
///
/// Checks:
/// - Signature verification
/// - Validity period (not expired)
/// - Record format
pub fn validate_ipns_record(routing_key: &[u8], record: &[u8]) -> Result<(), IpnsError> {
    // For now, this is a placeholder
    // In a full implementation, this would:
    // 1. Unmarshal the record
    // 2. Extract the public key
    // 3. Verify the signature
    // 4. Check the validity period
    // 5. Verify routing_key matches public key hash
    
    if record.is_empty() {
        return Err(IpnsError::InvalidRecord("Empty record".to_string()));
    }

    if routing_key.is_empty() {
        return Err(IpnsError::InvalidKey("Empty routing key".to_string()));
    }

    // TODO: Implement full validation using the ipns crate
    Ok(())
}

/// Select the best record from a list of records
///
/// Uses sequence number and validity to determine which record is best
pub fn select_best_record(routing_key: &[u8], records: &[Vec<u8>]) -> Result<usize, IpnsError> {
    if records.is_empty() {
        return Err(IpnsError::NotFound("No records provided".to_string()));
    }

    if records.len() == 1 {
        return Ok(0);
    }

    // For now, just return the first valid record
    // In a full implementation, this would:
    // 1. Parse all records
    // 2. Validate each one
    // 3. Compare sequence numbers
    // 4. Return the index of the best record

    Ok(0)
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
