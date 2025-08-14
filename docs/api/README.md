# API Reference

Complete API documentation for Eventuali's Python interface, generated from actual code annotations and verified through working examples.

## 📋 Core Classes

### Event Sourcing Foundation

- **[EventStore](event-store.md)** - Central event persistence and retrieval
- **[Event](event.md)** - Base event class with domain and system events  
- **[Aggregate](aggregate.md)** - Aggregate root pattern implementation
- **[User](user.md)** - Built-in user aggregate with business logic

### Streaming & Real-time Processing

- **[EventStreamer](streaming/event-streamer.md)** - High-performance event streaming
- **[Projection](streaming/projection.md)** - Real-time read model building
- **[SagaHandler](streaming/saga-handler.md)** - Distributed transaction coordination
- **[Subscription](streaming/subscription.md)** - Event subscription management

### Performance & Storage

- **[SnapshotService](performance/snapshot-service.md)** - Aggregate snapshots for optimization
- **[ConnectionPool](performance/connection-pool.md)** - Database connection management
- **[BatchProcessor](performance/batch-processor.md)** - High-throughput batch operations

### Security & Compliance

- **[EventEncryption](security/encryption.md)** - AES-256-GCM event encryption
- **[RbacManager](security/rbac.md)** - Role-based access control
- **[AuditManager](security/audit.md)** - Comprehensive audit trail
- **[GdprManager](security/gdpr.md)** - GDPR compliance features

### Multi-tenancy

- **[TenantManager](tenancy/manager.md)** - Tenant lifecycle management
- **[TenantConfiguration](tenancy/configuration.md)** - Per-tenant configuration
- **[TenantMetrics](tenancy/metrics.md)** - Tenant observability

### Observability

- **[ObservabilityService](observability/service.md)** - OpenTelemetry integration
- **[HealthMonitor](observability/health.md)** - System health monitoring
- **[PerformanceProfiler](observability/profiler.md)** - Performance analysis

### CLI Interface

- **[CLI Commands](cli/commands.md)** - Command-line interface reference
- **[Configuration](cli/configuration.md)** - CLI configuration management

## 🔍 Quick API Overview

### Basic Usage Pattern

```python
from eventuali import EventStore, Aggregate, Event

# Create event store
store = await EventStore.create("postgresql://...")

# Define events
class UserRegistered(Event):
    name: str
    email: str

# Define aggregates  
class User(Aggregate):
    def apply_user_registered(self, event: UserRegistered):
        self.name = event.name
        self.email = event.email

# Use the system
user = User()
user.apply(UserRegistered(name="John", email="john@example.com"))
await store.save(user)
```

### Streaming Pattern

```python
from eventuali import EventStreamer, Projection

# Create streamer
streamer = EventStreamer(store)

# Build projection
class UserProjection(Projection):
    async def handle_user_registered(self, event: UserRegistered):
        # Update read model
        pass

# Start streaming
await streamer.start()
```

### Security Pattern

```python
from eventuali import EventEncryption, KeyManager

# Setup encryption
key_manager = KeyManager()
encryption = EventEncryption(key_manager)

# Encrypt events automatically
encrypted_store = await EventStore.create(
    connection_string="postgresql://...",
    encryption=encryption
)
```

## 📊 Performance Characteristics

### Core Operations
- **Event Creation**: 79,000+ events/sec
- **Event Persistence**: 25,000+ events/sec
- **Event Loading**: 40,000+ events/sec
- **Aggregate Reconstruction**: 18.3x faster than pure Python

### Memory Efficiency
- **8-20x lower memory usage** compared to pure Python
- **Compression ratios**: 60-80% storage reduction with snapshots
- **Connection pooling**: Automatic scaling based on load

### Encryption Performance  
- **1M+ encryption operations/sec**
- **<1ms encryption/decryption latency**
- **<5% performance overhead** with encryption enabled

## 🔗 Related Documentation

- **[Architecture Guide](../architecture/README.md)** - System design overview
- **[Integration Guides](../guides/README.md)** - Step-by-step tutorials
- **[Examples](../../examples/README.md)** - 46+ working examples
- **[AI Schemas](../schemas/README.md)** - Machine-readable API specifications

---

**Next**: Explore specific API classes or check out the [Integration Guides](../guides/README.md) for practical examples.