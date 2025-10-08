use crate::{DnsLinkError, DnsLinkResult, ResolveOptions, MAX_RECURSIVE_DEPTH};
use crate::resolver::DnsResolver;
use crate::namespaces::{parse_ipfs, parse_ipns, extract_dnslink_domain, parse_txt_value};
use async_trait::async_trait;
use async_recursion::async_recursion;
use std::sync::Arc;
use tracing::{debug, error};

#[async_trait]
pub trait DNSLink: Send + Sync {
    async fn resolve(&self, domain: &str) -> Result<DnsLinkResult, DnsLinkError> {
        self.resolve_with_options(domain, ResolveOptions::default()).await
    }

    async fn resolve_with_options(
        &self,
        domain: &str,
        options: ResolveOptions,
    ) -> Result<DnsLinkResult, DnsLinkError>;
}

pub struct DnsLinkImpl {
    resolver: DnsResolver,
}

impl DnsLinkImpl {
    pub fn new(resolver: DnsResolver) -> Self {
        Self { resolver }
    }

    #[async_recursion]
    async fn resolve_domain(
        &self,
        domain: &str,
        depth: u32,
        options: &ResolveOptions,
    ) -> Result<DnsLinkResult, DnsLinkError> {
        if depth == 0 {
            return Err(DnsLinkError::RecursionLimit(
                options.max_recursive_depth.unwrap_or(MAX_RECURSIVE_DEPTH)
            ));
        }

        if options.offline {
            return Err(DnsLinkError::OfflineMode);
        }

        debug!("Resolving domain: {} (depth: {})", domain, depth);

        let prefixed_domain = if domain.starts_with("_dnslink.") {
            domain.to_string()
        } else {
            format!("_dnslink.{}", domain)
        };

        match self.resolve_dnslink(&prefixed_domain, depth, options).await {
            Ok(result) => return Ok(result),
            Err(_) => {
                if domain.starts_with("_dnslink.") {
                    let bare_domain = domain.strip_prefix("_dnslink.").unwrap();
                    return self.resolve_dnslink(bare_domain, depth, options).await;
                } else {
                    return self.resolve_dnslink(domain, depth, options).await;
                }
            }
        }
    }

    #[async_recursion]
    async fn resolve_dnslink(
        &self,
        domain: &str,
        depth: u32,
        options: &ResolveOptions,
    ) -> Result<DnsLinkResult, DnsLinkError> {
        if depth == 0 {
            return Err(DnsLinkError::RecursionLimit(
                options.max_recursive_depth.unwrap_or(MAX_RECURSIVE_DEPTH)
            ));
        }

        if options.nocache {
            self.resolver.clear_cache().await;
        }

        debug!("Querying TXT records for: {}", domain);
        
        let txt_records = self.resolver.query_txt(domain).await?;

        let mut records = txt_records;
        records.sort_by(|a, b| a.data.cmp(&b.data));

        debug!("Found {} TXT records for {}", records.len(), domain);

        for answer in &records {
            debug!("Processing TXT record: {}", answer.data);
            
            let value = match parse_txt_value(&answer.data) {
                Some(v) => v,
                None => continue,
            };

            debug!("Parsed value: {}", value);

            let parts: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                continue;
            }

            let namespace = parts[0];
            debug!("Namespace: {}", namespace);

            match namespace {
                "ipfs" => {
                    match parse_ipfs(&value, answer.clone()) {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            error!("Failed to parse IPFS namespace: {:?}", e);
                            continue;
                        }
                    }
                }
                "ipns" => {
                    match parse_ipns(&value, answer.clone()) {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            error!("Failed to parse IPNS namespace: {:?}", e);
                            continue;
                        }
                    }
                }
                "dnslink" => {
                    match extract_dnslink_domain(&value) {
                        Ok(next_domain) => {
                            debug!("Following recursive dnslink to: {}", next_domain);
                            return self.resolve_domain(&next_domain, depth - 1, options).await;
                        }
                        Err(e) => {
                            error!("Failed to extract dnslink domain: {:?}", e);
                            continue;
                        }
                    }
                }
                _ => {
                    debug!("Unknown namespace '{}', returning as Other", namespace);
                    return Ok(DnsLinkResult::Other {
                        answer: answer.clone(),
                        namespace: namespace.to_string(),
                        value,
                    });
                }
            }
        }

        let cname_records = self.resolver.query_cname(domain).await?;
        let mut cnames = cname_records;
        cnames.sort();

        for cname in cnames {
            match self.resolve_domain(&cname, depth - 1, options).await {
                Ok(result) => return Ok(result),
                Err(_) => continue,
            }
        }

        Err(DnsLinkError::NotFound(domain.to_string()))
    }
}

#[async_trait]
impl DNSLink for DnsLinkImpl {
    async fn resolve_with_options(
        &self,
        domain: &str,
        options: ResolveOptions,
    ) -> Result<DnsLinkResult, DnsLinkError> {
        let max_depth = options.max_recursive_depth.unwrap_or(MAX_RECURSIVE_DEPTH);
        self.resolve_domain(domain, max_depth, &options).await
    }
}

#[derive(Debug, Clone)]
pub struct DnsLinkInit {
    pub use_https: bool,
    pub cache_enabled: bool,
}

impl Default for DnsLinkInit {
    fn default() -> Self {
        Self {
            use_https: true,
            cache_enabled: true,
        }
    }
}

pub fn dns_link(init: DnsLinkInit) -> Result<Arc<dyn DNSLink>, DnsLinkError> {
    let config = if init.use_https {
        hickory_resolver::config::ResolverConfig::google()
    } else {
        hickory_resolver::config::ResolverConfig::default()
    };
    
    let resolver = DnsResolver::with_config(config, init.cache_enabled)?;

    Ok(Arc::new(DnsLinkImpl::new(resolver)))
}
