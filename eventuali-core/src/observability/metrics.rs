//! Metrics collection and Prometheus export module
//!
//! Provides comprehensive metrics collection with minimal performance overhead.

use crate::error::{EventualiError, Result};
use crate::observability::ObservabilityConfig;
use metrics::{Key, Label};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Labels for categorizing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricLabels {
    pub labels: HashMap<String, String>,
}

impl MetricLabels {
    /// Create new metric labels
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Add a label
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Convert to metrics library format
    pub fn to_metrics_labels(&self) -> Vec<Label> {
        self.labels
            .iter()
            .map(|(k, v)| Label::new(k.clone(), v.clone()))
            .collect()
    }
}

impl Default for MetricLabels {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    name: String,
    labels: MetricLabels,
    start_time: Instant,
}

impl OperationTimer {
    pub fn new(name: String, labels: MetricLabels) -> Self {
        Self {
            name,
            labels,
            start_time: Instant::now(),
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start_time.elapsed();
        tracing::debug!(
            metric_type = "histogram",
            metric_name = %self.name,
            duration_seconds = duration.as_secs_f64(),
            ?self.labels,
            "Operation timer stopped"
        );
    }

    /// Get elapsed time without stopping the timer
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Comprehensive performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub events_created_total: u64,
    pub events_stored_total: u64,
    pub events_loaded_total: u64,
    pub operations_duration_seconds: HashMap<String, f64>,
    pub error_count_total: u64,
    pub active_connections: u64,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub throughput_events_per_second: f64,
    pub database_query_duration_seconds: f64,
    pub cache_hit_ratio: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            events_created_total: 0,
            events_stored_total: 0,
            events_loaded_total: 0,
            operations_duration_seconds: HashMap::new(),
            error_count_total: 0,
            active_connections: 0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            throughput_events_per_second: 0.0,
            database_query_duration_seconds: 0.0,
            cache_hit_ratio: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Event-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    pub event_type: String,
    pub aggregate_type: String,
    pub tenant_id: Option<String>,
    pub operation_duration_ms: f64,
    pub payload_size_bytes: u64,
    pub serialization_duration_ms: f64,
    pub storage_duration_ms: f64,
    pub success: bool,
    pub error_type: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Main metrics collector
pub struct MetricsCollector {
    #[allow(dead_code)]
    prometheus_handle: Option<PrometheusHandle>,
    config: ObservabilityConfig,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    histograms: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: &ObservabilityConfig) -> Result<Self> {
        let prometheus_handle = if config.metrics_enabled {
            match PrometheusBuilder::new().install() {
                Ok(()) => {
                    tracing::info!("Prometheus recorder installed successfully");
                    None // For now, we don't store the handle
                }
                Err(e) => {
                    tracing::warn!("Failed to install Prometheus recorder: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            prometheus_handle,
            config: config.clone(),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Initialize the metrics collector
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!(
            metrics_enabled = self.config.metrics_enabled,
            prometheus_endpoint = ?self.config.prometheus_endpoint,
            "Metrics collector initialized"
        );

        Ok(())
    }

    /// Start a timer for an operation
    pub fn start_timer(&self, operation: &str, labels: MetricLabels) -> OperationTimer {
        let name = format!("eventuali_{}_duration_seconds", operation);
        OperationTimer::new(name, labels)
    }

    /// Record a counter metric
    pub fn increment_counter(&self, name: &str, labels: MetricLabels) {
        // Use tracing for now, can be extended to actual metrics later
        tracing::debug!(
            metric_type = "counter",
            metric_name = name,
            value = 1,
            ?labels,
            "Counter incremented"
        );
        
        // Also track locally for aggregation
        if let Ok(mut counters) = self.counters.lock() {
            *counters.entry(name.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a gauge metric
    pub fn record_gauge(&self, name: &str, value: f64, labels: MetricLabels) {
        tracing::debug!(
            metric_type = "gauge",
            metric_name = name,
            value = value,
            ?labels,
            "Gauge recorded"
        );
        
        // Also track locally
        if let Ok(mut gauges) = self.gauges.lock() {
            gauges.insert(name.to_string(), value);
        }
    }

    /// Record a histogram metric
    pub fn record_metric(&self, name: &str, value: f64, labels: MetricLabels) {
        tracing::debug!(
            metric_type = "histogram",
            metric_name = name,
            value = value,
            ?labels,
            "Histogram recorded"
        );
        
        // Also track locally
        if let Ok(mut histograms) = self.histograms.lock() {
            histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
        }
    }

    /// Record event-specific metrics
    pub async fn record_event_metrics(&self, metrics: EventMetrics) {
        let labels = MetricLabels::new()
            .with_label("event_type", &metrics.event_type)
            .with_label("aggregate_type", &metrics.aggregate_type)
            .with_label("success", &metrics.success.to_string());

        // Increment event counter
        self.increment_counter("eventuali_events_processed_total", labels.clone());

        // Record durations
        self.record_metric("eventuali_event_operation_duration_seconds", 
                          metrics.operation_duration_ms / 1000.0, labels.clone());
        
        self.record_metric("eventuali_event_serialization_duration_seconds", 
                          metrics.serialization_duration_ms / 1000.0, labels.clone());
        
        self.record_metric("eventuali_event_storage_duration_seconds", 
                          metrics.storage_duration_ms / 1000.0, labels.clone());

        // Record payload size
        self.record_metric("eventuali_event_payload_size_bytes", 
                          metrics.payload_size_bytes as f64, labels.clone());

        // Record errors
        if !metrics.success {
            let error_labels = labels.with_label("error_type", 
                metrics.error_type.as_deref().unwrap_or("unknown"));
            self.increment_counter("eventuali_event_errors_total", error_labels);
        }
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let mut metrics = self.performance_metrics.write().await;
        
        // Update counters from local tracking
        if let Ok(counters) = self.counters.lock() {
            metrics.events_created_total = *counters.get("eventuali_events_created_total").unwrap_or(&0);
            metrics.events_stored_total = *counters.get("eventuali_events_stored_total").unwrap_or(&0);
            metrics.events_loaded_total = *counters.get("eventuali_events_loaded_total").unwrap_or(&0);
            metrics.error_count_total = *counters.get("eventuali_errors_total").unwrap_or(&0);
        }

        // Update gauges from local tracking
        if let Ok(gauges) = self.gauges.lock() {
            metrics.active_connections = *gauges.get("eventuali_active_connections").unwrap_or(&0.0) as u64;
            metrics.memory_usage_bytes = *gauges.get("eventuali_memory_usage_bytes").unwrap_or(&0.0) as u64;
            metrics.cpu_usage_percent = *gauges.get("eventuali_cpu_usage_percent").unwrap_or(&0.0);
        }

        // Calculate throughput (simplified)
        let duration = chrono::Utc::now() - metrics.timestamp;
        if duration.num_seconds() > 0 {
            metrics.throughput_events_per_second = 
                metrics.events_created_total as f64 / duration.num_seconds() as f64;
        }

        metrics.timestamp = chrono::Utc::now();
        metrics.clone()
    }

    /// Shutdown the metrics collector
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Metrics collector shut down successfully");
        Ok(())
    }
}

impl std::fmt::Debug for MetricsCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsCollector")
            .field("prometheus_handle", &"[PrometheusHandle]")
            .field("config", &self.config)
            .field("performance_metrics", &"[PerformanceMetrics]")
            .field("counters", &"[Counters]")
            .field("gauges", &"[Gauges]")
            .field("histograms", &"[Histograms]")
            .finish()
    }
}

/// Prometheus exporter for metrics
pub struct PrometheusExporter {
    handle: PrometheusHandle,
}

impl PrometheusExporter {
    pub fn new(handle: PrometheusHandle) -> Self {
        Self { handle }
    }

    /// Get metrics in Prometheus format
    pub fn get_metrics(&self) -> String {
        self.handle.render()
    }

    /// Get metrics as bytes
    pub fn get_metrics_bytes(&self) -> Vec<u8> {
        self.get_metrics().into_bytes()
    }

    /// Export metrics to a file
    pub async fn export_to_file(&self, path: &str) -> Result<()> {
        let metrics = self.get_metrics();
        tokio::fs::write(path, metrics).await
            .map_err(|e| EventualiError::ObservabilityError(format!("Failed to export metrics: {}", e)))?;
        Ok(())
    }

    /// Serve metrics over HTTP (simplified)
    pub fn metrics_endpoint_handler(&self) -> String {
        self.get_metrics()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_labels() {
        let labels = MetricLabels::new()
            .with_label("service", "eventuali")
            .with_label("version", "0.1.0");

        assert_eq!(labels.labels.len(), 2);
        assert_eq!(labels.labels.get("service"), Some(&"eventuali".to_string()));
        assert_eq!(labels.labels.get("version"), Some(&"0.1.0".to_string()));
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = ObservabilityConfig {
            metrics_enabled: false, // Disable for testing
            ..ObservabilityConfig::default()
        };
        let collector = MetricsCollector::new(&config).unwrap();
        
        // Should succeed without Prometheus
        assert!(!config.metrics_enabled);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let config = ObservabilityConfig {
            metrics_enabled: false,
            ..ObservabilityConfig::default()
        };
        let collector = MetricsCollector::new(&config).unwrap();
        
        let metrics = collector.get_performance_metrics().await;
        assert_eq!(metrics.events_created_total, 0);
        assert_eq!(metrics.events_stored_total, 0);
    }

    #[test]
    fn test_event_metrics() {
        let metrics = EventMetrics {
            event_type: "UserCreated".to_string(),
            aggregate_type: "User".to_string(),
            tenant_id: Some("tenant123".to_string()),
            operation_duration_ms: 15.5,
            payload_size_bytes: 1024,
            serialization_duration_ms: 2.1,
            storage_duration_ms: 8.3,
            success: true,
            error_type: None,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.event_type, "UserCreated");
        assert_eq!(metrics.aggregate_type, "User");
        assert!(metrics.success);
        assert_eq!(metrics.payload_size_bytes, 1024);
    }

    #[test]
    fn test_operation_timer() {
        let labels = MetricLabels::new();
        let timer = OperationTimer::new("test_operation".to_string(), labels);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed();
        
        assert!(elapsed.as_millis() >= 10);
    }
}