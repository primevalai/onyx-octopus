use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type EventId = Uuid;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventData {
    Json(serde_json::Value),
    Protobuf(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub causation_id: Option<EventId>,
    pub correlation_id: Option<EventId>,
    pub user_id: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}

impl Event {
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

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            causation_id: None,
            correlation_id: None,
            user_id: None,
            headers: std::collections::HashMap::new(),
        }
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
                    .map_err(|e| crate::EventualiError::Protobuf(e))
            },
            EventData::Json(_) => Err(crate::EventualiError::InvalidEventData(
                "Cannot decode protobuf message from JSON data".to_string(),
            )),
        }
    }
}