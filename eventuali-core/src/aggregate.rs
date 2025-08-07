use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type AggregateId = String;
pub type AggregateVersion = i64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aggregate {
    pub id: AggregateId,
    pub version: AggregateVersion,
    pub aggregate_type: String,
}

impl Aggregate {
    pub fn new(id: AggregateId, aggregate_type: String) -> Self {
        Self {
            id,
            version: 0,
            aggregate_type,
        }
    }

    pub fn new_with_uuid(aggregate_type: String) -> Self {
        Self::new(Uuid::new_v4().to_string(), aggregate_type)
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn is_new(&self) -> bool {
        self.version == 0
    }
}

#[derive(Debug, Clone)]
pub struct AggregateSnapshot {
    pub aggregate_id: AggregateId,
    pub aggregate_type: String,
    pub version: AggregateVersion,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AggregateSnapshot {
    pub fn new(
        aggregate_id: AggregateId,
        aggregate_type: String,
        version: AggregateVersion,
        data: serde_json::Value,
    ) -> Self {
        Self {
            aggregate_id,
            aggregate_type,
            version,
            data,
            timestamp: chrono::Utc::now(),
        }
    }
}