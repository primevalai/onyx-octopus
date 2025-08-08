mod sqlite_store;

pub use sqlite_store::SqliteSnapshotStore;

use crate::{AggregateId, AggregateVersion, Result, EventualiError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a snapshot of an aggregate at a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateSnapshot {
    /// Unique identifier for the snapshot
    pub snapshot_id: Uuid,
    /// ID of the aggregate this snapshot represents
    pub aggregate_id: AggregateId,
    /// Type of the aggregate
    pub aggregate_type: String,
    /// Version of the aggregate when this snapshot was taken
    pub aggregate_version: AggregateVersion,
    /// Serialized aggregate state data
    pub state_data: Vec<u8>,
    /// Compression algorithm used (if any)
    pub compression: SnapshotCompression,
    /// Metadata about the snapshot
    pub metadata: SnapshotMetadata,
    /// When this snapshot was created
    pub created_at: DateTime<Utc>,
}

/// Compression algorithms supported for snapshots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotCompression {
    None,
    Gzip,
    Lz4,
}

/// Metadata for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Size of the original data before compression
    pub original_size: usize,
    /// Size of the compressed data
    pub compressed_size: usize,
    /// Number of events that were used to build this snapshot
    pub event_count: usize,
    /// Checksum of the snapshot data for integrity verification
    pub checksum: String,
    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

/// Configuration for snapshot behavior
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// How often to take snapshots (every N events)
    pub snapshot_frequency: AggregateVersion,
    /// Maximum age of snapshots before they should be replaced
    pub max_snapshot_age_hours: u64,
    /// Compression algorithm to use
    pub compression: SnapshotCompression,
    /// Whether to automatically clean up old snapshots
    pub auto_cleanup: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            snapshot_frequency: 100, // Snapshot every 100 events
            max_snapshot_age_hours: 24 * 7, // Keep snapshots for a week
            compression: SnapshotCompression::Gzip,
            auto_cleanup: true,
        }
    }
}

/// Trait for snapshot storage backends
#[async_trait]
pub trait SnapshotStore {
    /// Store a new snapshot
    async fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<()>;
    
    /// Load the latest snapshot for an aggregate
    async fn load_latest_snapshot(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateSnapshot>>;
    
    /// Load a specific snapshot by ID
    async fn load_snapshot(&self, snapshot_id: Uuid) -> Result<Option<AggregateSnapshot>>;
    
    /// Get all snapshots for an aggregate, ordered by version descending
    async fn list_snapshots(&self, aggregate_id: &AggregateId) -> Result<Vec<AggregateSnapshot>>;
    
    /// Delete a snapshot
    async fn delete_snapshot(&self, snapshot_id: Uuid) -> Result<()>;
    
    /// Clean up old snapshots based on configuration
    async fn cleanup_old_snapshots(&self, config: &SnapshotConfig) -> Result<u64>;
    
    /// Check if a snapshot should be taken for an aggregate at the given version
    async fn should_take_snapshot(
        &self, 
        aggregate_id: &AggregateId, 
        current_version: AggregateVersion,
        config: &SnapshotConfig
    ) -> Result<bool>;
}

/// Service for managing aggregate snapshots
pub struct SnapshotService<S: SnapshotStore> {
    store: S,
    config: SnapshotConfig,
}

impl<S: SnapshotStore> SnapshotService<S> {
    pub fn new(store: S, config: SnapshotConfig) -> Self {
        Self { store, config }
    }

    /// Create a snapshot from aggregate state data
    pub async fn create_snapshot(
        &self,
        aggregate_id: AggregateId,
        aggregate_type: String,
        aggregate_version: AggregateVersion,
        state_data: Vec<u8>,
        event_count: usize,
    ) -> Result<AggregateSnapshot> {
        let compressed_data = self.compress_data(&state_data)?;
        let checksum = self.calculate_checksum(&compressed_data);

        let metadata = SnapshotMetadata {
            original_size: state_data.len(),
            compressed_size: compressed_data.len(),
            event_count,
            checksum,
            custom: HashMap::new(),
        };

        let snapshot = AggregateSnapshot {
            snapshot_id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type,
            aggregate_version,
            state_data: compressed_data,
            compression: self.config.compression.clone(),
            metadata,
            created_at: Utc::now(),
        };

        self.store.save_snapshot(snapshot.clone()).await?;
        Ok(snapshot)
    }

    /// Load the most recent snapshot for an aggregate
    pub async fn load_latest_snapshot(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateSnapshot>> {
        self.store.load_latest_snapshot(aggregate_id).await
    }

    /// Decompress snapshot data
    pub fn decompress_snapshot_data(&self, snapshot: &AggregateSnapshot) -> Result<Vec<u8>> {
        self.decompress_data(&snapshot.state_data, &snapshot.compression)
    }

    /// Check if a snapshot should be taken
    pub async fn should_take_snapshot(
        &self,
        aggregate_id: &AggregateId,
        current_version: AggregateVersion,
    ) -> Result<bool> {
        self.store.should_take_snapshot(aggregate_id, current_version, &self.config).await
    }

    /// Compress data using the configured compression algorithm
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.compression {
            SnapshotCompression::None => Ok(data.to_vec()),
            SnapshotCompression::Gzip => {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                use std::io::Write;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data).map_err(EventualiError::Io)?;
                encoder.finish().map_err(EventualiError::Io)
            }
            SnapshotCompression::Lz4 => {
                // For now, fallback to no compression if lz4 is not available
                // In a real implementation, you'd use lz4_flex or similar crate
                Ok(data.to_vec())
            }
        }
    }

    /// Decompress data using the specified compression algorithm
    fn decompress_data(&self, data: &[u8], compression: &SnapshotCompression) -> Result<Vec<u8>> {
        match compression {
            SnapshotCompression::None => Ok(data.to_vec()),
            SnapshotCompression::Gzip => {
                use flate2::read::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed).map_err(EventualiError::Io)?;
                Ok(decompressed)
            }
            SnapshotCompression::Lz4 => {
                // For now, fallback to no compression
                Ok(data.to_vec())
            }
        }
    }

    /// Calculate checksum for data integrity
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Perform cleanup of old snapshots
    pub async fn cleanup_old_snapshots(&self) -> Result<u64> {
        self.store.cleanup_old_snapshots(&self.config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_compression_none() {
        let config = SnapshotConfig {
            compression: SnapshotCompression::None,
            ..Default::default()
        };
        
        // Create a mock store for testing
        struct MockStore;
        #[async_trait]
        impl SnapshotStore for MockStore {
            async fn save_snapshot(&self, _: AggregateSnapshot) -> Result<()> { Ok(()) }
            async fn load_latest_snapshot(&self, _: &AggregateId) -> Result<Option<AggregateSnapshot>> { Ok(None) }
            async fn load_snapshot(&self, _: Uuid) -> Result<Option<AggregateSnapshot>> { Ok(None) }
            async fn list_snapshots(&self, _: &AggregateId) -> Result<Vec<AggregateSnapshot>> { Ok(vec![]) }
            async fn delete_snapshot(&self, _: Uuid) -> Result<()> { Ok(()) }
            async fn cleanup_old_snapshots(&self, _: &SnapshotConfig) -> Result<u64> { Ok(0) }
            async fn should_take_snapshot(&self, _: &AggregateId, _: AggregateVersion, _: &SnapshotConfig) -> Result<bool> { Ok(false) }
        }
        
        let service = SnapshotService::new(MockStore, config);
        let data = b"test data".to_vec();
        let compressed = service.compress_data(&data).unwrap();
        
        assert_eq!(compressed, data);
    }

    #[test]
    fn test_snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert_eq!(config.snapshot_frequency, 100);
        assert_eq!(config.compression, SnapshotCompression::Gzip);
        assert!(config.auto_cleanup);
    }
}