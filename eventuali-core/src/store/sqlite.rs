use crate::{
    store::{traits::EventStoreBackend, EventStoreConfig},
    Event, EventData, EventMetadata, AggregateId, AggregateVersion, Result, EventualiError,
};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{sqlite::{SqlitePool, SqliteConnectOptions, SqliteJournalMode}, Row};
use std::str::FromStr;
use uuid::Uuid;

pub struct SQLiteBackend {
    pool: SqlitePool,
    table_name: String,
}

impl SQLiteBackend {
    pub async fn new(config: &EventStoreConfig) -> Result<Self> {
        match config {
            EventStoreConfig::SQLite {
                database_path,
                max_connections,
                table_name,
            } => {
                let pool = if database_path == ":memory:" {
                    // For in-memory databases, use the simple connection string
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(max_connections.unwrap_or(10))
                        .connect("sqlite://:memory:")
                        .await?
                } else {
                    // For file-based SQLite, use SqliteConnectOptions with create_if_missing
                    let path = std::path::Path::new(database_path);
                    let full_path = if path.is_absolute() {
                        database_path.clone()
                    } else {
                        // Convert relative path to absolute path
                        std::env::current_dir()
                            .map_err(|e| EventualiError::Configuration(format!("Cannot get current directory: {}", e)))?
                            .join(path)
                            .to_string_lossy()
                            .to_string()
                    };
                    
                    // Create parent directories if they don't exist
                    let db_path = std::path::Path::new(&full_path);
                    if let Some(parent) = db_path.parent() {
                        if !parent.exists() {
                            std::fs::create_dir_all(parent)
                                .map_err(|e| EventualiError::Configuration(format!("Cannot create directory {}: {}", parent.display(), e)))?;
                        }
                    }
                    
                    
                    // Use SqliteConnectOptions for proper file database creation
                    let connect_options = SqliteConnectOptions::from_str(&full_path)
                        .map_err(|e| EventualiError::Configuration(format!("Invalid SQLite path {}: {}", full_path, e)))?
                        .create_if_missing(true)
                        .journal_mode(SqliteJournalMode::Wal);
                    
                    sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(max_connections.unwrap_or(10))
                        .connect_with(connect_options)
                        .await?
                };

                let table_name = table_name
                    .as_deref()
                    .unwrap_or("events")
                    .to_string();

                let backend = Self { pool, table_name };
                Ok(backend)
            }
            _ => Err(EventualiError::Configuration(
                "Invalid configuration for SQLite backend".to_string(),
            )),
        }
    }

    async fn create_tables(&self) -> Result<()> {
        // Enable foreign keys (WAL mode is set in connection options)
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;

        let create_events_table = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                aggregate_id TEXT NOT NULL,
                aggregate_type TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_version INTEGER NOT NULL,
                aggregate_version INTEGER NOT NULL,
                event_data TEXT NOT NULL,
                event_data_type TEXT NOT NULL DEFAULT 'json',
                metadata TEXT NOT NULL,
                timestamp TEXT NOT NULL,
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
impl EventStoreBackend for SQLiteBackend {
    async fn initialize(&mut self) -> Result<()> {
        self.create_tables().await
    }

    async fn save_events(&self, events: Vec<Event>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for event in events {
            let (event_data_text, event_data_type) = match &event.data {
                EventData::Json(value) => (serde_json::to_string(value)?, "json"),
                EventData::Protobuf(bytes) => {
                    // Store protobuf as base64 for SQLite
                    let base64_data = general_purpose::STANDARD.encode(bytes);
                    (base64_data, "protobuf")
                }
            };

            let metadata_text = serde_json::to_string(&event.metadata)?;
            let timestamp_text = event.timestamp.to_rfc3339();

            let query = format!(
                r#"
                INSERT INTO {} (
                    id, aggregate_id, aggregate_type, event_type, event_version,
                    aggregate_version, event_data, event_data_type, metadata, timestamp
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                self.table_name
            );

            sqlx::query(&query)
                .bind(event.id.to_string())
                .bind(&event.aggregate_id)
                .bind(&event.aggregate_type)
                .bind(&event.event_type)
                .bind(&event.event_version)
                .bind(&event.aggregate_version)
                .bind(&event_data_text)
                .bind(event_data_type)
                .bind(&metadata_text)
                .bind(&timestamp_text)
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
                WHERE aggregate_id = ? AND aggregate_version > ?
                ORDER BY aggregate_version ASC
                "#,
                self.table_name
            ),
            None => format!(
                r#"
                SELECT id, aggregate_id, aggregate_type, event_type, event_version,
                       aggregate_version, event_data, event_data_type, metadata, timestamp
                FROM {} 
                WHERE aggregate_id = ?
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
                WHERE aggregate_type = ? AND aggregate_version > ?
                ORDER BY timestamp ASC
                "#,
                self.table_name
            ),
            None => format!(
                r#"
                SELECT id, aggregate_id, aggregate_type, event_type, event_version,
                       aggregate_version, event_data, event_data_type, metadata, timestamp
                FROM {} 
                WHERE aggregate_type = ?
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
            "SELECT MAX(aggregate_version) FROM {} WHERE aggregate_id = ?",
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

impl SQLiteBackend {
    fn row_to_event(&self, row: sqlx::sqlite::SqliteRow) -> Result<Event> {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| EventualiError::InvalidEventData("Invalid UUID format".to_string()))?;
        
        let aggregate_id: String = row.try_get("aggregate_id")?;
        let aggregate_type: String = row.try_get("aggregate_type")?;
        let event_type: String = row.try_get("event_type")?;
        let event_version: i32 = row.try_get("event_version")?;
        let aggregate_version: i64 = row.try_get("aggregate_version")?;
        let event_data_text: String = row.try_get("event_data")?;
        let event_data_type: String = row.try_get("event_data_type")?;
        let metadata_text: String = row.try_get("metadata")?;
        let timestamp_text: String = row.try_get("timestamp")?;

        let event_data = match event_data_type.as_str() {
            "json" => {
                let json_value: serde_json::Value = serde_json::from_str(&event_data_text)?;
                EventData::Json(json_value)
            }
            "protobuf" => {
                let bytes = general_purpose::STANDARD.decode(&event_data_text).map_err(|_| {
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

        let metadata: EventMetadata = serde_json::from_str(&metadata_text)?;
        let timestamp: DateTime<Utc> = DateTime::parse_from_rfc3339(&timestamp_text)
            .map_err(|_| EventualiError::InvalidEventData("Invalid timestamp format".to_string()))?
            .with_timezone(&Utc);

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