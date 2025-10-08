//! Metrics implementations

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;

use helia_interface::Metrics;

/// Simple in-memory metrics implementation
pub struct SimpleMetrics {
    counters: Arc<DashMap<String, u64>>,
    gauges: Arc<DashMap<String, f64>>,
    histograms: Arc<DashMap<String, Vec<f64>>>,
}

impl SimpleMetrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            histograms: Arc::new(DashMap::new()),
        }
    }

    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).map(|v| *v)
    }

    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).map(|v| *v)
    }

    pub fn get_histogram(&self, name: &str) -> Option<Vec<f64>> {
        self.histograms.get(name).map(|v| v.clone())
    }
}

impl Default for SimpleMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Metrics for SimpleMetrics {
    async fn record_counter(&self, name: &str, value: u64, _labels: HashMap<String, String>) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    async fn record_gauge(&self, name: &str, value: f64, _labels: HashMap<String, String>) {
        self.gauges.insert(name.to_string(), value);
    }

    async fn record_histogram(&self, name: &str, value: f64, _labels: HashMap<String, String>) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }
}