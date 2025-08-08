use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Unique identifier for a tenant with validation and formatting
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(String);

impl TenantId {
    /// Create a new TenantId with validation
    pub fn new(id: String) -> Result<Self, TenantError> {
        if id.is_empty() {
            return Err(TenantError::InvalidTenantId("Tenant ID cannot be empty".to_string()));
        }
        
        if id.len() > 128 {
            return Err(TenantError::InvalidTenantId("Tenant ID too long (max 128 chars)".to_string()));
        }
        
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(TenantError::InvalidTenantId("Tenant ID must contain only alphanumeric, dash, or underscore".to_string()));
        }
        
        Ok(TenantId(id))
    }
    
    /// Generate a new UUID-based TenantId
    pub fn generate() -> Self {
        TenantId(Uuid::new_v4().to_string())
    }
    
    /// Get the raw tenant ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the database prefix for this tenant (for table/schema isolation)
    pub fn db_prefix(&self) -> String {
        format!("tenant_{}", self.0.replace('-', "_"))
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TenantId {
    type Err = TenantError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TenantId::new(s.to_string())
    }
}

/// Configuration for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub isolation_level: IsolationLevel,
    pub resource_limits: ResourceLimits,
    pub encryption_enabled: bool,
    pub audit_enabled: bool,
    pub custom_settings: HashMap<String, String>,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            isolation_level: IsolationLevel::Database,
            resource_limits: ResourceLimits::default(),
            encryption_enabled: true,
            audit_enabled: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// Tenant isolation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Database-level isolation with separate schemas/tables
    Database,
    /// Application-level isolation with tenant filtering
    Application,
    /// Row-level isolation with tenant_id columns
    Row,
}

/// Resource limits for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_events_per_day: Option<u64>,
    pub max_storage_mb: Option<u64>,
    pub max_concurrent_streams: Option<u32>,
    pub max_projections: Option<u32>,
    pub max_aggregates: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_events_per_day: Some(1_000_000),
            max_storage_mb: Some(10_000), // 10GB
            max_concurrent_streams: Some(100),
            max_projections: Some(50),
            max_aggregates: Some(100_000),
        }
    }
}

/// Complete tenant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantInfo {
    pub id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: TenantStatus,
    pub config: TenantConfig,
    pub metadata: TenantMetadata,
}

impl TenantInfo {
    pub fn new(id: TenantId, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            created_at: now,
            updated_at: now,
            status: TenantStatus::Active,
            config: TenantConfig::default(),
            metadata: TenantMetadata::default(),
        }
    }
    
    /// Check if tenant is active and can perform operations
    pub fn is_active(&self) -> bool {
        matches!(self.status, TenantStatus::Active)
    }
}

/// Tenant operational status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantStatus {
    Active,
    Suspended,
    Disabled,
    PendingDeletion,
}

/// Tenant metadata for monitoring and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMetadata {
    pub total_events: u64,
    pub total_aggregates: u64,
    pub storage_used_mb: f64,
    pub last_activity: Option<DateTime<Utc>>,
    pub performance_metrics: PerformanceMetrics,
    pub custom_metadata: HashMap<String, String>,
}

impl Default for TenantMetadata {
    fn default() -> Self {
        Self {
            total_events: 0,
            total_aggregates: 0,
            storage_used_mb: 0.0,
            last_activity: None,
            performance_metrics: PerformanceMetrics::default(),
            custom_metadata: HashMap::new(),
        }
    }
}

/// Performance metrics for tenant monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub events_per_second: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_response_time_ms: 0.0,
            events_per_second: 0.0,
            error_rate: 0.0,
            uptime_percentage: 100.0,
        }
    }
}

/// Tenant-related errors
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Invalid tenant ID: {0}")]
    InvalidTenantId(String),
    
    #[error("Tenant not found: {0}")]
    TenantNotFound(TenantId),
    
    #[error("Tenant already exists: {0}")]
    TenantAlreadyExists(TenantId),
    
    #[error("Tenant is not active: {0}")]
    TenantNotActive(TenantId),
    
    #[error("Resource limit exceeded for tenant {tenant_id}: {limit_type}")]
    ResourceLimitExceeded {
        tenant_id: TenantId,
        limit_type: String,
    },
    
    #[error("Tenant isolation violation: {0}")]
    IsolationViolation(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<TenantError> for crate::error::EventualiError {
    fn from(err: TenantError) -> Self {
        crate::error::EventualiError::Tenant(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_validation() {
        // Valid IDs
        assert!(TenantId::new("tenant1".to_string()).is_ok());
        assert!(TenantId::new("tenant-123".to_string()).is_ok());
        assert!(TenantId::new("tenant_456".to_string()).is_ok());
        
        // Invalid IDs
        assert!(TenantId::new("".to_string()).is_err());
        assert!(TenantId::new("tenant@123".to_string()).is_err());
        assert!(TenantId::new("a".repeat(129)).is_err());
    }
    
    #[test]
    fn test_tenant_info_creation() {
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        let tenant_info = TenantInfo::new(tenant_id.clone(), "Test Tenant".to_string());
        
        assert_eq!(tenant_info.id, tenant_id);
        assert_eq!(tenant_info.name, "Test Tenant");
        assert!(tenant_info.is_active());
    }
}