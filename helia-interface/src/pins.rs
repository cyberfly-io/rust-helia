//! Pinning interface for content addressing

use std::collections::HashMap;

use async_trait::async_trait;
use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{AbortOptions, AwaitIterable, GetBlockProgressEvents, HeliaError, ProgressOptions};

/// Types of pins
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PinType {
    /// Pin recursively through all child links
    Recursive,
    /// Pin only this specific block
    Direct,
    /// Pinned indirectly through another pin
    Indirect,
}

/// A pin represents a pinned content identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    /// The content identifier
    pub cid: Cid,
    /// The depth of the pin (0 for direct, positive for recursive)
    pub depth: u64,
    /// User-defined metadata associated with the pin
    pub metadata: HashMap<String, PinMetadataValue>,
}

/// Valid metadata values for pins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PinMetadataValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

/// Progress events for adding pins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AddPinEvents {
    #[serde(rename = "helia:pin:add")]
    Add { cid: Cid },
}

/// Options for adding pins
#[derive(Debug, Default)]
pub struct AddOptions {
    /// Abort options
    pub abort: AbortOptions,
    /// Progress options
    pub progress: ProgressOptions<AddPinProgressEvents>,
    /// How deeply to pin the DAG, None means infinite depth
    pub depth: Option<u64>,
    /// Optional user-defined metadata to store with the pin
    pub metadata: HashMap<String, PinMetadataValue>,
}

impl Clone for AddOptions {
    fn clone(&self) -> Self {
        Self {
            abort: self.abort.clone(),
            progress: self.progress.clone(),
            depth: self.depth,
            metadata: self.metadata.clone(),
        }
    }
}

/// Combined progress events for adding pins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddPinProgressEvents {
    Add(AddPinEvents),
    GetBlock(GetBlockProgressEvents),
}

/// Options for removing pins
#[derive(Debug, Clone, Default)]
pub struct RmOptions {
    /// Abort options
    pub abort: AbortOptions,
}

/// Options for listing pins
#[derive(Debug, Clone, Default)]
pub struct LsOptions {
    /// Abort options
    pub abort: AbortOptions,
    /// Optional CID to filter pins by
    pub cid: Option<Cid>,
}

/// Options for checking if content is pinned
#[derive(Debug, Clone, Default)]
pub struct IsPinnedOptions {
    /// Abort options
    pub abort: AbortOptions,
}

/// Pinning interface
#[async_trait]
pub trait Pins: Send + Sync {
    /// Pin a CID with the given options
    async fn add(&self, cid: &Cid, options: Option<AddOptions>) -> Result<(), HeliaError>;

    /// Remove a pin for the given CID
    async fn rm(&self, cid: &Cid, options: Option<RmOptions>) -> Result<(), HeliaError>;

    /// List all pins, optionally filtered by CID
    async fn ls(&self, options: Option<LsOptions>) -> Result<AwaitIterable<Pin>, HeliaError>;

    /// Check if a CID is pinned
    async fn is_pinned(
        &self,
        cid: &Cid,
        options: Option<IsPinnedOptions>,
    ) -> Result<bool, HeliaError>;
}
