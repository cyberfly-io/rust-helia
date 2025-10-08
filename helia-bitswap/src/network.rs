use crate::{BitswapMessage, Result};
use async_trait::async_trait;
use bytes::Bytes;
use helia_interface::HeliaError;
use libp2p::PeerId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Network interface for bitswap protocol
#[async_trait]
pub trait Network: Send + Sync {
    /// Send a message to a peer
    async fn send_message(&mut self, peer: &PeerId, message: BitswapMessage) -> Result<()>;
    
    /// Receive messages from the network
    async fn receive_message(&mut self) -> Result<(PeerId, BitswapMessage)>;
    
    /// Connect to a peer
    async fn connect_to_peer(&mut self, peer: &PeerId) -> Result<()>;
    
    /// Disconnect from a peer
    async fn disconnect_from_peer(&mut self, peer: &PeerId) -> Result<()>;
    
    /// Get connected peers
    fn connected_peers(&self) -> Vec<PeerId>;
    
    /// Start the network
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the network
    async fn stop(&mut self) -> Result<()>;
}

/// Simple in-memory network implementation for testing
pub struct BitswapNetwork {
    /// Connected peers
    connected_peers: HashMap<PeerId, PeerConnection>,
    /// Message sender
    message_sender: Option<mpsc::UnboundedSender<NetworkMessage>>,
    /// Message receiver
    message_receiver: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
    /// Whether the network is started
    started: bool,
}

/// Connection to a peer
#[derive(Debug, Clone)]
pub struct PeerConnection {
    /// Peer ID
    pub peer_id: PeerId,
    /// Whether the connection is active
    pub active: bool,
    /// Number of messages sent to this peer
    pub messages_sent: u64,
    /// Number of messages received from this peer
    pub messages_received: u64,
}

/// Network message containing peer and message data
#[derive(Debug)]
pub struct NetworkMessage {
    /// Sender peer ID
    pub sender: PeerId,
    /// Recipient peer ID (None for broadcast)
    pub recipient: Option<PeerId>,
    /// Message content
    pub message: BitswapMessage,
}

impl BitswapNetwork {
    /// Create a new bitswap network
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            connected_peers: HashMap::new(),
            message_sender: Some(sender),
            message_receiver: Some(receiver),
            started: false,
        }
    }

    /// Add a peer connection
    pub fn add_peer(&mut self, peer_id: PeerId) -> Result<()> {
        debug!("Adding peer connection: {}", peer_id);
        
        let connection = PeerConnection {
            peer_id,
            active: true,
            messages_sent: 0,
            messages_received: 0,
        };
        
        self.connected_peers.insert(peer_id, connection);
        Ok(())
    }

    /// Remove a peer connection
    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Result<()> {
        debug!("Removing peer connection: {}", peer_id);
        
        if let Some(mut connection) = self.connected_peers.get_mut(peer_id) {
            connection.active = false;
        }
        
        self.connected_peers.remove(peer_id);
        Ok(())
    }

    /// Get peer connection info
    pub fn get_peer_connection(&self, peer_id: &PeerId) -> Option<&PeerConnection> {
        self.connected_peers.get(peer_id)
    }

    /// Get all peer connections
    pub fn get_all_connections(&self) -> &HashMap<PeerId, PeerConnection> {
        &self.connected_peers
    }

    /// Update peer statistics
    pub fn update_peer_stats(&mut self, peer_id: &PeerId, sent: bool) {
        if let Some(connection) = self.connected_peers.get_mut(peer_id) {
            if sent {
                connection.messages_sent += 1;
            } else {
                connection.messages_received += 1;
            }
        }
    }

    /// Broadcast message to all connected peers
    pub async fn broadcast_message(&mut self, message: BitswapMessage) -> Result<()> {
        let peers: Vec<PeerId> = self.connected_peers.keys().cloned().collect();
        
        for peer in peers {
            self.send_message(&peer, message.clone()).await?;
        }
        
        Ok(())
    }

    /// Check if peer is connected
    pub fn is_peer_connected(&self, peer_id: &PeerId) -> bool {
        self.connected_peers.get(peer_id)
            .map_or(false, |conn| conn.active)
    }

    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.connected_peers.values()
            .filter(|conn| conn.active)
            .count()
    }
}

impl Default for BitswapNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Network for BitswapNetwork {
    async fn send_message(&mut self, peer: &PeerId, message: BitswapMessage) -> Result<()> {
        if !self.started {
            return Err(HeliaError::other("Network not started"));
        }

        if !self.is_peer_connected(peer) {
            return Err(HeliaError::other(format!("Peer {} not connected", peer)));
        }

        debug!("Sending message to peer {}", peer);
        
        // Update statistics
        self.update_peer_stats(peer, true);
        
        // In a real implementation, this would send over libp2p
        // For now, we'll just log it
        debug!("Message sent to {}: {} blocks, {} presence entries", 
               peer, message.blocks.len(), message.block_presences.len());
        
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<(PeerId, BitswapMessage)> {
        if !self.started {
            return Err(HeliaError::other("Network not started"));
        }

        if let Some(ref mut receiver) = self.message_receiver {
            if let Some(net_msg) = receiver.recv().await {
                // Update statistics
                self.update_peer_stats(&net_msg.sender, false);
                
                debug!("Received message from peer {}", net_msg.sender);
                return Ok((net_msg.sender, net_msg.message));
            }
        }
        
        Err(HeliaError::other("No message received"))
    }

    async fn connect_to_peer(&mut self, peer: &PeerId) -> Result<()> {
        info!("Connecting to peer: {}", peer);
        self.add_peer(*peer)?;
        Ok(())
    }

    async fn disconnect_from_peer(&mut self, peer: &PeerId) -> Result<()> {
        info!("Disconnecting from peer: {}", peer);
        self.remove_peer(peer)?;
        Ok(())
    }

    fn connected_peers(&self) -> Vec<PeerId> {
        self.connected_peers.keys().cloned().collect()
    }

    async fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }

        info!("Starting bitswap network");
        self.started = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if !self.started {
            return Ok(());
        }

        info!("Stopping bitswap network");
        
        // Disconnect all peers
        let peers: Vec<PeerId> = self.connected_peers.keys().cloned().collect();
        for peer in peers {
            self.remove_peer(&peer)?;
        }
        
        self.started = false;
        Ok(())
    }
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Number of connected peers
    pub connected_peers: usize,
    /// Number of connection attempts
    pub connection_attempts: u64,
    /// Number of failed connections
    pub failed_connections: u64,
}

impl NetworkStats {
    /// Create new network statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update sent statistics
    pub fn record_sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.bytes_sent += bytes as u64;
    }

    /// Update received statistics
    pub fn record_received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.bytes_received += bytes as u64;
    }

    /// Record connection attempt
    pub fn record_connection_attempt(&mut self, success: bool) {
        self.connection_attempts += 1;
        if !success {
            self.failed_connections += 1;
        }
    }

    /// Update connected peer count
    pub fn update_peer_count(&mut self, count: usize) {
        self.connected_peers = count;
    }

    /// Get success rate for connections
    pub fn connection_success_rate(&self) -> f64 {
        if self.connection_attempts == 0 {
            0.0
        } else {
            (self.connection_attempts - self.failed_connections) as f64 / self.connection_attempts as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    fn create_test_peer() -> PeerId {
        let keypair = Keypair::generate_ed25519();
        PeerId::from(keypair.public())
    }

    #[tokio::test]
    async fn test_network_creation() {
        let mut network = BitswapNetwork::new();
        assert!(!network.started);
        assert_eq!(network.connected_peers().len(), 0);
        
        // Should start successfully
        assert!(network.start().await.is_ok());
        assert!(network.started);
        
        // Should stop successfully
        assert!(network.stop().await.is_ok());
        assert!(!network.started);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let mut network = BitswapNetwork::new();
        let peer = create_test_peer();
        
        // Add peer
        assert!(network.add_peer(peer).is_ok());
        assert!(network.is_peer_connected(&peer));
        assert_eq!(network.peer_count(), 1);
        
        // Remove peer
        assert!(network.remove_peer(&peer).is_ok());
        assert!(!network.is_peer_connected(&peer));
        assert_eq!(network.peer_count(), 0);
    }

    #[tokio::test]
    async fn test_message_sending_without_start() {
        let mut network = BitswapNetwork::new();
        let peer = create_test_peer();
        let message = BitswapMessage::new();
        
        // Should fail when not started
        assert!(network.send_message(&peer, message).await.is_err());
    }

    #[tokio::test]
    async fn test_message_sending_to_disconnected_peer() {
        let mut network = BitswapNetwork::new();
        let peer = create_test_peer();
        let message = BitswapMessage::new();
        
        assert!(network.start().await.is_ok());
        
        // Should fail when peer not connected
        assert!(network.send_message(&peer, message).await.is_err());
    }

    #[tokio::test]
    async fn test_peer_statistics() {
        let mut network = BitswapNetwork::new();
        let peer = create_test_peer();
        
        network.add_peer(peer).unwrap();
        
        // Initial stats
        let conn = network.get_peer_connection(&peer).unwrap();
        assert_eq!(conn.messages_sent, 0);
        assert_eq!(conn.messages_received, 0);
        
        // Update stats
        network.update_peer_stats(&peer, true);  // sent
        network.update_peer_stats(&peer, false); // received
        
        let conn = network.get_peer_connection(&peer).unwrap();
        assert_eq!(conn.messages_sent, 1);
        assert_eq!(conn.messages_received, 1);
    }

    #[test]
    fn test_network_stats() {
        let mut stats = NetworkStats::new();
        
        assert_eq!(stats.messages_sent, 0);
        assert_eq!(stats.connection_success_rate(), 0.0);
        
        stats.record_sent(100);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.bytes_sent, 100);
        
        stats.record_connection_attempt(true);
        stats.record_connection_attempt(false);
        assert_eq!(stats.connection_attempts, 2);
        assert_eq!(stats.failed_connections, 1);
        assert_eq!(stats.connection_success_rate(), 0.5);
    }
}