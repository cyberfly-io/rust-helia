//! Bitswap NetworkBehaviour implementation
//! 
//! Integrates Bitswap protocol with libp2p swarm for P2P block exchange.

use crate::{
    coordinator::Bitswap,
    pb::BitswapMessage as PbBitswapMessage,
};
use cid::Cid;
use futures::prelude::*;
use libp2p::{
    request_response::{
        self, InboundFailure, OutboundFailure, OutboundRequestId, ProtocolSupport,
        ResponseChannel,
    },
    swarm::{
        ConnectionId, NetworkBehaviour, ToSwarm,
    },
    PeerId, StreamProtocol,
};
use prost::Message;
use std::{
    collections::HashMap,
    io,
    sync::Arc,
    time::Duration,
};
use tracing::{debug, info, warn};

/// Bitswap protocol name (version 1.2.0)
const BITSWAP_PROTOCOL: &str = "/ipfs/bitswap/1.2.0";

/// Codec for Bitswap protocol messages
#[derive(Debug, Clone)]
pub struct BitswapCodec;

#[async_trait::async_trait]
impl request_response::Codec for BitswapCodec {
    type Protocol = StreamProtocol;
    type Request = PbBitswapMessage;
    type Response = PbBitswapMessage;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        
        PbBitswapMessage::decode(&buf[..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        
        PbBitswapMessage::decode(&buf[..])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = Vec::new();
        req.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let mut buf = Vec::new();
        res.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        io.close().await
    }
}

/// Events emitted by the Bitswap behaviour
#[derive(Debug)]
pub enum BitswapEvent {
    /// Message received from a peer
    MessageReceived {
        peer: PeerId,
        message: PbBitswapMessage,
    },
    /// Message sent to a peer
    MessageSent {
        peer: PeerId,
    },
    /// Failed to send message to peer
    SendError {
        peer: PeerId,
        error: String,
    },
}

/// Bitswap NetworkBehaviour implementation
pub struct BitswapBehaviour {
    /// Request-response behaviour for Bitswap protocol
    inner: request_response::Behaviour<BitswapCodec>,
    /// Reference to Bitswap coordinator
    coordinator: Option<Arc<Bitswap>>,
    /// Pending outbound requests
    pending_requests: HashMap<OutboundRequestId, PeerId>,
}

impl BitswapBehaviour {
    /// Create a new BitswapBehaviour
    pub fn new() -> Self {
        let protocols = std::iter::once((
            StreamProtocol::new(BITSWAP_PROTOCOL),
            ProtocolSupport::Full,
        ));
        
        let config = request_response::Config::default()
            .with_request_timeout(Duration::from_secs(30));
        
        let inner = request_response::Behaviour::with_codec(
            BitswapCodec,
            protocols,
            config,
        );

        Self {
            inner,
            coordinator: None,
            pending_requests: HashMap::new(),
        }
    }

    /// Set the coordinator for this behaviour
    pub fn set_coordinator(&mut self, coordinator: Arc<Bitswap>) {
        self.coordinator = Some(coordinator);
    }

    /// Send a Bitswap message to a peer
    pub fn send_message(&mut self, peer: PeerId, message: PbBitswapMessage) -> OutboundRequestId {
        debug!("Sending Bitswap message to peer {}", peer);
        let request_id = self.inner.send_request(&peer, message);
        self.pending_requests.insert(request_id, peer);
        request_id
    }

    /// Send a response to an inbound request
    pub fn send_response(
        &mut self,
        channel: ResponseChannel<PbBitswapMessage>,
        response: PbBitswapMessage,
    ) -> std::result::Result<(), PbBitswapMessage> {
        self.inner.send_response(channel, response)
    }
}

impl Default for BitswapBehaviour {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkBehaviour for BitswapBehaviour {
    type ConnectionHandler = <request_response::Behaviour<BitswapCodec> as NetworkBehaviour>::ConnectionHandler;
    type ToSwarm = BitswapEvent;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &libp2p::Multiaddr,
        remote_addr: &libp2p::Multiaddr,
    ) -> std::result::Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        debug!("Bitswap: Inbound connection from peer {}", peer);
        self.inner.handle_established_inbound_connection(
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
    ) -> std::result::Result<libp2p::swarm::THandler<Self>, libp2p::swarm::ConnectionDenied> {
        debug!("Bitswap: Outbound connection to peer {}", peer);
        self.inner.handle_established_outbound_connection(
            connection_id,
            peer,
            addr,
            role_override,
        )
    }

    fn on_swarm_event(&mut self, event: libp2p::swarm::FromSwarm) {
        self.inner.on_swarm_event(event);
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: libp2p::swarm::THandlerOutEvent<Self>,
    ) {
        self.inner.on_connection_handler_event(peer_id, connection_id, event);
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<ToSwarm<Self::ToSwarm, libp2p::swarm::THandlerInEvent<Self>>> {
        // Poll the inner request-response behaviour
        loop {
            match self.inner.poll(cx) {
                std::task::Poll::Ready(event) => {
                    match event {
                        ToSwarm::GenerateEvent(inner_event) => {
                            match inner_event {
                                request_response::Event::Message { peer, message } => {
                                    match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            debug!("Bitswap: Received request from peer {}", peer);
                                            
                                            // Process request and build response
                                            let mut response_blocks = Vec::new();
                                            let mut response_presences = Vec::new();
                                            const MAX_SIZE_REPLACE_HAS_WITH_BLOCK: usize = 1024; // JS Helia default
                                            
                                            if let Some(coordinator) = &self.coordinator {
                                                if let Some(wantlist) = &request.wantlist {
                                                    debug!("Bitswap: Processing wantlist with {} entries from peer {}", 
                                                          wantlist.entries.len(), peer);
                                                    
                                                    // Try to serve blocks from blockstore
                                                    // We use block_in_place to allow async calls in this sync context
                                                    let blockstore = coordinator.blockstore.clone();
                                                    
                                                    for entry in &wantlist.entries {
                                                        // Skip cancel entries
                                                        if entry.cancel {
                                                            debug!("Bitswap: Peer {} cancelled want", peer);
                                                            continue;
                                                        }
                                                        
                                                        // Try to parse CID from bytes
                                                        match Cid::try_from(&entry.cid[..]) {
                                                            Ok(cid) => {
                                                                let want_type = entry.want_type;
                                                                let is_want_have = want_type == (crate::pb::WantType::WantHave as i32);
                                                                
                                                                debug!("Bitswap: Peer {} wants {} {} (priority: {})", 
                                                                      peer, 
                                                                      if is_want_have { "HAVE for" } else { "BLOCK" },
                                                                      cid, 
                                                                      entry.priority);
                                                                
                                                                // Try to get block from blockstore
                                                                let block_result = tokio::task::block_in_place(|| {
                                                                    tokio::runtime::Handle::current().block_on(async {
                                                                        blockstore.get(&cid, None).await
                                                                    })
                                                                });
                                                                
                                                                match block_result {
                                                                    Ok(data) => {
                                                                        let block_size = data.len();
                                                                        
                                                                        // **OPTIMIZATION**: If peer wants HAVE but block is small,
                                                                        // send the block directly (saves a round trip)
                                                                        if is_want_have && block_size > MAX_SIZE_REPLACE_HAS_WITH_BLOCK {
                                                                            // Block is large, just send HAVE presence
                                                                            debug!("Bitswap: Sending HAVE for {} to peer {} ({} bytes > {} threshold)", 
                                                                                 cid, peer, block_size, MAX_SIZE_REPLACE_HAS_WITH_BLOCK);
                                                                            
                                                                            response_presences.push(crate::pb::BlockPresence {
                                                                                cid: entry.cid.clone(),
                                                                                r#type: crate::pb::BlockPresenceType::HaveBlock as i32,
                                                                            });
                                                                        } else {
                                                                            // Send the block (either explicitly requested or small enough)
                                                                            if is_want_have {
                                                                                info!("Bitswap: Sending block {} instead of HAVE to peer {} ({} bytes ≤ {} threshold)", 
                                                                                     cid, peer, block_size, MAX_SIZE_REPLACE_HAS_WITH_BLOCK);
                                                                            } else {
                                                                                info!("Bitswap: Serving block {} to peer {} ({} bytes)", 
                                                                                     cid, peer, block_size);
                                                                            }
                                                                            
                                                                            let cid_bytes = cid.to_bytes();
                                                                            response_blocks.push(crate::pb::Block {
                                                                                prefix: cid_bytes,
                                                                                data: data.to_vec(),
                                                                            });
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        debug!("Bitswap: Block {} not found in blockstore: {}", cid, e);
                                                                        
                                                                        // Send DONT_HAVE if requested
                                                                        if entry.send_dont_have {
                                                                            response_presences.push(crate::pb::BlockPresence {
                                                                                cid: entry.cid.clone(),
                                                                                r#type: crate::pb::BlockPresenceType::DoNotHaveBlock as i32,
                                                                            });
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                warn!("Bitswap: Failed to parse CID from wantlist entry: {}", e);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Build and send response
                                            let response = PbBitswapMessage {
                                                wantlist: None,
                                                blocks: response_blocks.clone(),
                                                block_presences: response_presences.clone(),
                                                pending_bytes: 0,
                                            };
                                            
                                            if !response_blocks.is_empty() || !response_presences.is_empty() {
                                                info!("Bitswap: Sending response to peer {} with {} blocks and {} presences", 
                                                      peer, response_blocks.len(), response_presences.len());
                                            }
                                            
                                            let _ = self.send_response(channel, response);
                                            
                                            // Emit event for received message
                                            let bitswap_event = BitswapEvent::MessageReceived {
                                                peer,
                                                message: request,
                                            };

                                            return std::task::Poll::Ready(ToSwarm::GenerateEvent(bitswap_event));
                                        }
                                        request_response::Message::Response { request_id, response } => {
                                            debug!("Bitswap: Received response for request {:?}", request_id);
                                            
                                            if let Some(peer) = self.pending_requests.remove(&request_id) {
                                                let bitswap_event = BitswapEvent::MessageReceived {
                                                    peer,
                                                    message: response,
                                                };
                                                return std::task::Poll::Ready(ToSwarm::GenerateEvent(bitswap_event));
                                            }
                                        }
                                    }
                                }
                                request_response::Event::OutboundFailure { peer, request_id, error } => {
                                    warn!("Bitswap: Outbound failure to peer {}: {:?}", peer, error);
                                    self.pending_requests.remove(&request_id);
                                    
                                    let bitswap_event = BitswapEvent::SendError {
                                        peer,
                                        error: format!("{:?}", error),
                                    };
                                    return std::task::Poll::Ready(ToSwarm::GenerateEvent(bitswap_event));
                                }
                                request_response::Event::InboundFailure { peer, error, .. } => {
                                    warn!("Bitswap: Inbound failure from peer {}: {:?}", peer, error);
                                }
                                request_response::Event::ResponseSent { peer, .. } => {
                                    debug!("Bitswap: Response sent to peer {}", peer);
                                    
                                    let bitswap_event = BitswapEvent::MessageSent { peer };
                                    return std::task::Poll::Ready(ToSwarm::GenerateEvent(bitswap_event));
                                }
                            }
                        }
                        other => {
                            // Forward other ToSwarm events
                            let mapped = match other {
                                ToSwarm::Dial { opts } => ToSwarm::Dial { opts },
                                ToSwarm::ListenOn { opts } => ToSwarm::ListenOn { opts },
                                ToSwarm::RemoveListener { id } => ToSwarm::RemoveListener { id },
                                ToSwarm::NotifyHandler { peer_id, handler, event } => {
                                    ToSwarm::NotifyHandler { peer_id, handler, event }
                                }
                                ToSwarm::NewExternalAddrCandidate(addr) => ToSwarm::NewExternalAddrCandidate(addr),
                                ToSwarm::ExternalAddrConfirmed(addr) => ToSwarm::ExternalAddrConfirmed(addr),
                                ToSwarm::ExternalAddrExpired(addr) => ToSwarm::ExternalAddrExpired(addr),
                                ToSwarm::CloseConnection { peer_id, connection } => {
                                    ToSwarm::CloseConnection { peer_id, connection }
                                }
                                ToSwarm::NewExternalAddrOfPeer { peer_id, address } => {
                                    ToSwarm::NewExternalAddrOfPeer { peer_id, address }
                                }
                                _ => continue, // Skip GenerateEvent as we handled it above
                            };
                            return std::task::Poll::Ready(mapped);
                        }
                    }
                }
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitswap_behaviour_creation() {
        let behaviour = BitswapBehaviour::new();
        assert!(behaviour.coordinator.is_none());
        assert!(behaviour.pending_requests.is_empty());
    }

    #[test]
    fn test_codec_creation() {
        let codec = BitswapCodec;
        // Just check it can be created
        let _ = format!("{:?}", codec);
    }
}
