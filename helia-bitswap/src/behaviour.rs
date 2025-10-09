//! Bitswap NetworkBehaviour implementation built on streaming substreams.
//!
//! This replaces the request/response based approach with a persistent
//! streaming protocol that mirrors the behaviour of Helia JS, allowing both
//! sides to push Bitswap messages over long-lived connections.

use crate::{
    coordinator::Bitswap,
    pb,
    pb::BitswapMessage as PbBitswapMessage,
    stream::{decode_payload, encode_frame},
};
use cid::Cid;
use futures::{io::AsyncReadExt as FuturesAsyncReadExt, StreamExt};
use libp2p::{
    swarm::{
        ConnectionDenied, ConnectionId, FromSwarm, NetworkBehaviour, THandler, THandlerInEvent,
        THandlerOutEvent, ToSwarm,
    },
    PeerId, Stream, StreamProtocol,
};
use libp2p_stream::{Behaviour as StreamBehaviour, Control, IncomingStreams, OpenStreamError};
use std::{
    collections::HashMap,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex};
use tokio_util::{
    codec::FramedRead,
    compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt},
};
use tracing::{debug, info, trace, warn};
use unsigned_varint::codec::UviBytes;

/// Bitswap protocol name (version 1.2.0)
const BITSWAP_PROTOCOL: &str = "/ipfs/bitswap/1.2.0";

/// Threshold (bytes) up to which we replace HAVE messages with full blocks.
const MAX_SIZE_REPLACE_HAS_WITH_BLOCK: usize = 1024;

/// Events emitted by the Bitswap behaviour.
#[derive(Debug)]
pub enum BitswapEvent {
    /// Message received from a peer.
    MessageReceived {
        peer: PeerId,
        message: PbBitswapMessage,
    },
    /// Message successfully sent to a peer.
    MessageSent { peer: PeerId },
    /// Failed to send message to peer.
    SendError { peer: PeerId, error: String },
}

/// Command directed at the outbound processing task.
#[derive(Debug)]
enum OutboundCommand {
    Send {
        peer: PeerId,
        message: PbBitswapMessage,
    },
}

/// Handle for an active streaming connection.
struct ConnectionHandle {
    sender: mpsc::UnboundedSender<PbBitswapMessage>,
}

/// Shared state accessible from background tasks.
struct SharedState {
    protocol: StreamProtocol,
    control: Arc<Mutex<Control>>,
    connections: Mutex<HashMap<PeerId, ConnectionHandle>>,
    event_tx: mpsc::UnboundedSender<BitswapEvent>,
    coordinator: Arc<Bitswap>,
}

/// Streaming Bitswap NetworkBehaviour implementation.
pub struct BitswapBehaviour {
    protocol: StreamProtocol,
    stream_behaviour: StreamBehaviour,
    control: Arc<Mutex<Control>>,
    coordinator: Option<Arc<Bitswap>>,
    shared_state: Option<Arc<SharedState>>,
    incoming_streams: Option<IncomingStreams>,
    outbound_rx: Option<mpsc::UnboundedReceiver<OutboundCommand>>,
    outbound_tx: mpsc::UnboundedSender<OutboundCommand>,
    event_tx: mpsc::UnboundedSender<BitswapEvent>,
    event_rx: mpsc::UnboundedReceiver<BitswapEvent>,
    tasks_started: bool,
}

impl BitswapBehaviour {
    /// Create a new Bitswap behaviour backed by streaming substreams.
    pub fn new() -> Self {
        let protocol = StreamProtocol::new(BITSWAP_PROTOCOL);
    let mut stream_behaviour = StreamBehaviour::new();
    let mut control = stream_behaviour.new_control();
        let incoming_streams = control
            .accept(protocol.clone())
            .expect("bitswap protocol should only be registered once");

        let control = Arc::new(Mutex::new(control));
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel();

        Self {
            protocol,
            stream_behaviour,
            control,
            coordinator: None,
            shared_state: None,
            incoming_streams: Some(incoming_streams),
            outbound_rx: Some(outbound_rx),
            outbound_tx,
            event_tx,
            event_rx,
            tasks_started: false,
        }
    }

    /// Set the coordinator for this behaviour (starts background tasks on first call).
    pub fn set_coordinator(&mut self, coordinator: Arc<Bitswap>) {
        if self.tasks_started {
            return;
        }

        let shared_state = Arc::new(SharedState {
            protocol: self.protocol.clone(),
            control: self.control.clone(),
            connections: Mutex::new(HashMap::new()),
            event_tx: self.event_tx.clone(),
            coordinator: coordinator.clone(),
        });

        self.coordinator = Some(coordinator);
        self.shared_state = Some(shared_state.clone());
        self.start_background_tasks(shared_state);
    }

    /// Send a Bitswap message to a peer via the streaming connection.
    pub fn send_message(&mut self, peer: PeerId, message: PbBitswapMessage) {
        if self
            .outbound_tx
            .send(OutboundCommand::Send { peer, message })
            .is_err()
        {
            warn!("Bitswap outbound worker channel dropped; message lost");
        }
    }

    fn start_background_tasks(&mut self, shared_state: Arc<SharedState>) {
        let Some(mut incoming_streams) = self.incoming_streams.take() else {
            warn!("No incoming stream listener available for Bitswap");
            return;
        };

        let Some(mut outbound_rx) = self.outbound_rx.take() else {
            warn!("No outbound receiver available for Bitswap");
            return;
        };

        self.tasks_started = true;

        // Accept inbound streams.
        let inbound_state = shared_state.clone();
        tokio::spawn(async move {
            trace!("Bitswap inbound accept loop started");
            while let Some((peer, stream)) = incoming_streams.next().await {
                trace!(peer = %peer, "Bitswap inbound stream established");
                if let Err(err) = register_connection(peer, stream, inbound_state.clone()).await {
                    warn!(peer = %peer, error = %err, "Failed to register inbound Bitswap stream");
                }
            }
            trace!("Bitswap inbound accept loop terminated");
        });

        // Process outbound commands.
        let outbound_state = shared_state;
        tokio::spawn(async move {
            trace!("Bitswap outbound worker started");
            while let Some(cmd) = outbound_rx.recv().await {
                match cmd {
                    OutboundCommand::Send { peer, message } => {
                        if let Err(err) =
                            send_via_stream(peer, message, outbound_state.clone()).await
                        {
                            warn!(peer = %peer, error = %err, "Failed to send Bitswap message");
                            let _ = outbound_state
                                .event_tx
                                .send(BitswapEvent::SendError { peer, error: err });
                        }
                    }
                }
            }
            trace!("Bitswap outbound worker terminated");
        });
    }
}

impl Default for BitswapBehaviour {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkBehaviour for BitswapBehaviour {
    type ConnectionHandler = <StreamBehaviour as NetworkBehaviour>::ConnectionHandler;
    type ToSwarm = BitswapEvent;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.stream_behaviour.handle_established_inbound_connection(
            connection_id,
            peer,
            local_addr,
            remote_addr,
        )
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        addr: &libp2p::Multiaddr,
        role_override: libp2p::core::Endpoint,
    port_use: libp2p::core::transport::PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.stream_behaviour
            .handle_established_outbound_connection(connection_id, peer, addr, role_override, port_use)
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        self.stream_behaviour.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        self.stream_behaviour
            .on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        if let Poll::Ready(Some(event)) = self.event_rx.poll_recv(cx) {
            return Poll::Ready(ToSwarm::GenerateEvent(event));
        }

        loop {
            match self.stream_behaviour.poll(cx) {
                Poll::Ready(ToSwarm::GenerateEvent(())) => continue,
                Poll::Ready(event) => return Poll::Ready(event.map_out(|_| unreachable!())),
                Poll::Pending => break,
            }
        }

        Poll::Pending
    }
}

async fn send_via_stream(
    peer: PeerId,
    message: PbBitswapMessage,
    state: Arc<SharedState>,
) -> Result<(), String> {
    let sender = ensure_connection(peer, state.clone()).await?;
    sender
        .send(message)
        .map_err(|e| format!("failed to queue message: {}", e))
}

async fn ensure_connection(
    peer: PeerId,
    state: Arc<SharedState>,
) -> Result<mpsc::UnboundedSender<PbBitswapMessage>, String> {
    if let Some(handle) = state.connections.lock().await.get(&peer) {
        return Ok(handle.sender.clone());
    }

    let protocol = state.protocol.clone();
    let open_result = {
        let mut control = state.control.lock().await;
        control.open_stream(peer, protocol).await
    };

    match open_result {
        Ok(stream) => register_connection(peer, stream, state.clone()).await,
        Err(OpenStreamError::UnsupportedProtocol(protocol)) => {
            Err(format!("peer does not support protocol {}", protocol))
        }
        Err(OpenStreamError::Io(e)) => Err(e.to_string()),
        Err(err) => Err(err.to_string()),
    }
}

async fn register_connection(
    peer: PeerId,
    stream: Stream,
    state: Arc<SharedState>,
) -> Result<mpsc::UnboundedSender<PbBitswapMessage>, String> {
    trace!(peer = %peer, "Registering Bitswap stream");

    let (reader, writer) = FuturesAsyncReadExt::split(stream);
    let (tx, mut rx) = mpsc::unbounded_channel();
    let writer_tx = tx.clone();

    state
        .connections
        .lock()
        .await
        .insert(peer, ConnectionHandle { sender: tx.clone() });

    // Writer task
    let write_state = state.clone();
    tokio::spawn(async move {
        let mut writer = writer.compat_write();
        while let Some(message) = rx.recv().await {
            match encode_frame(&message) {
                Ok(frame) => {
                    if let Err(err) = writer.write_all(&frame).await {
                        warn!(peer = %peer, error = %err, "Failed to write Bitswap message");
                        let _ = write_state.event_tx.send(BitswapEvent::SendError {
                            peer,
                            error: err.to_string(),
                        });
                        break;
                    }

                    if let Err(err) = writer.flush().await {
                        warn!(peer = %peer, error = %err, "Failed to flush Bitswap message");
                        let _ = write_state.event_tx.send(BitswapEvent::SendError {
                            peer,
                            error: err.to_string(),
                        });
                        break;
                    }

                    let _ = write_state
                        .event_tx
                        .send(BitswapEvent::MessageSent { peer });
                }
                Err(err) => {
                    warn!(peer = %peer, error = %err, "Failed to encode Bitswap message");
                }
            }
        }

        cleanup_connection(&write_state, peer).await;
    });

    // Reader task
    let read_state = state.clone();
    tokio::spawn(async move {
    let mut framed_read = FramedRead::new(reader.compat(), UviBytes::<Vec<u8>>::default());
        while let Some(frame) = framed_read.next().await {
            match frame {
                Ok(bytes) => match decode_payload(&bytes) {
                    Ok(message) => {
                        trace!(peer = %peer, "Bitswap message received");
                        let cloned = message.clone();
                        let _ = read_state.event_tx.send(BitswapEvent::MessageReceived {
                            peer,
                            message: cloned,
                        });

                        if let Some(response) =
                            prepare_response(&read_state.coordinator, &message).await
                        {
                            if writer_tx.send(response).is_err() {
                                warn!(peer = %peer, "Bitswap writer closed before response could be queued");
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        warn!(peer = %peer, error = %err, "Failed to decode inbound Bitswap payload");
                    }
                },
                Err(err) => {
                    debug!(peer = %peer, error = %err, "Error reading Bitswap frame");
                    break;
                }
            }
        }

        cleanup_connection(&read_state, peer).await;
    });

    Ok(tx)
}

async fn cleanup_connection(state: &Arc<SharedState>, peer: PeerId) {
    let mut connections = state.connections.lock().await;
    connections.remove(&peer);
    trace!(peer = %peer, "Bitswap stream closed");
}

async fn prepare_response(
    coordinator: &Arc<Bitswap>,
    message: &PbBitswapMessage,
) -> Option<PbBitswapMessage> {
    let wantlist = message.wantlist.as_ref()?;
    if wantlist.entries.is_empty() {
        return None;
    }

    let blockstore = coordinator.blockstore.clone();
    let mut response_blocks = Vec::new();
    let mut response_presences = Vec::new();

    for entry in &wantlist.entries {
        if entry.cancel {
            trace!("Peer cancelled want entry");
            continue;
        }

        let Ok(cid) = Cid::try_from(entry.cid.as_slice()) else {
            warn!("Failed to parse CID from wantlist entry");
            continue;
        };

        let is_want_have = entry.want_type == pb::WantType::WantHave as i32;

        match blockstore.get(&cid, None).await {
            Ok(data) => {
                let block_size = data.len();

                if is_want_have && block_size > MAX_SIZE_REPLACE_HAS_WITH_BLOCK {
                    response_presences.push(pb::BlockPresence {
                        cid: entry.cid.clone(),
                        r#type: pb::BlockPresenceType::HaveBlock as i32,
                    });
                } else {
                    if is_want_have {
                        info!(cid = %cid, "Serving HAVE request with block");
                    } else {
                        info!(cid = %cid, size = block_size, "Serving WANTBLOCK");
                    }

                    response_blocks.push(pb::Block {
                        prefix: cid.to_bytes(),
                        data: data.to_vec(),
                    });
                }
            }
            Err(err) => {
                debug!(cid = %cid, error = %err, "Block not found for CID");
                if entry.send_dont_have {
                    response_presences.push(pb::BlockPresence {
                        cid: entry.cid.clone(),
                        r#type: pb::BlockPresenceType::DoNotHaveBlock as i32,
                    });
                }
            }
        }
    }

    let raw_blocks = response_blocks
        .iter()
        .map(|block| block.data.clone())
        .collect();

    Some(PbBitswapMessage {
        wantlist: None,
        raw_blocks,
        block_presences: response_presences,
        pending_bytes: 0,
        blocks: response_blocks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitswap_behaviour_creation() {
        let behaviour = BitswapBehaviour::new();
        assert!(behaviour.coordinator.is_none());
        assert!(!behaviour.tasks_started);
    }
}
