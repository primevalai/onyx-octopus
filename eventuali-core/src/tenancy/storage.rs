use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::event::{Event, EventData, EventMetadata};
use crate::aggregate::{AggregateId, AggregateVersion};
use crate::store::{EventStore, EventStoreBackend};
use crate::error::{EventualiError, Result};
use super::tenant::TenantId;
use super::isolation::{TenantIsolation, TenantOperation};
use super::quota::{TenantQuota, ResourceType};

/// Tenant-aware event storage that ensures complete isolation between tenants
/// while providing high-performance event operations
pub struct TenantAwareEventStorage {
    tenant_id: TenantId,
    backend: Arc<dyn EventStoreBackend + Send + Sync>,
    isolation: Arc<TenantIsolation>,
    quota: Arc<TenantQuota>,
    metrics: Arc<RwLock<TenantStorageMetrics>>,
}

impl TenantAwareEventStorage {
    pub fn new(
        tenant_id: TenantId,
        backend: Arc<dyn EventStoreBackend + Send + Sync>,
        isolation: Arc<TenantIsolation>,
        quota: Arc<TenantQuota>,
    ) -> Self {
        Self {
            tenant_id,
            backend,
            isolation,
            quota,
            metrics: Arc::new(RwLock::new(TenantStorageMetrics::new())),
        }
    }
    
    /// Transform event to include tenant namespace
    fn tenant_scoped_event(&self, mut event: Event) -> Event {
        // Add tenant namespace to aggregate ID
        event.aggregate_id = format!("{}:{}", self.tenant_id.db_prefix(), event.aggregate_id);
        
        // Add tenant context to metadata headers
        event.metadata.headers.insert(
            "tenant_id".to_string(),
            self.tenant_id.as_str().to_string()
        );
        event.metadata.headers.insert(
            "tenant_namespace".to_string(),
            self.tenant_id.db_prefix()
        );
        
        event
    }
    
    /// Remove tenant namespace from event for external consumption
    fn unscoped_event(&self, mut event: Event) -> Event {
        // Remove tenant prefix from aggregate ID
        let prefix = format!("{}:", self.tenant_id.db_prefix());
        if event.aggregate_id.starts_with(&prefix) {
            event.aggregate_id = event.aggregate_id[prefix.len()..].to_string();
        }
        
        event
    }
    
    /// Get tenant-specific table/collection name
    fn tenant_table_name(&self, base_name: &str) -> String {
        format!("{}_{}", self.tenant_id.db_prefix(), base_name)
    }
    
    /// Validate and record event storage operation
    fn validate_and_record(&self, operation: TenantOperation, event_count: u64) -> Result<()> {
        // Validate tenant isolation
        self.isolation.validate_operation(&self.tenant_id, &operation)?;
        
        // Check quotas
        self.quota.check_quota(ResourceType::Events, event_count)?;
        
        // Record usage
        self.quota.record_usage(ResourceType::Events, event_count);
        
        // Update metrics
        let mut metrics = self.metrics.write().unwrap();
        metrics.record_operation(operation, event_count);
        
        Ok(())
    }
    
    pub fn get_metrics(&self) -> TenantStorageMetrics {
        self.metrics.read().unwrap().clone()
    }
}

#[async_trait]
impl EventStore for TenantAwareEventStorage {
    async fn save_events(&self, events: Vec<Event>) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Validate operation for the first event's aggregate (assuming batch operations on same aggregate)
        if let Some(first_event) = events.first() {
            self.validate_and_record(
                TenantOperation::CreateEvent {
                    aggregate_id: first_event.aggregate_id.clone()
                },
                events.len() as u64
            )?;
        }
        
        // Transform events to include tenant scoping
        let scoped_events: Vec<Event> = events
            .into_iter()
            .map(|event| self.tenant_scoped_event(event))
            .collect();
        
        // Delegate to backend
        let result = self.backend.save_events(scoped_events).await;
        
        // Record performance metrics
        let duration = start_time.elapsed();
        let mut metrics = self.metrics.write().unwrap();
        metrics.record_save_operation(duration, result.is_ok());
        
        result
    }
    
    async fn load_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>> {
        let start_time = std::time::Instant::now();
        
        // Validate operation
        self.isolation.validate_operation(&self.tenant_id, &TenantOperation::ReadEvents {
            aggregate_id: aggregate_id.clone()
        })?;
        
        // Transform aggregate ID to include tenant namespace
        let scoped_aggregate_id = format!("{}:{}", self.tenant_id.db_prefix(), aggregate_id);
        
        // Load events from backend
        let result = self.backend.load_events(&scoped_aggregate_id, from_version).await;
        
        // Transform events back and record metrics
        let final_result = match result {
            Ok(events) => {
                let unscoped_events = events
                    .into_iter()
                    .map(|event| self.unscoped_event(event))
                    .collect::<Vec<Event>>();
                
                let mut metrics = self.metrics.write().unwrap();
                metrics.record_load_operation(start_time.elapsed(), true, unscoped_events.len());
                
                Ok(unscoped_events)
            }
            Err(e) => {
                let mut metrics = self.metrics.write().unwrap();
                metrics.record_load_operation(start_time.elapsed(), false, 0);
                Err(e)
            }
        };
        
        final_result
    }
    
    async fn load_events_by_type(
        &self,
        aggregate_type: &str,
        from_version: Option<AggregateVersion>,
    ) -> Result<Vec<Event>> {
        let start_time = std::time::Instant::now();
        
        // Create tenant-scoped aggregate type
        let scoped_aggregate_type = format!("{}:{}", self.tenant_id.db_prefix(), aggregate_type);
        
        // Load events from backend
        let result = self.backend.load_events_by_type(&scoped_aggregate_type, from_version).await;
        
        // Transform events back and record metrics
        match result {
            Ok(events) => {
                let unscoped_events = events
                    .into_iter()
                    .map(|event| self.unscoped_event(event))
                    .collect::<Vec<Event>>();
                
                let mut metrics = self.metrics.write().unwrap();
                metrics.record_load_operation(start_time.elapsed(), true, unscoped_events.len());
                
                Ok(unscoped_events)
            }
            Err(e) => {
                let mut metrics = self.metrics.write().unwrap();
                metrics.record_load_operation(start_time.elapsed(), false, 0);
                Err(e)
            }
        }
    }
    
    async fn get_aggregate_version(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateVersion>> {
        // Validate operation
        self.isolation.validate_operation(&self.tenant_id, &TenantOperation::ReadEvents {
            aggregate_id: aggregate_id.clone()
        })?;
        
        // Transform aggregate ID to include tenant namespace
        let scoped_aggregate_id = format!("{}:{}", self.tenant_id.db_prefix(), aggregate_id);
        
        self.backend.get_aggregate_version(&scoped_aggregate_id).await
    }
    
    fn set_event_streamer(&mut self, _streamer: Arc<dyn crate::streaming::EventStreamer + Send + Sync>) {
        // For tenant-aware storage, streaming would need to be tenant-scoped as well
        // This would be implemented in a production system
    }
}

/// Performance and usage metrics for tenant event storage
#[derive(Debug, Clone)]
pub struct TenantStorageMetrics {
    pub tenant_id: TenantId,
    pub total_save_operations: u64,
    pub total_load_operations: u64,
    pub total_events_saved: u64,
    pub total_events_loaded: u64,
    pub successful_saves: u64,
    pub successful_loads: u64,
    pub average_save_time_ms: f64,
    pub average_load_time_ms: f64,
    pub max_save_time_ms: f64,
    pub max_load_time_ms: f64,
    pub last_operation: Option<DateTime<Utc>>,
    pub operations_by_type: HashMap<String, u64>,
}

impl TenantStorageMetrics {
    pub fn new() -> Self {
        Self {
            tenant_id: TenantId::generate(), // Will be set properly when used
            total_save_operations: 0,
            total_load_operations: 0,
            total_events_saved: 0,
            total_events_loaded: 0,
            successful_saves: 0,
            successful_loads: 0,
            average_save_time_ms: 0.0,
            average_load_time_ms: 0.0,
            max_save_time_ms: 0.0,
            max_load_time_ms: 0.0,
            last_operation: None,
            operations_by_type: HashMap::new(),
        }
    }
    
    pub fn record_operation(&mut self, operation: TenantOperation, _event_count: u64) {
        self.last_operation = Some(Utc::now());
        
        let operation_type = match operation {
            TenantOperation::CreateEvent { .. } => "create_event",
            TenantOperation::ReadEvents { .. } => "read_events",
            TenantOperation::CreateProjection { .. } => "create_projection",
            TenantOperation::StreamEvents { .. } => "stream_events",
        };
        
        *self.operations_by_type.entry(operation_type.to_string()).or_insert(0) += 1;
    }
    
    pub fn record_save_operation(&mut self, duration: std::time::Duration, success: bool) {
        self.total_save_operations += 1;
        if success {
            self.successful_saves += 1;
        }
        
        let duration_ms = duration.as_millis() as f64;
        self.average_save_time_ms = 
            (self.average_save_time_ms * (self.total_save_operations - 1) as f64 + duration_ms)
            / self.total_save_operations as f64;
        
        if duration_ms > self.max_save_time_ms {
            self.max_save_time_ms = duration_ms;
        }
        
        self.last_operation = Some(Utc::now());
    }
    
    pub fn record_load_operation(&mut self, duration: std::time::Duration, success: bool, event_count: usize) {
        self.total_load_operations += 1;
        if success {
            self.successful_loads += 1;
            self.total_events_loaded += event_count as u64;
        }
        
        let duration_ms = duration.as_millis() as f64;
        self.average_load_time_ms = 
            (self.average_load_time_ms * (self.total_load_operations - 1) as f64 + duration_ms)
            / self.total_load_operations as f64;
        
        if duration_ms > self.max_load_time_ms {
            self.max_load_time_ms = duration_ms;
        }
        
        self.last_operation = Some(Utc::now());
    }
    
    /// Calculate success rates
    pub fn save_success_rate(&self) -> f64 {
        if self.total_save_operations == 0 {
            return 100.0;
        }
        (self.successful_saves as f64 / self.total_save_operations as f64) * 100.0
    }
    
    pub fn load_success_rate(&self) -> f64 {
        if self.total_load_operations == 0 {
            return 100.0;
        }
        (self.successful_loads as f64 / self.total_load_operations as f64) * 100.0
    }
    
    /// Check if performance targets are met
    pub fn is_performance_target_met(&self) -> bool {
        self.average_save_time_ms < 50.0 && self.average_load_time_ms < 20.0
    }
}

/// Batch operations for efficient tenant event storage
pub struct TenantEventBatch {
    tenant_id: TenantId,
    events: Vec<Event>,
    max_batch_size: usize,
}

impl TenantEventBatch {
    pub fn new(tenant_id: TenantId, max_batch_size: Option<usize>) -> Self {
        Self {
            tenant_id,
            events: Vec::new(),
            max_batch_size: max_batch_size.unwrap_or(1000),
        }
    }
    
    pub fn add_event(&mut self, event: Event) -> Result<()> {
        if self.events.len() >= self.max_batch_size {
            return Err(EventualiError::Tenant(format!(
                "Batch size limit exceeded: {} events", 
                self.max_batch_size
            )));
        }
        
        self.events.push(event);
        Ok(())
    }
    
    pub fn size(&self) -> usize {
        self.events.len()
    }
    
    pub fn is_full(&self) -> bool {
        self.events.len() >= self.max_batch_size
    }
    
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    pub async fn flush(&mut self, storage: &TenantAwareEventStorage) -> Result<()> {
        if self.events.is_empty() {
            return Ok(());
        }
        
        let events_to_save = std::mem::take(&mut self.events);
        storage.save_events(events_to_save).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::sqlite::SQLiteBackend;
    use crate::tenancy::isolation::{TenantIsolation, IsolationPolicy};
    use crate::tenancy::quota::{TenantQuota};
    use crate::tenancy::tenant::{TenantConfig, ResourceLimits};
    
    #[tokio::test]
    async fn test_tenant_aware_storage_isolation() {
        // Create test tenant
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        
        // Create in-memory SQLite backend
        let mut backend = SQLiteBackend::new("sqlite://:memory:".to_string()).unwrap();
        backend.initialize().await.unwrap();
        
        // Set up isolation and quota
        let isolation = Arc::new(TenantIsolation::new());
        isolation.register_tenant(tenant_id.clone(), IsolationPolicy::strict()).unwrap();
        
        let limits = ResourceLimits::default();
        let quota = Arc::new(TenantQuota::new(tenant_id.clone(), limits));
        
        // Create tenant-aware storage
        let storage = TenantAwareEventStorage::new(
            tenant_id.clone(),
            Arc::new(backend),
            isolation,
            quota,
        );
        
        // Test event saving and loading
        let test_event = Event::new(
            "test-aggregate".to_string(),
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            1,
            EventData::Json(serde_json::json!({"test": "data"}))
        );
        
        // Save events
        storage.save_events(vec![test_event.clone()]).await.unwrap();
        
        // Load events
        let loaded_events = storage.load_events(&"test-aggregate".to_string(), None).await.unwrap();
        
        assert_eq!(loaded_events.len(), 1);
        assert_eq!(loaded_events[0].aggregate_id, "test-aggregate");
        assert_eq!(loaded_events[0].event_type, "TestEvent");
        
        // Verify metrics
        let metrics = storage.get_metrics();
        assert_eq!(metrics.total_save_operations, 1);
        assert_eq!(metrics.total_load_operations, 1);
        assert!(metrics.is_performance_target_met());
    }
    
    #[test]
    fn test_tenant_event_batch() {
        let tenant_id = TenantId::new("batch-test".to_string()).unwrap();
        let mut batch = TenantEventBatch::new(tenant_id, Some(3));
        
        // Add events to batch
        for i in 0..2 {
            let event = Event::new(
                format!("aggregate-{}", i),
                "TestAggregate".to_string(),
                "TestEvent".to_string(),
                1,
                i as i64 + 1,
                EventData::Json(serde_json::json!({"index": i}))
            );
            
            batch.add_event(event).unwrap();
        }
        
        assert_eq!(batch.size(), 2);
        assert!(!batch.is_full());
        
        // Add one more to reach limit
        let event = Event::new(
            "aggregate-3".to_string(),
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            3,
            EventData::Json(serde_json::json!({"index": 3}))
        );
        
        batch.add_event(event).unwrap();
        assert!(batch.is_full());
        
        // Try to add one more (should fail)
        let overflow_event = Event::new(
            "aggregate-4".to_string(),
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            4,
            EventData::Json(serde_json::json!({"index": 4}))
        );
        
        assert!(batch.add_event(overflow_event).is_err());
    }
}