//! Protocol Buffer definitions for Bitswap messages
//! Based on the Bitswap 1.2.0 specification

use prost::Message as ProstMessage;
use std::io::Cursor;

/// Want type for blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum WantType {
    /// Want the full block
    WantBlock = 0,
    /// Only want to know if the peer has the block
    WantHave = 1,
}

impl From<i32> for WantType {
    fn from(value: i32) -> Self {
        match value {
            0 => WantType::WantBlock,
            1 => WantType::WantHave,
            _ => WantType::WantBlock, // Default
        }
    }
}

/// Block presence type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BlockPresenceType {
    /// Peer has the block
    HaveBlock = 0,
    /// Peer does not have the block
    DoNotHaveBlock = 1,
}

impl From<i32> for BlockPresenceType {
    fn from(value: i32) -> Self {
        match value {
            0 => BlockPresenceType::HaveBlock,
            1 => BlockPresenceType::DoNotHaveBlock,
            _ => BlockPresenceType::DoNotHaveBlock,
        }
    }
}

/// Wantlist entry in a Bitswap message
#[derive(Clone, PartialEq, ProstMessage)]
pub struct WantlistEntry {
    /// Block CID as bytes
    #[prost(bytes, tag = "1")]
    pub cid: Vec<u8>,
    /// Priority of the request (higher = more important)
    #[prost(int32, tag = "2")]
    pub priority: i32,
    /// Whether to cancel this want
    #[prost(bool, tag = "3")]
    pub cancel: bool,
    /// Type of want (Block or Have)
    #[prost(enumeration = "i32", tag = "4")]
    pub want_type: i32,
    /// Whether to send DONT_HAVE messages
    #[prost(bool, tag = "5")]
    pub send_dont_have: bool,
}

/// Wantlist in a Bitswap message
#[derive(Clone, PartialEq, ProstMessage)]
pub struct Wantlist {
    /// List of wanted blocks
    #[prost(message, repeated, tag = "1")]
    pub entries: Vec<WantlistEntry>,
    /// Whether this is a full wantlist or an update
    #[prost(bool, tag = "2")]
    pub full: bool,
}

/// Block data in a Bitswap message
#[derive(Clone, PartialEq, ProstMessage)]
pub struct Block {
    /// CID prefix (version, codec, hash algorithm, hash length)
    #[prost(bytes, tag = "1")]
    pub prefix: Vec<u8>,
    /// Block data
    #[prost(bytes, tag = "2")]
    pub data: Vec<u8>,
}

/// Block presence information
#[derive(Clone, PartialEq, ProstMessage)]
pub struct BlockPresence {
    /// Block CID as bytes
    #[prost(bytes, tag = "1")]
    pub cid: Vec<u8>,
    /// Presence type (Have or DontHave)
    #[prost(enumeration = "i32", tag = "2")]
    pub r#type: i32,
}

/// Main Bitswap protocol message
#[derive(Clone, PartialEq, ProstMessage)]
pub struct BitswapMessage {
    /// Wantlist (optional)
    #[prost(message, optional, tag = "1")]
    pub wantlist: Option<Wantlist>,
    /// Raw block data (legacy field for compatibility)
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub raw_blocks: Vec<Vec<u8>>,
    /// Block presence information (HAVE / DONT_HAVE)
    #[prost(message, repeated, tag = "3")]
    pub block_presences: Vec<BlockPresence>,
    /// Number of bytes pending to be sent
    #[prost(int32, tag = "4", default = "0")]
    pub pending_bytes: i32,
    /// Structured block payload (Bitswap 1.2+)
    #[prost(message, repeated, tag = "5")]
    pub blocks: Vec<Block>,
}

impl BitswapMessage {
    /// Encode the message to bytes
    pub fn encode_to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode(&mut buf).expect("Failed to encode message");
        buf
    }

    /// Decode a message from bytes
    pub fn decode_from_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        Self::decode(&mut Cursor::new(bytes))
    }

    /// Check if message is empty
    pub fn is_empty(&self) -> bool {
        self.wantlist
            .as_ref()
            .map_or(true, |w| w.entries.is_empty())
            && self.blocks.is_empty()
            && self.raw_blocks.is_empty()
            && self.block_presences.is_empty()
    }

    /// Get estimated size of the message
    pub fn estimated_size(&self) -> usize {
        self.encoded_len()
    }
}

// Default is automatically derived by prost

impl WantlistEntry {
    /// Create a new wantlist entry
    pub fn new(cid: Vec<u8>, priority: i32, want_type: WantType, cancel: bool) -> Self {
        Self {
            cid,
            priority,
            cancel,
            want_type: want_type as i32,
            send_dont_have: false,
        }
    }

    /// Create a block request
    pub fn new_block_request(cid: Vec<u8>, priority: i32) -> Self {
        Self::new(cid, priority, WantType::WantBlock, false)
    }

    /// Create a have request
    pub fn new_have_request(cid: Vec<u8>, priority: i32) -> Self {
        Self::new(cid, priority, WantType::WantHave, false)
    }

    /// Create a cancellation
    pub fn new_cancel(cid: Vec<u8>) -> Self {
        Self::new(cid, 0, WantType::WantBlock, true)
    }

    /// Get the want type
    pub fn get_want_type(&self) -> WantType {
        WantType::from(self.want_type)
    }
}

impl Block {
    /// Create a new block
    pub fn new(prefix: Vec<u8>, data: Vec<u8>) -> Self {
        Self { prefix, data }
    }
}

impl BlockPresence {
    /// Create a new block presence
    pub fn new(cid: Vec<u8>, presence_type: BlockPresenceType) -> Self {
        Self {
            cid,
            r#type: presence_type as i32,
        }
    }

    /// Get the presence type
    pub fn get_presence_type(&self) -> BlockPresenceType {
        BlockPresenceType::from(self.r#type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_message() {
        let msg = BitswapMessage {
            wantlist: Some(Wantlist {
                entries: vec![WantlistEntry::new_block_request(vec![1, 2, 3], 10)],
                full: false,
            }),
            raw_blocks: vec![],
            blocks: vec![],
            block_presences: vec![],
            pending_bytes: 0,
        };

        let encoded = msg.encode_to_vec();
        let decoded = BitswapMessage::decode_from_bytes(&encoded).unwrap();

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_want_type_conversion() {
        assert_eq!(WantType::from(0), WantType::WantBlock);
        assert_eq!(WantType::from(1), WantType::WantHave);
    }

    #[test]
    fn test_block_presence_type_conversion() {
        assert_eq!(BlockPresenceType::from(0), BlockPresenceType::HaveBlock);
        assert_eq!(
            BlockPresenceType::from(1),
            BlockPresenceType::DoNotHaveBlock
        );
    }
}
