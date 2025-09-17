# Eventuali Core 🚀

A high-performance event sourcing library written in Rust, providing the core foundation for the Eventuali event sourcing ecosystem.

[![Crates.io](https://img.shields.io/crates/v/eventuali-core.svg)](https://crates.io/crates/eventuali-core)
[![Documentation](https://docs.rs/eventuali-core/badge.svg)](https://docs.rs/eventuali-core)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Features

- **🚄 High Performance**: 10-60x faster than pure Python implementations
- **🔒 Type Safety**: Built with Rust's memory safety guarantees
- **🗄️ Multi-Database**: Support for PostgreSQL and SQLite
- **📡 Real-time Streaming**: Event streaming with projections and sagas
- **📸 Snapshots**: Aggregate snapshot support with compression
- **🔐 Security**: Built-in event encryption and tenant isolation
- **🏢 Multi-tenancy**: Complete tenant isolation and resource management
- **📊 Observability**: OpenTelemetry integration with metrics and tracing

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
eventuali-core = "0.1"
```

### Basic Usage

```rust
use eventuali_core::{
    EventStoreConfig, create_event_store, Event, EventData, EventMetadata
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory SQLite event store
    let config = EventStoreConfig::sqlite(":memory:").await?;
    let store = create_event_store(config).await?;

    // Create an event
    let event = Event {
        id: Uuid::new_v4(),
        aggregate_id: "user-123".to_string(),
        aggregate_type: "User".to_string(),
        event_type: "UserRegistered".to_string(),
        event_version: 1,
        aggregate_version: 1,
        data: EventData::from_json(&json!({
            "name": "Alice",
            "email": "alice@example.com"
        }))?,
        metadata: EventMetadata::default(),
        timestamp: Utc::now(),
    };

    // Store the event
    store.append_events(vec![event]).await?;

    // Load events for an aggregate
    let events = store.load_events("user-123", None).await?;
    println!("Loaded {} events", events.len());

    Ok(())
}
```

### Event Streaming

```rust
use eventuali_core::{
    InMemoryEventStreamer, SubscriptionBuilder, Projection
};
use async_trait::async_trait;

// Define a projection
struct UserCountProjection {
    count: i32,
}

#[async_trait]
impl Projection for UserCountProjection {
    async fn handle_event(&mut self, event: &Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event.event_type == "UserRegistered" {
            self.count += 1;
            println!("User count: {}", self.count);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let streamer = InMemoryEventStreamer::new();
    let projection = UserCountProjection { count: 0 };

    // Subscribe to events
    let subscription = SubscriptionBuilder::new()
        .event_type("UserRegistered".to_string())
        .build();

    let mut receiver = streamer.subscribe(subscription).await?;

    // Process events as they arrive
    while let Ok(stream_event) = receiver.recv().await {
        projection.handle_event(&stream_event.event).await?;
    }

    Ok(())
}
```

## Feature Flags

Control which features are compiled:

```toml
[dependencies]
eventuali-core = { version = "0.1", features = ["postgres", "sqlite", "observability"] }
```

Available features:
- `postgres` - PostgreSQL backend support
- `sqlite` - SQLite backend support (included by default)
- `observability` - OpenTelemetry tracing and metrics (included by default)

## Performance

Eventuali Core achieves exceptional performance through Rust's zero-cost abstractions:

- **Event Creation**: 50,000+ events/second
- **Event Persistence**: 25,000+ events/second
- **Event Loading**: 40,000+ events/second
- **Memory Usage**: 8-20x more efficient than Python
- **Concurrent Throughput**: 2x better under load

## Architecture

The library is organized into several key modules:

- **`event`** - Core event types and serialization
- **`store`** - Event storage abstraction and implementations
- **`streaming`** - Real-time event streaming and projections
- **`aggregate`** - Aggregate root patterns and versioning
- **`snapshot`** - Snapshot storage and compression
- **`security`** - Event encryption and key management
- **`tenancy`** - Multi-tenant isolation and resource management
- **`observability`** - Metrics, tracing, and logging

## Database Support

### SQLite
```rust
let config = EventStoreConfig::sqlite("events.db").await?;
let store = create_event_store(config).await?;
```

### PostgreSQL
```rust
let config = EventStoreConfig::postgres("postgresql://user:password@localhost/eventstore").await?;
let store = create_event_store(config).await?;
```

## Advanced Features

### Event Encryption

```rust
use eventuali_core::{EventEncryption, KeyManager, EncryptionAlgorithm};

let key_manager = KeyManager::new();
let encryption = EventEncryption::new(key_manager, EncryptionAlgorithm::AesGcm);

// Events are automatically encrypted/decrypted
```

### Multi-tenancy

```rust
use eventuali_core::{TenantId, IsolatedEventStore};

let tenant_id = TenantId::new("tenant-1");
let isolated_store = IsolatedEventStore::new(store, tenant_id);

// All operations are automatically scoped to the tenant
```

### Snapshots

```rust
use eventuali_core::{SnapshotStore, SnapshotConfig, SnapshotCompression};

let snapshot_config = SnapshotConfig {
    compression: SnapshotCompression::Gzip,
    retention_count: 10,
};

let snapshot_store = SqliteSnapshotStore::new(snapshot_config).await?;
```

## Examples

The `examples/` directory contains comprehensive examples:

- `basic_event_store.rs` - Basic event storage and retrieval
- `streaming_projections.rs` - Real-time event streaming and projections
- `multi_tenant.rs` - Multi-tenant event isolation
- `rust_streaming_demo.rs` - Complete streaming demonstration

Run examples with:
```bash
cargo run --example basic_event_store
```

## Integration

Eventuali Core is designed to integrate seamlessly with:

- **Python**: Via [eventuali-python](https://pypi.org/project/eventuali/) PyO3 bindings
- **Web Frameworks**: Axum, Actix, Warp integration examples
- **Databases**: PostgreSQL 12+, SQLite 3.35+
- **Observability**: OpenTelemetry, Prometheus, Jaeger

## Contributing

Contributions are welcome! Please see our [contributing guidelines](../CONTRIBUTING.md).

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.