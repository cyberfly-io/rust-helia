//! HTTP Gateway Routing implementation
//!
//! This module provides a simple content router that returns HTTP gateway URLs
//! as "providers". This allows using HTTP gateways as a fallback for content routing.

use async_trait::async_trait;
use cid::Cid;
use futures::stream;
use helia_interface::{
    AwaitIterable, FindPeersOptions, FindProvidersOptions, GetOptions, HeliaError, PeerInfo,
    ProvideOptions, Provider, PutOptions, Routing, RoutingRecord, TransportMethod,
};
use libp2p::{Multiaddr, PeerId};
use std::str::FromStr;
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

    /// Convert gateway info to Provider format
    fn gateway_to_provider(&self, gateway: &Url) -> Option<Provider> {
        let peer_id = Self::gateway_to_peer_id(gateway);
        let addr = Self::gateway_to_multiaddr(gateway)?;

        Some(Provider {
            peer_info: PeerInfo {
                id: peer_id,
                multiaddrs: vec![addr],
                protocols: vec!["http".to_string()],
            },
            transport_methods: vec![TransportMethod::Http],
        })
    }
}

#[async_trait]
impl Routing for HTTPGatewayRouter {
    async fn find_providers(
        &self,
        cid: &Cid,
        _options: Option<FindProvidersOptions>,
    ) -> Result<AwaitIterable<Provider>, HeliaError> {
        debug!(
            "HTTPGatewayRouter: Returning {} gateways as providers for {}",
            self.gateways.len(),
            cid
        );

        let mut providers = Vec::new();

        for gateway in &self.gateways {
            if let Some(provider) = self.gateway_to_provider(gateway) {
                providers.push(provider);
            }
        }

        if providers.is_empty() {
            return Err(HeliaError::NotFound(format!(
                "No valid gateway providers found for CID: {}",
                cid
            )));
        }

        // Convert Vec to async stream
        Ok(Box::pin(stream::iter(providers)))
    }

    async fn provide(&self, _cid: &Cid, _options: Option<ProvideOptions>) -> Result<(), HeliaError> {
        // HTTP gateways are read-only, we can't announce content
        Err(HeliaError::OperationNotSupported(
            "HTTP gateway routing does not support content announcement".to_string(),
        ))
    }

    async fn find_peers(
        &self,
        _peer_id: &PeerId,
        _options: Option<FindPeersOptions>,
    ) -> Result<AwaitIterable<PeerInfo>, HeliaError> {
        // HTTP gateways don't support peer routing
        Err(HeliaError::OperationNotSupported(
            "HTTP gateway routing does not support peer discovery".to_string(),
        ))
    }

    async fn get(
        &self,
        _key: &[u8],
        _options: Option<GetOptions>,
    ) -> Result<Option<RoutingRecord>, HeliaError> {
        // HTTP gateways don't support DHT records
        Err(HeliaError::OperationNotSupported(
            "HTTP gateway routing does not support DHT record retrieval".to_string(),
        ))
    }

    async fn put(
        &self,
        _key: &[u8],
        _value: &[u8],
        _options: Option<PutOptions>,
    ) -> Result<(), HeliaError> {
        // HTTP gateways don't support DHT records
        Err(HeliaError::OperationNotSupported(
            "HTTP gateway routing does not support DHT record storage".to_string(),
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
pub fn http_gateway_routing(init: HTTPGatewayRoutingInit) -> Box<dyn Routing> {
    Box::new(HTTPGatewayRouter::new(init))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_gateway_router_creation() {
        let router = HTTPGatewayRouter::new(HTTPGatewayRoutingInit::default());
        assert_eq!(router.gateways.len(), 3);
    }

    #[tokio::test]
    async fn test_find_providers_returns_gateways() {
        let router = http_gateway_routing(HTTPGatewayRoutingInit::default());
        let cid = Cid::default();

        let mut providers = router.find_providers(&cid, None).await.unwrap();
        let provider_vec: Vec<_> = providers.collect().await;
        assert_eq!(provider_vec.len(), 3);

        for provider in &provider_vec {
            assert!(!provider.peer_info.multiaddrs.is_empty());
            assert_eq!(provider.transport_methods.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_provide_not_supported() {
        let router = http_gateway_routing(HTTPGatewayRoutingInit::default());
        let cid = Cid::default();

        let result = router.provide(&cid, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_custom_gateways() {
        let custom_gateway = Url::parse("https://custom.gateway.example").unwrap();

        let router = http_gateway_routing(HTTPGatewayRoutingInit {
            gateways: vec![custom_gateway],
        });

        let cid = Cid::default();
        let mut providers = router.find_providers(&cid, None).await.unwrap();
        let provider_vec: Vec<_> = providers.collect().await;

        assert_eq!(provider_vec.len(), 1);
    }

    #[tokio::test]
    async fn test_peer_routing_not_supported() {
        let router = http_gateway_routing(HTTPGatewayRoutingInit::default());
        let peer_id = PeerId::random();

        let result = router.find_peers(&peer_id, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dht_operations_not_supported() {
        let router = http_gateway_routing(HTTPGatewayRoutingInit::default());

        // Test get
        let get_result = router.get(b"test-key", None).await;
        assert!(get_result.is_err());

        // Test put
        let put_result = router.put(b"test-key", b"test-value", None).await;
        assert!(put_result.is_err());
    }
}
