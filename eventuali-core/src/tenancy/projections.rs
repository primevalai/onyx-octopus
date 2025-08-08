use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::event::Event;
use crate::streaming::Projection;
use crate::error::{EventualiError, Result};
use super::tenant::TenantId;
use super::isolation::{TenantIsolation, TenantOperation};
use super::quota::{TenantQuota, ResourceType};

/// Tenant-scoped projection that maintains read models isolated per tenant
pub struct TenantScopedProjection {
    tenant_id: TenantId,
    projection_name: String,
    inner_projection: Arc<dyn Projection + Send + Sync>,
    isolation: Arc<TenantIsolation>,
    quota: Arc<TenantQuota>,
    metrics: Arc<RwLock<TenantProjectionMetrics>>,
}

impl TenantScopedProjection {
    pub fn new(
        tenant_id: TenantId,
        projection_name: String,
        inner_projection: Arc<dyn Projection + Send + Sync>,
        isolation: Arc<TenantIsolation>,
        quota: Arc<TenantQuota>,
    ) -> Self {
        Self {
            tenant_id,
            projection_name,
            inner_projection,
            isolation,
            quota,
            metrics: Arc::new(RwLock::new(TenantProjectionMetrics::new())),
        }
    }
    
    /// Get tenant-scoped projection name
    pub fn scoped_name(&self) -> String {
        format!("{}:{}", self.tenant_id.db_prefix(), self.projection_name)
    }
    
    /// Validate that the event belongs to this tenant's namespace
    fn validate_event_belongs_to_tenant(&self, event: &Event) -> Result<()> {
        let expected_prefix = format!("{}:", self.tenant_id.db_prefix());
        
        if !event.aggregate_id.starts_with(&expected_prefix) {
            return Err(EventualiError::Tenant(format!(
                "Event aggregate_id '{}' does not belong to tenant '{}'",
                event.aggregate_id,
                self.tenant_id.as_str()
            )));
        }
        
        Ok(())
    }
    
    /// Transform event to remove tenant namespace for projection processing
    fn unscoped_event(&self, mut event: Event) -> Event {
        let prefix = format!("{}:", self.tenant_id.db_prefix());
        if event.aggregate_id.starts_with(&prefix) {
            event.aggregate_id = event.aggregate_id[prefix.len()..].to_string();
        }
        event
    }
    
    pub fn get_metrics(&self) -> TenantProjectionMetrics {
        self.metrics.read().unwrap().clone()
    }
}

#[async_trait]
impl Projection for TenantScopedProjection {
    async fn handle_event(&self, event: &Event) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Validate tenant isolation
        self.isolation.validate_operation(&self.tenant_id, &TenantOperation::CreateProjection {
            name: self.projection_name.clone()
        })?;
        
        // Validate event belongs to tenant
        self.validate_event_belongs_to_tenant(event)?;
        
        // Check quotas
        self.quota.check_quota(ResourceType::Projections, 1)?;
        
        // Transform event to remove tenant scoping for inner projection
        let unscoped_event = self.unscoped_event(event.clone());
        
        // Delegate to inner projection
        let result = self.inner_projection.handle_event(&unscoped_event).await;
        
        // Record metrics
        let duration = start_time.elapsed();
        let mut metrics = self.metrics.write().unwrap();
        metrics.record_event_processing(duration, result.is_ok());
        
        if result.is_ok() {
            self.quota.record_usage(ResourceType::Projections, 1);
        }
        
        result
    }
    
    async fn reset(&self) -> Result<()> {
        let result = self.inner_projection.reset().await;
        
        if result.is_ok() {
            let mut metrics = self.metrics.write().unwrap();
            metrics.reset_counters();
        }
        
        result
    }
    
    async fn get_last_processed_position(&self) -> Result<Option<u64>> {
        self.inner_projection.get_last_processed_position().await
    }
    
    async fn set_last_processed_position(&self, position: u64) -> Result<()> {
        self.inner_projection.set_last_processed_position(position).await
    }
}

/// Manager for tenant-scoped projections
pub struct TenantProjectionManager {
    tenant_id: TenantId,
    projections: Arc<RwLock<HashMap<String, Arc<TenantScopedProjection>>>>,
    isolation: Arc<TenantIsolation>,
    quota: Arc<TenantQuota>,
    registry: Arc<RwLock<TenantProjectionRegistry>>,
}

impl TenantProjectionManager {
    pub fn new(
        tenant_id: TenantId,
        isolation: Arc<TenantIsolation>,
        quota: Arc<TenantQuota>,
    ) -> Self {
        Self {
            tenant_id,
            projections: Arc::new(RwLock::new(HashMap::new())),
            isolation,
            quota,
            registry: Arc::new(RwLock::new(TenantProjectionRegistry::new())),
        }
    }
    
    /// Register a new projection for this tenant
    pub fn register_projection(
        &self,
        name: String,
        projection: Arc<dyn Projection + Send + Sync>,
    ) -> Result<Arc<TenantScopedProjection>> {
        // Check if we can add more projections
        self.quota.check_quota(ResourceType::Projections, 1)?;
        
        let tenant_projection = Arc::new(TenantScopedProjection::new(
            self.tenant_id.clone(),
            name.clone(),
            projection,
            self.isolation.clone(),
            self.quota.clone(),
        ));
        
        // Register the projection
        {
            let mut projections = self.projections.write().unwrap();
            if projections.contains_key(&name) {
                return Err(EventualiError::Tenant(format!(
                    "Projection '{}' already exists for tenant '{}'",
                    name,
                    self.tenant_id.as_str()
                )));
            }
            projections.insert(name.clone(), tenant_projection.clone());
        }
        
        // Update registry
        {
            let mut registry = self.registry.write().unwrap();
            registry.register_projection(name, self.tenant_id.clone())?;
        }
        
        // Record usage
        self.quota.record_usage(ResourceType::Projections, 1);
        
        Ok(tenant_projection)
    }
    
    /// Get a projection by name
    pub fn get_projection(&self, name: &str) -> Option<Arc<TenantScopedProjection>> {
        let projections = self.projections.read().unwrap();
        projections.get(name).cloned()
    }
    
    /// List all projections for this tenant
    pub fn list_projections(&self) -> Vec<String> {
        let projections = self.projections.read().unwrap();
        projections.keys().cloned().collect()
    }
    
    /// Remove a projection
    pub fn remove_projection(&self, name: &str) -> Result<()> {
        let mut projections = self.projections.write().unwrap();
        
        if projections.remove(name).is_some() {
            let mut registry = self.registry.write().unwrap();
            registry.unregister_projection(name);
            Ok(())
        } else {
            Err(EventualiError::Tenant(format!(
                "Projection '{}' not found for tenant '{}'",
                name,
                self.tenant_id.as_str()
            )))
        }
    }
    
    /// Get aggregated metrics for all projections
    pub fn get_aggregated_metrics(&self) -> TenantProjectionMetrics {
        let projections = self.projections.read().unwrap();
        let mut aggregated = TenantProjectionMetrics::new();
        
        for projection in projections.values() {
            let metrics = projection.get_metrics();
            aggregated.aggregate_with(&metrics);
        }
        
        aggregated
    }
    
    /// Process an event through all registered projections
    pub async fn process_event(&self, event: Event) -> Result<()> {
        let projections = {
            let projections_guard = self.projections.read().unwrap();
            projections_guard.values().cloned().collect::<Vec<_>>()
        };
        
        let mut results = Vec::new();
        
        for projection in projections {
            let result = projection.handle_event(&event).await;
            results.push(result);
        }
        
        // Check if all projections succeeded
        for result in results {
            result?;
        }
        
        Ok(())
    }
    
    pub fn get_registry(&self) -> TenantProjectionRegistry {
        self.registry.read().unwrap().clone()
    }
}

/// Registry for tracking tenant projections
#[derive(Debug, Clone)]
pub struct TenantProjectionRegistry {
    projections: HashMap<String, ProjectionRegistration>,
    #[allow(dead_code)] // Registry creation timestamp for auditing (stored but not currently accessed)
    created_at: DateTime<Utc>,
}

impl Default for TenantProjectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantProjectionRegistry {
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
            created_at: Utc::now(),
        }
    }
    
    pub fn register_projection(&mut self, name: String, tenant_id: TenantId) -> Result<()> {
        let registration = ProjectionRegistration {
            name: name.clone(),
            tenant_id,
            registered_at: Utc::now(),
            last_processed: None,
            event_count: 0,
            status: ProjectionStatus::Active,
        };
        
        self.projections.insert(name, registration);
        Ok(())
    }
    
    pub fn unregister_projection(&mut self, name: &str) {
        self.projections.remove(name);
    }
    
    pub fn get_projection_count(&self) -> usize {
        self.projections.len()
    }
    
    pub fn get_active_projections(&self) -> Vec<String> {
        self.projections
            .values()
            .filter(|reg| matches!(reg.status, ProjectionStatus::Active))
            .map(|reg| reg.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
struct ProjectionRegistration {
    #[allow(dead_code)] // Name stored for projection identification (used in registry operations but not directly accessed)
    name: String,
    #[allow(dead_code)] // Tenant ID for isolation tracking (stored but not currently queried in registration info)
    tenant_id: TenantId,
    #[allow(dead_code)] // Registration timestamp for audit purposes (stored but not currently accessed)
    registered_at: DateTime<Utc>,
    #[allow(dead_code)] // Last processing time for monitoring (stored but not actively used)
    last_processed: Option<DateTime<Utc>>,
    #[allow(dead_code)] // Event count tracking for analytics (stored but not currently queried)
    event_count: u64,
    status: ProjectionStatus,
}

#[derive(Debug, Clone)]
enum ProjectionStatus {
    Active,
    #[allow(dead_code)] // Paused status for future projection management features
    Paused,
    #[allow(dead_code)] // Error status for future error handling and recovery
    Error,
}

/// Performance and usage metrics for tenant projections
#[derive(Debug, Clone)]
pub struct TenantProjectionMetrics {
    pub events_processed: u64,
    pub successful_events: u64,
    pub failed_events: u64,
    pub total_processing_time_ms: f64,
    pub average_processing_time_ms: f64,
    pub max_processing_time_ms: f64,
    pub rebuilds_performed: u64,
    pub successful_rebuilds: u64,
    pub last_processed: Option<DateTime<Utc>>,
    pub last_rebuild: Option<DateTime<Utc>>,
}

impl Default for TenantProjectionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantProjectionMetrics {
    pub fn new() -> Self {
        Self {
            events_processed: 0,
            successful_events: 0,
            failed_events: 0,
            total_processing_time_ms: 0.0,
            average_processing_time_ms: 0.0,
            max_processing_time_ms: 0.0,
            rebuilds_performed: 0,
            successful_rebuilds: 0,
            last_processed: None,
            last_rebuild: None,
        }
    }
    
    pub fn record_event_processing(&mut self, duration: std::time::Duration, success: bool) {
        self.events_processed += 1;
        
        if success {
            self.successful_events += 1;
        } else {
            self.failed_events += 1;
        }
        
        let duration_ms = duration.as_millis() as f64;
        self.total_processing_time_ms += duration_ms;
        self.average_processing_time_ms = 
            self.total_processing_time_ms / self.events_processed as f64;
        
        if duration_ms > self.max_processing_time_ms {
            self.max_processing_time_ms = duration_ms;
        }
        
        self.last_processed = Some(Utc::now());
    }
    
    pub fn record_rebuild(&mut self, _duration: std::time::Duration, success: bool) {
        self.rebuilds_performed += 1;
        
        if success {
            self.successful_rebuilds += 1;
        }
        
        self.last_rebuild = Some(Utc::now());
    }
    
    pub fn reset_counters(&mut self) {
        self.events_processed = 0;
        self.successful_events = 0;
        self.failed_events = 0;
        self.total_processing_time_ms = 0.0;
        self.average_processing_time_ms = 0.0;
        self.max_processing_time_ms = 0.0;
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.events_processed == 0 {
            return 100.0;
        }
        (self.successful_events as f64 / self.events_processed as f64) * 100.0
    }
    
    pub fn rebuild_success_rate(&self) -> f64 {
        if self.rebuilds_performed == 0 {
            return 100.0;
        }
        (self.successful_rebuilds as f64 / self.rebuilds_performed as f64) * 100.0
    }
    
    pub fn is_performance_target_met(&self) -> bool {
        // Target: average processing time < 10ms
        self.average_processing_time_ms < 10.0
    }
    
    pub fn aggregate_with(&mut self, other: &TenantProjectionMetrics) {
        self.events_processed += other.events_processed;
        self.successful_events += other.successful_events;
        self.failed_events += other.failed_events;
        self.total_processing_time_ms += other.total_processing_time_ms;
        self.rebuilds_performed += other.rebuilds_performed;
        self.successful_rebuilds += other.successful_rebuilds;
        
        if self.events_processed > 0 {
            self.average_processing_time_ms = 
                self.total_processing_time_ms / self.events_processed as f64;
        }
        
        if other.max_processing_time_ms > self.max_processing_time_ms {
            self.max_processing_time_ms = other.max_processing_time_ms;
        }
        
        // Update timestamps to most recent
        if other.last_processed.is_some() && 
           (self.last_processed.is_none() || other.last_processed > self.last_processed) {
            self.last_processed = other.last_processed;
        }
        
        if other.last_rebuild.is_some() && 
           (self.last_rebuild.is_none() || other.last_rebuild > self.last_rebuild) {
            self.last_rebuild = other.last_rebuild;
        }
    }
}

/// Sample projection implementations for testing
pub mod sample_projections {
    use super::*;
    use serde_json::Value;
    
    /// A simple analytics projection that counts events by type
    pub struct EventAnalyticsProjection {
        #[allow(dead_code)] // Projection name for identification (stored but not currently accessed)
        name: String,
        data: Arc<RwLock<HashMap<String, EventTypeCount>>>,
    }
    
    impl EventAnalyticsProjection {
        pub fn new(name: String) -> Self {
            Self {
                name,
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        pub fn get_counts(&self) -> HashMap<String, EventTypeCount> {
            self.data.read().unwrap().clone()
        }
    }
    
    #[async_trait]
    impl Projection for EventAnalyticsProjection {
        async fn handle_event(&self, event: &Event) -> Result<()> {
            let mut data = self.data.write().unwrap();
            let count = data.entry(event.event_type.clone()).or_default();
            
            count.total_count += 1;
            count.last_seen = Some(Utc::now());
            
            // Try to parse the event data for additional analytics
            if let crate::event::EventData::Json(json_data) = &event.data {
                count.sample_data = Some(json_data.clone());
            }
            
            Ok(())
        }
        
        async fn reset(&self) -> Result<()> {
            // Clear all data for reset
            let mut data = self.data.write().unwrap();
            data.clear();
            Ok(())
        }
        
        async fn get_last_processed_position(&self) -> Result<Option<u64>> {
            // Simple implementation - doesn't track position
            Ok(None)
        }
        
        async fn set_last_processed_position(&self, _position: u64) -> Result<()> {
            // Simple implementation - doesn't track position
            Ok(())
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct EventTypeCount {
        pub total_count: u64,
        pub last_seen: Option<DateTime<Utc>>,
        pub sample_data: Option<Value>,
    }
    
    impl Default for EventTypeCount {
        fn default() -> Self {
            Self::new()
        }
    }

    impl EventTypeCount {
        pub fn new() -> Self {
            Self {
                total_count: 0,
                last_seen: None,
                sample_data: None,
            }
        }
    }
    
    /// A user activity projection that tracks user actions
    pub struct UserActivityProjection {
        #[allow(dead_code)] // Projection name for identification (stored but not currently accessed)
        name: String,
        data: Arc<RwLock<HashMap<String, UserActivity>>>,
    }
    
    impl UserActivityProjection {
        pub fn new(name: String) -> Self {
            Self {
                name,
                data: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        pub fn get_user_activity(&self, user_id: &str) -> Option<UserActivity> {
            self.data.read().unwrap().get(user_id).cloned()
        }
        
        pub fn get_all_activities(&self) -> HashMap<String, UserActivity> {
            self.data.read().unwrap().clone()
        }
    }
    
    #[async_trait]
    impl Projection for UserActivityProjection {
        async fn handle_event(&self, event: &Event) -> Result<()> {
            // Extract user_id from event metadata or data
            if let Some(user_id) = event.metadata.headers.get("user_id") {
                let mut data = self.data.write().unwrap();
                let activity = data.entry(user_id.clone()).or_insert(UserActivity::new(user_id.clone()));
                
                activity.total_events += 1;
                activity.last_activity = Some(Utc::now());
                activity.event_types.insert(event.event_type.clone());
            }
            
            Ok(())
        }
        
        async fn reset(&self) -> Result<()> {
            let mut data = self.data.write().unwrap();
            data.clear();
            Ok(())
        }
        
        async fn get_last_processed_position(&self) -> Result<Option<u64>> {
            // Simple implementation - doesn't track position
            Ok(None)
        }
        
        async fn set_last_processed_position(&self, _position: u64) -> Result<()> {
            // Simple implementation - doesn't track position
            Ok(())
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct UserActivity {
        pub user_id: String,
        pub total_events: u64,
        pub last_activity: Option<DateTime<Utc>>,
        pub event_types: std::collections::HashSet<String>,
    }
    
    impl UserActivity {
        pub fn new(user_id: String) -> Self {
            Self {
                user_id,
                total_events: 0,
                last_activity: None,
                event_types: std::collections::HashSet::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::sample_projections::*;
    use crate::tenancy::isolation::{TenantIsolation, IsolationPolicy};
    use crate::tenancy::quota::TenantQuota;
    use crate::tenancy::tenant::{TenantConfig, ResourceLimits};
    use crate::event::{Event, EventData, EventMetadata};
    use std::collections::HashMap;
    use chrono::Utc;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_tenant_scoped_projection() {
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        
        // Set up isolation and quota
        let isolation = Arc::new(TenantIsolation::new());
        isolation.register_tenant(tenant_id.clone(), IsolationPolicy::strict()).unwrap();
        
        let limits = ResourceLimits::default();
        let quota = Arc::new(TenantQuota::new(tenant_id.clone(), limits));
        
        // Create sample projection
        let analytics_projection = Arc::new(EventAnalyticsProjection::new("analytics".to_string()));
        
        // Create tenant-scoped projection
        let tenant_projection = TenantScopedProjection::new(
            tenant_id.clone(),
            "test-analytics".to_string(),
            analytics_projection.clone(),
            isolation,
            quota,
        );
        
        // Create test event with proper tenant scoping
        let test_event = Event::new(
            format!("{}:test-aggregate", tenant_id.db_prefix()),
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            1,
            EventData::Json(serde_json::json!({"test": "data"}))
        );
        
        // Process event
        tenant_projection.handle_event(&test_event).await.unwrap();
        
        // Verify metrics
        let metrics = tenant_projection.get_metrics();
        assert_eq!(metrics.events_processed, 1);
        assert_eq!(metrics.successful_events, 1);
        assert!(metrics.is_performance_target_met());
    }
    
    #[tokio::test]
    async fn test_tenant_projection_manager() {
        let tenant_id = TenantId::new("manager-test".to_string()).unwrap();
        
        let isolation = Arc::new(TenantIsolation::new());
        isolation.register_tenant(tenant_id.clone(), IsolationPolicy::strict()).unwrap();
        
        let limits = ResourceLimits::default();
        let quota = Arc::new(TenantQuota::new(tenant_id.clone(), limits));
        
        let manager = TenantProjectionManager::new(tenant_id.clone(), isolation, quota);
        
        // Register projections
        let analytics = Arc::new(EventAnalyticsProjection::new("analytics".to_string()));
        let user_activity = Arc::new(UserActivityProjection::new("user-activity".to_string()));
        
        manager.register_projection("analytics".to_string(), analytics).unwrap();
        manager.register_projection("user-activity".to_string(), user_activity).unwrap();
        
        // Verify projections are registered
        assert_eq!(manager.list_projections().len(), 2);
        assert!(manager.get_projection("analytics").is_some());
        assert!(manager.get_projection("user-activity").is_some());
        
        // Test registry
        let registry = manager.get_registry();
        assert_eq!(registry.get_projection_count(), 2);
        assert_eq!(registry.get_active_projections().len(), 2);
    }
}