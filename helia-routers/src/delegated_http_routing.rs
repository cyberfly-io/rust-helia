//! Delegated HTTP Routing implementation
//!
//! This module implements content routing using the Delegated Routing V1 HTTP API.
//! It queries HTTP endpoints to find providers for content.
//!
//! See: https://specs.ipfs.tech/routing/http-routing-v1/

use crate::{ContentRouting, ProviderInfo, RoutingError};
use async_trait::async_trait;
use cid::Cid;
use libp2p::{Multiaddr, PeerId};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// Default Delegated Routing V1 endpoints
const DEFAULT_ENDPOINTS: &[&str] = &["https://cid.contact", "https://delegated-ipfs.dev"];

/// Configuration for delegated HTTP routing
#[derive(Debug, Clone)]
pub struct DelegatedHTTPRoutingInit {
    /// List of delegated routing endpoints
    pub endpoints: Vec<Url>,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum number of providers to return
    pub max_providers: usize,
}

impl Default for DelegatedHTTPRoutingInit {
    fn default() -> Self {
        Self {
            endpoints: DEFAULT_ENDPOINTS
                .iter()
                .filter_map(|url| Url::parse(url).ok())
                .collect(),
            timeout_ms: 30000, // 30 seconds
            max_providers: 20,
        }
    }
}

/// Provider record from Delegated Routing V1 API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProviderRecord {
    /// Protocol identifier
    #[serde(default)]
    protocol: String,

    /// Schema identifier
    #[serde(default)]
    schema: String,

    /// Provider peer ID
    #[serde(rename = "ID")]
    id: Option<String>,

    /// Provider multiaddresses
    #[serde(default)]
    addrs: Vec<String>,
}

/// Response from Delegated Routing V1 API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RoutingResponse {
    /// List of provider records
    #[serde(default)]
    providers: Vec<ProviderRecord>,
}

/// Delegated HTTP Router implementation
pub struct DelegatedHTTPRouter {
    /// HTTP client
    client: Client,

    /// Routing endpoints
    endpoints: Vec<Url>,

    /// Configuration
    config: DelegatedHTTPRoutingInit,
}

impl DelegatedHTTPRouter {
    /// Create a new delegated HTTP router
    pub fn new(init: DelegatedHTTPRoutingInit) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(init.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            endpoints: init.endpoints.clone(),
            config: init,
        }
    }

    /// Query a single endpoint for providers
    async fn query_endpoint(
        &self,
        endpoint: &Url,
        cid: &Cid,
    ) -> Result<Vec<ProviderInfo>, RoutingError> {
        // Construct URL: {endpoint}/routing/v1/providers/{cid}
        let mut url = endpoint.clone();
        url.set_path(&format!("/routing/v1/providers/{}", cid));

        debug!("Querying delegated routing endpoint: {}", url);

        // Make HTTP request
        let response = self
            .client
            .get(url.clone())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to query {}: {}", url, e);
                RoutingError::RoutingFailed(format!("HTTP request failed: {}", e))
            })?;

        if !response.status().is_success() {
            warn!("Endpoint returned error status: {}", response.status());
            return Err(RoutingError::RoutingFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        // Parse JSON response
        let routing_response: RoutingResponse = response.json().await.map_err(|e| {
            warn!("Failed to parse response from {}: {}", url, e);
            RoutingError::RoutingFailed(format!("Invalid JSON response: {}", e))
        })?;

        debug!(
            "Found {} provider records from {}",
            routing_response.providers.len(),
            url
        );

        // Convert provider records to ProviderInfo
        let mut providers = Vec::new();
        for record in routing_response
            .providers
            .iter()
            .take(self.config.max_providers)
        {
            // Parse peer ID
            let peer_id = if let Some(id_str) = &record.id {
                match PeerId::from_str(id_str) {
                    Ok(pid) => pid,
                    Err(e) => {
                        warn!("Invalid peer ID '{}': {}", id_str, e);
                        continue;
                    }
                }
            } else {
                warn!("Provider record missing ID field");
                continue;
            };

            // Parse multiaddresses
            let mut addrs = Vec::new();
            for addr_str in &record.addrs {
                match Multiaddr::from_str(addr_str) {
                    Ok(addr) => addrs.push(addr),
                    Err(e) => {
                        warn!("Invalid multiaddr '{}': {}", addr_str, e);
                    }
                }
            }

            if addrs.is_empty() {
                debug!("Provider {} has no valid addresses, skipping", peer_id);
                continue;
            }

            providers.push(ProviderInfo { peer_id, addrs });
        }

        Ok(providers)
    }
}

#[async_trait]
impl ContentRouting for DelegatedHTTPRouter {
    async fn find_providers(&self, cid: &Cid) -> Result<Vec<ProviderInfo>, RoutingError> {
        let mut all_providers = Vec::new();
        let mut last_error = None;

        // Query all endpoints
        for endpoint in &self.endpoints {
            match self.query_endpoint(endpoint, cid).await {
                Ok(mut providers) => {
                    debug!("Got {} providers from {}", providers.len(), endpoint);
                    all_providers.append(&mut providers);
                }
                Err(e) => {
                    warn!("Endpoint {} failed: {}", endpoint, e);
                    last_error = Some(e);
                }
            }
        }

        // If we got any providers, return them
        if !all_providers.is_empty() {
            // Deduplicate by peer ID
            all_providers.sort_by_key(|p| p.peer_id);
            all_providers.dedup_by_key(|p| p.peer_id);

            // Limit to max_providers
            all_providers.truncate(self.config.max_providers);

            debug!("Returning {} unique providers", all_providers.len());
            return Ok(all_providers);
        }

        // All endpoints failed
        Err(last_error.unwrap_or_else(|| RoutingError::ContentNotFound(*cid)))
    }

    async fn provide(&self, _cid: &Cid) -> Result<(), RoutingError> {
        // Delegated routing is read-only, we can't announce content
        Err(RoutingError::RoutingFailed(
            "Delegated HTTP routing does not support content announcement".to_string(),
        ))
    }
}

/// Factory function to create a delegated HTTP router
///
/// # Example
///
/// ```no_run
/// use helia_routers::delegated_http_routing::{delegated_http_routing, DelegatedHTTPRoutingInit};
///
/// let router = delegated_http_routing(DelegatedHTTPRoutingInit::default());
/// ```
pub fn delegated_http_routing(init: DelegatedHTTPRoutingInit) -> Arc<dyn ContentRouting> {
    Arc::new(DelegatedHTTPRouter::new(init))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delegated_router_creation() {
        let router = DelegatedHTTPRouter::new(DelegatedHTTPRoutingInit::default());
        assert!(!router.endpoints.is_empty());
    }

    #[tokio::test]
    async fn test_provide_not_supported() {
        let router = delegated_http_routing(DelegatedHTTPRoutingInit::default());
        let cid = Cid::default();

        let result = router.provide(&cid).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_find_providers_real() {
        // Use a well-known CID
        let cid =
            Cid::try_from("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();

        let router = delegated_http_routing(DelegatedHTTPRoutingInit {
            timeout_ms: 10000,
            ..Default::default()
        });

        match router.find_providers(&cid).await {
            Ok(providers) => {
                println!("Found {} providers", providers.len());
                for provider in &providers {
                    println!(
                        "  Provider: {} with {} addrs",
                        provider.peer_id,
                        provider.addrs.len()
                    );
                }
            }
            Err(e) => {
                println!("Warning: Failed to find providers: {}", e);
            }
        }
    }
}
