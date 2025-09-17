//! Event store implementations and abstractions.
//!
//! This module provides the core event storage functionality for Eventuali:
//! - [`EventStore`] trait defining the storage interface
//! - [`EventStoreImpl`] concrete implementation with backend abstraction
//! - Multiple backends: PostgreSQL, SQLite
//! - Configuration and factory functions
//!
//! # Example
//!
//! ```rust
//! use eventuali_core::{EventStoreConfig, create_event_store};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create SQLite event store
//! let config = EventStoreConfig::sqlite(":memory:").await?;
//! let store = create_event_store(config).await?;
//!
//! // Create PostgreSQL event store
//! let config = EventStoreConfig::postgres(
//!     "postgresql://user:pass@localhost/events"
//! ).await?;
//! let store = create_event_store(config).await?;
//! # Ok(())
//! # }
//! ```

pub mod traits;
pub mod postgres;
pub mod sqlite;
pub mod config;

pub use traits::{EventStore, EventStoreBackend};
pub use config::EventStoreConfig;

use crate::{Event, AggregateId, AggregateVersion, Result};
use crate::streaming::EventStreamer;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Core event store implementation with pluggable backends.
///
/// EventStoreImpl provides a unified interface over different storage backends
/// while maintaining consistency guarantees and optional event streaming.
///
/// # Type Parameters
///
/// * `B` - The backend implementation (PostgreSQL, SQLite, etc.)
///
/// # Features
///
/// - Atomic event batching
/// - Optional real-time event streaming
/// - Global event ordering
/// - Backend abstraction
pub struct EventStoreImpl<B: EventStoreBackend> {
    backend: B,
    streamer: Option<Arc<dyn EventStreamer + Send + Sync>>,
    global_position: Arc<Mutex<u64>>,
}

impl<B: EventStoreBackend> EventStoreImpl<B> {
    /// Creates a new event store with the specified backend.
    ///
    /// # Arguments
    ///
    /// * `backend` - The storage backend implementation
    ///
    /// # Example
    ///
    /// ```rust
    /// use eventuali_core::store::{EventStoreImpl, sqlite::SQLiteBackend};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let backend = SQLiteBackend::new(":memory:").await?;
    /// let store = EventStoreImpl::new(backend);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: B) -> Self {
        Self { 
            backend,
            streamer: None,
            global_position: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl<B: EventStoreBackend + Send + Sync> EventStore for EventStoreImpl<B> {
    async fn save_events(&self, events: Vec<Event>) -> Result<()> {
        // Save events to backend first
        self.backend.save_events(events.clone()).await?;
        
        // If we have a streamer configured, publish the events
        if let Some(streamer) = &self.streamer {
            let mut global_pos = self.global_position.lock().await;
            
            for event in events {
                *global_pos += 1;
                let stream_position = event.aggregate_version as u64;
                
                streamer.publish_event(event, stream_position, *global_pos).await?;
            }
        }
        
        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>> {
        self.backend.load_events(aggregate_id, from_version).await
    }

    async fn load_events_by_type(
        &self,
        aggregate_type: &str,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>> {
        self.backend.load_events_by_type(aggregate_type, from_version).await
    }

    async fn get_aggregate_version(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateVersion>> {
        self.backend.get_aggregate_version(aggregate_id).await
    }
    
    fn set_event_streamer(&mut self, streamer: Arc<dyn EventStreamer + Send + Sync>) {
        self.streamer = Some(streamer);
    }
}

// Factory function for creating event stores
pub async fn create_event_store(config: EventStoreConfig) -> Result<Box<dyn EventStore + Send + Sync>> {
    match &config {
        #[cfg(feature = "postgres")]
        EventStoreConfig::PostgreSQL { .. } => {
            let mut backend = postgres::PostgreSQLBackend::new(&config).await?;
            backend.initialize().await?;
            Ok(Box::new(EventStoreImpl::new(backend)))
        }
        #[cfg(feature = "sqlite")]
        EventStoreConfig::SQLite { .. } => {
            let mut backend = sqlite::SQLiteBackend::new(&config).await?;
            backend.initialize().await?;
            Ok(Box::new(EventStoreImpl::new(backend)))
        }
        #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
        _ => Err(EventualiError::Configuration(
            "No database backend features enabled".to_string(),
        )),
    }
}