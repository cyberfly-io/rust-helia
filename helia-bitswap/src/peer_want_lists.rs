//! Peer WantLists - tracks what each peer wants from us
//! Based on @helia/bitswap/src/peer-want-lists/

use crate::{
    constants::*,
    pb::{WantType, BlockPresenceType},
    utils::QueuedBitswapMessage,
};
use bytes::Bytes;
use cid::Cid;
use libp2p::PeerId;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Entry in a peer's wantlist
#[derive(Debug, Clone)]
pub struct PeerWant {
    pub cid: Cid,
    pub priority: i32,
    pub want_type: WantType,
    pub send_dont_have: bool,
    pub created_at: Instant,
}

/// Wantlist for a single peer
#[derive(Debug)]
struct PeerWantList {
    peer: PeerId,
    wants: HashMap<Cid, PeerWant>,
}

impl PeerWantList {
    fn new(peer: PeerId) -> Self {
        Self {
            peer,
            wants: HashMap::new(),
        }
    }

    fn add_want(&mut self, cid: Cid, priority: i32, want_type: WantType, send_dont_have: bool) {
        self.wants.insert(
            cid.clone(),
            PeerWant {
                cid,
                priority,
                want_type,
                send_dont_have,
                created_at: Instant::now(),
            },
        );
    }

    fn remove_want(&mut self, cid: &Cid) -> Option<PeerWant> {
        self.wants.remove(cid)
    }

    fn has_want(&self, cid: &Cid) -> bool {
        self.wants.contains_key(cid)
    }

    fn wants_block(&self, cid: &Cid) -> bool {
        self.wants
            .get(cid)
            .map(|w| matches!(w.want_type, WantType::WantBlock))
            .unwrap_or(false)
    }

    fn wants_have(&self, cid: &Cid) -> bool {
        self.wants
            .get(cid)
            .map(|w| matches!(w.want_type, WantType::WantHave))
            .unwrap_or(false)
    }

    fn get_wants(&self) -> Vec<&PeerWant> {
        self.wants.values().collect()
    }
}

/// Manager for all peer wantlists
pub struct PeerWantLists {
    /// Peer wantlists (PeerId -> PeerWantList)
    peers: Arc<RwLock<HashMap<PeerId, PeerWantList>>>,
}

impl PeerWantLists {
    /// Create a new PeerWantLists manager
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a peer
    pub async fn add_peer(&self, peer: PeerId) {
        let mut peers = self.peers.write().await;
        if !peers.contains_key(&peer) {
            debug!("Adding peer wantlist for {}", peer);
            peers.insert(peer, PeerWantList::new(peer));
        }
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer: &PeerId) {
        let mut peers = self.peers.write().await;
        if peers.remove(peer).is_some() {
            debug!("Removed peer wantlist for {}", peer);
        }
    }

    /// Add a want for a peer
    pub async fn add_want(
        &self,
        peer: PeerId,
        cid: Cid,
        priority: i32,
        want_type: WantType,
        send_dont_have: bool,
    ) {
        let mut peers = self.peers.write().await;
        let peer_wantlist = peers
            .entry(peer)
            .or_insert_with(|| PeerWantList::new(peer));

        trace!(
            "Peer {} wants {} (type: {:?}, priority: {})",
            peer,
            cid,
            want_type,
            priority
        );
        peer_wantlist.add_want(cid, priority, want_type, send_dont_have);
    }

    /// Remove a want for a peer
    pub async fn remove_want(&self, peer: &PeerId, cid: &Cid) -> bool {
        let mut peers = self.peers.write().await;
        if let Some(peer_wantlist) = peers.get_mut(peer) {
            if peer_wantlist.remove_want(cid).is_some() {
                trace!("Peer {} no longer wants {}", peer, cid);
                return true;
            }
        }
        false
    }

    /// Check if a peer wants a CID
    pub async fn has_want(&self, peer: &PeerId, cid: &Cid) -> bool {
        let peers = self.peers.read().await;
        peers
            .get(peer)
            .map(|p| p.has_want(cid))
            .unwrap_or(false)
    }

    /// Check if a peer wants the full block
    pub async fn wants_block(&self, peer: &PeerId, cid: &Cid) -> bool {
        let peers = self.peers.read().await;
        peers
            .get(peer)
            .map(|p| p.wants_block(cid))
            .unwrap_or(false)
    }

    /// Check if a peer wants to know if we have the block
    pub async fn wants_have(&self, peer: &PeerId, cid: &Cid) -> bool {
        let peers = self.peers.read().await;
        peers
            .get(peer)
            .map(|p| p.wants_have(cid))
            .unwrap_or(false)
    }

    /// Get all peers that want a CID
    pub async fn get_peers_wanting(&self, cid: &Cid) -> Vec<PeerId> {
        let peers = self.peers.read().await;
        peers
            .iter()
            .filter(|(_, wantlist)| wantlist.has_want(cid))
            .map(|(peer, _)| *peer)
            .collect()
    }

    /// Get all peers that want the full block
    pub async fn get_peers_wanting_block(&self, cid: &Cid) -> Vec<PeerId> {
        let peers = self.peers.read().await;
        peers
            .iter()
            .filter(|(_, wantlist)| wantlist.wants_block(cid))
            .map(|(peer, _)| *peer)
            .collect()
    }

    /// Get all wants for a peer
    pub async fn get_peer_wants(&self, peer: &PeerId) -> Vec<Cid> {
        let peers = self.peers.read().await;
        peers
            .get(peer)
            .map(|p| p.wants.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Notify that a block was received
    /// Returns list of peers to send the block to
    pub async fn received_block(&self, cid: &Cid) -> Vec<PeerId> {
        debug!("Received block {}, checking peer wants", cid);
        self.get_peers_wanting_block(cid).await
    }

    /// Create messages to send to peers about a new block
    pub async fn create_block_messages(
        &self,
        cid: &Cid,
        block_data: &Bytes,
    ) -> HashMap<PeerId, QueuedBitswapMessage> {
        let peers = self.peers.read().await;
        let mut messages = HashMap::new();

        for (peer, wantlist) in peers.iter() {
            if wantlist.has_want(cid) {
                let mut message = QueuedBitswapMessage::new();

                if wantlist.wants_block(cid) {
                    // Send full block
                    let prefix = crate::utils::cid_to_prefix(cid);
                    message.add_block(cid, prefix, block_data.to_vec());
                    trace!("Creating block message for peer {}", peer);
                } else if wantlist.wants_have(cid) {
                    // Send block presence (have)
                    message.add_block_presence(cid, BlockPresenceType::HaveBlock);
                    trace!("Creating presence message for peer {}", peer);
                }

                if !message.is_empty() {
                    messages.insert(*peer, message);
                }
            }
        }

        debug!("Created {} messages for block {}", messages.len(), cid);
        messages
    }

    /// Create "don't have" messages for peers
    pub async fn create_dont_have_messages(&self, cid: &Cid) -> HashMap<PeerId, QueuedBitswapMessage> {
        let peers = self.peers.read().await;
        let mut messages = HashMap::new();

        for (peer, wantlist) in peers.iter() {
            if let Some(want) = wantlist.wants.get(cid) {
                if want.send_dont_have {
                    let mut message = QueuedBitswapMessage::new();
                    message.add_block_presence(cid, BlockPresenceType::DoNotHaveBlock);
                    messages.insert(*peer, message);
                    trace!("Creating don't-have message for peer {}", peer);
                }
            }
        }

        debug!("Created {} don't-have messages for {}", messages.len(), cid);
        messages
    }

    /// Get statistics
    pub async fn stats(&self) -> PeerWantListsStats {
        let peers = self.peers.read().await;
        let total_wants: usize = peers.values().map(|p| p.wants.len()).sum();

        PeerWantListsStats {
            num_peers: peers.len(),
            total_wants,
        }
    }
}

impl Default for PeerWantLists {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for peer wantlists
#[derive(Debug, Clone)]
pub struct PeerWantListsStats {
    pub num_peers: usize,
    pub total_wants: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_peer() {
        let peer_wants = PeerWantLists::new();
        let peer = PeerId::random();

        peer_wants.add_peer(peer).await;

        let stats = peer_wants.stats().await;
        assert_eq!(stats.num_peers, 1);
        assert_eq!(stats.total_wants, 0);
    }

    #[tokio::test]
    async fn test_add_want() {
        let peer_wants = PeerWantLists::new();
        let peer = PeerId::random();
        let cid = Cid::default();

        peer_wants.add_peer(peer).await;
        peer_wants
            .add_want(peer, cid.clone(), 1, WantType::WantBlock, true)
            .await;

        assert!(peer_wants.has_want(&peer, &cid).await);
        assert!(peer_wants.wants_block(&peer, &cid).await);

        let stats = peer_wants.stats().await;
        assert_eq!(stats.total_wants, 1);
    }

    #[tokio::test]
    async fn test_remove_want() {
        let peer_wants = PeerWantLists::new();
        let peer = PeerId::random();
        let cid = Cid::default();

        peer_wants.add_peer(peer).await;
        peer_wants
            .add_want(peer, cid.clone(), 1, WantType::WantBlock, true)
            .await;

        assert!(peer_wants.remove_want(&peer, &cid).await);
        assert!(!peer_wants.has_want(&peer, &cid).await);
    }

    #[tokio::test]
    async fn test_get_peers_wanting() {
        let peer_wants = PeerWantLists::new();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let cid = Cid::default();

        peer_wants.add_peer(peer1).await;
        peer_wants.add_peer(peer2).await;

        peer_wants
            .add_want(peer1, cid.clone(), 1, WantType::WantBlock, true)
            .await;
        peer_wants
            .add_want(peer2, cid.clone(), 1, WantType::WantHave, true)
            .await;

        let wanting = peer_wants.get_peers_wanting(&cid).await;
        assert_eq!(wanting.len(), 2);

        let wanting_block = peer_wants.get_peers_wanting_block(&cid).await;
        assert_eq!(wanting_block.len(), 1);
        assert_eq!(wanting_block[0], peer1);
    }
}
