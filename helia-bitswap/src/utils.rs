//! Utilities for Bitswap message handling
//! Based on @helia/bitswap utils

use crate::pb::{
    BitswapMessage, Block, BlockPresence, BlockPresenceType, WantType, Wantlist, WantlistEntry,
};
use cid::Cid;
use std::collections::HashMap;

/// A queued Bitswap message that can be built incrementally
#[derive(Debug, Clone)]
pub struct QueuedBitswapMessage {
    /// Whether this is a full wantlist
    pub full: bool,
    /// Wantlist entries by CID
    pub wantlist: HashMap<Vec<u8>, WantlistEntry>,
    /// Blocks by CID
    pub blocks: HashMap<Vec<u8>, Block>,
    /// Block presences by CID
    pub block_presences: HashMap<Vec<u8>, BlockPresence>,
    /// Pending bytes
    pub pending_bytes: i32,
}

impl QueuedBitswapMessage {
    /// Create a new empty queued message
    pub fn new() -> Self {
        Self {
            full: false,
            wantlist: HashMap::new(),
            blocks: HashMap::new(),
            block_presences: HashMap::new(),
            pending_bytes: 0,
        }
    }

    /// Create a new queued message with full wantlist flag
    pub fn new_full() -> Self {
        Self {
            full: true,
            wantlist: HashMap::new(),
            blocks: HashMap::new(),
            block_presences: HashMap::new(),
            pending_bytes: 0,
        }
    }

    /// Add a wantlist entry
    pub fn add_wantlist_entry(&mut self, cid: &Cid, entry: WantlistEntry) {
        self.wantlist.insert(cid.to_bytes(), entry);
    }

    /// Add a want for a block
    pub fn add_want_block(&mut self, cid: &Cid, priority: i32) {
        let entry = WantlistEntry::new_block_request(cid.to_bytes(), priority);
        self.wantlist.insert(cid.to_bytes(), entry);
    }

    /// Add a want to check if peer has block
    pub fn add_want_have(&mut self, cid: &Cid, priority: i32) {
        let entry = WantlistEntry::new_have_request(cid.to_bytes(), priority);
        self.wantlist.insert(cid.to_bytes(), entry);
    }

    /// Cancel a want
    pub fn add_cancel(&mut self, cid: &Cid) {
        let entry = WantlistEntry::new_cancel(cid.to_bytes());
        self.wantlist.insert(cid.to_bytes(), entry);
    }

    /// Add a block
    pub fn add_block(&mut self, cid: &Cid, prefix: Vec<u8>, data: Vec<u8>) {
        let block = Block::new(prefix, data);
        self.blocks.insert(cid.to_bytes(), block);
    }

    /// Add block presence
    pub fn add_block_presence(&mut self, cid: &Cid, presence_type: BlockPresenceType) {
        let presence = BlockPresence::new(cid.to_bytes(), presence_type);
        self.block_presences.insert(cid.to_bytes(), presence);
    }

    /// Check if message is empty
    pub fn is_empty(&self) -> bool {
        self.wantlist.is_empty() && self.blocks.is_empty() && self.block_presences.is_empty()
    }

    /// Convert to protocol message
    pub fn to_message(&self) -> BitswapMessage {
        let wantlist = if self.wantlist.is_empty() {
            None
        } else {
            Some(Wantlist {
                entries: self.wantlist.values().cloned().collect(),
                full: self.full,
            })
        };

        let blocks: Vec<Block> = self.blocks.values().cloned().collect();
        let raw_blocks = blocks.iter().map(|block| block.data.clone()).collect();

        BitswapMessage {
            wantlist,
            raw_blocks,
            block_presences: self.block_presences.values().cloned().collect(),
            pending_bytes: self.pending_bytes,
            blocks,
        }
    }

    /// Get estimated size
    pub fn estimated_size(&self) -> usize {
        self.to_message().estimated_size()
    }
}

impl Default for QueuedBitswapMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge two queued messages
pub fn merge_messages(
    mut base: QueuedBitswapMessage,
    other: QueuedBitswapMessage,
) -> QueuedBitswapMessage {
    // Merge wantlists
    for (cid, entry) in other.wantlist {
        base.wantlist.insert(cid, entry);
    }

    // Merge blocks
    for (cid, block) in other.blocks {
        base.blocks.insert(cid, block);
    }

    // Merge block presences
    for (cid, presence) in other.block_presences {
        base.block_presences.insert(cid, presence);
    }

    // Update full flag
    base.full = base.full || other.full;

    // Add pending bytes
    base.pending_bytes += other.pending_bytes;

    base
}

/// Split a message if it exceeds the maximum size
pub fn split_message(message: QueuedBitswapMessage, max_size: usize) -> Vec<BitswapMessage> {
    let estimated_size = message.estimated_size();

    if estimated_size <= max_size {
        return vec![message.to_message()];
    }

    let mut messages = Vec::new();
    let mut current = QueuedBitswapMessage::new();
    current.full = message.full;

    // Add blocks first (they're usually largest)
    for (cid_bytes, block) in message.blocks {
        let block_size = block.data.len() + block.prefix.len();

        if current.estimated_size() + block_size > max_size && !current.is_empty() {
            messages.push(current.to_message());
            current = QueuedBitswapMessage::new();
        }

        current.blocks.insert(cid_bytes, block);
    }

    // Add wantlist entries
    for (cid_bytes, entry) in message.wantlist {
        if current.estimated_size() + 100 > max_size && !current.is_empty() {
            messages.push(current.to_message());
            current = QueuedBitswapMessage::new();
        }

        current.wantlist.insert(cid_bytes, entry);
    }

    // Add block presences
    for (cid_bytes, presence) in message.block_presences {
        if current.estimated_size() + 50 > max_size && !current.is_empty() {
            messages.push(current.to_message());
            current = QueuedBitswapMessage::new();
        }

        current.block_presences.insert(cid_bytes, presence);
    }

    if !current.is_empty() {
        messages.push(current.to_message());
    }

    if messages.is_empty() {
        messages.push(BitswapMessage::default());
    }

    messages
}

/// Helper to create CID prefix from a CID
pub fn cid_to_prefix(cid: &Cid) -> Vec<u8> {
    let cid_bytes = cid.to_bytes();

    // Extract version, codec, and multihash info
    let mut prefix = Vec::new();

    // CID version
    prefix.push(match cid.version() {
        cid::Version::V0 => 0,
        cid::Version::V1 => 1,
    });

    // Codec
    let codec = cid.codec();
    let mut codec_buffer = unsigned_varint::encode::u64_buffer();
    let codec_bytes = unsigned_varint::encode::u64(codec, &mut codec_buffer);
    prefix.extend_from_slice(codec_bytes);

    // Hash algorithm and length from multihash
    if cid_bytes.len() > 2 {
        // Multihash starts after version and codec
        let hash_start = 1 + codec_bytes.len();
        if cid_bytes.len() > hash_start + 2 {
            prefix.push(cid_bytes[hash_start]); // hash algorithm
            prefix.push(cid_bytes[hash_start + 1]); // hash length
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queued_message_creation() {
        let mut msg = QueuedBitswapMessage::new();
        assert!(msg.is_empty());

        let cid = Cid::try_from("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG").unwrap();
        msg.add_want_block(&cid, 10);

        assert!(!msg.is_empty());
        assert_eq!(msg.wantlist.len(), 1);
    }

    #[test]
    fn test_merge_messages() {
        let cid1 = Cid::try_from("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG").unwrap();
        let cid2 = Cid::try_from("QmZfSNpGEAJXuJWGLzjoAXbNGcWE8y9YqJvpYfZZM6nqPs").unwrap();

        let mut msg1 = QueuedBitswapMessage::new();
        msg1.add_want_block(&cid1, 10);

        let mut msg2 = QueuedBitswapMessage::new();
        msg2.add_want_block(&cid2, 20);

        let merged = merge_messages(msg1, msg2);
        assert_eq!(merged.wantlist.len(), 2);
    }
}
