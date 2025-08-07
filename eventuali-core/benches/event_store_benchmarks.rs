use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use eventuali_core::{
    Event, EventData, EventStoreConfig, create_event_store
};
use uuid::Uuid;

async fn setup_sqlite_store() -> Box<dyn eventuali_core::EventStore + Send + Sync> {
    let config = EventStoreConfig::sqlite(":memory:".to_string());
    create_event_store(config).await.unwrap()
}

fn create_test_events(count: usize) -> Vec<Event> {
    let mut events = Vec::new();
    let aggregate_id = Uuid::new_v4().to_string();
    
    for i in 0..count {
        let event_data = EventData::from_json(&serde_json::json!({
            "user_id": format!("user-{}", i),
            "name": format!("User {}", i),
            "email": format!("user{}@example.com", i),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "sequence": i
        })).unwrap();
        
        let event = Event::new(
            format!("{}-{}", aggregate_id, i),
            "User".to_string(),
            "UserEvent".to_string(),
            1,
            i as i64 + 1,
            event_data,
        );
        events.push(event);
    }
    
    events
}

fn bench_event_creation(c: &mut Criterion) {
    c.bench_function("event_creation_1000", |b| {
        b.iter(|| {
            create_test_events(1000)
        })
    });
}

fn bench_event_serialization(c: &mut Criterion) {
    let events = create_test_events(100);
    
    c.bench_function("event_serialization_100", |b| {
        b.iter(|| {
            for event in &events {
                let _json = serde_json::to_string(&event).unwrap();
            }
        })
    });
}

fn bench_sqlite_bulk_save(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("sqlite_bulk_save");
    for size in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(BenchmarkId::new("events", size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let store = setup_sqlite_store().await;
                    let events = create_test_events(size);
                    store.save_events(events).await.unwrap();
                })
            });
        });
    }
    group.finish();
}

fn bench_sqlite_individual_save(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("sqlite_individual_save_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let store = setup_sqlite_store().await;
                let events = create_test_events(100);
                
                for event in events {
                    store.save_events(vec![event]).await.unwrap();
                }
            })
        });
    });
}

fn bench_sqlite_load(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Setup data
    let store = rt.block_on(setup_sqlite_store());
    let events = create_test_events(1000);
    let aggregate_id = events[0].aggregate_id.clone();
    rt.block_on(store.save_events(events)).unwrap();
    
    c.bench_function("sqlite_load_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _events = store.load_events(&aggregate_id, None).await.unwrap();
            })
        });
    });
}

fn bench_memory_usage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("memory_efficiency_10000", |b| {
        b.iter(|| {
            rt.block_on(async {
                let store = setup_sqlite_store().await;
                let events = create_test_events(10000);
                store.save_events(events).await.unwrap();
                
                // Load a subset
                let loaded = store.load_events(&"user-123".to_string(), None).await.unwrap_or_default();
                loaded.len() // Force evaluation
            })
        });
    });
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("concurrent_saves_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                let store = std::sync::Arc::new(setup_sqlite_store().await);
                let mut handles = Vec::new();
                
                for _task_id in 0..10 {
                    let store_clone = store.clone();
                    let handle = tokio::spawn(async move {
                        let events = create_test_events(100);
                        store_clone.save_events(events).await.unwrap();
                    });
                    handles.push(handle);
                }
                
                for handle in handles {
                    handle.await.unwrap();
                }
            })
        });
    });
}

criterion_group!(
    benches,
    bench_event_creation,
    bench_event_serialization,
    bench_sqlite_bulk_save,
    bench_sqlite_individual_save,
    bench_sqlite_load,
    bench_memory_usage,
    bench_concurrent_operations
);

criterion_main!(benches);