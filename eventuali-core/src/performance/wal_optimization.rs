//! Write-ahead logging performance optimization
//!
//! Provides WAL configuration and tuning for optimal write performance.
//! WAL (Write-Ahead Logging) is crucial for database performance and ACID compliance.

use std::sync::Arc;
use std::time::{Duration, Instant};
use rusqlite::Connection as SqliteConnection;
use crate::error::EventualiError;

/// WAL optimization configuration
#[derive(Debug, Clone)]
pub struct WalConfig {
    pub synchronous_mode: WalSynchronousMode,
    pub journal_mode: WalJournalMode,
    pub checkpoint_interval: u64,
    pub checkpoint_size_mb: u64,
    pub wal_autocheckpoint: u32,
    pub cache_size_kb: i32,
    pub temp_store: TempStoreMode,
    pub mmap_size_mb: u64,
    pub page_size: u32,
    pub auto_vacuum: AutoVacuumMode,
}

/// SQLite synchronous modes for different performance/durability tradeoffs
#[derive(Debug, Clone)]
pub enum WalSynchronousMode {
    Off,      // Fastest, no safety guarantees
    Normal,   // Good performance with some safety
    Full,     // Full safety, slower performance  
    Extra,    // Extra safety, slowest performance
}

/// Journal modes for different use cases
#[derive(Debug, Clone)]
pub enum WalJournalMode {
    Delete,   // Traditional rollback journal
    Truncate, // Truncate rollback journal
    Persist,  // Persistent rollback journal
    Memory,   // In-memory journal
    Wal,      // Write-Ahead Logging (recommended)
    Off,      // No journal (dangerous)
}

/// Temporary storage modes
#[derive(Debug, Clone)]
pub enum TempStoreMode {
    Default, // Use compile-time default
    File,    // Store temp tables/indices in files
    Memory,  // Store temp tables/indices in memory
}

/// Auto-vacuum modes
#[derive(Debug, Clone)]
pub enum AutoVacuumMode {
    None,        // No auto-vacuum
    Full,        // Full auto-vacuum on every commit
    Incremental, // Incremental auto-vacuum
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            synchronous_mode: WalSynchronousMode::Normal,
            journal_mode: WalJournalMode::Wal,
            checkpoint_interval: 1000,
            checkpoint_size_mb: 100,
            wal_autocheckpoint: 1000,
            cache_size_kb: -2000,  // 2MB cache
            temp_store: TempStoreMode::Memory,
            mmap_size_mb: 256,
            page_size: 4096,
            auto_vacuum: AutoVacuumMode::Incremental,
        }
    }
}

impl WalConfig {
    /// High-performance configuration for write-heavy workloads
    pub fn high_performance() -> Self {
        Self {
            synchronous_mode: WalSynchronousMode::Normal,
            journal_mode: WalJournalMode::Wal,
            checkpoint_interval: 2000,
            checkpoint_size_mb: 200,
            wal_autocheckpoint: 2000,
            cache_size_kb: -8000,  // 8MB cache
            temp_store: TempStoreMode::Memory,
            mmap_size_mb: 1024,    // 1GB mmap
            page_size: 4096,
            auto_vacuum: AutoVacuumMode::Incremental,
        }
    }

    /// Memory-optimized configuration for resource-constrained environments
    pub fn memory_optimized() -> Self {
        Self {
            synchronous_mode: WalSynchronousMode::Normal,
            journal_mode: WalJournalMode::Wal,
            checkpoint_interval: 500,
            checkpoint_size_mb: 50,
            wal_autocheckpoint: 500,
            cache_size_kb: -1000,  // 1MB cache
            temp_store: TempStoreMode::Memory,
            mmap_size_mb: 64,      // 64MB mmap
            page_size: 4096,
            auto_vacuum: AutoVacuumMode::Incremental,
        }
    }

    /// Safety-first configuration for critical data
    pub fn safety_first() -> Self {
        Self {
            synchronous_mode: WalSynchronousMode::Full,
            journal_mode: WalJournalMode::Wal,
            checkpoint_interval: 100,
            checkpoint_size_mb: 20,
            wal_autocheckpoint: 100,
            cache_size_kb: -2000,  // 2MB cache
            temp_store: TempStoreMode::File,
            mmap_size_mb: 128,
            page_size: 4096,
            auto_vacuum: AutoVacuumMode::Full,
        }
    }
}

/// WAL performance statistics
#[derive(Debug, Clone)]
pub struct WalStats {
    pub total_checkpoints: u64,
    pub avg_checkpoint_time_ms: f64,
    pub wal_file_size_kb: u64,
    pub pages_written: u64,
    pub pages_read: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

impl Default for WalStats {
    fn default() -> Self {
        Self {
            total_checkpoints: 0,
            avg_checkpoint_time_ms: 0.0,
            wal_file_size_kb: 0,
            pages_written: 0,
            pages_read: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_hit_rate: 0.0,
        }
    }
}

/// WAL optimizer for database performance tuning
pub struct WalOptimizer {
    config: WalConfig,
    stats: Arc<std::sync::Mutex<WalStats>>,
    last_checkpoint: Option<Instant>,
}

impl WalOptimizer {
    /// Create a new WAL optimizer with the specified configuration
    pub fn new(config: WalConfig) -> Self {
        Self {
            config,
            stats: Arc::new(std::sync::Mutex::new(WalStats::default())),
            last_checkpoint: None,
        }
    }

    /// Apply WAL optimizations to a database connection
    pub fn optimize_connection(&self, conn: &SqliteConnection) -> Result<(), EventualiError> {
        // Set journal mode
        let journal_mode = self.journal_mode_to_string(&self.config.journal_mode);
        conn.execute(&format!("PRAGMA journal_mode = {journal_mode}"), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set journal mode: {e}")))?;

        // Set synchronous mode
        let sync_mode = self.sync_mode_to_string(&self.config.synchronous_mode);
        conn.execute(&format!("PRAGMA synchronous = {sync_mode}"), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set synchronous mode: {e}")))?;

        // Set cache size
        conn.execute(&format!("PRAGMA cache_size = {}", self.config.cache_size_kb), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set cache size: {e}")))?;

        // Set temp store mode
        let temp_store = self.temp_store_to_string(&self.config.temp_store);
        conn.execute(&format!("PRAGMA temp_store = {temp_store}"), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set temp store: {e}")))?;

        // Set memory mapping size
        conn.execute(&format!("PRAGMA mmap_size = {}", self.config.mmap_size_mb * 1024 * 1024), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set mmap size: {e}")))?;

        // Set page size (only effective on new databases)
        conn.execute(&format!("PRAGMA page_size = {}", self.config.page_size), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set page size: {e}")))?;

        // Set auto-vacuum mode
        let auto_vacuum = self.auto_vacuum_to_string(&self.config.auto_vacuum);
        conn.execute(&format!("PRAGMA auto_vacuum = {auto_vacuum}"), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set auto vacuum: {e}")))?;

        // Set WAL auto-checkpoint
        conn.execute(&format!("PRAGMA wal_autocheckpoint = {}", self.config.wal_autocheckpoint), [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to set WAL autocheckpoint: {e}")))?;

        Ok(())
    }

    /// Perform a manual WAL checkpoint
    pub fn checkpoint(&mut self, conn: &SqliteConnection) -> Result<(), EventualiError> {
        let start_time = Instant::now();
        
        conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])
            .map_err(|e| EventualiError::Configuration(format!("Failed to checkpoint WAL: {e}")))?;

        let checkpoint_time = start_time.elapsed();
        self.last_checkpoint = Some(Instant::now());

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_checkpoints += 1;
            let total_checkpoints = stats.total_checkpoints as f64;
            stats.avg_checkpoint_time_ms = (stats.avg_checkpoint_time_ms * (total_checkpoints - 1.0) + 
                                           checkpoint_time.as_millis() as f64) / total_checkpoints;
        }

        Ok(())
    }

    /// Get current WAL statistics
    pub fn get_stats(&self, conn: &SqliteConnection) -> Result<WalStats, EventualiError> {
        let mut stats = if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            WalStats::default()
        };

        // Query database for additional stats
        if let Ok(_stmt) = conn.prepare("PRAGMA wal_checkpoint") {
            // This would get WAL file size and other metrics in a real implementation
        }

        // Calculate cache hit rate
        if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hit_rate = stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64;
        }

        Ok(stats)
    }

    /// Check if a checkpoint is needed based on configuration
    pub fn needs_checkpoint(&self) -> bool {
        if let Some(last) = self.last_checkpoint {
            last.elapsed() > Duration::from_millis(self.config.checkpoint_interval)
        } else {
            true
        }
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &WalConfig {
        &self.config
    }

    // Helper methods to convert enums to strings
    fn journal_mode_to_string(&self, mode: &WalJournalMode) -> &'static str {
        match mode {
            WalJournalMode::Delete => "DELETE",
            WalJournalMode::Truncate => "TRUNCATE",
            WalJournalMode::Persist => "PERSIST",
            WalJournalMode::Memory => "MEMORY",
            WalJournalMode::Wal => "WAL",
            WalJournalMode::Off => "OFF",
        }
    }

    fn sync_mode_to_string(&self, mode: &WalSynchronousMode) -> &'static str {
        match mode {
            WalSynchronousMode::Off => "OFF",
            WalSynchronousMode::Normal => "NORMAL",
            WalSynchronousMode::Full => "FULL",
            WalSynchronousMode::Extra => "EXTRA",
        }
    }

    fn temp_store_to_string(&self, mode: &TempStoreMode) -> &'static str {
        match mode {
            TempStoreMode::Default => "DEFAULT",
            TempStoreMode::File => "FILE",
            TempStoreMode::Memory => "MEMORY",
        }
    }

    fn auto_vacuum_to_string(&self, mode: &AutoVacuumMode) -> &'static str {
        match mode {
            AutoVacuumMode::None => "NONE",
            AutoVacuumMode::Full => "FULL",
            AutoVacuumMode::Incremental => "INCREMENTAL",
        }
    }
}

/// Benchmark different WAL configurations
pub async fn benchmark_wal_configurations(
    database_path: String,
    configs: Vec<(String, WalConfig)>,
    num_operations: usize,
) -> Result<Vec<(String, f64, WalStats)>, EventualiError> {
    use std::time::Instant;
    
    let mut results = Vec::new();
    
    for (config_name, config) in configs {
        let start_time = Instant::now();
        
        // Create connection and optimizer
        let conn = if database_path == ":memory:" {
            SqliteConnection::open_in_memory()
        } else {
            SqliteConnection::open(&database_path)
        }.map_err(|e| EventualiError::Configuration(format!("Failed to open database: {e}")))?;

        let mut optimizer = WalOptimizer::new(config.clone());
        optimizer.optimize_connection(&conn)?;

        // Create test table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_events (
                id INTEGER PRIMARY KEY,
                data TEXT,
                timestamp INTEGER
            )", []
        ).map_err(|e| EventualiError::Configuration(format!("Failed to create test table: {e}")))?;

        // Perform test operations
        for i in 0..num_operations {
            conn.execute(
                "INSERT INTO test_events (data, timestamp) VALUES (?, ?)",
                [&format!("test_data_{i}"), &format!("{i}")]
            ).map_err(|e| EventualiError::Configuration(format!("Failed to insert test data: {e}")))?;

            // Checkpoint periodically
            if i.is_multiple_of(100) && optimizer.needs_checkpoint() {
                optimizer.checkpoint(&conn)?;
            }
        }

        // Final checkpoint
        optimizer.checkpoint(&conn)?;
        
        let total_time = start_time.elapsed();
        let ops_per_sec = num_operations as f64 / total_time.as_secs_f64();
        let stats = optimizer.get_stats(&conn)?;
        
        results.push((config_name, ops_per_sec, stats));
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_config_creation() {
        let config = WalConfig::default();
        assert!(matches!(config.synchronous_mode, WalSynchronousMode::Normal));
        assert!(matches!(config.journal_mode, WalJournalMode::Wal));
    }

    #[test]
    fn test_wal_optimizer_creation() {
        let config = WalConfig::high_performance();
        let optimizer = WalOptimizer::new(config);
        assert_eq!(optimizer.config.cache_size_kb, -8000);
    }

    #[test]
    fn test_connection_optimization() {
        let config = WalConfig::default();
        let optimizer = WalOptimizer::new(config);
        
        let conn = SqliteConnection::open_in_memory().unwrap();
        let result = optimizer.optimize_connection(&conn);
        assert!(result.is_ok());
    }
}