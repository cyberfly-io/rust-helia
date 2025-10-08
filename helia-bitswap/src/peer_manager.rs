use crate::{WantListEntry, Result};
use cid::Cid;
use helia_interface::HeliaError;
use libp2p::PeerId;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Manages peer connections and their wantlists
pub struct PeerManager {
    /// Connected peers and their information
    peers: HashMap<PeerId, PeerInfo>,
    /// Mapping from CID to peers that want it
    cid_to_peers: HashMap<Cid, HashSet<PeerId>>,
    /// Whether the manager is running
    running: bool,
}

/// Information about a connected peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: PeerId,
    /// When the peer was first connected
    pub connected_at: Instant,
    /// Last time we received a message from this peer
    pub last_seen: Instant,
    /// Peer's current wantlist
    pub wantlist: Vec<WantListEntry>,
    /// Number of blocks sent to this peer
    pub blocks_sent: u64,
    /// Number of blocks received from this peer
    pub blocks_received: u64,
    /// Total bytes sent to this peer
    pub bytes_sent: u64,
    /// Total bytes received from this peer
    pub bytes_received: u64,
    /// Connection quality score (0.0 to 1.0)
    pub quality_score: f64,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(peer_id: PeerId) -> Self {
        let now = Instant::now();
        Self {
            peer_id,
            connected_at: now,
            last_seen: now,
            wantlist: Vec::new(),
            blocks_sent: 0,
            blocks_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            quality_score: 1.0,
        }
    }

    /// Update last seen time
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Record a block sent to this peer
    pub fn record_block_sent(&mut self, size: usize) {
        self.blocks_sent += 1;
        self.bytes_sent += size as u64;
        self.update_last_seen();
    }

    /// Record a block received from this peer
    pub fn record_block_received(&mut self, size: usize) {
        self.blocks_received += 1;
        self.bytes_received += size as u64;
        self.update_last_seen();
    }

    /// Get connection duration
    pub fn connection_duration(&self) -> Duration {
        Instant::now().duration_since(self.connected_at)
    }

    /// Get time since last seen
    pub fn time_since_last_seen(&self) -> Duration {
        Instant::now().duration_since(self.last_seen)
    }

    /// Calculate and update quality score based on various factors
    pub fn update_quality_score(&mut self) {
        let mut score = 1.0;
        
        // Reduce score if peer hasn't been seen recently
        let time_since_seen = self.time_since_last_seen().as_secs() as f64;
        if time_since_seen > 300.0 { // 5 minutes
            score *= 0.5;
        }
        
        // Increase score based on blocks exchanged
        let total_blocks = self.blocks_sent + self.blocks_received;
        if total_blocks > 0 {
            score += (total_blocks as f64).log10() * 0.1;
        }
        
        // Cap the score at 1.0
        self.quality_score = score.min(1.0).max(0.0);
    }

    /// Check if peer is considered stale
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.time_since_last_seen() > timeout
    }
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            cid_to_peers: HashMap::new(),
            running: false,
        }
    }

    /// Start the peer manager
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }

        info!("Starting peer manager");
        self.running = true;
        Ok(())
    }

    /// Stop the peer manager
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        info!("Stopping peer manager");
        
        // Clean up all peer data
        self.peers.clear();
        self.cid_to_peers.clear();
        
        self.running = false;
        Ok(())
    }

    /// Add a new peer
    pub fn add_peer(&mut self, peer_id: PeerId) -> Result<()> {
        debug!("Adding peer: {}", peer_id);
        
        if self.peers.contains_key(&peer_id) {
            return Err(HeliaError::other(format!("Peer {} already exists", peer_id)));
        }

        let peer_info = PeerInfo::new(peer_id);
        self.peers.insert(peer_id, peer_info);
        
        Ok(())
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Result<()> {
        debug!("Removing peer: {}", peer_id);
        
        if let Some(peer_info) = self.peers.remove(peer_id) {
            // Remove peer from all CID mappings
            for entry in &peer_info.wantlist {
                if let Some(peer_set) = self.cid_to_peers.get_mut(&entry.cid) {
                    peer_set.remove(peer_id);
                    if peer_set.is_empty() {
                        self.cid_to_peers.remove(&entry.cid);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Update a peer's wantlist
    pub fn update_peer_wantlist(&mut self, peer_id: &PeerId, wantlist: Vec<WantListEntry>) -> Result<()> {
        debug!("Updating wantlist for peer {} with {} entries", peer_id, wantlist.len());
        
        let peer_info = self.peers.get_mut(peer_id)
            .ok_or_else(|| HeliaError::other(format!("Peer {} not found", peer_id)))?;

        // Remove old wantlist from CID mappings
        for entry in &peer_info.wantlist {
            if let Some(peer_set) = self.cid_to_peers.get_mut(&entry.cid) {
                peer_set.remove(peer_id);
                if peer_set.is_empty() {
                    self.cid_to_peers.remove(&entry.cid);
                }
            }
        }

        // Add new wantlist to CID mappings
        for entry in &wantlist {
            if !entry.cancel {
                self.cid_to_peers
                    .entry(entry.cid)
                    .or_insert_with(HashSet::new)
                    .insert(*peer_id);
            }
        }

        // Update peer's wantlist
        peer_info.wantlist = wantlist;
        peer_info.update_last_seen();
        
        Ok(())
    }

    /// Get peers that want a specific CID
    pub fn peers_wanting_block(&self, cid: &Cid) -> Vec<PeerId> {
        self.cid_to_peers.get(cid)
            .map(|peers| peers.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get a peer's wantlist
    pub fn get_peer_wantlist(&self, peer_id: &PeerId) -> Vec<WantListEntry> {
        self.peers.get(peer_id)
            .map(|info| info.wantlist.clone())
            .unwrap_or_default()
    }

    /// Get connected peers
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.peers.keys().cloned().collect()
    }

    /// Get peer information
    pub fn get_peer_info(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get mutable peer information
    pub fn get_peer_info_mut(&mut self, peer_id: &PeerId) -> Option<&mut PeerInfo> {
        self.peers.get_mut(peer_id)
    }

    /// Record a block sent to a peer
    pub fn record_block_sent(&mut self, peer_id: &PeerId, size: usize) -> Result<()> {
        let peer_info = self.peers.get_mut(peer_id)
            .ok_or_else(|| HeliaError::other(format!("Peer {} not found", peer_id)))?;
        
        peer_info.record_block_sent(size);
        Ok(())
    }

    /// Record a block received from a peer
    pub fn record_block_received(&mut self, peer_id: &PeerId, size: usize) -> Result<()> {
        let peer_info = self.peers.get_mut(peer_id)
            .ok_or_else(|| HeliaError::other(format!("Peer {} not found", peer_id)))?;
        
        peer_info.record_block_received(size);
        Ok(())
    }

    /// Get total number of peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get peers sorted by quality score
    pub fn get_peers_by_quality(&self) -> Vec<(PeerId, f64)> {
        let mut peers: Vec<_> = self.peers.iter()
            .map(|(peer_id, info)| (*peer_id, info.quality_score))
            .collect();
        
        peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        peers
    }

    /// Clean up stale peers
    pub fn cleanup_stale_peers(&mut self, timeout: Duration) -> usize {
        let stale_peers: Vec<PeerId> = self.peers.iter()
            .filter(|(_, info)| info.is_stale(timeout))
            .map(|(peer_id, _)| *peer_id)
            .collect();

        let count = stale_peers.len();
        
        for peer_id in stale_peers {
            warn!("Removing stale peer: {}", peer_id);
            let _ = self.remove_peer(&peer_id);
        }
        
        count
    }

    /// Update quality scores for all peers
    pub fn update_all_quality_scores(&mut self) {
        for peer_info in self.peers.values_mut() {
            peer_info.update_quality_score();
        }
    }

    /// Get summary statistics
    pub fn get_statistics(&self) -> PeerManagerStats {
        let mut stats = PeerManagerStats::default();
        
        stats.total_peers = self.peers.len();
        stats.total_cids_wanted = self.cid_to_peers.len();
        
        for info in self.peers.values() {
            stats.total_blocks_sent += info.blocks_sent;
            stats.total_blocks_received += info.blocks_received;
            stats.total_bytes_sent += info.bytes_sent;
            stats.total_bytes_received += info.bytes_received;
            
            if info.quality_score > 0.8 {
                stats.high_quality_peers += 1;
            }
        }
        
        if !self.peers.is_empty() {
            stats.average_quality_score = self.peers.values()
                .map(|info| info.quality_score)
                .sum::<f64>() / self.peers.len() as f64;
        }
        
        stats
    }
}

impl Default for PeerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the peer manager
#[derive(Debug, Clone, Default)]
pub struct PeerManagerStats {
    /// Total number of connected peers
    pub total_peers: usize,
    /// Total number of unique CIDs being wanted
    pub total_cids_wanted: usize,
    /// Total blocks sent to all peers
    pub total_blocks_sent: u64,
    /// Total blocks received from all peers
    pub total_blocks_received: u64,
    /// Total bytes sent to all peers
    pub total_bytes_sent: u64,
    /// Total bytes received from all peers
    pub total_bytes_received: u64,
    /// Number of high-quality peers (score > 0.8)
    pub high_quality_peers: usize,
    /// Average quality score across all peers
    pub average_quality_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WantType;
    use libp2p::identity::Keypair;

    fn create_test_peer() -> PeerId {
        let keypair = Keypair::generate_ed25519();
        PeerId::from(keypair.public())
    }

    #[tokio::test]
    async fn test_peer_manager_lifecycle() {
        let mut manager = PeerManager::new();
        assert!(!manager.running);
        
        assert!(manager.start().await.is_ok());
        assert!(manager.running);
        
        assert!(manager.stop().await.is_ok());
        assert!(!manager.running);
    }

    #[test]
    fn test_peer_management() {
        let mut manager = PeerManager::new();
        let peer = create_test_peer();
        
        // Add peer
        assert!(manager.add_peer(peer).is_ok());
        assert_eq!(manager.peer_count(), 1);
        assert!(manager.connected_peers().contains(&peer));
        
        // Can't add same peer twice
        assert!(manager.add_peer(peer).is_err());
        
        // Remove peer
        assert!(manager.remove_peer(&peer).is_ok());
        assert_eq!(manager.peer_count(), 0);
    }

    #[test]
    fn test_wantlist_management() {
        let mut manager = PeerManager::new();
        let peer = create_test_peer();
        let cid = Cid::default();
        
        manager.add_peer(peer).unwrap();
        
        let wantlist = vec![WantListEntry {
            cid,
            priority: 1,
            want_type: WantType::Block,
            cancel: false,
            send_dont_have: false,
        }];
        
        // Update wantlist
        assert!(manager.update_peer_wantlist(&peer, wantlist.clone()).is_ok());
        
        // Check wantlist was stored
        let stored_wantlist = manager.get_peer_wantlist(&peer);
        assert_eq!(stored_wantlist.len(), 1);
        assert_eq!(stored_wantlist[0].cid, cid);
        
        // Check CID to peer mapping
        let wanting_peers = manager.peers_wanting_block(&cid);
        assert_eq!(wanting_peers.len(), 1);
        assert!(wanting_peers.contains(&peer));
    }

    #[test]
    fn test_block_statistics() {
        let mut manager = PeerManager::new();
        let peer = create_test_peer();
        
        manager.add_peer(peer).unwrap();
        
        // Record blocks
        assert!(manager.record_block_sent(&peer, 100).is_ok());
        assert!(manager.record_block_received(&peer, 200).is_ok());
        
        let info = manager.get_peer_info(&peer).unwrap();
        assert_eq!(info.blocks_sent, 1);
        assert_eq!(info.blocks_received, 1);
        assert_eq!(info.bytes_sent, 100);
        assert_eq!(info.bytes_received, 200);
    }

    #[test]
    fn test_quality_scoring() {
        let mut info = PeerInfo::new(create_test_peer());
        
        // Initial quality should be 1.0
        assert_eq!(info.quality_score, 1.0);
        
        // Record some activity
        info.record_block_sent(100);
        info.record_block_received(100);
        info.update_quality_score();
        
        // Quality should still be good
        assert!(info.quality_score >= 1.0);
    }

    #[test]
    fn test_peer_statistics() {
        let mut manager = PeerManager::new();
        let peer1 = create_test_peer();
        let peer2 = create_test_peer();
        
        manager.add_peer(peer1).unwrap();
        manager.add_peer(peer2).unwrap();
        
        manager.record_block_sent(&peer1, 100).unwrap();
        manager.record_block_received(&peer2, 200).unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_peers, 2);
        assert_eq!(stats.total_blocks_sent, 1);
        assert_eq!(stats.total_blocks_received, 1);
        assert_eq!(stats.total_bytes_sent, 100);
        assert_eq!(stats.total_bytes_received, 200);
    }
}