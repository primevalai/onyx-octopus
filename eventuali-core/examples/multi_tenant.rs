/*!
 * Multi-Tenant Example
 *
 * This example demonstrates the multi-tenancy features of Eventuali Core:
 * - Creating tenant-isolated event stores
 * - Tenant-scoped operations
 * - Resource quotas and management
 * - Tenant metadata and configuration
 */

use eventuali_core::{
    EventStoreConfig, create_event_store, Event, EventData, EventMetadata,
    TenantId, TenantInfo, TenantConfig, TenantIsolation, IsolatedEventStore,
    TenantManager, TenantOperations, TenantQuota, ResourceType,
    TenantAwareEventStorage, TenantScopedProjection, TenantProjectionManager
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 Eventuali Core - Multi-Tenant Example");
    println!("========================================");

    // Create the base event store
    let config = EventStoreConfig::sqlite(":memory:").await?;
    let base_store = create_event_store(config).await?;

    println!("\n🏗️  Setting up tenant management...");

    // Create tenant manager
    let tenant_manager = TenantManager::new(base_store.clone()).await?;

    // Example 1: Create multiple tenants
    println!("\n👥 Example 1: Creating tenants");

    let tenant_a_id = TenantId::new("acme-corp");
    let tenant_a_config = TenantConfig {
        isolation: TenantIsolation::Database,
        quotas: vec![
            TenantQuota {
                resource_type: ResourceType::Events,
                limit: 10000,
            },
            TenantQuota {
                resource_type: ResourceType::Storage,
                limit: 100 * 1024 * 1024, // 100MB
            },
        ],
        ..Default::default()
    };

    let tenant_a_info = TenantInfo {
        id: tenant_a_id.clone(),
        name: "ACME Corporation".to_string(),
        created_at: Utc::now(),
        config: tenant_a_config.clone(),
        metadata: Default::default(),
    };

    tenant_manager.create_tenant(tenant_a_info).await?;
    println!("✅ Created tenant: {} (ACME Corporation)", tenant_a_id.value());

    let tenant_b_id = TenantId::new("beta-solutions");
    let tenant_b_config = TenantConfig {
        isolation: TenantIsolation::Schema,
        quotas: vec![
            TenantQuota {
                resource_type: ResourceType::Events,
                limit: 5000,
            },
        ],
        ..Default::default()
    };

    let tenant_b_info = TenantInfo {
        id: tenant_b_id.clone(),
        name: "Beta Solutions Ltd".to_string(),
        created_at: Utc::now(),
        config: tenant_b_config.clone(),
        metadata: Default::default(),
    };

    tenant_manager.create_tenant(tenant_b_info).await?;
    println!("✅ Created tenant: {} (Beta Solutions Ltd)", tenant_b_id.value());

    // Example 2: Create isolated event stores
    println!("\n🔒 Example 2: Tenant-isolated event operations");

    let tenant_a_store = IsolatedEventStore::new(base_store.clone(), tenant_a_id.clone());
    let tenant_b_store = IsolatedEventStore::new(base_store.clone(), tenant_b_id.clone());

    // Create events for Tenant A (ACME Corp)
    let acme_events = vec![
        create_user_event("user-a1", "Alice ACME", "alice@acme.com", &tenant_a_id),
        create_user_event("user-a2", "Bob ACME", "bob@acme.com", &tenant_a_id),
        create_order_event("order-a1", "user-a1", 299.99, &tenant_a_id),
    ];

    tenant_a_store.append_events(acme_events).await?;
    println!("✅ Stored 3 events for ACME Corp");

    // Create events for Tenant B (Beta Solutions)
    let beta_events = vec![
        create_user_event("user-b1", "Carol Beta", "carol@beta.com", &tenant_b_id),
        create_order_event("order-b1", "user-b1", 149.99, &tenant_b_id),
        create_order_event("order-b2", "user-b1", 75.50, &tenant_b_id),
    ];

    tenant_b_store.append_events(beta_events).await?;
    println!("✅ Stored 3 events for Beta Solutions");

    // Example 3: Verify tenant isolation
    println!("\n🔍 Example 3: Verifying tenant isolation");

    // Load events for each tenant
    let acme_user_events = tenant_a_store.load_events("user-a1", None).await?;
    let beta_user_events = tenant_b_store.load_events("user-b1", None).await?;

    println!("📊 ACME Corp user-a1 has {} events", acme_user_events.len());
    println!("📊 Beta Solutions user-b1 has {} events", beta_user_events.len());

    // Verify cross-tenant isolation (should return 0 events)
    let cross_tenant_events = tenant_a_store.load_events("user-b1", None).await?;
    println!("🔒 ACME Corp accessing Beta user-b1: {} events (expected 0)", cross_tenant_events.len());

    // Example 4: Tenant resource monitoring
    println!("\n📈 Example 4: Tenant resource monitoring");

    let acme_metrics = tenant_manager.get_tenant_metrics(&tenant_a_id).await?;
    let beta_metrics = tenant_manager.get_tenant_metrics(&tenant_b_id).await?;

    println!("📊 ACME Corp metrics:");
    println!("   Events stored: {}", acme_metrics.event_count);
    println!("   Storage used: {} bytes", acme_metrics.storage_bytes);

    println!("📊 Beta Solutions metrics:");
    println!("   Events stored: {}", beta_metrics.event_count);
    println!("   Storage used: {} bytes", beta_metrics.storage_bytes);

    // Example 5: Tenant quota enforcement
    println!("\n⚖️  Example 5: Tenant quota management");

    // Check current quota usage
    let acme_quota_status = tenant_manager.check_quota(&tenant_a_id, &ResourceType::Events).await?;
    println!("📋 ACME Corp event quota: {}/{} ({}% used)",
        acme_quota_status.used,
        acme_quota_status.limit,
        (acme_quota_status.used as f64 / acme_quota_status.limit as f64 * 100.0) as i32
    );

    // Example 6: Tenant-scoped projections
    println!("\n🎯 Example 6: Tenant-scoped projections");

    let projection_manager = TenantProjectionManager::new();

    // Register a user count projection for each tenant
    let acme_projection = Arc::new(tokio::sync::Mutex::new(UserCountProjection::new("ACME Corp")));
    let beta_projection = Arc::new(tokio::sync::Mutex::new(UserCountProjection::new("Beta Solutions")));

    projection_manager.register_projection(
        &tenant_a_id,
        "user_count".to_string(),
        acme_projection.clone()
    ).await?;

    projection_manager.register_projection(
        &tenant_b_id,
        "user_count".to_string(),
        beta_projection.clone()
    ).await?;

    // Process events through projections
    for event in &acme_user_events {
        if let Some(projection) = projection_manager.get_projection(&tenant_a_id, "user_count").await? {
            projection.lock().await.handle_event(event).await?;
        }
    }

    for event in &beta_user_events {
        if let Some(projection) = projection_manager.get_projection(&tenant_b_id, "user_count").await? {
            projection.lock().await.handle_event(event).await?;
        }
    }

    // Display projection results
    println!("👥 ACME Corp user count: {}", acme_projection.lock().await.count);
    println!("👥 Beta Solutions user count: {}", beta_projection.lock().await.count);

    // Example 7: Tenant lifecycle management
    println!("\n🔄 Example 7: Tenant lifecycle operations");

    // List all tenants
    let all_tenants = tenant_manager.list_tenants().await?;
    println!("📋 Total tenants: {}", all_tenants.len());

    for tenant in &all_tenants {
        println!("   - {} ({})", tenant.name, tenant.id.value());
    }

    // Update tenant configuration
    let mut updated_config = tenant_a_config.clone();
    updated_config.quotas.push(TenantQuota {
        resource_type: ResourceType::Projections,
        limit: 50,
    });

    tenant_manager.update_tenant_config(&tenant_a_id, updated_config).await?;
    println!("✅ Updated ACME Corp configuration with projection quota");

    // Example 8: Tenant archival (soft delete)
    println!("\n📦 Example 8: Tenant archival");

    tenant_manager.archive_tenant(&tenant_b_id).await?;
    println!("📦 Archived Beta Solutions tenant (data preserved but inaccessible)");

    // Verify archived tenant is not accessible
    let active_tenants = tenant_manager.list_active_tenants().await?;
    println!("📋 Active tenants: {}", active_tenants.len());

    println!("\n🎉 Multi-tenant example completed!");
    println!("💡 Key takeaways:");
    println!("   - Each tenant has completely isolated data");
    println!("   - Resource quotas prevent tenant abuse");
    println!("   - Projections can be scoped per tenant");
    println!("   - Tenant lifecycle is fully managed");
    println!("   - Different isolation levels (database, schema, logical)");
    println!("   - Comprehensive monitoring and metrics");

    Ok(())
}

/// Helper function to create user events with tenant context
fn create_user_event(user_id: &str, name: &str, email: &str, tenant_id: &TenantId) -> Event {
    let mut metadata = EventMetadata::default();
    metadata.tenant_id = Some(tenant_id.clone());

    Event {
        id: Uuid::new_v4(),
        aggregate_id: user_id.to_string(),
        aggregate_type: "User".to_string(),
        event_type: "UserRegistered".to_string(),
        event_version: 1,
        aggregate_version: 1,
        data: EventData::from_json(&json!({
            "user_id": user_id,
            "name": name,
            "email": email,
            "tenant_id": tenant_id.value(),
            "registered_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata,
        timestamp: Utc::now(),
    }
}

/// Helper function to create order events with tenant context
fn create_order_event(order_id: &str, user_id: &str, amount: f64, tenant_id: &TenantId) -> Event {
    let mut metadata = EventMetadata::default();
    metadata.tenant_id = Some(tenant_id.clone());

    Event {
        id: Uuid::new_v4(),
        aggregate_id: order_id.to_string(),
        aggregate_type: "Order".to_string(),
        event_type: "OrderCreated".to_string(),
        event_version: 1,
        aggregate_version: 1,
        data: EventData::from_json(&json!({
            "order_id": order_id,
            "user_id": user_id,
            "amount": amount,
            "tenant_id": tenant_id.value(),
            "created_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata,
        timestamp: Utc::now(),
    }
}

/// Simple projection to count users per tenant
use async_trait::async_trait;

struct UserCountProjection {
    tenant_name: String,
    count: i32,
}

impl UserCountProjection {
    fn new(tenant_name: &str) -> Self {
        Self {
            tenant_name: tenant_name.to_string(),
            count: 0,
        }
    }
}

#[async_trait]
impl TenantScopedProjection for UserCountProjection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event.event_type == "UserRegistered" {
            self.count += 1;
            println!("   🎯 {} user count updated: {}", self.tenant_name, self.count);
        }
        Ok(())
    }

    fn tenant_id(&self) -> &TenantId {
        // In a real implementation, this would be stored with the projection
        // For this example, we'll use a placeholder
        static PLACEHOLDER: TenantId = TenantId::new("placeholder");
        &PLACEHOLDER
    }
}