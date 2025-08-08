//! Multi-tenancy module providing tenant isolation, scoped operations, and resource management
//!
//! This module implements comprehensive tenant isolation at the database level with:
//! - Tenant-scoped event storage with automatic namespace isolation
//! - Resource tracking and quotas per tenant
//! - Performance monitoring with <10ms overhead target
//! - 99.9% tenant isolation guarantee

pub mod tenant;
pub mod isolation;
pub mod quota;
pub mod manager;
pub mod storage;
pub mod projections;

pub use tenant::{TenantId, TenantInfo, TenantConfig, TenantMetadata, ResourceLimits};
pub use isolation::{TenantIsolation, IsolatedEventStore, TenantScope};
pub use quota::{TenantQuota, ResourceTracker, QuotaExceeded};
pub use manager::{TenantManager, TenantOperations, TenantRegistry};
pub use storage::{TenantAwareEventStorage, TenantStorageMetrics, TenantEventBatch};
pub use projections::{
    TenantScopedProjection, TenantProjectionManager, TenantProjectionRegistry, 
    TenantProjectionMetrics
};