#![allow(non_local_definitions)]
use pyo3::prelude::*;

mod event_store;
mod event;
mod aggregate;
mod error;
mod streaming;
mod snapshot;
mod security;
mod tenancy;
mod performance;

#[cfg(feature = "observability")]
mod observability;

use event_store::PyEventStore;
use event::PyEvent;
use aggregate::PyAggregate;
use streaming::{PyEventStreamer, PyEventStreamReceiver, PySubscriptionBuilder, PyProjection};
use snapshot::{PySnapshotService, PySnapshotConfig, PyAggregateSnapshot};
use security::{PyEventEncryption, PyKeyManager, PyEncryptionKey, PyEncryptedEventData, PyEncryptionAlgorithm, PySecurityUtils};
use tenancy::{PyTenantId, PyTenantInfo, PyTenantConfig, PyTenantMetadata, PyResourceLimits, PyTenantManager, PyTenantStorageMetrics};

#[pymodule]
fn _eventuali(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEventStore>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyAggregate>()?;
    
    // Register streaming classes
    m.add_class::<PyEventStreamer>()?;
    m.add_class::<PyEventStreamReceiver>()?;
    m.add_class::<PySubscriptionBuilder>()?;
    m.add_class::<PyProjection>()?;
    
    // Register snapshot classes
    m.add_class::<PySnapshotService>()?;
    m.add_class::<PySnapshotConfig>()?;
    m.add_class::<PyAggregateSnapshot>()?;
    
    // Register security classes
    m.add_class::<PyEventEncryption>()?;
    m.add_class::<PyKeyManager>()?;
    m.add_class::<PyEncryptionKey>()?;
    m.add_class::<PyEncryptedEventData>()?;
    m.add_class::<PyEncryptionAlgorithm>()?;
    m.add_class::<PySecurityUtils>()?;
    
    // Register tenancy classes
    m.add_class::<PyTenantId>()?;
    m.add_class::<PyTenantInfo>()?;
    m.add_class::<PyTenantConfig>()?;
    m.add_class::<PyTenantMetadata>()?;
    m.add_class::<PyResourceLimits>()?;
    m.add_class::<PyTenantManager>()?;
    m.add_class::<PyTenantStorageMetrics>()?;
    
    // Register custom exceptions
    error::register_exceptions(py, m)?;
    
    // Register observability classes if the feature is enabled
    #[cfg(feature = "observability")]
    observability::register_observability_classes(py, m)?;
    
    // Register performance optimization classes
    performance::register_performance_module(py, m)?;
    
    Ok(())
}