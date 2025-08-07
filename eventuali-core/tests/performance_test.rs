use eventuali_core::{
    Event, EventData, EventStoreConfig, create_event_store,
    streaming::{InMemoryEventStreamer, EventStreamer, SubscriptionBuilder}
};
use std::time::Instant;
use tokio;
use uuid::Uuid;

#[tokio::test]
async fn test_event_store_performance() {
    let config = EventStoreConfig::sqlite(":memory:".to_string());
    let store = create_event_store(config).await.unwrap();
    
    let aggregate_id = Uuid::new_v4().to_string();
    let event_count = 1000;
    
    // Create test events
    let mut events = Vec::new();
    for i in 0..event_count {
        let event_data = EventData::from_json(&serde_json::json!({
            "user_id": format!("user-{}", i),
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i)
        })).unwrap();
        
        let event = Event::new(
            format!("{}-{}", aggregate_id, i),
            "User".to_string(),
            "UserRegistered".to_string(),
            1,
            i as i64 + 1,
            event_data,
        );
        events.push(event);
    }
    
    // Test bulk save performance
    let start = Instant::now();
    store.save_events(events.clone()).await.unwrap();
    let save_duration = start.elapsed();
    
    let events_per_sec = event_count as f64 / save_duration.as_secs_f64();
    println!("Saved {} events in {:?} ({:.2} events/sec)", event_count, save_duration, events_per_sec);
    
    // Should achieve at least 1000 events/sec for SQLite
    assert!(events_per_sec > 1000.0, "Performance below threshold: {} events/sec", events_per_sec);
    
    // Test individual load performance
    let start = Instant::now();
    for i in 0..10 {
        let test_aggregate_id = format!("{}-{}", aggregate_id, i);
        let loaded_events = store.load_events(&test_aggregate_id, None).await.unwrap();
        assert_eq!(loaded_events.len(), 1);
    }
    let load_duration = start.elapsed();
    
    println!("Loaded 10 individual aggregates in {:?}", load_duration);
    assert!(load_duration.as_millis() < 100, "Load performance too slow: {:?}", load_duration);
}

#[tokio::test]
async fn test_streaming_performance() {
    let streamer = InMemoryEventStreamer::new(1000);
    let subscription = SubscriptionBuilder::new()
        .with_id("perf-test".to_string())
        .build();
    
    let receiver = streamer.subscribe(subscription).await.unwrap();
    
    let event_count = 5000;
    let start = Instant::now();
    
    // Publish events
    for i in 0..event_count {
        let event_data = EventData::from_json(&serde_json::json!({
            "sequence": i,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })).unwrap();
        
        let event = Event::new(
            format!("stream-{}", i % 10), // 10 different streams
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            i as i64 + 1,
            event_data,
        );
        
        streamer.publish_event(event, i as u64 + 1, i as u64 + 1).await.unwrap();
    }
    
    let publish_duration = start.elapsed();
    let events_per_sec = event_count as f64 / publish_duration.as_secs_f64();
    
    println!("Published {} events in {:?} ({:.2} events/sec)", event_count, publish_duration, events_per_sec);
    
    // Should achieve at least 10000 events/sec for in-memory streaming
    assert!(events_per_sec > 10000.0, "Streaming performance below threshold: {} events/sec", events_per_sec);
}

#[tokio::test]
async fn test_memory_efficiency() {
    let config = EventStoreConfig::sqlite(":memory:".to_string());
    let store = create_event_store(config).await.unwrap();
    
    // Create many events to test memory usage
    let event_count = 10000;
    let mut events = Vec::new();
    
    for i in 0..event_count {
        let event_data = EventData::from_json(&serde_json::json!({
            "id": i,
            "data": format!("test-data-{}", i),
            "metadata": {
                "source": "performance-test",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        })).unwrap();
        
        let event = Event::new(
            format!("aggregate-{}", i % 100), // 100 different aggregates
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            (i / 100) as i64 + 1, // Multiple events per aggregate
            event_data,
        );
        events.push(event);
    }
    
    // Save all events
    let start = Instant::now();
    store.save_events(events).await.unwrap();
    let duration = start.elapsed();
    
    println!("Processed {} events in {:?}", event_count, duration);
    
    // Test aggregate loading
    let load_start = Instant::now();
    let loaded_events = store.load_events(&"aggregate-50".to_string(), None).await.unwrap();
    let load_duration = load_start.elapsed();
    
    println!("Loaded {} events for aggregate in {:?}", loaded_events.len(), load_duration);
    assert!(loaded_events.len() == 100); // Should have 100 events per aggregate
}

#[tokio::test]
async fn test_concurrent_performance() {
    use std::sync::Arc;
    use tokio::task;
    
    let config = EventStoreConfig::sqlite(":memory:".to_string());
    let store = Arc::new(create_event_store(config).await.unwrap());
    
    let concurrent_tasks = 10;
    let events_per_task = 100;
    
    let start = Instant::now();
    
    // Spawn concurrent tasks
    let mut handles = Vec::new();
    for task_id in 0..concurrent_tasks {
        let store_clone = store.clone();
        
        let handle = task::spawn(async move {
            let mut events = Vec::new();
            for i in 0..events_per_task {
                let event_data = EventData::from_json(&serde_json::json!({
                    "task_id": task_id,
                    "event_id": i,
                    "data": format!("concurrent-test-{}-{}", task_id, i)
                })).unwrap();
                
                let event = Event::new(
                    format!("concurrent-{}-{}", task_id, i),
                    "ConcurrentTest".to_string(),
                    "ConcurrentEvent".to_string(),
                    1,
                    i as i64 + 1,
                    event_data,
                );
                events.push(event);
            }
            
            store_clone.save_events(events).await.unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start.elapsed();
    let total_events = concurrent_tasks * events_per_task;
    let events_per_sec = total_events as f64 / duration.as_secs_f64();
    
    println!("Processed {} events concurrently across {} tasks in {:?} ({:.2} events/sec)", 
             total_events, concurrent_tasks, duration, events_per_sec);
    
    // Should handle concurrent load efficiently
    assert!(events_per_sec > 1000.0, "Concurrent performance below threshold: {} events/sec", events_per_sec);
}