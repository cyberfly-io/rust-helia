//! Network layer for Bitswap protocol
//! Based on @helia/bitswap/src/network.ts

use crate::{
    constants::*,
    coordinator::OutboundMessage,
    pb::BitswapMessage as PbBitswapMessage,
    utils::{merge_messages, split_message, QueuedBitswapMessage},
    Result,
};
use bytes::Bytes;
use cid::Cid;
use helia_interface::HeliaError;
use libp2p::PeerId;
use prost::Message;
use std::{
    collections::{HashMap, VecDeque},
    io::Cursor,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, trace};

/// Bitswap message event detail
#[derive(Debug, Clone)]
pub struct BitswapMessageEvent {
    pub peer: PeerId,
    pub message: PbBitswapMessage,
}

/// Network events
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Bitswap message received
    BitswapMessage(BitswapMessageEvent),
    /// Peer connected
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
}

/// Network component for Bitswap
pub struct Network {
    /// Supported protocol versions
    protocols: Vec<String>,
    /// Whether the network is running
    running: bool,
    /// Maximum inbound streams
    max_inbound_streams: usize,
    /// Maximum outbound streams
    max_outbound_streams: usize,
    /// Message receive timeout
    message_receive_timeout: Duration,
    /// Run on limited connections
    run_on_limited_connections: bool,
    /// Maximum incoming message size
    max_incoming_message_size: usize,
    /// Maximum outgoing message size
    max_outgoing_message_size: usize,
    /// Message send queue
    send_queue: Arc<RwLock<HashMap<PeerId, VecDeque<QueuedBitswapMessage>>>>,
    /// Event sender
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
    /// Event receiver
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<NetworkEvent>>>,
    /// Outbound swarm sender shared with coordinator
    outbound_tx: Arc<RwLock<Option<mpsc::UnboundedSender<OutboundMessage>>>>,
}

/// Network initialization parameters
#[derive(Debug, Clone)]
pub struct NetworkInit {
    pub protocols: Option<Vec<String>>,
    pub max_inbound_streams: Option<usize>,
    pub max_outbound_streams: Option<usize>,
    pub message_receive_timeout: Option<Duration>,
    pub message_send_concurrency: Option<usize>,
    pub run_on_limited_connections: Option<bool>,
    pub max_outgoing_message_size: Option<usize>,
    pub max_incoming_message_size: Option<usize>,
}

impl Default for NetworkInit {
    fn default() -> Self {
        Self {
            protocols: None,
            max_inbound_streams: Some(DEFAULT_MAX_INBOUND_STREAMS),
            max_outbound_streams: Some(DEFAULT_MAX_OUTBOUND_STREAMS),
            message_receive_timeout: Some(Duration::from_millis(DEFAULT_MESSAGE_RECEIVE_TIMEOUT)),
            message_send_concurrency: Some(DEFAULT_MESSAGE_SEND_CONCURRENCY),
            run_on_limited_connections: Some(DEFAULT_RUN_ON_TRANSIENT_CONNECTIONS),
            max_outgoing_message_size: Some(DEFAULT_MAX_OUTGOING_MESSAGE_SIZE),
            max_incoming_message_size: Some(DEFAULT_MAX_INCOMING_MESSAGE_SIZE),
        }
    }
}

impl Network {
    /// Create a new network
    pub fn new(
        init: NetworkInit,
        outbound_tx: Arc<RwLock<Option<mpsc::UnboundedSender<OutboundMessage>>>>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            protocols: init.protocols.unwrap_or_else(|| {
                vec![
                    BITSWAP_120.to_string(),
                    BITSWAP_110.to_string(),
                    BITSWAP_100.to_string(),
                ]
            }),
            running: false,
            max_inbound_streams: init
                .max_inbound_streams
                .unwrap_or(DEFAULT_MAX_INBOUND_STREAMS),
            max_outbound_streams: init
                .max_outbound_streams
                .unwrap_or(DEFAULT_MAX_OUTBOUND_STREAMS),
            message_receive_timeout: init
                .message_receive_timeout
                .unwrap_or(Duration::from_millis(DEFAULT_MESSAGE_RECEIVE_TIMEOUT)),
            run_on_limited_connections: init
                .run_on_limited_connections
                .unwrap_or(DEFAULT_RUN_ON_TRANSIENT_CONNECTIONS),
            max_incoming_message_size: init
                .max_incoming_message_size
                .unwrap_or(DEFAULT_MAX_INCOMING_MESSAGE_SIZE),
            max_outgoing_message_size: init
                .max_outgoing_message_size
                .unwrap_or(DEFAULT_MAX_OUTGOING_MESSAGE_SIZE),
            send_queue: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            outbound_tx,
        }
    }

    /// Set (or replace) the outbound sender
    pub async fn set_outbound_sender(&self, sender: mpsc::UnboundedSender<OutboundMessage>) {
        let mut guard = self.outbound_tx.write().await;
        *guard = Some(sender);
    }

    /// Start the network
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }

        info!(
            "Starting Bitswap network with protocols: {:?}",
            self.protocols
        );
        self.running = true;

        Ok(())
    }

    /// Stop the network
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        info!("Stopping Bitswap network");
        self.running = false;

        // Clear send queue
        self.send_queue.write().await.clear();

        Ok(())
    }

    /// Check if network is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Handle incoming stream with bitswap message
    pub async fn handle_incoming_stream(&self, peer: PeerId, data: Bytes) -> Result<()> {
        if !self.running {
            return Err(HeliaError::network("Network is not running"));
        }

        trace!("Handling incoming stream from {}", peer);

        // Decode message
        let message = PbBitswapMessage::decode(&mut Cursor::new(&data))
            .map_err(|e| HeliaError::network(format!("Failed to decode message: {}", e)))?;

        debug!(
            "Received message from {} with {} structured blocks, {} raw blocks, {} wantlist entries",
            peer,
            message.blocks.len(),
            message.raw_blocks.len(),
            message
                .wantlist
                .as_ref()
                .map(|w| w.entries.len())
                .unwrap_or(0)
        );

        // Send event
        let _ = self
            .event_tx
            .send(NetworkEvent::BitswapMessage(BitswapMessageEvent {
                peer,
                message,
            }));

        Ok(())
    }

    /// Send a message to a peer
    pub async fn send_message(&self, peer: PeerId, message: QueuedBitswapMessage) -> Result<()> {
        if !self.running {
            debug!("Network facade not marked running; queuing message anyway");
        }

        // Check if there's already a queued message for this peer
        let mut queue = self.send_queue.write().await;
        let peer_queue = queue.entry(peer).or_insert_with(VecDeque::new);

        // Merge with existing message if present
        if let Some(existing) = peer_queue.pop_back() {
            let merged = merge_messages(existing, message);
            peer_queue.push_back(merged);
        } else {
            peer_queue.push_back(message);
        }

        // Get the message to send
        let message_to_send = peer_queue
            .pop_front()
            .ok_or_else(|| HeliaError::network("No message to send"))?;

        drop(queue);

        debug!("Sending message to {}", peer);

        // Split message if too large
        let messages = split_message(message_to_send, self.max_outgoing_message_size);

        let sender = {
            let guard = self.outbound_tx.read().await;
            guard.clone()
        };

        let outbound =
            sender.ok_or_else(|| HeliaError::network("Outbound message channel not connected"))?;

        for msg in messages {
            outbound
                .send(OutboundMessage { peer, message: msg })
                .map_err(|e| {
                    HeliaError::network(format!("Failed to queue outbound message: {}", e))
                })?;
        }

        Ok(())
    }

    /// Find providers for a CID
    pub async fn find_providers(&self, _cid: &Cid) -> Result<Vec<PeerId>> {
        // This will be implemented when integrated with libp2p routing
        debug!("Finding providers (not yet implemented)");
        Ok(Vec::new())
    }

    /// Find and connect to providers
    pub async fn find_and_connect(&self, cid: &Cid) -> Result<()> {
        debug!("Finding and connecting to providers for {}", cid);

        // This will be implemented when integrated with libp2p
        // For now, just log
        Ok(())
    }

    /// Connect to a peer
    pub async fn connect_to(&self, peer: PeerId) -> Result<()> {
        if !self.running {
            return Err(HeliaError::network("Network is not running"));
        }

        debug!("Connecting to peer {}", peer);

        // This will be implemented when integrated with libp2p
        Ok(())
    }

    /// Get next event
    pub async fn next_event(&self) -> Option<NetworkEvent> {
        self.event_rx.write().await.recv().await
    }

    /// Dispatch an event
    pub fn dispatch_event(&self, event: NetworkEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Get supported protocols
    pub fn protocols(&self) -> &[String] {
        &self.protocols
    }

    /// Get max outgoing message size
    pub fn max_outgoing_message_size(&self) -> usize {
        self.max_outgoing_message_size
    }

    /// Get max incoming message size
    pub fn max_incoming_message_size(&self) -> usize {
        self.max_incoming_message_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_start_stop() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let sender_slot = Arc::new(RwLock::new(Some(tx)));
        let mut network = Network::new(NetworkInit::default(), sender_slot);
        assert!(!network.is_running());

        network.start().await.unwrap();
        assert!(network.is_running());

        network.stop().await.unwrap();
        assert!(!network.is_running());
    }

    #[tokio::test]
    async fn test_network_protocols() {
        let sender_slot = Arc::new(RwLock::new(None));
        let network = Network::new(NetworkInit::default(), sender_slot);
        let protocols = network.protocols();

        assert!(protocols.contains(&BITSWAP_120.to_string()));
        assert!(protocols.contains(&BITSWAP_110.to_string()));
        assert!(protocols.contains(&BITSWAP_100.to_string()));
    }

    #[tokio::test]
    async fn test_send_message() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let sender_slot = Arc::new(RwLock::new(Some(tx)));
        let mut network = Network::new(NetworkInit::default(), sender_slot);
        network.start().await.unwrap();

        let peer = PeerId::random();
        let message = QueuedBitswapMessage::new();

        // Should be able to send message
        let result = network.send_message(peer, message).await;
        assert!(result.is_ok());

        // Sending an empty message still queues an outbound packet (default empty)
        assert!(rx.try_recv().is_ok());
    }
}
