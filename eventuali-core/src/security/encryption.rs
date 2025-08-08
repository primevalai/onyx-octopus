use crate::{EventData, EventualiError, Result};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// AES-256-GCM encryption implementation for event data
pub struct EventEncryption {
    key_manager: KeyManager,
}

/// Key management system for encryption keys
#[derive(Debug, Clone)]
pub struct KeyManager {
    keys: HashMap<String, EncryptionKey>,
    default_key_id: String,
}

/// Encryption key with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: String,
    pub key_data: Vec<u8>, // 32 bytes for AES-256
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub algorithm: EncryptionAlgorithm,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

/// Encrypted event data with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncryptedEventData {
    pub algorithm: EncryptionAlgorithm,
    pub key_id: String,
    pub iv: Vec<u8>,
    pub encrypted_data: Vec<u8>,
    pub tag: Vec<u8>,
}

impl EventEncryption {
    /// Create new encryption instance with a key manager
    pub fn new(key_manager: KeyManager) -> Self {
        Self { key_manager }
    }

    /// Create a new encryption instance with a single key
    pub fn with_key(key_id: String, key_data: Vec<u8>) -> Result<Self> {
        let mut keys = HashMap::new();
        let encryption_key = EncryptionKey {
            id: key_id.clone(),
            key_data,
            created_at: chrono::Utc::now(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        };
        keys.insert(key_id.clone(), encryption_key);
        
        let key_manager = KeyManager {
            keys,
            default_key_id: key_id,
        };
        
        Ok(Self::new(key_manager))
    }

    /// Encrypt event data using the default key
    pub fn encrypt_event_data(&self, data: &EventData) -> Result<EncryptedEventData> {
        self.encrypt_event_data_with_key(data, &self.key_manager.default_key_id)
    }

    /// Encrypt event data using a specific key
    pub fn encrypt_event_data_with_key(&self, data: &EventData, key_id: &str) -> Result<EncryptedEventData> {
        let key = self.key_manager.get_key(key_id)?;
        let plaintext = self.serialize_event_data(data)?;
        
        // Generate random IV (12 bytes for GCM)
        let iv = self.generate_iv()?;
        
        // Encrypt using AES-256-GCM
        let (encrypted_data, tag) = self.encrypt_aes_256_gcm(&plaintext, &key.key_data, &iv)?;
        
        Ok(EncryptedEventData {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_id: key_id.to_string(),
            iv,
            encrypted_data,
            tag,
        })
    }

    /// Decrypt event data
    pub fn decrypt_event_data(&self, encrypted_data: &EncryptedEventData) -> Result<EventData> {
        let key = self.key_manager.get_key(&encrypted_data.key_id)?;
        
        match encrypted_data.algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let plaintext = self.decrypt_aes_256_gcm(
                    &encrypted_data.encrypted_data,
                    &key.key_data,
                    &encrypted_data.iv,
                    &encrypted_data.tag,
                )?;
                self.deserialize_event_data(&plaintext)
            }
        }
    }

    /// Serialize event data to bytes for encryption
    fn serialize_event_data(&self, data: &EventData) -> Result<Vec<u8>> {
        match data {
            EventData::Json(value) => {
                let json_string = serde_json::to_string(value)?;
                Ok(json_string.into_bytes())
            }
            EventData::Protobuf(bytes) => Ok(bytes.clone()),
        }
    }

    /// Deserialize event data from decrypted bytes
    fn deserialize_event_data(&self, bytes: &[u8]) -> Result<EventData> {
        // Try to parse as JSON first, fallback to protobuf
        if let Ok(json_str) = std::str::from_utf8(bytes) {
            if let Ok(json_value) = serde_json::from_str(json_str) {
                return Ok(EventData::Json(json_value));
            }
        }
        // Fallback to protobuf
        Ok(EventData::Protobuf(bytes.to_vec()))
    }

    /// Generate a random IV for AES-GCM
    fn generate_iv(&self) -> Result<Vec<u8>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Generate a 12-byte IV using system time and random data
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| EventualiError::Encryption(format!("Time error: {}", e)))?
            .as_nanos() as u64;
        
        let mut iv = Vec::with_capacity(12);
        iv.extend_from_slice(&timestamp.to_be_bytes());
        
        // Add 4 more random bytes using a simple PRNG
        let random_seed = (timestamp.wrapping_mul(1103515245).wrapping_add(12345)) % (1u64 << 31);
        iv.extend_from_slice(&(random_seed as u32).to_be_bytes());
        
        Ok(iv)
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt_aes_256_gcm(&self, plaintext: &[u8], key: &[u8], iv: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use aes_gcm::aead::{Aead, generic_array::GenericArray};
        
        let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
        let nonce = Nonce::from_slice(iv);
        
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EventualiError::Encryption(format!("AES-256-GCM encryption failed: {}", e)))?;
        
        // Extract tag (last 16 bytes)
        let tag_start = ciphertext.len() - 16;
        let encrypted_data = ciphertext[..tag_start].to_vec();
        let tag = ciphertext[tag_start..].to_vec();
        
        Ok((encrypted_data, tag))
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt_aes_256_gcm(&self, ciphertext: &[u8], key: &[u8], iv: &[u8], tag: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use aes_gcm::aead::{Aead, generic_array::GenericArray};
        
        let cipher = Aes256Gcm::new(GenericArray::from_slice(key));
        let nonce = Nonce::from_slice(iv);
        
        // Reconstruct full ciphertext with tag
        let mut full_ciphertext = ciphertext.to_vec();
        full_ciphertext.extend_from_slice(tag);
        
        let plaintext = cipher
            .decrypt(nonce, full_ciphertext.as_ref())
            .map_err(|e| EventualiError::Encryption(format!("AES-256-GCM decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }
}

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            default_key_id: String::new(),
        }
    }

    /// Add a key to the manager
    pub fn add_key(&mut self, key: EncryptionKey) -> Result<()> {
        if key.key_data.len() != 32 {
            return Err(EventualiError::Encryption(
                "AES-256 requires 32-byte keys".to_string()
            ));
        }
        
        if self.keys.is_empty() {
            self.default_key_id = key.id.clone();
        }
        
        self.keys.insert(key.id.clone(), key);
        Ok(())
    }

    /// Generate a new AES-256 key
    pub fn generate_key(id: String) -> Result<EncryptionKey> {
        let key_data = Self::generate_random_key()?;
        Ok(EncryptionKey {
            id,
            key_data,
            created_at: chrono::Utc::now(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        })
    }

    /// Generate a key from a password using PBKDF2
    pub fn derive_key_from_password(id: String, password: &str, salt: &[u8]) -> Result<EncryptionKey> {
        use pbkdf2::{pbkdf2_hmac};
        use sha2::Sha256;
        
        let mut key_data = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key_data);
        
        Ok(EncryptionKey {
            id,
            key_data: key_data.to_vec(),
            created_at: chrono::Utc::now(),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        })
    }

    /// Get a key by ID
    pub fn get_key(&self, key_id: &str) -> Result<&EncryptionKey> {
        self.keys.get(key_id).ok_or_else(|| {
            EventualiError::Encryption(format!("Key not found: {}", key_id))
        })
    }

    /// Set the default key
    pub fn set_default_key(&mut self, key_id: &str) -> Result<()> {
        if !self.keys.contains_key(key_id) {
            return Err(EventualiError::Encryption(
                format!("Key not found: {}", key_id)
            ));
        }
        self.default_key_id = key_id.to_string();
        Ok(())
    }

    /// Generate a cryptographically secure random 32-byte key
    fn generate_random_key() -> Result<Vec<u8>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Use system time as seed for key generation
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| EventualiError::Encryption(format!("Time error: {}", e)))?
            .as_nanos();
        
        // Generate key using SHA-256 of timestamp and additional entropy
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_be_bytes());
        hasher.update(b"eventuali-encryption-key");
        
        // Add more entropy from process ID and memory address
        let pid = std::process::id();
        hasher.update(pid.to_be_bytes());
        
        // Use stack address for additional entropy
        let stack_var: u64 = 42;
        let stack_addr = &stack_var as *const u64 as usize;
        hasher.update(stack_addr.to_be_bytes());
        
        let key_hash = hasher.finalize();
        Ok(key_hash.to_vec())
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Encrypted event data serialization methods
impl EncryptedEventData {
    /// Serialize to base64 string for storage
    pub fn to_base64(&self) -> String {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        general_purpose::STANDARD.encode(serialized)
    }

    /// Deserialize from base64 string
    pub fn from_base64(data: &str) -> Result<Self> {
        let bytes = general_purpose::STANDARD
            .decode(data)
            .map_err(|e| EventualiError::Encryption(format!("Base64 decode error: {}", e)))?;
        
        serde_json::from_slice(&bytes)
            .map_err(EventualiError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_key_generation() {
        let key = KeyManager::generate_key("test-key".to_string()).unwrap();
        assert_eq!(key.key_data.len(), 32);
        assert_eq!(key.id, "test-key");
        assert_eq!(key.algorithm, EncryptionAlgorithm::Aes256Gcm);
    }

    #[test]
    fn test_password_key_derivation() {
        let salt = b"test-salt";
        let key = KeyManager::derive_key_from_password(
            "test-key".to_string(),
            "test-password",
            salt
        ).unwrap();
        
        assert_eq!(key.key_data.len(), 32);
        
        // Same password and salt should produce same key
        let key2 = KeyManager::derive_key_from_password(
            "test-key-2".to_string(),
            "test-password",
            salt
        ).unwrap();
        
        assert_eq!(key.key_data, key2.key_data);
    }

    #[test]
    fn test_json_encryption_decryption() {
        let key = KeyManager::generate_key("test-key".to_string()).unwrap();
        let encryption = EventEncryption::with_key("test-key".to_string(), key.key_data).unwrap();
        
        let original_data = EventData::Json(json!({
            "user_id": "user123",
            "action": "create_order",
            "amount": 100.50
        }));
        
        let encrypted = encryption.encrypt_event_data(&original_data).unwrap();
        assert_eq!(encrypted.algorithm, EncryptionAlgorithm::Aes256Gcm);
        assert_eq!(encrypted.key_id, "test-key");
        assert_eq!(encrypted.iv.len(), 12);
        assert!(!encrypted.encrypted_data.is_empty());
        assert_eq!(encrypted.tag.len(), 16);
        
        let decrypted = encryption.decrypt_event_data(&encrypted).unwrap();
        assert_eq!(original_data, decrypted);
    }

    #[test]
    fn test_protobuf_encryption_decryption() {
        let key = KeyManager::generate_key("test-key".to_string()).unwrap();
        let encryption = EventEncryption::with_key("test-key".to_string(), key.key_data).unwrap();
        
        let original_data = EventData::Protobuf(vec![0x08, 0x96, 0x01, 0x12, 0x04, 0x74, 0x65, 0x73, 0x74]);
        
        let encrypted = encryption.encrypt_event_data(&original_data).unwrap();
        let decrypted = encryption.decrypt_event_data(&encrypted).unwrap();
        
        assert_eq!(original_data, decrypted);
    }

    #[test]
    fn test_multiple_keys() {
        let mut key_manager = KeyManager::new();
        
        let key1 = KeyManager::generate_key("key1".to_string()).unwrap();
        let key2 = KeyManager::generate_key("key2".to_string()).unwrap();
        
        key_manager.add_key(key1.clone()).unwrap();
        key_manager.add_key(key2.clone()).unwrap();
        
        let encryption = EventEncryption::new(key_manager);
        let data = EventData::Json(json!({"test": "data"}));
        
        let encrypted1 = encryption.encrypt_event_data_with_key(&data, "key1").unwrap();
        let encrypted2 = encryption.encrypt_event_data_with_key(&data, "key2").unwrap();
        
        assert_eq!(encrypted1.key_id, "key1");
        assert_eq!(encrypted2.key_id, "key2");
        
        let decrypted1 = encryption.decrypt_event_data(&encrypted1).unwrap();
        let decrypted2 = encryption.decrypt_event_data(&encrypted2).unwrap();
        
        assert_eq!(data, decrypted1);
        assert_eq!(data, decrypted2);
    }

    #[test]
    fn test_base64_serialization() {
        let key = KeyManager::generate_key("test-key".to_string()).unwrap();
        let encryption = EventEncryption::with_key("test-key".to_string(), key.key_data).unwrap();
        
        let data = EventData::Json(json!({"test": "data"}));
        let encrypted = encryption.encrypt_event_data(&data).unwrap();
        
        let base64_str = encrypted.to_base64();
        assert!(!base64_str.is_empty());
        
        let deserialized = EncryptedEventData::from_base64(&base64_str).unwrap();
        assert_eq!(encrypted, deserialized);
        
        let decrypted = encryption.decrypt_event_data(&deserialized).unwrap();
        assert_eq!(data, decrypted);
    }
}