use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc, Duration};

use super::tenant::{TenantId, ResourceLimits};
use crate::error::{EventualiError, Result};

/// Resource quota management for tenants
pub struct TenantQuota {
    tenant_id: TenantId,
    limits: ResourceLimits,
    tracker: Arc<RwLock<ResourceTracker>>,
}

impl TenantQuota {
    pub fn new(tenant_id: TenantId, limits: ResourceLimits) -> Self {
        Self {
            tenant_id,
            limits,
            tracker: Arc::new(RwLock::new(ResourceTracker::new())),
        }
    }
    
    /// Check if an operation would exceed quotas
    pub fn check_quota(&self, resource_type: ResourceType, amount: u64) -> Result<()> {
        let tracker = self.tracker.read().unwrap();
        
        match resource_type {
            ResourceType::Events => {
                if let Some(limit) = self.limits.max_events_per_day {
                    let current_daily = tracker.get_daily_events();
                    if current_daily + amount > limit {
                        return Err(EventualiError::from(QuotaExceeded {
                            tenant_id: self.tenant_id.clone(),
                            resource_type: "daily_events".to_string(),
                            current_usage: current_daily,
                            limit,
                            attempted: amount,
                        }));
                    }
                }
            },
            ResourceType::Storage => {
                if let Some(limit_mb) = self.limits.max_storage_mb {
                    let current_mb = tracker.storage_used_mb as u64;
                    if current_mb + amount > limit_mb {
                        return Err(EventualiError::from(QuotaExceeded {
                            tenant_id: self.tenant_id.clone(),
                            resource_type: "storage_mb".to_string(),
                            current_usage: current_mb,
                            limit: limit_mb,
                            attempted: amount,
                        }));
                    }
                }
            },
            ResourceType::Streams => {
                if let Some(limit) = self.limits.max_concurrent_streams {
                    let current = tracker.concurrent_streams as u64;
                    if current + amount > limit as u64 {
                        return Err(EventualiError::from(QuotaExceeded {
                            tenant_id: self.tenant_id.clone(),
                            resource_type: "concurrent_streams".to_string(),
                            current_usage: current,
                            limit: limit as u64,
                            attempted: amount,
                        }));
                    }
                }
            },
            ResourceType::Projections => {
                if let Some(limit) = self.limits.max_projections {
                    let current = tracker.total_projections as u64;
                    if current + amount > limit as u64 {
                        return Err(EventualiError::from(QuotaExceeded {
                            tenant_id: self.tenant_id.clone(),
                            resource_type: "projections".to_string(),
                            current_usage: current,
                            limit: limit as u64,
                            attempted: amount,
                        }));
                    }
                }
            },
            ResourceType::Aggregates => {
                if let Some(limit) = self.limits.max_aggregates {
                    let current = tracker.total_aggregates;
                    if current + amount > limit {
                        return Err(EventualiError::from(QuotaExceeded {
                            tenant_id: self.tenant_id.clone(),
                            resource_type: "aggregates".to_string(),
                            current_usage: current,
                            limit,
                            attempted: amount,
                        }));
                    }
                }
            },
        }
        
        Ok(())
    }
    
    /// Record resource usage
    pub fn record_usage(&self, resource_type: ResourceType, amount: u64) {
        let mut tracker = self.tracker.write().unwrap();
        tracker.record_usage(resource_type, amount);
    }
    
    /// Get current usage statistics
    pub fn get_usage(&self) -> ResourceUsage {
        let tracker = self.tracker.read().unwrap();
        ResourceUsage {
            tenant_id: self.tenant_id.clone(),
            daily_events: tracker.get_daily_events(),
            storage_used_mb: tracker.storage_used_mb,
            concurrent_streams: tracker.concurrent_streams,
            total_projections: tracker.total_projections,
            total_aggregates: tracker.total_aggregates,
            limits: self.limits.clone(),
            last_updated: tracker.last_updated,
        }
    }
    
    /// Reset daily counters (typically called by a background task)
    pub fn reset_daily_counters(&self) {
        let mut tracker = self.tracker.write().unwrap();
        tracker.reset_daily_counters();
    }
}

/// Types of resources that can be tracked and limited
#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Events,
    Storage,
    Streams,
    Projections,
    Aggregates,
}

/// Internal resource tracker
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    daily_events: u64,
    daily_reset_time: DateTime<Utc>,
    storage_used_mb: f64,
    concurrent_streams: u32,
    total_projections: u32,
    total_aggregates: u64,
    last_updated: DateTime<Utc>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            daily_events: 0,
            daily_reset_time: now,
            storage_used_mb: 0.0,
            concurrent_streams: 0,
            total_projections: 0,
            total_aggregates: 0,
            last_updated: now,
        }
    }
    
    pub fn record_usage(&mut self, resource_type: ResourceType, amount: u64) {
        self.last_updated = Utc::now();
        
        match resource_type {
            ResourceType::Events => {
                self.ensure_daily_counter_fresh();
                self.daily_events += amount;
            },
            ResourceType::Storage => {
                self.storage_used_mb += amount as f64;
            },
            ResourceType::Streams => {
                self.concurrent_streams += amount as u32;
            },
            ResourceType::Projections => {
                self.total_projections += amount as u32;
            },
            ResourceType::Aggregates => {
                self.total_aggregates += amount;
            },
        }
    }
    
    pub fn get_daily_events(&self) -> u64 {
        if self.is_daily_counter_stale() {
            0 // Reset if stale
        } else {
            self.daily_events
        }
    }
    
    pub fn reset_daily_counters(&mut self) {
        self.daily_events = 0;
        self.daily_reset_time = Utc::now();
    }
    
    fn ensure_daily_counter_fresh(&mut self) {
        if self.is_daily_counter_stale() {
            self.reset_daily_counters();
        }
    }
    
    fn is_daily_counter_stale(&self) -> bool {
        let now = Utc::now();
        now.signed_duration_since(self.daily_reset_time) >= Duration::days(1)
    }
}

/// Current resource usage for a tenant
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub tenant_id: TenantId,
    pub daily_events: u64,
    pub storage_used_mb: f64,
    pub concurrent_streams: u32,
    pub total_projections: u32,
    pub total_aggregates: u64,
    pub limits: ResourceLimits,
    pub last_updated: DateTime<Utc>,
}

impl ResourceUsage {
    /// Calculate utilization percentage for a resource
    pub fn utilization_percentage(&self, resource_type: ResourceType) -> Option<f64> {
        match resource_type {
            ResourceType::Events => {
                self.limits.max_events_per_day.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.daily_events as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Storage => {
                self.limits.max_storage_mb.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.storage_used_mb / limit as f64) * 100.0 }
                })
            },
            ResourceType::Streams => {
                self.limits.max_concurrent_streams.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.concurrent_streams as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Projections => {
                self.limits.max_projections.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.total_projections as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Aggregates => {
                self.limits.max_aggregates.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.total_aggregates as f64 / limit as f64) * 100.0 }
                })
            },
        }
    }
    
    /// Check if any resource is near its limit (>80%)
    pub fn has_resources_near_limit(&self) -> bool {
        [
            ResourceType::Events,
            ResourceType::Storage,
            ResourceType::Streams,
            ResourceType::Projections,
            ResourceType::Aggregates,
        ].iter().any(|&resource_type| {
            self.utilization_percentage(resource_type)
                .map_or(false, |percentage| percentage > 80.0)
        })
    }
}

/// Error type for quota violations
#[derive(Debug, thiserror::Error)]
#[error("Quota exceeded for tenant {tenant_id}: {resource_type} - current: {current_usage}, limit: {limit}, attempted: {attempted}")]
pub struct QuotaExceeded {
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub current_usage: u64,
    pub limit: u64,
    pub attempted: u64,
}

impl From<QuotaExceeded> for crate::error::EventualiError {
    fn from(err: QuotaExceeded) -> Self {
        crate::error::EventualiError::Tenant(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quota_enforcement() {
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        let limits = ResourceLimits {
            max_events_per_day: Some(1000),
            max_storage_mb: Some(100),
            max_concurrent_streams: Some(10),
            max_projections: Some(5),
            max_aggregates: Some(50),
        };
        
        let quota = TenantQuota::new(tenant_id, limits);
        
        // Should allow usage within limits
        assert!(quota.check_quota(ResourceType::Events, 500).is_ok());
        
        // Record some usage
        quota.record_usage(ResourceType::Events, 500);
        
        // Should still allow more usage within limits
        assert!(quota.check_quota(ResourceType::Events, 400).is_ok());
        
        // Should reject usage that exceeds limits
        assert!(quota.check_quota(ResourceType::Events, 600).is_err());
    }
    
    #[test]
    fn test_resource_usage_calculation() {
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        let limits = ResourceLimits {
            max_events_per_day: Some(1000),
            ..Default::default()
        };
        
        let quota = TenantQuota::new(tenant_id, limits);
        quota.record_usage(ResourceType::Events, 800);
        
        let usage = quota.get_usage();
        let utilization = usage.utilization_percentage(ResourceType::Events).unwrap();
        
        assert_eq!(utilization, 80.0);
        assert!(usage.has_resources_near_limit());
    }
}