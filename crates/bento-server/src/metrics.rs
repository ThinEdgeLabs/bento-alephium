use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Prometheus metrics registry and collectors for the Alephium indexer
#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Mutex<Registry>>,
    
    // Health metrics
    pub health_status: Gauge<f64, AtomicU64>,
    
    // Sync status metrics
    pub current_block_height: Gauge<f64, AtomicU64>,
    pub latest_network_block_height: Gauge<f64, AtomicU64>,
    pub blocks_behind_count: Gauge<f64, AtomicU64>,
    pub last_successful_sync_timestamp: Gauge<f64, AtomicU64>,
    
    // Resource usage metrics
    pub memory_usage_bytes: Gauge<f64, AtomicU64>,
    pub cpu_utilization_percent: Gauge<f64, AtomicU64>,
    pub disk_usage_bytes: Gauge<f64, AtomicU64>,
    
    // Request latency histograms
    pub api_endpoint_latency: Histogram,
    pub node_request_latency: Histogram,
    
    // Request counters
    pub api_requests_total: Counter<u64, AtomicU64>,
    pub node_requests_total: Counter<u64, AtomicU64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Create a new metrics instance with all collectors registered
    pub fn new() -> Self {
        let mut registry = Registry::default();
        
        // Health metrics
        let health_status = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_health_status",
            "Health status of the indexer (1 = healthy, 0 = unhealthy)",
            health_status.clone(),
        );
        
        // Sync status metrics
        let current_block_height = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_current_block_height",
            "Current block height indexed by the indexer",
            current_block_height.clone(),
        );
        
        let latest_network_block_height = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_latest_network_block_height",
            "Latest block height available on the Alephium network",
            latest_network_block_height.clone(),
        );
        
        let blocks_behind_count = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_blocks_behind_count",
            "Number of blocks the indexer is behind the network",
            blocks_behind_count.clone(),
        );
        
        let last_successful_sync_timestamp = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_last_successful_sync_timestamp",
            "Unix timestamp of the last successful sync operation",
            last_successful_sync_timestamp.clone(),
        );
        
        // Resource usage metrics
        let memory_usage_bytes = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_memory_usage_bytes",
            "Current memory usage of the indexer in bytes",
            memory_usage_bytes.clone(),
        );
        
        let cpu_utilization_percent = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_cpu_utilization_percent",
            "Current CPU utilization percentage",
            cpu_utilization_percent.clone(),
        );
        
        let disk_usage_bytes = Gauge::<f64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_disk_usage_bytes",
            "Current disk usage for indexed data in bytes",
            disk_usage_bytes.clone(),
        );
        
        // Request latency histograms
        let api_endpoint_latency = Histogram::new(exponential_buckets(0.001, 2.0, 10));
        registry.register(
            "alephium_indexer_api_endpoint_duration_seconds",
            "Histogram of API endpoint response times in seconds",
            api_endpoint_latency.clone(),
        );
        
        let node_request_latency = Histogram::new(exponential_buckets(0.001, 2.0, 10));
        registry.register(
            "alephium_indexer_node_request_duration_seconds",
            "Histogram of Alephium node request latencies in seconds",
            node_request_latency.clone(),
        );
        
        // Request counters
        let api_requests_total = Counter::<u64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_api_requests_total",
            "Total number of API requests received",
            api_requests_total.clone(),
        );
        
        let node_requests_total = Counter::<u64, AtomicU64>::default();
        registry.register(
            "alephium_indexer_node_requests_total",
            "Total number of requests made to Alephium nodes",
            node_requests_total.clone(),
        );
        
        Self {
            registry: Arc::new(Mutex::new(registry)),
            health_status,
            current_block_height,
            latest_network_block_height,
            blocks_behind_count,
            last_successful_sync_timestamp,
            memory_usage_bytes,
            cpu_utilization_percent,
            disk_usage_bytes,
            api_endpoint_latency,
            node_request_latency,
            api_requests_total,
            node_requests_total,
        }
    }
    
    /// Set the health status (1 for healthy, 0 for unhealthy)
    pub fn set_health_status(&self, healthy: bool) {
        self.health_status.set(if healthy { 1.0 } else { 0.0 });
    }
    
    /// Update sync status metrics
    pub fn update_sync_status(&self, current_height: u64, network_height: u64) {
        self.current_block_height.set(current_height as f64);
        self.latest_network_block_height.set(network_height as f64);
        self.blocks_behind_count.set((network_height.saturating_sub(current_height)) as f64);
        
        // Update last successful sync timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;
        self.last_successful_sync_timestamp.set(timestamp);
    }
    
    /// Update resource usage metrics
    pub fn update_resource_usage(&self, memory_bytes: u64, cpu_percent: f64, disk_bytes: u64) {
        self.memory_usage_bytes.set(memory_bytes as f64);
        self.cpu_utilization_percent.set(cpu_percent);
        self.disk_usage_bytes.set(disk_bytes as f64);
    }
    
    /// Record API endpoint latency
    pub fn record_api_latency(&self, duration_seconds: f64) {
        self.api_endpoint_latency.observe(duration_seconds);
        self.api_requests_total.inc();
    }
    
    /// Record node request latency
    pub fn record_node_latency(&self, duration_seconds: f64) {
        self.node_request_latency.observe(duration_seconds);
        self.node_requests_total.inc();
    }
    
    /// Encode metrics in Prometheus format
    pub fn encode_metrics(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let registry = self.registry.lock().map_err(|_| "Failed to lock registry")?;
        let mut buffer = String::new();
        encode(&mut buffer, &registry)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        
        // Test initial values
        assert_eq!(metrics.health_status.get(), 0.0);
        assert_eq!(metrics.current_block_height.get(), 0.0);
        assert_eq!(metrics.latest_network_block_height.get(), 0.0);
        assert_eq!(metrics.blocks_behind_count.get(), 0.0);
    }
    
    #[test]
    fn test_health_status_update() {
        let metrics = Metrics::new();
        
        metrics.set_health_status(true);
        assert_eq!(metrics.health_status.get(), 1.0);
        
        metrics.set_health_status(false);
        assert_eq!(metrics.health_status.get(), 0.0);
    }
    
    #[test]
    fn test_sync_status_update() {
        let metrics = Metrics::new();
        
        metrics.update_sync_status(100, 150);
        
        assert_eq!(metrics.current_block_height.get(), 100.0);
        assert_eq!(metrics.latest_network_block_height.get(), 150.0);
        assert_eq!(metrics.blocks_behind_count.get(), 50.0);
        assert!(metrics.last_successful_sync_timestamp.get() > 0.0);
    }
    
    #[test]
    fn test_resource_usage_update() {
        let metrics = Metrics::new();
        
        metrics.update_resource_usage(1024 * 1024 * 100, 75.5, 1024 * 1024 * 1024);
        
        assert_eq!(metrics.memory_usage_bytes.get(), (1024 * 1024 * 100) as f64);
        assert_eq!(metrics.cpu_utilization_percent.get(), 75.5);
        assert_eq!(metrics.disk_usage_bytes.get(), (1024 * 1024 * 1024) as f64);
    }
    
    #[test]
    fn test_latency_recording() {
        let metrics = Metrics::new();
        
        // Record some API latencies
        metrics.record_api_latency(0.1);
        metrics.record_api_latency(0.2);
        
        // Record some node latencies
        metrics.record_node_latency(0.05);
        metrics.record_node_latency(0.15);
        
        // Check counters were incremented
        assert_eq!(metrics.api_requests_total.get(), 2);
        assert_eq!(metrics.node_requests_total.get(), 2);
    }
    
    #[test]
    fn test_metrics_encoding() {
        let metrics = Metrics::new();
        
        // Set some test values
        metrics.set_health_status(true);
        metrics.update_sync_status(100, 105);
        metrics.record_api_latency(0.1);
        
        let encoded = metrics.encode_metrics().expect("Failed to encode metrics");
        
        // Check that the output contains expected metric names
        assert!(encoded.contains("alephium_indexer_health_status"));
        assert!(encoded.contains("alephium_indexer_current_block_height"));
        assert!(encoded.contains("alephium_indexer_api_requests_total"));
    }
    
    #[test]
    fn test_blocks_behind_calculation() {
        let metrics = Metrics::new();
        
        // Test when we're behind
        metrics.update_sync_status(90, 100);
        assert_eq!(metrics.blocks_behind_count.get(), 10.0);
        
        // Test when we're caught up
        metrics.update_sync_status(100, 100);
        assert_eq!(metrics.blocks_behind_count.get(), 0.0);
        
        // Test edge case where current > network (shouldn't happen but handle gracefully)
        metrics.update_sync_status(105, 100);
        assert_eq!(metrics.blocks_behind_count.get(), 0.0);
    }
}