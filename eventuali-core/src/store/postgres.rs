use crate::{
    store::{traits::EventStoreBackend, EventStoreConfig},
    Event, EventData, EventMetadata, AggregateId, AggregateVersion, Result, EventualiError,
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{postgres::PgPool, Row};
use uuid::Uuid;

pub struct PostgreSQLBackend {
    pool: PgPool,
    table_name: String,
}

impl PostgreSQLBackend {
    pub async fn new(config: &EventStoreConfig) -> Result<Self> {
        match config {
            EventStoreConfig::PostgreSQL {
                connection_string,
                max_connections,
                table_name,
            } => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(max_connections.unwrap_or(10))
                    .connect(connection_string)
                    .await?;

                let table_name = table_name
                    .as_deref()
                    .unwrap_or("events")
                    .to_string();

                let backend = Self { pool, table_name };
                Ok(backend)
            }
            _ => Err(EventualiError::Configuration(
                "Invalid configuration for PostgreSQL backend".to_string(),
            )),
        }
    }

    async fn create_tables(&self) -> Result<()> {
        let create_events_table = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id UUID PRIMARY KEY,
                aggregate_id VARCHAR NOT NULL,
                aggregate_type VARCHAR NOT NULL,
                event_type VARCHAR NOT NULL,
                event_version INTEGER NOT NULL,
                aggregate_version BIGINT NOT NULL,
                event_data JSONB NOT NULL,
                event_data_type VARCHAR NOT NULL DEFAULT 'json',
                metadata JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(aggregate_id, aggregate_version)
            );
            
            CREATE INDEX IF NOT EXISTS idx_{}_aggregate_id ON {} (aggregate_id);
            CREATE INDEX IF NOT EXISTS idx_{}_aggregate_type ON {} (aggregate_type);
            CREATE INDEX IF NOT EXISTS idx_{}_timestamp ON {} (timestamp);
            "#,
            self.table_name, 
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name
        );

        sqlx::query(&create_events_table)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl EventStoreBackend for PostgreSQLBackend {
    async fn initialize(&mut self) -> Result<()> {
        self.create_tables().await
    }

    async fn save_events(&self, events: Vec<Event>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for event in events {
            let (event_data_json, event_data_type) = match &event.data {
                EventData::Json(value) => (value.clone(), "json"),
                EventData::Protobuf(bytes) => {
                    // Store protobuf as base64 encoded JSON for PostgreSQL
                    let base64_data = general_purpose::STANDARD.encode(bytes);
                    (serde_json::json!({ "data": base64_data }), "protobuf")
                }
            };

            let metadata_json = serde_json::to_value(&event.metadata)?;

            let query = format!(
                r#"
                INSERT INTO {} (
                    id, aggregate_id, aggregate_type, event_type, event_version,
                    aggregate_version, event_data, event_data_type, metadata, timestamp
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                self.table_name
            );

            sqlx::query(&query)
                .bind(&event.id)
                .bind(&event.aggregate_id)
                .bind(&event.aggregate_type)
                .bind(&event.event_type)
                .bind(&event.event_version)
                .bind(&event.aggregate_version)
                .bind(&event_data_json)
                .bind(event_data_type)
                .bind(&metadata_json)
                .bind(&event.timestamp)
                .execute(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                        EventualiError::OptimisticConcurrency {
                            expected: event.aggregate_version,
                            actual: event.aggregate_version - 1,
                        }
                    }
                    _ => EventualiError::Database(e),
                })?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>> {
        let query = match from_version {
            Some(_version) => format!(
                r#"
                SELECT id, aggregate_id, aggregate_type, event_type, event_version,
                       aggregate_version, event_data, event_data_type, metadata, timestamp
                FROM {} 
                WHERE aggregate_id = $1 AND aggregate_version > $2
                ORDER BY aggregate_version ASC
                "#,
                self.table_name
            ),
            None => format!(
                r#"
                SELECT id, aggregate_id, aggregate_type, event_type, event_version,
                       aggregate_version, event_data, event_data_type, metadata, timestamp
                FROM {} 
                WHERE aggregate_id = $1
                ORDER BY aggregate_version ASC
                "#,
                self.table_name
            ),
        };

        let rows = if let Some(version) = from_version {
            sqlx::query(&query)
                .bind(aggregate_id)
                .bind(version)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&query)
                .bind(aggregate_id)
                .fetch_all(&self.pool)
                .await?
        };

        let mut events = Vec::new();
        for row in rows {
            let event = self.row_to_event(row)?;
            events.push(event);
        }

        Ok(events)
    }

    async fn load_events_by_type(
        &self,
        aggregate_type: &str,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>> {
        let query = match from_version {
            Some(_version) => format!(
                r#"
                SELECT id, aggregate_id, aggregate_type, event_type, event_version,
                       aggregate_version, event_data, event_data_type, metadata, timestamp
                FROM {} 
                WHERE aggregate_type = $1 AND aggregate_version > $2
                ORDER BY timestamp ASC
                "#,
                self.table_name
            ),
            None => format!(
                r#"
                SELECT id, aggregate_id, aggregate_type, event_type, event_version,
                       aggregate_version, event_data, event_data_type, metadata, timestamp
                FROM {} 
                WHERE aggregate_type = $1
                ORDER BY timestamp ASC
                "#,
                self.table_name
            ),
        };

        let rows = if let Some(version) = from_version {
            sqlx::query(&query)
                .bind(aggregate_type)
                .bind(version)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&query)
                .bind(aggregate_type)
                .fetch_all(&self.pool)
                .await?
        };

        let mut events = Vec::new();
        for row in rows {
            let event = self.row_to_event(row)?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_aggregate_version(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateVersion>> {
        let query = format!(
            "SELECT MAX(aggregate_version) FROM {} WHERE aggregate_id = $1",
            self.table_name
        );

        let row = sqlx::query(&query)
            .bind(aggregate_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let version: Option<i64> = row.try_get(0)?;
            Ok(version)
        } else {
            Ok(None)
        }
    }
}

impl PostgreSQLBackend {
    fn row_to_event(&self, row: sqlx::postgres::PgRow) -> Result<Event> {
        let id: Uuid = row.try_get("id")?;
        let aggregate_id: String = row.try_get("aggregate_id")?;
        let aggregate_type: String = row.try_get("aggregate_type")?;
        let event_type: String = row.try_get("event_type")?;
        let event_version: i32 = row.try_get("event_version")?;
        let aggregate_version: i64 = row.try_get("aggregate_version")?;
        let event_data_json: serde_json::Value = row.try_get("event_data")?;
        let event_data_type: String = row.try_get("event_data_type")?;
        let metadata_json: serde_json::Value = row.try_get("metadata")?;
        let timestamp: DateTime<Utc> = row.try_get("timestamp")?;

        let event_data = match event_data_type.as_str() {
            "json" => EventData::Json(event_data_json),
            "protobuf" => {
                let base64_data = event_data_json
                    .get("data")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        EventualiError::InvalidEventData("Invalid protobuf data format".to_string())
                    })?;
                let bytes = general_purpose::STANDARD.decode(base64_data).map_err(|_| {
                    EventualiError::InvalidEventData("Invalid base64 protobuf data".to_string())
                })?;
                EventData::Protobuf(bytes)
            }
            _ => {
                return Err(EventualiError::InvalidEventData(format!(
                    "Unknown event data type: {}",
                    event_data_type
                )))
            }
        };

        let metadata: EventMetadata = serde_json::from_value(metadata_json)?;

        Ok(Event {
            id,
            aggregate_id,
            aggregate_type,
            event_type,
            event_version,
            aggregate_version,
            data: event_data,
            metadata,
            timestamp,
        })
    }
}