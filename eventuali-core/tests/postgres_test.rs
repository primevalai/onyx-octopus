use eventuali_core::{
    Event, EventData, EventStoreConfig, create_event_store
};
use uuid::Uuid;

// Note: These tests require PostgreSQL to be running
// They are conditional and will be skipped if PostgreSQL is not available

const POSTGRES_URL: &str = "postgresql://eventuali:eventuali@localhost/eventuali_test";

async fn setup_postgres_test_db() -> Option<Box<dyn eventuali_core::EventStore + Send + Sync>> {
    // Try to connect to PostgreSQL - skip tests if not available
    let config = EventStoreConfig::postgres(POSTGRES_URL.to_string());
    match create_event_store(config).await {
        Ok(store) => Some(store),
        Err(_) => {
            println!("PostgreSQL not available, skipping PostgreSQL tests");
            None
        }
    }
}

#[tokio::test]
async fn test_postgres_basic_operations() {
    let store = match setup_postgres_test_db().await {
        Some(store) => store,
        None => return, // Skip test
    };
    
    let aggregate_id = format!("postgres-test-{}", Uuid::new_v4());
    
    // Create test event
    let event_data = EventData::from_json(&serde_json::json!({
        "name": "PostgreSQL User",
        "email": "pg_user@example.com",
        "created_at": chrono::Utc::now().to_rfc3339()
    })).unwrap();
    
    let event = Event::new(
        aggregate_id.clone(),
        "User".to_string(),
        "UserRegistered".to_string(),
        1,
        1,
        event_data,
    );
    
    // Test save
    store.save_events(vec![event.clone()]).await.unwrap();
    
    // Test load
    let loaded_events = store.load_events(&aggregate_id, None).await.unwrap();
    assert_eq!(loaded_events.len(), 1);
    assert_eq!(loaded_events[0].aggregate_id, aggregate_id);
    assert_eq!(loaded_events[0].event_type, "UserRegistered");
    
    // Test version
    let version = store.get_aggregate_version(&aggregate_id).await.unwrap();
    assert_eq!(version, Some(1));
    
    println!("✓ PostgreSQL basic operations test passed");
}

#[tokio::test]
async fn test_postgres_multiple_events() {
    let store = match setup_postgres_test_db().await {
        Some(store) => store,
        None => return, // Skip test
    };
    
    let aggregate_id = format!("postgres-multi-{}", Uuid::new_v4());
    let mut events = Vec::new();
    
    // Create multiple events for the same aggregate
    for i in 1..=5 {
        let event_data = EventData::from_json(&serde_json::json!({
            "sequence": i,
            "action": format!("Action {}", i),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })).unwrap();
        
        let event = Event::new(
            aggregate_id.clone(),
            "TestAggregate".to_string(),
            "TestEvent".to_string(),
            1,
            i,
            event_data,
        );
        events.push(event);
    }
    
    // Save all events
    store.save_events(events.clone()).await.unwrap();
    
    // Load and verify
    let loaded_events = store.load_events(&aggregate_id, None).await.unwrap();
    assert_eq!(loaded_events.len(), 5);
    
    // Verify ordering
    for (i, event) in loaded_events.iter().enumerate() {
        assert_eq!(event.aggregate_version, (i as i64) + 1);
    }
    
    // Test loading from specific version
    let partial_events = store.load_events(&aggregate_id, Some(2)).await.unwrap();
    assert_eq!(partial_events.len(), 3); // Events 3, 4, 5
    assert_eq!(partial_events[0].aggregate_version, 3);
    
    println!("✓ PostgreSQL multiple events test passed");
}

#[tokio::test]
async fn test_postgres_concurrent_access() {
    let store = match setup_postgres_test_db().await {
        Some(store) => store,
        None => return, // Skip test
    };
    
    let store = std::sync::Arc::new(store);
    let mut handles = Vec::new();
    
    // Spawn concurrent tasks
    for task_id in 0..5 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let aggregate_id = format!("postgres-concurrent-{}-{}", task_id, Uuid::new_v4());
            
            // Each task creates and saves events
            let mut events = Vec::new();
            for event_num in 1..=3 {
                let event_data = EventData::from_json(&serde_json::json!({
                    "task_id": task_id,
                    "event_number": event_num,
                    "data": format!("concurrent-data-{}-{}", task_id, event_num)
                })).unwrap();
                
                let event = Event::new(
                    aggregate_id.clone(),
                    "ConcurrentTest".to_string(),
                    "ConcurrentEvent".to_string(),
                    1,
                    event_num,
                    event_data,
                );
                events.push(event);
            }
            
            store_clone.save_events(events).await.unwrap();
            
            // Verify the save worked
            let loaded = store_clone.load_events(&aggregate_id, None).await.unwrap();
            assert_eq!(loaded.len(), 3);
            
            aggregate_id // Return for verification
        });
        handles.push(handle);
    }
    
    // Wait for all tasks and collect results
    let mut aggregate_ids = Vec::new();
    for handle in handles {
        let aggregate_id = handle.await.unwrap();
        aggregate_ids.push(aggregate_id);
    }
    
    // Verify all aggregates exist independently
    for aggregate_id in aggregate_ids {
        let events = store.load_events(&aggregate_id, None).await.unwrap();
        assert_eq!(events.len(), 3);
    }
    
    println!("✓ PostgreSQL concurrent access test passed");
}

#[tokio::test]
async fn test_postgres_performance() {
    let store = match setup_postgres_test_db().await {
        Some(store) => store,
        None => return, // Skip test
    };
    
    let event_count = 500; // Smaller count for PostgreSQL network overhead
    let mut events = Vec::new();
    
    for i in 0..event_count {
        let event_data = EventData::from_json(&serde_json::json!({
            "performance_test_id": i,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": format!("performance-test-data-{}", i)
        })).unwrap();
        
        let event = Event::new(
            format!("postgres-perf-{}", i % 50), // 50 different aggregates
            "PerformanceTest".to_string(),
            "PerformanceEvent".to_string(),
            1,
            (i / 50) + 1, // Multiple events per aggregate
            event_data,
        );
        events.push(event);
    }
    
    // Measure save performance
    let start = std::time::Instant::now();
    store.save_events(events).await.unwrap();
    let save_duration = start.elapsed();
    
    let events_per_sec = event_count as f64 / save_duration.as_secs_f64();
    println!("PostgreSQL: Saved {} events in {:?} ({:.2} events/sec)", 
             event_count, save_duration, events_per_sec);
    
    // PostgreSQL should achieve reasonable performance even with network overhead
    assert!(events_per_sec > 100.0, "PostgreSQL performance too low: {} events/sec", events_per_sec);
    
    // Test individual aggregate loading
    let load_start = std::time::Instant::now();
    let test_events = store.load_events(&"postgres-perf-25".to_string(), None).await.unwrap();
    let load_duration = load_start.elapsed();
    
    assert!(test_events.len() == 10); // Should have 10 events for this aggregate
    println!("PostgreSQL: Loaded {} events in {:?}", test_events.len(), load_duration);
    
    println!("✓ PostgreSQL performance test passed");
}

#[tokio::test] 
async fn test_postgres_transaction_safety() {
    let store = match setup_postgres_test_db().await {
        Some(store) => store,
        None => return, // Skip test
    };
    
    let aggregate_id = format!("postgres-transaction-{}", Uuid::new_v4());
    
    // Save initial event
    let event1 = Event::new(
        aggregate_id.clone(),
        "TransactionTest".to_string(),
        "InitialEvent".to_string(),
        1,
        1,
        EventData::from_json(&serde_json::json!({"step": 1})).unwrap(),
    );
    
    store.save_events(vec![event1]).await.unwrap();
    
    // Verify initial state
    let events = store.load_events(&aggregate_id, None).await.unwrap();
    assert_eq!(events.len(), 1);
    
    // Create a batch of events (should be transactional)
    let batch_events = vec![
        Event::new(
            aggregate_id.clone(),
            "TransactionTest".to_string(),
            "BatchEvent1".to_string(),
            1,
            2,
            EventData::from_json(&serde_json::json!({"step": 2})).unwrap(),
        ),
        Event::new(
            aggregate_id.clone(),
            "TransactionTest".to_string(),
            "BatchEvent2".to_string(),
            1,
            3,
            EventData::from_json(&serde_json::json!({"step": 3})).unwrap(),
        ),
    ];
    
    store.save_events(batch_events).await.unwrap();
    
    // Verify all events are present (transaction worked)
    let all_events = store.load_events(&aggregate_id, None).await.unwrap();
    assert_eq!(all_events.len(), 3);
    
    // Verify ordering
    assert_eq!(all_events[0].aggregate_version, 1);
    assert_eq!(all_events[1].aggregate_version, 2);
    assert_eq!(all_events[2].aggregate_version, 3);
    
    println!("✓ PostgreSQL transaction safety test passed");
}