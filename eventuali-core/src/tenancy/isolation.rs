use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::event::Event;
use crate::aggregate::{AggregateId, AggregateVersion};
use crate::store::EventStore;
use crate::error::{EventualiError, Result};
use super::tenant::{TenantId, TenantError};

/// Tenant isolation enforcement mechanism
pub struct TenantIsolation {
    isolation_policies: Arc<RwLock<HashMap<TenantId, IsolationPolicy>>>,
    performance_monitor: Arc<RwLock<IsolationMetrics>>,
}

impl Default for TenantIsolation {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantIsolation {
    pub fn new() -> Self {
        Self {
            isolation_policies: Arc::new(RwLock::new(HashMap::new())),
            performance_monitor: Arc::new(RwLock::new(IsolationMetrics::new())),
        }
    }
    
    /// Register a new tenant with isolation policy
    pub fn register_tenant(&self, tenant_id: TenantId, policy: IsolationPolicy) -> Result<()> {
        let mut policies = self.isolation_policies.write().unwrap();
        policies.insert(tenant_id, policy);
        Ok(())
    }
    
    /// Validate that an operation is allowed for the tenant
    pub fn validate_operation(&self, tenant_id: &TenantId, operation: &TenantOperation) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        let policies = self.isolation_policies.read().unwrap();
        let policy = policies.get(tenant_id)
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))?;
        
        let result = policy.validate_operation(operation);
        
        // Track performance
        let duration = start_time.elapsed();
        if duration.as_millis() > 10 {
            // Log if we exceed 10ms target
            eprintln!("Warning: Tenant isolation check took {}ms", duration.as_millis());
        }
        
        let mut metrics = self.performance_monitor.write().unwrap();
        metrics.record_validation(duration, result.is_ok());
        
        result
    }
    
    /// Get isolation metrics for monitoring
    pub fn get_metrics(&self) -> IsolationMetrics {
        self.performance_monitor.read().unwrap().clone()
    }
}

/// Tenant operation types for validation
#[derive(Debug, Clone)]
pub enum TenantOperation {
    CreateEvent { aggregate_id: AggregateId },
    ReadEvents { aggregate_id: AggregateId },
    CreateProjection { name: String },
    StreamEvents { from_timestamp: Option<DateTime<Utc>> },
}

/// Isolation policy for a tenant
#[derive(Debug, Clone)]
pub struct IsolationPolicy {
    pub enforce_namespace: bool,
    pub validate_access_patterns: bool,
    pub audit_all_operations: bool,
    pub max_cross_tenant_references: Option<u32>,
}

impl IsolationPolicy {
    pub fn strict() -> Self {
        Self {
            enforce_namespace: true,
            validate_access_patterns: true,
            audit_all_operations: true,
            max_cross_tenant_references: Some(0),
        }
    }
    
    pub fn relaxed() -> Self {
        Self {
            enforce_namespace: true,
            validate_access_patterns: false,
            audit_all_operations: false,
            max_cross_tenant_references: Some(10),
        }
    }
    
    fn validate_operation(&self, operation: &TenantOperation) -> Result<()> {
        // Implement validation logic
        match operation {
            TenantOperation::CreateEvent { aggregate_id } => {
                if self.enforce_namespace && !self.validate_aggregate_namespace(aggregate_id) {
                    return Err(EventualiError::from(TenantError::IsolationViolation(
                        "Aggregate ID does not match tenant namespace".to_string()
                    )));
                }
            },
            TenantOperation::ReadEvents { aggregate_id } => {
                if self.enforce_namespace && !self.validate_aggregate_namespace(aggregate_id) {
                    return Err(EventualiError::from(TenantError::IsolationViolation(
                        "Aggregate ID does not match tenant namespace".to_string()
                    )));
                }
            },
            TenantOperation::CreateProjection { .. } => {
                // Additional validation for projections
            },
            TenantOperation::StreamEvents { .. } => {
                // Validate streaming permissions
            },
        }
        Ok(())
    }
    
    fn validate_aggregate_namespace(&self, _aggregate_id: &AggregateId) -> bool {
        // Simple validation - in real implementation this would check namespace prefixes
        true
    }
}

/// Performance metrics for tenant isolation
#[derive(Debug, Clone)]
pub struct IsolationMetrics {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub average_validation_time_ms: f64,
    pub max_validation_time_ms: f64,
    pub violations_detected: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for IsolationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl IsolationMetrics {
    pub fn new() -> Self {
        Self {
            total_validations: 0,
            successful_validations: 0,
            average_validation_time_ms: 0.0,
            max_validation_time_ms: 0.0,
            violations_detected: 0,
            last_updated: Utc::now(),
        }
    }
    
    pub fn record_validation(&mut self, duration: std::time::Duration, success: bool) {
        self.total_validations += 1;
        if success {
            self.successful_validations += 1;
        } else {
            self.violations_detected += 1;
        }
        
        let duration_ms = duration.as_millis() as f64;
        self.average_validation_time_ms = 
            (self.average_validation_time_ms * (self.total_validations - 1) as f64 + duration_ms) 
            / self.total_validations as f64;
        
        if duration_ms > self.max_validation_time_ms {
            self.max_validation_time_ms = duration_ms;
        }
        
        self.last_updated = Utc::now();
    }
    
    /// Check if we're meeting the <10ms performance target
    pub fn is_performance_target_met(&self) -> bool {
        self.average_validation_time_ms < 10.0 && self.max_validation_time_ms < 50.0
    }
    
    /// Calculate isolation success rate (target: 99.9%)
    pub fn isolation_success_rate(&self) -> f64 {
        if self.total_validations == 0 {
            return 100.0;
        }
        (self.successful_validations as f64 / self.total_validations as f64) * 100.0
    }
}

/// Tenant-scoped event store wrapper
pub struct IsolatedEventStore {
    tenant_id: TenantId,
    inner_store: Arc<dyn EventStore + Send + Sync>,
    isolation: Arc<TenantIsolation>,
}

impl IsolatedEventStore {
    pub fn new(
        tenant_id: TenantId, 
        inner_store: Arc<dyn EventStore + Send + Sync>,
        isolation: Arc<TenantIsolation>
    ) -> Self {
        Self {
            tenant_id,
            inner_store,
            isolation,
        }
    }
    
    /// Get the tenant ID this store is scoped to
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    
    /// Transform aggregate ID to include tenant namespace
    fn tenant_scoped_aggregate_id(&self, aggregate_id: &AggregateId) -> AggregateId {
        format!("{}:{}", self.tenant_id.db_prefix(), aggregate_id)
    }
}

#[async_trait]
impl EventStore for IsolatedEventStore {
    async fn save_events(&self, events: Vec<Event>) -> Result<()> {
        // For each event, validate operation and transform aggregate IDs
        let mut scoped_events = Vec::new();
        
        for mut event in events {
            // Validate operation
            self.isolation.validate_operation(&self.tenant_id, &TenantOperation::CreateEvent { 
                aggregate_id: event.aggregate_id.clone() 
            })?;
            
            // Transform aggregate ID to include tenant namespace
            event.aggregate_id = self.tenant_scoped_aggregate_id(&event.aggregate_id);
            scoped_events.push(event);
        }
        
        // Delegate to inner store
        self.inner_store.save_events(scoped_events).await
    }
    
    async fn load_events(&self, aggregate_id: &AggregateId, from_version: Option<AggregateVersion>) -> Result<Vec<Event>> {
        // Validate operation
        self.isolation.validate_operation(&self.tenant_id, &TenantOperation::ReadEvents { 
            aggregate_id: aggregate_id.clone() 
        })?;
        
        // Transform aggregate ID to include tenant namespace
        let scoped_aggregate_id = self.tenant_scoped_aggregate_id(aggregate_id);
        
        // Delegate to inner store
        let mut events = self.inner_store.load_events(&scoped_aggregate_id, from_version).await?;
        
        // Transform aggregate IDs back to unscoped versions for the caller
        for event in &mut events {
            event.aggregate_id = aggregate_id.clone();
        }
        
        Ok(events)
    }
    
    async fn load_events_by_type(&self, aggregate_type: &str, from_version: Option<AggregateVersion>) -> Result<Vec<Event>> {
        // Create a tenant-scoped aggregate type
        let scoped_aggregate_type = format!("{}:{}", self.tenant_id.db_prefix(), aggregate_type);
        
        // Delegate to inner store
        let mut events = self.inner_store.load_events_by_type(&scoped_aggregate_type, from_version).await?;
        
        // Transform aggregate IDs back to unscoped versions for the caller
        for event in &mut events {
            // Remove tenant prefix from aggregate ID
            if let Some(unscoped) = event.aggregate_id.strip_prefix(&format!("{}:", self.tenant_id.db_prefix())) {
                event.aggregate_id = unscoped.to_string();
            }
        }
        
        Ok(events)
    }
    
    async fn get_aggregate_version(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateVersion>> {
        // Validate operation (as read)
        self.isolation.validate_operation(&self.tenant_id, &TenantOperation::ReadEvents { 
            aggregate_id: aggregate_id.clone() 
        })?;
        
        // Transform aggregate ID to include tenant namespace
        let scoped_aggregate_id = self.tenant_scoped_aggregate_id(aggregate_id);
        
        // Delegate to inner store
        self.inner_store.get_aggregate_version(&scoped_aggregate_id).await
    }
    
    fn set_event_streamer(&mut self, _streamer: Arc<dyn crate::streaming::EventStreamer + Send + Sync>) {
        // This would need to be handled differently as we have a reference to the inner store
        // For now, we'll need to assume the inner store is mutable or use interior mutability
        // This is a design limitation that would need to be addressed in a real implementation
    }
}

/// Tenant scope utility for ensuring operations stay within tenant boundaries
pub struct TenantScope {
    pub tenant_id: TenantId,
    pub context: TenantContext,
}

impl TenantScope {
    pub fn new(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            context: TenantContext::new(),
        }
    }
    
    /// Execute a closure within this tenant scope
    pub fn execute<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TenantScope) -> Result<T>,
    {
        f(self)
    }
}

/// Context information for tenant operations
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub operation_id: String,
    pub started_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Default for TenantContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantContext {
    pub fn new() -> Self {
        Self {
            operation_id: uuid::Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_isolation_creation() {
        let isolation = TenantIsolation::new();
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        
        let policy = IsolationPolicy::strict();
        assert!(isolation.register_tenant(tenant_id, policy).is_ok());
    }
    
    #[test]
    fn test_isolation_metrics_performance_target() {
        let mut metrics = IsolationMetrics::new();
        
        // Record some fast validations
        metrics.record_validation(std::time::Duration::from_millis(5), true);
        metrics.record_validation(std::time::Duration::from_millis(8), true);
        
        assert!(metrics.is_performance_target_met());
        assert!(metrics.isolation_success_rate() > 99.0);
    }
}