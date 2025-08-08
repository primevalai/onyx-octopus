use super::{AggregateSnapshot, SnapshotStore, SnapshotConfig, SnapshotCompression};
use crate::{AggregateId, AggregateVersion, Result, EventualiError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{sqlite::SqlitePool, Row};
use uuid::Uuid;

pub struct SqliteSnapshotStore {
    pool: SqlitePool,
    table_name: String,
}

impl SqliteSnapshotStore {
    pub fn new(pool: SqlitePool, table_name: Option<String>) -> Self {
        Self {
            pool,
            table_name: table_name.unwrap_or_else(|| "aggregate_snapshots".to_string()),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let create_table = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                snapshot_id TEXT PRIMARY KEY,
                aggregate_id TEXT NOT NULL,
                aggregate_type TEXT NOT NULL,
                aggregate_version INTEGER NOT NULL,
                state_data BLOB NOT NULL,
                compression TEXT NOT NULL,
                metadata TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(aggregate_id, aggregate_version)
            );
            
            CREATE INDEX IF NOT EXISTS idx_{}_aggregate_id ON {} (aggregate_id);
            CREATE INDEX IF NOT EXISTS idx_{}_aggregate_type ON {} (aggregate_type);
            CREATE INDEX IF NOT EXISTS idx_{}_created_at ON {} (created_at);
            CREATE INDEX IF NOT EXISTS idx_{}_aggregate_version ON {} (aggregate_id, aggregate_version DESC);
            "#,
            self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name
        );

        sqlx::query(&create_table)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl SnapshotStore for SqliteSnapshotStore {
    async fn save_snapshot(&self, snapshot: AggregateSnapshot) -> Result<()> {
        let compression_str = match snapshot.compression {
            SnapshotCompression::None => "none",
            SnapshotCompression::Gzip => "gzip",
            SnapshotCompression::Lz4 => "lz4",
        };

        let metadata_json = serde_json::to_string(&snapshot.metadata)?;

        let query = format!(
            r#"
            INSERT INTO {} (
                snapshot_id, aggregate_id, aggregate_type, aggregate_version,
                state_data, compression, metadata, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            self.table_name
        );

        sqlx::query(&query)
            .bind(snapshot.snapshot_id.to_string())
            .bind(&snapshot.aggregate_id)
            .bind(&snapshot.aggregate_type)
            .bind(snapshot.aggregate_version)
            .bind(&snapshot.state_data)
            .bind(compression_str)
            .bind(&metadata_json)
            .bind(snapshot.created_at.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                    EventualiError::Configuration(format!(
                        "Snapshot already exists for aggregate {} at version {}",
                        snapshot.aggregate_id, snapshot.aggregate_version
                    ))
                }
                _ => EventualiError::Database(e),
            })?;

        Ok(())
    }

    async fn load_latest_snapshot(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateSnapshot>> {
        let query = format!(
            r#"
            SELECT snapshot_id, aggregate_id, aggregate_type, aggregate_version,
                   state_data, compression, metadata, created_at
            FROM {}
            WHERE aggregate_id = ?
            ORDER BY aggregate_version DESC
            LIMIT 1
            "#,
            self.table_name
        );

        let row = sqlx::query(&query)
            .bind(aggregate_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_snapshot(row)?))
        } else {
            Ok(None)
        }
    }

    async fn load_snapshot(&self, snapshot_id: Uuid) -> Result<Option<AggregateSnapshot>> {
        let query = format!(
            r#"
            SELECT snapshot_id, aggregate_id, aggregate_type, aggregate_version,
                   state_data, compression, metadata, created_at
            FROM {}
            WHERE snapshot_id = ?
            "#,
            self.table_name
        );

        let row = sqlx::query(&query)
            .bind(snapshot_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_snapshot(row)?))
        } else {
            Ok(None)
        }
    }

    async fn list_snapshots(&self, aggregate_id: &AggregateId) -> Result<Vec<AggregateSnapshot>> {
        let query = format!(
            r#"
            SELECT snapshot_id, aggregate_id, aggregate_type, aggregate_version,
                   state_data, compression, metadata, created_at
            FROM {}
            WHERE aggregate_id = ?
            ORDER BY aggregate_version DESC
            "#,
            self.table_name
        );

        let rows = sqlx::query(&query)
            .bind(aggregate_id)
            .fetch_all(&self.pool)
            .await?;

        let mut snapshots = Vec::new();
        for row in rows {
            snapshots.push(self.row_to_snapshot(row)?);
        }

        Ok(snapshots)
    }

    async fn delete_snapshot(&self, snapshot_id: Uuid) -> Result<()> {
        let query = format!("DELETE FROM {} WHERE snapshot_id = ?", self.table_name);

        sqlx::query(&query)
            .bind(snapshot_id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn cleanup_old_snapshots(&self, config: &SnapshotConfig) -> Result<u64> {
        if !config.auto_cleanup {
            return Ok(0);
        }

        let cutoff_time = Utc::now() - chrono::Duration::hours(config.max_snapshot_age_hours as i64);

        let query = format!(
            "DELETE FROM {} WHERE created_at < ?",
            self.table_name
        );

        let result = sqlx::query(&query)
            .bind(cutoff_time.to_rfc3339())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    async fn should_take_snapshot(
        &self,
        aggregate_id: &AggregateId,
        current_version: AggregateVersion,
        config: &SnapshotConfig,
    ) -> Result<bool> {
        // Check if we should take a snapshot based on frequency
        if current_version % config.snapshot_frequency != 0 {
            return Ok(false);
        }

        // Check if we already have a snapshot at this version
        let query = format!(
            "SELECT COUNT(*) FROM {} WHERE aggregate_id = ? AND aggregate_version = ?",
            self.table_name
        );

        let row = sqlx::query(&query)
            .bind(aggregate_id)
            .bind(current_version)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.try_get(0)?;
        Ok(count == 0)
    }
}

impl SqliteSnapshotStore {
    fn row_to_snapshot(&self, row: sqlx::sqlite::SqliteRow) -> Result<AggregateSnapshot> {
        let snapshot_id_str: String = row.try_get("snapshot_id")?;
        let snapshot_id = Uuid::parse_str(&snapshot_id_str)
            .map_err(|_| EventualiError::InvalidEventData("Invalid snapshot UUID format".to_string()))?;

        let aggregate_id: String = row.try_get("aggregate_id")?;
        let aggregate_type: String = row.try_get("aggregate_type")?;
        let aggregate_version: i64 = row.try_get("aggregate_version")?;
        let state_data: Vec<u8> = row.try_get("state_data")?;
        let compression_str: String = row.try_get("compression")?;
        let metadata_json: String = row.try_get("metadata")?;
        let created_at_str: String = row.try_get("created_at")?;

        let compression = match compression_str.as_str() {
            "none" => SnapshotCompression::None,
            "gzip" => SnapshotCompression::Gzip,
            "lz4" => SnapshotCompression::Lz4,
            _ => return Err(EventualiError::InvalidEventData(format!(
                "Unknown compression type: {compression_str}"
            ))),
        };

        let metadata = serde_json::from_str(&metadata_json)?;

        let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| EventualiError::InvalidEventData("Invalid timestamp format".to_string()))?
            .with_timezone(&Utc);

        Ok(AggregateSnapshot {
            snapshot_id,
            aggregate_id,
            aggregate_type,
            aggregate_version,
            state_data,
            compression,
            metadata,
            created_at,
        })
    }
}