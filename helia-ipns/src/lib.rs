//! IPNS (InterPlanetary Name System) implementation for Helia in Rust
//!
//! IPNS provides mutable pointers to content-addressed data.

mod constants;
mod errors;
mod ipns_impl;
pub mod keys;
mod local_store;
mod protobuf;
pub mod record;
pub mod routing;

pub use errors::IpnsError;
pub use local_store::{LocalStore, RecordMetadata};
pub use record::{
    select_best_record, sign_record, validate_ipns_record, verify_signature, IpnsRecord,
};
pub use routing::{
    DhtRouter, GetOptions, HttpRouter, IpnsRouting, LocalRouter, PutOptions, RoutingEvent,
};

use async_trait::async_trait;
use cid::Cid;
use libp2p_identity::PeerId;
use std::sync::Arc;
use std::time::Duration;

pub use constants::*;

/// Options for publishing IPNS records
#[derive(Debug, Clone)]
pub struct PublishOptions {
    pub lifetime: Option<u64>,
    pub offline: bool,
    pub ttl: Option<u64>,
    pub v1_compatible: bool,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            lifetime: Some(DEFAULT_LIFETIME_MS),
            offline: false,
            ttl: Some(DEFAULT_TTL_NS / 1_000_000),
            v1_compatible: true,
        }
    }
}

/// Options for resolving IPNS names
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub offline: bool,
    pub nocache: bool,
    pub max_depth: Option<u32>,
    pub timeout: Option<Duration>,
}

/// Result of resolving an IPNS name
#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub cid: Cid,
    pub path: String,
    pub record: IpnsRecord,
}

/// Result of publishing an IPNS record
#[derive(Debug, Clone)]
pub struct PublishResult {
    pub record: IpnsRecord,
    pub public_key: Vec<u8>,
}

/// Initialization options for IPNS
#[derive(Debug, Clone)]
pub struct IpnsInit {
    pub routers: Vec<Arc<dyn IpnsRouting>>,
    pub republish_interval: Option<Duration>,
    pub republish_concurrency: Option<usize>,
    pub enable_republish: bool,
}

impl Default for IpnsInit {
    fn default() -> Self {
        Self {
            routers: Vec::new(),
            republish_interval: Some(Duration::from_millis(DEFAULT_REPUBLISH_INTERVAL_MS)),
            republish_concurrency: Some(5),
            enable_republish: true,
        }
    }
}

/// The main IPNS interface
#[async_trait]
pub trait Ipns: Send + Sync {
    fn routers(&self) -> &[Arc<dyn IpnsRouting>];

    async fn publish(
        &self,
        key_name: &str,
        value: &Cid,
        options: PublishOptions,
    ) -> Result<PublishResult, IpnsError>;

    async fn resolve(
        &self,
        key: &[u8],
        options: ResolveOptions,
    ) -> Result<ResolveResult, IpnsError>;

    async fn resolve_peer_id(
        &self,
        peer_id: &PeerId,
        options: ResolveOptions,
    ) -> Result<ResolveResult, IpnsError>;

    async fn unpublish(&self, key_name: &str) -> Result<(), IpnsError>;

    async fn start(&self) -> Result<(), IpnsError>;

    async fn stop(&self) -> Result<(), IpnsError>;
}

/// Factory function to create an IPNS instance
pub fn ipns(init: IpnsInit) -> Result<Arc<dyn Ipns>, IpnsError> {
    ipns_impl::IpnsImpl::new(init)
}
