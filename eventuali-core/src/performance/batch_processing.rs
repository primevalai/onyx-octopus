//! High-throughput batch processing optimization
//!
//! Provides intelligent batch operations with backpressure control, worker pools,
//! and transaction management for optimal throughput in enterprise environments.

use crate::error::EventualiError;
use crate::event::Event;
use crate::performance::connection_pool::ConnectionPool;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar};
use tokio::sync::Mutex;
use std::time::{Instant, Duration};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::sleep;
use std::future::Future;
use std::pin::Pin;

/// Batch processing configuration with intelligent sizing and flow control
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of items in a batch
    pub max_batch_size: usize,
    /// Minimum number of items before triggering batch processing
    pub min_batch_size: usize,
    /// Maximum time to wait before processing partial batch
    pub max_wait_ms: u64,
    /// Target processing time per batch
    pub target_batch_time_ms: u64,
    /// Number of worker threads for parallel processing
    pub worker_pool_size: usize,
    /// Maximum number of pending batches
    pub max_pending_batches: usize,
    /// Backpressure threshold (0.0 to 1.0)
    pub backpressure_threshold: f64,
    /// Enable adaptive batch sizing
    pub adaptive_sizing: bool,
    /// Maximum memory usage for buffering (bytes)
    pub max_buffer_memory_mb: usize,
    /// Transaction batch size for rollback management
    pub transaction_batch_size: usize,
    /// Enable parallel worker processing
    pub parallel_processing: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            min_batch_size: 100,
            max_wait_ms: 100,
            target_batch_time_ms: 50,
            worker_pool_size: 4,
            max_pending_batches: 10,
            backpressure_threshold: 0.8,
            adaptive_sizing: true,
            max_buffer_memory_mb: 64,
            transaction_batch_size: 500,
            parallel_processing: true,
        }
    }
}

impl BatchConfig {
    /// High-performance configuration for maximum throughput
    pub fn high_throughput() -> Self {
        Self {
            max_batch_size: 2000,
            min_batch_size: 200,
            max_wait_ms: 50,
            target_batch_time_ms: 25,
            worker_pool_size: 8,
            max_pending_batches: 20,
            backpressure_threshold: 0.9,
            adaptive_sizing: true,
            max_buffer_memory_mb: 128,
            transaction_batch_size: 1000,
            parallel_processing: true,
        }
    }
    
    /// Memory-optimized configuration for resource-constrained environments
    pub fn memory_optimized() -> Self {
        Self {
            max_batch_size: 500,
            min_batch_size: 50,
            max_wait_ms: 200,
            target_batch_time_ms: 100,
            worker_pool_size: 2,
            max_pending_batches: 5,
            backpressure_threshold: 0.7,
            adaptive_sizing: true,
            max_buffer_memory_mb: 32,
            transaction_batch_size: 250,
            parallel_processing: false,
        }
    }
    
    /// Low-latency configuration for real-time processing
    pub fn low_latency() -> Self {
        Self {
            max_batch_size: 200,
            min_batch_size: 10,
            max_wait_ms: 10,
            target_batch_time_ms: 5,
            worker_pool_size: 6,
            max_pending_batches: 15,
            backpressure_threshold: 0.6,
            adaptive_sizing: true,
            max_buffer_memory_mb: 16,
            transaction_batch_size: 100,
            parallel_processing: true,
        }
    }
}

/// Batch processing statistics and monitoring
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub total_items_processed: u64,
    pub total_batches_processed: u64,
    pub successful_batches: u64,
    pub failed_batches: u64,
    pub avg_batch_size: f64,
    pub avg_processing_time_ms: f64,
    pub current_throughput_per_sec: f64,
    pub peak_throughput_per_sec: f64,
    pub current_queue_depth: usize,
    pub max_queue_depth: usize,
    pub backpressure_events: u64,
    pub adaptive_size_adjustments: u64,
    pub memory_usage_mb: f64,
    pub success_rate: f64,
}

/// Represents a batch of items to be processed
#[derive(Debug)]
pub struct Batch<T> {
    pub items: Vec<T>,
    pub created_at: Instant,
    pub batch_id: u64,
    pub priority: BatchPriority,
}

/// Priority levels for batch processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BatchPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for BatchPriority {
    fn default() -> Self {
        BatchPriority::Normal
    }
}

/// Batch processing result with detailed metrics
#[derive(Debug)]
pub struct BatchResult {
    pub batch_id: u64,
    pub items_processed: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub processing_time_ms: u64,
    pub throughput_per_sec: f64,
    pub errors: Vec<String>,
}

/// Worker pool for parallel batch processing
struct WorkerPool<T> {
    workers: Vec<tokio::task::JoinHandle<()>>,
    task_sender: mpsc::UnboundedSender<BatchTask<T>>,
    semaphore: Arc<Semaphore>,
}

impl<T> Clone for WorkerPool<T> {
    fn clone(&self) -> Self {
        // Note: We can't clone JoinHandles, so we create an empty vec
        // This is a simplified implementation for demo purposes
        Self {
            workers: Vec::new(),
            task_sender: self.task_sender.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}

/// Task sent to worker pool
struct BatchTask<T> {
    batch: Batch<T>,
    processor: Arc<dyn BatchItemProcessor<T> + Send + Sync>,
    result_sender: mpsc::UnboundedSender<BatchResult>,
}

/// Trait for processing individual items in a batch
pub trait BatchItemProcessor<T: Send>: Send + Sync {
    fn process_item(&self, item: T) -> Pin<Box<dyn Future<Output = Result<(), EventualiError>> + Send + '_>>;
    fn process_batch<'a>(&'a self, items: Vec<T>) -> Pin<Box<dyn Future<Output = Result<Vec<Result<(), EventualiError>>, EventualiError>> + Send + 'a>> where T: 'a {
        Box::pin(async move {
            let mut results = Vec::with_capacity(items.len());
            for item in items {
                results.push(self.process_item(item).await);
            }
            Ok(results)
        })
    }
}

/// High-performance batch processor with intelligent batching and flow control
#[derive(Clone)]
pub struct BatchProcessor<T> {
    config: BatchConfig,
    buffer: Arc<Mutex<VecDeque<T>>>,
    worker_pool: Option<WorkerPool<T>>,
    connection_pool: Option<Arc<ConnectionPool>>,
    stats: Arc<Mutex<BatchStats>>,
    batch_counter: Arc<Mutex<u64>>,
    running: Arc<Mutex<bool>>,
    buffer_condvar: Arc<Condvar>,
    processor: Option<Arc<dyn BatchItemProcessor<T> + Send + Sync>>,
    adaptive_size: Arc<Mutex<usize>>,
}

impl<T: Send + 'static> BatchProcessor<T> {
    /// Create a new batch processor
    pub fn new(config: BatchConfig) -> Self {
        let adaptive_size = Arc::new(Mutex::new(config.max_batch_size));
        
        Self {
            config,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            worker_pool: None,
            connection_pool: None,
            stats: Arc::new(Mutex::new(BatchStats::default())),
            batch_counter: Arc::new(Mutex::new(0)),
            running: Arc::new(Mutex::new(false)),
            buffer_condvar: Arc::new(Condvar::new()),
            processor: None,
            adaptive_size,
        }
    }
    
    /// Set the connection pool for database operations
    pub fn with_connection_pool(mut self, pool: Arc<ConnectionPool>) -> Self {
        self.connection_pool = Some(pool);
        self
    }
    
    /// Set the item processor
    pub fn with_processor(mut self, processor: Arc<dyn BatchItemProcessor<T> + Send + Sync>) -> Self {
        self.processor = Some(processor);
        self
    }
    
    /// Start the batch processor with worker pool
    pub async fn start(&mut self) -> Result<(), EventualiError> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(EventualiError::InvalidState("Batch processor already running".to_string()));
        }
        *running = true;
        drop(running);
        
        if self.config.parallel_processing {
            self.start_worker_pool().await?;
        }
        
        // Start batch collection and processing loop
        self.start_processing_loop().await;
        
        Ok(())
    }
    
    /// Stop the batch processor
    pub async fn stop(&mut self) -> Result<(), EventualiError> {
        let mut running = self.running.lock().await;
        if !*running {
            return Ok(());
        }
        *running = false;
        self.buffer_condvar.notify_all();
        drop(running);
        
        // Stop worker pool
        if let Some(worker_pool) = self.worker_pool.take() {
            self.stop_worker_pool(worker_pool).await;
        }
        
        Ok(())
    }
    
    /// Add an item to the batch processor
    pub async fn add_item(&self, item: T) -> Result<(), EventualiError> {
        self.add_item_with_priority(item, BatchPriority::Normal).await
    }
    
    /// Add an item with specific priority
    pub async fn add_item_with_priority(&self, item: T, _priority: BatchPriority) -> Result<(), EventualiError> {
        let running = *self.running.lock().await;
        if !running {
            return Err(EventualiError::InvalidState("Batch processor not running".to_string()));
        }
        
        // Check backpressure
        if self.should_apply_backpressure().await {
            return Err(EventualiError::BackpressureApplied("Batch processor overloaded".to_string()));
        }
        
        // Add to buffer
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push_back(item);
            
            // Update stats
            let mut stats = self.stats.lock().await;
            stats.current_queue_depth = buffer.len();
            if buffer.len() > stats.max_queue_depth {
                stats.max_queue_depth = buffer.len();
            }
        }
        
        self.buffer_condvar.notify_one();
        Ok(())
    }
    
    /// Get current batch processing statistics
    pub fn get_stats(&self) -> BatchStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Force process any pending items in the buffer
    pub async fn flush(&self) -> Result<(), EventualiError> {
        let items = {
            let mut buffer = self.buffer.lock().unwrap();
            let items: Vec<T> = buffer.drain(..).collect();
            items
        };
        
        if !items.is_empty() {
            self.process_batch_items(items, BatchPriority::Normal).await?;
        }
        
        Ok(())
    }
    
    /// Check if backpressure should be applied
    async fn should_apply_backpressure(&self) -> bool {
        let stats = self.stats.lock().unwrap();
        let queue_utilization = stats.current_queue_depth as f64 / self.config.max_pending_batches as f64;
        queue_utilization > self.config.backpressure_threshold
    }
    
    /// Start the worker pool for parallel processing
    async fn start_worker_pool(&mut self) -> Result<(), EventualiError> {
        let (task_sender, mut task_receiver) = mpsc::unbounded_channel::<BatchTask<T>>();
        let semaphore = Arc::new(Semaphore::new(self.config.worker_pool_size));
        let mut workers = Vec::new();
        
        // For simplicity, create one worker that processes tasks sequentially
        // In a production implementation, you would use a proper work-stealing queue
        let stats_clone = self.stats.clone();
        let semaphore_clone = semaphore.clone();
        
        let worker = tokio::spawn(async move {
            while let Some(task) = task_receiver.recv().await {
                let _permit = semaphore_clone.acquire().await.unwrap();
                
                let start_time = Instant::now();
                let batch_size = task.batch.items.len();
                
                // Process the batch
                let results = task.processor.process_batch(task.batch.items).await;
                let processing_time = start_time.elapsed();
                
                // Calculate results
                let (successful_items, failed_items, errors) = match results {
                    Ok(item_results) => {
                        let successful = item_results.iter().filter(|r| r.is_ok()).count();
                        let failed = item_results.len() - successful;
                        let errors: Vec<String> = item_results.iter()
                            .filter_map(|r| r.as_ref().err())
                            .map(|e| e.to_string())
                            .collect();
                        (successful, failed, errors)
                    }
                    Err(e) => (0, batch_size, vec![e.to_string()])
                };
                
                let batch_result = BatchResult {
                    batch_id: task.batch.batch_id,
                    items_processed: batch_size,
                    successful_items,
                    failed_items,
                    processing_time_ms: processing_time.as_millis() as u64,
                    throughput_per_sec: successful_items as f64 / processing_time.as_secs_f64(),
                    errors,
                };
                
                // Update stats
                {
                    let mut stats = stats_clone.lock().unwrap();
                    stats.total_batches_processed += 1;
                    stats.total_items_processed += successful_items as u64;
                    
                    if successful_items > 0 {
                        stats.successful_batches += 1;
                    } else {
                        stats.failed_batches += 1;
                    }
                    
                    // Update averages
                    let total_batches = stats.total_batches_processed;
                    stats.avg_batch_size = (stats.avg_batch_size * (total_batches - 1) as f64 + batch_size as f64) / total_batches as f64;
                    stats.avg_processing_time_ms = (stats.avg_processing_time_ms * (total_batches - 1) as f64 + processing_time.as_millis() as f64) / total_batches as f64;
                    
                    // Update throughput
                    stats.current_throughput_per_sec = batch_result.throughput_per_sec;
                    if batch_result.throughput_per_sec > stats.peak_throughput_per_sec {
                        stats.peak_throughput_per_sec = batch_result.throughput_per_sec;
                    }
                    
                    // Update success rate
                    stats.success_rate = stats.successful_batches as f64 / total_batches as f64;
                }
                
                // Send result
                let _ = task.result_sender.send(batch_result);
            }
        });
        
        workers.push(worker);
        
        self.worker_pool = Some(WorkerPool {
            workers,
            task_sender,
            semaphore,
        });
        
        Ok(())
    }
    
    /// Stop the worker pool
    async fn stop_worker_pool(&self, worker_pool: WorkerPool<T>) {
        // Drop the task sender to signal workers to stop
        drop(worker_pool.task_sender);
        
        // Wait for all workers to finish
        for worker in worker_pool.workers {
            let _ = worker.await;
        }
    }
    
    /// Start the main processing loop
    async fn start_processing_loop(&self) {
        let buffer_clone = self.buffer.clone();
        let stats_clone = self.stats.clone();
        let running_clone = self.running.clone();
        let condvar_clone = self.buffer_condvar.clone();
        let adaptive_size_clone = self.adaptive_size.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut last_adaptive_adjustment = Instant::now();
            
            loop {
                let running = *running_clone.lock().unwrap();
                if !running {
                    break;
                }
                
                // Wait for items or timeout
                let items = {
                    let buffer_guard = buffer_clone.lock().unwrap();
                    let mut buffer_guard = condvar_clone.wait_timeout(buffer_guard, Duration::from_millis(config.max_wait_ms)).unwrap().0;
                    
                    let buffer_size = buffer_guard.len();
                    let current_batch_size = *adaptive_size_clone.lock().unwrap();
                    
                    if buffer_size >= config.min_batch_size || 
                       (buffer_size > 0 && buffer_guard.front().map_or(false, |_| {
                           // Check if oldest item has been waiting too long
                           true // Simplified - in practice, you'd track item timestamps
                       })) {
                        let take_count = std::cmp::min(buffer_size, current_batch_size);
                        let items: Vec<T> = buffer_guard.drain(..take_count).collect();
                        items
                    } else {
                        Vec::new()
                    }
                };
                
                if !items.is_empty() {
                    // Process the batch (this would send to worker pool or process directly)
                    // Implementation depends on whether parallel processing is enabled
                }
                
                // Adaptive sizing adjustment
                if config.adaptive_sizing && last_adaptive_adjustment.elapsed() > Duration::from_secs(5) {
                    // Adjust batch size based on recent performance
                    let stats = stats_clone.lock().unwrap();
                    let avg_time = stats.avg_processing_time_ms;
                    let target_time = config.target_batch_time_ms as f64;
                    
                    if avg_time > target_time * 1.2 {
                        // Processing too slow, reduce batch size
                        let mut adaptive_size = adaptive_size_clone.lock().unwrap();
                        *adaptive_size = std::cmp::max(config.min_batch_size, (*adaptive_size as f64 * 0.9) as usize);
                    } else if avg_time < target_time * 0.8 {
                        // Processing too fast, increase batch size
                        let mut adaptive_size = adaptive_size_clone.lock().unwrap();
                        *adaptive_size = std::cmp::min(config.max_batch_size, (*adaptive_size as f64 * 1.1) as usize);
                    }
                    
                    last_adaptive_adjustment = Instant::now();
                }
                
                // Brief pause to prevent busy waiting
                sleep(Duration::from_millis(1)).await;
            }
        });
    }
    
    /// Process a batch of items
    async fn process_batch_items(&self, items: Vec<T>, priority: BatchPriority) -> Result<(), EventualiError> {
        if items.is_empty() {
            return Ok(());
        }
        
        let batch_id = {
            let mut counter = self.batch_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        let batch = Batch {
            items,
            created_at: Instant::now(),
            batch_id,
            priority,
        };
        
        if let Some(ref processor) = self.processor {
            if let Some(ref worker_pool) = self.worker_pool {
                // Send to worker pool for parallel processing
                let (result_sender, _result_receiver) = mpsc::unbounded_channel();
                
                let task = BatchTask {
                    batch,
                    processor: processor.clone(),
                    result_sender,
                };
                
                worker_pool.task_sender.send(task).map_err(|_| 
                    EventualiError::InvalidState("Worker pool unavailable".to_string()))?;
            } else {
                // Process directly
                let start_time = Instant::now();
                let results = processor.process_batch(batch.items).await?;
                let processing_time = start_time.elapsed();
                
                // Update stats
                let mut stats = self.stats.lock().unwrap();
                stats.total_batches_processed += 1;
                
                let successful_items = results.iter().filter(|r| r.is_ok()).count();
                if successful_items > 0 {
                    stats.successful_batches += 1;
                    stats.total_items_processed += successful_items as u64;
                }
                
                stats.avg_processing_time_ms = processing_time.as_millis() as f64;
                stats.current_throughput_per_sec = successful_items as f64 / processing_time.as_secs_f64();
            }
        }
        
        Ok(())
    }
}

/// Event batch processor for database operations
pub struct EventBatchProcessor {
    connection_pool: Arc<ConnectionPool>,
}

impl EventBatchProcessor {
    pub fn new(connection_pool: Arc<ConnectionPool>) -> Self {
        Self { connection_pool }
    }
}

impl BatchItemProcessor<Event> for EventBatchProcessor {
    fn process_item(&self, event: Event) -> Pin<Box<dyn Future<Output = Result<(), EventualiError>> + Send + '_>> {
        Box::pin(async move {
            let guard = self.connection_pool.get_connection().await?;
            let conn = guard.create_connection()?;
            
            // Serialize event data
            let event_data_str = match &event.data {
                crate::event::EventData::Json(value) => value.to_string(),
                crate::event::EventData::Protobuf(bytes) => {
                    use base64::{Engine, engine::general_purpose};
                    general_purpose::STANDARD.encode(bytes)
                },
            };
            
            // Process single event
            let sql = "INSERT INTO events (aggregate_id, event_type, event_data, version, created_at) VALUES (?, ?, ?, ?, ?)";
            conn.execute(sql, [
                event.aggregate_id.as_str(),
                event.event_type.as_str(), 
                &event_data_str,
                &event.aggregate_version.to_string(),
                &event.timestamp.to_rfc3339(),
            ]).map_err(|e| EventualiError::DatabaseError(e.to_string()))?;
            
            Ok(())
        })
    }
    
    fn process_batch<'a>(&'a self, events: Vec<Event>) -> Pin<Box<dyn Future<Output = Result<Vec<Result<(), EventualiError>>, EventualiError>> + Send + 'a>> where Event: 'a {
        Box::pin(async move {
            let guard = self.connection_pool.get_connection().await?;
            let conn = guard.create_connection()?;
            
            // Begin transaction for batch
            conn.execute("BEGIN TRANSACTION", []).map_err(|e| EventualiError::DatabaseError(e.to_string()))?;
            
            let mut results = Vec::with_capacity(events.len());
            
            // Process all events in transaction
            for event in events {
                // Serialize event data
                let event_data_str = match &event.data {
                    crate::event::EventData::Json(value) => value.to_string(),
                    crate::event::EventData::Protobuf(bytes) => {
                        use base64::{Engine, engine::general_purpose};
                        general_purpose::STANDARD.encode(bytes)
                    },
                };
                
                let result = {
                    let sql = "INSERT INTO events (aggregate_id, event_type, event_data, version, created_at) VALUES (?, ?, ?, ?, ?)";
                    conn.execute(sql, [
                        event.aggregate_id.as_str(),
                        event.event_type.as_str(), 
                        &event_data_str,
                        &event.aggregate_version.to_string(),
                        &event.timestamp.to_rfc3339(),
                    ]).map_err(|e| EventualiError::DatabaseError(e.to_string())).map(|_| ())
                };
                
                if result.is_err() {
                    // Rollback on first error
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(EventualiError::BatchProcessingError("Batch failed, transaction rolled back".to_string()));
                }
                
                results.push(result);
            }
            
            // Commit transaction
            conn.execute("COMMIT", []).map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                EventualiError::DatabaseError(format!("Commit failed: {}", e))
            })?;
            
            Ok(results)
        })
    }
}