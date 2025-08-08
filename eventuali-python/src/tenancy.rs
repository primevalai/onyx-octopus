use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyo3::exceptions::PyRuntimeError;
use eventuali_core::tenancy::{
    TenantId as CoreTenantId, TenantInfo as CoreTenantInfo, TenantConfig as CoreTenantConfig,
    TenantMetadata as CoreTenantMetadata, ResourceLimits as CoreResourceLimits,
    TenantManager as CoreTenantManager,
    TenantStorageMetrics as CoreTenantStorageMetrics,
    ResourceType as CoreResourceType, QuotaTier as CoreQuotaTier, QuotaCheckResult as CoreQuotaCheckResult,
    EnhancedResourceUsage as CoreEnhancedResourceUsage, QuotaAlert as CoreQuotaAlert,
    AlertType as CoreAlertType, BillingAnalytics as CoreBillingAnalytics,
    TenantConfigurationManager as CoreTenantConfigurationManager, ConfigurationValue as CoreConfigurationValue,
    ConfigurationEnvironment as CoreConfigurationEnvironment, ConfigurationSchema as CoreConfigurationSchema,
    TenantMetricsCollector as CoreTenantMetricsCollector, MetricDataPoint as CoreMetricDataPoint,
    TenantHealthScore as CoreTenantHealthScore, HealthStatus as CoreHealthStatus
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
            .map_err(|e| PyRuntimeError::new_err(format!("Tenant ID error: {e}")))?;
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
            _ => return Err(PyRuntimeError::new_err(format!("Invalid resource type: {resource_type}"))),
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
            _ => return Err(PyRuntimeError::new_err(format!("Invalid resource type: {resource_type}"))),
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

/// Python wrapper for TenantStorageMetrics
#[pyclass(name = "TenantStorageMetrics")]
#[derive(Clone)]
pub struct PyTenantStorageMetrics {
    inner: CoreTenantStorageMetrics,
}

#[pymethods]
impl PyTenantStorageMetrics {
    #[getter]
    fn tenant_id(&self) -> PyTenantId {
        PyTenantId {
            inner: self.inner.tenant_id.clone(),
        }
    }
    
    #[getter]
    fn total_save_operations(&self) -> u64 {
        self.inner.total_save_operations
    }
    
    #[getter]
    fn total_load_operations(&self) -> u64 {
        self.inner.total_load_operations
    }
    
    #[getter]
    fn total_events_saved(&self) -> u64 {
        self.inner.total_events_saved
    }
    
    #[getter]
    fn total_events_loaded(&self) -> u64 {
        self.inner.total_events_loaded
    }
    
    #[getter]
    fn successful_saves(&self) -> u64 {
        self.inner.successful_saves
    }
    
    #[getter]
    fn successful_loads(&self) -> u64 {
        self.inner.successful_loads
    }
    
    #[getter]
    fn average_save_time_ms(&self) -> f64 {
        self.inner.average_save_time_ms
    }
    
    #[getter]
    fn average_load_time_ms(&self) -> f64 {
        self.inner.average_load_time_ms
    }
    
    #[getter]
    fn max_save_time_ms(&self) -> f64 {
        self.inner.max_save_time_ms
    }
    
    #[getter]
    fn max_load_time_ms(&self) -> f64 {
        self.inner.max_load_time_ms
    }
    
    #[getter]
    fn last_operation(&self) -> Option<String> {
        self.inner.last_operation.map(|dt| dt.to_rfc3339())
    }
    
    #[getter]
    fn operations_by_type(&self) -> HashMap<String, u64> {
        self.inner.operations_by_type.clone()
    }
    
    fn save_success_rate(&self) -> f64 {
        self.inner.save_success_rate()
    }
    
    fn load_success_rate(&self) -> f64 {
        self.inner.load_success_rate()
    }
    
    fn is_performance_target_met(&self) -> bool {
        self.inner.is_performance_target_met()
    }
    
    fn __str__(&self) -> String {
        format!(
            "TenantStorageMetrics(saves={}, loads={}, save_success_rate={:.1}%, load_success_rate={:.1}%)",
            self.inner.total_save_operations,
            self.inner.total_load_operations,
            self.inner.save_success_rate(),
            self.inner.load_success_rate()
        )
    }
}

/// Python wrapper for QuotaTier
#[pyclass(name = "QuotaTier")]
#[derive(Clone)]
pub struct PyQuotaTier {
    inner: CoreQuotaTier,
}

#[pymethods]
impl PyQuotaTier {
    #[new]
    fn new(tier: String) -> PyResult<Self> {
        let tier = match tier.as_str() {
            "starter" => CoreQuotaTier::Starter,
            "standard" => CoreQuotaTier::Standard,
            "professional" => CoreQuotaTier::Professional,
            "enterprise" => CoreQuotaTier::Enterprise,
            _ => return Err(PyRuntimeError::new_err(format!("Invalid quota tier: {tier}"))),
        };
        
        Ok(Self { inner: tier })
    }
    
    #[classmethod]
    fn starter(_cls: &PyType) -> Self {
        Self { inner: CoreQuotaTier::Starter }
    }
    
    #[classmethod]
    fn standard(_cls: &PyType) -> Self {
        Self { inner: CoreQuotaTier::Standard }
    }
    
    #[classmethod]
    fn professional(_cls: &PyType) -> Self {
        Self { inner: CoreQuotaTier::Professional }
    }
    
    #[classmethod]
    fn enterprise(_cls: &PyType) -> Self {
        Self { inner: CoreQuotaTier::Enterprise }
    }
    
    fn __str__(&self) -> String {
        match self.inner {
            CoreQuotaTier::Starter => "starter".to_string(),
            CoreQuotaTier::Standard => "standard".to_string(),
            CoreQuotaTier::Professional => "professional".to_string(),
            CoreQuotaTier::Enterprise => "enterprise".to_string(),
        }
    }
    
    fn __repr__(&self) -> String {
        format!("QuotaTier('{}')", self.__str__())
    }
}

/// Python wrapper for AlertType
#[pyclass(name = "AlertType")]
#[derive(Clone)]
pub struct PyAlertType {
    inner: CoreAlertType,
}

#[pymethods]
impl PyAlertType {
    #[new]
    fn new(alert_type: String) -> PyResult<Self> {
        let alert_type = match alert_type.as_str() {
            "warning" => CoreAlertType::Warning,
            "critical" => CoreAlertType::Critical,
            "exceeded" => CoreAlertType::Exceeded,
            "violation" => CoreAlertType::Violation,
            _ => return Err(PyRuntimeError::new_err(format!("Invalid alert type: {alert_type}"))),
        };
        
        Ok(Self { inner: alert_type })
    }
    
    fn __str__(&self) -> String {
        match self.inner {
            CoreAlertType::Warning => "warning".to_string(),
            CoreAlertType::Critical => "critical".to_string(),
            CoreAlertType::Exceeded => "exceeded".to_string(),
            CoreAlertType::Violation => "violation".to_string(),
        }
    }
}

/// Python wrapper for QuotaCheckResult
#[pyclass(name = "QuotaCheckResult")]
#[derive(Clone)]
pub struct PyQuotaCheckResult {
    inner: CoreQuotaCheckResult,
}

#[pymethods]
impl PyQuotaCheckResult {
    #[getter]
    fn allowed(&self) -> bool {
        self.inner.allowed
    }
    
    #[getter]
    fn current_usage(&self) -> u64 {
        self.inner.current_usage
    }
    
    #[getter]
    fn limit(&self) -> Option<u64> {
        self.inner.limit
    }
    
    #[getter]
    fn utilization_percentage(&self) -> f64 {
        self.inner.utilization_percentage
    }
    
    #[getter]
    fn grace_period_active(&self) -> bool {
        self.inner.grace_period_active
    }
    
    #[getter]
    fn warning_triggered(&self) -> bool {
        self.inner.warning_triggered
    }
    
    #[getter]
    fn estimated_overage_cost(&self) -> f64 {
        self.inner.estimated_overage_cost
    }
    
    fn __str__(&self) -> String {
        format!(
            "QuotaCheckResult(allowed={}, utilization={:.1}%, overage_cost=${:.4})",
            self.inner.allowed,
            self.inner.utilization_percentage,
            self.inner.estimated_overage_cost
        )
    }
}

/// Python wrapper for QuotaAlert
#[pyclass(name = "QuotaAlert")]
#[derive(Clone)]
pub struct PyQuotaAlert {
    inner: CoreQuotaAlert,
}

#[pymethods]
impl PyQuotaAlert {
    #[getter]
    fn tenant_id(&self) -> PyTenantId {
        PyTenantId { inner: self.inner.tenant_id.clone() }
    }
    
    #[getter]
    fn resource_type(&self) -> String {
        format!("{:?}", self.inner.resource_type).to_lowercase()
    }
    
    #[getter]
    fn alert_type(&self) -> PyAlertType {
        PyAlertType { inner: self.inner.alert_type.clone() }
    }
    
    #[getter]
    fn current_usage(&self) -> u64 {
        self.inner.current_usage
    }
    
    #[getter]
    fn limit(&self) -> u64 {
        self.inner.limit
    }
    
    #[getter]
    fn utilization_percentage(&self) -> f64 {
        self.inner.utilization_percentage
    }
    
    #[getter]
    fn message(&self) -> String {
        self.inner.message.clone()
    }
    
    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }
    
    #[getter]
    fn acknowledged(&self) -> bool {
        self.inner.acknowledged
    }
    
    fn __str__(&self) -> String {
        format!(
            "QuotaAlert({} - {} at {:.1}%: {})",
            self.alert_type().__str__(),
            self.resource_type(),
            self.inner.utilization_percentage,
            self.inner.message
        )
    }
}

/// Python wrapper for BillingAnalytics
#[pyclass(name = "BillingAnalytics")]
#[derive(Clone)]
pub struct PyBillingAnalytics {
    inner: CoreBillingAnalytics,
}

#[pymethods]
impl PyBillingAnalytics {
    #[getter]
    fn tenant_id(&self) -> PyTenantId {
        PyTenantId { inner: self.inner.tenant_id.clone() }
    }
    
    #[getter]
    fn current_month_cost(&self) -> f64 {
        self.inner.current_month_cost
    }
    
    #[getter]
    fn projected_month_cost(&self) -> f64 {
        self.inner.projected_month_cost
    }
    
    #[getter]
    fn billing_period_start(&self) -> String {
        self.inner.billing_period_start.to_rfc3339()
    }
    
    #[getter]
    fn billing_period_end(&self) -> String {
        self.inner.billing_period_end.to_rfc3339()
    }
    
    #[getter]
    fn overage_costs(&self) -> Py<PyDict> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (resource_type, cost) in &self.inner.overage_costs {
                let _ = dict.set_item(format!("{resource_type:?}").to_lowercase(), cost);
            }
            dict.into_py(py)
        })
    }
    
    #[getter]
    fn cost_breakdown(&self) -> Py<PyDict> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (resource_type, cost) in &self.inner.cost_breakdown {
                let _ = dict.set_item(format!("{resource_type:?}").to_lowercase(), cost);
            }
            dict.into_py(py)
        })
    }
    
    fn __str__(&self) -> String {
        format!(
            "BillingAnalytics(current=${:.2}, projected=${:.2}, overage_total=${:.2})",
            self.inner.current_month_cost,
            self.inner.projected_month_cost,
            self.inner.overage_costs.values().sum::<f64>()
        )
    }
}

/// Python wrapper for EnhancedResourceUsage
#[pyclass(name = "EnhancedResourceUsage")]
#[derive(Clone)]
pub struct PyEnhancedResourceUsage {
    inner: CoreEnhancedResourceUsage,
}

#[pymethods]
impl PyEnhancedResourceUsage {
    #[getter]
    fn tenant_id(&self) -> PyTenantId {
        PyTenantId { inner: self.inner.tenant_id.clone() }
    }
    
    #[getter]
    fn tier(&self) -> PyQuotaTier {
        PyQuotaTier { inner: self.inner.tier.clone() }
    }
    
    #[getter]
    fn daily_events(&self) -> u64 {
        self.inner.daily_events
    }
    
    #[getter]
    fn storage_used_mb(&self) -> f64 {
        self.inner.storage_used_mb
    }
    
    #[getter]
    fn concurrent_streams(&self) -> u32 {
        self.inner.concurrent_streams
    }
    
    #[getter]
    fn total_projections(&self) -> u32 {
        self.inner.total_projections
    }
    
    #[getter]
    fn total_aggregates(&self) -> u64 {
        self.inner.total_aggregates
    }
    
    #[getter]
    fn api_calls_today(&self) -> u64 {
        self.inner.api_calls_today
    }
    
    #[getter]
    fn performance_score(&self) -> f64 {
        self.inner.performance_score
    }
    
    #[getter]
    fn last_updated(&self) -> String {
        self.inner.last_updated.to_rfc3339()
    }
    
    #[getter]
    fn resource_limits(&self) -> PyResourceLimits {
        PyResourceLimits { inner: self.inner.limits.clone() }
    }
    
    #[getter]
    fn billing_analytics(&self) -> PyBillingAnalytics {
        PyBillingAnalytics { inner: self.inner.cost_analytics.clone() }
    }
    
    fn utilization_percentage(&self, resource_type: String) -> Option<f64> {
        let resource_type = match resource_type.as_str() {
            "events" => CoreResourceType::Events,
            "storage" => CoreResourceType::Storage,
            "streams" => CoreResourceType::Streams,
            "projections" => CoreResourceType::Projections,
            "aggregates" => CoreResourceType::Aggregates,
            "api_calls" => CoreResourceType::ApiCalls,
            _ => return None,
        };
        
        self.inner.utilization_percentage(resource_type)
    }
    
    fn has_resources_near_limit(&self) -> bool {
        self.inner.has_resources_near_limit()
    }
    
    fn get_alert_summary(&self) -> Py<PyDict> {
        let alert_summary = &self.inner.alert_summary;
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let _ = dict.set_item("total_alerts", alert_summary.total_alerts);
            let _ = dict.set_item("unacknowledged_alerts", alert_summary.unacknowledged_alerts);
            let _ = dict.set_item("critical_alerts", alert_summary.critical_alerts);
            let _ = dict.set_item("warning_alerts", alert_summary.warning_alerts);
            if let Some(last_alert) = alert_summary.last_alert {
                let _ = dict.set_item("last_alert", last_alert.to_rfc3339());
            }
            dict.into_py(py)
        })
    }
    
    fn __str__(&self) -> String {
        format!(
            "EnhancedResourceUsage(tier={}, events={}, storage={:.1}MB, score={:.1})",
            self.tier().__str__(),
            self.inner.daily_events,
            self.inner.storage_used_mb,
            self.inner.performance_score
        )
    }
}

/// Python wrapper for ConfigurationEnvironment
#[pyclass(name = "ConfigurationEnvironment")]
#[derive(Clone)]
pub struct PyConfigurationEnvironment {
    inner: CoreConfigurationEnvironment,
}

#[pymethods]
impl PyConfigurationEnvironment {
    #[new]
    fn new(env: String) -> PyResult<Self> {
        let environment = match env.as_str() {
            "development" => CoreConfigurationEnvironment::Development,
            "staging" => CoreConfigurationEnvironment::Staging,
            "production" => CoreConfigurationEnvironment::Production,
            "testing" => CoreConfigurationEnvironment::Testing,
            _ => return Err(PyRuntimeError::new_err(format!("Invalid environment: {env}"))),
        };
        
        Ok(Self { inner: environment })
    }
    
    #[classmethod]
    fn development(_cls: &PyType) -> Self {
        Self { inner: CoreConfigurationEnvironment::Development }
    }
    
    #[classmethod]
    fn staging(_cls: &PyType) -> Self {
        Self { inner: CoreConfigurationEnvironment::Staging }
    }
    
    #[classmethod]
    fn production(_cls: &PyType) -> Self {
        Self { inner: CoreConfigurationEnvironment::Production }
    }
    
    #[classmethod]
    fn testing(_cls: &PyType) -> Self {
        Self { inner: CoreConfigurationEnvironment::Testing }
    }
    
    fn __str__(&self) -> String {
        match self.inner {
            CoreConfigurationEnvironment::Development => "development".to_string(),
            CoreConfigurationEnvironment::Staging => "staging".to_string(),
            CoreConfigurationEnvironment::Production => "production".to_string(),
            CoreConfigurationEnvironment::Testing => "testing".to_string(),
        }
    }
}

/// Python wrapper for ConfigurationValue
#[pyclass(name = "ConfigurationValue")]
#[derive(Clone)]
pub struct PyConfigurationValue {
    inner: CoreConfigurationValue,
}

#[pymethods]
impl PyConfigurationValue {
    #[staticmethod]
    fn string(value: String) -> Self {
        Self {
            inner: CoreConfigurationValue::String(value)
        }
    }
    
    #[staticmethod]
    fn integer(value: i64) -> Self {
        Self {
            inner: CoreConfigurationValue::Integer(value)
        }
    }
    
    #[staticmethod]
    fn float(value: f64) -> Self {
        Self {
            inner: CoreConfigurationValue::Float(value)
        }
    }
    
    #[staticmethod]
    fn boolean(value: bool) -> Self {
        Self {
            inner: CoreConfigurationValue::Boolean(value)
        }
    }
    
    fn to_json(&self) -> PyResult<String> {
        let json_value = self.inner.to_json();
        serde_json::to_string(&json_value)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON serialization error: {e}")))
    }
    
    #[staticmethod]
    fn from_json(json_str: &str) -> PyResult<Self> {
        let json_value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON parsing error: {e}")))?;
        
        Ok(Self {
            inner: CoreConfigurationValue::from_json(&json_value)
        })
    }
    
    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// Python wrapper for TenantConfigurationManager
#[pyclass(name = "TenantConfigurationManager")]
pub struct PyTenantConfigurationManager {
    inner: CoreTenantConfigurationManager,
}

#[pymethods]
impl PyTenantConfigurationManager {
    #[new]
    fn new(tenant_id: PyTenantId) -> Self {
        Self {
            inner: CoreTenantConfigurationManager::new(tenant_id.inner)
        }
    }
    
    #[pyo3(signature = (key, value, changed_by, change_reason, environment=None))]
    fn set_configuration(
        &self,
        key: String,
        value: PyConfigurationValue,
        changed_by: String,
        change_reason: String,
        environment: Option<PyConfigurationEnvironment>,
    ) -> PyResult<()> {
        // Create a configuration value and schema from the Python value
        let (config_value, schema) = match &value.inner {
            CoreConfigurationValue::String(s) => (
                CoreConfigurationValue::String(s.clone()),
                CoreConfigurationSchema::String {
                    min_length: None,
                    max_length: None,
                    pattern: None,
                }
            ),
            CoreConfigurationValue::Integer(i) => (
                CoreConfigurationValue::Integer(*i),
                CoreConfigurationSchema::Integer {
                    min: None,
                    max: None,
                }
            ),
            CoreConfigurationValue::Float(f) => (
                CoreConfigurationValue::Float(*f),
                CoreConfigurationSchema::Float {
                    min: None,
                    max: None,
                }
            ),
            CoreConfigurationValue::Boolean(b) => (
                CoreConfigurationValue::Boolean(*b),
                CoreConfigurationSchema::Boolean
            ),
            _ => return Err(PyRuntimeError::new_err("Unsupported configuration value type")),
        };
        
        let env = environment.map(|e| e.inner);
        
        self.inner.set_configuration(
            key,
            config_value,
            schema,
            env,
            changed_by,
            change_reason,
        ).map_err(|e| PyRuntimeError::new_err(format!("Configuration error: {e}")))
    }
    
    fn get_configuration(
        &self,
        key: &str,
        environment: Option<PyConfigurationEnvironment>,
    ) -> Option<PyConfigurationValue> {
        let env = environment.map(|e| e.inner);
        self.inner.get_configuration(key, env)
            .map(|value| PyConfigurationValue { inner: value })
    }
    
    fn get_all_configurations(
        &self,
        environment: Option<PyConfigurationEnvironment>,
    ) -> Py<PyDict> {
        let env = environment.map(|e| e.inner);
        let configs = self.inner.get_all_configurations(env);
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (key, value) in configs {
                let py_value = PyConfigurationValue { inner: value };
                let _ = dict.set_item(key, Py::new(py, py_value).unwrap());
            }
            dict.into_py(py)
        })
    }
    
    #[pyo3(signature = (key, changed_by, change_reason, environment=None))]
    fn delete_configuration(
        &self,
        key: &str,
        changed_by: String,
        change_reason: String,
        environment: Option<PyConfigurationEnvironment>,
    ) -> PyResult<bool> {
        let env = environment.map(|e| e.inner);
        self.inner.delete_configuration(key, env, changed_by, change_reason)
            .map_err(|e| PyRuntimeError::new_err(format!("Configuration error: {e}")))
    }
    
    fn export_configurations(
        &self,
        environment: Option<PyConfigurationEnvironment>,
    ) -> String {
        let env = environment.map(|e| e.inner);
        let export = self.inner.export_configurations(env);
        export.to_string()
    }
    
    fn import_configurations(
        &self,
        json_data: &str,
        environment: PyConfigurationEnvironment,
        changed_by: String,
    ) -> PyResult<usize> {
        let json_value: serde_json::Value = serde_json::from_str(json_data)
            .map_err(|e| PyRuntimeError::new_err(format!("JSON parsing error: {e}")))?;
        
        self.inner.import_configurations(&json_value, environment.inner, changed_by)
            .map_err(|e| PyRuntimeError::new_err(format!("Import error: {e}")))
    }
    
    fn get_metrics(&self) -> Py<PyDict> {
        let metrics = self.inner.get_metrics();
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let _ = dict.set_item("tenant_id", metrics.tenant_id.as_str());
            let _ = dict.set_item("total_configurations", metrics.total_configurations);
            let _ = dict.set_item("total_changes_today", metrics.total_changes_today);
            let _ = dict.set_item("cache_hit_rate", metrics.cache_hit_rate);
            let _ = dict.set_item("average_retrieval_time_ms", metrics.average_retrieval_time_ms);
            
            if let Some(last_change) = metrics.last_change_timestamp {
                let _ = dict.set_item("last_change_timestamp", last_change.to_rfc3339());
            }
            
            dict.into_py(py)
        })
    }
    
    fn set_environment(&mut self, environment: PyConfigurationEnvironment) {
        self.inner.set_environment(environment.inner);
    }
    
    fn set_hot_reload_enabled(&mut self, enabled: bool) {
        self.inner.set_hot_reload_enabled(enabled);
    }
    
    fn set_validation_enabled(&mut self, enabled: bool) {
        self.inner.set_validation_enabled(enabled);
    }
}

/// Python wrapper for HealthStatus
#[pyclass(name = "HealthStatus")]
#[derive(Clone)]
pub struct PyHealthStatus {
    inner: CoreHealthStatus,
}

#[pymethods]
impl PyHealthStatus {
    fn __str__(&self) -> String {
        match self.inner {
            CoreHealthStatus::Excellent => "excellent".to_string(),
            CoreHealthStatus::Good => "good".to_string(),
            CoreHealthStatus::Fair => "fair".to_string(),
            CoreHealthStatus::Poor => "poor".to_string(),
            CoreHealthStatus::Critical => "critical".to_string(),
        }
    }
    
    fn __repr__(&self) -> String {
        format!("HealthStatus('{}')", self.__str__())
    }
}

/// Python wrapper for TenantHealthScore
#[pyclass(name = "TenantHealthScore")]
#[derive(Clone)]
pub struct PyTenantHealthScore {
    inner: CoreTenantHealthScore,
}

#[pymethods]
impl PyTenantHealthScore {
    #[getter]
    fn overall_score(&self) -> f64 {
        self.inner.overall_score
    }
    
    #[getter]
    fn status(&self) -> PyHealthStatus {
        PyHealthStatus { inner: self.inner.status.clone() }
    }
    
    #[getter]
    fn component_scores(&self) -> Py<PyDict> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (component, score) in &self.inner.component_scores {
                let _ = dict.set_item(component, *score);
            }
            dict.into_py(py)
        })
    }
    
    #[getter]
    fn active_alerts_count(&self) -> usize {
        self.inner.active_alerts_count
    }
    
    #[getter]
    fn critical_alerts_count(&self) -> usize {
        self.inner.critical_alerts_count
    }
    
    #[getter]
    fn calculated_at(&self) -> String {
        self.inner.calculated_at.to_rfc3339()
    }
    
    #[getter]
    fn recommendations(&self) -> Vec<String> {
        self.inner.recommendations.clone()
    }
    
    fn __str__(&self) -> String {
        format!(
            "TenantHealthScore(score={:.1}, status={}, alerts={}, critical={})",
            self.inner.overall_score,
            match self.inner.status {
                CoreHealthStatus::Excellent => "excellent",
                CoreHealthStatus::Good => "good",
                CoreHealthStatus::Fair => "fair",
                CoreHealthStatus::Poor => "poor",
                CoreHealthStatus::Critical => "critical",
            },
            self.inner.active_alerts_count,
            self.inner.critical_alerts_count
        )
    }
}

/// Python wrapper for MetricDataPoint
#[pyclass(name = "MetricDataPoint")]
#[derive(Clone)]
pub struct PyMetricDataPoint {
    inner: CoreMetricDataPoint,
}

#[pymethods]
impl PyMetricDataPoint {
    #[new]
    fn new(value: f64) -> Self {
        Self {
            inner: CoreMetricDataPoint::new(value)
        }
    }
    
    fn with_label(&mut self, key: String, value: String) {
        self.inner.add_label(key, value);
    }
    
    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }
    
    #[getter]
    fn value(&self) -> f64 {
        self.inner.value
    }
    
    #[getter]
    fn labels(&self) -> Py<PyDict> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (key, value) in &self.inner.labels {
                let _ = dict.set_item(key, value);
            }
            dict.into_py(py)
        })
    }
}

/// Python wrapper for TenantMetricsCollector
#[pyclass(name = "TenantMetricsCollector")]
pub struct PyTenantMetricsCollector {
    inner: CoreTenantMetricsCollector,
}

#[pymethods]
impl PyTenantMetricsCollector {
    #[new]
    fn new(tenant_id: PyTenantId) -> Self {
        Self {
            inner: CoreTenantMetricsCollector::new(tenant_id.inner)
        }
    }
    
    fn record_metric(
        &self,
        name: String,
        value: f64,
        labels: Option<Py<PyDict>>,
    ) -> PyResult<()> {
        let labels_map = if let Some(labels_py) = labels {
            Python::with_gil(|py| {
                let mut map = HashMap::new();
                let dict = labels_py.as_ref(py);
                
                for item in dict.items() {
                    let (key, value) = item.extract::<(String, String)>()?;
                    map.insert(key, value);
                }
                
                Ok::<HashMap<String, String>, PyErr>(map)
            })?
        } else {
            HashMap::new()
        };
        
        let labels_opt = if labels_map.is_empty() {
            None
        } else {
            Some(labels_map)
        };
        
        self.inner.record_metric(name, value, labels_opt);
        Ok(())
    }
    
    fn record_metrics(&self, metrics: Vec<(String, f64)>) -> PyResult<()> {
        let metrics_with_labels: Vec<(String, f64, Option<HashMap<String, String>>)> = 
            metrics.into_iter()
                .map(|(name, value)| (name, value, None))
                .collect();
        
        self.inner.record_metrics(metrics_with_labels);
        Ok(())
    }
    
    fn get_current_metric_value(&self, name: &str) -> Option<f64> {
        self.inner.get_current_metric_value(name)
    }
    
    fn get_metric_timeseries(
        &self,
        name: &str,
        start: Option<String>,
        end: Option<String>,
    ) -> PyResult<Vec<PyMetricDataPoint>> {
        let start_time = if let Some(start_str) = start {
            Some(chrono::DateTime::parse_from_rfc3339(&start_str)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid start time: {e}")))?
                .with_timezone(&chrono::Utc))
        } else {
            None
        };
        
        let end_time = if let Some(end_str) = end {
            Some(chrono::DateTime::parse_from_rfc3339(&end_str)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid end time: {e}")))?
                .with_timezone(&chrono::Utc))
        } else {
            None
        };
        
        if let Some(timeseries) = self.inner.get_metric_timeseries(name, start_time, end_time) {
            Ok(timeseries.into_iter()
                .map(|point| PyMetricDataPoint { inner: point })
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn detect_anomalies(&self, threshold_multiplier: f64) -> Py<PyDict> {
        let anomalies = self.inner.detect_anomalies(threshold_multiplier);
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (metric_name, points) in anomalies {
                let py_points: Vec<Py<PyMetricDataPoint>> = points.into_iter()
                    .map(|point| Py::new(py, PyMetricDataPoint { inner: point }).unwrap())
                    .collect();
                let _ = dict.set_item(metric_name, py_points);
            }
            dict.into_py(py)
        })
    }
    
    fn get_usage_patterns(&self) -> Py<PyDict> {
        let patterns = self.inner.get_usage_patterns();
        
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            for (metric_name, pattern) in patterns {
                let pattern_str = match pattern {
                    eventuali_core::tenancy::quota::UsagePattern::Stable => "stable",
                    eventuali_core::tenancy::quota::UsagePattern::Growing => "growing",
                    eventuali_core::tenancy::quota::UsagePattern::Declining => "declining",
                    eventuali_core::tenancy::quota::UsagePattern::Volatile => "volatile",
                    eventuali_core::tenancy::quota::UsagePattern::Seasonal => "seasonal",
                };
                let _ = dict.set_item(metric_name, pattern_str);
            }
            dict.into_py(py)
        })
    }
    
    fn calculate_health_score(&self) -> PyTenantHealthScore {
        PyTenantHealthScore {
            inner: self.inner.calculate_health_score()
        }
    }
    
    fn export_metrics(&self, format: String, time_range: Option<(String, String)>) -> PyResult<String> {
        let export_format = match format.as_str() {
            "json" => eventuali_core::tenancy::metrics::ExportFormat::Json,
            "csv" => eventuali_core::tenancy::metrics::ExportFormat::Csv,
            "prometheus" => eventuali_core::tenancy::metrics::ExportFormat::Prometheus,
            _ => return Err(PyRuntimeError::new_err(format!("Invalid export format: {format}"))),
        };
        
        let time_range_parsed = if let Some((start_str, end_str)) = time_range {
            let start = chrono::DateTime::parse_from_rfc3339(&start_str)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid start time: {e}")))?
                .with_timezone(&chrono::Utc);
            let end = chrono::DateTime::parse_from_rfc3339(&end_str)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid end time: {e}")))?
                .with_timezone(&chrono::Utc);
            Some((start, end))
        } else {
            None
        };
        
        self.inner.export_metrics(export_format, time_range_parsed)
            .map_err(|e| PyRuntimeError::new_err(format!("Export error: {e}")))
    }
}