use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyo3::exceptions::PyRuntimeError;
use eventuali_core::tenancy::{
    TenantId as CoreTenantId, TenantInfo as CoreTenantInfo, TenantConfig as CoreTenantConfig,
    TenantMetadata as CoreTenantMetadata, ResourceLimits as CoreResourceLimits,
    TenantManager as CoreTenantManager
};
use crate::error::map_rust_error_to_python;
use std::collections::HashMap;
use std::sync::Arc;

/// Python wrapper for TenantId
#[pyclass(name = "TenantId")]
#[derive(Clone)]
pub struct PyTenantId {
    inner: CoreTenantId,
}

#[pymethods]
impl PyTenantId {
    #[new]
    fn new(id: String) -> PyResult<Self> {
        let tenant_id = CoreTenantId::new(id)
            .map_err(|e| PyRuntimeError::new_err(format!("Tenant ID error: {}", e)))?;
        Ok(Self { inner: tenant_id })
    }
    
    #[classmethod]
    fn generate(_cls: &PyType) -> Self {
        Self {
            inner: CoreTenantId::generate(),
        }
    }
    
    fn as_str(&self) -> &str {
        self.inner.as_str()
    }
    
    fn db_prefix(&self) -> String {
        self.inner.db_prefix()
    }
    
    fn __str__(&self) -> String {
        self.inner.to_string()
    }
    
    fn __repr__(&self) -> String {
        format!("TenantId('{}')", self.inner.as_str())
    }
}

/// Python wrapper for ResourceLimits
#[pyclass(name = "ResourceLimits")]
#[derive(Clone)]
pub struct PyResourceLimits {
    inner: CoreResourceLimits,
}

#[pymethods]
impl PyResourceLimits {
    #[new]
    fn new(
        max_events_per_day: Option<u64>,
        max_storage_mb: Option<u64>,
        max_concurrent_streams: Option<u32>,
        max_projections: Option<u32>,
        max_aggregates: Option<u64>,
    ) -> Self {
        Self {
            inner: CoreResourceLimits {
                max_events_per_day,
                max_storage_mb,
                max_concurrent_streams,
                max_projections,
                max_aggregates,
            },
        }
    }
    
    #[classmethod]
    fn default(_cls: &PyType) -> Self {
        Self {
            inner: CoreResourceLimits::default(),
        }
    }
    
    #[getter]
    fn max_events_per_day(&self) -> Option<u64> {
        self.inner.max_events_per_day
    }
    
    #[setter]
    fn set_max_events_per_day(&mut self, value: Option<u64>) {
        self.inner.max_events_per_day = value;
    }
    
    #[getter]
    fn max_storage_mb(&self) -> Option<u64> {
        self.inner.max_storage_mb
    }
    
    #[setter]
    fn set_max_storage_mb(&mut self, value: Option<u64>) {
        self.inner.max_storage_mb = value;
    }
    
    #[getter]
    fn max_concurrent_streams(&self) -> Option<u32> {
        self.inner.max_concurrent_streams
    }
    
    #[setter]
    fn set_max_concurrent_streams(&mut self, value: Option<u32>) {
        self.inner.max_concurrent_streams = value;
    }
    
    #[getter]
    fn max_projections(&self) -> Option<u32> {
        self.inner.max_projections
    }
    
    #[setter]
    fn set_max_projections(&mut self, value: Option<u32>) {
        self.inner.max_projections = value;
    }
    
    #[getter]
    fn max_aggregates(&self) -> Option<u64> {
        self.inner.max_aggregates
    }
    
    #[setter]
    fn set_max_aggregates(&mut self, value: Option<u64>) {
        self.inner.max_aggregates = value;
    }
}

/// Python wrapper for TenantConfig
#[pyclass(name = "TenantConfig")]
#[derive(Clone)]
pub struct PyTenantConfig {
    inner: CoreTenantConfig,
}

#[pymethods]
impl PyTenantConfig {
    #[new]
    fn new(
        isolation_level: Option<String>,
        resource_limits: Option<PyResourceLimits>,
        encryption_enabled: Option<bool>,
        audit_enabled: Option<bool>,
        custom_settings: Option<HashMap<String, String>>,
    ) -> Self {
        let isolation_level = match isolation_level.as_deref() {
            Some("database") => eventuali_core::tenancy::tenant::IsolationLevel::Database,
            Some("application") => eventuali_core::tenancy::tenant::IsolationLevel::Application,
            Some("row") => eventuali_core::tenancy::tenant::IsolationLevel::Row,
            _ => eventuali_core::tenancy::tenant::IsolationLevel::Database,
        };
        
        Self {
            inner: CoreTenantConfig {
                isolation_level,
                resource_limits: resource_limits.map(|rl| rl.inner).unwrap_or_default(),
                encryption_enabled: encryption_enabled.unwrap_or(true),
                audit_enabled: audit_enabled.unwrap_or(true),
                custom_settings: custom_settings.unwrap_or_default(),
            },
        }
    }
    
    #[classmethod]
    fn default(_cls: &PyType) -> Self {
        Self {
            inner: CoreTenantConfig::default(),
        }
    }
    
    #[getter]
    fn isolation_level(&self) -> String {
        match self.inner.isolation_level {
            eventuali_core::tenancy::tenant::IsolationLevel::Database => "database".to_string(),
            eventuali_core::tenancy::tenant::IsolationLevel::Application => "application".to_string(),
            eventuali_core::tenancy::tenant::IsolationLevel::Row => "row".to_string(),
        }
    }
    
    #[getter]
    fn resource_limits(&self) -> PyResourceLimits {
        PyResourceLimits {
            inner: self.inner.resource_limits.clone(),
        }
    }
    
    #[getter]
    fn encryption_enabled(&self) -> bool {
        self.inner.encryption_enabled
    }
    
    #[getter]
    fn audit_enabled(&self) -> bool {
        self.inner.audit_enabled
    }
    
    #[getter]
    fn custom_settings(&self) -> HashMap<String, String> {
        self.inner.custom_settings.clone()
    }
}

/// Python wrapper for TenantMetadata
#[pyclass(name = "TenantMetadata")]
#[derive(Clone)]
pub struct PyTenantMetadata {
    inner: CoreTenantMetadata,
}

#[pymethods]
impl PyTenantMetadata {
    #[getter]
    fn total_events(&self) -> u64 {
        self.inner.total_events
    }
    
    #[getter]
    fn total_aggregates(&self) -> u64 {
        self.inner.total_aggregates
    }
    
    #[getter]
    fn storage_used_mb(&self) -> f64 {
        self.inner.storage_used_mb
    }
    
    #[getter]
    fn last_activity(&self) -> Option<String> {
        self.inner.last_activity.map(|dt| dt.to_rfc3339())
    }
    
    #[getter]
    fn average_response_time_ms(&self) -> f64 {
        self.inner.performance_metrics.average_response_time_ms
    }
    
    #[getter]
    fn events_per_second(&self) -> f64 {
        self.inner.performance_metrics.events_per_second
    }
    
    #[getter]
    fn error_rate(&self) -> f64 {
        self.inner.performance_metrics.error_rate
    }
    
    #[getter]
    fn uptime_percentage(&self) -> f64 {
        self.inner.performance_metrics.uptime_percentage
    }
}

/// Python wrapper for TenantInfo
#[pyclass(name = "TenantInfo")]
#[derive(Clone)]
pub struct PyTenantInfo {
    inner: CoreTenantInfo,
}

#[pymethods]
impl PyTenantInfo {
    #[getter]
    fn id(&self) -> PyTenantId {
        PyTenantId {
            inner: self.inner.id.clone(),
        }
    }
    
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }
    
    #[getter]
    fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }
    
    #[getter]
    fn created_at(&self) -> String {
        self.inner.created_at.to_rfc3339()
    }
    
    #[getter]
    fn updated_at(&self) -> String {
        self.inner.updated_at.to_rfc3339()
    }
    
    #[getter]
    fn status(&self) -> String {
        match self.inner.status {
            eventuali_core::tenancy::tenant::TenantStatus::Active => "active".to_string(),
            eventuali_core::tenancy::tenant::TenantStatus::Suspended => "suspended".to_string(),
            eventuali_core::tenancy::tenant::TenantStatus::Disabled => "disabled".to_string(),
            eventuali_core::tenancy::tenant::TenantStatus::PendingDeletion => "pending_deletion".to_string(),
        }
    }
    
    #[getter]
    fn config(&self) -> PyTenantConfig {
        PyTenantConfig {
            inner: self.inner.config.clone(),
        }
    }
    
    #[getter]
    fn metadata(&self) -> PyTenantMetadata {
        PyTenantMetadata {
            inner: self.inner.metadata.clone(),
        }
    }
    
    fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}

/// Python wrapper for TenantManager
#[pyclass(name = "TenantManager")]
pub struct PyTenantManager {
    inner: Arc<CoreTenantManager>,
}

#[pymethods]
impl PyTenantManager {
    #[new]
    fn new() -> Self {
        Self {
            inner: Arc::new(CoreTenantManager::new()),
        }
    }
    
    fn create_tenant(
        &self, 
        tenant_id: PyTenantId, 
        name: String, 
        config: Option<PyTenantConfig>
    ) -> PyResult<PyTenantInfo> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            self.inner.create_tenant(
                tenant_id.inner,
                name,
                config.map(|c| c.inner)
            ).await
        });
        
        match result {
            Ok(tenant_info) => Ok(PyTenantInfo { inner: tenant_info }),
            Err(e) => Err(map_rust_error_to_python(e)),
        }
    }
    
    fn get_tenant(&self, tenant_id: PyTenantId) -> PyResult<PyTenantInfo> {
        match self.inner.get_tenant(&tenant_id.inner) {
            Ok(tenant_info) => Ok(PyTenantInfo { inner: tenant_info }),
            Err(e) => Err(map_rust_error_to_python(e)),
        }
    }
    
    fn list_tenants(&self, status_filter: Option<String>) -> PyResult<Vec<PyTenantInfo>> {
        let status = match status_filter.as_deref() {
            Some("active") => Some(eventuali_core::tenancy::tenant::TenantStatus::Active),
            Some("suspended") => Some(eventuali_core::tenancy::tenant::TenantStatus::Suspended),
            Some("disabled") => Some(eventuali_core::tenancy::tenant::TenantStatus::Disabled),
            Some("pending_deletion") => Some(eventuali_core::tenancy::tenant::TenantStatus::PendingDeletion),
            _ => None,
        };
        
        let tenants = self.inner.list_tenants(status);
        Ok(tenants.into_iter()
            .map(|t| PyTenantInfo { inner: t })
            .collect())
    }
    
    fn delete_tenant(&self, tenant_id: PyTenantId) -> PyResult<()> {
        self.inner.delete_tenant(&tenant_id.inner)
            .map_err(map_rust_error_to_python)
    }
    
    fn get_tenant_usage(&self, tenant_id: PyTenantId) -> PyResult<Py<PyDict>> {
        match self.inner.get_tenant_usage(&tenant_id.inner) {
            Ok(usage) => {
                Python::with_gil(|py| {
                    let dict = PyDict::new(py);
                    dict.set_item("tenant_id", tenant_id.as_str())?;
                    dict.set_item("daily_events", usage.daily_events)?;
                    dict.set_item("storage_used_mb", usage.storage_used_mb)?;
                    dict.set_item("concurrent_streams", usage.concurrent_streams)?;
                    dict.set_item("total_projections", usage.total_projections)?;
                    dict.set_item("total_aggregates", usage.total_aggregates)?;
                    dict.set_item("last_updated", usage.last_updated.to_rfc3339())?;
                    Ok(dict.into_py(py))
                })
            }
            Err(e) => Err(map_rust_error_to_python(e)),
        }
    }
    
    fn check_tenant_quota(
        &self, 
        tenant_id: PyTenantId, 
        resource_type: String, 
        amount: u64
    ) -> PyResult<()> {
        let resource_type = match resource_type.as_str() {
            "events" => eventuali_core::tenancy::quota::ResourceType::Events,
            "storage" => eventuali_core::tenancy::quota::ResourceType::Storage,
            "streams" => eventuali_core::tenancy::quota::ResourceType::Streams,
            "projections" => eventuali_core::tenancy::quota::ResourceType::Projections,
            "aggregates" => eventuali_core::tenancy::quota::ResourceType::Aggregates,
            _ => return Err(PyRuntimeError::new_err(format!("Invalid resource type: {}", resource_type))),
        };
        
        self.inner.check_tenant_quota(&tenant_id.inner, resource_type, amount)
            .map_err(map_rust_error_to_python)
    }
    
    fn record_tenant_usage(
        &self, 
        tenant_id: PyTenantId, 
        resource_type: String, 
        amount: u64
    ) -> PyResult<()> {
        let resource_type = match resource_type.as_str() {
            "events" => eventuali_core::tenancy::quota::ResourceType::Events,
            "storage" => eventuali_core::tenancy::quota::ResourceType::Storage,
            "streams" => eventuali_core::tenancy::quota::ResourceType::Streams,
            "projections" => eventuali_core::tenancy::quota::ResourceType::Projections,
            "aggregates" => eventuali_core::tenancy::quota::ResourceType::Aggregates,
            _ => return Err(PyRuntimeError::new_err(format!("Invalid resource type: {}", resource_type))),
        };
        
        self.inner.record_tenant_usage(&tenant_id.inner, resource_type, amount)
            .map_err(map_rust_error_to_python)
    }
    
    fn get_tenants_near_limits(&self) -> Vec<Py<PyDict>> {
        let tenants = self.inner.get_tenants_near_limits();
        
        Python::with_gil(|py| {
            tenants.into_iter()
                .filter_map(|(tenant_id, usage)| {
                    let dict = PyDict::new(py);
                    dict.set_item("tenant_id", tenant_id.as_str()).ok()?;
                    dict.set_item("daily_events", usage.daily_events).ok()?;
                    dict.set_item("storage_used_mb", usage.storage_used_mb).ok()?;
                    dict.set_item("concurrent_streams", usage.concurrent_streams).ok()?;
                    dict.set_item("total_projections", usage.total_projections).ok()?;
                    dict.set_item("total_aggregates", usage.total_aggregates).ok()?;
                    dict.set_item("last_updated", usage.last_updated.to_rfc3339()).ok()?;
                    Some(dict.into_py(py))
                })
                .collect()
        })
    }
    
    fn get_isolation_metrics(&self) -> Py<PyDict> {
        let metrics = self.inner.get_isolation_metrics();
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("total_validations", metrics.total_validations).unwrap();
            dict.set_item("successful_validations", metrics.successful_validations).unwrap();
            dict.set_item("average_validation_time_ms", metrics.average_validation_time_ms).unwrap();
            dict.set_item("max_validation_time_ms", metrics.max_validation_time_ms).unwrap();
            dict.set_item("violations_detected", metrics.violations_detected).unwrap();
            dict.set_item("isolation_success_rate", metrics.isolation_success_rate()).unwrap();
            dict.set_item("is_performance_target_met", metrics.is_performance_target_met()).unwrap();
            dict.set_item("last_updated", metrics.last_updated.to_rfc3339()).unwrap();
            dict.into_py(py)
        })
    }
}