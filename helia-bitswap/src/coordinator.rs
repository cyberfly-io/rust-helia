//! Bitswap coordinator - High-level API for block exchange
//! Based on @helia/bitswap/src/index.ts

use crate::{
    constants::*,
    network_new::{Network, NetworkEvent, NetworkInit},
    wantlist_new::{WantList, WantResult},
    pb::{self, BlockPresenceType, WantType},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use helia_interface::{Blocks, HeliaError};
use libp2p::PeerId;
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

/// Bitswap statistics
#[derive(Debug, Clone, Default)]
pub struct BitswapStats {
    /// Total blocks sent
    pub blocks_sent: u64,
    /// Total blocks received
    pub blocks_received: u64,
    /// Total data sent (bytes)
    pub data_sent: u64,
    /// Total data received (bytes)
    pub data_received: u64,
    /// Total duplicate blocks received
    pub dup_blocks_received: u64,
    /// Total duplicate data received (bytes)
    pub dup_data_received: u64,
    /// Messages received
    pub messages_received: u64,
    /// Blocks sent by peer
    pub blocks_sent_by_peer: HashMap<PeerId, u64>,
    /// Blocks received by peer
    pub blocks_received_by_peer: HashMap<PeerId, u64>,
}

/// Options for wanting a block
#[derive(Debug, Clone)]
pub struct WantOptions {
    /// Timeout for the want operation
    pub timeout: Option<Duration>,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Whether to accept block presence messages
    pub accept_block_presence: bool,
    /// Specific peer to request from (for session-based requests)
    pub peer: Option<PeerId>,
}

impl Default for WantOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_millis(DEFAULT_WANT_TIMEOUT)),
            priority: DEFAULT_PRIORITY,
            accept_block_presence: true,
            peer: None,
        }
    }
}

/// Options for notifying new blocks
#[derive(Debug, Clone, Default)]
pub struct NotifyOptions {
    /// Whether to announce to all connected peers
    pub broadcast: bool,
}

/// Bitswap configuration
#[derive(Debug, Clone)]
pub struct BitswapConfig {
    /// Network configuration
    pub network: NetworkInit,
}

impl Default for BitswapConfig {
    fn default() -> Self {
        Self {
            network: NetworkInit::default(),
        }
    }
}

/// Main Bitswap coordinator
/// 
/// Provides high-level API for block exchange via the Bitswap protocol.
/// 
/// # Example
/// 
/// ```no_run
/// use helia_bitswap::{Bitswap, BitswapConfig, WantOptions};
/// use bytes::Bytes;
/// use cid::Cid;
/// use std::sync::Arc;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create blockstore (example - use appropriate implementation)
/// # use helia_interface::Blocks;
/// # let blockstore: Arc<dyn Blocks> = unimplemented!();
/// let bitswap = Bitswap::new(blockstore, BitswapConfig::default()).await?;
/// 
/// // Start the bitswap node
/// bitswap.start().await?;
/// 
/// // Want a block
/// let cid: Cid = "QmHash".parse()?;
/// let block = bitswap.want(&cid, WantOptions::default()).await?;
/// 
/// // Notify that we have new blocks
/// let blocks = vec![(cid, block)];
/// bitswap.notify_new_blocks(blocks, Default::default()).await?;
/// 
/// // Stop the bitswap node
/// bitswap.stop().await?;
/// # Ok(())
/// # }
/// ```
/// Outbound message request to be sent via the swarm
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub peer: PeerId,
    pub message: pb::BitswapMessage,
}

pub struct Bitswap {
    /// Network layer (deprecated - kept for compatibility)
    network: Arc<RwLock<Network>>,
    /// WantList manager
    wantlist: Arc<WantList>,
    /// Blockstore for local block storage
    pub(crate) blockstore: Arc<dyn Blocks>,
    /// Statistics
    stats: Arc<RwLock<BitswapStats>>,
    /// Running flag
    running: Arc<RwLock<bool>>,
    /// Configuration
    config: BitswapConfig,
    /// Channel for sending messages via the swarm
    outbound_tx: Option<tokio::sync::mpsc::UnboundedSender<OutboundMessage>>,
    /// Connected peers
    connected_peers: Arc<RwLock<Vec<PeerId>>>,
    /// Block notification broadcast channel (for event-driven want resolution)
    block_notify_tx: tokio::sync::broadcast::Sender<Cid>,
}

impl Bitswap {
    /// Create a new Bitswap coordinator
    pub async fn new(blockstore: Arc<dyn Blocks>, config: BitswapConfig) -> Result<Self> {
        info!("Creating Bitswap coordinator");
        
        // Create network (kept for compatibility, but messages should go through swarm)
        let network = Arc::new(RwLock::new(Network::new(config.network.clone())));
        
        // Create wantlist with a separate Network instance
        let network_for_wantlist = Arc::new(Network::new(config.network.clone()));
        let wantlist = Arc::new(WantList::new(network_for_wantlist));
        
        // Create block notification channel (capacity of 1000 pending notifications)
        let (block_notify_tx, _) = tokio::sync::broadcast::channel(1000);
        
        Ok(Self {
            network,
            wantlist,
            blockstore,
            stats: Arc::new(RwLock::new(BitswapStats::default())),
            running: Arc::new(RwLock::new(false)),
            config,
            outbound_tx: None,
            connected_peers: Arc::new(RwLock::new(Vec::new())),
            block_notify_tx,
        })
    }
    
    /// Set the outbound message sender (connected to swarm)
    pub fn set_outbound_sender(&mut self, tx: tokio::sync::mpsc::UnboundedSender<OutboundMessage>) {
        self.outbound_tx = Some(tx);
        info!("Bitswap coordinator connected to swarm message channel");
    }
    
    /// Add a connected peer
    pub async fn add_peer(&self, peer: PeerId) {
        let mut peers = self.connected_peers.write().await;
        if !peers.contains(&peer) {
            peers.push(peer);
            info!("Bitswap: Added peer {}", peer);
        }
    }
    
    /// Remove a disconnected peer
    pub async fn remove_peer(&self, peer: &PeerId) {
        let mut peers = self.connected_peers.write().await;
        peers.retain(|p| p != peer);
        info!("Bitswap: Removed peer {}", peer);
    }
    
    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerId> {
        self.connected_peers.read().await.clone()
    }
    
    /// Send a message via the swarm
    fn send_via_swarm(&self, peer: PeerId, message: pb::BitswapMessage) -> Result<()> {
        if let Some(tx) = &self.outbound_tx {
            tx.send(OutboundMessage { peer, message })
                .map_err(|e| HeliaError::network(format!("Failed to queue outbound message: {}", e)))?;
            Ok(())
        } else {
            Err(HeliaError::network("Outbound message channel not connected to swarm"))
        }
    }
    
    /// Broadcast WANT for a block to connected peers via swarm
    pub fn broadcast_want_via_swarm(&self, cid: &Cid, priority: i32, peers: Vec<PeerId>) -> Result<()> {
        if peers.is_empty() {
            debug!("No peers to send WANT to");
            return Ok(());
        }
        
        // Build wantlist message
        let wantlist_entry = pb::WantlistEntry {
            cid: cid.to_bytes(),
            priority,
            cancel: false,
            want_type: pb::WantType::WantBlock as i32,
            send_dont_have: true,
        };
        
        let message = pb::BitswapMessage {
            wantlist: Some(pb::Wantlist {
                entries: vec![wantlist_entry],
                full: false,
            }),
            blocks: Vec::new(),
            block_presences: Vec::new(),
            pending_bytes: 0,
        };
        
        // Send to all peers
        for peer in peers {
            debug!("Sending WANT for {} to peer {} via swarm", cid, peer);
            if let Err(e) = self.send_via_swarm(peer, message.clone()) {
                warn!("Failed to send WANT to peer {}: {}", peer, e);
            }
        }
        
        Ok(())
    }
    
    /// Start the Bitswap coordinator
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        
        info!("Starting Bitswap coordinator");
        
        // Start network
        self.network.write().await.start().await?;
        
        // Start wantlist
        self.wantlist.start();
        
        *running = true;
        info!("Bitswap coordinator started");
        Ok(())
    }
    
    /// Stop the Bitswap coordinator
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        
        info!("Stopping Bitswap coordinator");
        
        // Stop wantlist
        self.wantlist.stop().await;
        
        // Stop network
        self.network.write().await.stop().await?;
        
        *running = false;
        info!("Bitswap coordinator stopped");
        Ok(())
    }
    
    /// Want a block
    /// 
    /// Requests a block from the network. This will:
    /// 1. Check local blockstore first
    /// 2. If not found locally, add to wantlist
    /// 3. Send want messages to connected peers
    /// 4. Wait for block to arrive or timeout (EVENT-DRIVEN, not polling)
    /// 
    /// # Arguments
    /// 
    /// * `cid` - CID of the block to retrieve
    /// * `options` - Want options (timeout, priority, etc.)
    /// 
    /// # Returns
    /// 
    /// The block data if found, or an error if timeout or not found
    pub async fn want(&self, cid: &Cid, options: WantOptions) -> Result<Bytes> {
        debug!("Wanting block: {}", cid);
        
        // Check if we already have it
        if let Ok(block) = self.blockstore.get(cid, None).await {
            debug!("Block {} found in local blockstore", cid);
            return Ok(block);
        }
        
        // Send WANT via swarm to connected peers
        let peers = self.get_connected_peers().await;
        if peers.is_empty() {
            return Err(HeliaError::network("No connected peers to request block from"));
        }
        
        info!("Sending WANT for {} to {} peers via swarm", cid, peers.len());
        self.broadcast_want_via_swarm(cid, options.priority, peers)?;
        
        // Subscribe to block notifications BEFORE sending want
        let mut block_rx = self.block_notify_tx.subscribe();
        let target_cid = cid.clone();
        
        // Wait for the block to arrive with timeout (EVENT-DRIVEN)
        let timeout = options.timeout.unwrap_or(Duration::from_secs(30));
        
        // Use tokio::select to wait for either block notification or timeout
        tokio::select! {
            _ = tokio::time::sleep(timeout) => {
                debug!("Timeout waiting for block {}", target_cid);
                Err(HeliaError::Timeout)
            }
            result = async {
                loop {
                    // Wait for block notification
                    match block_rx.recv().await {
                        Ok(received_cid) => {
                            if received_cid == target_cid {
                                // This is our block! Try to get it from blockstore
                                match self.blockstore.get(&target_cid, None).await {
                                    Ok(block) => {
                                        debug!("Block {} received from network", target_cid);
                                        
                                        // Update stats
                                        let mut stats = self.stats.write().await;
                                        stats.blocks_received += 1;
                                        stats.data_received += block.len() as u64;
                                        
                                        return Ok(block);
                                    }
                                    Err(e) => {
                                        // Block was notified but not in blockstore? Strange, keep waiting
                                        warn!("Block {} notified but not in blockstore: {}", target_cid, e);
                                    }
                                }
                            }
                            // Not our block, keep waiting
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            // Channel lagged, check if block arrived while we were catching up
                            if let Ok(block) = self.blockstore.get(&target_cid, None).await {
                                debug!("Block {} found in blockstore after channel lag", target_cid);
                                
                                let mut stats = self.stats.write().await;
                                stats.blocks_received += 1;
                                stats.data_received += block.len() as u64;
                                
                                return Ok(block);
                            }
                            // Not found, continue waiting
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            return Err(HeliaError::network("Block notification channel closed"));
                        }
                    }
                }
            } => result
        }
    }
    
    /// Notify that we have new blocks
    /// 
    /// Announces to connected peers that we have these blocks.
    /// This allows peers that are waiting for these blocks to request them.
    /// Also broadcasts internally to wake up any local want() calls.
    /// 
    /// # Arguments
    /// 
    /// * `blocks` - Vector of (CID, block data) pairs
    /// * `options` - Notify options
    pub async fn notify_new_blocks(
        &self,
        blocks: Vec<(Cid, Bytes)>,
        _options: NotifyOptions,
    ) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }
        
        debug!("Notifying {} new blocks", blocks.len());
        
        // Store blocks in blockstore
        for (cid, block) in &blocks {
            self.blockstore.put(cid, block.clone(), None).await?;
        }
        
        // Notify wantlist about the blocks
        for (cid, _block) in &blocks {
            self.wantlist.received_block(cid).await?;
            
            // Broadcast block arrival to wake up waiting want() calls
            // Ignore send errors (no receivers is fine)
            let _ = self.block_notify_tx.send(cid.clone());
            trace!("Broadcasted block notification for {}", cid);
        }
        
        Ok(())
    }
    
    /// Notify a single block arrival (called from event loop when block received)
    /// 
    /// This is the KEY optimization from JS Helia - immediately notify waiting
    /// requests instead of having them poll.
    pub fn notify_block_received(&self, cid: &Cid) {
        // Broadcast block arrival
        let _ = self.block_notify_tx.send(cid.clone());
        trace!("Broadcasted block notification for {}", cid);
    }
    
    
    /// Get current statistics
    pub async fn stats(&self) -> BitswapStats {
        self.stats.read().await.clone()
    }
    
    /// Get the wantlist
    pub fn wantlist(&self) -> Arc<WantList> {
        self.wantlist.clone()
    }
    
    /// Check if bitswap is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helia_utils::blockstore::SledBlockstore;
    use helia_utils::BlockstoreConfig;
    
    #[tokio::test]
    async fn test_bitswap_creation() {
        let blockstore = Arc::new(SledBlockstore::new(BlockstoreConfig::default()).unwrap());
        let config = BitswapConfig::default();
        let bitswap = Bitswap::new(blockstore, config).await;
        assert!(bitswap.is_ok());
    }
    
    #[tokio::test]
    async fn test_bitswap_start_stop() {
        let blockstore = Arc::new(SledBlockstore::new(BlockstoreConfig::default()).unwrap());
        let config = BitswapConfig::default();
        let bitswap = Bitswap::new(blockstore, config).await.unwrap();
        
        assert!(!bitswap.is_running().await);
        
        bitswap.start().await.unwrap();
        assert!(bitswap.is_running().await);
        
        bitswap.stop().await.unwrap();
        assert!(!bitswap.is_running().await);
    }
    
    #[tokio::test]
    async fn test_bitswap_stats() {
        let blockstore = Arc::new(SledBlockstore::new(BlockstoreConfig::default()).unwrap());
        let config = BitswapConfig::default();
        let bitswap = Bitswap::new(blockstore, config).await.unwrap();
        
        let stats = bitswap.stats().await;
        assert_eq!(stats.blocks_sent, 0);
        assert_eq!(stats.blocks_received, 0);
    }
}
