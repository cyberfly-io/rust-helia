//! WantList manager for Bitswap protocol
//! Based on @helia/bitswap/src/want-list.ts

use crate::{
    constants::*,
    network_new::{Network, NetworkEvent},
    pb::{BitswapMessage as PbBitswapMessage, BlockPresenceType, WantType},
    utils::QueuedBitswapMessage,
    Result,
};
use bytes::Bytes;
use cid::Cid;
use helia_interface::HeliaError;
use libp2p::PeerId;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{oneshot, RwLock},
    time::sleep,
};
use tracing::{debug, info, trace};

/// Entry in a wantlist
#[derive(Debug, Clone)]
pub struct WantListEntry {
    pub cid: Cid,
    pub priority: i32,
    pub want_type: WantType,
    pub cancel: bool,
    pub send_dont_have: bool,
}

/// Want for a block with response channel
#[derive(Debug)]
struct BlockWant {
    cid: Cid,
    priority: i32,
    want_type: WantType,
    created_at: Instant,
    responders: Vec<oneshot::Sender<WantResult>>,
}

/// Result of a want operation
#[derive(Debug, Clone)]
pub struct WantResult {
    pub sender: PeerId,
    pub has: bool,
    pub cid: Cid,
    pub block: Option<Bytes>,
}

/// Session want for a specific peer
#[derive(Debug)]
struct SessionWant {
    cid: Cid,
    peer: PeerId,
    priority: i32,
    created_at: Instant,
    responder: oneshot::Sender<WantResult>,
}

/// WantList manager
pub struct WantList {
    /// Network reference
    network: Arc<Network>,
    /// Connected peers
    pub peers: Arc<RwLock<HashMap<PeerId, HashSet<Cid>>>>,
    /// Active wants (CID -> BlockWant)
    wants: Arc<RwLock<HashMap<Cid, BlockWant>>>,
    /// Session wants
    session_wants: Arc<RwLock<Vec<SessionWant>>>,
    /// Send message delay
    send_messages_delay: Duration,
    /// Running flag
    running: Arc<RwLock<bool>>,
    /// Message send task handle
    send_task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl WantList {
    /// Create a new WantList
    pub fn new(network: Arc<Network>) -> Self {
        Self {
            network,
            peers: Arc::new(RwLock::new(HashMap::new())),
            wants: Arc::new(RwLock::new(HashMap::new())),
            session_wants: Arc::new(RwLock::new(Vec::new())),
            send_messages_delay: Duration::from_millis(DEFAULT_MESSAGE_SEND_DELAY),
            running: Arc::new(RwLock::new(false)),
            send_task_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the wantlist manager
    pub fn start(&self) {
        let running = self.running.clone();
        let network = self.network.clone();
        let peers = self.peers.clone();
        let wants = self.wants.clone();
        let send_delay = self.send_messages_delay;
        let send_task_handle = self.send_task_handle.clone();

        let handle = tokio::spawn(async move {
            *running.write().await = true;
            info!("WantList manager started");

            // Start message processing loop
            while *running.read().await {
                // Process network events
                if let Some(event) = network.next_event().await {
                    match event {
                        NetworkEvent::BitswapMessage(msg_event) => {
                            // Handle incoming message
                            Self::handle_message_static(
                                msg_event.peer,
                                msg_event.message,
                                wants.clone(),
                            )
                            .await;
                        }
                        NetworkEvent::PeerConnected(peer) => {
                            debug!("Peer connected: {}", peer);
                            peers.write().await.insert(peer, HashSet::new());
                        }
                        NetworkEvent::PeerDisconnected(peer) => {
                            debug!("Peer disconnected: {}", peer);
                            peers.write().await.remove(&peer);
                        }
                    }
                }

                // Send pending messages
                sleep(send_delay).await;
            }

            info!("WantList manager stopped");
        });

        // Spawn a task to store the handle since we can't await in a non-async fn
        let send_task_handle_clone = send_task_handle.clone();
        tokio::spawn(async move {
            *send_task_handle_clone.write().await = Some(handle);
        });
    }

    /// Stop the wantlist manager
    pub async fn stop(&self) {
        *self.running.write().await = false;

        // Wait for task to complete
        if let Some(handle) = self.send_task_handle.write().await.take() {
            let _ = handle.await;
        }

        // Clear state
        self.wants.write().await.clear();
        self.session_wants.write().await.clear();
        self.peers.write().await.clear();
    }

    /// Want a block from any peer
    pub async fn want_block(&self, cid: Cid, priority: i32) -> Result<Bytes> {
        debug!("Wanting block: {}", cid);

        let (tx, rx) = oneshot::channel();

        // Add to wants
        {
            let mut wants = self.wants.write().await;
            let want = wants.entry(cid.clone()).or_insert_with(|| BlockWant {
                cid: cid.clone(),
                priority,
                want_type: WantType::WantBlock,
                created_at: Instant::now(),
                responders: Vec::new(),
            });
            want.responders.push(tx);
        }

        // Send want to all connected peers
        self.send_wants_to_peers().await?;

        // Wait for response with timeout
        let result = tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| HeliaError::other("Want timeout"))?
            .map_err(|_| HeliaError::other("Want cancelled"))?;

        if let Some(block) = result.block {
            Ok(block)
        } else {
            Err(HeliaError::BlockNotFound { cid })
        }
    }

    /// Want a block from a specific peer (session want)
    pub async fn want_session_block(
        &self,
        cid: Cid,
        peer: PeerId,
        priority: i32,
    ) -> Result<WantResult> {
        debug!("Wanting block {} from peer {}", cid, peer);

        let (tx, rx) = oneshot::channel();

        // Add session want
        {
            let mut session_wants = self.session_wants.write().await;
            session_wants.push(SessionWant {
                cid: cid.clone(),
                peer,
                priority,
                created_at: Instant::now(),
                responder: tx,
            });
        }

        // Send want to specific peer
        self.send_want_to_peer(peer, cid.clone(), priority).await?;

        // Wait for response
        let result = tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| HeliaError::other("Session want timeout"))?
            .map_err(|_| HeliaError::other("Session want cancelled"))?;

        Ok(result)
    }

    /// Notify that a block was received
    pub async fn received_block(&self, cid: &Cid) -> Result<()> {
        debug!("Received block: {}", cid);

        // Remove from wants and notify responders
        if let Some(mut want) = self.wants.write().await.remove(cid) {
            let result = WantResult {
                sender: PeerId::random(), // Will be set properly when integrated
                has: true,
                cid: cid.clone(),
                block: None, // Block data comes from blockstore
            };

            for responder in want.responders.drain(..) {
                let _ = responder.send(result.clone());
            }
        }

        // Check session wants
        let mut session_wants = self.session_wants.write().await;
        let mut remaining_wants = Vec::new();
        for sw in session_wants.drain(..) {
            if &sw.cid == cid {
                let result = WantResult {
                    sender: sw.peer,
                    has: true,
                    cid: cid.clone(),
                    block: None,
                };
                let _ = sw.responder.send(result);
            } else {
                remaining_wants.push(sw);
            }
        }
        *session_wants = remaining_wants;

        Ok(())
    }

    /// Send wants to all connected peers
    async fn send_wants_to_peers(&self) -> Result<()> {
        let peers: Vec<PeerId> = self.peers.read().await.keys().cloned().collect();
        let wants = self.wants.read().await;

        for peer in peers {
            let mut message = QueuedBitswapMessage::new();

            for (cid, want) in wants.iter() {
                message.add_want_block(cid, want.priority);
            }

            if !message.is_empty() {
                self.network.send_message(peer, message).await?;
                trace!("Queued want message to {} via swarm", peer);
            }
        }

        Ok(())
    }

    /// Send want to a specific peer
    async fn send_want_to_peer(&self, peer: PeerId, cid: Cid, priority: i32) -> Result<()> {
        let mut message = QueuedBitswapMessage::new();
        message.add_want_block(&cid, priority);

        self.network.send_message(peer, message).await?;
        trace!("Queued session want for {} to {}", cid, peer);

        Ok(())
    }

    /// Handle incoming message (static method for use in task)
    async fn handle_message_static(
        peer: PeerId,
        message: PbBitswapMessage,
        wants: Arc<RwLock<HashMap<Cid, BlockWant>>>,
    ) {
        trace!("Handling message from {}", peer);

        // Process blocks
        for block in message.blocks {
            if let Ok(cid) = Cid::try_from(&block.prefix[..]) {
                debug!("Received block {} from {}", cid, peer);

                // Notify wants
                if let Some(mut want) = wants.write().await.remove(&cid) {
                    let result = WantResult {
                        sender: peer,
                        has: true,
                        cid: cid.clone(),
                        block: Some(Bytes::from(block.data)),
                    };

                    for responder in want.responders.drain(..) {
                        let _ = responder.send(result.clone());
                    }
                }
            }
        }

        // Process block presences
        for presence in message.block_presences {
            if let Ok(cid) = Cid::try_from(&presence.cid[..]) {
                let presence_type = BlockPresenceType::from(presence.r#type);
                let has = matches!(presence_type, BlockPresenceType::HaveBlock);

                debug!("Received presence for {} from {}: has={}", cid, peer, has);

                // Notify wants about presence
                if !has {
                    // Peer doesn't have the block
                    if let Some(_want) = wants.read().await.get(&cid) {
                        // Could implement fallback to other peers here
                    }
                }
            }
        }
    }

    /// Get current wantlist
    pub async fn get_wantlist(&self) -> Vec<WantListEntry> {
        self.wants
            .read()
            .await
            .values()
            .map(|want| WantListEntry {
                cid: want.cid.clone(),
                priority: want.priority,
                want_type: want.want_type,
                cancel: false,
                send_dont_have: true,
            })
            .collect()
    }

    /// Dispatch a synthetic network event directly to the wantlist network
    pub fn dispatch_event(&self, event: NetworkEvent) {
        self.network.dispatch_event(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_new::NetworkInit;

    #[tokio::test]
    async fn test_wantlist_creation() {
        let sender_slot = Arc::new(RwLock::new(None));
        let network = Arc::new(Network::new(NetworkInit::default(), sender_slot));
        let wantlist = WantList::new(network);

        let entries = wantlist.get_wantlist().await;
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_add_peer() {
        let sender_slot = Arc::new(RwLock::new(None));
        let network = Arc::new(Network::new(NetworkInit::default(), sender_slot));
        let wantlist = WantList::new(network);

        let peer = PeerId::random();
        wantlist.peers.write().await.insert(peer, HashSet::new());

        assert_eq!(wantlist.peers.read().await.len(), 1);
    }
}
