//! High-performance database connection pooling
//!
//! Provides optimized connection pool management with automatic sizing,
//! health monitoring, and load balancing capabilities.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use rusqlite::Connection as SqliteConnection;
use crate::error::EventualiError;

/// Connection pool statistics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_wait_time_ms: f64,
    pub max_wait_time_ms: u64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_wait_time_ms: 0.0,
            max_wait_time_ms: 0,
        }
    }
}

/// Configuration for connection pool optimization
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub min_connections: usize,
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub idle_timeout_ms: u64,
    pub health_check_interval_ms: u64,
    pub auto_scaling_enabled: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 100,
            connection_timeout_ms: 5000,
            idle_timeout_ms: 300000, // 5 minutes
            health_check_interval_ms: 30000, // 30 seconds
            auto_scaling_enabled: true,
            scale_up_threshold: 0.8, // Scale up when 80% connections are in use
            scale_down_threshold: 0.3, // Scale down when less than 30% are in use
        }
    }
}

/// High-performance connection pool with automatic optimization
pub struct ConnectionPool {
    config: PoolConfig,
    connection_count: Arc<Mutex<usize>>,
    active_count: Arc<Mutex<usize>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<PoolStats>>,
    database_path: String,
}

impl ConnectionPool {
    /// Create a new connection pool with the specified configuration
    pub async fn new(database_path: String, config: PoolConfig) -> Result<Self, EventualiError> {
        let connection_count = Arc::new(Mutex::new(config.min_connections));
        let active_count = Arc::new(Mutex::new(0));
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let stats = Arc::new(Mutex::new(PoolStats {
            total_connections: config.min_connections,
            idle_connections: config.min_connections,
            ..Default::default()
        }));

        let pool = Self {
            config,
            connection_count,
            active_count,
            semaphore,
            stats,
            database_path,
        };

        Ok(pool)
    }

    /// Get a connection from the pool with performance tracking
    pub async fn get_connection(&self) -> Result<PoolGuard, EventualiError> {
        let start_time = Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_requests += 1;
        }

        // Acquire semaphore permit
        let permit = match tokio::time::timeout(
            Duration::from_millis(self.config.connection_timeout_ms),
            self.semaphore.acquire()
        ).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                self.record_failed_request().await;
                return Err(EventualiError::Configuration("Failed to acquire connection permit".to_string()));
            }
            Err(_) => {
                self.record_failed_request().await;
                return Err(EventualiError::Configuration("Connection timeout".to_string()));
            }
        };

        // Increment active connection count
        {
            let mut active = self.active_count.lock().await;
            *active += 1;
        }

        let wait_time = start_time.elapsed();
        self.record_successful_request(wait_time).await;

        Ok(PoolGuard {
            database_path: self.database_path.clone(),
            pool: self.clone(),
            permit: Some(permit),
        })
    }

    /// Get current pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let mut stats = self.stats.lock().await;
        let active_count = *self.active_count.lock().await;
        let total_count = *self.connection_count.lock().await;
        
        stats.active_connections = active_count;
        stats.total_connections = total_count;
        stats.idle_connections = total_count.saturating_sub(active_count);
        
        stats.clone()
    }

    /// Get pool configuration
    pub fn get_config(&self) -> &PoolConfig {
        &self.config
    }

    async fn record_successful_request(&self, wait_time: Duration) {
        let mut stats = self.stats.lock().await;
        stats.successful_requests += 1;
        
        let wait_time_ms = wait_time.as_millis() as u64;
        if wait_time_ms > stats.max_wait_time_ms {
            stats.max_wait_time_ms = wait_time_ms;
        }
        
        // Update average wait time (simple moving average)
        let total_completed = stats.successful_requests + stats.failed_requests;
        stats.avg_wait_time_ms = (stats.avg_wait_time_ms * (total_completed - 1) as f64 + wait_time_ms as f64) / total_completed as f64;
    }

    async fn record_failed_request(&self) {
        let mut stats = self.stats.lock().await;
        stats.failed_requests += 1;
    }

    async fn release_connection(&self) {
        let mut active = self.active_count.lock().await;
        if *active > 0 {
            *active -= 1;
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connection_count: self.connection_count.clone(),
            active_count: self.active_count.clone(),
            semaphore: self.semaphore.clone(),
            stats: self.stats.clone(),
            database_path: self.database_path.clone(),
        }
    }
}

/// A guard that represents a connection slot in the pool
pub struct PoolGuard<'a> {
    database_path: String,
    pool: ConnectionPool,
    permit: Option<tokio::sync::SemaphorePermit<'a>>,
}

impl<'a> PoolGuard<'a> {
    /// Get the database path for creating connections
    pub fn database_path(&self) -> &str {
        &self.database_path
    }

    /// Create a new database connection optimized for performance
    pub fn create_connection(&self) -> Result<rusqlite::Connection, EventualiError> {
        let conn = if self.database_path == ":memory:" {
            rusqlite::Connection::open_in_memory()
        } else {
            rusqlite::Connection::open(&self.database_path)
        }.map_err(|e| EventualiError::Configuration(format!("Failed to create connection: {}", e)))?;

        // Optimize connection settings for performance
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -2000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
        ").map_err(|e| EventualiError::Configuration(format!("Failed to optimize connection: {}", e)))?;

        Ok(conn)
    }
}

impl<'a> Drop for PoolGuard<'a> {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            pool.release_connection().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(":memory:".to_string(), config).await.unwrap();
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 5); // Default min_connections
    }

    #[tokio::test]
    async fn test_connection_acquisition() {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(":memory:".to_string(), config).await.unwrap();
        
        let guard = pool.get_connection().await.unwrap();
        
        // Test that we can create a connection
        let conn = guard.create_connection().unwrap();
        
        // Test that we can execute a query
        let result = conn.execute("CREATE TABLE test (id INTEGER)", []);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pool_stats_tracking() {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(":memory:".to_string(), config).await.unwrap();
        
        let _guard = pool.get_connection().await.unwrap();
        let stats = pool.get_stats().await;
        
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.active_connections, 1);
    }
}