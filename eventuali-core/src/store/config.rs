use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStoreConfig {
    PostgreSQL {
        connection_string: String,
        max_connections: Option<u32>,
        table_name: Option<String>,
    },
    SQLite {
        database_path: String,
        max_connections: Option<u32>,
        table_name: Option<String>,
    },
}

impl EventStoreConfig {
    pub fn postgres(connection_string: String) -> Self {
        Self::PostgreSQL {
            connection_string,
            max_connections: None,
            table_name: None,
        }
    }

    pub fn postgres_with_pool(connection_string: String, max_connections: u32) -> Self {
        Self::PostgreSQL {
            connection_string,
            max_connections: Some(max_connections),
            table_name: None,
        }
    }

    pub fn sqlite(database_path: String) -> Self {
        Self::SQLite {
            database_path,
            max_connections: None,
            table_name: None,
        }
    }

    pub fn sqlite_with_pool(database_path: String, max_connections: u32) -> Self {
        Self::SQLite {
            database_path,
            max_connections: Some(max_connections),
            table_name: None,
        }
    }

    pub fn with_table_name(mut self, table_name: String) -> Self {
        match &mut self {
            EventStoreConfig::PostgreSQL { table_name: t, .. } => *t = Some(table_name),
            EventStoreConfig::SQLite { table_name: t, .. } => *t = Some(table_name),
        }
        self
    }

    pub fn table_name(&self) -> &str {
        match self {
            EventStoreConfig::PostgreSQL { table_name, .. } |
            EventStoreConfig::SQLite { table_name, .. } => {
                table_name.as_deref().unwrap_or("events")
            }
        }
    }

    pub fn max_connections(&self) -> u32 {
        match self {
            EventStoreConfig::PostgreSQL { max_connections, .. } |
            EventStoreConfig::SQLite { max_connections, .. } => {
                max_connections.unwrap_or(10)
            }
        }
    }
}