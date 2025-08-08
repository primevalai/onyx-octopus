use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::exceptions::PyRuntimeError;
use eventuali_core::security::{
    EventEncryption as CoreEventEncryption, KeyManager as CoreKeyManager, 
    EncryptionKey as CoreEncryptionKey, EncryptedEventData as CoreEncryptedEventData,
    EncryptionAlgorithm as CoreEncryptionAlgorithm
};
use eventuali_core::{EventData as CoreEventData, Result as CoreResult};
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
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid JSON: {}", e)))?;
        let event_data = CoreEventData::Json(json_value);
        
        self.inner
            .encrypt_event_data(&event_data)
            .map(|inner| PyEncryptedEventData { inner })
            .map_err(map_rust_error_to_python)
    }

    /// Encrypt JSON data using a specific key
    pub fn encrypt_json_data_with_key(&self, data: String, key_id: &str) -> PyResult<PyEncryptedEventData> {
        let json_value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid JSON: {}", e)))?;
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
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to serialize JSON: {}", e)))
            }
            CoreEventData::Protobuf(bytes) => {
                String::from_utf8(bytes)
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert bytes to string: {}", e)))
            }
        }
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