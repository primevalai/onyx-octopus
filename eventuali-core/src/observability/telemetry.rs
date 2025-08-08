//! OpenTelemetry integration module for distributed tracing
//!
//! Provides comprehensive tracing capabilities with minimal performance overhead.

use crate::error::Result;
use crate::observability::correlation::{CorrelationId, generate_correlation_id};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for observability features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing_enabled: bool,
    pub metrics_enabled: bool,
    pub structured_logging: bool,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: Option<String>,
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub sample_rate: f64,
    pub max_events_per_span: u32,
    pub export_timeout_millis: u64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            tracing_enabled: true,
            metrics_enabled: true,
            structured_logging: true,
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            prometheus_endpoint: Some("http://localhost:9090".to_string()),
            service_name: "eventuali".to_string(),
            service_version: "0.1.0".to_string(),
            environment: "development".to_string(),
            sample_rate: 1.0, // Sample all traces in development
            max_events_per_span: 128,
            export_timeout_millis: 30000,
        }
    }
}

/// Main telemetry provider for OpenTelemetry integration
#[derive(Debug)]
pub struct TelemetryProvider {
    config: ObservabilityConfig,
    active_traces: Arc<RwLock<HashMap<CorrelationId, TraceContext>>>,
}

impl TelemetryProvider {
    /// Create a new telemetry provider
    pub async fn new(config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            active_traces: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize the telemetry provider
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!(
            service_name = %self.config.service_name,
            service_version = %self.config.service_version,
            environment = %self.config.environment,
            "Telemetry provider initialized"
        );

        Ok(())
    }

    /// Create a new trace context
    pub async fn create_trace(&self, operation: &str, _parent: Option<&TraceContext>) -> TraceContext {
        let correlation_id = generate_correlation_id();

        let trace_context = TraceContext::new(operation.to_string(), correlation_id.clone());

        // Store the active trace
        self.active_traces.write().await.insert(correlation_id, trace_context.clone());

        trace_context
    }

    /// Get an active trace by correlation ID
    pub async fn get_trace(&self, correlation_id: &CorrelationId) -> Option<TraceContext> {
        self.active_traces.read().await.get(correlation_id).cloned()
    }

    /// End a trace and clean up
    pub async fn end_trace(&self, trace: &TraceContext) {
        self.active_traces.write().await.remove(&trace.correlation_id);
        
        let duration = trace.start_time.elapsed();
        tracing::debug!(
            operation = %trace.operation,
            correlation_id = %trace.correlation_id,
            duration_ms = duration.as_millis(),
            "Trace completed"
        );
    }

    /// Shutdown the telemetry provider
    pub async fn shutdown(&self) -> Result<()> {
        // Clear all active traces
        self.active_traces.write().await.clear();

        tracing::info!("Telemetry provider shut down successfully");
        Ok(())
    }
}

/// Represents a trace context for an operation
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub operation: String,
    pub correlation_id: CorrelationId,
    pub start_time: std::time::Instant,
    pub attributes: HashMap<String, String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

impl TraceContext {
    pub fn new(operation: String, correlation_id: CorrelationId) -> Self {
        Self {
            operation,
            correlation_id,
            start_time: std::time::Instant::now(),
            attributes: HashMap::new(),
            trace_id: Some(uuid::Uuid::new_v4().to_string()),
            span_id: Some(uuid::Uuid::new_v4().to_string()),
        }
    }

    /// Add an attribute to this trace
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
        
        tracing::debug!(
            trace_id = ?self.trace_id,
            span_id = ?self.span_id,
            correlation_id = %self.correlation_id,
            key = key,
            value = value,
            "Added trace attribute"
        );
    }

    /// Add an event to this trace
    pub fn add_event(&self, name: &str, attributes: HashMap<String, String>) {
        tracing::info!(
            trace_id = ?self.trace_id,
            span_id = ?self.span_id,
            correlation_id = %self.correlation_id,
            event_name = name,
            ?attributes,
            "Trace event"
        );
    }

    /// Record an error in this trace
    pub fn record_error(&self, error: &dyn std::error::Error) {
        tracing::error!(
            trace_id = ?self.trace_id,
            span_id = ?self.span_id,
            correlation_id = %self.correlation_id,
            error = %error,
            "Trace error recorded"
        );
    }

    /// Get elapsed time for this trace
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Service for managing distributed tracing
#[derive(Debug)]
pub struct TracingService {
    provider: Arc<TelemetryProvider>,
}

impl TracingService {
    pub fn new(provider: Arc<TelemetryProvider>) -> Self {
        Self { provider }
    }

    /// Start a new trace for an operation
    pub async fn start_trace(&self, operation: &str) -> TraceContext {
        self.provider.create_trace(operation, None).await
    }

    /// Start a child trace
    pub async fn start_child_trace(&self, operation: &str, parent: &TraceContext) -> TraceContext {
        self.provider.create_trace(operation, Some(parent)).await
    }

    /// End a trace
    pub async fn end_trace(&self, trace: TraceContext) {
        self.provider.end_trace(&trace).await;
    }
}

/// Represents an event trace with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrace {
    pub event_id: Uuid,
    pub trace_id: String,
    pub span_id: String,
    pub operation: String,
    pub correlation_id: CorrelationId,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ns: Option<u64>,
    pub attributes: HashMap<String, String>,
    pub status: TraceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    Ok,
    Error(String),
    Cancelled,
}

/// Builder for creating spans with fluent API
pub struct SpanBuilder {
    operation: String,
    attributes: HashMap<String, String>,
    parent: Option<TraceContext>,
}

impl SpanBuilder {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            attributes: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_parent(mut self, parent: TraceContext) -> Self {
        self.parent = Some(parent);
        self
    }

    pub async fn start(self, provider: &TelemetryProvider) -> TraceContext {
        let mut trace = provider.create_trace(&self.operation, self.parent.as_ref()).await;
        
        for (key, value) in self.attributes {
            trace.add_attribute(&key, &value);
        }

        trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_provider_creation() {
        let config = ObservabilityConfig::default();
        let provider = TelemetryProvider::new(&config).await.unwrap();
        assert_eq!(provider.config.service_name, "eventuali");
    }

    #[tokio::test]
    async fn test_trace_context_creation() {
        let config = ObservabilityConfig {
            jaeger_endpoint: None, // Disable export for testing
            ..ObservabilityConfig::default()
        };
        let provider = TelemetryProvider::new(&config).await.unwrap();
        
        let trace = provider.create_trace("test_operation", None).await;
        assert_eq!(trace.operation, "test_operation");
        assert!(!trace.correlation_id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_trace_attributes() {
        let config = ObservabilityConfig {
            jaeger_endpoint: None,
            ..ObservabilityConfig::default()
        };
        let provider = TelemetryProvider::new(&config).await.unwrap();
        
        let mut trace = provider.create_trace("test_operation", None).await;
        trace.add_attribute("test_key", "test_value");
        
        assert_eq!(trace.attributes.get("test_key"), Some(&"test_value".to_string()));
    }
}