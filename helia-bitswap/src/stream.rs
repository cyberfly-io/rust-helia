//! Utilities for length-prefixed Bitswap framing compatible with Helia JS.

use crate::pb::BitswapMessage;
use bytes::{Bytes, BytesMut};
use prost::Message;
use thiserror::Error;
use tokio_util::codec::Encoder;
use unsigned_varint::codec::UviBytes;

/// Errors that can occur while encoding or decoding framed Bitswap messages.
#[derive(Debug, Error)]
pub enum FrameError {
    /// Underlying protobuf encoding failure.
    #[error("failed to encode bitswap message: {0}")]
    Encode(#[from] prost::EncodeError),
    /// Underlying protobuf decoding failure.
    #[error("failed to decode bitswap message: {0}")]
    Decode(#[from] prost::DecodeError),
}

/// Encode a [`BitswapMessage`] into a length-prefixed frame.
pub fn encode_frame(message: &BitswapMessage) -> Result<Vec<u8>, FrameError> {
    let mut payload = Vec::with_capacity(message.encoded_len());
    message.encode(&mut payload)?;

    let mut encoder = UviBytes::default();
    let mut framed = BytesMut::with_capacity(payload.len() + 8); // varint len is small
    encoder
        .encode(Bytes::from(payload), &mut framed)
        .expect("encoding into BytesMut never fails");
    Ok(framed.freeze().to_vec())
}

/// Decode a [`BitswapMessage`] from a frame that already had its length-prefix stripped.
///
/// The caller is responsible for ensuring `payload` contains an entire protobuf message.
pub fn decode_payload(payload: &[u8]) -> Result<BitswapMessage, FrameError> {
    Ok(BitswapMessage::decode(payload)?)
}
