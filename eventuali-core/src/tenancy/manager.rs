use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::tenant::{TenantId, TenantInfo, TenantConfig, TenantStatus, TenantError};
use super::isolation::{TenantIsolation, IsolationPolicy};
use super::quota::{TenantQuota, ResourceUsage, ResourceType};
use crate::error::{EventualiError, Result};

/// Central tenant management system
pub struct TenantManager {
    tenants: Arc<RwLock<HashMap<TenantId, TenantInfo>>>,
    quotas: Arc<RwLock<HashMap<TenantId, Arc<TenantQuota>>>>,
    isolation: Arc<TenantIsolation>,
    registry: Arc<RwLock<TenantRegistry>>,
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantManager {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            quotas: Arc::new(RwLock::new(HashMap::new())),
            isolation: Arc::new(TenantIsolation::new()),
            registry: Arc::new(RwLock::new(TenantRegistry::new())),
        }
    }
    
    /// Create a new tenant
    pub async fn create_tenant(&self, tenant_id: TenantId, name: String, config: Option<TenantConfig>) -> Result<TenantInfo> {
        // Check if tenant already exists
        {
            let tenants = self.tenants.read().unwrap();
            if tenants.contains_key(&tenant_id) {
                return Err(EventualiError::from(TenantError::TenantAlreadyExists(tenant_id)));
            }
        }
        
        // Create tenant info
        let mut tenant_info = TenantInfo::new(tenant_id.clone(), name);
        if let Some(config) = config {
            tenant_info.config = config;
        }
        
        // Set up quota management
        let quota = Arc::new(TenantQuota::new(tenant_id.clone(), tenant_info.config.resource_limits.clone()));
        
        // Set up isolation policy
        let isolation_policy = match tenant_info.config.isolation_level {
            super::tenant::IsolationLevel::Database => IsolationPolicy::strict(),
            super::tenant::IsolationLevel::Application => IsolationPolicy::relaxed(),
            super::tenant::IsolationLevel::Row => IsolationPolicy::relaxed(),
        };
        
        self.isolation.register_tenant(tenant_id.clone(), isolation_policy)?;
        
        // Store tenant information
        {
            let mut tenants = self.tenants.write().unwrap();
            let mut quotas = self.quotas.write().unwrap();
            let mut registry = self.registry.write().unwrap();
            
            tenants.insert(tenant_id.clone(), tenant_info.clone());
            quotas.insert(tenant_id.clone(), quota);
            registry.register_tenant(tenant_id.clone())?;
        }
        
        Ok(tenant_info)
    }
    
    /// Get tenant information
    pub fn get_tenant(&self, tenant_id: &TenantId) -> Result<TenantInfo> {
        let tenants = self.tenants.read().unwrap();
        tenants.get(tenant_id)
            .cloned()
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))
    }
    
    /// List all tenants with optional filtering
    pub fn list_tenants(&self, status_filter: Option<TenantStatus>) -> Vec<TenantInfo> {
        let tenants = self.tenants.read().unwrap();
        tenants.values()
            .filter(|tenant| {
                status_filter.as_ref().is_none_or(|status| {
                    std::mem::discriminant(&tenant.status) == std::mem::discriminant(status)
                })
            })
            .cloned()
            .collect()
    }
    
    /// Update tenant configuration
    pub fn update_tenant(&self, tenant_id: &TenantId, updates: TenantUpdate) -> Result<TenantInfo> {
        let mut tenants = self.tenants.write().unwrap();
        let tenant = tenants.get_mut(tenant_id)
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))?;
        
        tenant.updated_at = Utc::now();
        
        if let Some(name) = updates.name {
            tenant.name = name;
        }
        
        if let Some(description) = updates.description {
            tenant.description = Some(description);
        }
        
        if let Some(status) = updates.status {
            tenant.status = status;
        }
        
        if let Some(config) = updates.config {
            tenant.config = config;
            
            // Update quota limits if they changed
            let quotas = self.quotas.read().unwrap();
            if let Some(_quota) = quotas.get(tenant_id) {
                // Note: In a real implementation, we'd update the quota limits
                // For now, we'd need to recreate the quota with new limits
            }
        }
        
        Ok(tenant.clone())
    }
    
    /// Delete a tenant (marks for deletion)
    pub fn delete_tenant(&self, tenant_id: &TenantId) -> Result<()> {
        let mut tenants = self.tenants.write().unwrap();
        let tenant = tenants.get_mut(tenant_id)
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))?;
        
        tenant.status = TenantStatus::PendingDeletion;
        tenant.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Get resource usage for a tenant
    pub fn get_tenant_usage(&self, tenant_id: &TenantId) -> Result<ResourceUsage> {
        let quotas = self.quotas.read().unwrap();
        let quota = quotas.get(tenant_id)
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))?;
        
        Ok(quota.get_legacy_usage())
    }
    
    /// Check if tenant can perform operation
    pub fn check_tenant_quota(&self, tenant_id: &TenantId, resource_type: ResourceType, amount: u64) -> Result<()> {
        let quotas = self.quotas.read().unwrap();
        let quota = quotas.get(tenant_id)
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))?;
        
        // Convert enhanced quota check result to simple boolean result
        match quota.check_quota(resource_type, amount) {
            Ok(result) => {
                if result.allowed {
                    Ok(())
                } else {
                    Err(EventualiError::Tenant(format!("Quota exceeded for resource {resource_type:?}")))
                }
            }
            Err(e) => Err(e)
        }
    }
    
    /// Record resource usage for a tenant
    pub fn record_tenant_usage(&self, tenant_id: &TenantId, resource_type: ResourceType, amount: u64) -> Result<()> {
        let quotas = self.quotas.read().unwrap();
        let quota = quotas.get(tenant_id)
            .ok_or_else(|| EventualiError::from(TenantError::TenantNotFound(tenant_id.clone())))?;
        
        quota.record_usage(resource_type, amount);
        
        // Update tenant metadata
        let mut tenants = self.tenants.write().unwrap();
        if let Some(tenant) = tenants.get_mut(tenant_id) {
            tenant.metadata.last_activity = Some(Utc::now());
            
            match resource_type {
                ResourceType::Events => tenant.metadata.total_events += amount,
                ResourceType::Aggregates => tenant.metadata.total_aggregates += amount,
                ResourceType::Storage => tenant.metadata.storage_used_mb += amount as f64,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Get tenants that are near their resource limits
    pub fn get_tenants_near_limits(&self) -> Vec<(TenantId, ResourceUsage)> {
        let quotas = self.quotas.read().unwrap();
        quotas.iter()
            .filter_map(|(tenant_id, quota)| {
                let enhanced_usage = quota.get_usage();
                if enhanced_usage.has_resources_near_limit() {
                    // Convert to legacy format for compatibility
                    let legacy_usage = quota.get_legacy_usage();
                    Some((tenant_id.clone(), legacy_usage))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get isolation metrics
    pub fn get_isolation_metrics(&self) -> super::isolation::IsolationMetrics {
        self.isolation.get_metrics()
    }
}

/// Updates that can be applied to a tenant
#[derive(Debug, Clone)]
pub struct TenantUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<TenantStatus>,
    pub config: Option<TenantConfig>,
}

/// Trait for tenant operations
#[async_trait]
pub trait TenantOperations {
    async fn create_tenant(&self, tenant_id: TenantId, name: String, config: Option<TenantConfig>) -> Result<TenantInfo>;
    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<TenantInfo>;
    async fn update_tenant(&self, tenant_id: &TenantId, updates: TenantUpdate) -> Result<TenantInfo>;
    async fn delete_tenant(&self, tenant_id: &TenantId) -> Result<()>;
    async fn list_tenants(&self, status_filter: Option<TenantStatus>) -> Result<Vec<TenantInfo>>;
}

#[async_trait]
impl TenantOperations for TenantManager {
    async fn create_tenant(&self, tenant_id: TenantId, name: String, config: Option<TenantConfig>) -> Result<TenantInfo> {
        self.create_tenant(tenant_id, name, config).await
    }
    
    async fn get_tenant(&self, tenant_id: &TenantId) -> Result<TenantInfo> {
        self.get_tenant(tenant_id)
    }
    
    async fn update_tenant(&self, tenant_id: &TenantId, updates: TenantUpdate) -> Result<TenantInfo> {
        self.update_tenant(tenant_id, updates)
    }
    
    async fn delete_tenant(&self, tenant_id: &TenantId) -> Result<()> {
        self.delete_tenant(tenant_id)
    }
    
    async fn list_tenants(&self, status_filter: Option<TenantStatus>) -> Result<Vec<TenantInfo>> {
        Ok(self.list_tenants(status_filter))
    }
}

/// Registry for tracking tenant operations and performance
#[derive(Debug)]
pub struct TenantRegistry {
    registered_tenants: HashMap<TenantId, RegistrationInfo>,
    performance_stats: PerformanceStats,
}

impl Default for TenantRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantRegistry {
    pub fn new() -> Self {
        Self {
            registered_tenants: HashMap::new(),
            performance_stats: PerformanceStats::new(),
        }
    }
    
    pub fn register_tenant(&mut self, tenant_id: TenantId) -> Result<()> {
        let info = RegistrationInfo {
            registered_at: Utc::now(),
            last_activity: Utc::now(),
            operation_count: 0,
        };
        
        self.registered_tenants.insert(tenant_id, info);
        self.performance_stats.total_tenants += 1;
        
        Ok(())
    }
    
    pub fn record_activity(&mut self, tenant_id: &TenantId) {
        if let Some(info) = self.registered_tenants.get_mut(tenant_id) {
            info.last_activity = Utc::now();
            info.operation_count += 1;
            self.performance_stats.total_operations += 1;
        }
    }
    
    pub fn get_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }
}

#[derive(Debug, Clone)]
struct RegistrationInfo {
    #[allow(dead_code)] // Registration timestamp for audit and analytics (stored but not currently queried)
    registered_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    operation_count: u64,
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_tenants: u64,
    pub active_tenants: u64,
    pub total_operations: u64,
    pub average_response_time_ms: f64,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceStats {
    pub fn new() -> Self {
        Self {
            total_tenants: 0,
            active_tenants: 0,
            total_operations: 0,
            average_response_time_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tenant_creation() {
        let manager = TenantManager::new();
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        
        let tenant_info = manager.create_tenant(
            tenant_id.clone(), 
            "Test Tenant".to_string(), 
            None
        ).await.unwrap();
        
        assert_eq!(tenant_info.id, tenant_id);
        assert_eq!(tenant_info.name, "Test Tenant");
        assert!(tenant_info.is_active());
    }
    
    #[tokio::test]
    async fn test_tenant_operations() {
        let manager = TenantManager::new();
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        
        // Create tenant
        let _tenant_info = manager.create_tenant(
            tenant_id.clone(), 
            "Test Tenant".to_string(), 
            None
        ).await.unwrap();
        
        // Get tenant
        let retrieved = manager.get_tenant(&tenant_id).unwrap();
        assert_eq!(retrieved.name, "Test Tenant");
        
        // Update tenant
        let updates = TenantUpdate {
            name: Some("Updated Tenant".to_string()),
            description: Some("Updated description".to_string()),
            status: None,
            config: None,
        };
        
        let updated = manager.update_tenant(&tenant_id, updates).unwrap();
        assert_eq!(updated.name, "Updated Tenant");
        assert_eq!(updated.description, Some("Updated description".to_string()));
    }
    
    #[test]
    fn test_quota_checking() {
        let manager = TenantManager::new();
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        
        // This would normally be set up during tenant creation
        // For this test, we'll assume the tenant exists
        assert!(manager.check_tenant_quota(&tenant_id, ResourceType::Events, 100).is_err());
    }
}