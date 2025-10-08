use crate::{Result, WantListEntry, WantType};
use cid::Cid;
use helia_interface::HeliaError;
use libp2p::PeerId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// A bitswap session for efficient block retrieval
pub struct Session {
    /// Session ID
    id: String,
    /// Blocks this session is interested in
    interests: HashSet<Cid>,
    /// Blocks that have been received
    received_blocks: HashSet<Cid>,
    /// Peers that are part of this session
    peers: HashSet<PeerId>,
    /// When the session was created
    created_at: Instant,
    /// Last activity time
    last_activity: Instant,
    /// Session timeout
    timeout: Duration,
    /// Whether the session is active
    active: bool,
    /// Session priority
    priority: i32,
    /// Maximum number of peers to use
    max_peers: usize,
    /// Session statistics
    stats: SessionStats,
}

/// Session statistics
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    /// Number of blocks requested
    pub blocks_requested: u64,
    /// Number of blocks received
    pub blocks_received: u64,
    /// Number of blocks failed
    pub blocks_failed: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Number of peers tried
    pub peers_tried: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Session duration
    pub duration: Duration,
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session timeout
    pub timeout: Duration,
    /// Session priority
    pub priority: i32,
    /// Maximum number of peers to use
    pub max_peers: usize,
    /// Whether to rebroadcast wants periodically
    pub rebroadcast_wants: bool,
    /// Rebroadcast interval
    pub rebroadcast_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            priority: 1,
            max_peers: 10,
            rebroadcast_wants: true,
            rebroadcast_interval: Duration::from_secs(30),
        }
    }
}

impl Session {
    /// Create a new session
    pub fn new(id: String, config: SessionConfig) -> Self {
        let now = Instant::now();
        Self {
            id,
            interests: HashSet::new(),
            received_blocks: HashSet::new(),
            peers: HashSet::new(),
            created_at: now,
            last_activity: now,
            timeout: config.timeout,
            active: true,
            priority: config.priority,
            max_peers: config.max_peers,
            stats: SessionStats::default(),
        }
    }

    /// Get session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.active && !self.is_expired()
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.last_activity) > self.timeout
    }

    /// Add interest in a block
    pub fn add_interest(&mut self, cid: Cid) {
        debug!("Session {} adding interest in {}", self.id, cid);
        self.interests.insert(cid);
        self.update_activity();
        self.stats.blocks_requested += 1;
    }

    /// Remove interest in a block
    pub fn remove_interest(&mut self, cid: &Cid) {
        debug!("Session {} removing interest in {}", self.id, cid);
        self.interests.remove(cid);
        self.update_activity();
    }

    /// Get all interests
    pub fn interests(&self) -> &HashSet<Cid> {
        &self.interests
    }

    /// Get pending interests (not yet received)
    pub fn pending_interests(&self) -> HashSet<Cid> {
        self.interests.difference(&self.received_blocks).cloned().collect()
    }

    /// Mark a block as received
    pub fn mark_block_received(&mut self, cid: &Cid, size: usize) {
        debug!("Session {} received block {}", self.id, cid);
        self.received_blocks.insert(*cid);
        self.interests.remove(cid);
        self.update_activity();
        
        self.stats.blocks_received += 1;
        self.stats.bytes_received += size as u64;
    }

    /// Mark a block as failed
    pub fn mark_block_failed(&mut self, cid: &Cid) {
        warn!("Session {} failed to get block {}", self.id, cid);
        self.interests.remove(cid);
        self.update_activity();
        self.stats.blocks_failed += 1;
    }

    /// Add a peer to the session
    pub fn add_peer(&mut self, peer_id: PeerId) -> bool {
        if self.peers.len() >= self.max_peers {
            return false;
        }
        
        if self.peers.insert(peer_id) {
            debug!("Session {} added peer {}", self.id, peer_id);
            self.update_activity();
            self.stats.peers_tried += 1;
            true
        } else {
            false
        }
    }

    /// Remove a peer from the session
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        if self.peers.remove(peer_id) {
            debug!("Session {} removed peer {}", self.id, peer_id);
            self.update_activity();
        }
    }

    /// Get session peers
    pub fn peers(&self) -> &HashSet<PeerId> {
        &self.peers
    }

    /// Get session priority
    pub fn priority(&self) -> i32 {
        self.priority
    }

    /// Set session priority
    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
        self.update_activity();
    }

    /// Generate wantlist for this session
    pub fn generate_wantlist(&self) -> Vec<WantListEntry> {
        self.pending_interests()
            .into_iter()
            .map(|cid| WantListEntry {
                cid,
                priority: self.priority,
                want_type: WantType::Block,
                cancel: false,
                send_dont_have: true,
            })
            .collect()
    }

    /// Check if session wants a specific block
    pub fn wants_block(&self, cid: &Cid) -> bool {
        self.interests.contains(cid) && !self.received_blocks.contains(cid)
    }

    /// Close the session
    pub fn close(&mut self) {
        info!("Closing session {}", self.id);
        self.active = false;
        self.interests.clear();
        self.peers.clear();
    }

    /// Update last activity time
    fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Get session statistics
    pub fn statistics(&self) -> SessionStats {
        let mut stats = self.stats.clone();
        stats.duration = Instant::now().duration_since(self.created_at);
        
        // Calculate average response time if we have received blocks
        if stats.blocks_received > 0 {
            // This is a simplified calculation - in a real implementation,
            // you'd track individual request/response times
            stats.average_response_time_ms = stats.duration.as_millis() as f64 / stats.blocks_received as f64;
        }
        
        stats
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.stats.blocks_requested == 0 {
            return 100.0;
        }
        
        (self.stats.blocks_received as f64 / self.stats.blocks_requested as f64) * 100.0
    }
}

/// Manages multiple bitswap sessions
pub struct SessionManager {
    /// Active sessions
    sessions: HashMap<String, Session>,
    /// Session creation counter for unique IDs
    session_counter: u64,
    /// Default session configuration
    default_config: SessionConfig,
    /// Whether the manager is running
    running: bool,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_counter: 0,
            default_config: SessionConfig::default(),
            running: false,
        }
    }

    /// Create a new session manager with custom config
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            session_counter: 0,
            default_config: config,
            running: false,
        }
    }

    /// Start the session manager
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }

        info!("Starting session manager");
        self.running = true;
        Ok(())
    }

    /// Stop the session manager
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        info!("Stopping session manager");
        
        // Close all sessions
        for session in self.sessions.values_mut() {
            session.close();
        }
        self.sessions.clear();
        
        self.running = false;
        Ok(())
    }

    /// Create a new session
    pub fn create_session(&mut self) -> Result<String> {
        self.create_session_with_config(self.default_config.clone())
    }

    /// Create a new session with custom config
    pub fn create_session_with_config(&mut self, config: SessionConfig) -> Result<String> {
        if !self.running {
            return Err(HeliaError::other("Session manager not running"));
        }

        self.session_counter += 1;
        let session_id = format!("session-{}", self.session_counter);
        
        let session = Session::new(session_id.clone(), config);
        self.sessions.insert(session_id.clone(), session);
        
        info!("Created session {}", session_id);
        Ok(session_id)
    }

    /// Get a session
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// Get a mutable session
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    /// Close a session
    pub fn close_session(&mut self, session_id: &str) -> Result<()> {
        if let Some(mut session) = self.sessions.remove(session_id) {
            session.close();
            info!("Closed session {}", session_id);
        }
        Ok(())
    }

    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<&Session> {
        self.sessions.values().filter(|s| s.is_active()).collect()
    }

    /// Get sessions interested in a specific block
    pub fn sessions_wanting_block(&self, cid: &Cid) -> Vec<&Session> {
        self.sessions.values()
            .filter(|s| s.is_active() && s.wants_block(cid))
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) -> usize {
        let expired_sessions: Vec<String> = self.sessions.iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired_sessions.len();
        
        for session_id in expired_sessions {
            warn!("Removing expired session: {}", session_id);
            let _ = self.close_session(&session_id);
        }
        
        count
    }

    /// Get all wantlists from active sessions
    pub fn get_all_wantlists(&self) -> HashMap<String, Vec<WantListEntry>> {
        self.sessions.iter()
            .filter(|(_, session)| session.is_active())
            .map(|(id, session)| (id.clone(), session.generate_wantlist()))
            .collect()
    }

    /// Get aggregated wantlist across all sessions
    pub fn get_aggregated_wantlist(&self) -> Vec<WantListEntry> {
        let mut aggregated: HashMap<Cid, WantListEntry> = HashMap::new();
        
        for session in self.sessions.values() {
            if session.is_active() {
                for entry in session.generate_wantlist() {
                    // Use highest priority for each CID across sessions
                    aggregated.entry(entry.cid)
                        .and_modify(|existing| {
                            if entry.priority > existing.priority {
                                existing.priority = entry.priority;
                            }
                        })
                        .or_insert(entry);
                }
            }
        }
        
        aggregated.into_values().collect()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_active()).count()
    }

    /// Get manager statistics
    pub fn get_statistics(&self) -> SessionManagerStats {
        let mut stats = SessionManagerStats::default();
        
        stats.total_sessions = self.sessions.len();
        stats.active_sessions = self.active_session_count();
        
        for session in self.sessions.values() {
            let session_stats = session.statistics();
            stats.total_blocks_requested += session_stats.blocks_requested;
            stats.total_blocks_received += session_stats.blocks_received;
            stats.total_blocks_failed += session_stats.blocks_failed;
            stats.total_bytes_received += session_stats.bytes_received;
            
            if session.is_active() {
                stats.total_pending_interests += session.pending_interests().len() as u64;
            }
        }
        
        if stats.total_blocks_requested > 0 {
            stats.success_rate = (stats.total_blocks_received as f64 / stats.total_blocks_requested as f64) * 100.0;
        }
        
        stats
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the session manager
#[derive(Debug, Clone, Default)]
pub struct SessionManagerStats {
    /// Total number of sessions
    pub total_sessions: usize,
    /// Number of active sessions
    pub active_sessions: usize,
    /// Total blocks requested across all sessions
    pub total_blocks_requested: u64,
    /// Total blocks received across all sessions
    pub total_blocks_received: u64,
    /// Total blocks failed across all sessions
    pub total_blocks_failed: u64,
    /// Total bytes received across all sessions
    pub total_bytes_received: u64,
    /// Total pending interests across active sessions
    pub total_pending_interests: u64,
    /// Success rate as percentage
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_lifecycle() {
        let mut manager = SessionManager::new();
        assert!(!manager.running);
        
        assert!(manager.start().await.is_ok());
        assert!(manager.running);
        
        assert!(manager.stop().await.is_ok());
        assert!(!manager.running);
    }

    #[tokio::test]
    async fn test_session_creation() {
        let mut manager = SessionManager::new();
        manager.start().await.unwrap();
        
        let session_id = manager.create_session().unwrap();
        assert_eq!(manager.session_count(), 1);
        
        let session = manager.get_session(&session_id).unwrap();
        assert!(session.is_active());
        assert_eq!(session.id(), session_id);
    }

    #[test]
    fn test_session_interests() {
        let mut session = Session::new("test".to_string(), SessionConfig::default());
        let cid = Cid::default();
        
        assert!(!session.wants_block(&cid));
        
        session.add_interest(cid);
        assert!(session.wants_block(&cid));
        assert_eq!(session.interests().len(), 1);
        
        session.mark_block_received(&cid, 100);
        assert!(!session.wants_block(&cid));
        assert_eq!(session.pending_interests().len(), 0);
    }

    #[test]
    fn test_session_wantlist() {
        let mut session = Session::new("test".to_string(), SessionConfig::default());
        let cid1 = Cid::default();
        
        session.add_interest(cid1);
        
        let wantlist = session.generate_wantlist();
        assert_eq!(wantlist.len(), 1);
        assert_eq!(wantlist[0].cid, cid1);
        assert!(!wantlist[0].cancel);
    }

    #[test]
    fn test_session_statistics() {
        let mut session = Session::new("test".to_string(), SessionConfig::default());
        let cid = Cid::default();
        
        session.add_interest(cid);
        session.mark_block_received(&cid, 100);
        
        let stats = session.statistics();
        assert_eq!(stats.blocks_requested, 1);
        assert_eq!(stats.blocks_received, 1);
        assert_eq!(stats.bytes_received, 100);
        assert_eq!(session.completion_percentage(), 100.0);
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let mut manager = SessionManager::new();
        let config = SessionConfig {
            timeout: Duration::from_millis(1), // Very short timeout
            ..Default::default()
        };
        manager.start().await.unwrap();
        
        let session_id = manager.create_session_with_config(config).unwrap();
        assert_eq!(manager.session_count(), 1);
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let cleaned = manager.cleanup_expired_sessions();
        assert_eq!(cleaned, 1);
        assert_eq!(manager.session_count(), 0);
    }
}