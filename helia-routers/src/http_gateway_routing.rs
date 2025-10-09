//! HTTP Gateway Routing implementation
//!
//! This module provides a simple content router that returns HTTP gateway URLs
//! as "providers". This allows using HTTP gateways as a fallback for content routing.

use crate::{ContentRouting, ProviderInfo, RoutingError};
use async_trait::async_trait;
use cid::Cid;
use libp2p::{Multiaddr, PeerId};
use std::str::FromStr;
use std::sync::Arc;
use tracing::debug;
use url::Url;

/// Configuration for HTTP gateway routing
#[derive(Debug, Clone)]
pub struct HTTPGatewayRoutingInit {
    /// List of HTTP gateways to use
    pub gateways: Vec<Url>,
}

impl Default for HTTPGatewayRoutingInit {
    fn default() -> Self {
        Self {
            gateways: vec![
                Url::parse("https://ipfs.io").unwrap(),
                Url::parse("https://dweb.link").unwrap(),
                Url::parse("https://cloudflare-ipfs.com").unwrap(),
            ],
        }
    }
}

/// HTTP Gateway Router implementation
///
/// This router returns gateway URLs as "providers". It creates synthetic peer IDs
/// and multiaddresses that represent HTTP gateways rather than P2P peers.
pub struct HTTPGatewayRouter {
    /// Gateway URLs
    gateways: Vec<Url>,
}

impl HTTPGatewayRouter {
    /// Create a new HTTP gateway router
    pub fn new(init: HTTPGatewayRoutingInit) -> Self {
        Self {
            gateways: init.gateways,
        }
    }

    /// Create a synthetic peer ID from a gateway URL
    fn gateway_to_peer_id(gateway: &Url) -> PeerId {
        // Create a deterministic peer ID from the gateway URL
        // In a real implementation, this might use a hash of the URL
        // For now, we'll create a simple synthetic peer ID

        // Use the URL host as a seed for the peer ID
        let host = gateway.host_str().unwrap_or("unknown");
        let hash = seahash::hash(host.as_bytes());

        // Create a peer ID from the hash (simplified)
        // In production, you'd use proper libp2p key generation
        let peer_id_bytes = format!("12D3KooW{:032x}", hash);
        PeerId::from_str(&peer_id_bytes).unwrap_or_else(|_| {
            // Fallback to a default peer ID if parsing fails
            PeerId::random()
        })
    }

    /// Create a synthetic multiaddress for an HTTP gateway
    fn gateway_to_multiaddr(gateway: &Url) -> Option<Multiaddr> {
        // Convert HTTP(S) URL to a multiaddr
        // Format: /dns4/{host}/tcp/{port}/http or /dns4/{host}/tcp/{port}/https

        let host = gateway.host_str()?;
        let port = gateway
            .port()
            .unwrap_or(if gateway.scheme() == "https" { 443 } else { 80 });
        let protocol = if gateway.scheme() == "https" {
            "https"
        } else {
            "http"
        };

        let multiaddr_str = format!("/dns4/{}/tcp/{}/{}", host, port, protocol);
        Multiaddr::from_str(&multiaddr_str).ok()
    }
}

#[async_trait]
impl ContentRouting for HTTPGatewayRouter {
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<ProviderInfo>, RoutingError> {
        debug!(
            "HTTPGatewayRouter: Returning {} gateways as providers for {}",
            self.gateways.len(),
            cid
        );

        let mut providers = Vec::new();

        for gateway in &self.gateways {
            let peer_id = Self::gateway_to_peer_id(gateway);

            let addrs = if let Some(addr) = Self::gateway_to_multiaddr(gateway) {
                vec![addr]
            } else {
                // Fallback: create a simple multiaddr from the URL
                vec![]
            };

            if !addrs.is_empty() {
                providers.push(ProviderInfo { peer_id, addrs });
            }
        }

        if providers.is_empty() {
            return Err(RoutingError::ContentNotFound(*cid));
        }

        Ok(providers)
    }

    async fn provide(&self, _cid: &Cid) -> Result<(), RoutingError> {
        // HTTP gateways are read-only, we can't announce content
        Err(RoutingError::RoutingFailed(
            "HTTP gateway routing does not support content announcement".to_string(),
        ))
    }
}

/// Factory function to create an HTTP gateway router
///
/// # Example
///
/// ```no_run
/// use helia_routers::http_gateway_routing::{http_gateway_routing, HTTPGatewayRoutingInit};
/// use url::Url;
///
/// let router = http_gateway_routing(HTTPGatewayRoutingInit {
///     gateways: vec![Url::parse("https://ipfs.io").unwrap()],
/// });
/// ```
pub fn http_gateway_routing(init: HTTPGatewayRoutingInit) -> Arc<dyn ContentRouting> {
    Arc::new(HTTPGatewayRouter::new(init))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_router_creation() {
        let router = HTTPGatewayRouter::new(HTTPGatewayRoutingInit::default());
        assert_eq!(router.gateways.len(), 3);
    }

    #[tokio::test]
    async fn test_find_providers_returns_gateways() {
        let router = http_gateway_routing(HTTPGatewayRoutingInit::default());
        let cid = Cid::default();

        let providers = router.find_providers(&cid).await.unwrap();
        assert_eq!(providers.len(), 3);

        for provider in &providers {
            assert!(!provider.addrs.is_empty());
        }
    }

    #[tokio::test]
    async fn test_provide_not_supported() {
        let router = http_gateway_routing(HTTPGatewayRoutingInit::default());
        let cid = Cid::default();

        let result = router.provide(&cid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_custom_gateways() {
        let custom_gateway = Url::parse("https://custom.gateway.example").unwrap();

        let router = http_gateway_routing(HTTPGatewayRoutingInit {
            gateways: vec![custom_gateway],
        });

        let cid = Cid::default();
        let providers = router.find_providers(&cid).await.unwrap();

        assert_eq!(providers.len(), 1);
    }
}
