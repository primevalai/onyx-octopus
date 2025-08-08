pub mod event;
pub mod aggregate;
pub mod store;
pub mod error;
pub mod proto;
pub mod streaming;
pub mod snapshot;
pub mod security;
pub mod tenancy;

pub use event::{Event, EventData, EventId, EventMetadata};
pub use aggregate::{Aggregate, AggregateId, AggregateVersion};
pub use store::{EventStore, EventStoreConfig, EventStoreImpl, create_event_store};
pub use error::{EventualiError, Result};
pub use proto::ProtoSerializer;
pub use streaming::{
    EventStreamer, EventStreamReceiver, StreamEvent, Subscription, SubscriptionBuilder,
    InMemoryEventStreamer, EventStreamProcessor, Projection, ProjectionProcessor,
    SagaHandler, SagaProcessor
};
pub use snapshot::{
    AggregateSnapshot, SnapshotStore, SnapshotService, SnapshotConfig, SnapshotCompression,
    SnapshotMetadata, SqliteSnapshotStore
};
pub use security::{
    EventEncryption, KeyManager, EncryptionKey, EncryptedEventData, EncryptionAlgorithm
};
pub use tenancy::{
    TenantId, TenantInfo, TenantConfig, TenantMetadata, TenantIsolation, 
    IsolatedEventStore, TenantScope, TenantQuota, ResourceTracker, 
    TenantManager, TenantOperations, TenantAwareEventStorage, 
    TenantStorageMetrics, TenantEventBatch
};

// Re-export specific backend implementations
#[cfg(feature = "postgres")]
pub use store::postgres::PostgreSQLBackend;

#[cfg(feature = "sqlite")]
pub use store::sqlite::SQLiteBackend;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}