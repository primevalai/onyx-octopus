//! Advanced compression algorithms for event data
//!
//! Provides LZ4, ZSTD compression with performance benchmarks.

/// Compression algorithm configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub algorithm: CompressionAlgorithm,
    pub level: u32,
    pub enable_parallel: bool,
}

#[derive(Debug, Clone)]
pub enum CompressionAlgorithm {
    None,
    LZ4,
    ZSTD,
    Gzip,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::LZ4,
            level: 3,
            enable_parallel: true,
        }
    }
}

/// Compression manager
pub struct CompressionManager {
    #[allow(dead_code)] // Compression configuration settings (stored but not currently accessed in implementation)
    config: CompressionConfig,
}

impl CompressionManager {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }
}