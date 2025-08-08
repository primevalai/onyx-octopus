//! Python bindings for performance optimization features
//!
//! Provides Python access to high-performance connection pooling, WAL optimization,
//! batch processing, read replicas, caching, and compression features.

use pyo3::prelude::*;
use std::collections::HashMap;
use eventuali_core::performance::{ConnectionPool, PoolConfig, PoolStats};
use eventuali_core::EventualiError;

/// Python wrapper for PoolConfig
#[pyclass(name = "PoolConfig")]
#[derive(Clone)]
pub struct PyPoolConfig {
    pub inner: PoolConfig,
}

#[pymethods]
impl PyPoolConfig {
    #[new]
    #[pyo3(signature = (
        min_connections = 5,
        max_connections = 100,
        connection_timeout_ms = 5000,
        idle_timeout_ms = 300000,
        health_check_interval_ms = 30000,
        auto_scaling_enabled = true,
        scale_up_threshold = 0.8,
        scale_down_threshold = 0.3
    ))]
    pub fn new(
        min_connections: usize,
        max_connections: usize,
        connection_timeout_ms: u64,
        idle_timeout_ms: u64,
        health_check_interval_ms: u64,
        auto_scaling_enabled: bool,
        scale_up_threshold: f64,
        scale_down_threshold: f64,
    ) -> Self {
        Self {
            inner: PoolConfig {
                min_connections,
                max_connections,
                connection_timeout_ms,
                idle_timeout_ms,
                health_check_interval_ms,
                auto_scaling_enabled,
                scale_up_threshold,
                scale_down_threshold,
            }
        }
    }

    #[staticmethod]
    pub fn default() -> Self {
        Self {
            inner: PoolConfig::default(),
        }
    }

    #[staticmethod]
    pub fn high_performance() -> Self {
        Self {
            inner: PoolConfig {
                min_connections: 10,
                max_connections: 200,
                connection_timeout_ms: 2000,
                idle_timeout_ms: 180000, // 3 minutes
                health_check_interval_ms: 15000, // 15 seconds
                auto_scaling_enabled: true,
                scale_up_threshold: 0.7,
                scale_down_threshold: 0.2,
            }
        }
    }

    #[staticmethod]
    pub fn memory_optimized() -> Self {
        Self {
            inner: PoolConfig {
                min_connections: 3,
                max_connections: 50,
                connection_timeout_ms: 10000,
                idle_timeout_ms: 600000, // 10 minutes
                health_check_interval_ms: 60000, // 1 minute
                auto_scaling_enabled: true,
                scale_up_threshold: 0.9,
                scale_down_threshold: 0.1,
            }
        }
    }

    #[getter]
    pub fn min_connections(&self) -> usize {
        self.inner.min_connections
    }

    #[setter]
    pub fn set_min_connections(&mut self, value: usize) {
        self.inner.min_connections = value;
    }

    #[getter]
    pub fn max_connections(&self) -> usize {
        self.inner.max_connections
    }

    #[setter]
    pub fn set_max_connections(&mut self, value: usize) {
        self.inner.max_connections = value;
    }

    #[getter]
    pub fn connection_timeout_ms(&self) -> u64 {
        self.inner.connection_timeout_ms
    }

    #[setter]
    pub fn set_connection_timeout_ms(&mut self, value: u64) {
        self.inner.connection_timeout_ms = value;
    }

    #[getter]
    pub fn auto_scaling_enabled(&self) -> bool {
        self.inner.auto_scaling_enabled
    }

    #[setter]
    pub fn set_auto_scaling_enabled(&mut self, value: bool) {
        self.inner.auto_scaling_enabled = value;
    }

    #[getter]
    pub fn scale_up_threshold(&self) -> f64 {
        self.inner.scale_up_threshold
    }

    #[setter]
    pub fn set_scale_up_threshold(&mut self, value: f64) {
        self.inner.scale_up_threshold = value;
    }

    #[getter]
    pub fn scale_down_threshold(&self) -> f64 {
        self.inner.scale_down_threshold
    }

    #[setter]
    pub fn set_scale_down_threshold(&mut self, value: f64) {
        self.inner.scale_down_threshold = value;
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PoolConfig(min_connections={}, max_connections={}, connection_timeout_ms={}, auto_scaling_enabled={})",
            self.inner.min_connections,
            self.inner.max_connections,
            self.inner.connection_timeout_ms,
            self.inner.auto_scaling_enabled
        )
    }
}

/// Python wrapper for PoolStats
#[pyclass(name = "PoolStats")]
#[derive(Clone)]
pub struct PyPoolStats {
    pub inner: PoolStats,
}

#[pymethods]
impl PyPoolStats {
    #[getter]
    pub fn total_connections(&self) -> usize {
        self.inner.total_connections
    }

    #[getter]
    pub fn active_connections(&self) -> usize {
        self.inner.active_connections
    }

    #[getter]
    pub fn idle_connections(&self) -> usize {
        self.inner.idle_connections
    }

    #[getter]
    pub fn total_requests(&self) -> u64 {
        self.inner.total_requests
    }

    #[getter]
    pub fn successful_requests(&self) -> u64 {
        self.inner.successful_requests
    }

    #[getter]
    pub fn failed_requests(&self) -> u64 {
        self.inner.failed_requests
    }

    #[getter]
    pub fn avg_wait_time_ms(&self) -> f64 {
        self.inner.avg_wait_time_ms
    }

    #[getter]
    pub fn max_wait_time_ms(&self) -> u64 {
        self.inner.max_wait_time_ms
    }

    #[getter]
    pub fn success_rate(&self) -> f64 {
        if self.inner.total_requests == 0 {
            0.0
        } else {
            self.inner.successful_requests as f64 / self.inner.total_requests as f64
        }
    }

    #[getter]
    pub fn utilization(&self) -> f64 {
        if self.inner.total_connections == 0 {
            0.0
        } else {
            self.inner.active_connections as f64 / self.inner.total_connections as f64
        }
    }

    pub fn to_dict(&self) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        result.insert("total_connections".to_string(), self.inner.total_connections as f64);
        result.insert("active_connections".to_string(), self.inner.active_connections as f64);
        result.insert("idle_connections".to_string(), self.inner.idle_connections as f64);
        result.insert("total_requests".to_string(), self.inner.total_requests as f64);
        result.insert("successful_requests".to_string(), self.inner.successful_requests as f64);
        result.insert("failed_requests".to_string(), self.inner.failed_requests as f64);
        result.insert("avg_wait_time_ms".to_string(), self.inner.avg_wait_time_ms);
        result.insert("max_wait_time_ms".to_string(), self.inner.max_wait_time_ms as f64);
        result.insert("success_rate".to_string(), self.success_rate());
        result.insert("utilization".to_string(), self.utilization());
        result
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PoolStats(total_connections={}, active_connections={}, idle_connections={}, success_rate={:.2}%, utilization={:.2}%)",
            self.inner.total_connections,
            self.inner.active_connections,
            self.inner.idle_connections,
            self.success_rate() * 100.0,
            self.utilization() * 100.0
        )
    }
}

/// Python wrapper for ConnectionPool
#[pyclass(name = "ConnectionPool")]
pub struct PyConnectionPool {
    pub inner: Option<ConnectionPool>,
}

#[pymethods]
impl PyConnectionPool {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: None,
        }
    }

    #[staticmethod]
    #[pyo3(signature = (_database_path, config = None))]
    pub fn create_pool(
        _database_path: String,
        config: Option<PyPoolConfig>,
    ) -> Self {
        let _config = config.map(|c| c.inner).unwrap_or_default();
        
        // For now, create a simple pool without async initialization
        // In production, you would use an async method
        Self {
            inner: None, // Will be set by actual async initialization
        }
    }

    pub fn get_stats<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        if let Some(ref pool) = self.inner {
            let pool_clone = pool.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                Ok(PyPoolStats { inner: pool_clone.get_stats().await })
            })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection pool not initialized"))
        }
    }

    pub fn get_config(&self) -> PyResult<PyPoolConfig> {
        if let Some(ref pool) = self.inner {
            Ok(PyPoolConfig {
                inner: pool.get_config().clone(),
            })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection pool not initialized"))
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }

    pub fn __repr__(&self) -> String {
        if let Some(ref _pool) = self.inner {
            "ConnectionPool(initialized=True)".to_string()
        } else {
            "ConnectionPool(initialized=False)".to_string()
        }
    }
}

/// Performance benchmark utilities
#[pyfunction]
#[pyo3(signature = (database_path, config = None, num_operations = 1000, concurrency = 10))]
pub fn benchmark_connection_pool<'py>(
    py: Python<'py>,
    database_path: String,
    config: Option<PyPoolConfig>,
    num_operations: usize,
    concurrency: usize,
) -> PyResult<&'py PyAny> {
    let config = config.map(|c| c.inner).unwrap_or_default();
    
    pyo3_asyncio::tokio::future_into_py(py, async move {
        match benchmark_pool_performance(database_path, config, num_operations, concurrency).await {
            Ok(result) => Ok(result),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
        }
    })
}

#[pyfunction]
#[pyo3(signature = (database_path, configs, num_operations = 1000))]
pub fn compare_pool_configurations<'py>(
    py: Python<'py>,
    database_path: String,
    configs: Vec<PyPoolConfig>,
    num_operations: usize,
) -> PyResult<&'py PyAny> {
    let configs: Vec<PoolConfig> = configs.into_iter().map(|c| c.inner).collect();
    
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let mut results = Vec::new();
        
        for config in configs {
            match benchmark_pool_performance(database_path.clone(), config, num_operations, 10).await {
                Ok(result) => results.push(result),
                Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e))),
            }
        }
        
        Ok(results)
    })
}

async fn benchmark_pool_performance(
    database_path: String,
    config: PoolConfig,
    num_operations: usize,
    concurrency: usize,
) -> Result<HashMap<String, f64>, eventuali_core::error::EventualiError> {
    use std::time::Instant;
    use tokio::task::JoinSet;
    
    let start_time = Instant::now();
    let pool = ConnectionPool::new(database_path, config).await?;
    let init_time = start_time.elapsed();
    
    // Warm up the pool
    for _ in 0..5 {
        let _guard = pool.get_connection().await?;
    }
    
    let benchmark_start = Instant::now();
    let mut join_set = JoinSet::new();
    let operations_per_task = num_operations / concurrency;
    
    for _ in 0..concurrency {
        let pool_clone = pool.clone();
        join_set.spawn(async move {
            let task_start = Instant::now();
            let mut successful_ops = 0;
            
            for _ in 0..operations_per_task {
                match pool_clone.get_connection().await {
                    Ok(guard) => {
                        // Perform a simple operation
                        if let Ok(conn) = guard.create_connection() {
                            if let Ok(_) = conn.execute("SELECT 1", []) {
                                successful_ops += 1;
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            
            (successful_ops, task_start.elapsed())
        });
    }
    
    let mut total_successful = 0;
    let mut total_task_time = std::time::Duration::ZERO;
    
    while let Some(result) = join_set.join_next().await {
        if let Ok((successful, task_time)) = result {
            total_successful += successful;
            total_task_time += task_time;
        }
    }
    
    let total_time = benchmark_start.elapsed();
    let final_stats = pool.get_stats().await;
    
    let mut results = HashMap::new();
    results.insert("init_time_ms".to_string(), init_time.as_millis() as f64);
    results.insert("total_time_ms".to_string(), total_time.as_millis() as f64);
    results.insert("avg_task_time_ms".to_string(), total_task_time.as_millis() as f64 / concurrency as f64);
    results.insert("operations_per_second".to_string(), total_successful as f64 / total_time.as_secs_f64());
    results.insert("successful_operations".to_string(), total_successful as f64);
    results.insert("success_rate".to_string(), total_successful as f64 / num_operations as f64);
    results.insert("final_total_connections".to_string(), final_stats.total_connections as f64);
    results.insert("final_avg_wait_time_ms".to_string(), final_stats.avg_wait_time_ms);
    results.insert("final_max_wait_time_ms".to_string(), final_stats.max_wait_time_ms as f64);
    
    Ok(results)
}

/// Register performance optimization Python module
pub fn register_performance_module(py: Python, m: &PyModule) -> PyResult<()> {
    let performance_module = PyModule::new(py, "performance")?;
    
    performance_module.add_class::<PyPoolConfig>()?;
    performance_module.add_class::<PyPoolStats>()?;
    performance_module.add_class::<PyConnectionPool>()?;
    performance_module.add_function(wrap_pyfunction!(benchmark_connection_pool, performance_module)?)?;
    performance_module.add_function(wrap_pyfunction!(compare_pool_configurations, performance_module)?)?;
    
    m.add_submodule(performance_module)?;
    Ok(())
}