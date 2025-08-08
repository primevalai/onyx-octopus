//! Observability module for comprehensive monitoring, tracing, and metrics collection.
//!
//! This module provides:
//! - OpenTelemetry integration with distributed tracing
//! - Prometheus metrics export
//! - Structured logging with correlation IDs
//! - Performance monitoring with minimal overhead (<2%)

pub mod telemetry;
pub mod metrics;
pub mod logging;
pub mod correlation;

pub use telemetry::{
    ObservabilityConfig, TelemetryProvider, TracingService, 
    EventTrace, TraceContext, SpanBuilder
};
pub use metrics::{
    MetricsCollector, PrometheusExporter, EventMetrics, 
    PerformanceMetrics, OperationTimer, MetricLabels
};
pub use logging::{
    StructuredLogger, LogLevel, LogContext, CorrelationLogger,
    ObservabilityLogger, LogEntry, LogAggregator
};
pub use correlation::{
    CorrelationId, CorrelationContext, CorrelationTracker,
    RequestContext, TraceCorrelation, generate_correlation_id
};

use crate::error::{EventualiError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main observability service that coordinates all monitoring aspects
#[derive(Debug, Clone)]
pub struct ObservabilityService {
    telemetry: Arc<TelemetryProvider>,
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
    correlation: Arc<CorrelationTracker>,
    config: ObservabilityConfig,
}

impl ObservabilityService {
    /// Create a new observability service with the given configuration
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        let telemetry = Arc::new(TelemetryProvider::new(&config).await?);
        let metrics = Arc::new(MetricsCollector::new(&config)?);
        let logger = Arc::new(StructuredLogger::new(&config)?);
        let correlation = Arc::new(CorrelationTracker::new());

        Ok(Self {
            telemetry,
            metrics,
            logger,
            correlation,
            config,
        })
    }

    /// Initialize observability for the entire system
    pub async fn initialize(&self) -> Result<()> {
        self.telemetry.initialize().await?;
        self.metrics.initialize().await?;
        self.logger.initialize().await?;
        
        tracing::info!("Observability service initialized successfully");
        Ok(())
    }

    /// Create a new trace context for an operation
    pub fn create_trace_context(&self, operation: &str) -> TraceContext {
        let correlation_id = generate_correlation_id();
        self.correlation.register(correlation_id.clone());
        
        TraceContext::new(operation.to_string(), correlation_id)
    }

    /// Start timing an operation
    pub fn start_timer(&self, operation: &str, labels: MetricLabels) -> OperationTimer {
        self.metrics.start_timer(operation, labels)
    }

    /// Log an event with full observability context
    pub fn log_event(&self, level: LogLevel, message: &str, context: &TraceContext) {
        self.logger.log_with_context(level, message, context);
    }

    /// Record metrics for an operation
    pub fn record_metric(&self, name: &str, value: f64, labels: MetricLabels) {
        self.metrics.record_metric(name, value, labels);
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.metrics.get_performance_metrics().await
    }

    /// Shutdown the observability service
    pub async fn shutdown(&self) -> Result<()> {
        self.telemetry.shutdown().await?;
        self.metrics.shutdown().await?;
        self.logger.shutdown().await?;
        
        tracing::info!("Observability service shut down successfully");
        Ok(())
    }
}

/// Builder for creating observability service instances
pub struct ObservabilityServiceBuilder {
    config: ObservabilityConfig,
}

impl ObservabilityServiceBuilder {
    pub fn new() -> Self {
        Self {
            config: ObservabilityConfig::default(),
        }
    }

    pub fn with_config(mut self, config: ObservabilityConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_tracing_enabled(mut self, enabled: bool) -> Self {
        self.config.tracing_enabled = enabled;
        self
    }

    pub fn with_metrics_enabled(mut self, enabled: bool) -> Self {
        self.config.metrics_enabled = enabled;
        self
    }

    pub fn with_structured_logging(mut self, enabled: bool) -> Self {
        self.config.structured_logging = enabled;
        self
    }

    pub async fn build(self) -> Result<ObservabilityService> {
        ObservabilityService::new(self.config).await
    }
}

impl Default for ObservabilityServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}