use crate::{Event, EventData, EventMetadata, Result, EventualiError};
use crate::aggregate::AggregateSnapshot;
use prost::Message;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Include generated protobuf code
pub mod eventuali {
    include!(concat!(env!("OUT_DIR"), "/eventuali.rs"));
}

pub struct ProtoSerializer;

impl ProtoSerializer {
    pub fn new() -> Self {
        Self
    }

    /// Serialize an event to Protocol Buffers format
    pub fn serialize_event(&self, event: &Event) -> Result<Vec<u8>> {
        let proto_event = self.event_to_proto(event)?;
        let mut buf = Vec::new();
        proto_event.encode(&mut buf)
            .map_err(|e| EventualiError::InvalidEventData(e.to_string()))?;
        Ok(buf)
    }

    /// Deserialize an event from Protocol Buffers format
    pub fn deserialize_event(&self, data: &[u8]) -> Result<Event> {
        let proto_event = eventuali::Event::decode(data)
            .map_err(|e| EventualiError::Protobuf(prost::DecodeError::new(e.to_string())))?;
        self.proto_to_event(proto_event)
    }

    /// Serialize an aggregate snapshot to Protocol Buffers format
    pub fn serialize_snapshot(&self, snapshot: &AggregateSnapshot) -> Result<Vec<u8>> {
        let data_bytes = serde_json::to_vec(&snapshot.data)
            .map_err(EventualiError::Serialization)?;
        
        let proto_snapshot = eventuali::AggregateSnapshot {
            aggregate_id: snapshot.aggregate_id.clone(),
            aggregate_type: snapshot.aggregate_type.clone(),
            version: snapshot.version,
            data: data_bytes,
            timestamp: snapshot.timestamp.timestamp(),
        };

        let mut buf = Vec::new();
        proto_snapshot.encode(&mut buf)
            .map_err(|e| EventualiError::InvalidEventData(e.to_string()))?;
        Ok(buf)
    }

    /// Deserialize an aggregate snapshot from Protocol Buffers format
    pub fn deserialize_snapshot(&self, data: &[u8]) -> Result<AggregateSnapshot> {
        let proto_snapshot = eventuali::AggregateSnapshot::decode(data)
            .map_err(|e| EventualiError::Protobuf(prost::DecodeError::new(e.to_string())))?;

        let data = if !proto_snapshot.data.is_empty() {
            serde_json::from_slice(&proto_snapshot.data)
                .map_err(EventualiError::Serialization)?
        } else {
            serde_json::json!({})
        };

        let timestamp = DateTime::from_timestamp(proto_snapshot.timestamp, 0)
            .unwrap_or_else(Utc::now)
            .with_timezone(&Utc);

        Ok(AggregateSnapshot {
            aggregate_id: proto_snapshot.aggregate_id,
            aggregate_type: proto_snapshot.aggregate_type,
            version: proto_snapshot.version,
            data,
            timestamp,
        })
    }

    fn event_to_proto(&self, event: &Event) -> Result<eventuali::Event> {
        let data_bytes = match &event.data {
            EventData::Json(json) => {
                serde_json::to_vec(json)
                    .map_err(EventualiError::Serialization)?
            },
            EventData::Protobuf(bytes) => bytes.clone(),
        };

        let metadata = eventuali::EventMetadata {
            causation_id: event.metadata.causation_id.map(|id| id.to_string()).unwrap_or_default(),
            correlation_id: event.metadata.correlation_id.map(|id| id.to_string()).unwrap_or_default(),
            user_id: event.metadata.user_id.clone().unwrap_or_default(),
            headers: event.metadata.headers.clone(),
        };

        Ok(eventuali::Event {
            id: event.id.to_string(),
            aggregate_id: event.aggregate_id.clone(),
            aggregate_type: event.aggregate_type.clone(),
            event_type: event.event_type.clone(),
            event_version: event.event_version,
            aggregate_version: event.aggregate_version,
            data: data_bytes,
            metadata: Some(metadata),
            timestamp: event.timestamp.timestamp(),
        })
    }

    fn proto_to_event(&self, proto_event: eventuali::Event) -> Result<Event> {
        let id = Uuid::parse_str(&proto_event.id)
            .map_err(|_| EventualiError::InvalidEventData("Invalid UUID".to_string()))?;

        let data = if !proto_event.data.is_empty() {
            EventData::Protobuf(proto_event.data)
        } else {
            EventData::Json(serde_json::json!({}))
        };

        let metadata = if let Some(meta) = proto_event.metadata {
            EventMetadata {
                causation_id: if meta.causation_id.is_empty() { 
                    None 
                } else { 
                    Uuid::parse_str(&meta.causation_id).ok() 
                },
                correlation_id: if meta.correlation_id.is_empty() { 
                    None 
                } else { 
                    Uuid::parse_str(&meta.correlation_id).ok() 
                },
                user_id: if meta.user_id.is_empty() { None } else { Some(meta.user_id) },
                headers: meta.headers,
            }
        } else {
            EventMetadata::default()
        };

        let timestamp = DateTime::from_timestamp(proto_event.timestamp, 0)
            .unwrap_or_else(Utc::now)
            .with_timezone(&Utc);

        Ok(Event {
            id,
            aggregate_id: proto_event.aggregate_id,
            aggregate_type: proto_event.aggregate_type,
            event_type: proto_event.event_type,
            event_version: proto_event.event_version,
            aggregate_version: proto_event.aggregate_version,
            data,
            metadata,
            timestamp,
        })
    }
}

impl Default for ProtoSerializer {
    fn default() -> Self {
        Self::new()
    }
}

// Utility functions for specific event types
impl ProtoSerializer {
    /// Create a UserRegistered event
    pub fn create_user_registered(name: String, email: String) -> eventuali::UserRegistered {
        eventuali::UserRegistered { name, email }
    }

    /// Create a UserEmailChanged event
    pub fn create_user_email_changed(old_email: String, new_email: String) -> eventuali::UserEmailChanged {
        eventuali::UserEmailChanged { old_email, new_email }
    }

    /// Create a UserDeactivated event
    pub fn create_user_deactivated(reason: String) -> eventuali::UserDeactivated {
        eventuali::UserDeactivated { reason }
    }

    /// Create an OrderPlaced event
    pub fn create_order_placed(
        customer_id: String, 
        items: Vec<eventuali::OrderItem>, 
        total_amount: f64
    ) -> eventuali::OrderPlaced {
        eventuali::OrderPlaced { customer_id, items, total_amount }
    }
}