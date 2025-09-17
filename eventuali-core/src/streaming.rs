//! Real-time event streaming and projections.
//!
//! This module provides real-time event streaming capabilities for reactive
//! event sourcing architectures:
//! - [`EventStreamer`] trait for streaming implementations
//! - [`InMemoryEventStreamer`] for development and testing
//! - [`Projection`] trait for building read models
//! - Subscription management and filtering
//!
//! # Example
//!
//! ```rust
//! use eventuali_core::{InMemoryEventStreamer, SubscriptionBuilder};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let streamer = InMemoryEventStreamer::new();
//!
//! // Subscribe to user events
//! let subscription = SubscriptionBuilder::new()
//!     .aggregate_type("User".to_string())
//!     .build();
//!
//! let mut receiver = streamer.subscribe(subscription).await?;
//!
//! // Process events as they arrive
//! while let Ok(stream_event) = receiver.recv().await {
//!     println!("Received event: {}", stream_event.event.event_type);
//! }
//! # Ok(())
//! # }
//! ```

use crate::{Event, Result, EventualiError};
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Event stream subscription configuration.
///
/// Subscriptions define which events a consumer wants to receive from the event stream.
/// They support filtering by aggregate type, event type, and time ranges.
///
/// # Example
///
/// ```rust
/// use eventuali_core::SubscriptionBuilder;
///
/// let subscription = SubscriptionBuilder::new()
///     .aggregate_type("User".to_string())
///     .event_type("UserRegistered".to_string())
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: String,
    pub aggregate_type_filter: Option<String>,
    pub event_type_filter: Option<String>,
    pub from_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Event stream message containing an event and its position information.
///
/// StreamEvent wraps an event with position metadata for stream processing:
/// - `stream_position`: Position within the aggregate stream
/// - `global_position`: Global position across all events
///
/// This enables ordered processing and checkpoint management.
#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub event: Event,
    pub stream_position: u64,
    pub global_position: u64,
}

/// Trait for event streaming implementations.
///
/// EventStreamer provides the core streaming interface for real-time event processing.
/// Implementations can be in-memory (for testing), Redis-based, or other message brokers.
///
/// # Example Implementation
///
/// ```rust
/// use eventuali_core::{EventStreamer, Subscription, EventStreamReceiver, Event, Result};
/// use async_trait::async_trait;
///
/// struct MyStreamer;
///
/// #[async_trait]
/// impl EventStreamer for MyStreamer {
///     async fn subscribe(&self, subscription: Subscription) -> Result<EventStreamReceiver> {
///         // Implementation here
///         todo!()
///     }
///
///     async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
///         // Implementation here
///         todo!()
///     }
///
///     async fn publish_event(&self, event: Event, stream_position: u64, global_position: u64) -> Result<()> {
///         // Implementation here
///         todo!()
///     }
///
///     async fn get_stream_position(&self, stream_id: &str) -> Result<Option<u64>> {
///         // Implementation here
///         todo!()
///     }
///
///     async fn get_global_position(&self) -> Result<u64> {
///         // Implementation here
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait EventStreamer {
    async fn subscribe(&self, subscription: Subscription) -> Result<EventStreamReceiver>;
    async fn unsubscribe(&self, subscription_id: &str) -> Result<()>;
    async fn publish_event(&self, event: Event, stream_position: u64, global_position: u64) -> Result<()>;
    async fn get_stream_position(&self, stream_id: &str) -> Result<Option<u64>>;
    async fn get_global_position(&self) -> Result<u64>;
}

/// Event stream receiver
pub type EventStreamReceiver = tokio::sync::broadcast::Receiver<StreamEvent>;

/// In-memory event streamer implementation for development and testing.
///
/// InMemoryEventStreamer provides a simple, fast event streaming implementation
/// that runs entirely in memory. It's perfect for:
/// - Development and testing
/// - Single-process applications
/// - Prototyping event-driven architectures
///
/// For production use with multiple processes, consider a distributed streaming
/// solution like Redis Streams or Apache Kafka.
///
/// # Example
///
/// ```rust
/// use eventuali_core::{InMemoryEventStreamer, SubscriptionBuilder};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let streamer = InMemoryEventStreamer::new();
///
/// let subscription = SubscriptionBuilder::new().build();
/// let mut receiver = streamer.subscribe(subscription).await?;
/// # Ok(())
/// # }
/// ```
pub struct InMemoryEventStreamer {
    sender: broadcast::Sender<StreamEvent>,
    subscriptions: Arc<Mutex<HashMap<String, Subscription>>>,
    stream_positions: Arc<Mutex<HashMap<String, u64>>>,
    global_position: Arc<Mutex<u64>>,
}

impl InMemoryEventStreamer {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        
        Self {
            sender,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            stream_positions: Arc::new(Mutex::new(HashMap::new())),
            global_position: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl EventStreamer for InMemoryEventStreamer {
    async fn subscribe(&self, subscription: Subscription) -> Result<EventStreamReceiver> {
        let mut subscriptions = self.subscriptions.lock()
            .map_err(|_| EventualiError::Configuration("Failed to acquire subscriptions lock".to_string()))?;
        
        subscriptions.insert(subscription.id.clone(), subscription);
        
        Ok(self.sender.subscribe())
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.lock()
            .map_err(|_| EventualiError::Configuration("Failed to acquire subscriptions lock".to_string()))?;
        
        subscriptions.remove(subscription_id);
        Ok(())
    }

    async fn publish_event(&self, event: Event, stream_position: u64, global_position: u64) -> Result<()> {
        // Update positions
        {
            let mut positions = self.stream_positions.lock()
                .map_err(|_| EventualiError::Configuration("Failed to acquire stream positions lock".to_string()))?;
            positions.insert(event.aggregate_id.clone(), stream_position);
        }
        
        {
            let mut global_pos = self.global_position.lock()
                .map_err(|_| EventualiError::Configuration("Failed to acquire global position lock".to_string()))?;
            *global_pos = global_position;
        }

        let stream_event = StreamEvent {
            event,
            stream_position,
            global_position,
        };

        // Send to all subscribers (ignore errors for disconnected receivers)
        let _ = self.sender.send(stream_event);
        
        Ok(())
    }

    async fn get_stream_position(&self, stream_id: &str) -> Result<Option<u64>> {
        let positions = self.stream_positions.lock()
            .map_err(|_| EventualiError::Configuration("Failed to acquire stream positions lock".to_string()))?;
        
        Ok(positions.get(stream_id).copied())
    }

    async fn get_global_position(&self) -> Result<u64> {
        let global_pos = self.global_position.lock()
            .map_err(|_| EventualiError::Configuration("Failed to acquire global position lock".to_string()))?;
        
        Ok(*global_pos)
    }
}

/// Event stream processor for handling events as they arrive
#[async_trait]
pub trait EventStreamProcessor {
    async fn process_event(&self, event: &StreamEvent) -> Result<()>;
}

/// Built-in processors
/// Projection processor that updates read models
pub struct ProjectionProcessor<P: Projection> {
    projection: Arc<P>,
}

impl<P: Projection> ProjectionProcessor<P> {
    pub fn new(projection: P) -> Self {
        Self {
            projection: Arc::new(projection),
        }
    }
}

#[async_trait]
impl<P: Projection + Send + Sync> EventStreamProcessor for ProjectionProcessor<P> {
    async fn process_event(&self, event: &StreamEvent) -> Result<()> {
        self.projection.handle_event(&event.event).await
    }
}

/// Projection trait for building read models
#[async_trait]
pub trait Projection {
    async fn handle_event(&self, event: &Event) -> Result<()>;
    async fn reset(&self) -> Result<()>;
    async fn get_last_processed_position(&self) -> Result<Option<u64>>;
    async fn set_last_processed_position(&self, position: u64) -> Result<()>;
}

/// Saga processor for long-running workflows
pub struct SagaProcessor {
    saga_handlers: HashMap<String, Box<dyn SagaHandler + Send + Sync>>,
}

impl SagaProcessor {
    pub fn new() -> Self {
        Self {
            saga_handlers: HashMap::new(),
        }
    }

    pub fn register_handler<H: SagaHandler + Send + Sync + 'static>(&mut self, event_type: String, handler: H) {
        self.saga_handlers.insert(event_type, Box::new(handler));
    }
}

#[async_trait]
impl EventStreamProcessor for SagaProcessor {
    async fn process_event(&self, event: &StreamEvent) -> Result<()> {
        if let Some(handler) = self.saga_handlers.get(&event.event.event_type) {
            handler.handle_event(&event.event).await?;
        }
        Ok(())
    }
}

/// Saga handler trait
#[async_trait]
pub trait SagaHandler {
    async fn handle_event(&self, event: &Event) -> Result<()>;
}

/// Event stream subscription builder
pub struct SubscriptionBuilder {
    subscription: Subscription,
}

impl SubscriptionBuilder {
    pub fn new() -> Self {
        Self {
            subscription: Subscription {
                id: Uuid::new_v4().to_string(),
                aggregate_type_filter: None,
                event_type_filter: None,
                from_timestamp: None,
            },
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.subscription.id = id;
        self
    }

    pub fn filter_by_aggregate_type(mut self, aggregate_type: String) -> Self {
        self.subscription.aggregate_type_filter = Some(aggregate_type);
        self
    }

    pub fn filter_by_event_type(mut self, event_type: String) -> Self {
        self.subscription.event_type_filter = Some(event_type);
        self
    }

    pub fn from_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.subscription.from_timestamp = Some(timestamp);
        self
    }

    pub fn build(self) -> Subscription {
        self.subscription
    }
}

impl Default for SubscriptionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SagaProcessor {
    fn default() -> Self {
        Self::new()
    }
}