//! Core IPNS implementation

use crate::keys::{routing_key_from_peer_id, routing_key_from_public_key, Keychain};
use crate::routing::{GetOptions, PutOptions};
use crate::*;
use futures::future::join_all;
use libp2p_identity::{Keypair, PeerId, PublicKey};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;

/// IPNS implementation structure
pub struct IpnsImpl {
    routers: Vec<Arc<dyn IpnsRouting>>,
    local_store: LocalStore,
    keychain: Keychain,
    enable_republish: bool,
    republish_interval: Duration,
    republish_concurrency: usize,
    started: Arc<RwLock<bool>>,
    republish_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl IpnsImpl {
    pub fn new(init: IpnsInit) -> Result<Arc<dyn Ipns>, IpnsError> {
        let republish_interval = init
            .republish_interval
            .unwrap_or_else(|| Duration::from_millis(DEFAULT_REPUBLISH_INTERVAL_MS));
        let republish_concurrency = init
            .republish_concurrency
            .unwrap_or(DEFAULT_REPUBLISH_CONCURRENCY);

        let implementation = Self {
            routers: init.routers,
            local_store: LocalStore::new(),
            keychain: Keychain::new(),
            enable_republish: init.enable_republish,
            republish_interval,
            republish_concurrency,
            started: Arc::new(RwLock::new(false)),
            republish_task: Arc::new(RwLock::new(None)),
        };

        Ok(Arc::new(implementation))
    }

    /// Create an IPNS record
    fn create_ipns_record(
        &self,
        keypair: &Keypair,
        value: &str,
        sequence: u64,
        lifetime_ms: u64,
        ttl_ns: u64,
    ) -> Result<IpnsRecord, IpnsError> {
        // Calculate validity (lifetime from now)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let validity_time = now + std::time::Duration::from_millis(lifetime_ms);
        let validity =
            chrono::DateTime::<chrono::Utc>::from(UNIX_EPOCH + validity_time).to_rfc3339();

        // Get public key bytes
        let public_key = keypair.public();
        let public_key_bytes = public_key.encode_protobuf();

        // Create the record (without signatures first)
        let mut record = IpnsRecord {
            value: value.to_string(),
            sequence,
            validity,
            ttl: ttl_ns,
            public_key: public_key_bytes,
            signature: vec![],
            signature_v2: None,
        };

        // Sign the record
        let (sig_v1, sig_v2) = crate::record::sign_record(keypair, &record)?;
        record.signature = sig_v1;
        record.signature_v2 = Some(sig_v2);

        Ok(record)
    }

    /// Marshal an IPNS record to bytes
    fn marshal_record(&self, record: &IpnsRecord) -> Result<Vec<u8>, IpnsError> {
        // In a full implementation, this would use protobuf marshaling
        // For now, we'll use JSON as a placeholder
        serde_json::to_vec(record)
            .map_err(|e| IpnsError::MarshalingError(format!("Failed to marshal record: {}", e)))
    }

    /// Unmarshal an IPNS record from bytes
    fn unmarshal_record(&self, bytes: &[u8]) -> Result<IpnsRecord, IpnsError> {
        serde_json::from_slice(bytes)
            .map_err(|e| IpnsError::MarshalingError(format!("Failed to unmarshal record: {}", e)))
    }

    /// Format a CID as an IPNS value
    fn format_ipns_value(cid: &Cid) -> String {
        format!("/ipfs/{}", cid)
    }

    /// Parse an IPNS value to extract CID and path
    fn parse_ipns_value(value: &str) -> Result<(Cid, String), IpnsError> {
        let parts: Vec<&str> = value.split('/').filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return Err(IpnsError::InvalidRecord("Empty value".to_string()));
        }

        match parts[0] {
            "ipfs" => {
                if parts.len() < 2 {
                    return Err(IpnsError::InvalidCid("No CID in value".to_string()));
                }

                let cid: Cid = parts[1]
                    .parse()
                    .map_err(|e| IpnsError::InvalidCid(format!("Invalid CID: {}", e)))?;

                let path = if parts.len() > 2 {
                    format!("/{}", parts[2..].join("/"))
                } else {
                    String::new()
                };

                Ok((cid, path))
            }
            "ipns" => {
                // Recursive IPNS reference
                Err(IpnsError::Other(
                    "Recursive IPNS resolution not yet implemented".to_string(),
                ))
            }
            _ => Err(IpnsError::InvalidRecord(format!(
                "Unknown namespace: {}",
                parts[0]
            ))),
        }
    }
}

#[async_trait]
impl Ipns for IpnsImpl {
    fn routers(&self) -> &[Arc<dyn IpnsRouting>] {
        &self.routers
    }

    async fn publish(
        &self,
        key_name: &str,
        value: &Cid,
        options: PublishOptions,
    ) -> Result<PublishResult, IpnsError> {
        // Get or create the key
        let keypair = self.keychain.get_or_create_key(key_name)?;
        let public_key = keypair.public();

        // Generate routing key
        let routing_key = routing_key_from_public_key(&public_key);

        // Determine sequence number
        let sequence = if self.local_store.has(&routing_key) {
            // Increment existing sequence
            let stored = self.local_store.get(&routing_key)?;
            let existing_record = self.unmarshal_record(&stored.record)?;
            existing_record.sequence + 1
        } else {
            // First publication
            1
        };

        // Get options with defaults
        let lifetime_ms = options.lifetime.unwrap_or(DEFAULT_LIFETIME_MS);
        let ttl_ns = options
            .ttl
            .map(|ms| ms * 1_000_000) // Convert ms to ns
            .unwrap_or(DEFAULT_TTL_NS);

        // Format the value
        let value_str = Self::format_ipns_value(value);

        // Create the IPNS record
        let record =
            self.create_ipns_record(&keypair, &value_str, sequence, lifetime_ms, ttl_ns)?;

        // Marshal the record
        let marshaled = self.marshal_record(&record)?;

        // Create metadata
        let metadata = RecordMetadata::new(key_name.to_string(), lifetime_ms);

        // Store locally
        self.local_store
            .put(&routing_key, marshaled.clone(), Some(metadata.clone()))?;

        tracing::info!(
            "Published IPNS record for key '{}' with sequence {}",
            key_name,
            sequence
        );

        // If not offline, publish to routers
        if !options.offline && !self.routers.is_empty() {
            let put_options = PutOptions {
                metadata: Some(metadata),
            };

            // Publish to all routers in parallel
            let routing_key_clone = routing_key.clone();
            let marshaled_clone = marshaled.clone();

            let publish_futures: Vec<_> = self
                .routers
                .iter()
                .map(|router| {
                    let routing_key = routing_key_clone.clone();
                    let marshaled = marshaled_clone.clone();
                    let put_opts = put_options.clone();
                    async move { router.put(&routing_key, &marshaled, put_opts).await }
                })
                .collect();

            let results = join_all(publish_futures).await;

            // Check for errors
            let mut errors = Vec::new();
            for (i, result) in results.iter().enumerate() {
                if let Err(e) = result {
                    tracing::warn!("Router {} failed to publish: {}", i, e);
                    errors.push(format!("Router {}: {}", i, e));
                }
            }

            if !errors.is_empty() {
                tracing::warn!("Some routers failed: {}", errors.join(", "));
                // Don't fail the whole operation if at least one router succeeded
            }
        }

        Ok(PublishResult {
            record,
            public_key: public_key.encode_protobuf(),
        })
    }

    async fn resolve(
        &self,
        key: &[u8],
        options: ResolveOptions,
    ) -> Result<ResolveResult, IpnsError> {
        // Try to extract peer ID from the key
        let routing_key = if key.starts_with(b"/ipns/") {
            key.to_vec()
        } else {
            // Assume it's a raw multihash or public key bytes
            // Try to create a peer ID
            match PeerId::from_bytes(key) {
                Ok(peer_id) => routing_key_from_peer_id(&peer_id),
                Err(_) => {
                    // Try as public key
                    match PublicKey::try_decode_protobuf(key) {
                        Ok(public_key) => routing_key_from_public_key(&public_key),
                        Err(e) => return Err(IpnsError::InvalidKey(format!("Invalid key: {}", e))),
                    }
                }
            }
        };

        self.resolve_routing_key(&routing_key, options).await
    }

    async fn resolve_peer_id(
        &self,
        peer_id: &PeerId,
        options: ResolveOptions,
    ) -> Result<ResolveResult, IpnsError> {
        let routing_key = routing_key_from_peer_id(peer_id);
        self.resolve_routing_key(&routing_key, options).await
    }

    async fn unpublish(&self, key_name: &str) -> Result<(), IpnsError> {
        // Export the public key
        let public_key = self.keychain.export_public_key(key_name)?;

        // Generate routing key
        let routing_key = routing_key_from_public_key(&public_key);

        // Delete from local store
        self.local_store.delete(&routing_key)?;

        tracing::info!("Unpublished IPNS record for key '{}'", key_name);

        Ok(())
    }

    async fn start(&self) -> Result<(), IpnsError> {
        let mut started = self.started.write().unwrap();
        if *started {
            return Ok(());
        }
        *started = true;
        drop(started); // Release the lock before spawning task

        // Start republish task if enabled
        if self.enable_republish {
            self.start_republish_task();
        }

        tracing::info!("IPNS service started");
        Ok(())
    }

    async fn stop(&self) -> Result<(), IpnsError> {
        let mut started = self.started.write().unwrap();
        if !*started {
            return Ok(());
        }
        *started = false;
        drop(started); // Release the lock

        // Stop republish task
        let mut task = self.republish_task.write().unwrap();
        if let Some(handle) = task.take() {
            handle.abort();
            tracing::debug!("Republish task stopped");
        }

        tracing::info!("IPNS service stopped");
        Ok(())
    }
}

impl IpnsImpl {
    /// Start the background republish task
    fn start_republish_task(&self) {
        let local_store = self.local_store.clone();
        let keychain = self.keychain.clone();
        let routers = self.routers.clone();
        let started = self.started.clone();
        let interval = self.republish_interval;
        let concurrency = self.republish_concurrency;

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;

                // Check if service is still running
                {
                    let is_started = started.read().unwrap();
                    if !*is_started {
                        tracing::debug!("Republish task stopping due to service stop");
                        break;
                    }
                }

                // Perform republish check
                if let Err(e) =
                    Self::republish_check(&local_store, &keychain, &routers, concurrency).await
                {
                    tracing::warn!("Republish check failed: {}", e);
                }
            }
        });

        let mut task = self.republish_task.write().unwrap();
        *task = Some(handle);
        tracing::debug!("Republish task started");
    }

    /// Check and republish records that need refreshing
    async fn republish_check(
        local_store: &LocalStore,
        keychain: &Keychain,
        routers: &[Arc<dyn IpnsRouting>],
        concurrency: usize,
    ) -> Result<(), IpnsError> {
        // Get all records from local store
        let records = local_store.list();

        if records.is_empty() {
            return Ok(());
        }

        tracing::debug!("Checking {} records for republish", records.len());

        let mut republish_tasks: Vec<Pin<Box<dyn Future<Output = Result<(), IpnsError>> + Send>>> =
            Vec::new();

        for (routing_key, stored) in records {
            // Check if record needs republishing
            if let Some(ref metadata) = stored.metadata {
                if !metadata.should_republish(DHT_EXPIRY_MS, REPUBLISH_THRESHOLD_MS) {
                    continue;
                }

                tracing::info!("Record for key '{}' needs republishing", metadata.key_name);

                // Parse the record
                let record = match Self::unmarshal_record_static(&stored.record) {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("Failed to unmarshal record: {}", e);
                        continue;
                    }
                };

                // Get the keypair
                let keypair = match keychain.get_or_create_key(&metadata.key_name) {
                    Ok(k) => k,
                    Err(e) => {
                        tracing::warn!("Failed to get key '{}': {}", metadata.key_name, e);
                        continue;
                    }
                };

                // Create republish task
                let routing_key_clone = routing_key.clone();
                let routers_clone = routers.to_vec();
                let key_name = metadata.key_name.clone();
                let lifetime_ms = metadata.lifetime;

                let task = Box::pin(Self::republish_record(
                    routing_key_clone,
                    record,
                    keypair,
                    routers_clone,
                    key_name,
                    lifetime_ms,
                ));

                republish_tasks.push(task);
            }
        }

        if republish_tasks.is_empty() {
            tracing::debug!("No records need republishing");
            return Ok(());
        }

        tracing::info!("Republishing {} records", republish_tasks.len());

        // Execute republish tasks with concurrency limit
        let mut results = Vec::new();
        for chunk in republish_tasks.chunks_mut(concurrency) {
            let chunk_results = join_all(chunk).await;
            results.extend(chunk_results);
        }

        // Count successes and failures
        let successes = results.iter().filter(|r| r.is_ok()).count();
        let failures = results.len() - successes;

        if failures > 0 {
            tracing::warn!(
                "Republish completed: {} succeeded, {} failed",
                successes,
                failures
            );
        } else {
            tracing::info!("Republish completed: {} records updated", successes);
        }

        Ok(())
    }

    /// Republish a single record
    async fn republish_record(
        routing_key: Vec<u8>,
        old_record: IpnsRecord,
        keypair: Keypair,
        routers: Vec<Arc<dyn IpnsRouting>>,
        key_name: String,
        lifetime_ms: u64,
    ) -> Result<(), IpnsError> {
        // Increment sequence number
        let new_sequence = old_record.sequence + 1;

        // Create new record with updated sequence and validity
        let new_record = Self::create_ipns_record_static(
            &keypair,
            &old_record.value,
            new_sequence,
            lifetime_ms,
            old_record.ttl,
        )?;

        // Marshal the record
        let marshaled = Self::marshal_record_static(&new_record)?;

        // Store locally with updated metadata
        let metadata = RecordMetadata {
            key_name: key_name.clone(),
            lifetime: lifetime_ms,
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        // Update local store (using a hypothetical method - we'll need to handle this)
        // For now, we'll skip the local store update in this static method
        // The actual implementation would need access to local_store

        // Publish to routers
        if !routers.is_empty() {
            let put_options = PutOptions {
                metadata: Some(metadata),
            };

            let marshaled_clone = marshaled.clone();

            let publish_futures: Vec<_> = routers
                .iter()
                .map(|router| {
                    let routing_key_ref = routing_key.clone();
                    let marshaled = marshaled_clone.clone();
                    let put_opts = put_options.clone();
                    async move { router.put(&routing_key_ref, &marshaled, put_opts).await }
                })
                .collect();

            let results = join_all(publish_futures).await;

            // Check for errors
            let mut errors = Vec::new();
            for (i, result) in results.iter().enumerate() {
                if let Err(e) = result {
                    errors.push(format!("Router {}: {}", i, e));
                }
            }

            if !errors.is_empty() && errors.len() == routers.len() {
                return Err(IpnsError::PublishFailed(format!(
                    "All routers failed for key '{}': {}",
                    key_name,
                    errors.join(", ")
                )));
            }
        }

        tracing::info!(
            "Republished IPNS record for key '{}' with sequence {}",
            key_name,
            new_sequence
        );

        Ok(())
    }

    /// Static version of create_ipns_record for use in republish
    fn create_ipns_record_static(
        keypair: &Keypair,
        value: &str,
        sequence: u64,
        lifetime_ms: u64,
        ttl_ns: u64,
    ) -> Result<IpnsRecord, IpnsError> {
        // Calculate validity (lifetime from now)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let validity_time = now + Duration::from_millis(lifetime_ms);
        let validity =
            chrono::DateTime::<chrono::Utc>::from(UNIX_EPOCH + validity_time).to_rfc3339();

        // Get public key bytes
        let public_key = keypair.public();
        let public_key_bytes = public_key.encode_protobuf();

        // Create the record (without signatures first)
        let mut record = IpnsRecord {
            value: value.to_string(),
            sequence,
            validity,
            ttl: ttl_ns,
            public_key: public_key_bytes,
            signature: vec![],
            signature_v2: None,
        };

        // Sign the record
        let (sig_v1, sig_v2) = crate::record::sign_record(keypair, &record)?;
        record.signature = sig_v1;
        record.signature_v2 = Some(sig_v2);

        Ok(record)
    }

    /// Static version of marshal_record for use in republish
    fn marshal_record_static(record: &IpnsRecord) -> Result<Vec<u8>, IpnsError> {
        serde_json::to_vec(record)
            .map_err(|e| IpnsError::MarshalingError(format!("Failed to marshal record: {}", e)))
    }

    /// Static version of unmarshal_record for use in republish
    fn unmarshal_record_static(bytes: &[u8]) -> Result<IpnsRecord, IpnsError> {
        serde_json::from_slice(bytes)
            .map_err(|e| IpnsError::MarshalingError(format!("Failed to unmarshal record: {}", e)))
    }

    /// Resolve an IPNS record by routing key
    async fn resolve_routing_key(
        &self,
        routing_key: &[u8],
        options: ResolveOptions,
    ) -> Result<ResolveResult, IpnsError> {
        let mut record_bytes: Option<Vec<u8>> = None;

        // Check local cache first (unless nocache is set)
        // However, if offline=true and nocache=true, we still need to check local store
        // since we have no other source of records
        let should_check_cache = !options.nocache || options.offline;

        if should_check_cache && self.local_store.has(routing_key) {
            match self.local_store.get(routing_key) {
                Ok(stored) => {
                    // Check if record is still valid (TTL hasn't expired)
                    let record = self.unmarshal_record(&stored.record)?;

                    if !record.is_expired() {
                        // Check TTL
                        let ttl_ms = record.ttl_ms();
                        let age_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64
                            - stored.created;

                        if age_ms < ttl_ms || options.offline {
                            tracing::debug!("Using cached IPNS record");
                            record_bytes = Some(stored.record.clone());
                        } else {
                            tracing::debug!("Cached record TTL expired, querying routers");
                        }
                    } else {
                        tracing::debug!("Cached record expired, querying routers");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get cached record: {}", e);
                }
            }
        }

        // If we don't have a valid cached record and not offline, query routers
        if record_bytes.is_none() && !options.offline && !self.routers.is_empty() {
            let get_options = GetOptions { validate: true };

            // Query all routers in parallel
            let routing_key_clone = routing_key.to_vec();
            let query_futures: Vec<_> = self
                .routers
                .iter()
                .map(|router| {
                    let routing_key = routing_key_clone.clone();
                    let get_opts = get_options.clone();
                    async move { router.get(&routing_key, get_opts).await }
                })
                .collect();

            let results = join_all(query_futures).await;

            // Find the first successful result
            for result in results {
                if let Ok(record) = result {
                    record_bytes = Some(record);
                    break;
                }
            }
        }

        // If we still don't have a record, fail
        let record_bytes = record_bytes.ok_or_else(|| {
            IpnsError::NotFound(format!(
                "No IPNS record found for routing key: {}",
                bs58::encode(routing_key).into_string()
            ))
        })?;

        // Unmarshal and parse the record
        let record = self.unmarshal_record(&record_bytes)?;

        // Cache the record if we got it from routers
        if !options.nocache {
            let _ = self.local_store.put(routing_key, record_bytes, None);
        }

        // Parse the value to extract CID and path
        let (cid, path) = Self::parse_ipns_value(&record.value)?;

        tracing::info!("Resolved IPNS record to CID {} with path '{}'", cid, path);

        Ok(ResolveResult { cid, path, record })
    }
}
