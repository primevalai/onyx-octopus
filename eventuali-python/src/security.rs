use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::exceptions::PyRuntimeError;
use eventuali_core::security::{
    EventEncryption as CoreEventEncryption, KeyManager as CoreKeyManager, 
    EncryptionKey as CoreEncryptionKey, EncryptedEventData as CoreEncryptedEventData,
    EncryptionAlgorithm as CoreEncryptionAlgorithm,
    RbacManager as CoreRbacManager, User as CoreUser, Role as CoreRole,
    Permission as CorePermission, Session as CoreSession, SecurityLevel as CoreSecurityLevel,
    AccessDecision as CoreAccessDecision, AuditEntry as CoreAuditEntry,
    AuditManager as CoreAuditManager, AuditTrailEntry as CoreAuditTrailEntry,
    AuditEventType as CoreAuditEventType, AuditOutcome as CoreAuditOutcome,
    RiskLevel as CoreRiskLevel, DataClassification as CoreDataClassification,
    ComplianceTag as CoreComplianceTag, AuditSearchCriteria as CoreAuditSearchCriteria,
    ComplianceReport as CoreComplianceReport, IntegrityStatus as CoreIntegrityStatus,
    GdprManager as CoreGdprManager, DataSubject as CoreDataSubject,
    ConsentRecord as CoreConsentRecord,
    BreachNotification as CoreBreachNotification,
    SubjectRightsRequest as CoreSubjectRightsRequest,
    GdprComplianceStatus as CoreGdprComplianceStatus,
    GdprComplianceReport as CoreGdprComplianceReport, PersonalDataType as CorePersonalDataType,
    LawfulBasisType as CoreLawfulBasisType, ConsentStatus as CoreConsentStatus,
    ConsentMethod as CoreConsentMethod, ConsentEvidence as CoreConsentEvidence,
    DataSubjectRight as CoreDataSubjectRight, RequestStatus as CoreRequestStatus,
    BreachType as CoreBreachType, ExportFormat as CoreExportFormat,
    // Digital signatures
    EventSigner as CoreEventSigner, SigningKeyManager as CoreSigningKeyManager,
    SigningKey as CoreSigningKey, SignatureAlgorithm as CoreSignatureAlgorithm,
    EventSignature as CoreEventSignature, SignedEvent as CoreSignedEvent,
    // Data retention
    RetentionPolicyManager as CoreRetentionPolicyManager,
    RetentionPeriod as CoreRetentionPeriod, DeletionMethod as CoreDeletionMethod,
    DataCategory as CoreDataCategory, RetentionEnforcementResult as CoreRetentionEnforcementResult,
    LegalHold as CoreLegalHold,
    EventDataClassification as CoreEventDataClassification,
    // Vulnerability scanning
    VulnerabilityScanner as CoreVulnerabilityScanner, VulnerabilityScanResult as CoreVulnerabilityScanResult,
    VulnerabilityFinding as CoreVulnerabilityFinding, VulnerabilityCategory as CoreVulnerabilityCategory,
    VulnerabilitySeverity as CoreVulnerabilitySeverity,
    PenetrationTestFramework as CorePenetrationTestFramework, PenetrationTest as CorePenetrationTest
};
use eventuali_core::{EventData as CoreEventData};
use eventuali_core::security::retention::RetentionPolicy as CoreRetentionPolicy;
use crate::event::PyEvent;
use crate::error::map_rust_error_to_python;
use std::collections::HashMap;

/// Python wrapper for EventEncryption
#[pyclass(name = "EventEncryption")]
pub struct PyEventEncryption {
    pub(crate) inner: CoreEventEncryption,
}

/// Python wrapper for KeyManager  
#[pyclass(name = "KeyManager")]
#[derive(Clone)]
pub struct PyKeyManager {
    pub(crate) inner: CoreKeyManager,
}

/// Python wrapper for EncryptionKey
#[pyclass(name = "EncryptionKey")]
pub struct PyEncryptionKey {
    pub(crate) inner: CoreEncryptionKey,
}

/// Python wrapper for EncryptedEventData
#[pyclass(name = "EncryptedEventData")]
pub struct PyEncryptedEventData {
    pub(crate) inner: CoreEncryptedEventData,
}

/// Python wrapper for EncryptionAlgorithm
#[pyclass(name = "EncryptionAlgorithm")]
#[derive(Clone)]
pub struct PyEncryptionAlgorithm {
    pub(crate) inner: CoreEncryptionAlgorithm,
}

#[pymethods]
impl PyEventEncryption {
    /// Create new encryption instance with a key manager
    #[new]
    pub fn new(key_manager: PyKeyManager) -> Self {
        Self {
            inner: CoreEventEncryption::new(key_manager.inner),
        }
    }

    /// Create encryption instance with a key manager containing a single key
    #[classmethod]
    pub fn with_generated_key(_cls: &PyType, key_id: String) -> PyResult<Self> {
        let key = CoreKeyManager::generate_key(key_id.clone())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        CoreEventEncryption::with_key(key_id, key.key_data)
            .map(|inner| Self { inner })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Create encryption instance from a key manager
    #[classmethod]
    pub fn from_key_manager(_cls: &PyType, key_manager: PyKeyManager) -> Self {
        Self {
            inner: CoreEventEncryption::new(key_manager.inner),
        }
    }

    /// Encrypt JSON data using the default key
    pub fn encrypt_json_data(&self, data: String) -> PyResult<PyEncryptedEventData> {
        let json_value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid JSON: {e}")))?;
        let event_data = CoreEventData::Json(json_value);
        
        self.inner
            .encrypt_event_data(&event_data)
            .map(|inner| PyEncryptedEventData { inner })
            .map_err(map_rust_error_to_python)
    }

    /// Encrypt JSON data using a specific key
    pub fn encrypt_json_data_with_key(&self, data: String, key_id: &str) -> PyResult<PyEncryptedEventData> {
        let json_value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid JSON: {e}")))?;
        let event_data = CoreEventData::Json(json_value);
        
        self.inner
            .encrypt_event_data_with_key(&event_data, key_id)
            .map(|inner| PyEncryptedEventData { inner })
            .map_err(map_rust_error_to_python)
    }

    /// Decrypt data and return as JSON string
    pub fn decrypt_to_json(&self, encrypted_data: &PyEncryptedEventData) -> PyResult<String> {
        let decrypted_data = self.inner
            .decrypt_event_data(&encrypted_data.inner)
            .map_err(map_rust_error_to_python)?;
        
        match decrypted_data {
            CoreEventData::Json(value) => {
                serde_json::to_string(&value)
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize JSON: {e}")))
            }
            CoreEventData::Protobuf(bytes) => {
                String::from_utf8(bytes)
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert bytes to string: {e}")))
            }
        }
    }
}

impl Default for PyKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyKeyManager {
    /// Create a new key manager
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreKeyManager::new(),
        }
    }

    /// Add a key to the manager
    pub fn add_key(&mut self, key: &PyEncryptionKey) -> PyResult<()> {
        self.inner
            .add_key(key.inner.clone())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Generate a new AES-256 key
    #[classmethod]
    pub fn generate_key(_cls: &PyType, id: String) -> PyResult<PyEncryptionKey> {
        CoreKeyManager::generate_key(id)
            .map(|inner| PyEncryptionKey { inner })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Generate a key from a password using PBKDF2
    #[classmethod]
    pub fn derive_key_from_password(
        _cls: &PyType,
        id: String,
        password: String,
        salt: Vec<u8>,
    ) -> PyResult<PyEncryptionKey> {
        CoreKeyManager::derive_key_from_password(id, &password, &salt)
            .map(|inner| PyEncryptionKey { inner })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Set the default key
    pub fn set_default_key(&mut self, key_id: &str) -> PyResult<()> {
        self.inner
            .set_default_key(key_id)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Get all key IDs
    pub fn get_key_ids(&self) -> Vec<String> {
        // Since we can't access the inner HashMap directly, we'll return an empty vec for now
        // In a real implementation, we'd add a method to CoreKeyManager to list key IDs
        vec![]
    }
}

#[pymethods]
impl PyEncryptionKey {
    /// Get the key ID
    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    /// Get the key algorithm
    #[getter]
    pub fn algorithm(&self) -> PyEncryptionAlgorithm {
        PyEncryptionAlgorithm {
            inner: self.inner.algorithm.clone(),
        }
    }

    /// Get the creation timestamp as ISO string
    #[getter]
    pub fn created_at(&self) -> String {
        self.inner.created_at.to_rfc3339()
    }

    /// Get key data length (for verification, not the actual data for security)
    #[getter]
    pub fn key_length(&self) -> usize {
        self.inner.key_data.len()
    }
}

#[pymethods]
impl PyEncryptedEventData {
    /// Get the encryption algorithm
    #[getter]
    pub fn algorithm(&self) -> PyEncryptionAlgorithm {
        PyEncryptionAlgorithm {
            inner: self.inner.algorithm.clone(),
        }
    }

    /// Get the key ID used for encryption
    #[getter]
    pub fn key_id(&self) -> String {
        self.inner.key_id.clone()
    }

    /// Get the initialization vector length
    #[getter]
    pub fn iv_length(&self) -> usize {
        self.inner.iv.len()
    }

    /// Get the encrypted data size
    #[getter]
    pub fn encrypted_size(&self) -> usize {
        self.inner.encrypted_data.len()
    }

    /// Serialize to base64 string for storage
    pub fn to_base64(&self) -> String {
        self.inner.to_base64()
    }

    /// Deserialize from base64 string
    #[classmethod]
    pub fn from_base64(_cls: &PyType, data: String) -> PyResult<Self> {
        CoreEncryptedEventData::from_base64(&data)
            .map(|inner| Self { inner })
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

#[pymethods]
impl PyEncryptionAlgorithm {
    /// String representation of the algorithm
    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreEncryptionAlgorithm::Aes256Gcm => "AES-256-GCM",
        }
    }

    /// Create AES-256-GCM algorithm
    #[classmethod]
    pub fn aes256gcm(_cls: &PyType) -> Self {
        Self {
            inner: CoreEncryptionAlgorithm::Aes256Gcm,
        }
    }
}

/// Python wrapper for RBAC Manager
#[pyclass(name = "RbacManager")]
pub struct PyRbacManager {
    pub(crate) inner: CoreRbacManager,
}

/// Python wrapper for User
#[pyclass(name = "User")]
#[derive(Clone)]
pub struct PyUser {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreUser,
}

/// Python wrapper for Role
#[pyclass(name = "Role")]
#[derive(Clone)]
pub struct PyRole {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreRole,
}

/// Python wrapper for Permission
#[pyclass(name = "Permission")]
#[derive(Clone)]
pub struct PyPermission {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CorePermission,
}

/// Python wrapper for SecurityLevel
#[pyclass(name = "SecurityLevel")]
#[derive(Clone)]
pub struct PySecurityLevel {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreSecurityLevel,
}

/// Python wrapper for Session
#[pyclass(name = "Session")]
#[derive(Clone)]
pub struct PySession {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreSession,
}

/// Python wrapper for AccessDecision
#[pyclass(name = "AccessDecision")]
#[derive(Clone)]
pub struct PyAccessDecision {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreAccessDecision,
}

/// Python wrapper for AuditEntry
#[pyclass(name = "AuditEntry")]
#[derive(Clone)]
pub struct PyAuditEntry {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreAuditEntry,
}

/// Security utilities for Python
#[pyclass(name = "SecurityUtils")]
pub struct PySecurityUtils;

#[pymethods]
impl PySecurityUtils {
    /// Generate a cryptographically secure random salt
    #[classmethod]
    pub fn generate_salt(_cls: &PyType, length: Option<usize>) -> Vec<u8> {
        let len = length.unwrap_or(32);
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Generate salt using system time and additional entropy
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        let mut salt = Vec::with_capacity(len);
        let timestamp_bytes = timestamp.to_be_bytes();
        
        // Repeat timestamp bytes to fill the salt
        for i in 0..len {
            salt.push(timestamp_bytes[i % timestamp_bytes.len()]);
        }
        
        // Add some variation based on index
        for (i, byte) in salt.iter_mut().enumerate() {
            *byte = byte.wrapping_add(i as u8);
        }
        
        salt
    }

    /// Benchmark encryption performance
    #[classmethod] 
    pub fn benchmark_encryption(_cls: &PyType, iterations: Option<usize>) -> PyResult<HashMap<String, f64>> {
        let iter_count = iterations.unwrap_or(1000);
        let key = CoreKeyManager::generate_key("benchmark-key".to_string())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let encryption = CoreEventEncryption::with_key("benchmark-key".to_string(), key.key_data)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        // Create test data
        let test_data = CoreEventData::Json(serde_json::json!({
            "user_id": "user123",
            "action": "test_action",
            "data": "A".repeat(1000), // 1KB of data
            "timestamp": "2024-01-01T00:00:00Z"
        }));

        let start = std::time::Instant::now();
        
        for _ in 0..iter_count {
            let encrypted = encryption.encrypt_event_data(&test_data)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            let _ = encryption.decrypt_event_data(&encrypted)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        }
        
        let duration = start.elapsed();
        let total_ms = duration.as_millis() as f64;
        let per_operation_ms = total_ms / (iter_count * 2) as f64; // encrypt + decrypt
        let operations_per_sec = 1000.0 / per_operation_ms;

        let mut results = HashMap::new();
        results.insert("total_time_ms".to_string(), total_ms);
        results.insert("per_operation_ms".to_string(), per_operation_ms);
        results.insert("operations_per_sec".to_string(), operations_per_sec);
        results.insert("iterations".to_string(), iter_count as f64);

        Ok(results)
    }
}

impl Default for PyRbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyRbacManager {
    /// Create a new RBAC manager
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreRbacManager::new(),
        }
    }

    /// Create a new user
    pub fn create_user(&mut self, username: String, email: String, security_level: PySecurityLevel) -> PyResult<String> {
        self.inner
            .create_user(username, email, security_level.inner)
            .map_err(map_rust_error_to_python)
    }

    /// Assign role to user
    pub fn assign_role_to_user(&mut self, user_id: String, role_id: String) -> PyResult<()> {
        self.inner
            .assign_role_to_user(&user_id, &role_id)
            .map_err(map_rust_error_to_python)
    }

    /// Create a new role
    pub fn create_role(&mut self, name: String, description: String) -> PyResult<String> {
        self.inner
            .create_role(name, description)
            .map_err(map_rust_error_to_python)
    }

    /// Authenticate user and return session token
    pub fn authenticate(&mut self, username: String, password: String, ip_address: Option<String>) -> PyResult<String> {
        self.inner
            .authenticate(&username, &password, ip_address)
            .map_err(map_rust_error_to_python)
    }

    /// Check access permission
    pub fn check_access(&mut self, token: String, resource: String, action: String, context: Option<HashMap<String, String>>) -> PyAccessDecision {
        let decision = self.inner.check_access(&token, &resource, &action, context);
        PyAccessDecision { inner: decision }
    }

    /// Revoke session
    pub fn revoke_session(&mut self, token: String) -> PyResult<()> {
        self.inner
            .revoke_session(&token)
            .map_err(map_rust_error_to_python)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        self.inner.cleanup_expired_sessions();
    }

    /// Get audit trail
    pub fn get_audit_trail(&self, limit: Option<usize>) -> Vec<PyAuditEntry> {
        self.inner
            .get_audit_trail(limit)
            .into_iter()
            .map(|entry| PyAuditEntry { inner: entry.clone() })
            .collect()
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> HashMap<String, String> {
        let stats = self.inner.get_system_stats();
        stats.into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect()
    }
}

#[pymethods]
impl PySecurityLevel {
    /// Create Public security level
    #[classmethod]
    pub fn public(_cls: &PyType) -> Self {
        Self { inner: CoreSecurityLevel::Public }
    }

    /// Create Internal security level
    #[classmethod]
    pub fn internal(_cls: &PyType) -> Self {
        Self { inner: CoreSecurityLevel::Internal }
    }

    /// Create Confidential security level
    #[classmethod]
    pub fn confidential(_cls: &PyType) -> Self {
        Self { inner: CoreSecurityLevel::Confidential }
    }

    /// Create Secret security level
    #[classmethod]
    pub fn secret(_cls: &PyType) -> Self {
        Self { inner: CoreSecurityLevel::Secret }
    }

    /// Create TopSecret security level
    #[classmethod]
    pub fn top_secret(_cls: &PyType) -> Self {
        Self { inner: CoreSecurityLevel::TopSecret }
    }

    /// String representation
    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreSecurityLevel::Public => "Public",
            CoreSecurityLevel::Internal => "Internal",
            CoreSecurityLevel::Confidential => "Confidential",
            CoreSecurityLevel::Secret => "Secret",
            CoreSecurityLevel::TopSecret => "TopSecret",
        }
    }

    /// Check if this level can access the required level
    pub fn can_access(&self, required_level: PySecurityLevel) -> bool {
        self.inner.can_access(&required_level.inner)
    }
}

#[pymethods]
impl PyAccessDecision {
    /// Check if access is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self.inner, CoreAccessDecision::Allow)
    }

    /// Check if access is denied
    pub fn is_denied(&self) -> bool {
        !matches!(self.inner, CoreAccessDecision::Allow)
    }

    /// Get denial reason if any
    pub fn get_reason(&self) -> Option<String> {
        match &self.inner {
            CoreAccessDecision::DenyWithReason(reason) => Some(reason.clone()),
            _ => None,
        }
    }

    /// String representation
    pub fn __str__(&self) -> String {
        match &self.inner {
            CoreAccessDecision::Allow => "Allow".to_string(),
            CoreAccessDecision::Deny => "Deny".to_string(),
            CoreAccessDecision::DenyWithReason(reason) => format!("Deny: {reason}"),
        }
    }
}

#[pymethods]
impl PyAuditEntry {
    /// Get audit ID
    #[getter]
    pub fn audit_id(&self) -> String {
        self.inner.audit_id.clone()
    }

    /// Get user ID
    #[getter]
    pub fn user_id(&self) -> String {
        self.inner.user_id.clone()
    }

    /// Get action
    #[getter]
    pub fn action(&self) -> String {
        self.inner.action.clone()
    }

    /// Get resource
    #[getter]
    pub fn resource(&self) -> String {
        self.inner.resource.clone()
    }

    /// Get timestamp
    #[getter]
    pub fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// Get access decision
    #[getter]
    pub fn decision(&self) -> PyAccessDecision {
        PyAccessDecision { inner: self.inner.decision.clone() }
    }

    /// Get IP address
    #[getter]
    pub fn ip_address(&self) -> Option<String> {
        self.inner.ip_address.clone()
    }

    /// Get reason
    #[getter]
    pub fn reason(&self) -> Option<String> {
        self.inner.reason.clone()
    }

    /// String representation
    pub fn __str__(&self) -> String {
        format!(
            "AuditEntry(user={}, action={}, resource={}, decision={}, timestamp={})",
            self.inner.user_id,
            self.inner.action,
            self.inner.resource,
            PyAccessDecision { inner: self.inner.decision.clone() }.__str__(),
            self.inner.timestamp.to_rfc3339()
        )
    }
}

// ============================================================================
// COMPREHENSIVE AUDIT TRAIL SYSTEM - Python Bindings
// ============================================================================

/// Python wrapper for AuditManager
#[pyclass(name = "AuditManager")]
pub struct PyAuditManager {
    pub(crate) inner: CoreAuditManager,
}

/// Python wrapper for AuditTrailEntry
#[pyclass(name = "AuditTrailEntry")]
#[derive(Clone)]
pub struct PyAuditTrailEntry {
    pub(crate) inner: CoreAuditTrailEntry,
}

/// Python wrapper for AuditEventType
#[pyclass(name = "AuditEventType")]
#[derive(Clone)]
pub struct PyAuditEventType {
    pub(crate) inner: CoreAuditEventType,
}

/// Python wrapper for AuditOutcome
#[pyclass(name = "AuditOutcome")]
#[derive(Clone)]
pub struct PyAuditOutcome {
    pub(crate) inner: CoreAuditOutcome,
}

/// Python wrapper for RiskLevel
#[pyclass(name = "RiskLevel")]
#[derive(Clone)]
pub struct PyRiskLevel {
    pub(crate) inner: CoreRiskLevel,
}

/// Python wrapper for DataClassification
#[pyclass(name = "DataClassification")]
#[derive(Clone)]
pub struct PyDataClassification {
    pub(crate) inner: CoreDataClassification,
}

/// Python wrapper for ComplianceTag
#[pyclass(name = "ComplianceTag")]
#[derive(Clone)]
pub struct PyComplianceTag {
    pub(crate) inner: CoreComplianceTag,
}

/// Python wrapper for ComplianceReport
#[pyclass(name = "ComplianceReport")]
#[derive(Clone)]
pub struct PyComplianceReport {
    pub(crate) inner: CoreComplianceReport,
}

/// Python wrapper for IntegrityStatus
#[pyclass(name = "IntegrityStatus")]
#[derive(Clone)]
pub struct PyIntegrityStatus {
    pub(crate) inner: CoreIntegrityStatus,
}

impl Default for PyAuditManager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyAuditManager {
    /// Create a new audit manager
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreAuditManager::new(),
        }
    }

    /// Create audit manager with compliance frameworks
    #[classmethod]
    pub fn with_compliance(_cls: &PyType, frameworks: Vec<PyComplianceTag>) -> Self {
        let core_frameworks = frameworks.into_iter().map(|f| f.inner).collect();
        Self {
            inner: CoreAuditManager::with_compliance(core_frameworks),
        }
    }

    /// Log an audit event
    pub fn log_audit_event(
        &mut self,
        event_type: PyAuditEventType,
        user_id: String,
        action: String,
        resource: String,
        outcome: PyAuditOutcome,
        metadata: Option<HashMap<String, String>>,
    ) -> PyResult<String> {
        self.inner
            .log_audit_event(
                event_type.inner,
                user_id,
                action,
                resource,
                outcome.inner,
                metadata,
            )
            .map_err(map_rust_error_to_python)
    }

    /// Log authentication event
    #[pyo3(signature = (user_id, success, session_id=None, ip_address=None, user_agent=None, failure_reason=None))]
    pub fn log_authentication_event(
        &mut self,
        user_id: String,
        success: bool,
        session_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        failure_reason: Option<String>,
    ) -> PyResult<String> {
        self.inner
            .log_authentication_event(
                user_id,
                session_id,
                ip_address,
                user_agent,
                success,
                failure_reason,
            )
            .map_err(map_rust_error_to_python)
    }

    /// Log data access event
    #[pyo3(signature = (user_id, resource, resource_id, operation, data_classification, success))]
    pub fn log_data_access_event(
        &mut self,
        user_id: String,
        resource: String,
        resource_id: Option<String>,
        operation: String,
        data_classification: PyDataClassification,
        success: bool,
    ) -> PyResult<String> {
        self.inner
            .log_data_access_event(
                user_id,
                resource,
                resource_id,
                operation,
                data_classification.inner,
                success,
            )
            .map_err(map_rust_error_to_python)
    }

    /// Search audit entries
    pub fn search_audit_entries(
        &self,
        user_id: Option<String>,
        event_types: Option<Vec<PyAuditEventType>>,
        start_time: Option<String>,
        end_time: Option<String>,
        limit: Option<usize>,
    ) -> PyResult<Vec<PyAuditTrailEntry>> {
        use chrono::DateTime;
        
        let core_event_types = event_types.map(|types| {
            types.into_iter().map(|t| t.inner).collect()
        });
        
        let start_dt = if let Some(time_str) = start_time {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid start_time format: {e}")))?
                .with_timezone(&chrono::Utc))
        } else {
            None
        };
        
        let end_dt = if let Some(time_str) = end_time {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid end_time format: {e}")))?
                .with_timezone(&chrono::Utc))
        } else {
            None
        };

        let criteria = CoreAuditSearchCriteria {
            user_id,
            event_types: core_event_types,
            resources: None,
            start_time: start_dt,
            end_time: end_dt,
            risk_levels: None,
            compliance_tags: None,
            ip_addresses: None,
            outcomes: None,
            text_search: None,
        };

        let results = self.inner.search_audit_entries(&criteria, limit);
        
        Ok(results
            .into_iter()
            .map(|entry| PyAuditTrailEntry { inner: entry.clone() })
            .collect())
    }

    /// Generate compliance report
    pub fn generate_compliance_report(
        &self,
        framework: PyComplianceTag,
        start_time: String,
        end_time: String,
    ) -> PyResult<PyComplianceReport> {
        use chrono::DateTime;
        
        let start_dt = DateTime::parse_from_rfc3339(&start_time)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid start_time format: {e}")))?
            .with_timezone(&chrono::Utc);
        
        let end_dt = DateTime::parse_from_rfc3339(&end_time)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid end_time format: {e}")))?
            .with_timezone(&chrono::Utc);

        self.inner
            .generate_compliance_report(framework.inner, start_dt, end_dt)
            .map(|report| PyComplianceReport { inner: report })
            .map_err(map_rust_error_to_python)
    }

    /// Verify audit trail integrity
    pub fn verify_integrity(&self) -> PyIntegrityStatus {
        let status = self.inner.verify_integrity();
        PyIntegrityStatus { inner: status }
    }

    /// Get audit statistics
    pub fn get_audit_statistics(&self, last_hours: u32) -> HashMap<String, String> {
        let stats = self.inner.get_audit_statistics(last_hours);
        stats.into_iter()
            .map(|(k, v)| (k, v.to_string()))
            .collect()
    }
}

#[pymethods]
impl PyAuditEventType {
    #[classmethod]
    pub fn authentication(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::Authentication }
    }

    #[classmethod]
    pub fn authorization(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::Authorization }
    }

    #[classmethod]
    pub fn data_access(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::DataAccess }
    }

    #[classmethod]
    pub fn data_modification(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::DataModification }
    }

    #[classmethod]
    pub fn system_access(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::SystemAccess }
    }

    #[classmethod]
    pub fn configuration_change(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::ConfigurationChange }
    }

    #[classmethod]
    pub fn security_violation(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::SecurityViolation }
    }

    #[classmethod]
    pub fn privileged_operation(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::PrivilegedOperation }
    }

    #[classmethod]
    pub fn data_export(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::DataExport }
    }

    #[classmethod]
    pub fn policy_violation(_cls: &PyType) -> Self {
        Self { inner: CoreAuditEventType::PolicyViolation }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreAuditEventType::Authentication => "Authentication",
            CoreAuditEventType::Authorization => "Authorization",
            CoreAuditEventType::DataAccess => "DataAccess",
            CoreAuditEventType::DataModification => "DataModification",
            CoreAuditEventType::SystemAccess => "SystemAccess",
            CoreAuditEventType::ConfigurationChange => "ConfigurationChange",
            CoreAuditEventType::SecurityViolation => "SecurityViolation",
            CoreAuditEventType::PrivilegedOperation => "PrivilegedOperation",
            CoreAuditEventType::DataExport => "DataExport",
            CoreAuditEventType::AccountManagement => "AccountManagement",
            CoreAuditEventType::SessionManagement => "SessionManagement",
            CoreAuditEventType::PolicyViolation => "PolicyViolation",
            CoreAuditEventType::Backup => "Backup",
            CoreAuditEventType::Recovery => "Recovery",
            CoreAuditEventType::SystemMaintenance => "SystemMaintenance",
        }
    }
}

#[pymethods]
impl PyAuditOutcome {
    #[classmethod]
    pub fn success(_cls: &PyType) -> Self {
        Self { inner: CoreAuditOutcome::Success }
    }

    #[classmethod]
    pub fn failure(_cls: &PyType) -> Self {
        Self { inner: CoreAuditOutcome::Failure }
    }

    #[classmethod]
    pub fn warning(_cls: &PyType) -> Self {
        Self { inner: CoreAuditOutcome::Warning }
    }

    #[classmethod]
    pub fn blocked(_cls: &PyType) -> Self {
        Self { inner: CoreAuditOutcome::Blocked }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreAuditOutcome::Success => "Success",
            CoreAuditOutcome::Failure => "Failure",
            CoreAuditOutcome::Partial => "Partial",
            CoreAuditOutcome::Warning => "Warning",
            CoreAuditOutcome::Blocked => "Blocked",
            CoreAuditOutcome::Escalated => "Escalated",
        }
    }
}

#[pymethods]
impl PyRiskLevel {
    #[classmethod]
    pub fn low(_cls: &PyType) -> Self {
        Self { inner: CoreRiskLevel::Low }
    }

    #[classmethod]
    pub fn medium(_cls: &PyType) -> Self {
        Self { inner: CoreRiskLevel::Medium }
    }

    #[classmethod]
    pub fn high(_cls: &PyType) -> Self {
        Self { inner: CoreRiskLevel::High }
    }

    #[classmethod]
    pub fn critical(_cls: &PyType) -> Self {
        Self { inner: CoreRiskLevel::Critical }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreRiskLevel::Low => "Low",
            CoreRiskLevel::Medium => "Medium",
            CoreRiskLevel::High => "High",
            CoreRiskLevel::Critical => "Critical",
        }
    }
}

#[pymethods]
impl PyDataClassification {
    #[classmethod]
    pub fn public(_cls: &PyType) -> Self {
        Self { inner: CoreDataClassification::Public }
    }

    #[classmethod]
    pub fn internal(_cls: &PyType) -> Self {
        Self { inner: CoreDataClassification::Internal }
    }

    #[classmethod]
    pub fn confidential(_cls: &PyType) -> Self {
        Self { inner: CoreDataClassification::Confidential }
    }

    #[classmethod]
    pub fn healthcare_data(_cls: &PyType) -> Self {
        Self { inner: CoreDataClassification::HealthcareData }
    }

    #[classmethod]
    pub fn financial_data(_cls: &PyType) -> Self {
        Self { inner: CoreDataClassification::FinancialData }
    }

    #[classmethod]
    pub fn personal_data(_cls: &PyType) -> Self {
        Self { inner: CoreDataClassification::PersonalData }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreDataClassification::Public => "Public",
            CoreDataClassification::Internal => "Internal",
            CoreDataClassification::Confidential => "Confidential",
            CoreDataClassification::Restricted => "Restricted",
            CoreDataClassification::HealthcareData => "HealthcareData",
            CoreDataClassification::FinancialData => "FinancialData",
            CoreDataClassification::PersonalData => "PersonalData",
        }
    }
}

#[pymethods]
impl PyComplianceTag {
    #[classmethod]
    pub fn sox(_cls: &PyType) -> Self {
        Self { inner: CoreComplianceTag::SOX }
    }

    #[classmethod]
    pub fn gdpr(_cls: &PyType) -> Self {
        Self { inner: CoreComplianceTag::GDPR }
    }

    #[classmethod]
    pub fn hipaa(_cls: &PyType) -> Self {
        Self { inner: CoreComplianceTag::HIPAA }
    }

    #[classmethod]
    pub fn pci_dss(_cls: &PyType) -> Self {
        Self { inner: CoreComplianceTag::PciDss }
    }

    #[classmethod]
    pub fn iso27001(_cls: &PyType) -> Self {
        Self { inner: CoreComplianceTag::ISO27001 }
    }

    #[classmethod]
    pub fn nist(_cls: &PyType) -> Self {
        Self { inner: CoreComplianceTag::NIST }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreComplianceTag::SOX => "SOX",
            CoreComplianceTag::GDPR => "GDPR",
            CoreComplianceTag::HIPAA => "HIPAA",
            CoreComplianceTag::PciDss => "PCI_DSS",
            CoreComplianceTag::ISO27001 => "ISO27001",
            CoreComplianceTag::NIST => "NIST",
            CoreComplianceTag::COBIT => "COBIT",
            CoreComplianceTag::ITIL => "ITIL",
        }
    }
}

#[pymethods]
impl PyAuditTrailEntry {
    #[getter]
    pub fn entry_id(&self) -> String {
        self.inner.entry_id.clone()
    }

    #[getter]
    pub fn event_type(&self) -> PyAuditEventType {
        PyAuditEventType { inner: self.inner.event_type.clone() }
    }

    #[getter]
    pub fn user_id(&self) -> String {
        self.inner.user_id.clone()
    }

    #[getter]
    pub fn action(&self) -> String {
        self.inner.action.clone()
    }

    #[getter]
    pub fn resource(&self) -> String {
        self.inner.resource.clone()
    }

    #[getter]
    pub fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    #[getter]
    pub fn outcome(&self) -> PyAuditOutcome {
        PyAuditOutcome { inner: self.inner.outcome.clone() }
    }

    #[getter]
    pub fn risk_level(&self) -> PyRiskLevel {
        PyRiskLevel { inner: self.inner.risk_level.clone() }
    }

    #[getter]
    pub fn data_classification(&self) -> PyDataClassification {
        PyDataClassification { inner: self.inner.data_classification.clone() }
    }

    #[getter]
    pub fn ip_address(&self) -> Option<String> {
        self.inner.ip_address.clone()
    }

    #[getter]
    pub fn session_id(&self) -> Option<String> {
        self.inner.session_id.clone()
    }

    #[getter]
    pub fn metadata(&self) -> HashMap<String, String> {
        self.inner.metadata.clone()
    }

    pub fn __str__(&self) -> String {
        format!(
            "AuditTrailEntry(user={}, event_type={}, resource={}, outcome={}, risk_level={}, timestamp={})",
            self.inner.user_id,
            PyAuditEventType { inner: self.inner.event_type.clone() }.__str__(),
            self.inner.resource,
            PyAuditOutcome { inner: self.inner.outcome.clone() }.__str__(),
            PyRiskLevel { inner: self.inner.risk_level.clone() }.__str__(),
            self.inner.timestamp.to_rfc3339()
        )
    }
}

#[pymethods]
impl PyComplianceReport {
    #[getter]
    pub fn report_id(&self) -> String {
        self.inner.report_id.clone()
    }

    #[getter]
    pub fn framework(&self) -> PyComplianceTag {
        PyComplianceTag { inner: self.inner.framework.clone() }
    }

    #[getter]
    pub fn generated_at(&self) -> String {
        self.inner.generated_at.to_rfc3339()
    }

    #[getter]
    pub fn period_start(&self) -> String {
        self.inner.period_start.to_rfc3339()
    }

    #[getter]
    pub fn period_end(&self) -> String {
        self.inner.period_end.to_rfc3339()
    }

    #[getter]
    pub fn total_events(&self) -> usize {
        self.inner.total_events
    }

    #[getter]
    pub fn security_violations(&self) -> usize {
        self.inner.security_violations
    }

    #[getter]
    pub fn policy_violations(&self) -> usize {
        self.inner.policy_violations
    }

    #[getter]
    pub fn failed_authentications(&self) -> usize {
        self.inner.failed_authentications
    }

    #[getter]
    pub fn privileged_operations(&self) -> usize {
        self.inner.privileged_operations
    }

    #[getter]
    pub fn data_access_events(&self) -> usize {
        self.inner.data_access_events
    }

    #[getter]
    pub fn integrity_status(&self) -> PyIntegrityStatus {
        PyIntegrityStatus { inner: self.inner.integrity_status.clone() }
    }

    #[getter]
    pub fn recommendations(&self) -> Vec<String> {
        self.inner.recommendations.clone()
    }

    pub fn __str__(&self) -> String {
        format!(
            "ComplianceReport(framework={}, total_events={}, security_violations={}, period={} to {})",
            PyComplianceTag { inner: self.inner.framework.clone() }.__str__(),
            self.inner.total_events,
            self.inner.security_violations,
            self.inner.period_start.format("%Y-%m-%d"),
            self.inner.period_end.format("%Y-%m-%d")
        )
    }
}

#[pymethods]
impl PyIntegrityStatus {
    #[getter]
    pub fn chain_verified(&self) -> bool {
        self.inner.chain_verified
    }

    #[getter]
    pub fn tamper_detected(&self) -> bool {
        self.inner.tamper_detected
    }

    #[getter]
    pub fn last_verification(&self) -> String {
        self.inner.last_verification.to_rfc3339()
    }

    #[getter]
    pub fn total_entries(&self) -> usize {
        self.inner.total_entries
    }

    #[getter]
    pub fn verification_errors(&self) -> Vec<String> {
        self.inner.verification_errors.clone()
    }

    pub fn __str__(&self) -> String {
        format!(
            "IntegrityStatus(verified={}, tamper_detected={}, total_entries={})",
            self.inner.chain_verified,
            self.inner.tamper_detected,
            self.inner.total_entries
        )
    }
}

// ============================================================================
// GDPR COMPLIANCE SYSTEM - Python Bindings
// ============================================================================

/// Python wrapper for GDPR Manager
#[pyclass(name = "GdprManager")]
pub struct PyGdprManager {
    pub(crate) inner: CoreGdprManager,
}

/// Python wrapper for DataSubject
#[pyclass(name = "DataSubject")]
#[derive(Clone)]
pub struct PyDataSubject {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreDataSubject,
}

/// Python wrapper for ConsentRecord
#[pyclass(name = "ConsentRecord")]
#[derive(Clone)]
pub struct PyConsentRecord {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreConsentRecord,
}

/// Python wrapper for SubjectRightsRequest
#[pyclass(name = "SubjectRightsRequest")]
#[derive(Clone)]
pub struct PySubjectRightsRequest {
    pub(crate) inner: CoreSubjectRightsRequest,
}

/// Python wrapper for BreachNotification
#[pyclass(name = "BreachNotification")]
#[derive(Clone)]
pub struct PyBreachNotification {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreBreachNotification,
}

/// Python wrapper for GdprComplianceStatus
#[pyclass(name = "GdprComplianceStatus")]
#[derive(Clone)]
pub struct PyGdprComplianceStatus {
    pub(crate) inner: CoreGdprComplianceStatus,
}

/// Python wrapper for GdprComplianceReport
#[pyclass(name = "GdprComplianceReport")]
#[derive(Clone)]
pub struct PyGdprComplianceReport {
    pub(crate) inner: CoreGdprComplianceReport,
}

/// Python wrapper for PersonalDataType
#[pyclass(name = "PersonalDataType")]
#[derive(Clone)]
pub struct PyPersonalDataType {
    pub(crate) inner: CorePersonalDataType,
}

/// Python wrapper for LawfulBasisType
#[pyclass(name = "LawfulBasisType")]
#[derive(Clone)]
pub struct PyLawfulBasisType {
    pub(crate) inner: CoreLawfulBasisType,
}

/// Python wrapper for ConsentMethod
#[pyclass(name = "ConsentMethod")]
#[derive(Clone)]
pub struct PyConsentMethod {
    pub(crate) inner: CoreConsentMethod,
}

/// Python wrapper for ConsentStatus
#[pyclass(name = "ConsentStatus")]
#[derive(Clone)]
pub struct PyConsentStatus {
    pub(crate) inner: CoreConsentStatus,
}

/// Python wrapper for DataSubjectRight
#[pyclass(name = "DataSubjectRight")]
#[derive(Clone)]
pub struct PyDataSubjectRight {
    pub(crate) inner: CoreDataSubjectRight,
}

/// Python wrapper for RequestStatus
#[pyclass(name = "RequestStatus")]
#[derive(Clone)]
pub struct PyRequestStatus {
    pub(crate) inner: CoreRequestStatus,
}

/// Python wrapper for BreachType
#[pyclass(name = "BreachType")]
#[derive(Clone)]
pub struct PyBreachType {
    pub(crate) inner: CoreBreachType,
}

/// Python wrapper for ExportFormat
#[pyclass(name = "ExportFormat")]
#[derive(Clone)]
pub struct PyExportFormat {
    pub(crate) inner: CoreExportFormat,
}

impl Default for PyGdprManager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyGdprManager {
    /// Create a new GDPR manager
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreGdprManager::new(),
        }
    }

    /// Create GDPR manager with EU configuration
    #[classmethod]
    pub fn with_eu_configuration(_cls: &PyType) -> Self {
        Self {
            inner: CoreGdprManager::with_eu_configuration(),
        }
    }

    /// Register a new data subject
    pub fn register_data_subject(
        &mut self,
        external_id: String,
        email: Option<String>,
        name: Option<String>,
    ) -> PyResult<String> {
        self.inner
            .register_data_subject(external_id, email, name)
            .map_err(map_rust_error_to_python)
    }

    /// Record consent from data subject
    pub fn record_consent(
        &mut self,
        data_subject_id: String,
        purpose: String,
        consent_text: String,
        consent_method: PyConsentMethod,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> PyResult<String> {
        let evidence = CoreConsentEvidence {
            timestamp: chrono::Utc::now(),
            ip_address,
            user_agent,
            form_version: None,
            witness: None,
            digital_signature: None,
            audit_trail: vec!["Consent recorded via Python API".to_string()],
        };

        self.inner
            .record_consent(
                data_subject_id,
                purpose,
                consent_text,
                consent_method.inner,
                evidence,
            )
            .map_err(map_rust_error_to_python)
    }

    /// Withdraw consent
    pub fn withdraw_consent(&mut self, consent_id: String, withdrawal_method: String) -> PyResult<()> {
        self.inner
            .withdraw_consent(consent_id, withdrawal_method)
            .map_err(map_rust_error_to_python)
    }

    /// Process data subject access request (Article 15)
    pub fn process_access_request(
        &mut self,
        data_subject_id: String,
        request_details: String,
    ) -> PyResult<PySubjectRightsRequest> {
        self.inner
            .process_access_request(data_subject_id, request_details)
            .map(|request| PySubjectRightsRequest { inner: request })
            .map_err(map_rust_error_to_python)
    }

    /// Process data rectification request (Article 16)
    pub fn process_rectification_request(
        &mut self,
        data_subject_id: String,
        corrections: HashMap<String, String>,
    ) -> PyResult<PySubjectRightsRequest> {
        self.inner
            .process_rectification_request(data_subject_id, corrections)
            .map(|request| PySubjectRightsRequest { inner: request })
            .map_err(map_rust_error_to_python)
    }

    /// Process data erasure request (Article 17 - Right to be Forgotten)
    pub fn process_erasure_request(
        &mut self,
        data_subject_id: String,
        erasure_grounds: String,
    ) -> PyResult<PySubjectRightsRequest> {
        self.inner
            .process_erasure_request(data_subject_id, erasure_grounds)
            .map(|request| PySubjectRightsRequest { inner: request })
            .map_err(map_rust_error_to_python)
    }

    /// Process data portability request (Article 20)
    pub fn process_portability_request(
        &mut self,
        data_subject_id: String,
        export_format: PyExportFormat,
    ) -> PyResult<PySubjectRightsRequest> {
        self.inner
            .process_portability_request(data_subject_id, export_format.inner)
            .map(|request| PySubjectRightsRequest { inner: request })
            .map_err(map_rust_error_to_python)
    }

    /// Report a personal data breach (Articles 33-34)
    pub fn report_data_breach(
        &mut self,
        breach_type: PyBreachType,
        affected_subjects: usize,
        affected_data_categories: Vec<PyPersonalDataType>,
        consequences: String,
        measures_taken: Vec<String>,
    ) -> PyResult<String> {
        let core_categories = affected_data_categories
            .into_iter()
            .map(|cat| cat.inner)
            .collect();

        self.inner
            .report_data_breach(
                breach_type.inner,
                affected_subjects,
                core_categories,
                consequences,
                measures_taken,
            )
            .map_err(map_rust_error_to_python)
    }

    /// Create Data Protection Impact Assessment
    pub fn create_dpia(&mut self, processing_operation: String, description: String) -> PyResult<String> {
        self.inner
            .create_dpia(processing_operation, description)
            .map_err(map_rust_error_to_python)
    }

    /// Get compliance status overview
    pub fn get_compliance_status(&self) -> PyGdprComplianceStatus {
        let status = self.inner.get_compliance_status();
        PyGdprComplianceStatus { inner: status }
    }

    /// Generate GDPR compliance report
    pub fn generate_gdpr_compliance_report(
        &self,
        start_date: String,
        end_date: String,
    ) -> PyResult<PyGdprComplianceReport> {
        use chrono::DateTime;

        let start_dt = DateTime::parse_from_rfc3339(&start_date)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid start_date format: {e}")))?
            .with_timezone(&chrono::Utc);

        let end_dt = DateTime::parse_from_rfc3339(&end_date)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid end_date format: {e}")))?
            .with_timezone(&chrono::Utc);

        let report = self.inner.generate_gdpr_compliance_report(start_dt, end_dt);
        Ok(PyGdprComplianceReport { inner: report })
    }
}

#[pymethods]
impl PyPersonalDataType {
    #[classmethod]
    pub fn basic_personal_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::BasicPersonalData }
    }

    #[classmethod]
    pub fn special_category_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::SpecialCategoryData }
    }

    #[classmethod]
    pub fn biometric_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::BiometricData }
    }

    #[classmethod]
    pub fn location_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::LocationData }
    }

    #[classmethod]
    pub fn behavioral_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::BehavioralData }
    }

    #[classmethod]
    pub fn communication_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::CommunicationData }
    }

    #[classmethod]
    pub fn financial_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::FinancialData }
    }

    #[classmethod]
    pub fn identification_data(_cls: &PyType) -> Self {
        Self { inner: CorePersonalDataType::IdentificationData }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CorePersonalDataType::BasicPersonalData => "BasicPersonalData",
            CorePersonalDataType::SpecialCategoryData => "SpecialCategoryData",
            CorePersonalDataType::BiometricData => "BiometricData",
            CorePersonalDataType::LocationData => "LocationData",
            CorePersonalDataType::BehavioralData => "BehavioralData",
            CorePersonalDataType::CommunicationData => "CommunicationData",
            CorePersonalDataType::FinancialData => "FinancialData",
            CorePersonalDataType::IdentificationData => "IdentificationData",
            CorePersonalDataType::TechnicalData => "TechnicalData",
            CorePersonalDataType::ProfessionalData => "ProfessionalData",
        }
    }
}

#[pymethods]
impl PyLawfulBasisType {
    #[classmethod]
    pub fn consent(_cls: &PyType) -> Self {
        Self { inner: CoreLawfulBasisType::Consent }
    }

    #[classmethod]
    pub fn contract(_cls: &PyType) -> Self {
        Self { inner: CoreLawfulBasisType::Contract }
    }

    #[classmethod]
    pub fn legal_obligation(_cls: &PyType) -> Self {
        Self { inner: CoreLawfulBasisType::LegalObligation }
    }

    #[classmethod]
    pub fn vital_interests(_cls: &PyType) -> Self {
        Self { inner: CoreLawfulBasisType::VitalInterests }
    }

    #[classmethod]
    pub fn public_task(_cls: &PyType) -> Self {
        Self { inner: CoreLawfulBasisType::PublicTask }
    }

    #[classmethod]
    pub fn legitimate_interests(_cls: &PyType) -> Self {
        Self { inner: CoreLawfulBasisType::LegitimateInterests }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreLawfulBasisType::Consent => "Consent",
            CoreLawfulBasisType::Contract => "Contract",
            CoreLawfulBasisType::LegalObligation => "LegalObligation",
            CoreLawfulBasisType::VitalInterests => "VitalInterests",
            CoreLawfulBasisType::PublicTask => "PublicTask",
            CoreLawfulBasisType::LegitimateInterests => "LegitimateInterests",
        }
    }
}

#[pymethods]
impl PyConsentMethod {
    #[classmethod]
    pub fn web_form(_cls: &PyType) -> Self {
        Self { inner: CoreConsentMethod::WebForm }
    }

    #[classmethod]
    pub fn email(_cls: &PyType) -> Self {
        Self { inner: CoreConsentMethod::Email }
    }

    #[classmethod]
    pub fn physical_document(_cls: &PyType) -> Self {
        Self { inner: CoreConsentMethod::PhysicalDocument }
    }

    #[classmethod]
    pub fn api_call(_cls: &PyType) -> Self {
        Self { inner: CoreConsentMethod::APICall }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreConsentMethod::WebForm => "WebForm",
            CoreConsentMethod::Email => "Email",
            CoreConsentMethod::PhysicalDocument => "PhysicalDocument",
            CoreConsentMethod::VerbalConsent => "VerbalConsent",
            CoreConsentMethod::ImpliedConsent => "ImpliedConsent",
            CoreConsentMethod::APICall => "APICall",
        }
    }
}

#[pymethods]
impl PyConsentStatus {
    #[classmethod]
    pub fn given(_cls: &PyType) -> Self {
        Self { inner: CoreConsentStatus::Given }
    }

    #[classmethod]
    pub fn withdrawn(_cls: &PyType) -> Self {
        Self { inner: CoreConsentStatus::Withdrawn }
    }

    #[classmethod]
    pub fn expired(_cls: &PyType) -> Self {
        Self { inner: CoreConsentStatus::Expired }
    }

    #[classmethod]
    pub fn refused(_cls: &PyType) -> Self {
        Self { inner: CoreConsentStatus::Refused }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreConsentStatus::Given => "Given",
            CoreConsentStatus::Withdrawn => "Withdrawn",
            CoreConsentStatus::Expired => "Expired",
            CoreConsentStatus::Refused => "Refused",
            CoreConsentStatus::Pending => "Pending",
        }
    }
}

#[pymethods]
impl PyDataSubjectRight {
    #[classmethod]
    pub fn right_to_information(_cls: &PyType) -> Self {
        Self { inner: CoreDataSubjectRight::RightToInformation }
    }

    #[classmethod]
    pub fn right_of_access(_cls: &PyType) -> Self {
        Self { inner: CoreDataSubjectRight::RightOfAccess }
    }

    #[classmethod]
    pub fn right_to_rectification(_cls: &PyType) -> Self {
        Self { inner: CoreDataSubjectRight::RightToRectification }
    }

    #[classmethod]
    pub fn right_to_erasure(_cls: &PyType) -> Self {
        Self { inner: CoreDataSubjectRight::RightToErasure }
    }

    #[classmethod]
    pub fn right_to_data_portability(_cls: &PyType) -> Self {
        Self { inner: CoreDataSubjectRight::RightToDataPortability }
    }

    #[classmethod]
    pub fn right_to_object(_cls: &PyType) -> Self {
        Self { inner: CoreDataSubjectRight::RightToObject }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreDataSubjectRight::RightToInformation => "RightToInformation",
            CoreDataSubjectRight::RightOfAccess => "RightOfAccess",
            CoreDataSubjectRight::RightToRectification => "RightToRectification",
            CoreDataSubjectRight::RightToErasure => "RightToErasure",
            CoreDataSubjectRight::RightToRestrictProcessing => "RightToRestrictProcessing",
            CoreDataSubjectRight::RightToDataPortability => "RightToDataPortability",
            CoreDataSubjectRight::RightToObject => "RightToObject",
            CoreDataSubjectRight::RightsRelatedToAutomatedDecisionMaking => "RightsRelatedToAutomatedDecisionMaking",
        }
    }
}

#[pymethods]
impl PyBreachType {
    #[classmethod]
    pub fn confidentiality_breach(_cls: &PyType) -> Self {
        Self { inner: CoreBreachType::ConfidentialityBreach }
    }

    #[classmethod]
    pub fn integrity_breach(_cls: &PyType) -> Self {
        Self { inner: CoreBreachType::IntegrityBreach }
    }

    #[classmethod]
    pub fn availability_breach(_cls: &PyType) -> Self {
        Self { inner: CoreBreachType::AvailabilityBreach }
    }

    pub fn __str__(&self) -> &'static str {
        match &self.inner {
            CoreBreachType::ConfidentialityBreach => "ConfidentialityBreach",
            CoreBreachType::IntegrityBreach => "IntegrityBreach",
            CoreBreachType::AvailabilityBreach => "AvailabilityBreach",
            CoreBreachType::Combined(_) => "CombinedBreach",
        }
    }
}

#[pymethods]
impl PyExportFormat {
    #[classmethod]
    pub fn json(_cls: &PyType) -> Self {
        Self { inner: CoreExportFormat::Json }
    }

    #[classmethod]
    pub fn xml(_cls: &PyType) -> Self {
        Self { inner: CoreExportFormat::Xml }
    }

    #[classmethod]
    pub fn csv(_cls: &PyType) -> Self {
        Self { inner: CoreExportFormat::Csv }
    }

    #[classmethod]
    pub fn pdf(_cls: &PyType) -> Self {
        Self { inner: CoreExportFormat::Pdf }
    }

    pub fn __str__(&self) -> &'static str {
        match &self.inner {
            CoreExportFormat::Json => "Json",
            CoreExportFormat::Xml => "Xml",
            CoreExportFormat::Csv => "Csv",
            CoreExportFormat::Pdf => "Pdf",
            CoreExportFormat::StructuredFormat(_) => "StructuredFormat",
        }
    }
}

#[pymethods]
impl PySubjectRightsRequest {
    #[getter]
    pub fn request_id(&self) -> String {
        self.inner.request_id.clone()
    }

    #[getter]
    pub fn data_subject_id(&self) -> String {
        self.inner.data_subject_id.clone()
    }

    #[getter]
    pub fn request_type(&self) -> PyDataSubjectRight {
        PyDataSubjectRight { inner: self.inner.request_type.clone() }
    }

    #[getter]
    pub fn request_details(&self) -> String {
        self.inner.request_details.clone()
    }

    #[getter]
    pub fn requested_at(&self) -> String {
        self.inner.requested_at.to_rfc3339()
    }

    #[getter]
    pub fn request_status(&self) -> PyRequestStatus {
        PyRequestStatus { inner: self.inner.request_status.clone() }
    }

    pub fn __str__(&self) -> String {
        format!(
            "SubjectRightsRequest(id={}, type={}, subject_id={}, status={}, requested_at={})",
            self.inner.request_id,
            PyDataSubjectRight { inner: self.inner.request_type.clone() }.__str__(),
            self.inner.data_subject_id,
            PyRequestStatus { inner: self.inner.request_status.clone() }.__str__(),
            self.inner.requested_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[pymethods]
impl PyRequestStatus {
    #[classmethod]
    pub fn received(_cls: &PyType) -> Self {
        Self { inner: CoreRequestStatus::Received }
    }

    #[classmethod]
    pub fn in_progress(_cls: &PyType) -> Self {
        Self { inner: CoreRequestStatus::InProgress }
    }

    #[classmethod]
    pub fn completed(_cls: &PyType) -> Self {
        Self { inner: CoreRequestStatus::Completed }
    }

    #[classmethod]
    pub fn rejected(_cls: &PyType) -> Self {
        Self { inner: CoreRequestStatus::Rejected }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreRequestStatus::Received => "Received",
            CoreRequestStatus::IdentityVerificationRequired => "IdentityVerificationRequired",
            CoreRequestStatus::InProgress => "InProgress",
            CoreRequestStatus::Completed => "Completed",
            CoreRequestStatus::Rejected => "Rejected",
            CoreRequestStatus::PartiallyFulfilled => "PartiallyFulfilled",
            CoreRequestStatus::Extended => "Extended",
        }
    }
}

#[pymethods]
impl PyGdprComplianceStatus {
    #[getter]
    pub fn total_data_subjects(&self) -> usize {
        self.inner.total_data_subjects
    }

    #[getter]
    pub fn active_consents(&self) -> usize {
        self.inner.active_consents
    }

    #[getter]
    pub fn withdrawn_consents(&self) -> usize {
        self.inner.withdrawn_consents
    }

    #[getter]
    pub fn pending_subject_requests(&self) -> usize {
        self.inner.pending_subject_requests
    }

    #[getter]
    pub fn completed_subject_requests(&self) -> usize {
        self.inner.completed_subject_requests
    }

    #[getter]
    pub fn total_data_breaches(&self) -> usize {
        self.inner.total_data_breaches
    }

    #[getter]
    pub fn unresolved_breaches(&self) -> usize {
        self.inner.unresolved_breaches
    }

    #[getter]
    pub fn deletion_records(&self) -> usize {
        self.inner.deletion_records
    }

    pub fn __str__(&self) -> String {
        format!(
            "GdprComplianceStatus(subjects={}, active_consents={}, pending_requests={}, breaches={})",
            self.inner.total_data_subjects,
            self.inner.active_consents,
            self.inner.pending_subject_requests,
            self.inner.total_data_breaches
        )
    }
}

#[pymethods]
impl PyGdprComplianceReport {
    #[getter]
    pub fn report_id(&self) -> String {
        self.inner.report_id.clone()
    }

    #[getter]
    pub fn generated_at(&self) -> String {
        self.inner.generated_at.to_rfc3339()
    }

    #[getter]
    pub fn period_start(&self) -> String {
        self.inner.period_start.to_rfc3339()
    }

    #[getter]
    pub fn period_end(&self) -> String {
        self.inner.period_end.to_rfc3339()
    }

    #[getter]
    pub fn total_data_subjects(&self) -> usize {
        self.inner.total_data_subjects
    }

    #[getter]
    pub fn subject_requests_received(&self) -> usize {
        self.inner.subject_requests_received
    }

    #[getter]
    pub fn subject_requests_completed_on_time(&self) -> usize {
        self.inner.subject_requests_completed_on_time
    }

    #[getter]
    pub fn data_breaches_reported(&self) -> usize {
        self.inner.data_breaches_reported
    }

    #[getter]
    pub fn breaches_reported_within_72h(&self) -> usize {
        self.inner.breaches_reported_within_72h
    }

    #[getter]
    pub fn compliance_score(&self) -> f64 {
        self.inner.compliance_score
    }

    #[getter]
    pub fn recommendations(&self) -> Vec<String> {
        self.inner.recommendations.clone()
    }

    #[getter]
    pub fn key_risks_identified(&self) -> Vec<String> {
        self.inner.key_risks_identified.clone()
    }

    pub fn __str__(&self) -> String {
        format!(
            "GdprComplianceReport(id={}, subjects={}, compliance_score={:.1}%, period={} to {})",
            self.inner.report_id,
            self.inner.total_data_subjects,
            self.inner.compliance_score,
            self.inner.period_start.format("%Y-%m-%d"),
            self.inner.period_end.format("%Y-%m-%d")
        )
    }
}

// ============================================================================
// DIGITAL SIGNATURES SYSTEM - Python Bindings
// ============================================================================

/// Python wrapper for EventSigner
#[pyclass(name = "EventSigner")]
pub struct PyEventSigner {
    pub(crate) inner: CoreEventSigner,
}

/// Python wrapper for SigningKeyManager
#[pyclass(name = "SigningKeyManager")]
#[derive(Clone)]
pub struct PySigningKeyManager {
    pub(crate) inner: CoreSigningKeyManager,
}

/// Python wrapper for SigningKey
#[pyclass(name = "SigningKey")]
#[derive(Clone)]
pub struct PySigningKey {
    pub(crate) inner: CoreSigningKey,
}

/// Python wrapper for SignatureAlgorithm
#[pyclass(name = "SignatureAlgorithm")]
#[derive(Clone)]
pub struct PySignatureAlgorithm {
    pub(crate) inner: CoreSignatureAlgorithm,
}

/// Python wrapper for EventSignature
#[pyclass(name = "EventSignature")]
#[derive(Clone)]
pub struct PyEventSignature {
    pub(crate) inner: CoreEventSignature,
}

/// Python wrapper for SignedEvent
#[pyclass(name = "SignedEvent")]
#[derive(Clone)]
pub struct PySignedEvent {
    pub(crate) inner: CoreSignedEvent,
}

#[pymethods]
impl PyEventSigner {
    /// Create new signer with key manager
    #[classmethod]
    pub fn new(_cls: &PyType, key_manager: PySigningKeyManager) -> Self {
        Self {
            inner: CoreEventSigner::new(key_manager.inner),
        }
    }

    /// Create signer with a single key
    #[classmethod]
    pub fn with_key(_cls: &PyType, key_id: String, key_data: Vec<u8>) -> PyResult<Self> {
        CoreEventSigner::with_key(key_id, key_data)
            .map(|inner| Self { inner })
            .map_err(map_rust_error_to_python)
    }

    /// Sign an event using default key
    pub fn sign_event(&self, event: &PyEvent) -> PyResult<PySignedEvent> {
        self.inner
            .sign_event(&event.inner)
            .map(|signed| PySignedEvent { inner: signed })
            .map_err(map_rust_error_to_python)
    }

    /// Sign event with specific key
    pub fn sign_event_with_key(&self, event: &PyEvent, key_id: &str) -> PyResult<PySignedEvent> {
        self.inner
            .sign_event_with_key(&event.inner, key_id)
            .map(|signed| PySignedEvent { inner: signed })
            .map_err(map_rust_error_to_python)
    }

    /// Verify event signature
    pub fn verify_signature(&self, signed_event: &PySignedEvent) -> PyResult<bool> {
        self.inner
            .verify_signature(&signed_event.inner)
            .map_err(map_rust_error_to_python)
    }

    /// Sign raw data
    pub fn sign_data(&self, data: Vec<u8>, key_id: &str) -> PyResult<PyEventSignature> {
        self.inner
            .sign_data(&data, key_id)
            .map(|sig| PyEventSignature { inner: sig })
            .map_err(map_rust_error_to_python)
    }

    /// Verify data signature
    pub fn verify_data_signature(&self, data: Vec<u8>, signature: &PyEventSignature) -> PyResult<bool> {
        self.inner
            .verify_data_signature(&data, &signature.inner)
            .map_err(map_rust_error_to_python)
    }
}

impl Default for PySigningKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PySigningKeyManager {
    /// Create new signing key manager
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreSigningKeyManager::new(),
        }
    }

    /// Generate new signing key
    #[classmethod]
    pub fn generate_key(_cls: &PyType, id: String, algorithm: PySignatureAlgorithm) -> PyResult<PySigningKey> {
        CoreSigningKeyManager::generate_key(id, algorithm.inner)
            .map(|key| PySigningKey { inner: key })
            .map_err(map_rust_error_to_python)
    }

    /// Derive key from password
    #[classmethod]
    pub fn derive_key_from_password(
        _cls: &PyType,
        id: String,
        password: String,
        salt: Vec<u8>,
        algorithm: PySignatureAlgorithm,
    ) -> PyResult<PySigningKey> {
        CoreSigningKeyManager::derive_key_from_password(id, &password, &salt, algorithm.inner)
            .map(|key| PySigningKey { inner: key })
            .map_err(map_rust_error_to_python)
    }

    /// Add key to manager
    pub fn add_key(&mut self, key: PySigningKey) -> PyResult<()> {
        self.inner
            .add_key(key.inner)
            .map_err(map_rust_error_to_python)
    }

    /// Set default key
    pub fn set_default_key(&mut self, key_id: &str) -> PyResult<()> {
        self.inner
            .set_default_key(key_id)
            .map_err(map_rust_error_to_python)
    }

    /// List all key IDs
    pub fn list_key_ids(&self) -> Vec<String> {
        self.inner.list_key_ids()
    }
}

#[pymethods]
impl PySignatureAlgorithm {
    #[classmethod]
    pub fn hmac_sha256(_cls: &PyType) -> Self {
        Self {
            inner: CoreSignatureAlgorithm::HmacSha256,
        }
    }

    #[classmethod]
    pub fn hmac_sha512(_cls: &PyType) -> Self {
        Self {
            inner: CoreSignatureAlgorithm::HmacSha512,
        }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreSignatureAlgorithm::HmacSha256 => "HMAC-SHA256",
            CoreSignatureAlgorithm::HmacSha512 => "HMAC-SHA512",
        }
    }

    /// Get key size for algorithm
    pub fn key_size(&self) -> usize {
        self.inner.key_size()
    }

    /// Get signature size for algorithm
    pub fn signature_size(&self) -> usize {
        self.inner.signature_size()
    }
}

#[pymethods]
impl PySigningKey {
    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    pub fn algorithm(&self) -> PySignatureAlgorithm {
        PySignatureAlgorithm {
            inner: self.inner.algorithm.clone(),
        }
    }

    #[getter]
    pub fn created_at(&self) -> String {
        self.inner.created_at.to_rfc3339()
    }

    #[getter]
    pub fn key_length(&self) -> usize {
        self.inner.key_data.len()
    }
}

#[pymethods]
impl PyEventSignature {
    #[getter]
    pub fn algorithm(&self) -> PySignatureAlgorithm {
        PySignatureAlgorithm {
            inner: self.inner.algorithm.clone(),
        }
    }

    #[getter]
    pub fn key_id(&self) -> String {
        self.inner.key_id.clone()
    }

    #[getter]
    pub fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    #[getter]
    pub fn signature_length(&self) -> usize {
        self.inner.signature.len()
    }

    /// Serialize to base64
    pub fn to_base64(&self) -> String {
        self.inner.to_base64()
    }

    /// Deserialize from base64
    #[classmethod]
    pub fn from_base64(_cls: &PyType, data: String) -> PyResult<Self> {
        CoreEventSignature::from_base64(&data)
            .map(|sig| Self { inner: sig })
            .map_err(map_rust_error_to_python)
    }
}

#[pymethods]
impl PySignedEvent {
    #[getter]
    pub fn event(&self) -> PyEvent {
        PyEvent {
            inner: self.inner.event.clone(),
        }
    }

    #[getter]
    pub fn signature(&self) -> PyEventSignature {
        PyEventSignature {
            inner: self.inner.signature.clone(),
        }
    }

    /// Serialize to base64
    pub fn to_base64(&self) -> String {
        self.inner.to_base64()
    }

    /// Deserialize from base64
    #[classmethod]
    pub fn from_base64(_cls: &PyType, data: String) -> PyResult<Self> {
        CoreSignedEvent::from_base64(&data)
            .map(|signed| Self { inner: signed })
            .map_err(map_rust_error_to_python)
    }
}

// ============================================================================
// DATA RETENTION POLICIES - Python Bindings
// ============================================================================

/// Python wrapper for RetentionPolicyManager
#[pyclass(name = "RetentionPolicyManager")]
pub struct PyRetentionPolicyManager {
    pub(crate) inner: CoreRetentionPolicyManager,
}

/// Python wrapper for RetentionPolicy
#[pyclass(name = "RetentionPolicy")]
#[derive(Clone)]
pub struct PyRetentionPolicy {
    pub(crate) inner: CoreRetentionPolicy,
}

/// Python wrapper for RetentionPeriod
#[pyclass(name = "RetentionPeriod")]
#[derive(Clone)]
pub struct PyRetentionPeriod {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreRetentionPeriod,
}

/// Python wrapper for DeletionMethod
#[pyclass(name = "DeletionMethod")]
#[derive(Clone)]
pub struct PyDeletionMethod {
    pub(crate) inner: CoreDeletionMethod,
}

/// Python wrapper for DataCategory
#[pyclass(name = "DataCategory")]
#[derive(Clone)]
pub struct PyDataCategory {
    pub(crate) inner: CoreDataCategory,
}

/// Python wrapper for RetentionEnforcementResult
#[pyclass(name = "RetentionEnforcementResult")]
#[derive(Clone)]
pub struct PyRetentionEnforcementResult {
    #[allow(dead_code)] // Inner field used by PyO3 for Python integration
    pub(crate) inner: CoreRetentionEnforcementResult,
}

/// Python wrapper for LegalHold
#[pyclass(name = "LegalHold")]
#[derive(Clone)]
pub struct PyLegalHold {
    pub(crate) inner: CoreLegalHold,
}

/// Python wrapper for EventDataClassification
#[pyclass(name = "EventDataClassification")]
#[derive(Clone)]
pub struct PyEventDataClassification {
    pub(crate) inner: CoreEventDataClassification,
}

impl Default for PyRetentionPolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyRetentionPolicyManager {
    /// Create new retention policy manager
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreRetentionPolicyManager::new(),
        }
    }

    /// Add retention policy
    pub fn add_policy(&mut self, policy: PyRetentionPolicy) -> PyResult<()> {
        self.inner
            .add_policy(policy.inner)
            .map_err(map_rust_error_to_python)
    }

    /// Set default policy
    pub fn set_default_policy(&mut self, name: &str) -> PyResult<()> {
        self.inner
            .set_default_policy(name)
            .map_err(map_rust_error_to_python)
    }

    /// Classify event data
    pub fn classify_event(&self, event: &PyEvent) -> PyResult<PyEventDataClassification> {
        self.inner
            .classify_event(&event.inner)
            .map(|classification| PyEventDataClassification { inner: classification })
            .map_err(map_rust_error_to_python)
    }

    /// List all policy names
    pub fn list_policies(&self) -> Vec<String> {
        self.inner.list_policies()
    }

    /// Get retention statistics
    pub fn get_retention_stats(&self) -> HashMap<String, usize> {
        self.inner.get_retention_stats()
    }
}

#[pymethods]
impl PyRetentionPolicy {
    /// Create GDPR default policy
    #[classmethod]
    pub fn gdpr_default(_cls: &PyType) -> Self {
        Self {
            inner: CoreRetentionPolicy::gdpr_default(),
        }
    }

    /// Create financial data policy
    #[classmethod]
    pub fn financial_data_policy(_cls: &PyType) -> Self {
        Self {
            inner: CoreRetentionPolicy::financial_data_policy(),
        }
    }

    /// Create health data policy
    #[classmethod]
    pub fn health_data_policy(_cls: &PyType) -> Self {
        Self {
            inner: CoreRetentionPolicy::health_data_policy(),
        }
    }

    /// Create marketing data policy
    #[classmethod]
    pub fn marketing_data_policy(_cls: &PyType) -> Self {
        Self {
            inner: CoreRetentionPolicy::marketing_data_policy(),
        }
    }

    #[getter]
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    pub fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    pub fn deletion_method(&self) -> PyDeletionMethod {
        PyDeletionMethod {
            inner: self.inner.deletion_method.clone(),
        }
    }

    #[getter]
    pub fn legal_hold_exempt(&self) -> bool {
        self.inner.legal_hold_exempt
    }

    /// Check if policy applies to data category
    pub fn applies_to_category(&self, category: PyDataCategory) -> bool {
        self.inner.data_categories.contains(&category.inner)
    }

    /// Update timestamp
    pub fn touch(&mut self) {
        self.inner.updated_at = chrono::Utc::now();
    }
}

#[pymethods]
impl PyDeletionMethod {
    #[classmethod]
    pub fn soft_delete(_cls: &PyType) -> Self {
        Self {
            inner: CoreDeletionMethod::SoftDelete,
        }
    }

    #[classmethod]
    pub fn hard_delete(_cls: &PyType) -> Self {
        Self {
            inner: CoreDeletionMethod::HardDelete,
        }
    }

    #[classmethod]
    pub fn anonymize(_cls: &PyType) -> Self {
        Self {
            inner: CoreDeletionMethod::Anonymize,
        }
    }

    #[classmethod]
    pub fn archive(_cls: &PyType) -> Self {
        Self {
            inner: CoreDeletionMethod::Archive,
        }
    }

    #[classmethod]
    pub fn encrypt(_cls: &PyType) -> Self {
        Self {
            inner: CoreDeletionMethod::Encrypt,
        }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreDeletionMethod::SoftDelete => "SoftDelete",
            CoreDeletionMethod::HardDelete => "HardDelete",
            CoreDeletionMethod::Anonymize => "Anonymize",
            CoreDeletionMethod::Archive => "Archive",
            CoreDeletionMethod::Encrypt => "Encrypt",
        }
    }
}

#[pymethods]
impl PyDataCategory {
    #[classmethod]
    pub fn personal_data(_cls: &PyType) -> Self {
        Self {
            inner: CoreDataCategory::PersonalData,
        }
    }

    #[classmethod]
    pub fn sensitive_personal_data(_cls: &PyType) -> Self {
        Self {
            inner: CoreDataCategory::SensitivePersonalData,
        }
    }

    #[classmethod]
    pub fn financial_data(_cls: &PyType) -> Self {
        Self {
            inner: CoreDataCategory::FinancialData,
        }
    }

    #[classmethod]
    pub fn health_data(_cls: &PyType) -> Self {
        Self {
            inner: CoreDataCategory::HealthData,
        }
    }

    #[classmethod]
    pub fn operational_data(_cls: &PyType) -> Self {
        Self {
            inner: CoreDataCategory::OperationalData,
        }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreDataCategory::PersonalData => "PersonalData",
            CoreDataCategory::SensitivePersonalData => "SensitivePersonalData",
            CoreDataCategory::FinancialData => "FinancialData",
            CoreDataCategory::HealthData => "HealthData",
            CoreDataCategory::CommunicationData => "CommunicationData",
            CoreDataCategory::BehavioralData => "BehavioralData",
            CoreDataCategory::TechnicalData => "TechnicalData",
            CoreDataCategory::MarketingData => "MarketingData",
            CoreDataCategory::OperationalData => "OperationalData",
            CoreDataCategory::LegalData => "LegalData",
            CoreDataCategory::AuditData => "AuditData",
            CoreDataCategory::BackupData => "BackupData",
        }
    }
}

#[pymethods]
impl PyLegalHold {
    /// Create new legal hold
    #[classmethod]
    pub fn new(
        _cls: &PyType,
        id: String,
        reason: String,
        authority: String,
        data_categories: Vec<PyDataCategory>,
        aggregate_patterns: Vec<String>,
        created_by: String,
    ) -> Self {
        let core_categories = data_categories.into_iter().map(|cat| cat.inner).collect();
        Self {
            inner: CoreLegalHold::new(
                id,
                reason,
                authority,
                core_categories,
                aggregate_patterns,
                created_by,
            ),
        }
    }

    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    pub fn reason(&self) -> String {
        self.inner.reason.clone()
    }

    #[getter]
    pub fn authority(&self) -> String {
        self.inner.authority.clone()
    }

    /// Release legal hold
    pub fn release(&mut self) {
        self.inner.release()
    }

    /// Check if hold is active
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}

#[pymethods]
impl PyEventDataClassification {
    #[getter]
    pub fn event_id(&self) -> String {
        self.inner.event_id.clone()
    }

    #[getter]
    pub fn aggregate_id(&self) -> String {
        self.inner.aggregate_id.clone()
    }

    #[getter]
    pub fn retention_policy(&self) -> String {
        self.inner.retention_policy.clone()
    }

    #[getter]
    pub fn classified_at(&self) -> String {
        self.inner.classified_at.to_rfc3339()
    }

    #[getter]
    pub fn expires_at(&self) -> Option<String> {
        self.inner.expires_at.map(|dt| dt.to_rfc3339())
    }
}

// ============================================================================
// VULNERABILITY SCANNING - Python Bindings
// ============================================================================

/// Python wrapper for VulnerabilityScanner
#[pyclass(name = "VulnerabilityScanner")]
pub struct PyVulnerabilityScanner {
    pub(crate) inner: CoreVulnerabilityScanner,
}

/// Python wrapper for VulnerabilityScanResult
#[pyclass(name = "VulnerabilityScanResult")]
#[derive(Clone)]
pub struct PyVulnerabilityScanResult {
    pub(crate) inner: CoreVulnerabilityScanResult,
}

/// Python wrapper for VulnerabilityFinding
#[pyclass(name = "VulnerabilityFinding")]
#[derive(Clone)]
pub struct PyVulnerabilityFinding {
    pub(crate) inner: CoreVulnerabilityFinding,
}

/// Python wrapper for VulnerabilityCategory
#[pyclass(name = "VulnerabilityCategory")]
#[derive(Clone)]
pub struct PyVulnerabilityCategory {
    pub(crate) inner: CoreVulnerabilityCategory,
}

/// Python wrapper for VulnerabilitySeverity
#[pyclass(name = "VulnerabilitySeverity")]
#[derive(Clone)]
pub struct PyVulnerabilitySeverity {
    pub(crate) inner: CoreVulnerabilitySeverity,
}

/// Python wrapper for PenetrationTestFramework
#[pyclass(name = "PenetrationTestFramework")]
pub struct PyPenetrationTestFramework {
    pub(crate) inner: CorePenetrationTestFramework,
}

/// Python wrapper for PenetrationTest
#[pyclass(name = "PenetrationTest")]
#[derive(Clone)]
pub struct PyPenetrationTest {
    pub(crate) inner: CorePenetrationTest,
}

impl Default for PyVulnerabilityScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyVulnerabilityScanner {
    /// Create new vulnerability scanner
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CoreVulnerabilityScanner::new(),
        }
    }

    /// Scan events for vulnerabilities
    pub fn scan_events(&self, events: Vec<PyEvent>) -> PyResult<PyVulnerabilityScanResult> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create tokio runtime: {e}")))?;
        
        let core_events = events.into_iter().map(|e| e.inner).collect();
        
        rt.block_on(async {
            self.inner
                .scan_events(core_events)
                .await
                .map(|result| PyVulnerabilityScanResult { inner: result })
                .map_err(map_rust_error_to_python)
        })
    }

    /// Add aggregate to whitelist
    pub fn add_to_whitelist(&mut self, aggregate_id: String) {
        self.inner.add_to_whitelist(aggregate_id)
    }

    /// Remove aggregate from whitelist
    pub fn remove_from_whitelist(&mut self, aggregate_id: &str) -> bool {
        self.inner.remove_from_whitelist(aggregate_id)
    }

    /// Get scan statistics
    pub fn get_scan_statistics(&self) -> HashMap<String, usize> {
        self.inner.get_scan_statistics()
    }
}

#[pymethods]
impl PyVulnerabilityScanResult {
    #[getter]
    pub fn scan_id(&self) -> String {
        self.inner.scan_id.clone()
    }

    #[getter]
    pub fn events_scanned(&self) -> usize {
        self.inner.events_scanned
    }

    #[getter]
    pub fn vulnerabilities_found(&self) -> Vec<PyVulnerabilityFinding> {
        self.inner
            .vulnerabilities_found
            .iter()
            .map(|finding| PyVulnerabilityFinding {
                inner: finding.clone(),
            })
            .collect()
    }

    #[getter]
    pub fn scan_duration_ms(&self) -> u64 {
        self.inner.scan_duration_ms
    }

    #[getter]
    pub fn compliance_score(&self) -> f64 {
        self.inner.compliance_score
    }

    #[getter]
    pub fn scan_timestamp(&self) -> String {
        self.inner.scan_timestamp.to_rfc3339()
    }
}

#[pymethods]
impl PyVulnerabilityFinding {
    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    pub fn event_id(&self) -> String {
        self.inner.event_id.clone()
    }

    #[getter]
    pub fn category(&self) -> PyVulnerabilityCategory {
        PyVulnerabilityCategory {
            inner: self.inner.category.clone(),
        }
    }

    #[getter]
    pub fn severity(&self) -> PyVulnerabilitySeverity {
        PyVulnerabilitySeverity {
            inner: self.inner.severity.clone(),
        }
    }

    #[getter]
    pub fn title(&self) -> String {
        self.inner.title.clone()
    }

    #[getter]
    pub fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    pub fn evidence(&self) -> String {
        self.inner.evidence.clone()
    }

    #[getter]
    pub fn remediation(&self) -> String {
        self.inner.remediation.clone()
    }

    #[getter]
    pub fn found_at(&self) -> String {
        self.inner.found_at.to_rfc3339()
    }
}

#[pymethods]
impl PyVulnerabilityCategory {
    #[classmethod]
    pub fn data_leakage(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilityCategory::DataLeakage,
        }
    }

    #[classmethod]
    pub fn injection_attack(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilityCategory::InjectionAttack,
        }
    }

    #[classmethod]
    pub fn access_control(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilityCategory::AccessControl,
        }
    }

    #[classmethod]
    pub fn authentication(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilityCategory::Authentication,
        }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreVulnerabilityCategory::DataLeakage => "DataLeakage",
            CoreVulnerabilityCategory::InjectionAttack => "InjectionAttack",
            CoreVulnerabilityCategory::AccessControl => "AccessControl",
            CoreVulnerabilityCategory::Cryptographic => "Cryptographic",
            CoreVulnerabilityCategory::Authentication => "Authentication",
            CoreVulnerabilityCategory::Authorization => "Authorization",
            CoreVulnerabilityCategory::InputValidation => "InputValidation",
            CoreVulnerabilityCategory::OutputEncoding => "OutputEncoding",
            CoreVulnerabilityCategory::SessionManagement => "SessionManagement",
            CoreVulnerabilityCategory::ErrorHandling => "ErrorHandling",
            CoreVulnerabilityCategory::Logging => "Logging",
            CoreVulnerabilityCategory::Configuration => "Configuration",
            CoreVulnerabilityCategory::BusinessLogic => "BusinessLogic",
            CoreVulnerabilityCategory::ApiSecurity => "ApiSecurity",
            CoreVulnerabilityCategory::NetworkSecurity => "NetworkSecurity",
        }
    }
}

#[pymethods]
impl PyVulnerabilitySeverity {
    #[classmethod]
    pub fn critical(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilitySeverity::Critical,
        }
    }

    #[classmethod]
    pub fn high(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilitySeverity::High,
        }
    }

    #[classmethod]
    pub fn medium(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilitySeverity::Medium,
        }
    }

    #[classmethod]
    pub fn low(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilitySeverity::Low,
        }
    }

    #[classmethod]
    pub fn info(_cls: &PyType) -> Self {
        Self {
            inner: CoreVulnerabilitySeverity::Info,
        }
    }

    pub fn __str__(&self) -> &'static str {
        match self.inner {
            CoreVulnerabilitySeverity::Critical => "Critical",
            CoreVulnerabilitySeverity::High => "High",
            CoreVulnerabilitySeverity::Medium => "Medium",
            CoreVulnerabilitySeverity::Low => "Low",
            CoreVulnerabilitySeverity::Info => "Info",
        }
    }
}

impl Default for PyPenetrationTestFramework {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyPenetrationTestFramework {
    /// Create new penetration test framework
    #[new]
    pub fn new() -> Self {
        Self {
            inner: CorePenetrationTestFramework::new(),
        }
    }

    /// Start new penetration test
    pub fn start_test(&mut self, test_name: String, target_scope: Vec<String>) -> PyResult<String> {
        self.inner
            .start_test(test_name, target_scope)
            .map_err(map_rust_error_to_python)
    }

    /// Execute penetration test
    pub fn execute_test(&mut self, test_id: &str, events: Vec<PyEvent>) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create tokio runtime: {e}")))?;
        
        let core_events = events.into_iter().map(|e| e.inner).collect();
        
        rt.block_on(async {
            self.inner
                .execute_test(test_id, core_events)
                .await
                .map_err(map_rust_error_to_python)
        })
    }

    /// Get test results
    pub fn get_test_results(&self, test_id: &str) -> PyResult<PyPenetrationTest> {
        self.inner
            .get_test_results(test_id)
            .map(|test| PyPenetrationTest {
                inner: test.clone(),
            })
            .map_err(map_rust_error_to_python)
    }

    /// List all tests
    pub fn list_tests(&self) -> Vec<PyPenetrationTest> {
        self.inner
            .list_tests()
            .into_iter()
            .map(|test| PyPenetrationTest {
                inner: test.clone(),
            })
            .collect()
    }
}

#[pymethods]
impl PyPenetrationTest {
    #[getter]
    pub fn test_id(&self) -> String {
        self.inner.test_id.clone()
    }

    #[getter]
    pub fn test_name(&self) -> String {
        self.inner.test_name.clone()
    }

    #[getter]
    pub fn started_at(&self) -> String {
        self.inner.started_at.to_rfc3339()
    }

    #[getter]
    pub fn completed_at(&self) -> Option<String> {
        self.inner.completed_at.map(|dt| dt.to_rfc3339())
    }

    #[getter]
    pub fn findings_count(&self) -> usize {
        self.inner.findings.len()
    }
}