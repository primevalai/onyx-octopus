use eventuali_core::{
    Event, EventData, EventMetadata, Aggregate, 
    EventStoreConfig, create_event_store,
};
use tokio;
use uuid::Uuid;

#[tokio::test]
async fn test_sqlite_event_store() {
    // Create test database
    let config = EventStoreConfig::sqlite(":memory:".to_string());
    let store = create_event_store(config).await.unwrap();
    
    // Create test event
    let aggregate_id = Uuid::new_v4().to_string();
    let event_data = EventData::from_json(&serde_json::json!({
        "name": "John Doe",
        "email": "john@example.com"
    })).unwrap();
    
    let event = Event::new(
        aggregate_id.clone(),
        "User".to_string(),
        "UserRegistered".to_string(),
        1,
        1,
        event_data,
    );
    
    // Save event
    store.save_events(vec![event.clone()]).await.unwrap();
    
    // Load events
    let loaded_events = store.load_events(&aggregate_id, None).await.unwrap();
    assert_eq!(loaded_events.len(), 1);
    assert_eq!(loaded_events[0].aggregate_id, aggregate_id);
    assert_eq!(loaded_events[0].event_type, "UserRegistered");
    
    // Test version retrieval
    let version = store.get_aggregate_version(&aggregate_id).await.unwrap();
    assert_eq!(version, Some(1));
}

#[tokio::test]
async fn test_event_data_serialization() {
    // Test JSON serialization
    let json_data = serde_json::json!({
        "name": "Test User",
        "age": 30,
        "active": true
    });
    
    let event_data = EventData::from_json(&json_data).unwrap();
    let deserialized: serde_json::Value = event_data.to_json().unwrap();
    
    assert_eq!(deserialized["name"], "Test User");
    assert_eq!(deserialized["age"], 30);
    assert_eq!(deserialized["active"], true);
}

#[tokio::test]
async fn test_aggregate_creation() {
    let aggregate = Aggregate::new_with_uuid("User".to_string());
    
    assert!(!aggregate.id.is_empty());
    assert_eq!(aggregate.version, 0);
    assert_eq!(aggregate.aggregate_type, "User");
    assert!(aggregate.is_new());
}

#[test]
fn test_event_metadata() {
    let mut metadata = EventMetadata::default();
    metadata.user_id = Some("user-123".to_string());
    metadata.headers.insert("source".to_string(), "web-app".to_string());
    
    assert_eq!(metadata.user_id, Some("user-123".to_string()));
    assert_eq!(metadata.headers.get("source"), Some(&"web-app".to_string()));
}