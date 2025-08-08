//! Security module providing encryption, digital signatures, audit trails, and RBAC

pub mod encryption;

pub use encryption::{
    EventEncryption, KeyManager, EncryptionKey, EncryptedEventData, EncryptionAlgorithm
};