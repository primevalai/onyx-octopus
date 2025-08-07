use pyo3::prelude::*;
use pyo3::exceptions;
use eventuali_core::EventualiError as CoreError;

/// Convert a Rust error to a Python exception
pub fn map_rust_error_to_python(error: CoreError) -> PyErr {
    match error {
        CoreError::Database(e) => {
            PyErr::new::<exceptions::PyRuntimeError, _>(format!("Database error: {}", e))
        }
        CoreError::Serialization(e) => {
            PyErr::new::<exceptions::PyValueError, _>(format!("Serialization error: {}", e))
        }
        CoreError::Protobuf(e) => {
            PyErr::new::<exceptions::PyValueError, _>(format!("Protobuf error: {}", e))
        }
        CoreError::AggregateNotFound { id } => {
            PyErr::new::<exceptions::PyKeyError, _>(format!("Aggregate not found: {}", id))
        }
        CoreError::OptimisticConcurrency { expected, actual } => {
            PyErr::new::<exceptions::PyRuntimeError, _>(format!(
                "Optimistic concurrency error: expected version {}, got {}",
                expected, actual
            ))
        }
        CoreError::InvalidEventData(msg) => {
            PyErr::new::<exceptions::PyValueError, _>(format!("Invalid event data: {}", msg))
        }
        CoreError::Configuration(msg) => {
            PyErr::new::<exceptions::PyRuntimeError, _>(format!("Configuration error: {}", msg))
        }
        CoreError::Io(e) => {
            PyErr::new::<exceptions::PyIOError, _>(format!("IO error: {}", e))
        }
    }
}

pub fn register_exceptions(_py: Python, _m: &PyModule) -> PyResult<()> {
    // Simplified - just use built-in exceptions for now
    Ok(())
}