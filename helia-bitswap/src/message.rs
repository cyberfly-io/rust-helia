use cid::Cid;
use serde::{Deserialize, Serialize};

/// Bitswap protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitswapMessage {
    /// Wantlist in the message
    pub wantlist: Option<Wantlist>,
    /// Blocks being sent
    pub blocks: Vec<Block>,
    /// Block presence information
    pub block_presences: Vec<BlockPresenceProto>,
    /// Number of pending bytes
    pub pending_bytes: i32,
}

/// Wantlist in a bitswap message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wantlist {
    /// Wantlist entries
    pub entries: Vec<WantListEntryProto>,
    /// Whether this is a full wantlist or partial update
    pub full: bool,
}

/// Wantlist entry in protocol format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantListEntryProto {
    /// Block CID as bytes
    pub block: Vec<u8>,
    /// Priority of the request
    pub priority: i32,
    /// Whether this is a cancellation
    pub cancel: bool,
    /// Want type (0 = Block, 1 = Have)
    pub want_type: i32,
    /// Send don't have flag
    pub send_dont_have: bool,
}

/// Block in a bitswap message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block prefix (CID prefix)
    pub prefix: Vec<u8>,
    /// Block data
    pub data: Vec<u8>,
}

/// Block presence information in protocol format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPresenceProto {
    /// Block CID as bytes
    pub cid: Vec<u8>,
    /// Type of presence (0 = Have, 1 = DontHave)
    pub r#type: i32,
}

impl BitswapMessage {
    /// Create a new empty message
    pub fn new() -> Self {
        Self {
            wantlist: None,
            blocks: Vec::new(),
            block_presences: Vec::new(),
            pending_bytes: 0,
        }
    }

    /// Create a message with wantlist
    pub fn with_wantlist(entries: Vec<WantListEntryProto>, full: bool) -> Self {
        Self {
            wantlist: Some(Wantlist { entries, full }),
            blocks: Vec::new(),
            block_presences: Vec::new(),
            pending_bytes: 0,
        }
    }

    /// Create a message with blocks
    pub fn with_blocks(blocks: Vec<Block>) -> Self {
        Self {
            wantlist: None,
            blocks,
            block_presences: Vec::new(),
            pending_bytes: 0,
        }
    }

    /// Add a block to the message
    pub fn add_block(&mut self, cid: Cid, data: Vec<u8>) {
        let block = Block {
            prefix: cid.to_bytes()[..4].to_vec(), // Simplified prefix
            data,
        };
        self.blocks.push(block);
    }

    /// Add block presence information
    pub fn add_block_presence(&mut self, cid: Cid, presence_type: BlockPresenceType) {
        let presence = BlockPresenceProto {
            cid: cid.to_bytes(),
            r#type: match presence_type {
                BlockPresenceType::Have => 0,
                BlockPresenceType::DontHave => 1,
            },
        };
        self.block_presences.push(presence);
    }

    /// Check if message is empty
    pub fn is_empty(&self) -> bool {
        self.wantlist.as_ref().map_or(true, |w| w.entries.is_empty())
            && self.blocks.is_empty()
            && self.block_presences.is_empty()
    }

    /// Get estimated size of the message in bytes
    pub fn estimated_size(&self) -> usize {
        let mut size = 0;
        
        if let Some(wantlist) = &self.wantlist {
            size += wantlist.entries.len() * 50; // Approximate size per entry
        }
        
        for block in &self.blocks {
            size += block.data.len() + block.prefix.len();
        }
        
        for presence in &self.block_presences {
            size += presence.cid.len() + 4; // CID + type field
        }
        
        size + 20 // Base overhead
    }
}

/// Block presence type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPresenceType {
    Have,
    DontHave,
}

impl Default for BitswapMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl WantListEntryProto {
    /// Create a new wantlist entry
    pub fn new(cid: Cid, priority: i32, want_type: i32, cancel: bool) -> Self {
        Self {
            block: cid.to_bytes(),
            priority,
            cancel,
            want_type,
            send_dont_have: false,
        }
    }

    /// Create a block request entry
    pub fn new_block_request(cid: Cid, priority: i32) -> Self {
        Self::new(cid, priority, 0, false) // 0 = Block
    }

    /// Create a have request entry
    pub fn new_have_request(cid: Cid, priority: i32) -> Self {
        Self::new(cid, priority, 1, false) // 1 = Have
    }

    /// Create a cancel entry
    pub fn new_cancel(cid: Cid) -> Self {
        Self::new(cid, 0, 0, true)
    }

    /// Get the CID from the entry
    pub fn get_cid(&self) -> Result<Cid, cid::Error> {
        Cid::try_from(&self.block[..])
    }
}

/// Message builder for constructing bitswap messages
pub struct MessageBuilder {
    message: BitswapMessage,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            message: BitswapMessage::new(),
        }
    }

    /// Add wantlist entries
    pub fn with_wantlist(mut self, entries: Vec<WantListEntryProto>, full: bool) -> Self {
        self.message.wantlist = Some(Wantlist { entries, full });
        self
    }

    /// Add a block
    pub fn with_block(mut self, cid: Cid, data: Vec<u8>) -> Self {
        self.message.add_block(cid, data);
        self
    }

    /// Add block presence
    pub fn with_presence(mut self, cid: Cid, presence_type: BlockPresenceType) -> Self {
        self.message.add_block_presence(cid, presence_type);
        self
    }

    /// Set pending bytes
    pub fn with_pending_bytes(mut self, pending: i32) -> Self {
        self.message.pending_bytes = pending;
        self
    }

    /// Build the message
    pub fn build(self) -> BitswapMessage {
        self.message
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_message() {
        let msg = BitswapMessage::new();
        assert!(msg.is_empty());
        assert_eq!(msg.blocks.len(), 0);
        assert_eq!(msg.block_presences.len(), 0);
        assert!(msg.wantlist.is_none());
    }

    #[test]
    fn test_message_with_blocks() {
        let mut msg = BitswapMessage::new();
        let cid = Cid::default();
        let data = vec![1, 2, 3, 4, 5];
        
        msg.add_block(cid, data.clone());
        
        assert!(!msg.is_empty());
        assert_eq!(msg.blocks.len(), 1);
        assert_eq!(msg.blocks[0].data, data);
    }

    #[test]
    fn test_message_builder() {
        let cid = Cid::default();
        let entry = WantListEntryProto::new_block_request(cid, 10);
        
        let msg = MessageBuilder::new()
            .with_wantlist(vec![entry], true)
            .with_pending_bytes(100)
            .build();
        
        assert!(msg.wantlist.is_some());
        assert_eq!(msg.pending_bytes, 100);
        
        let wantlist = msg.wantlist.unwrap();
        assert!(wantlist.full);
        assert_eq!(wantlist.entries.len(), 1);
        assert_eq!(wantlist.entries[0].priority, 10);
    }

    #[test]
    fn test_wantlist_entry_types() {
        let cid = Cid::default();
        
        let block_entry = WantListEntryProto::new_block_request(cid, 5);
        assert_eq!(block_entry.want_type, 0);
        assert!(!block_entry.cancel);
        
        let have_entry = WantListEntryProto::new_have_request(cid, 3);
        assert_eq!(have_entry.want_type, 1);
        assert!(!have_entry.cancel);
        
        let cancel_entry = WantListEntryProto::new_cancel(cid);
        assert!(cancel_entry.cancel);
    }

    #[test]
    fn test_estimated_size() {
        let msg = BitswapMessage::new();
        assert!(msg.estimated_size() > 0);
        
        let mut msg_with_block = BitswapMessage::new();
        msg_with_block.add_block(Cid::default(), vec![0; 1000]);
        assert!(msg_with_block.estimated_size() > 1000);
    }
}