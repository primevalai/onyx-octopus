use pyo3::prelude::*;
use pyo3::types::PyBytes;

use eventuali_core::{
    AggregateSnapshot, SnapshotService, SnapshotConfig, 
    SnapshotCompression, SqliteSnapshotStore
};

/// Python wrapper for AggregateSnapshot
#[pyclass(name = "AggregateSnapshot")]
#[derive(Clone)]
pub struct PyAggregateSnapshot {
    inner: AggregateSnapshot,
}

#[pymethods]
impl PyAggregateSnapshot {
    #[getter]
    fn snapshot_id(&self) -> String {
        self.inner.snapshot_id.to_string()
    }

    #[getter]
    fn aggregate_id(&self) -> &str {
        &self.inner.aggregate_id
    }

    #[getter]
    fn aggregate_type(&self) -> &str {
        &self.inner.aggregate_type
    }

    #[getter]
    fn aggregate_version(&self) -> i64 {
        self.inner.aggregate_version
    }

    #[getter]
    fn state_data<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        Ok(PyBytes::new(py, &self.inner.state_data))
    }

    #[getter]
    fn compression(&self) -> String {
        match self.inner.compression {
            SnapshotCompression::None => "none".to_string(),
            SnapshotCompression::Gzip => "gzip".to_string(),
            SnapshotCompression::Lz4 => "lz4".to_string(),
        }
    }

    #[getter]
    fn created_at(&self) -> String {
        self.inner.created_at.to_rfc3339()
    }

    #[getter]
    fn original_size(&self) -> usize {
        self.inner.metadata.original_size
    }

    #[getter]
    fn compressed_size(&self) -> usize {
        self.inner.metadata.compressed_size
    }

    #[getter]
    fn event_count(&self) -> usize {
        self.inner.metadata.event_count
    }

    #[getter]
    fn checksum(&self) -> &str {
        &self.inner.metadata.checksum
    }

    fn __repr__(&self) -> String {
        format!(
            "AggregateSnapshot(id={}, aggregate_id={}, version={}, size={})",
            self.inner.snapshot_id,
            self.inner.aggregate_id,
            self.inner.aggregate_version,
            self.inner.metadata.compressed_size
        )
    }
}

impl From<AggregateSnapshot> for PyAggregateSnapshot {
    fn from(inner: AggregateSnapshot) -> Self {
        Self { inner }
    }
}

/// Python wrapper for SnapshotConfig
#[pyclass(name = "SnapshotConfig")]
#[derive(Clone)]
pub struct PySnapshotConfig {
    inner: SnapshotConfig,
}

#[pymethods]
impl PySnapshotConfig {
    #[new]
    #[pyo3(signature = (snapshot_frequency=100, max_snapshot_age_hours=168, compression="gzip", auto_cleanup=true))]
    fn new(
        snapshot_frequency: i64,
        max_snapshot_age_hours: u64,
        compression: &str,
        auto_cleanup: bool,
    ) -> PyResult<Self> {
        let compression_enum = match compression {
            "none" => SnapshotCompression::None,
            "gzip" => SnapshotCompression::Gzip,
            "lz4" => SnapshotCompression::Lz4,
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Unknown compression type: {}", compression)
            )),
        };

        Ok(Self {
            inner: SnapshotConfig {
                snapshot_frequency,
                max_snapshot_age_hours,
                compression: compression_enum,
                auto_cleanup,
            }
        })
    }

    #[getter]
    fn snapshot_frequency(&self) -> i64 {
        self.inner.snapshot_frequency
    }

    #[getter]
    fn max_snapshot_age_hours(&self) -> u64 {
        self.inner.max_snapshot_age_hours
    }

    #[getter]
    fn compression(&self) -> String {
        match self.inner.compression {
            SnapshotCompression::None => "none".to_string(),
            SnapshotCompression::Gzip => "gzip".to_string(),
            SnapshotCompression::Lz4 => "lz4".to_string(),
        }
    }

    #[getter]
    fn auto_cleanup(&self) -> bool {
        self.inner.auto_cleanup
    }

    fn __repr__(&self) -> String {
        format!(
            "SnapshotConfig(frequency={}, max_age={}h, compression={})",
            self.inner.snapshot_frequency,
            self.inner.max_snapshot_age_hours,
            self.compression()
        )
    }
}

/// Python wrapper for SnapshotService with SQLite backend
#[pyclass(name = "SnapshotService")]
pub struct PySnapshotService {
    inner: Option<SnapshotService<SqliteSnapshotStore>>,
}

#[pymethods]
impl PySnapshotService {
    #[new]
    fn new() -> Self {
        Self { inner: None }
    }

    /// Initialize the snapshot service with SQLite database
    fn initialize(&mut self, database_url: &str, config: &PySnapshotConfig) -> PyResult<()> {
        pyo3_asyncio::tokio::get_runtime()
            .block_on(async {
                // Create database pool
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(10)
                    .connect(database_url)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Database error: {}", e)))?;

                // Create and initialize snapshot store
                let store = SqliteSnapshotStore::new(pool, None);
                store.initialize().await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Database error: {}", e)))?;

                // Create snapshot service
                let service = SnapshotService::new(store, config.inner.clone());
                self.inner = Some(service);

                Ok(())
            })
    }

    /// Create a snapshot from aggregate state data
    fn create_snapshot(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        aggregate_version: i64,
        state_data: &[u8],
        event_count: usize,
    ) -> PyResult<PyAggregateSnapshot> {
        let service = self.inner.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("SnapshotService not initialized")
        })?;

        pyo3_asyncio::tokio::get_runtime()
            .block_on(async {
                let snapshot = service.create_snapshot(
                    aggregate_id.to_string(),
                    aggregate_type.to_string(),
                    aggregate_version,
                    state_data.to_vec(),
                    event_count,
                ).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Database error: {}", e)))?;

                Ok(PyAggregateSnapshot::from(snapshot))
            })
    }

    /// Load the most recent snapshot for an aggregate
    fn load_latest_snapshot(&self, aggregate_id: &str) -> PyResult<Option<PyAggregateSnapshot>> {
        let service = self.inner.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("SnapshotService not initialized")
        })?;

        pyo3_asyncio::tokio::get_runtime()
            .block_on(async {
                let snapshot = service.load_latest_snapshot(&aggregate_id.to_string())
                    .await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Database error: {}", e)))?;

                Ok(snapshot.map(PyAggregateSnapshot::from))
            })
    }

    /// Decompress snapshot data
    fn decompress_snapshot_data(&self, snapshot: &PyAggregateSnapshot) -> PyResult<Vec<u8>> {
        let service = self.inner.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("SnapshotService not initialized")
        })?;

        let decompressed = service.decompress_snapshot_data(&snapshot.inner)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Serialization error: {}", e)))?;

        Ok(decompressed)
    }

    /// Check if a snapshot should be taken
    fn should_take_snapshot(&self, aggregate_id: &str, current_version: i64) -> PyResult<bool> {
        let service = self.inner.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("SnapshotService not initialized")
        })?;

        pyo3_asyncio::tokio::get_runtime()
            .block_on(async {
                let should_take = service.should_take_snapshot(&aggregate_id.to_string(), current_version)
                    .await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Database error: {}", e)))?;

                Ok(should_take)
            })
    }

    /// Perform cleanup of old snapshots
    fn cleanup_old_snapshots(&self) -> PyResult<u64> {
        let service = self.inner.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("SnapshotService not initialized")
        })?;

        pyo3_asyncio::tokio::get_runtime()
            .block_on(async {
                let cleaned_count = service.cleanup_old_snapshots()
                    .await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Database error: {}", e)))?;

                Ok(cleaned_count)
            })
    }

    fn __repr__(&self) -> String {
        if self.inner.is_some() {
            "SnapshotService(initialized)".to_string()
        } else {
            "SnapshotService(not initialized)".to_string()
        }
    }
}