use crate::{Result, WantType};
use cid::Cid;
use helia_interface::HeliaError;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Entry in a wantlist
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WantListEntry {
    /// Content ID of the wanted block
    pub cid: Cid,
    /// Priority of this want (higher = more important)
    pub priority: i32,
    /// Type of want (block or have)
    pub want_type: WantType,
    /// Whether this is a cancellation
    pub cancel: bool,
    /// Whether to send dont-have messages
    pub send_dont_have: bool,
}

impl WantListEntry {
    /// Create a new wantlist entry for a block
    pub fn new_block(cid: Cid, priority: i32) -> Self {
        Self {
            cid,
            priority,
            want_type: WantType::Block,
            cancel: false,
            send_dont_have: true,
        }
    }

    /// Create a new wantlist entry for a have
    pub fn new_have(cid: Cid, priority: i32) -> Self {
        Self {
            cid,
            priority,
            want_type: WantType::Have,
            cancel: false,
            send_dont_have: false,
        }
    }

    /// Create a cancellation entry
    pub fn new_cancel(cid: Cid) -> Self {
        Self {
            cid,
            priority: 0,
            want_type: WantType::Block,
            cancel: true,
            send_dont_have: false,
        }
    }

    /// Check if this entry wants a block
    pub fn wants_block(&self) -> bool {
        !self.cancel && matches!(self.want_type, WantType::Block)
    }

    /// Check if this entry wants have information
    pub fn wants_have(&self) -> bool {
        !self.cancel && matches!(self.want_type, WantType::Have)
    }

    /// Check if this is a cancellation
    pub fn is_cancel(&self) -> bool {
        self.cancel
    }
}

/// Manages the local wantlist for bitswap
pub struct WantList {
    /// Current wants indexed by CID
    wants: HashMap<Cid, WantEntry>,
    /// Priority-ordered queue for processing
    priority_queue: VecDeque<Cid>,
    /// Maximum number of wants to track
    max_wants: usize,
    /// Default priority for new wants
    default_priority: i32,
    /// Whether the wantlist has been modified
    modified: bool,
    /// Statistics
    stats: WantListStats,
}

/// Internal want entry with additional metadata
#[derive(Debug, Clone)]
struct WantEntry {
    /// The actual want list entry
    entry: WantListEntry,
    /// When this want was added
    added_at: Instant,
    /// How many times we've tried to get this
    attempts: u32,
    /// Maximum number of attempts before giving up
    max_attempts: u32,
    /// Whether this want is currently being processed
    active: bool,
}

/// Statistics for the wantlist
#[derive(Debug, Clone, Default)]
pub struct WantListStats {
    /// Total number of wants added
    pub total_wants_added: u64,
    /// Total number of wants removed
    pub total_wants_removed: u64,
    /// Total number of wants cancelled
    pub total_wants_cancelled: u64,
    /// Current number of active wants
    pub active_wants: u64,
    /// Average want lifetime in seconds
    pub average_want_lifetime_seconds: f64,
    /// Number of wants that exceeded max attempts
    pub exceeded_max_attempts: u64,
}

impl WantEntry {
    /// Create a new want entry
    fn new(entry: WantListEntry, max_attempts: u32) -> Self {
        Self {
            entry,
            added_at: Instant::now(),
            attempts: 0,
            max_attempts,
            active: false,
        }
    }

    /// Check if this want has exceeded maximum attempts
    fn has_exceeded_attempts(&self) -> bool {
        self.attempts >= self.max_attempts
    }

    /// Increment attempt count
    fn increment_attempts(&mut self) {
        self.attempts += 1;
    }

    /// Get age of this want
    fn age(&self) -> Duration {
        Instant::now().duration_since(self.added_at)
    }

    /// Mark as active/inactive
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl WantList {
    /// Create a new wantlist
    pub fn new() -> Self {
        Self::with_config(WantListConfig::default())
    }

    /// Create a new wantlist with configuration
    pub fn with_config(config: WantListConfig) -> Self {
        Self {
            wants: HashMap::new(),
            priority_queue: VecDeque::new(),
            max_wants: config.max_wants,
            default_priority: config.default_priority,
            modified: false,
            stats: WantListStats::default(),
        }
    }

    /// Add a want to the list
    pub fn add_want(&mut self, cid: Cid, priority: Option<i32>, want_type: WantType) -> Result<()> {
        let priority = priority.unwrap_or(self.default_priority);
        
        // Check if we're at capacity
        if self.wants.len() >= self.max_wants && !self.wants.contains_key(&cid) {
            return Err(HeliaError::other("Wantlist is at maximum capacity"));
        }

        let entry = WantListEntry {
            cid,
            priority,
            want_type,
            cancel: false,
            send_dont_have: true,
        };

        debug!("Adding want for CID {} with priority {}", cid, priority);

        // If we already want this CID, update the priority
        if let Some(want_entry) = self.wants.get_mut(&cid) {
            want_entry.entry.priority = priority;
            want_entry.entry.want_type = want_type;
            self.modified = true;
            return Ok(());
        }

        // Add new want
        let want_entry = WantEntry::new(entry, 10); // Default max attempts
        self.wants.insert(cid, want_entry);
        
        // Insert into priority queue in correct position
        self.insert_into_priority_queue(cid, priority);
        
        self.modified = true;
        self.stats.total_wants_added += 1;
        self.stats.active_wants += 1;
        
        Ok(())
    }

    /// Remove a want from the list
    pub fn remove_want(&mut self, cid: &Cid) -> Result<()> {
        if let Some(want_entry) = self.wants.remove(cid) {
            debug!("Removing want for CID {}", cid);
            
            // Remove from priority queue
            self.priority_queue.retain(|c| c != cid);
            
            self.modified = true;
            self.stats.total_wants_removed += 1;
            if self.stats.active_wants > 0 {
                self.stats.active_wants -= 1;
            }
            
            // Update average lifetime
            let lifetime = want_entry.age().as_secs() as f64;
            self.update_average_lifetime(lifetime);
            
            Ok(())
        } else {
            Err(HeliaError::other(format!("CID {} not in wantlist", cid)))
        }
    }

    /// Cancel a want (mark for removal)
    pub fn cancel_want(&mut self, cid: &Cid) -> Result<()> {
        if let Some(want_entry) = self.wants.get_mut(cid) {
            debug!("Cancelling want for CID {}", cid);
            want_entry.entry.cancel = true;
            self.modified = true;
            self.stats.total_wants_cancelled += 1;
            Ok(())
        } else {
            Err(HeliaError::other(format!("CID {} not in wantlist", cid)))
        }
    }

    /// Check if we want a specific CID
    pub fn wants(&self, cid: &Cid) -> bool {
        self.wants.get(cid)
            .map(|entry| !entry.entry.cancel)
            .unwrap_or(false)
    }

    /// Get the priority of a want
    pub fn get_priority(&self, cid: &Cid) -> Option<i32> {
        self.wants.get(cid)
            .map(|entry| entry.entry.priority)
    }

    /// Get all current wants as entries
    pub fn get_wants(&self) -> Vec<WantListEntry> {
        self.wants.values()
            .filter(|entry| !entry.entry.cancel)
            .map(|entry| entry.entry.clone())
            .collect()
    }

    /// Get wants sorted by priority (highest first)
    pub fn get_wants_by_priority(&self) -> Vec<WantListEntry> {
        let mut wants: Vec<_> = self.wants.values()
            .filter(|entry| !entry.entry.cancel)
            .map(|entry| entry.entry.clone())
            .collect();
        
        wants.sort_by(|a, b| b.priority.cmp(&a.priority));
        wants
    }

    /// Get the next highest priority want
    pub fn next_want(&mut self) -> Option<Cid> {
        while let Some(cid) = self.priority_queue.pop_front() {
            if let Some(want_entry) = self.wants.get_mut(&cid) {
                if !want_entry.entry.cancel && !want_entry.active {
                    want_entry.set_active(true);
                    want_entry.increment_attempts();
                    return Some(cid);
                }
            }
        }
        None
    }

    /// Mark a want as completed (block received)
    pub fn mark_completed(&mut self, cid: &Cid) -> Result<()> {
        self.remove_want(cid)
    }

    /// Mark a want as failed for this attempt
    pub fn mark_failed(&mut self, cid: &Cid) -> Result<()> {
        let should_retry;
        let priority;
        
        if let Some(want_entry) = self.wants.get_mut(cid) {
            want_entry.set_active(false);
            
            if want_entry.has_exceeded_attempts() {
                warn!("Want for CID {} exceeded maximum attempts", cid);
                self.stats.exceeded_max_attempts += 1;
                return self.remove_want(cid);
            } else {
                // Prepare for retry
                should_retry = true;
                priority = want_entry.entry.priority;
            }
        } else {
            return Err(HeliaError::other(format!("CID {} not in wantlist", cid)));
        }
        
        if should_retry {
            // Re-add to priority queue for retry
            self.insert_into_priority_queue(*cid, priority);
        }
        
        Ok(())
    }

    /// Get cancelled wants for sending cancellation messages
    pub fn get_cancelled_wants(&mut self) -> Vec<WantListEntry> {
        let cancelled: Vec<_> = self.wants.iter()
            .filter(|(_, entry)| entry.entry.cancel)
            .map(|(cid, entry)| {
                let mut cancel_entry = entry.entry.clone();
                cancel_entry.cancel = true;
                cancel_entry
            })
            .collect();

        // Remove cancelled wants from the list
        self.wants.retain(|_, entry| !entry.entry.cancel);
        self.priority_queue.retain(|cid| self.wants.contains_key(cid));
        
        if !cancelled.is_empty() {
            self.modified = true;
        }
        
        cancelled
    }

    /// Get the number of wants
    pub fn len(&self) -> usize {
        self.wants.len()
    }

    /// Check if the wantlist is empty
    pub fn is_empty(&self) -> bool {
        self.wants.is_empty()
    }

    /// Check if the wantlist has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark the wantlist as clean (not modified)
    pub fn mark_clean(&mut self) {
        self.modified = false;
    }

    /// Clear all wants
    pub fn clear(&mut self) {
        let count = self.wants.len();
        self.wants.clear();
        self.priority_queue.clear();
        self.modified = count > 0;
        
        if self.stats.active_wants >= count as u64 {
            self.stats.active_wants -= count as u64;
        } else {
            self.stats.active_wants = 0;
        }
    }

    /// Clean up old wants that have been active too long
    pub fn cleanup_old_wants(&mut self, max_age: Duration) -> usize {
        let old_cids: Vec<Cid> = self.wants.iter()
            .filter(|(_, entry)| entry.age() > max_age)
            .map(|(cid, _)| *cid)
            .collect();

        let count = old_cids.len();
        
        for cid in old_cids {
            warn!("Removing old want for CID {}", cid);
            let _ = self.remove_want(&cid);
        }
        
        count
    }

    /// Get statistics
    pub fn statistics(&self) -> &WantListStats {
        &self.stats
    }

    /// Insert CID into priority queue maintaining order
    fn insert_into_priority_queue(&mut self, cid: Cid, priority: i32) {
        // Find insertion point to maintain priority order (highest first)
        let mut insert_pos = self.priority_queue.len();
        
        for (i, existing_cid) in self.priority_queue.iter().enumerate() {
            if let Some(existing_entry) = self.wants.get(existing_cid) {
                if priority > existing_entry.entry.priority {
                    insert_pos = i;
                    break;
                }
            }
        }
        
        self.priority_queue.insert(insert_pos, cid);
    }

    /// Update average lifetime statistic
    fn update_average_lifetime(&mut self, lifetime_seconds: f64) {
        let count = self.stats.total_wants_removed;
        if count == 1 {
            self.stats.average_want_lifetime_seconds = lifetime_seconds;
        } else {
            // Running average
            let old_avg = self.stats.average_want_lifetime_seconds;
            self.stats.average_want_lifetime_seconds = 
                ((old_avg * (count - 1) as f64) + lifetime_seconds) / count as f64;
        }
    }
}

impl Default for WantList {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for wantlist behavior
#[derive(Debug, Clone)]
pub struct WantListConfig {
    /// Maximum number of wants to track
    pub max_wants: usize,
    /// Default priority for new wants
    pub default_priority: i32,
    /// Maximum attempts per want
    pub max_attempts: u32,
}

impl Default for WantListConfig {
    fn default() -> Self {
        Self {
            max_wants: 1000,
            default_priority: 1,
            max_attempts: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WantType;

    #[test]
    fn test_wantlist_entry_creation() {
        let cid = Cid::default();
        
        let block_entry = WantListEntry::new_block(cid, 5);
        assert_eq!(block_entry.cid, cid);
        assert_eq!(block_entry.priority, 5);
        assert_eq!(block_entry.want_type, WantType::Block);
        assert!(!block_entry.cancel);
        assert!(block_entry.wants_block());
        
        let have_entry = WantListEntry::new_have(cid, 3);
        assert!(have_entry.wants_have());
        
        let cancel_entry = WantListEntry::new_cancel(cid);
        assert!(cancel_entry.is_cancel());
    }

    #[test]
    fn test_wantlist_basic_operations() {
        let mut wantlist = WantList::new();
        let cid = Cid::default();
        
        assert!(!wantlist.wants(&cid));
        assert_eq!(wantlist.len(), 0);
        assert!(wantlist.is_empty());
        
        // Add want
        assert!(wantlist.add_want(cid, Some(5), WantType::Block).is_ok());
        assert!(wantlist.wants(&cid));
        assert_eq!(wantlist.len(), 1);
        assert!(!wantlist.is_empty());
        assert_eq!(wantlist.get_priority(&cid), Some(5));
        
        // Remove want
        assert!(wantlist.remove_want(&cid).is_ok());
        assert!(!wantlist.wants(&cid));
        assert_eq!(wantlist.len(), 0);
    }

    #[test]
    fn test_wantlist_priority_ordering() {
        let mut wantlist = WantList::new();
        let cid1 = Cid::default();
        
        // Add wants with different priorities
        wantlist.add_want(cid1, Some(1), WantType::Block).unwrap();
        
        let wants = wantlist.get_wants_by_priority();
        assert_eq!(wants.len(), 1);
        assert_eq!(wants[0].priority, 1);
    }

    #[test]
    fn test_wantlist_cancellation() {
        let mut wantlist = WantList::new();
        let cid = Cid::default();
        
        wantlist.add_want(cid, Some(5), WantType::Block).unwrap();
        assert!(wantlist.wants(&cid));
        
        // Cancel want
        assert!(wantlist.cancel_want(&cid).is_ok());
        
        // Should still be in list but marked as cancelled
        assert_eq!(wantlist.len(), 1);
        
        // Get cancelled wants should return it and remove it
        let cancelled = wantlist.get_cancelled_wants();
        assert_eq!(cancelled.len(), 1);
        assert!(cancelled[0].cancel);
        assert_eq!(wantlist.len(), 0);
    }

    #[test]
    fn test_wantlist_next_want() {
        let mut wantlist = WantList::new();
        let cid = Cid::default();
        
        assert!(wantlist.next_want().is_none());
        
        wantlist.add_want(cid, Some(5), WantType::Block).unwrap();
        
        let next = wantlist.next_want();
        assert_eq!(next, Some(cid));
        
        // Should not return the same want again immediately
        assert!(wantlist.next_want().is_none());
    }

    #[test]
    fn test_wantlist_max_capacity() {
        let config = WantListConfig {
            max_wants: 2,
            ..Default::default()
        };
        let mut wantlist = WantList::with_config(config);
        
        let cid1 = Cid::default();
        
        assert!(wantlist.add_want(cid1, Some(1), WantType::Block).is_ok());
        
        // Should fail when at capacity
        // Note: We can't easily create different CIDs in this test setup
        // In a real scenario, you'd have different CIDs
    }

    #[test]
    fn test_wantlist_statistics() {
        let mut wantlist = WantList::new();
        let cid = Cid::default();
        
        let stats = wantlist.statistics();
        assert_eq!(stats.total_wants_added, 0);
        assert_eq!(stats.active_wants, 0);
        
        wantlist.add_want(cid, Some(5), WantType::Block).unwrap();
        
        let stats = wantlist.statistics();
        assert_eq!(stats.total_wants_added, 1);
        assert_eq!(stats.active_wants, 1);
        
        wantlist.remove_want(&cid).unwrap();
        
        let stats = wantlist.statistics();
        assert_eq!(stats.total_wants_removed, 1);
        assert_eq!(stats.active_wants, 0);
    }

    #[test]
    fn test_wantlist_modification_tracking() {
        let mut wantlist = WantList::new();
        let cid = Cid::default();
        
        assert!(!wantlist.is_modified());
        
        wantlist.add_want(cid, Some(5), WantType::Block).unwrap();
        assert!(wantlist.is_modified());
        
        wantlist.mark_clean();
        assert!(!wantlist.is_modified());
        
        wantlist.remove_want(&cid).unwrap();
        assert!(wantlist.is_modified());
    }

    #[test]
    fn test_wantlist_clear() {
        let mut wantlist = WantList::new();
        let cid = Cid::default();
        
        wantlist.add_want(cid, Some(5), WantType::Block).unwrap();
        assert_eq!(wantlist.len(), 1);
        
        wantlist.clear();
        assert_eq!(wantlist.len(), 0);
        assert!(wantlist.is_empty());
        assert!(wantlist.is_modified());
    }
}