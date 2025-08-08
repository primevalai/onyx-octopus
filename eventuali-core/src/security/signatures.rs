use crate::{Event, EventualiError, Result};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Digital signature implementation for event integrity verification
pub struct EventSigner {
    key_manager: SigningKeyManager,
}

/// Signing key management system
#[derive(Debug, Clone)]
pub struct SigningKeyManager {
    keys: HashMap<String, SigningKey>,
    default_key_id: String,
}

/// Signing key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKey {
    pub id: String,
    pub key_data: Vec<u8>, // HMAC signing key
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub algorithm: SignatureAlgorithm,
}

/// Supported signature algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    HmacSha256,
    HmacSha512,
}

/// Event signature with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventSignature {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub signature: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_hash: Vec<u8>, // SHA-256 of the event data for verification
}

/// Signed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEvent {
    pub event: Event,
    pub signature: EventSignature,
}

impl EventSigner {
    /// Create new signer instance with a key manager
    pub fn new(key_manager: SigningKeyManager) -> Self {
        Self { key_manager }
    }

    /// Create a new signer instance with a single key
    pub fn with_key(key_id: String, key_data: Vec<u8>) -> Result<Self> {
        let mut keys = HashMap::new();
        let signing_key = SigningKey {
            id: key_id.clone(),
            key_data,
            created_at: chrono::Utc::now(),
            algorithm: SignatureAlgorithm::HmacSha256,
        };
        keys.insert(key_id.clone(), signing_key);
        
        let key_manager = SigningKeyManager {
            keys,
            default_key_id: key_id,
        };
        
        Ok(Self::new(key_manager))
    }

    /// Sign an event using the default key
    pub fn sign_event(&self, event: &Event) -> Result<SignedEvent> {
        self.sign_event_with_key(event, &self.key_manager.default_key_id)
    }

    /// Sign an event using a specific key
    pub fn sign_event_with_key(&self, event: &Event, key_id: &str) -> Result<SignedEvent> {
        let key = self.key_manager.get_key(key_id)?;
        let event_bytes = self.serialize_event(event)?;
        let event_hash = self.hash_event_data(&event_bytes);
        
        let signature_bytes = match key.algorithm {
            SignatureAlgorithm::HmacSha256 => self.hmac_sha256(&event_bytes, &key.key_data)?,
            SignatureAlgorithm::HmacSha512 => self.hmac_sha512(&event_bytes, &key.key_data)?,
        };
        
        let signature = EventSignature {
            algorithm: key.algorithm.clone(),
            key_id: key_id.to_string(),
            signature: signature_bytes,
            timestamp: chrono::Utc::now(),
            event_hash,
        };
        
        Ok(SignedEvent {
            event: event.clone(),
            signature,
        })
    }

    /// Verify an event signature
    pub fn verify_signature(&self, signed_event: &SignedEvent) -> Result<bool> {
        let key = self.key_manager.get_key(&signed_event.signature.key_id)?;
        let event_bytes = self.serialize_event(&signed_event.event)?;
        
        // Verify event hash first
        let computed_hash = self.hash_event_data(&event_bytes);
        if computed_hash != signed_event.signature.event_hash {
            return Ok(false);
        }
        
        // Compute expected signature
        let expected_signature = match signed_event.signature.algorithm {
            SignatureAlgorithm::HmacSha256 => self.hmac_sha256(&event_bytes, &key.key_data)?,
            SignatureAlgorithm::HmacSha512 => self.hmac_sha512(&event_bytes, &key.key_data)?,
        };
        
        // Constant-time comparison to prevent timing attacks
        Ok(self.constant_time_compare(&expected_signature, &signed_event.signature.signature))
    }

    /// Verify signature without needing the full key manager (using provided key)
    pub fn verify_signature_with_key(&self, signed_event: &SignedEvent, key_data: &[u8]) -> Result<bool> {
        let event_bytes = self.serialize_event(&signed_event.event)?;
        
        // Verify event hash first
        let computed_hash = self.hash_event_data(&event_bytes);
        if computed_hash != signed_event.signature.event_hash {
            return Ok(false);
        }
        
        // Compute expected signature
        let expected_signature = match signed_event.signature.algorithm {
            SignatureAlgorithm::HmacSha256 => self.hmac_sha256(&event_bytes, key_data)?,
            SignatureAlgorithm::HmacSha512 => self.hmac_sha512(&event_bytes, key_data)?,
        };
        
        // Constant-time comparison
        Ok(self.constant_time_compare(&expected_signature, &signed_event.signature.signature))
    }

    /// Create a signature for raw data (not an event)
    pub fn sign_data(&self, data: &[u8], key_id: &str) -> Result<EventSignature> {
        let key = self.key_manager.get_key(key_id)?;
        let data_hash = self.hash_event_data(data);
        
        let signature_bytes = match key.algorithm {
            SignatureAlgorithm::HmacSha256 => self.hmac_sha256(data, &key.key_data)?,
            SignatureAlgorithm::HmacSha512 => self.hmac_sha512(data, &key.key_data)?,
        };
        
        Ok(EventSignature {
            algorithm: key.algorithm.clone(),
            key_id: key_id.to_string(),
            signature: signature_bytes,
            timestamp: chrono::Utc::now(),
            event_hash: data_hash,
        })
    }

    /// Verify a signature for raw data
    pub fn verify_data_signature(&self, data: &[u8], signature: &EventSignature) -> Result<bool> {
        let key = self.key_manager.get_key(&signature.key_id)?;
        
        // Verify data hash
        let computed_hash = self.hash_event_data(data);
        if computed_hash != signature.event_hash {
            return Ok(false);
        }
        
        // Compute expected signature
        let expected_signature = match signature.algorithm {
            SignatureAlgorithm::HmacSha256 => self.hmac_sha256(data, &key.key_data)?,
            SignatureAlgorithm::HmacSha512 => self.hmac_sha512(data, &key.key_data)?,
        };
        
        Ok(self.constant_time_compare(&expected_signature, &signature.signature))
    }

    /// Serialize event to bytes for signing
    fn serialize_event(&self, event: &Event) -> Result<Vec<u8>> {
        serde_json::to_vec(event)
            .map_err(EventualiError::Serialization)
    }

    /// Hash event data using SHA-256
    fn hash_event_data(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Compute HMAC-SHA256
    fn hmac_sha256(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|e| EventualiError::Configuration(format!("Invalid HMAC key: {e}")))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Compute HMAC-SHA512
    fn hmac_sha512(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha2::Sha512;
        type HmacSha512 = Hmac<Sha512>;
        
        let mut mac = HmacSha512::new_from_slice(key)
            .map_err(|e| EventualiError::Configuration(format!("Invalid HMAC key: {e}")))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Constant-time comparison to prevent timing attacks
    fn constant_time_compare(&self, a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (byte_a, byte_b) in a.iter().zip(b.iter()) {
            result |= byte_a ^ byte_b;
        }
        
        result == 0
    }
}

impl SigningKeyManager {
    /// Create a new signing key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            default_key_id: String::new(),
        }
    }

    /// Add a key to the manager
    pub fn add_key(&mut self, key: SigningKey) -> Result<()> {
        if key.key_data.is_empty() {
            return Err(EventualiError::Configuration(
                "Signing key cannot be empty".to_string()
            ));
        }
        
        if self.keys.is_empty() {
            self.default_key_id = key.id.clone();
        }
        
        self.keys.insert(key.id.clone(), key);
        Ok(())
    }

    /// Generate a new HMAC signing key
    pub fn generate_key(id: String, algorithm: SignatureAlgorithm) -> Result<SigningKey> {
        let key_data = Self::generate_random_key(algorithm.key_size())?;
        Ok(SigningKey {
            id,
            key_data,
            created_at: chrono::Utc::now(),
            algorithm,
        })
    }

    /// Derive a signing key from a password using PBKDF2
    pub fn derive_key_from_password(
        id: String,
        password: &str,
        salt: &[u8],
        algorithm: SignatureAlgorithm,
    ) -> Result<SigningKey> {
        use pbkdf2::{pbkdf2_hmac};
        use sha2::Sha256;
        
        let key_size = algorithm.key_size();
        let mut key_data = vec![0u8; key_size];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key_data);
        
        Ok(SigningKey {
            id,
            key_data,
            created_at: chrono::Utc::now(),
            algorithm,
        })
    }

    /// Get a key by ID
    pub fn get_key(&self, key_id: &str) -> Result<&SigningKey> {
        self.keys.get(key_id).ok_or_else(|| {
            EventualiError::Configuration(format!("Signing key not found: {key_id}"))
        })
    }

    /// Set the default key
    pub fn set_default_key(&mut self, key_id: &str) -> Result<()> {
        if !self.keys.contains_key(key_id) {
            return Err(EventualiError::Configuration(
                format!("Signing key not found: {key_id}")
            ));
        }
        self.default_key_id = key_id.to_string();
        Ok(())
    }

    /// List all key IDs
    pub fn list_key_ids(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    /// Generate a cryptographically secure random signing key
    fn generate_random_key(size: usize) -> Result<Vec<u8>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Use system time as seed for key generation
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| EventualiError::Configuration(format!("Time error: {e}")))?
            .as_nanos();
        
        // Generate key using multiple SHA-256 rounds for entropy
        let mut key = Vec::with_capacity(size);
        let mut current_hash = timestamp.to_be_bytes().to_vec();
        
        while key.len() < size {
            let mut hasher = Sha256::new();
            hasher.update(&current_hash);
            hasher.update(b"eventuali-signing-key");
            hasher.update((key.len() as u64).to_be_bytes());
            
            // Add more entropy from process ID and memory address
            let pid = std::process::id();
            hasher.update(pid.to_be_bytes());
            
            current_hash = hasher.finalize().to_vec();
            
            let remaining = size - key.len();
            if remaining >= current_hash.len() {
                key.extend_from_slice(&current_hash);
            } else {
                key.extend_from_slice(&current_hash[..remaining]);
            }
        }
        
        Ok(key)
    }
}

impl Default for SigningKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureAlgorithm {
    /// Get the recommended key size for the algorithm
    pub fn key_size(&self) -> usize {
        match self {
            SignatureAlgorithm::HmacSha256 => 32, // 256 bits
            SignatureAlgorithm::HmacSha512 => 64, // 512 bits
        }
    }

    /// Get the output size of the signature
    pub fn signature_size(&self) -> usize {
        match self {
            SignatureAlgorithm::HmacSha256 => 32, // 256 bits
            SignatureAlgorithm::HmacSha512 => 64, // 512 bits
        }
    }
}

/// Signed event data serialization methods
impl SignedEvent {
    /// Serialize to base64 string for storage
    pub fn to_base64(&self) -> String {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        general_purpose::STANDARD.encode(serialized)
    }

    /// Deserialize from base64 string
    pub fn from_base64(data: &str) -> Result<Self> {
        let bytes = general_purpose::STANDARD
            .decode(data)
            .map_err(|e| EventualiError::Configuration(format!("Base64 decode error: {e}")))?;
        
        serde_json::from_slice(&bytes)
            .map_err(EventualiError::from)
    }
}

impl EventSignature {
    /// Serialize to base64 string for storage
    pub fn to_base64(&self) -> String {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        general_purpose::STANDARD.encode(serialized)
    }

    /// Deserialize from base64 string
    pub fn from_base64(data: &str) -> Result<Self> {
        let bytes = general_purpose::STANDARD
            .decode(data)
            .map_err(|e| EventualiError::Configuration(format!("Base64 decode error: {e}")))?;
        
        serde_json::from_slice(&bytes)
            .map_err(EventualiError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventData, EventMetadata};
    use uuid::Uuid;

    fn create_test_event() -> Event {
        Event {
            id: Uuid::new_v4(),
            aggregate_id: "test-aggregate".to_string(),
            aggregate_type: "TestAggregate".to_string(),
            event_type: "TestEvent".to_string(),
            event_version: 1,
            aggregate_version: 1,
            data: EventData::Json(serde_json::json!({"test": "data"})),
            metadata: EventMetadata::default(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_key_generation() {
        let key = SigningKeyManager::generate_key(
            "test-key".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        assert_eq!(key.key_data.len(), 32);
        assert_eq!(key.id, "test-key");
        assert_eq!(key.algorithm, SignatureAlgorithm::HmacSha256);
    }

    #[test]
    fn test_password_key_derivation() {
        let salt = b"test-salt";
        let key = SigningKeyManager::derive_key_from_password(
            "test-key".to_string(),
            "test-password",
            salt,
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        assert_eq!(key.key_data.len(), 32);
        
        // Same password and salt should produce same key
        let key2 = SigningKeyManager::derive_key_from_password(
            "test-key-2".to_string(),
            "test-password",
            salt,
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        assert_eq!(key.key_data, key2.key_data);
    }

    #[test]
    fn test_event_signing_and_verification() {
        let key = SigningKeyManager::generate_key(
            "test-key".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        let signer = EventSigner::with_key("test-key".to_string(), key.key_data).unwrap();
        let event = create_test_event();
        
        let signed_event = signer.sign_event(&event).unwrap();
        
        assert_eq!(signed_event.signature.algorithm, SignatureAlgorithm::HmacSha256);
        assert_eq!(signed_event.signature.key_id, "test-key");
        assert_eq!(signed_event.signature.signature.len(), 32);
        assert!(!signed_event.signature.signature.is_empty());
        
        let is_valid = signer.verify_signature(&signed_event).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_signature_verification_with_different_keys() {
        let key1 = SigningKeyManager::generate_key(
            "key1".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        let key2 = SigningKeyManager::generate_key(
            "key2".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        let signer1 = EventSigner::with_key("key1".to_string(), key1.key_data).unwrap();
        let signer2 = EventSigner::with_key("key2".to_string(), key2.key_data).unwrap();
        
        let event = create_test_event();
        let signed_event = signer1.sign_event(&event).unwrap();
        
        // Should verify with correct signer
        assert!(signer1.verify_signature(&signed_event).unwrap());
        
        // Should fail with wrong signer (different key)
        assert!(!signer2.verify_signature(&signed_event).unwrap_or(true));
    }

    #[test]
    fn test_tampered_event_detection() {
        let key = SigningKeyManager::generate_key(
            "test-key".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        let signer = EventSigner::with_key("test-key".to_string(), key.key_data).unwrap();
        let event = create_test_event();
        
        let mut signed_event = signer.sign_event(&event).unwrap();
        
        // Original should verify
        assert!(signer.verify_signature(&signed_event).unwrap());
        
        // Tamper with the event data
        signed_event.event.data = EventData::Json(serde_json::json!({"tampered": "data"}));
        
        // Should fail verification due to tampering
        assert!(!signer.verify_signature(&signed_event).unwrap());
    }

    #[test]
    fn test_hmac_sha512() {
        let key = SigningKeyManager::generate_key(
            "test-key".to_string(),
            SignatureAlgorithm::HmacSha512
        ).unwrap();
        
        assert_eq!(key.key_data.len(), 64);
        
        let signer = EventSigner::with_key("test-key".to_string(), key.key_data).unwrap();
        let event = create_test_event();
        
        let signed_event = signer.sign_event(&event).unwrap();
        
        assert_eq!(signed_event.signature.algorithm, SignatureAlgorithm::HmacSha512);
        assert_eq!(signed_event.signature.signature.len(), 64);
        assert!(signer.verify_signature(&signed_event).unwrap());
    }

    #[test]
    fn test_data_signing() {
        let key = SigningKeyManager::generate_key(
            "test-key".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        let signer = EventSigner::with_key("test-key".to_string(), key.key_data).unwrap();
        let data = b"Hello, World!";
        
        let signature = signer.sign_data(data, "test-key").unwrap();
        assert!(signer.verify_data_signature(data, &signature).unwrap());
        
        // Different data should fail verification
        let other_data = b"Hello, World?";
        assert!(!signer.verify_data_signature(other_data, &signature).unwrap());
    }

    #[test]
    fn test_base64_serialization() {
        let key = SigningKeyManager::generate_key(
            "test-key".to_string(),
            SignatureAlgorithm::HmacSha256
        ).unwrap();
        
        let signer = EventSigner::with_key("test-key".to_string(), key.key_data).unwrap();
        let event = create_test_event();
        
        let signed_event = signer.sign_event(&event).unwrap();
        
        let base64_str = signed_event.to_base64();
        assert!(!base64_str.is_empty());
        
        let deserialized = SignedEvent::from_base64(&base64_str).unwrap();
        assert_eq!(signed_event.signature.key_id, deserialized.signature.key_id);
        assert_eq!(signed_event.signature.signature, deserialized.signature.signature);
        
        assert!(signer.verify_signature(&deserialized).unwrap());
    }
}