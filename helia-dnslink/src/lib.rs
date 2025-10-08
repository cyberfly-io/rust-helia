//! DNSLink resolution for Helia

mod dnslink;
mod errors;
mod resolver;
mod namespaces;

pub use dnslink::{DNSLink, dns_link, DnsLinkInit};
pub use errors::DnsLinkError;
pub use resolver::{DnsResolver, TxtRecord};

use cid::Cid;
use libp2p_identity::PeerId;

pub const MAX_RECURSIVE_DEPTH: u32 = 32;

#[derive(Debug, Clone, PartialEq)]
pub enum DnsLinkResult {
    IPFS {
        answer: TxtRecord,
        namespace: String,
        cid: Cid,
        path: String,
    },
    IPNS {
        answer: TxtRecord,
        namespace: String,
        peer_id: PeerId,
        path: String,
    },
    Other {
        answer: TxtRecord,
        namespace: String,
        value: String,
    },
}

#[derive(Debug, Clone)]
pub struct ResolveOptions {
    pub nocache: bool,
    pub offline: bool,
    pub max_recursive_depth: Option<u32>,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            nocache: false,
            offline: false,
            max_recursive_depth: Some(MAX_RECURSIVE_DEPTH),
        }
    }
}
