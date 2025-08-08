//! Performance optimization modules for Eventuali
//!
//! This module provides high-performance optimizations for database operations,
//! including connection pooling, WAL optimization, batch processing, read replicas,
//! caching layers, and advanced compression.

pub mod connection_pool;
pub mod wal_optimization;
pub mod batch_processing_stub;
pub mod read_replicas;
pub mod caching;
pub mod compression;

pub use connection_pool::*;
pub use wal_optimization::*;
pub use batch_processing_stub::{BatchConfig, BatchStats, BatchProcessor, EventBatchProcessor};
pub use read_replicas::*;
pub use caching::*;
pub use compression::*;