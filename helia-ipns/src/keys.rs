//! Key management for IPNS

use crate::errors::IpnsError;
use libp2p_identity::{Keypair, PeerId, PublicKey};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Simple in-memory keychain for managing IPNS keys
#[derive(Debug, Clone)]
pub struct Keychain {
    keys: Arc<RwLock<HashMap<String, Keypair>>>,
}

impl Keychain {
    /// Create a new empty keychain
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate or load a key by name
    pub fn get_or_create_key(&self, key_name: &str) -> Result<Keypair, IpnsError> {
        let keys = self.keys.read().unwrap();
        
        if let Some(keypair) = keys.get(key_name) {
            // Return a copy of the existing key
            return Self::copy_keypair(keypair);
        }
        
        drop(keys);

        // Generate a new Ed25519 key
        let keypair = Keypair::generate_ed25519();
        
        let mut keys = self.keys.write().unwrap();
        keys.insert(key_name.to_string(), Self::copy_keypair(&keypair)?);
        
        Ok(keypair)
    }

    /// Export a public key by key name
    pub fn export_public_key(&self, key_name: &str) -> Result<PublicKey, IpnsError> {
        let keys = self.keys.read().unwrap();
        
        keys.get(key_name)
            .map(|kp| kp.public())
            .ok_or_else(|| IpnsError::KeyNotFound(key_name.to_string()))
    }

    /// Import a keypair
    pub fn import_key(&self, key_name: &str, keypair: Keypair) -> Result<(), IpnsError> {
        let mut keys = self.keys.write().unwrap();
        keys.insert(key_name.to_string(), keypair);
        Ok(())
    }

    /// Delete a key
    pub fn remove_key(&self, key_name: &str) -> Result<(), IpnsError> {
        let mut keys = self.keys.write().unwrap();
        
        if keys.remove(key_name).is_some() {
            Ok(())
        } else {
            Err(IpnsError::KeyNotFound(key_name.to_string()))
        }
    }

    /// List all key names
    pub fn list_keys(&self) -> Vec<String> {
        let keys = self.keys.read().unwrap();
        keys.keys().cloned().collect()
    }

    /// Helper to copy a keypair (since Keypair doesn't implement Clone)
    fn copy_keypair(keypair: &Keypair) -> Result<Keypair, IpnsError> {
        // Serialize and deserialize to copy
        let proto_bytes = keypair.to_protobuf_encoding()
            .map_err(|e| IpnsError::InvalidKey(format!("Failed to encode keypair: {}", e)))?;
        
        Keypair::from_protobuf_encoding(&proto_bytes)
            .map_err(|e| IpnsError::InvalidKey(format!("Failed to decode keypair: {}", e)))
    }
}

impl Default for Keychain {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the IPNS routing key from a public key
pub fn routing_key_from_public_key(public_key: &PublicKey) -> Vec<u8> {
    let peer_id = public_key.to_peer_id();
    routing_key_from_peer_id(&peer_id)
}

/// Compute the IPNS routing key from a peer ID
pub fn routing_key_from_peer_id(peer_id: &PeerId) -> Vec<u8> {
    // IPNS routing key is: /ipns/<base36(peer_id)>
    // But for routing purposes, we use the raw multihash bytes
    let multihash = peer_id.as_ref();
    
    // Prepend the "/ipns/" prefix as bytes
    let mut key = b"/ipns/".to_vec();
    key.extend_from_slice(multihash.to_bytes().as_slice());
    key
}

/// Extract peer ID from routing key
pub fn peer_id_from_routing_key(routing_key: &[u8]) -> Result<PeerId, IpnsError> {
    if !routing_key.starts_with(b"/ipns/") {
        return Err(IpnsError::InvalidKey("Routing key must start with /ipns/".to_string()));
    }

    let multihash_bytes = &routing_key[6..]; // Skip "/ipns/"
    
    PeerId::from_bytes(multihash_bytes)
        .map_err(|e| IpnsError::InvalidKey(format!("Invalid peer ID in routing key: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychain_operations() {
        let keychain = Keychain::new();
        
        // Generate a key
        let key1 = keychain.get_or_create_key("test-key").unwrap();
        
        // Getting it again should return the same public key
        let key2 = keychain.get_or_create_key("test-key").unwrap();
        assert_eq!(key1.public(), key2.public());
        
        // Export public key
        let public_key = keychain.export_public_key("test-key").unwrap();
        assert_eq!(public_key, key1.public());
        
        // List keys
        let keys = keychain.list_keys();
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"test-key".to_string()));
        
        // Remove key
        keychain.remove_key("test-key").unwrap();
        assert!(keychain.export_public_key("test-key").is_err());
    }

    #[test]
    fn test_routing_key_conversion() {
        let keypair = Keypair::generate_ed25519();
        let public_key = keypair.public();
        let peer_id = public_key.to_peer_id();
        
        // Create routing key
        let routing_key = routing_key_from_public_key(&public_key);
        assert!(routing_key.starts_with(b"/ipns/"));
        
        // Extract peer ID back
        let extracted_peer_id = peer_id_from_routing_key(&routing_key).unwrap();
        assert_eq!(peer_id, extracted_peer_id);
    }
}
