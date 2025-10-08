use crate::errors::DnsLinkError;
use hickory_resolver::{config::ResolverConfig, TokioAsyncResolver};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxtRecord {
    pub name: String,
    pub ttl: u32,
    pub data: String,
}

pub struct DnsResolver {
    resolver: Arc<RwLock<TokioAsyncResolver>>,
    cache_enabled: bool,
}

impl DnsResolver {
    pub fn new() -> Result<Self, DnsLinkError> {
        Self::with_config(ResolverConfig::cloudflare_https(), true)
    }

    pub fn with_config(config: ResolverConfig, cache_enabled: bool) -> Result<Self, DnsLinkError> {
        let resolver = TokioAsyncResolver::tokio(config, Default::default());
        Ok(Self {
            resolver: Arc::new(RwLock::new(resolver)),
            cache_enabled,
        })
    }

    pub async fn query_txt(&self, domain: &str) -> Result<Vec<TxtRecord>, DnsLinkError> {
        debug!("Querying TXT records for: {}", domain);
        let resolver = self.resolver.read().await;
        
        let lookup = resolver
            .txt_lookup(domain)
            .await
            .map_err(|e| DnsLinkError::DnsResolutionFailed(format!("{}: {}", domain, e)))?;

        let mut records = Vec::new();
        for txt in lookup.iter() {
            let data = txt
                .txt_data()
                .iter()
                .map(|bytes| String::from_utf8_lossy(bytes).to_string())
                .collect::<Vec<_>>()
                .join("");

            records.push(TxtRecord {
                name: domain.to_string(),
                ttl: 60,
                data,
            });
        }

        debug!("Found {} TXT records for {}", records.len(), domain);
        Ok(records)
    }

    pub async fn query_cname(&self, _domain: &str) -> Result<Vec<String>, DnsLinkError> {
        Ok(Vec::new())
    }

    pub async fn clear_cache(&self) {
        if self.cache_enabled {
            let resolver = self.resolver.write().await;
            resolver.clear_cache();
        }
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create default DNS resolver")
    }
}
