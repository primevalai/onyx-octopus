/*!
 * Basic Event Store Example
 *
 * This example demonstrates the fundamental event sourcing operations:
 * - Creating an event store
 * - Storing events
 * - Loading events by aggregate
 * - Basic error handling
 */

use eventuali_core::{
    EventStoreConfig, create_event_store, Event, EventData, EventMetadata
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Eventuali Core - Basic Event Store Example");
    println!("==============================================");

    // Create an in-memory SQLite event store for this example
    println!("\n📦 Creating event store...");
    let config = EventStoreConfig::sqlite(":memory:".to_string());
    let store = create_event_store(config).await?;
    println!("✅ Event store created successfully");

    // Example 1: Create and store a single event
    println!("\n📝 Example 1: Storing a single event");
    let user_id = "user-123";
    let registration_event = create_user_registered_event(
        user_id.to_string(),
        "Alice Johnson".to_string(),
        "alice@example.com".to_string(),
    );

    store.save_events(vec![registration_event.clone()]).await?;
    println!("✅ Stored UserRegistered event for user: {}", user_id);

    // Example 2: Store multiple events for the same aggregate
    println!("\n📝 Example 2: Storing multiple events");
    let email_change_event = create_user_email_changed_event(
        user_id.to_string(),
        "alice@example.com".to_string(),
        "alice.johnson@newcompany.com".to_string(),
        2,
    );

    let profile_update_event = create_user_profile_updated_event(
        user_id.to_string(),
        "Alice J. Johnson".to_string(),
        3,
    );

    store.save_events(vec![email_change_event, profile_update_event]).await?;
    println!("✅ Stored 2 additional events for user: {}", user_id);

    // Example 3: Load all events for an aggregate
    println!("\n📖 Example 3: Loading events for aggregate");
    let events = store.load_events(&user_id.to_string(), None).await?;
    println!("📚 Loaded {} events for user: {}", events.len(), user_id);

    for (i, event) in events.iter().enumerate() {
        println!("  {}. {} (v{}) at {}",
            i + 1,
            event.event_type,
            event.aggregate_version,
            event.timestamp.format("%Y-%m-%d %H:%M:%S")
        );
    }

    // Example 4: Load events from a specific version
    println!("\n📖 Example 4: Loading events from version 2");
    let recent_events = store.load_events(&user_id.to_string(), Some(2)).await?;
    println!("📚 Loaded {} events from version 2+", recent_events.len());

    // Example 5: Try to load events for non-existent aggregate
    println!("\n📖 Example 5: Loading events for non-existent aggregate");
    let missing_events = store.load_events(&"non-existent-user".to_string(), None).await?;
    println!("📚 Loaded {} events for non-existent user (expected 0)", missing_events.len());

    // Example 6: Create events for multiple aggregates
    println!("\n📝 Example 6: Events across multiple aggregates");
    let user2_events = vec![
        create_user_registered_event(
            "user-456".to_string(),
            "Bob Smith".to_string(),
            "bob@example.com".to_string(),
        ),
        create_user_registered_event(
            "user-789".to_string(),
            "Carol Davis".to_string(),
            "carol@example.com".to_string(),
        ),
    ];

    store.save_events(user2_events).await?;
    println!("✅ Created events for 2 additional users");

    // Example 7: Demonstrate aggregate isolation
    println!("\n🔍 Example 7: Aggregate isolation verification");
    let user2_events = store.load_events(&"user-456".to_string(), None).await?;
    let user3_events = store.load_events(&"user-789".to_string(), None).await?;

    println!("📚 User 456 has {} events", user2_events.len());
    println!("📚 User 789 has {} events", user3_events.len());
    println!("✅ Events are properly isolated per aggregate");

    println!("\n🎉 Example completed successfully!");
    println!("💡 Key takeaways:");
    println!("   - Events are stored and retrieved efficiently");
    println!("   - Each aggregate maintains its own event stream");
    println!("   - Version-based loading enables incremental processing");
    println!("   - In-memory SQLite is perfect for testing and development");

    Ok(())
}

/// Helper function to create a UserRegistered event
fn create_user_registered_event(user_id: String, name: String, email: String) -> Event {
    Event {
        id: Uuid::new_v4(),
        aggregate_id: user_id.clone(),
        aggregate_type: "User".to_string(),
        event_type: "UserRegistered".to_string(),
        event_version: 1,
        aggregate_version: 1,
        data: EventData::from_json(&json!({
            "user_id": user_id,
            "name": name,
            "email": email,
            "registered_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    }
}

/// Helper function to create a UserEmailChanged event
fn create_user_email_changed_event(
    user_id: String,
    old_email: String,
    new_email: String,
    version: i64
) -> Event {
    Event {
        id: Uuid::new_v4(),
        aggregate_id: user_id.clone(),
        aggregate_type: "User".to_string(),
        event_type: "UserEmailChanged".to_string(),
        event_version: 1,
        aggregate_version: version,
        data: EventData::from_json(&json!({
            "user_id": user_id,
            "old_email": old_email,
            "new_email": new_email,
            "changed_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    }
}

/// Helper function to create a UserProfileUpdated event
fn create_user_profile_updated_event(
    user_id: String,
    new_name: String,
    version: i64
) -> Event {
    Event {
        id: Uuid::new_v4(),
        aggregate_id: user_id.clone(),
        aggregate_type: "User".to_string(),
        event_type: "UserProfileUpdated".to_string(),
        event_version: 1,
        aggregate_version: version,
        data: EventData::from_json(&json!({
            "user_id": user_id,
            "name": new_name,
            "updated_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    }
}