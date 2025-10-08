//! # Helia Interface
//!
//! The API defined by a Helia node
//!
//! This crate provides the core interfaces and traits that define the Helia IPFS implementation.
//! 
//! ## Example
//!
//! ```rust
//! use helia_interface::Helia;
//!
//! async fn do_something<H: Helia>(helia: H) {
//!     // use helia node functions here
//! }
//! ```

pub mod blocks;
pub mod pins;
pub mod routing;
pub mod errors;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use cid::Cid;
use futures::Stream;
use libp2p::Swarm;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use trust_dns_resolver::TokioAsyncResolver;

pub use blocks::*;
pub use pins::*;
pub use routing::*;
pub use errors::*;

/// Type alias for async iterables/streams
pub type AwaitIterable<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

/// Type alias for awaitable results
pub type Await<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Options that include an abort signal for canceling operations
#[derive(Debug, Default)]
pub struct AbortOptions {
    // For now, we'll use a simpler approach without tokio channels
    // pub signal: Option<mpsc::Receiver<()>>,
}

impl Clone for AbortOptions {
    fn clone(&self) -> Self {
        // AbortOptions can't be cloned due to the receiver, so we create a new default one
        Self::default()
    }
}

/// Progress event for tracking operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent<T> {
    pub event_type: String,
    pub detail: T,
}

/// Options for operations that support progress tracking
pub struct ProgressOptions<T> {
    /// Optional progress event handler
    pub on_progress: Option<Box<dyn Fn(ProgressEvent<T>) + Send + Sync>>,
}

impl<T> std::fmt::Debug for ProgressOptions<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressOptions")
            .field("on_progress", &self.on_progress.as_ref().map(|_| "Some(closure)"))
            .finish()
    }
}

impl<T> Default for ProgressOptions<T> {
    fn default() -> Self {
        Self {
            on_progress: None,
        }
    }
}

impl<T> Clone for ProgressOptions<T> {
    fn clone(&self) -> Self {
        // Progress handlers can't be cloned, so we create a new default one
        Self::default()
    }
}

/// Codec loader for loading IPLD codecs
#[async_trait]
pub trait CodecLoader: Send + Sync {
    /// Load a codec by its code
    async fn load_codec(&self, code: u64) -> Result<Box<dyn Codec>, HeliaError>;
}

/// Hasher loader for loading multihash hashers
#[async_trait]
pub trait HasherLoader: Send + Sync {
    /// Load a hasher by its code
    async fn load_hasher(&self, code: u64) -> Result<Box<dyn Hasher>, HeliaError>;
}

/// IPLD codec trait
#[async_trait]
pub trait Codec: Send + Sync {
    /// Encode data using this codec
    async fn encode(&self, data: &[u8]) -> Result<Bytes, HeliaError>;
    
    /// Decode data using this codec
    async fn decode(&self, data: &[u8]) -> Result<Bytes, HeliaError>;
    
    /// Get the codec code
    fn code(&self) -> u64;
}

/// Multihash hasher trait
#[async_trait]
pub trait Hasher: Send + Sync {
    /// Hash data using this hasher
    async fn hash(&self, data: &[u8]) -> Result<multihash::Multihash<64>, HeliaError>;
    
    /// Get the hasher code
    fn code(&self) -> u64;
}

/// Events emitted by a Helia node
#[derive(Debug, Clone)]
pub enum HeliaEvent {
    /// Node has started
    Start,
    /// Node has stopped
    Stop,
}

/// Garbage collection options
#[derive(Debug)]
pub struct GcOptions {
    /// Abort options
    pub abort: AbortOptions,
    /// Progress options for GC events
    pub progress: ProgressOptions<GcEvent>,
}

impl Default for GcOptions {
    fn default() -> Self {
        Self {
            abort: AbortOptions::default(),
            progress: ProgressOptions::default(),
        }
    }
}

impl Clone for GcOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
        }
    }
}

/// Events emitted during garbage collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GcEvent {
    /// A CID was deleted
    Deleted(Cid),
    /// An error occurred during GC
    Error(String),
}

/// Component logger for structured logging
pub trait ComponentLogger: Send + Sync {
    /// Log a debug message
    fn debug(&self, message: &str);
    /// Log an info message
    fn info(&self, message: &str);
    /// Log a warning message
    fn warn(&self, message: &str);
    /// Log an error message
    fn error(&self, message: &str);
}

/// Metrics collection interface
#[async_trait]
pub trait Metrics: Send + Sync {
    /// Record a counter metric
    async fn record_counter(&self, name: &str, value: u64, labels: HashMap<String, String>);
    
    /// Record a gauge metric
    async fn record_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>);
    
    /// Record a histogram metric
    async fn record_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>);
}

/// Non-generic Helia trait for backward compatibility and trait objects
#[async_trait]
pub trait Helia: Send + Sync {
    /// The blockstore for storing blocks
    fn blockstore(&self) -> &dyn Blocks;
    
    /// The datastore for key-value storage
    fn datastore(&self) -> &dyn Datastore;
    
    /// Pinning operations
    fn pins(&self) -> &dyn Pins;
    
    /// The logger component
    fn logger(&self) -> &dyn ComponentLogger;
    
    /// The routing component
    fn routing(&self) -> &dyn Routing;
    
    /// DNS resolver
    fn dns(&self) -> &TokioAsyncResolver;
    
    /// Optional metrics collector
    fn metrics(&self) -> Option<&dyn Metrics>;
    
    /// Start the Helia node
    async fn start(&self) -> Result<(), HeliaError>;
    
    /// Stop the Helia node
    async fn stop(&self) -> Result<(), HeliaError>;
    
    /// Perform garbage collection
    async fn gc(&self, options: Option<GcOptions>) -> Result<(), HeliaError>;
    
    /// Load an IPLD codec
    async fn get_codec(&self, code: u64) -> Result<Box<dyn Codec>, HeliaError>;
    
    /// Load a hasher
    async fn get_hasher(&self, code: u64) -> Result<Box<dyn Hasher>, HeliaError>;
}

/// Generic Helia trait with libp2p type parameter for concrete implementations
#[async_trait]
pub trait HeliaWithLibp2p<T>: Helia 
where 
    T: libp2p::swarm::NetworkBehaviour + Send + 'static
{
    /// The libp2p swarm instance (wrapped in Arc<Mutex<>> for thread safety)
    fn libp2p(&self) -> Arc<Mutex<Swarm<T>>>;
}

/// Key-value datastore interface
#[async_trait]
pub trait Datastore: Send + Sync {
    /// Get a value by key
    async fn get(&self, key: &[u8]) -> Result<Option<Bytes>, HeliaError>;
    
    /// Put a key-value pair
    async fn put(&self, key: &[u8], value: Bytes) -> Result<(), HeliaError>;
    
    /// Delete a key
    async fn delete(&self, key: &[u8]) -> Result<(), HeliaError>;
    
    /// Check if a key exists
    async fn has(&self, key: &[u8]) -> Result<bool, HeliaError>;
    
    /// Query for keys with optional filters
    async fn query(&self, prefix: Option<&[u8]>) -> Result<AwaitIterable<Bytes>, HeliaError>;
}