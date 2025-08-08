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
use security::{
    PyEventEncryption, PyKeyManager, PyEncryptionKey, PyEncryptedEventData, PyEncryptionAlgorithm, PySecurityUtils,
    PyRbacManager, PyUser, PyRole, PyPermission, PySecurityLevel, PySession, PyAccessDecision, PyAuditEntry,
    PyAuditManager, PyAuditTrailEntry, PyAuditEventType, PyAuditOutcome, PyRiskLevel,
    PyDataClassification, PyComplianceTag, PyComplianceReport, PyIntegrityStatus,
    PyGdprManager, PyDataSubject, PyConsentRecord, PySubjectRightsRequest, PyBreachNotification,
    PyGdprComplianceStatus, PyGdprComplianceReport, PyPersonalDataType, PyLawfulBasisType,
    PyConsentMethod, PyConsentStatus, PyDataSubjectRight, PyRequestStatus, PyBreachType, PyExportFormat,
    // Digital signatures
    PyEventSigner, PySigningKeyManager, PySigningKey, PySignatureAlgorithm, PyEventSignature, PySignedEvent,
    // Data retention
    PyRetentionPolicyManager, PyRetentionPolicy, PyRetentionPeriod, PyDeletionMethod, PyDataCategory,
    PyRetentionEnforcementResult, PyLegalHold, PyEventDataClassification,
    // Vulnerability scanning
    PyVulnerabilityScanner, PyVulnerabilityScanResult, PyVulnerabilityFinding,
    PyVulnerabilityCategory, PyVulnerabilitySeverity, PyPenetrationTestFramework, PyPenetrationTest
};
use tenancy::{
    PyTenantId, PyTenantInfo, PyTenantConfig, PyTenantMetadata, PyResourceLimits, PyTenantManager, PyTenantStorageMetrics,
    PyQuotaTier, PyAlertType, PyQuotaCheckResult, PyQuotaAlert, PyBillingAnalytics, PyEnhancedResourceUsage,
    PyConfigurationEnvironment, PyConfigurationValue, PyTenantConfigurationManager,
    PyHealthStatus, PyTenantHealthScore, PyMetricDataPoint, PyTenantMetricsCollector
};

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
    
    // Register RBAC classes
    m.add_class::<PyRbacManager>()?;
    m.add_class::<PyUser>()?;
    m.add_class::<PyRole>()?;
    m.add_class::<PyPermission>()?;
    m.add_class::<PySecurityLevel>()?;
    m.add_class::<PySession>()?;
    m.add_class::<PyAccessDecision>()?;
    m.add_class::<PyAuditEntry>()?;
    
    // Register comprehensive audit trail classes
    m.add_class::<PyAuditManager>()?;
    m.add_class::<PyAuditTrailEntry>()?;
    m.add_class::<PyAuditEventType>()?;
    m.add_class::<PyAuditOutcome>()?;
    m.add_class::<PyRiskLevel>()?;
    m.add_class::<PyDataClassification>()?;
    m.add_class::<PyComplianceTag>()?;
    m.add_class::<PyComplianceReport>()?;
    m.add_class::<PyIntegrityStatus>()?;
    
    // Register GDPR compliance classes
    m.add_class::<PyGdprManager>()?;
    m.add_class::<PyDataSubject>()?;
    m.add_class::<PyConsentRecord>()?;
    m.add_class::<PySubjectRightsRequest>()?;
    m.add_class::<PyBreachNotification>()?;
    m.add_class::<PyGdprComplianceStatus>()?;
    m.add_class::<PyGdprComplianceReport>()?;
    m.add_class::<PyPersonalDataType>()?;
    m.add_class::<PyLawfulBasisType>()?;
    m.add_class::<PyConsentMethod>()?;
    m.add_class::<PyConsentStatus>()?;
    m.add_class::<PyDataSubjectRight>()?;
    m.add_class::<PyRequestStatus>()?;
    m.add_class::<PyBreachType>()?;
    m.add_class::<PyExportFormat>()?;
    
    // Register digital signature classes
    m.add_class::<PyEventSigner>()?;
    m.add_class::<PySigningKeyManager>()?;
    m.add_class::<PySigningKey>()?;
    m.add_class::<PySignatureAlgorithm>()?;
    m.add_class::<PyEventSignature>()?;
    m.add_class::<PySignedEvent>()?;
    
    // Register data retention classes
    m.add_class::<PyRetentionPolicyManager>()?;
    m.add_class::<PyRetentionPolicy>()?;
    m.add_class::<PyRetentionPeriod>()?;
    m.add_class::<PyDeletionMethod>()?;
    m.add_class::<PyDataCategory>()?;
    m.add_class::<PyRetentionEnforcementResult>()?;
    m.add_class::<PyLegalHold>()?;
    m.add_class::<PyEventDataClassification>()?;
    
    // Register vulnerability scanning classes
    m.add_class::<PyVulnerabilityScanner>()?;
    m.add_class::<PyVulnerabilityScanResult>()?;
    m.add_class::<PyVulnerabilityFinding>()?;
    m.add_class::<PyVulnerabilityCategory>()?;
    m.add_class::<PyVulnerabilitySeverity>()?;
    m.add_class::<PyPenetrationTestFramework>()?;
    m.add_class::<PyPenetrationTest>()?;
    
    // Register tenancy classes
    m.add_class::<PyTenantId>()?;
    m.add_class::<PyTenantInfo>()?;
    m.add_class::<PyTenantConfig>()?;
    m.add_class::<PyTenantMetadata>()?;
    m.add_class::<PyResourceLimits>()?;
    m.add_class::<PyTenantManager>()?;
    m.add_class::<PyTenantStorageMetrics>()?;
    
    // Register enhanced quota classes
    m.add_class::<PyQuotaTier>()?;
    m.add_class::<PyAlertType>()?;
    m.add_class::<PyQuotaCheckResult>()?;
    m.add_class::<PyQuotaAlert>()?;
    m.add_class::<PyBillingAnalytics>()?;
    m.add_class::<PyEnhancedResourceUsage>()?;
    
    // Register configuration management classes
    m.add_class::<PyConfigurationEnvironment>()?;
    m.add_class::<PyConfigurationValue>()?;
    m.add_class::<PyTenantConfigurationManager>()?;
    
    // Register metrics classes
    m.add_class::<PyHealthStatus>()?;
    m.add_class::<PyTenantHealthScore>()?;
    m.add_class::<PyMetricDataPoint>()?;
    m.add_class::<PyTenantMetricsCollector>()?;
    
    // Register custom exceptions
    error::register_exceptions(py, m)?;
    
    // Register observability classes if the feature is enabled
    #[cfg(feature = "observability")]
    observability::register_observability_classes(py, m)?;
    
    // Register performance optimization classes  
    performance::register_performance_module(py, m)?;
    
    Ok(())
}