//! Python bindings for the observability module

use pyo3::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use eventuali_core::{
    ObservabilityService, ObservabilityConfig,
    CorrelationId, generate_correlation_id,
    TraceContext, LogLevel, PerformanceMetrics,
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
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))?;
        
        let inner = runtime.block_on(async {
            ObservabilityService::new(config.inner).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create observability service: {}", e)))?;

        Ok(Self { inner, runtime })
    }

    pub fn initialize(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.initialize().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to initialize: {}", e)))?;
        
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
        let labels = eventuali_core::MetricLabels {
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

    pub fn shutdown(&self) -> PyResult<()> {
        self.runtime.block_on(async {
            self.inner.shutdown().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to shutdown: {}", e)))?;
        
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
    service: Option<Arc<PyObservabilityService>>,
    trace_context: Option<PyTraceContext>,
}

#[pymethods]
impl PyOperationTimer {
    #[new]
    pub fn new(operation: String, service: Option<Arc<PyObservabilityService>>) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            operation,
            service,
            trace_context: None,
        }
    }

    pub fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __exit__(&self, _exc_type: Option<PyObject>, _exc_val: Option<PyObject>, _exc_tb: Option<PyObject>) {
        let duration = self.start_time.elapsed();
        
        if let Some(service) = &self.service {
            let labels = HashMap::new();
            service.record_metric(
                format!("{}_duration_seconds", self.operation),
                duration.as_secs_f64(),
                Some(labels),
            );
        }
        
        // Also log the timing using basic println for now
        println!("Operation '{}' completed in {}ms", self.operation, duration.as_millis());
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

pub fn register_observability_classes(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyObservabilityConfig>()?;
    m.add_class::<PyObservabilityService>()?;
    m.add_class::<PyCorrelationId>()?;
    m.add_class::<PyTraceContext>()?;
    m.add_class::<PyLogLevel>()?;
    m.add_class::<PyPerformanceMetrics>()?;
    m.add_class::<PyOperationTimer>()?;
    
    m.add_function(wrap_pyfunction!(generate_correlation_id_py, m)?)?;
    
    Ok(())
}