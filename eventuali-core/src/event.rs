//! Core event types and structures for the Eventuali event sourcing library.
//!
//! This module defines the fundamental building blocks of event sourcing:
//! - [`Event`] - The core event structure containing all event data
//! - [`EventData`] - Flexible event payload supporting JSON and Protocol Buffers
//! - [`EventMetadata`] - Correlation, causation, and tracing information
//!
//! # Example
//!
//! ```rust
//! use eventuali_core::{Event, EventData, EventMetadata};
//! use chrono::Utc;
//! use serde_json::json;
//! use uuid::Uuid;
//!
//! let event = Event {
//!     id: Uuid::new_v4(),
//!     aggregate_id: "user-123".to_string(),
//!     aggregate_type: "User".to_string(),
//!     event_type: "UserRegistered".to_string(),
//!     event_version: 1,
//!     aggregate_version: 1,
//!     data: EventData::from_json(&json!({
//!         "name": "Alice",
//!         "email": "alice@example.com"
//!     })).unwrap(),
//!     metadata: EventMetadata::default(),
//!     timestamp: Utc::now(),
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for events, using UUID for global uniqueness
pub type EventId = Uuid;

/// Core event structure representing a domain event in the event store.
///
/// Events are immutable records of things that happened in your domain.
/// Each event belongs to an aggregate and represents a state change.
///
/// # Fields
///
/// * `id` - Unique identifier for this event
/// * `aggregate_id` - ID of the aggregate this event belongs to
/// * `aggregate_type` - Type of aggregate (e.g., "User", "Order")
/// * `event_type` - Type of event (e.g., "UserRegistered", "OrderCreated")
/// * `event_version` - Schema version of the event type
/// * `aggregate_version` - Version of the aggregate after this event
/// * `data` - The event payload (JSON or Protocol Buffers)
/// * `metadata` - Additional event metadata for tracing and correlation
/// * `timestamp` - When the event occurred
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_version: i32,
    pub aggregate_version: i64,
    pub data: EventData,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
}

/// Event payload data supporting multiple serialization formats.
///
/// EventData provides flexibility in how you store event payloads:
/// - JSON for human-readable, schema-flexible events
/// - Protocol Buffers for compact, strongly-typed events
///
/// # Examples
///
/// ```rust
/// use eventuali_core::EventData;
/// use serde_json::json;
///
/// // JSON event data
/// let json_data = EventData::from_json(&json!({
///     "user_id": "123",
///     "action": "login"
/// })).unwrap();
///
/// // Protocol Buffer event data
/// let proto_data = EventData::Protobuf(vec![1, 2, 3, 4]);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventData {
    Json(serde_json::Value),
    Protobuf(Vec<u8>),
}

/// Metadata associated with events for tracing, correlation, and context.
///
/// EventMetadata provides essential context for event processing:
/// - Causation and correlation IDs for distributed tracing
/// - User context for audit trails
/// - Custom headers for additional context
///
/// # Example
///
/// ```rust
/// use eventuali_core::EventMetadata;
/// use uuid::Uuid;
/// use std::collections::HashMap;
///
/// let mut metadata = EventMetadata::default();
/// metadata.user_id = Some("user-123".to_string());
/// metadata.correlation_id = Some(Uuid::new_v4());
/// metadata.headers.insert("source".to_string(), "web-app".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EventMetadata {
    pub causation_id: Option<EventId>,
    pub correlation_id: Option<EventId>,
    pub user_id: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}

impl Event {
    /// Creates a new event with the specified parameters.
    ///
    /// This is a convenience constructor that generates a new UUID for the event ID
    /// and sets the timestamp to the current UTC time.
    ///
    /// # Arguments
    ///
    /// * `aggregate_id` - ID of the aggregate this event belongs to
    /// * `aggregate_type` - Type of the aggregate
    /// * `event_type` - Type of the event
    /// * `event_version` - Schema version of the event
    /// * `aggregate_version` - Version of the aggregate after this event
    /// * `data` - The event payload
    ///
    /// # Example
    ///
    /// ```rust
    /// use eventuali_core::{Event, EventData};
    /// use serde_json::json;
    ///
    /// let event = Event::new(
    ///     "user-123".to_string(),
    ///     "User".to_string(),
    ///     "UserRegistered".to_string(),
    ///     1,
    ///     1,
    ///     EventData::from_json(&json!({"name": "Alice"})).unwrap(),
    /// );
    /// ```
    pub fn new(
        aggregate_id: String,
        aggregate_type: String,
        event_type: String,
        event_version: i32,
        aggregate_version: i64,
        data: EventData,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type,
            event_type,
            event_version,
            aggregate_version,
            data,
            metadata: EventMetadata::default(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}


impl EventData {
    pub fn from_json<T: Serialize>(value: &T) -> crate::Result<Self> {
        let json_value = serde_json::to_value(value)?;
        Ok(EventData::Json(json_value))
    }

    pub fn to_json<T: for<'de> Deserialize<'de>>(&self) -> crate::Result<T> {
        match self {
            EventData::Json(value) => Ok(serde_json::from_value(value.clone())?),
            EventData::Protobuf(_) => Err(crate::EventualiError::InvalidEventData(
                "Cannot deserialize protobuf data as JSON".to_string(),
            )),
        }
    }

    pub fn from_protobuf(data: Vec<u8>) -> Self {
        EventData::Protobuf(data)
    }

    pub fn to_protobuf(&self) -> crate::Result<&[u8]> {
        match self {
            EventData::Protobuf(data) => Ok(data),
            EventData::Json(_) => Err(crate::EventualiError::InvalidEventData(
                "Cannot get protobuf data from JSON event".to_string(),
            )),
        }
    }

    pub fn from_protobuf_message<T: prost::Message>(message: &T) -> crate::Result<Self> {
        let mut buf = Vec::new();
        message.encode(&mut buf)
            .map_err(|e| crate::EventualiError::InvalidEventData(e.to_string()))?;
        Ok(EventData::Protobuf(buf))
    }

    pub fn to_protobuf_message<T: prost::Message + Default>(&self) -> crate::Result<T> {
        match self {
            EventData::Protobuf(data) => {
                T::decode(&data[..])
                    .map_err(crate::EventualiError::Protobuf)
            },
            EventData::Json(_) => Err(crate::EventualiError::InvalidEventData(
                "Cannot decode protobuf message from JSON data".to_string(),
            )),
        }
    }
}