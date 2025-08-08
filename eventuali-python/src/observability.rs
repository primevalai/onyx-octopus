//! Python bindings for the observability module

use pyo3::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use eventuali_core::{
    ObservabilityService, ObservabilityConfig,
    CorrelationId, generate_correlation_id,
    TraceContext, LogLevel, PerformanceMetrics,
};
use eventuali_core::observability::{
    HealthStatus, HealthCheckResult, HealthReport, HealthConfig,
    HealthMonitorService, SystemMetrics, MetricLabels,
    ProfileType, ProfileEntry, MemoryInfo, IoInfo, ProfilingConfig,
    RegressionDetection, PerformanceSnapshot, RegressionSeverity,
    FlameGraph, FlameGraphNode, BottleneckAnalysis, Bottleneck,
    BottleneckType, OptimizationSuggestion,
};
use std::sync::Arc;

#[pyclass(name = "ObservabilityConfig")]
#[derive(Clone)]
pub struct PyObservabilityConfig {
    inner: ObservabilityConfig,
}

#[pymethods]
impl PyObservabilityConfig {
    #[new]
    #[pyo3(signature = (
        service_name = "eventuali".to_string(),
        service_version = "0.1.0".to_string(),
        environment = "development".to_string(),
        tracing_enabled = true,
        metrics_enabled = true,
        structured_logging = true,
        jaeger_endpoint = None,
        prometheus_endpoint = None,
        sample_rate = 1.0,
        max_events_per_span = 128,
        export_timeout_millis = 30000
    ))]
    pub fn new(
        service_name: String,
        service_version: String,
        environment: String,
        tracing_enabled: bool,
        metrics_enabled: bool,
        structured_logging: bool,
        jaeger_endpoint: Option<String>,
        prometheus_endpoint: Option<String>,
        sample_rate: f64,
        max_events_per_span: u32,
        export_timeout_millis: u64,
    ) -> Self {
        Self {
            inner: ObservabilityConfig {
                service_name,
                service_version,
                environment,
                tracing_enabled,
                metrics_enabled,
                structured_logging,
                jaeger_endpoint,
                prometheus_endpoint,
                sample_rate,
                max_events_per_span,
                export_timeout_millis,
            },
        }
    }

    #[getter]
    pub fn service_name(&self) -> String {
        self.inner.service_name.clone()
    }

    #[getter]
    pub fn service_version(&self) -> String {
        self.inner.service_version.clone()
    }

    #[getter]
    pub fn environment(&self) -> String {
        self.inner.environment.clone()
    }

    #[getter]
    pub fn tracing_enabled(&self) -> bool {
        self.inner.tracing_enabled
    }

    #[getter]
    pub fn metrics_enabled(&self) -> bool {
        self.inner.metrics_enabled
    }

    #[getter]
    pub fn structured_logging(&self) -> bool {
        self.inner.structured_logging
    }
}

#[pyclass(name = "CorrelationId")]
#[derive(Clone)]
pub struct PyCorrelationId {
    inner: CorrelationId,
}

#[pymethods]
impl PyCorrelationId {
    #[new]
    pub fn new(id: Option<String>) -> Self {
        let inner = if let Some(id) = id {
            CorrelationId::new(id)
        } else {
            generate_correlation_id()
        };
        Self { inner }
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("CorrelationId('{}')", self.inner)
    }

    pub fn as_str(&self) -> String {
        self.inner.to_string()
    }
}

#[pyclass(name = "TraceContext")]
#[derive(Clone)]
pub struct PyTraceContext {
    inner: TraceContext,
}

#[pymethods]
impl PyTraceContext {
    #[getter]
    pub fn operation(&self) -> String {
        self.inner.operation.clone()
    }

    #[getter]
    pub fn correlation_id(&self) -> PyCorrelationId {
        PyCorrelationId {
            inner: self.inner.correlation_id.clone(),
        }
    }

    #[getter]
    pub fn trace_id(&self) -> Option<String> {
        self.inner.trace_id.clone()
    }

    #[getter]
    pub fn span_id(&self) -> Option<String> {
        self.inner.span_id.clone()
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        self.inner.add_attribute(&key, &value);
    }

    pub fn add_event(&self, name: String, attributes: Option<HashMap<String, String>>) {
        let attrs = attributes.unwrap_or_default();
        self.inner.add_event(&name, attrs);
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.inner.elapsed().as_millis() as u64
    }

    pub fn __str__(&self) -> String {
        format!("TraceContext(operation='{}', correlation_id='{}')", 
                self.inner.operation, self.inner.correlation_id)
    }
}

#[pyclass(name = "LogLevel")]
#[derive(Clone, Copy)]
pub enum PyLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<PyLogLevel> for LogLevel {
    fn from(level: PyLogLevel) -> Self {
        match level {
            PyLogLevel::Error => LogLevel::Error,
            PyLogLevel::Warn => LogLevel::Warn,
            PyLogLevel::Info => LogLevel::Info,
            PyLogLevel::Debug => LogLevel::Debug,
            PyLogLevel::Trace => LogLevel::Trace,
        }
    }
}

#[pyclass(name = "PerformanceMetrics")]
#[derive(Clone)]
pub struct PyPerformanceMetrics {
    inner: PerformanceMetrics,
}

#[pymethods]
impl PyPerformanceMetrics {
    #[getter]
    pub fn events_created_total(&self) -> u64 {
        self.inner.events_created_total
    }

    #[getter]
    pub fn events_stored_total(&self) -> u64 {
        self.inner.events_stored_total
    }

    #[getter]
    pub fn events_loaded_total(&self) -> u64 {
        self.inner.events_loaded_total
    }

    #[getter]
    pub fn error_count_total(&self) -> u64 {
        self.inner.error_count_total
    }

    #[getter]
    pub fn throughput_events_per_second(&self) -> f64 {
        self.inner.throughput_events_per_second
    }

    #[getter]
    pub fn memory_usage_bytes(&self) -> u64 {
        self.inner.memory_usage_bytes
    }

    #[getter]
    pub fn cpu_usage_percent(&self) -> f64 {
        self.inner.cpu_usage_percent
    }

    pub fn __str__(&self) -> String {
        format!("PerformanceMetrics(events_created={}, throughput={:.2} events/sec)", 
                self.inner.events_created_total, self.inner.throughput_events_per_second)
    }
}

#[pyclass(name = "ObservabilityService")]
pub struct PyObservabilityService {
    inner: ObservabilityService,
    runtime: Runtime,
}

#[pymethods]
impl PyObservabilityService {
    #[new]
    pub fn new(config: PyObservabilityConfig) -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {e}")))?;
        
        let inner = runtime.block_on(async {
            ObservabilityService::new(config.inner).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create observability service: {e}")))?;

        Ok(Self { inner, runtime })
    }

    pub fn initialize(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.initialize().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to initialize: {e}")))?;
        
        Ok(())
    }

    pub fn create_trace_context(&self, operation: String) -> PyTraceContext {
        let trace_context = self.inner.create_trace_context(&operation);
        PyTraceContext { inner: trace_context }
    }

    pub fn log_event(&self, level: PyLogLevel, message: String, trace_context: &PyTraceContext) {
        self.inner.log_event(level.into(), &message, &trace_context.inner);
    }

    pub fn record_metric(&self, name: String, value: f64, labels: Option<HashMap<String, String>>) {
        let labels = MetricLabels {
            labels: labels.unwrap_or_default(),
        };
        self.inner.record_metric(&name, value, labels);
    }

    pub fn get_performance_metrics(&self) -> PyResult<PyPerformanceMetrics> {
        let metrics = self.runtime.block_on(async {
            self.inner.get_performance_metrics().await
        });
        
        Ok(PyPerformanceMetrics { inner: metrics })
    }

    // Profiling methods
    pub fn start_profiling(&self, profile_type: PyProfileType, metadata: Option<HashMap<String, String>>) -> PyResult<String> {
        let metadata = metadata.unwrap_or_default();
        self.runtime.block_on(async {
            self.inner.start_profiling(profile_type.into(), metadata).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start profiling: {e}")))
    }

    pub fn end_profiling(&self, session_id: String) -> PyResult<PyProfileEntry> {
        let entry = self.runtime.block_on(async {
            self.inner.end_profiling(&session_id).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to end profiling: {e}")))?;
        
        Ok(PyProfileEntry { inner: entry })
    }

    pub fn generate_flame_graph(&self, profile_type: PyProfileType, start_time: Option<u64>, end_time: Option<u64>) -> PyResult<PyFlameGraph> {
        let time_range = if let (Some(start), Some(end)) = (start_time, end_time) {
            use std::time::{UNIX_EPOCH, Duration};
            Some((
                UNIX_EPOCH + Duration::from_secs(start),
                UNIX_EPOCH + Duration::from_secs(end)
            ))
        } else {
            None
        };

        let flame_graph = self.runtime.block_on(async {
            self.inner.generate_flame_graph(profile_type.into(), time_range).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to generate flame graph: {e}")))?;
        
        Ok(PyFlameGraph { inner: flame_graph })
    }

    pub fn detect_regressions(&self, operation: String) -> PyResult<Option<PyRegressionDetection>> {
        let detection = self.runtime.block_on(async {
            self.inner.detect_regressions(&operation).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to detect regressions: {e}")))?;
        
        Ok(detection.map(|d| PyRegressionDetection { inner: d }))
    }

    pub fn identify_bottlenecks(&self, profile_type: PyProfileType) -> PyResult<PyBottleneckAnalysis> {
        let analysis = self.runtime.block_on(async {
            self.inner.identify_bottlenecks(profile_type.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to identify bottlenecks: {e}")))?;
        
        Ok(PyBottleneckAnalysis { inner: analysis })
    }

    pub fn set_baseline(&self, operation: String) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.set_baseline(&operation).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to set baseline: {e}")))?;
        
        Ok(())
    }

    pub fn get_profiling_statistics(&self) -> PyResult<HashMap<String, String>> {
        let stats = self.runtime.block_on(async {
            self.inner.get_profiling_statistics().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get profiling statistics: {e}")))?;
        
        // Convert serde_json::Value to String for Python compatibility
        let mut result = HashMap::new();
        for (k, v) in stats {
            result.insert(k, v.to_string());
        }
        Ok(result)
    }

    pub fn shutdown(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.shutdown().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to shutdown: {e}")))?;
        
        Ok(())
    }

    pub fn __str__(&self) -> String {
        "ObservabilityService".to_string()
    }
}

#[pyfunction]
pub fn generate_correlation_id_py() -> PyCorrelationId {
    PyCorrelationId {
        inner: generate_correlation_id(),
    }
}

/// Timer context manager for Python
#[pyclass(name = "OperationTimer")]
pub struct PyOperationTimer {
    start_time: std::time::Instant,
    operation: String,
    #[allow(dead_code)]
    trace_context: Option<PyTraceContext>,
}

#[pymethods]
impl PyOperationTimer {
    #[new]
    pub fn new(operation: String) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            operation,
            trace_context: None,
        }
    }

    pub fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __exit__(&self, _exc_type: Option<PyObject>, _exc_val: Option<PyObject>, _exc_tb: Option<PyObject>) {
        let duration = self.start_time.elapsed();
        
        // Log the timing using basic println for now
        println!("Operation '{}' completed in {}ms", self.operation, duration.as_millis());
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

// Health monitoring classes

#[pyclass(name = "HealthStatus")]
#[derive(Clone, Copy, Debug)]
pub enum PyHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl From<HealthStatus> for PyHealthStatus {
    fn from(status: HealthStatus) -> Self {
        match status {
            HealthStatus::Healthy => PyHealthStatus::Healthy,
            HealthStatus::Degraded => PyHealthStatus::Degraded,
            HealthStatus::Unhealthy => PyHealthStatus::Unhealthy,
            HealthStatus::Unknown => PyHealthStatus::Unknown,
        }
    }
}

impl From<PyHealthStatus> for HealthStatus {
    fn from(status: PyHealthStatus) -> Self {
        match status {
            PyHealthStatus::Healthy => HealthStatus::Healthy,
            PyHealthStatus::Degraded => HealthStatus::Degraded,
            PyHealthStatus::Unhealthy => HealthStatus::Unhealthy,
            PyHealthStatus::Unknown => HealthStatus::Unknown,
        }
    }
}

#[pymethods]
impl PyHealthStatus {
    pub fn score(&self) -> u8 {
        HealthStatus::from(*self).score()
    }

    pub fn http_status(&self) -> u16 {
        HealthStatus::from(*self).http_status()
    }

    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

#[pyclass(name = "HealthCheckResult")]
#[derive(Clone)]
pub struct PyHealthCheckResult {
    inner: HealthCheckResult,
}

#[pymethods]
impl PyHealthCheckResult {
    #[new]
    pub fn new(component: String, status: PyHealthStatus, message: String) -> Self {
        Self {
            inner: HealthCheckResult::new(component, status.into(), message),
        }
    }

    #[getter]
    pub fn component(&self) -> String {
        self.inner.component.clone()
    }

    #[getter]
    pub fn status(&self) -> PyHealthStatus {
        self.inner.status.into()
    }

    #[getter]
    pub fn message(&self) -> String {
        self.inner.message.clone()
    }

    #[getter]
    pub fn details(&self) -> HashMap<String, String> {
        self.inner.details.iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }

    #[getter]
    pub fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    #[getter]
    pub fn duration_ms(&self) -> u64 {
        self.inner.duration_ms
    }

    #[getter]
    pub fn critical(&self) -> bool {
        self.inner.critical
    }

    pub fn __str__(&self) -> String {
        format!("HealthCheckResult(component='{}', status={:?}, message='{}')", 
                self.inner.component, self.inner.status, self.inner.message)
    }
}

#[pyclass(name = "SystemMetrics")]
#[derive(Clone)]
pub struct PySystemMetrics {
    inner: SystemMetrics,
}

#[pymethods]
impl PySystemMetrics {
    #[getter]
    pub fn cpu_usage_percent(&self) -> f64 {
        self.inner.cpu_usage_percent
    }

    #[getter]
    pub fn memory_used_bytes(&self) -> u64 {
        self.inner.memory_used_bytes
    }

    #[getter]
    pub fn memory_total_bytes(&self) -> u64 {
        self.inner.memory_total_bytes
    }

    #[getter]
    pub fn memory_usage_percent(&self) -> f64 {
        self.inner.memory_usage_percent
    }

    #[getter]
    pub fn disk_used_bytes(&self) -> u64 {
        self.inner.disk_used_bytes
    }

    #[getter]
    pub fn disk_total_bytes(&self) -> u64 {
        self.inner.disk_total_bytes
    }

    #[getter]
    pub fn disk_usage_percent(&self) -> f64 {
        self.inner.disk_usage_percent
    }

    #[getter]
    pub fn network_bytes_received(&self) -> u64 {
        self.inner.network_bytes_received
    }

    #[getter]
    pub fn network_bytes_transmitted(&self) -> u64 {
        self.inner.network_bytes_transmitted
    }

    #[getter]
    pub fn active_connections(&self) -> u32 {
        self.inner.active_connections
    }

    #[getter]
    pub fn uptime_seconds(&self) -> u64 {
        self.inner.uptime_seconds
    }

    pub fn __str__(&self) -> String {
        format!("SystemMetrics(cpu={}%, memory={}%, disk={}%)", 
                self.inner.cpu_usage_percent, 
                self.inner.memory_usage_percent,
                self.inner.disk_usage_percent)
    }
}

#[pyclass(name = "HealthReport")]
#[derive(Clone)]
pub struct PyHealthReport {
    inner: HealthReport,
}

#[pymethods]
impl PyHealthReport {
    #[getter]
    pub fn overall_status(&self) -> PyHealthStatus {
        self.inner.overall_status.into()
    }

    #[getter]
    pub fn overall_score(&self) -> f64 {
        self.inner.overall_score
    }

    #[getter]
    pub fn components(&self) -> Vec<PyHealthCheckResult> {
        self.inner.components.iter()
            .map(|c| PyHealthCheckResult { inner: c.clone() })
            .collect()
    }

    #[getter]
    pub fn system_metrics(&self) -> PySystemMetrics {
        PySystemMetrics {
            inner: self.inner.system_metrics.clone(),
        }
    }

    #[getter]
    pub fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    #[getter]
    pub fn generation_time_ms(&self) -> u64 {
        self.inner.generation_time_ms
    }

    pub fn http_status(&self) -> u16 {
        self.inner.http_status()
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to serialize health report: {e}")))
    }

    pub fn __str__(&self) -> String {
        format!("HealthReport(status={:?}, score={:.1}, components={})", 
                self.inner.overall_status, self.inner.overall_score, self.inner.components.len())
    }
}

#[pyclass(name = "HealthConfig")]
#[derive(Clone)]
pub struct PyHealthConfig {
    inner: HealthConfig,
}

#[pymethods]
impl PyHealthConfig {
    #[new]
    #[pyo3(signature = (
        check_interval_seconds = 30,
        check_timeout_seconds = 5,
        include_system_metrics = true,
        background_checks = true,
        http_port = 8080,
        http_bind_address = "0.0.0.0".to_string(),
        service_name = "eventuali".to_string(),
        service_version = "0.1.0".to_string(),
        environment = "development".to_string()
    ))]
    pub fn new(
        check_interval_seconds: u64,
        check_timeout_seconds: u64,
        include_system_metrics: bool,
        background_checks: bool,
        http_port: u16,
        http_bind_address: String,
        service_name: String,
        service_version: String,
        environment: String,
    ) -> Self {
        let mut config = HealthConfig::default();
        config.check_interval_seconds = check_interval_seconds;
        config.check_timeout_seconds = check_timeout_seconds;
        config.include_system_metrics = include_system_metrics;
        config.background_checks = background_checks;
        config.http_port = http_port;
        config.http_bind_address = http_bind_address;
        config.service_info.name = service_name;
        config.service_info.version = service_version;
        config.service_info.environment = environment;

        Self { inner: config }
    }

    #[getter]
    pub fn check_interval_seconds(&self) -> u64 {
        self.inner.check_interval_seconds
    }

    #[getter]
    pub fn http_port(&self) -> u16 {
        self.inner.http_port
    }

    #[getter]
    pub fn service_name(&self) -> String {
        self.inner.service_info.name.clone()
    }
}

#[pyclass(name = "HealthMonitorService")]
pub struct PyHealthMonitorService {
    inner: HealthMonitorService,
    runtime: Runtime,
}

#[pymethods]
impl PyHealthMonitorService {
    #[new]
    pub fn new(config: PyHealthConfig) -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {e}")))?;
        
        let mut inner = HealthMonitorService::new(config.inner);
        // Add default health checkers
        inner.add_default_checkers("sqlite://:memory:");

        Ok(Self { inner, runtime })
    }

    pub fn with_database(&mut self, connection_string: String) -> PyResult<()> {
        self.inner.add_default_checkers(&connection_string);
        Ok(())
    }

    pub fn start(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.start().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to start health monitor: {e}")))?;
        
        Ok(())
    }

    pub fn stop(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.stop().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to stop health monitor: {e}")))?;
        
        Ok(())
    }

    pub fn run_health_checks(&self) -> PyResult<PyHealthReport> {
        let report = self.runtime.block_on(async {
            self.inner.run_health_checks().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to run health checks: {e}")))?;
        
        Ok(PyHealthReport { inner: report })
    }

    pub fn get_latest_report(&self) -> Option<PyHealthReport> {
        self.runtime.block_on(async {
            self.inner.get_latest_report().await.map(|report| PyHealthReport { inner: report })
        })
    }

    pub fn is_ready(&self) -> bool {
        self.runtime.block_on(async {
            self.inner.is_ready().await
        })
    }

    pub fn is_live(&self) -> bool {
        self.runtime.block_on(async {
            self.inner.is_live().await
        })
    }

    pub fn get_health_summary(&self) -> HashMap<String, String> {
        let summary = self.runtime.block_on(async {
            self.inner.get_health_summary().await
        });
        
        summary.into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect()
    }

    pub fn __str__(&self) -> String {
        "HealthMonitorService".to_string()
    }
}

/// HTTP Health Server for Kubernetes integration  
#[pyclass(name = "HealthHttpServer")]
pub struct PyHealthHttpServer {
    health_service: HealthMonitorService,
    port: u16,
    bind_address: String,
    is_running: Arc<std::sync::RwLock<bool>>,
    runtime: Runtime,
}

#[pymethods]
impl PyHealthHttpServer {
    #[new]
    pub fn new(health_config: PyHealthConfig, port: Option<u16>, bind_address: Option<String>) -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {e}")))?;
        
        let mut health_service = HealthMonitorService::new(health_config.inner);
        health_service.add_default_checkers("sqlite://:memory:");
        
        Ok(Self {
            health_service,
            port: port.unwrap_or(8080),
            bind_address: bind_address.unwrap_or("0.0.0.0".to_string()),
            is_running: Arc::new(std::sync::RwLock::new(false)),
            runtime,
        })
    }

    pub fn start_server(&self) -> PyResult<()> {
        let mut running = self.is_running.write()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to acquire lock: {e}")))?;
        
        if *running {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Health server is already running"));
        }
        
        *running = true;
        
        // In a real implementation, this would start an actual HTTP server
        // For now, we'll simulate it
        println!("Health HTTP server starting on {}:{}", self.bind_address, self.port);
        println!("Health endpoints available:");
        println!("  GET /health - Full health report");
        println!("  GET /ready - Readiness check");
        println!("  GET /live - Liveness check");
        println!("  GET /metrics - Health metrics (Prometheus format)");
        
        Ok(())
    }

    pub fn stop_server(&self) -> PyResult<()> {
        let mut running = self.is_running.write()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to acquire lock: {e}")))?;
        
        *running = false;
        println!("Health HTTP server stopped");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.read().map(|v| *v).unwrap_or(false)
    }

    pub fn get_health_json(&self) -> PyResult<String> {
        let report = self.runtime.block_on(async {
            self.health_service.run_health_checks().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to run health checks: {e}")))?;
        
        serde_json::to_string(&report)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to serialize health report: {e}")))
    }

    pub fn get_readiness_json(&self) -> String {
        let is_ready = self.runtime.block_on(async {
            self.health_service.is_ready().await
        });
        let status = if is_ready { "ready" } else { "not_ready" };
        let http_code = if is_ready { 200 } else { 503 };
        
        format!(r#"{{"status": "{}", "http_code": {}}}"#, status, http_code)
    }

    pub fn get_liveness_json(&self) -> String {
        let is_live = self.runtime.block_on(async {
            self.health_service.is_live().await
        });
        let status = if is_live { "live" } else { "not_live" };
        let http_code = if is_live { 200 } else { 503 };
        
        format!(r#"{{"status": "{}", "http_code": {}}}"#, status, http_code)
    }

    pub fn get_metrics_prometheus(&self) -> String {
        // Generate Prometheus-format metrics from health data
        let report = self.runtime.block_on(async {
            self.health_service.get_latest_report().await
        });
        
        if let Some(report) = report {
            let mut metrics = String::new();
            
            // Overall health metrics
            metrics.push_str(&format!("# HELP eventuali_health_score Overall health score (0-100)\n"));
            metrics.push_str(&format!("# TYPE eventuali_health_score gauge\n"));
            metrics.push_str(&format!("eventuali_health_score {}\n", report.overall_score));
            
            metrics.push_str(&format!("# HELP eventuali_health_status Overall health status (0=Unknown, 1=Unhealthy, 2=Degraded, 3=Healthy)\n"));
            metrics.push_str(&format!("# TYPE eventuali_health_status gauge\n"));
            metrics.push_str(&format!("eventuali_health_status {}\n", 
                match report.overall_status {
                    HealthStatus::Healthy => 3,
                    HealthStatus::Degraded => 2,
                    HealthStatus::Unhealthy => 1,
                    HealthStatus::Unknown => 0,
                }
            ));
            
            // Component health metrics
            for component in &report.components {
                let component_name = &component.component;
                metrics.push_str(&format!("# HELP eventuali_component_health_{component_name} Component health status\n"));
                metrics.push_str(&format!("# TYPE eventuali_component_health_{component_name} gauge\n"));
                metrics.push_str(&format!("eventuali_component_health_{component_name} {}\n", component.status.score()));
            }
            
            // System metrics
            let sys_metrics = &report.system_metrics;
            metrics.push_str(&format!("# HELP eventuali_cpu_usage_percent CPU usage percentage\n"));
            metrics.push_str(&format!("# TYPE eventuali_cpu_usage_percent gauge\n"));
            metrics.push_str(&format!("eventuali_cpu_usage_percent {}\n", sys_metrics.cpu_usage_percent));
            
            metrics.push_str(&format!("# HELP eventuali_memory_usage_percent Memory usage percentage\n"));
            metrics.push_str(&format!("# TYPE eventuali_memory_usage_percent gauge\n"));
            metrics.push_str(&format!("eventuali_memory_usage_percent {}\n", sys_metrics.memory_usage_percent));
            
            metrics
        } else {
            "# No health data available\n".to_string()
        }
    }

    pub fn __str__(&self) -> String {
        format!("HealthHttpServer({}:{}, running={})", self.bind_address, self.port, self.is_running())
    }
}

// Profiling Classes

#[pyclass(name = "ProfileType")]
#[derive(Clone, Copy, Debug)]
pub enum PyProfileType {
    Cpu,
    Memory,
    Io,
    Method,
    Combined,
}

impl From<PyProfileType> for ProfileType {
    fn from(py_type: PyProfileType) -> Self {
        match py_type {
            PyProfileType::Cpu => ProfileType::Cpu,
            PyProfileType::Memory => ProfileType::Memory,
            PyProfileType::Io => ProfileType::Io,
            PyProfileType::Method => ProfileType::Method,
            PyProfileType::Combined => ProfileType::Combined,
        }
    }
}

impl From<ProfileType> for PyProfileType {
    fn from(profile_type: ProfileType) -> Self {
        match profile_type {
            ProfileType::Cpu => PyProfileType::Cpu,
            ProfileType::Memory => PyProfileType::Memory,
            ProfileType::Io => PyProfileType::Io,
            ProfileType::Method => PyProfileType::Method,
            ProfileType::Combined => PyProfileType::Combined,
        }
    }
}

#[pymethods]
impl PyProfileType {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

#[pyclass(name = "ProfilingConfig")]
#[derive(Clone)]
pub struct PyProfilingConfig {
    inner: ProfilingConfig,
}

#[pymethods]
impl PyProfilingConfig {
    #[new]
    #[pyo3(signature = (
        enabled = true,
        cpu_sampling_interval_us = 1000,
        memory_allocation_threshold = 1024,
        io_threshold_us = 100,
        max_stack_frames = 32,
        data_retention_seconds = 3600,
        enable_flame_graphs = true,
        regression_threshold_percent = 10.0
    ))]
    pub fn new(
        enabled: bool,
        cpu_sampling_interval_us: u64,
        memory_allocation_threshold: usize,
        io_threshold_us: u64,
        max_stack_frames: usize,
        data_retention_seconds: u64,
        enable_flame_graphs: bool,
        regression_threshold_percent: f64,
    ) -> Self {
        Self {
            inner: ProfilingConfig {
                enabled,
                cpu_sampling_interval_us,
                memory_allocation_threshold,
                io_threshold_us,
                max_stack_frames,
                data_retention_seconds,
                enable_flame_graphs,
                regression_threshold_percent,
            },
        }
    }

    #[getter]
    pub fn enabled(&self) -> bool {
        self.inner.enabled
    }

    #[getter]
    pub fn cpu_sampling_interval_us(&self) -> u64 {
        self.inner.cpu_sampling_interval_us
    }

    #[getter]
    pub fn memory_allocation_threshold(&self) -> usize {
        self.inner.memory_allocation_threshold
    }

    #[getter]
    pub fn enable_flame_graphs(&self) -> bool {
        self.inner.enable_flame_graphs
    }
}

#[pyclass(name = "MemoryInfo")]
#[derive(Clone)]
pub struct PyMemoryInfo {
    inner: MemoryInfo,
}

#[pymethods]
impl PyMemoryInfo {
    #[getter]
    pub fn allocated_bytes(&self) -> usize {
        self.inner.allocated_bytes
    }

    #[getter]
    pub fn deallocated_bytes(&self) -> usize {
        self.inner.deallocated_bytes
    }

    #[getter]
    pub fn current_usage_bytes(&self) -> usize {
        self.inner.current_usage_bytes
    }

    #[getter]
    pub fn peak_usage_bytes(&self) -> usize {
        self.inner.peak_usage_bytes
    }

    #[getter]
    pub fn allocation_count(&self) -> usize {
        self.inner.allocation_count
    }

    pub fn __str__(&self) -> String {
        format!("MemoryInfo(current={}KB, peak={}KB, allocations={})", 
                self.inner.current_usage_bytes / 1024,
                self.inner.peak_usage_bytes / 1024,
                self.inner.allocation_count)
    }
}

#[pyclass(name = "IoInfo")]
#[derive(Clone)]
pub struct PyIoInfo {
    inner: IoInfo,
}

#[pymethods]
impl PyIoInfo {
    #[getter]
    pub fn operation_type(&self) -> String {
        self.inner.operation_type.clone()
    }

    #[getter]
    pub fn bytes_read(&self) -> u64 {
        self.inner.bytes_read
    }

    #[getter]
    pub fn bytes_written(&self) -> u64 {
        self.inner.bytes_written
    }

    #[getter]
    pub fn operation_count(&self) -> u64 {
        self.inner.operation_count
    }

    #[getter]
    pub fn target(&self) -> String {
        self.inner.target.clone()
    }

    pub fn __str__(&self) -> String {
        format!("IoInfo(type={}, read={}B, written={}B)", 
                self.inner.operation_type,
                self.inner.bytes_read,
                self.inner.bytes_written)
    }
}

#[pyclass(name = "ProfileEntry")]
#[derive(Clone)]
pub struct PyProfileEntry {
    inner: ProfileEntry,
}

#[pymethods]
impl PyProfileEntry {
    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    pub fn profile_type(&self) -> PyProfileType {
        self.inner.profile_type.into()
    }

    #[getter]
    pub fn duration_ms(&self) -> u64 {
        self.inner.duration.as_millis() as u64
    }

    #[getter]
    pub fn stack_trace(&self) -> Vec<String> {
        self.inner.stack_trace.clone()
    }

    #[getter]
    pub fn memory_info(&self) -> Option<PyMemoryInfo> {
        self.inner.memory_info.as_ref().map(|info| PyMemoryInfo { inner: info.clone() })
    }

    #[getter]
    pub fn io_info(&self) -> Option<PyIoInfo> {
        self.inner.io_info.as_ref().map(|info| PyIoInfo { inner: info.clone() })
    }

    #[getter]
    pub fn metadata(&self) -> HashMap<String, String> {
        self.inner.metadata.clone()
    }

    pub fn __str__(&self) -> String {
        format!("ProfileEntry(id={}, type={:?}, duration={}ms)", 
                self.inner.id, self.inner.profile_type, self.inner.duration.as_millis())
    }
}

#[pyclass(name = "RegressionSeverity")]
#[derive(Clone, Copy, Debug)]
pub enum PyRegressionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl From<RegressionSeverity> for PyRegressionSeverity {
    fn from(severity: RegressionSeverity) -> Self {
        match severity {
            RegressionSeverity::Low => PyRegressionSeverity::Low,
            RegressionSeverity::Medium => PyRegressionSeverity::Medium,
            RegressionSeverity::High => PyRegressionSeverity::High,
            RegressionSeverity::Critical => PyRegressionSeverity::Critical,
        }
    }
}

#[pymethods]
impl PyRegressionSeverity {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

#[pyclass(name = "PerformanceSnapshot")]
#[derive(Clone)]
pub struct PyPerformanceSnapshot {
    inner: PerformanceSnapshot,
}

#[pymethods]
impl PyPerformanceSnapshot {
    #[getter]
    pub fn avg_execution_time_ms(&self) -> u64 {
        self.inner.avg_execution_time.as_millis() as u64
    }

    #[getter]
    pub fn p95_execution_time_ms(&self) -> u64 {
        self.inner.p95_execution_time.as_millis() as u64
    }

    #[getter]
    pub fn p99_execution_time_ms(&self) -> u64 {
        self.inner.p99_execution_time.as_millis() as u64
    }

    #[getter]
    pub fn throughput(&self) -> f64 {
        self.inner.throughput
    }

    #[getter]
    pub fn memory_usage_bytes(&self) -> usize {
        self.inner.memory_usage_bytes
    }

    #[getter]
    pub fn error_rate(&self) -> f64 {
        self.inner.error_rate
    }

    pub fn __str__(&self) -> String {
        format!("PerformanceSnapshot(avg={}ms, p95={}ms, throughput={:.2}/s)", 
                self.inner.avg_execution_time.as_millis(),
                self.inner.p95_execution_time.as_millis(),
                self.inner.throughput)
    }
}

#[pyclass(name = "RegressionDetection")]
#[derive(Clone)]
pub struct PyRegressionDetection {
    inner: RegressionDetection,
}

#[pymethods]
impl PyRegressionDetection {
    #[getter]
    pub fn operation(&self) -> String {
        self.inner.operation.clone()
    }

    #[getter]
    pub fn current_metrics(&self) -> PyPerformanceSnapshot {
        PyPerformanceSnapshot { inner: self.inner.current_metrics.clone() }
    }

    #[getter]
    pub fn baseline_metrics(&self) -> PyPerformanceSnapshot {
        PyPerformanceSnapshot { inner: self.inner.baseline_metrics.clone() }
    }

    #[getter]
    pub fn change_percent(&self) -> f64 {
        self.inner.change_percent
    }

    #[getter]
    pub fn is_regression(&self) -> bool {
        self.inner.is_regression
    }

    #[getter]
    pub fn severity(&self) -> PyRegressionSeverity {
        self.inner.severity.into()
    }

    #[getter]
    pub fn recommendations(&self) -> Vec<String> {
        self.inner.recommendations.clone()
    }

    pub fn __str__(&self) -> String {
        format!("RegressionDetection(operation={}, change={:.1}%, regression={})", 
                self.inner.operation, self.inner.change_percent, self.inner.is_regression)
    }
}

#[pyclass(name = "FlameGraphNode")]
#[derive(Clone)]
pub struct PyFlameGraphNode {
    inner: FlameGraphNode,
}

#[pymethods]
impl PyFlameGraphNode {
    #[getter]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    pub fn total_time_ms(&self) -> u64 {
        self.inner.total_time.as_millis() as u64
    }

    #[getter]
    pub fn self_time_ms(&self) -> u64 {
        self.inner.self_time.as_millis() as u64
    }

    #[getter]
    pub fn sample_count(&self) -> usize {
        self.inner.sample_count
    }

    #[getter]
    pub fn percentage(&self) -> f64 {
        self.inner.percentage
    }

    #[getter]
    pub fn children(&self) -> HashMap<String, PyFlameGraphNode> {
        self.inner.children.iter()
            .map(|(k, v)| (k.clone(), PyFlameGraphNode { inner: v.clone() }))
            .collect()
    }

    pub fn __str__(&self) -> String {
        format!("FlameGraphNode(name={}, percentage={:.1}%, samples={})", 
                self.inner.name, self.inner.percentage, self.inner.sample_count)
    }
}

#[pyclass(name = "FlameGraph")]
#[derive(Clone)]
pub struct PyFlameGraph {
    inner: FlameGraph,
}

#[pymethods]
impl PyFlameGraph {
    #[getter]
    pub fn root(&self) -> PyFlameGraphNode {
        PyFlameGraphNode { inner: self.inner.root.clone() }
    }

    #[getter]
    pub fn total_duration_ms(&self) -> u64 {
        self.inner.total_duration.as_millis() as u64
    }

    #[getter]
    pub fn sample_count(&self) -> usize {
        self.inner.sample_count
    }

    #[getter]
    pub fn metadata(&self) -> HashMap<String, String> {
        self.inner.metadata.clone()
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to serialize flame graph: {}", e)))
    }

    pub fn __str__(&self) -> String {
        format!("FlameGraph(duration={}ms, samples={})", 
                self.inner.total_duration.as_millis(), self.inner.sample_count)
    }
}

#[pyclass(name = "BottleneckType")]
#[derive(Clone, Copy, Debug)]
pub enum PyBottleneckType {
    Cpu,
    Memory,
    Io,
    Lock,
    Network,
    Database,
    Serialization,
}

impl From<BottleneckType> for PyBottleneckType {
    fn from(bottleneck_type: BottleneckType) -> Self {
        match bottleneck_type {
            BottleneckType::Cpu => PyBottleneckType::Cpu,
            BottleneckType::Memory => PyBottleneckType::Memory,
            BottleneckType::Io => PyBottleneckType::Io,
            BottleneckType::Lock => PyBottleneckType::Lock,
            BottleneckType::Network => PyBottleneckType::Network,
            BottleneckType::Database => PyBottleneckType::Database,
            BottleneckType::Serialization => PyBottleneckType::Serialization,
        }
    }
}

#[pymethods]
impl PyBottleneckType {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

#[pyclass(name = "Bottleneck")]
#[derive(Clone)]
pub struct PyBottleneck {
    inner: Bottleneck,
}

#[pymethods]
impl PyBottleneck {
    #[getter]
    pub fn location(&self) -> String {
        self.inner.location.clone()
    }

    #[getter]
    pub fn bottleneck_type(&self) -> PyBottleneckType {
        self.inner.bottleneck_type.into()
    }

    #[getter]
    pub fn impact_score(&self) -> f64 {
        self.inner.impact_score
    }

    #[getter]
    pub fn time_spent_ms(&self) -> u64 {
        self.inner.time_spent.as_millis() as u64
    }

    #[getter]
    pub fn percentage_of_total(&self) -> f64 {
        self.inner.percentage_of_total
    }

    #[getter]
    pub fn call_frequency(&self) -> u64 {
        self.inner.call_frequency
    }

    #[getter]
    pub fn description(&self) -> String {
        self.inner.description.clone()
    }

    pub fn __str__(&self) -> String {
        format!("Bottleneck(location={}, type={:?}, impact={:.1}%)", 
                self.inner.location, self.inner.bottleneck_type, self.inner.impact_score)
    }
}

#[pyclass(name = "OptimizationSuggestion")]
#[derive(Clone)]
pub struct PyOptimizationSuggestion {
    inner: OptimizationSuggestion,
}

#[pymethods]
impl PyOptimizationSuggestion {
    #[getter]
    pub fn target(&self) -> String {
        self.inner.target.clone()
    }

    #[getter]
    pub fn optimization_type(&self) -> String {
        self.inner.optimization_type.clone()
    }

    #[getter]
    pub fn expected_impact(&self) -> String {
        self.inner.expected_impact.clone()
    }

    #[getter]
    pub fn effort_level(&self) -> String {
        self.inner.effort_level.clone()
    }

    #[getter]
    pub fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    pub fn examples(&self) -> Vec<String> {
        self.inner.examples.clone()
    }

    pub fn __str__(&self) -> String {
        format!("OptimizationSuggestion(target={}, type={}, impact={})", 
                self.inner.target, self.inner.optimization_type, self.inner.expected_impact)
    }
}

#[pyclass(name = "BottleneckAnalysis")]
#[derive(Clone)]
pub struct PyBottleneckAnalysis {
    inner: BottleneckAnalysis,
}

#[pymethods]
impl PyBottleneckAnalysis {
    #[getter]
    pub fn bottlenecks(&self) -> Vec<PyBottleneck> {
        self.inner.bottlenecks.iter()
            .map(|b| PyBottleneck { inner: b.clone() })
            .collect()
    }

    #[getter]
    pub fn analysis_duration_ms(&self) -> u64 {
        self.inner.analysis_duration.as_millis() as u64
    }

    #[getter]
    pub fn optimization_suggestions(&self) -> Vec<PyOptimizationSuggestion> {
        self.inner.optimization_suggestions.iter()
            .map(|s| PyOptimizationSuggestion { inner: s.clone() })
            .collect()
    }

    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to serialize bottleneck analysis: {}", e)))
    }

    pub fn __str__(&self) -> String {
        format!("BottleneckAnalysis(bottlenecks={}, suggestions={})", 
                self.inner.bottlenecks.len(), self.inner.optimization_suggestions.len())
    }
}

pub fn register_observability_classes(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyObservabilityConfig>()?;
    m.add_class::<PyObservabilityService>()?;
    m.add_class::<PyCorrelationId>()?;
    m.add_class::<PyTraceContext>()?;
    m.add_class::<PyLogLevel>()?;
    m.add_class::<PyPerformanceMetrics>()?;
    m.add_class::<PyOperationTimer>()?;
    
    // Health monitoring classes
    m.add_class::<PyHealthStatus>()?;
    m.add_class::<PyHealthCheckResult>()?;
    m.add_class::<PySystemMetrics>()?;
    m.add_class::<PyHealthReport>()?;
    m.add_class::<PyHealthConfig>()?;
    m.add_class::<PyHealthMonitorService>()?;
    m.add_class::<PyHealthHttpServer>()?;
    
    // Profiling classes
    m.add_class::<PyProfileType>()?;
    m.add_class::<PyProfilingConfig>()?;
    m.add_class::<PyMemoryInfo>()?;
    m.add_class::<PyIoInfo>()?;
    m.add_class::<PyProfileEntry>()?;
    m.add_class::<PyRegressionSeverity>()?;
    m.add_class::<PyPerformanceSnapshot>()?;
    m.add_class::<PyRegressionDetection>()?;
    m.add_class::<PyFlameGraphNode>()?;
    m.add_class::<PyFlameGraph>()?;
    m.add_class::<PyBottleneckType>()?;
    m.add_class::<PyBottleneck>()?;
    m.add_class::<PyOptimizationSuggestion>()?;
    m.add_class::<PyBottleneckAnalysis>()?;
    
    m.add_function(wrap_pyfunction!(generate_correlation_id_py, m)?)?;
    
    Ok(())
}