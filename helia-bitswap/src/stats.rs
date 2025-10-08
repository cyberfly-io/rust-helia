use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use libp2p::PeerId;
use cid::Cid;

/// Comprehensive statistics for the bitswap protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitswapStats {
    /// General protocol statistics
    pub protocol: ProtocolStats,
    /// Network-related statistics
    pub network: NetworkStats,
    /// Block-related statistics
    pub blocks: BlockStats,
    /// Peer-related statistics
    pub peers: PeerStats,
    /// Session-related statistics
    pub sessions: SessionStats,
    /// Performance metrics
    pub performance: PerformanceStats,
    /// Error statistics
    pub errors: ErrorStats,
    /// Timestamp when stats were collected
    pub timestamp: u64,
}

/// General protocol statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// Total number of messages sent
    pub messages_sent: u64,
    /// Total number of messages received
    pub messages_received: u64,
    /// Total bytes sent in messages
    pub message_bytes_sent: u64,
    /// Total bytes received in messages
    pub message_bytes_received: u64,
    /// Protocol uptime in seconds
    pub uptime_seconds: u64,
    /// Number of protocol restarts
    pub restarts: u64,
}

/// Network-related statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total number of connections established
    pub connections_established: u64,
    /// Total number of connections closed
    pub connections_closed: u64,
    /// Current number of active connections
    pub active_connections: u64,
    /// Total connection failures
    pub connection_failures: u64,
    /// Average connection duration in seconds
    pub average_connection_duration_seconds: f64,
    /// Network bandwidth utilization (bytes per second)
    pub bandwidth_utilization_bps: u64,
}

/// Block-related statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockStats {
    /// Total blocks sent to peers
    pub blocks_sent: u64,
    /// Total blocks received from peers
    pub blocks_received: u64,
    /// Total blocks requested but not received
    pub blocks_requested: u64,
    /// Total blocks that failed to be retrieved
    pub blocks_failed: u64,
    /// Total bytes of block data sent
    pub block_bytes_sent: u64,
    /// Total bytes of block data received
    pub block_bytes_received: u64,
    /// Average block size in bytes
    pub average_block_size: f64,
    /// Number of duplicate blocks received
    pub duplicate_blocks: u64,
    /// Hit rate for block requests (percentage)
    pub hit_rate_percentage: f64,
}

/// Peer-related statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerStats {
    /// Total number of unique peers encountered
    pub total_peers: u64,
    /// Current number of connected peers
    pub connected_peers: u64,
    /// Number of high-quality peers (score > 0.8)
    pub high_quality_peers: u64,
    /// Average peer quality score
    pub average_peer_quality: f64,
    /// Total number of peer timeouts
    pub peer_timeouts: u64,
    /// Total number of peer disconnections
    pub peer_disconnections: u64,
    /// Average number of blocks per peer
    pub average_blocks_per_peer: f64,
}

/// Session-related statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of sessions created
    pub total_sessions: u64,
    /// Current number of active sessions
    pub active_sessions: u64,
    /// Total sessions completed successfully
    pub completed_sessions: u64,
    /// Total sessions that timed out
    pub expired_sessions: u64,
    /// Average session duration in seconds
    pub average_session_duration_seconds: f64,
    /// Average blocks per session
    pub average_blocks_per_session: f64,
    /// Session success rate (percentage)
    pub session_success_rate_percentage: f64,
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// Average response time for block requests in milliseconds
    pub average_response_time_ms: f64,
    /// 95th percentile response time in milliseconds
    pub p95_response_time_ms: f64,
    /// 99th percentile response time in milliseconds
    pub p99_response_time_ms: f64,
    /// Average throughput in blocks per second
    pub throughput_blocks_per_second: f64,
    /// Average throughput in bytes per second
    pub throughput_bytes_per_second: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU utilization percentage
    pub cpu_utilization_percentage: f64,
}

/// Error statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Total number of errors encountered
    pub total_errors: u64,
    /// Network-related errors
    pub network_errors: u64,
    /// Protocol errors
    pub protocol_errors: u64,
    /// Timeout errors
    pub timeout_errors: u64,
    /// Peer-related errors
    pub peer_errors: u64,
    /// Block validation errors
    pub validation_errors: u64,
    /// Other/unknown errors
    pub other_errors: u64,
}

/// Statistics collector for bitswap operations
pub struct StatsCollector {
    /// Current statistics
    stats: BitswapStats,
    /// Start time for uptime calculation
    start_time: Instant,
    /// Response time samples for percentile calculation
    response_times: Vec<f64>,
    /// Maximum number of response time samples to keep
    max_samples: usize,
    /// Per-peer statistics
    peer_stats: HashMap<PeerId, PeerStatsEntry>,
    /// Per-session statistics
    session_stats: HashMap<String, SessionStatsEntry>,
}

/// Per-peer statistics entry
#[derive(Debug, Clone)]
struct PeerStatsEntry {
    /// Number of blocks sent to this peer
    blocks_sent: u64,
    /// Number of blocks received from this peer
    blocks_received: u64,
    /// Total bytes sent to this peer
    bytes_sent: u64,
    /// Total bytes received from this peer
    bytes_received: u64,
    /// Connection start time
    connected_at: Instant,
    /// Last activity time
    last_activity: Instant,
    /// Quality score
    quality_score: f64,
}

/// Per-session statistics entry
#[derive(Debug, Clone)]
struct SessionStatsEntry {
    /// Session start time
    started_at: Instant,
    /// Number of blocks requested in this session
    blocks_requested: u64,
    /// Number of blocks received in this session
    blocks_received: u64,
    /// Whether the session completed successfully
    completed: bool,
    /// Whether the session expired
    expired: bool,
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            stats: BitswapStats::new(),
            start_time: Instant::now(),
            response_times: Vec::new(),
            max_samples: 10000,
            peer_stats: HashMap::new(),
            session_stats: HashMap::new(),
        }
    }

    /// Record a message sent
    pub fn record_message_sent(&mut self, size: usize) {
        self.stats.protocol.messages_sent += 1;
        self.stats.protocol.message_bytes_sent += size as u64;
    }

    /// Record a message received
    pub fn record_message_received(&mut self, size: usize) {
        self.stats.protocol.messages_received += 1;
        self.stats.protocol.message_bytes_received += size as u64;
    }

    /// Record a block sent
    pub fn record_block_sent(&mut self, peer_id: &PeerId, cid: &Cid, size: usize) {
        self.stats.blocks.blocks_sent += 1;
        self.stats.blocks.block_bytes_sent += size as u64;
        
        let peer_entry = self.peer_stats.entry(*peer_id).or_insert_with(|| PeerStatsEntry {
            blocks_sent: 0,
            blocks_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            quality_score: 1.0,
        });
        
        peer_entry.blocks_sent += 1;
        peer_entry.bytes_sent += size as u64;
        peer_entry.last_activity = Instant::now();
    }

    /// Record a block received
    pub fn record_block_received(&mut self, peer_id: &PeerId, cid: &Cid, size: usize, response_time_ms: f64) {
        self.stats.blocks.blocks_received += 1;
        self.stats.blocks.block_bytes_received += size as u64;
        
        // Record response time
        self.response_times.push(response_time_ms);
        if self.response_times.len() > self.max_samples {
            self.response_times.remove(0);
        }
        
        let peer_entry = self.peer_stats.entry(*peer_id).or_insert_with(|| PeerStatsEntry {
            blocks_sent: 0,
            blocks_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            quality_score: 1.0,
        });
        
        peer_entry.blocks_received += 1;
        peer_entry.bytes_received += size as u64;
        peer_entry.last_activity = Instant::now();
    }

    /// Record a block request
    pub fn record_block_requested(&mut self, cid: &Cid) {
        self.stats.blocks.blocks_requested += 1;
    }

    /// Record a block request failure
    pub fn record_block_failed(&mut self, cid: &Cid) {
        self.stats.blocks.blocks_failed += 1;
    }

    /// Record a duplicate block
    pub fn record_duplicate_block(&mut self, cid: &Cid) {
        self.stats.blocks.duplicate_blocks += 1;
    }

    /// Record a peer connection
    pub fn record_peer_connected(&mut self, peer_id: PeerId) {
        self.stats.network.connections_established += 1;
        self.stats.network.active_connections += 1;
        
        self.peer_stats.insert(peer_id, PeerStatsEntry {
            blocks_sent: 0,
            blocks_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            quality_score: 1.0,
        });
    }

    /// Record a peer disconnection
    pub fn record_peer_disconnected(&mut self, peer_id: &PeerId) {
        self.stats.network.connections_closed += 1;
        if self.stats.network.active_connections > 0 {
            self.stats.network.active_connections -= 1;
        }
        self.stats.peers.peer_disconnections += 1;
        
        self.peer_stats.remove(peer_id);
    }

    /// Record a session started
    pub fn record_session_started(&mut self, session_id: String) {
        self.stats.sessions.total_sessions += 1;
        self.stats.sessions.active_sessions += 1;
        
        self.session_stats.insert(session_id, SessionStatsEntry {
            started_at: Instant::now(),
            blocks_requested: 0,
            blocks_received: 0,
            completed: false,
            expired: false,
        });
    }

    /// Record a session completed
    pub fn record_session_completed(&mut self, session_id: &str) {
        if let Some(entry) = self.session_stats.get_mut(session_id) {
            entry.completed = true;
        }
        
        self.stats.sessions.completed_sessions += 1;
        if self.stats.sessions.active_sessions > 0 {
            self.stats.sessions.active_sessions -= 1;
        }
    }

    /// Record a session expired
    pub fn record_session_expired(&mut self, session_id: &str) {
        if let Some(entry) = self.session_stats.get_mut(session_id) {
            entry.expired = true;
        }
        
        self.stats.sessions.expired_sessions += 1;
        if self.stats.sessions.active_sessions > 0 {
            self.stats.sessions.active_sessions -= 1;
        }
    }

    /// Record an error
    pub fn record_error(&mut self, error_type: ErrorType) {
        self.stats.errors.total_errors += 1;
        
        match error_type {
            ErrorType::Network => self.stats.errors.network_errors += 1,
            ErrorType::Protocol => self.stats.errors.protocol_errors += 1,
            ErrorType::Timeout => self.stats.errors.timeout_errors += 1,
            ErrorType::Peer => self.stats.errors.peer_errors += 1,
            ErrorType::Validation => self.stats.errors.validation_errors += 1,
            ErrorType::Other => self.stats.errors.other_errors += 1,
        }
    }

    /// Update computed statistics
    pub fn update_computed_stats(&mut self) {
        // Update uptime
        self.stats.protocol.uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Update block statistics
        if self.stats.blocks.blocks_received > 0 {
            self.stats.blocks.average_block_size = 
                self.stats.blocks.block_bytes_received as f64 / self.stats.blocks.blocks_received as f64;
        }
        
        if self.stats.blocks.blocks_requested > 0 {
            self.stats.blocks.hit_rate_percentage = 
                (self.stats.blocks.blocks_received as f64 / self.stats.blocks.blocks_requested as f64) * 100.0;
        }
        
        // Update peer statistics
        self.stats.peers.connected_peers = self.peer_stats.len() as u64;
        self.stats.peers.total_peers = self.stats.network.connections_established;
        
        if !self.peer_stats.is_empty() {
            let total_quality: f64 = self.peer_stats.values().map(|p| p.quality_score).sum();
            self.stats.peers.average_peer_quality = total_quality / self.peer_stats.len() as f64;
            
            let high_quality_count = self.peer_stats.values()
                .filter(|p| p.quality_score > 0.8)
                .count();
            self.stats.peers.high_quality_peers = high_quality_count as u64;
            
            let total_blocks: u64 = self.peer_stats.values()
                .map(|p| p.blocks_sent + p.blocks_received)
                .sum();
            self.stats.peers.average_blocks_per_peer = total_blocks as f64 / self.peer_stats.len() as f64;
        }
        
        // Update performance statistics
        if !self.response_times.is_empty() {
            let sum: f64 = self.response_times.iter().sum();
            self.stats.performance.average_response_time_ms = sum / self.response_times.len() as f64;
            
            // Calculate percentiles
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let p95_index = ((sorted_times.len() as f64) * 0.95) as usize;
            let p99_index = ((sorted_times.len() as f64) * 0.99) as usize;
            
            if p95_index < sorted_times.len() {
                self.stats.performance.p95_response_time_ms = sorted_times[p95_index];
            }
            
            if p99_index < sorted_times.len() {
                self.stats.performance.p99_response_time_ms = sorted_times[p99_index];
            }
        }
        
        // Update throughput
        let uptime_seconds = self.stats.protocol.uptime_seconds as f64;
        if uptime_seconds > 0.0 {
            self.stats.performance.throughput_blocks_per_second = 
                self.stats.blocks.blocks_received as f64 / uptime_seconds;
            self.stats.performance.throughput_bytes_per_second = 
                self.stats.blocks.block_bytes_received as f64 / uptime_seconds;
        }
        
        // Update session statistics
        if !self.session_stats.is_empty() {
            let total_duration: Duration = self.session_stats.values()
                .map(|s| s.started_at.elapsed())
                .sum();
            self.stats.sessions.average_session_duration_seconds = 
                total_duration.as_secs() as f64 / self.session_stats.len() as f64;
            
            let total_blocks: u64 = self.session_stats.values()
                .map(|s| s.blocks_received)
                .sum();
            self.stats.sessions.average_blocks_per_session = 
                total_blocks as f64 / self.session_stats.len() as f64;
            
            let completed_sessions = self.session_stats.values()
                .filter(|s| s.completed)
                .count();
            self.stats.sessions.session_success_rate_percentage = 
                (completed_sessions as f64 / self.session_stats.len() as f64) * 100.0;
        }
        
        // Update timestamp
        self.stats.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get current statistics
    pub fn get_stats(&mut self) -> BitswapStats {
        self.update_computed_stats();
        self.stats.clone()
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        self.stats = BitswapStats::new();
        self.start_time = Instant::now();
        self.response_times.clear();
        self.peer_stats.clear();
        self.session_stats.clear();
    }

    /// Get peer-specific statistics
    pub fn get_peer_stats(&self, peer_id: &PeerId) -> Option<&PeerStatsEntry> {
        self.peer_stats.get(peer_id)
    }

    /// Get session-specific statistics
    pub fn get_session_stats(&self, session_id: &str) -> Option<&SessionStatsEntry> {
        self.session_stats.get(session_id)
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl BitswapStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            protocol: ProtocolStats::default(),
            network: NetworkStats::default(),
            blocks: BlockStats::default(),
            peers: PeerStats::default(),
            sessions: SessionStats::default(),
            performance: PerformanceStats::default(),
            errors: ErrorStats::default(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

impl Default for BitswapStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for statistics tracking
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Network,
    Protocol,
    Timeout,
    Peer,
    Validation,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    fn create_test_peer() -> PeerId {
        let keypair = Keypair::generate_ed25519();
        PeerId::from(keypair.public())
    }

    #[test]
    fn test_stats_collector_creation() {
        let collector = StatsCollector::new();
        let stats = collector.stats;
        
        assert_eq!(stats.protocol.messages_sent, 0);
        assert_eq!(stats.blocks.blocks_sent, 0);
        assert_eq!(stats.peers.connected_peers, 0);
    }

    #[test]
    fn test_message_statistics() {
        let mut collector = StatsCollector::new();
        
        collector.record_message_sent(100);
        collector.record_message_received(200);
        
        let stats = collector.get_stats();
        assert_eq!(stats.protocol.messages_sent, 1);
        assert_eq!(stats.protocol.messages_received, 1);
        assert_eq!(stats.protocol.message_bytes_sent, 100);
        assert_eq!(stats.protocol.message_bytes_received, 200);
    }

    #[test]
    fn test_block_statistics() {
        let mut collector = StatsCollector::new();
        let peer = create_test_peer();
        let cid = Cid::default();
        
        collector.record_block_sent(&peer, &cid, 1024);
        collector.record_block_received(&peer, &cid, 2048, 100.0);
        
        let stats = collector.get_stats();
        assert_eq!(stats.blocks.blocks_sent, 1);
        assert_eq!(stats.blocks.blocks_received, 1);
        assert_eq!(stats.blocks.block_bytes_sent, 1024);
        assert_eq!(stats.blocks.block_bytes_received, 2048);
        assert_eq!(stats.blocks.average_block_size, 2048.0);
    }

    #[test]
    fn test_peer_statistics() {
        let mut collector = StatsCollector::new();
        let peer = create_test_peer();
        
        collector.record_peer_connected(peer);
        assert_eq!(collector.get_stats().network.active_connections, 1);
        
        collector.record_peer_disconnected(&peer);
        assert_eq!(collector.get_stats().network.active_connections, 0);
        assert_eq!(collector.get_stats().network.connections_closed, 1);
    }

    #[test]
    fn test_session_statistics() {
        let mut collector = StatsCollector::new();
        let session_id = "test-session".to_string();
        
        collector.record_session_started(session_id.clone());
        assert_eq!(collector.get_stats().sessions.active_sessions, 1);
        
        collector.record_session_completed(&session_id);
        assert_eq!(collector.get_stats().sessions.completed_sessions, 1);
        assert_eq!(collector.get_stats().sessions.active_sessions, 0);
    }

    #[test]
    fn test_error_statistics() {
        let mut collector = StatsCollector::new();
        
        collector.record_error(ErrorType::Network);
        collector.record_error(ErrorType::Protocol);
        collector.record_error(ErrorType::Timeout);
        
        let stats = collector.get_stats();
        assert_eq!(stats.errors.total_errors, 3);
        assert_eq!(stats.errors.network_errors, 1);
        assert_eq!(stats.errors.protocol_errors, 1);
        assert_eq!(stats.errors.timeout_errors, 1);
    }

    #[test]
    fn test_response_time_percentiles() {
        let mut collector = StatsCollector::new();
        
        // Add some response times
        for i in 1..=100 {
            collector.response_times.push(i as f64);
        }
        
        let stats = collector.get_stats();
        assert_eq!(stats.performance.average_response_time_ms, 50.5);
        // P95 could be 95 or 96 depending on implementation
        assert!(stats.performance.p95_response_time_ms >= 94.0 && stats.performance.p95_response_time_ms <= 96.0);
        // P99 could be 99 or 100 depending on implementation
        assert!(stats.performance.p99_response_time_ms >= 98.0 && stats.performance.p99_response_time_ms <= 100.0);
    }

    #[test]
    fn test_reset_statistics() {
        let mut collector = StatsCollector::new();
        
        collector.record_message_sent(100);
        collector.record_block_received(&create_test_peer(), &Cid::default(), 1024, 50.0);
        
        assert!(collector.get_stats().protocol.messages_sent > 0);
        
        collector.reset();
        
        let stats = collector.get_stats();
        assert_eq!(stats.protocol.messages_sent, 0);
        assert_eq!(stats.blocks.blocks_received, 0);
    }
}