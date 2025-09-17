/*!
 * Streaming Projections Example
 *
 * This example demonstrates real-time event streaming capabilities:
 * - Creating event streams
 * - Building projections that react to events
 * - Handling multiple event types
 * - Error handling in stream processing
 */

use eventuali_core::{
    EventStoreConfig, create_event_store, Event, EventData, EventMetadata,
    InMemoryEventStreamer, SubscriptionBuilder, Projection, ProjectionProcessor,
    EventStreamProcessor, StreamEvent
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Eventuali Core - Streaming Projections Example");
    println!("================================================");

    // Set up event store and streamer
    let config = EventStoreConfig::sqlite(":memory:").await?;
    let store = create_event_store(config).await?;
    let streamer = Arc::new(InMemoryEventStreamer::new());

    println!("\n🏗️  Setting up projections...");

    // Example 1: User Statistics Projection
    let user_stats = Arc::new(Mutex::new(UserStatisticsProjection::new()));
    let user_stats_clone = user_stats.clone();

    // Example 2: Activity Log Projection
    let activity_log = Arc::new(Mutex::new(ActivityLogProjection::new()));
    let activity_log_clone = activity_log.clone();

    // Create subscriptions for different event types
    let user_subscription = SubscriptionBuilder::new()
        .aggregate_type("User".to_string())
        .build();

    let order_subscription = SubscriptionBuilder::new()
        .aggregate_type("Order".to_string())
        .build();

    let all_events_subscription = SubscriptionBuilder::new()
        .build(); // Subscribe to all events

    println!("✅ Created 3 subscriptions (User events, Order events, All events)");

    // Start projection processors
    let streamer_clone1 = streamer.clone();
    let user_processor = tokio::spawn(async move {
        let mut receiver = streamer_clone1.subscribe(user_subscription).await.unwrap();

        while let Ok(stream_event) = receiver.recv().await {
            if let Err(e) = user_stats_clone.lock().unwrap().handle_event(&stream_event.event).await {
                eprintln!("Error in user stats projection: {}", e);
            }
        }
    });

    let streamer_clone2 = streamer.clone();
    let activity_processor = tokio::spawn(async move {
        let mut receiver = streamer_clone2.subscribe(all_events_subscription).await.unwrap();

        while let Ok(stream_event) = receiver.recv().await {
            if let Err(e) = activity_log_clone.lock().unwrap().handle_event(&stream_event.event).await {
                eprintln!("Error in activity log projection: {}", e);
            }
        }
    });

    // Give processors time to start
    sleep(Duration::from_millis(100)).await;

    println!("\n📝 Publishing events to demonstrate streaming...");

    // Publish some events
    let events = vec![
        create_user_registered_event("user-1", "Alice", "alice@example.com"),
        create_user_registered_event("user-2", "Bob", "bob@example.com"),
        create_order_created_event("order-1", "user-1", 99.99),
        create_user_email_changed_event("user-1", "alice@example.com", "alice.new@example.com", 2),
        create_order_shipped_event("order-1", 2),
        create_user_registered_event("user-3", "Carol", "carol@example.com"),
        create_order_created_event("order-2", "user-2", 149.99),
    ];

    // Store events and publish to stream
    for event in events {
        store.append_events(vec![event.clone()]).await?;
        streamer.publish_event(StreamEvent { event }).await?;

        // Small delay to see the streaming in action
        sleep(Duration::from_millis(200)).await;
    }

    // Give projections time to process all events
    sleep(Duration::from_millis(500)).await;

    // Display results
    println!("\n📊 Projection Results:");
    println!("======================");

    // User Statistics
    {
        let stats = user_stats.lock().unwrap();
        println!("\n👥 User Statistics:");
        println!("   Total Users: {}", stats.total_users);
        println!("   Email Changes: {}", stats.email_changes);

        println!("   User Details:");
        for (user_id, user) in &stats.users {
            println!("     {} -> {} ({})", user_id, user.name, user.email);
        }
    }

    // Activity Log
    {
        let log = activity_log.lock().unwrap();
        println!("\n📋 Activity Log ({} activities):", log.activities.len());
        for activity in &log.activities {
            println!("   [{}] {} - {}", activity.timestamp.format("%H:%M:%S"), activity.event_type, activity.description);
        }
    }

    // Example 3: Demonstrate error handling
    println!("\n🚨 Example 3: Error handling in projections");

    let error_projection = Arc::new(Mutex::new(ErrorDemoProjection::new()));
    let error_projection_clone = error_projection.clone();

    let error_subscription = SubscriptionBuilder::new()
        .event_type("OrderCreated".to_string())
        .build();

    let streamer_clone3 = streamer.clone();
    let error_processor = tokio::spawn(async move {
        let mut receiver = streamer_clone3.subscribe(error_subscription).await.unwrap();

        while let Ok(stream_event) = receiver.recv().await {
            if let Err(e) = error_projection_clone.lock().unwrap().handle_event(&stream_event.event).await {
                println!("   ⚠️  Handled projection error: {}", e);
            }
        }
    });

    sleep(Duration::from_millis(100)).await;

    // Trigger an error condition
    let problematic_event = create_order_created_event("bad-order", "user-999", -50.0); // Negative amount
    streamer.publish_event(StreamEvent { event: problematic_event }).await?;

    sleep(Duration::from_millis(200)).await;

    // Clean up
    user_processor.abort();
    activity_processor.abort();
    error_processor.abort();

    println!("\n🎉 Streaming example completed!");
    println!("💡 Key takeaways:");
    println!("   - Projections process events in real-time");
    println!("   - Multiple projections can subscribe to the same events");
    println!("   - Event filtering allows selective processing");
    println!("   - Error handling prevents one projection from affecting others");
    println!("   - Streaming enables reactive architectures");

    Ok(())
}

// Domain Events
fn create_user_registered_event(user_id: &str, name: &str, email: &str) -> Event {
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
            "registered_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    }
}

fn create_user_email_changed_event(user_id: &str, old_email: &str, new_email: &str, version: i64) -> Event {
    Event {
        id: Uuid::new_v4(),
        aggregate_id: user_id.to_string(),
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

fn create_order_created_event(order_id: &str, user_id: &str, amount: f64) -> Event {
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
            "created_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    }
}

fn create_order_shipped_event(order_id: &str, version: i64) -> Event {
    Event {
        id: Uuid::new_v4(),
        aggregate_id: order_id.to_string(),
        aggregate_type: "Order".to_string(),
        event_type: "OrderShipped".to_string(),
        event_version: 1,
        aggregate_version: version,
        data: EventData::from_json(&json!({
            "order_id": order_id,
            "shipped_at": Utc::now().to_rfc3339()
        })).unwrap(),
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    }
}

// Projection implementations
#[derive(Debug)]
struct UserInfo {
    name: String,
    email: String,
}

struct UserStatisticsProjection {
    total_users: i32,
    email_changes: i32,
    users: HashMap<String, UserInfo>,
}

impl UserStatisticsProjection {
    fn new() -> Self {
        Self {
            total_users: 0,
            email_changes: 0,
            users: HashMap::new(),
        }
    }
}

#[async_trait]
impl Projection for UserStatisticsProjection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.event_type.as_str() {
            "UserRegistered" => {
                self.total_users += 1;
                let data: serde_json::Value = serde_json::from_str(&event.data.to_json())?;

                self.users.insert(
                    event.aggregate_id.clone(),
                    UserInfo {
                        name: data["name"].as_str().unwrap_or("Unknown").to_string(),
                        email: data["email"].as_str().unwrap_or("unknown@example.com").to_string(),
                    }
                );

                println!("   📊 User registered: {} (Total: {})", event.aggregate_id, self.total_users);
            }
            "UserEmailChanged" => {
                self.email_changes += 1;
                let data: serde_json::Value = serde_json::from_str(&event.data.to_json())?;

                if let Some(user) = self.users.get_mut(&event.aggregate_id) {
                    user.email = data["new_email"].as_str().unwrap_or("unknown@example.com").to_string();
                }

                println!("   📊 Email changed for: {} (Total changes: {})", event.aggregate_id, self.email_changes);
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Activity {
    timestamp: chrono::DateTime<Utc>,
    event_type: String,
    description: String,
}

struct ActivityLogProjection {
    activities: Vec<Activity>,
}

impl ActivityLogProjection {
    fn new() -> Self {
        Self {
            activities: Vec::new(),
        }
    }
}

#[async_trait]
impl Projection for ActivityLogProjection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let description = match event.event_type.as_str() {
            "UserRegistered" => format!("New user {} registered", event.aggregate_id),
            "UserEmailChanged" => format!("User {} changed email", event.aggregate_id),
            "OrderCreated" => format!("Order {} created", event.aggregate_id),
            "OrderShipped" => format!("Order {} shipped", event.aggregate_id),
            _ => format!("Unknown event type: {}", event.event_type),
        };

        self.activities.push(Activity {
            timestamp: event.timestamp,
            event_type: event.event_type.clone(),
            description,
        });

        println!("   📝 Activity logged: {}", description);
        Ok(())
    }
}

struct ErrorDemoProjection {
    processed_orders: i32,
}

impl ErrorDemoProjection {
    fn new() -> Self {
        Self {
            processed_orders: 0,
        }
    }
}

#[async_trait]
impl Projection for ErrorDemoProjection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event.event_type == "OrderCreated" {
            let data: serde_json::Value = serde_json::from_str(&event.data.to_json())?;
            let amount = data["amount"].as_f64().unwrap_or(0.0);

            // Simulate validation error
            if amount < 0.0 {
                return Err(format!("Invalid order amount: {}", amount).into());
            }

            self.processed_orders += 1;
            println!("   ✅ Order processed successfully: {} (Amount: ${})", event.aggregate_id, amount);
        }
        Ok(())
    }
}