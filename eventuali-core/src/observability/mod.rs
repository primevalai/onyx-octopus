//! Observability module for comprehensive monitoring, tracing, and metrics collection.
//!
//! This module provides:
//! - OpenTelemetry integration with distributed tracing
//! - Prometheus metrics export
//! - Structured logging with correlation IDs
//! - Performance monitoring with minimal overhead (<2%)
//! - Health monitoring and check endpoints
//! - Performance profiling with flame graphs and regression detection

pub mod telemetry;
pub mod metrics;
pub mod logging;
pub mod correlation;
pub mod health;
pub mod profiling;

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
pub use health::{
    HealthStatus, HealthCheckResult, SystemMetrics, SystemHealthThresholds,
    HealthReport, ServiceInfo, HealthConfig, HealthChecker, 
    DatabaseHealthChecker, EventStoreHealthChecker, StreamingHealthChecker,
    SecurityHealthChecker, TenancyHealthChecker, HealthMonitorService
};
pub use profiling::{
    PerformanceProfiler, PerformanceProfilerBuilder, ProfilingConfig,
    ProfileType, ProfileEntry, MemoryInfo, IoInfo, CallGraphNode,
    RegressionDetection, PerformanceSnapshot, RegressionSeverity,
    FlameGraph, FlameGraphNode, BottleneckAnalysis, Bottleneck,
    BottleneckType, OptimizationSuggestion
};

use crate::error::Result;
use std::sync::Arc;
// use tokio::sync::RwLock; // Available for future async state management

/// Main observability service that coordinates all monitoring aspects
#[derive(Debug, Clone)]
pub struct ObservabilityService {
    telemetry: Arc<TelemetryProvider>,
    metrics: Arc<MetricsCollector>,
    logger: Arc<StructuredLogger>,
    correlation: Arc<CorrelationTracker>,
    profiler: Arc<PerformanceProfiler>,
    #[allow(dead_code)] // Configuration stored but not accessed after initialization
    config: ObservabilityConfig,
}

impl ObservabilityService {
    /// Create a new observability service with the given configuration
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        let telemetry = Arc::new(TelemetryProvider::new(&config).await?);
        let metrics = Arc::new(MetricsCollector::new(&config)?);
        let logger = Arc::new(StructuredLogger::new(&config)?);
        let correlation = Arc::new(CorrelationTracker::new());
        
        // Create profiler with default configuration
        let profiling_config = ProfilingConfig::default();
        let profiler = Arc::new(PerformanceProfiler::new(profiling_config));

        Ok(Self {
            telemetry,
            metrics,
            logger,
            correlation,
            profiler,
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

    /// Start a profiling session
    pub async fn start_profiling(
        &self,
        profile_type: ProfileType,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let correlation_id = generate_correlation_id();
        self.profiler.start_profiling(profile_type, Some(correlation_id), metadata).await
    }

    /// End a profiling session
    pub async fn end_profiling(&self, session_id: &str) -> Result<ProfileEntry> {
        self.profiler.end_profiling(session_id).await
    }

    /// Generate a flame graph
    pub async fn generate_flame_graph(
        &self,
        profile_type: ProfileType,
        time_range: Option<(std::time::SystemTime, std::time::SystemTime)>,
    ) -> Result<FlameGraph> {
        self.profiler.generate_flame_graph(profile_type, time_range).await
    }

    /// Detect performance regressions
    pub async fn detect_regressions(&self, operation: &str) -> Result<Option<RegressionDetection>> {
        self.profiler.detect_regressions(operation).await
    }

    /// Identify bottlenecks
    pub async fn identify_bottlenecks(&self, profile_type: ProfileType) -> Result<BottleneckAnalysis> {
        self.profiler.identify_bottlenecks(profile_type).await
    }

    /// Set baseline metrics for regression detection
    pub async fn set_baseline(&self, operation: &str) -> Result<()> {
        self.profiler.set_baseline(operation).await
    }

    /// Get profiling statistics
    pub async fn get_profiling_statistics(&self) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        self.profiler.get_statistics().await
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
    profiling_config: Option<ProfilingConfig>,
}

impl ObservabilityServiceBuilder {
    pub fn new() -> Self {
        Self {
            config: ObservabilityConfig::default(),
            profiling_config: None,
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

    pub fn with_profiling_config(mut self, config: ProfilingConfig) -> Self {
        self.profiling_config = Some(config);
        self
    }

    pub async fn build(self) -> Result<ObservabilityService> {
        let mut service = ObservabilityService::new(self.config).await?;
        
        // Replace profiler if custom config was provided
        if let Some(profiling_config) = self.profiling_config {
            service.profiler = Arc::new(PerformanceProfiler::new(profiling_config));
        }
        
        Ok(service)
    }
}

impl Default for ObservabilityServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}