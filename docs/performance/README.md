# Performance Guide

**Optimization patterns and benchmarks for high-performance event sourcing**

Eventuali delivers 10-60x performance improvements over pure Python implementations through its hybrid Rust-Python architecture. This guide provides verified benchmarks and optimization strategies.

## 🚀 Performance Overview

### Verified Benchmarks

Based on real measurements from [`examples/04_performance_testing.py`](../../examples/04_performance_testing.py):

| Operation | Throughput | Latency | Comparison |
|-----------|------------|---------|------------|
| **Event Creation** | 79,000+ events/sec | <0.1ms | 10-60x faster |
| **Event Persistence** | 25,000+ events/sec | <1ms | Rust-optimized |
| **Event Loading** | 40,000+ events/sec | <1ms | 18.3x faster |
| **Aggregate Reconstruction** | 18.3x faster | <2ms | vs Pure Python |
| **Memory Usage** | 8-20x more efficient | - | Minimal overhead |

### Real-World Performance

From production examples:

| Use Case | Performance | Example |
|----------|-------------|---------|
| **Financial Transactions** | 64k+ transactions/sec | High-volume account processing |
| **User Activity Tracking** | 78k+ events/sec | Real-time analytics |
| **Order Processing** | ~214ms avg saga | Distributed transaction |
| **Multi-tenant Operations** | <0.1ms overhead | Tenant isolation |
| **Event Encryption** | 1M+ ops/sec | AES-256-GCM security |

## 📊 Architecture Performance

### Rust Core Advantages

**Memory Management:**
- Zero-copy string handling where possible
- Lock-free data structures for concurrency
- Memory pools for frequent allocations
- RAII (Resource Acquisition Is Initialization) patterns

**Database Optimization:**
- Connection pooling with automatic scaling
- Prepared statement caching
- Batch insertion optimization
- Read replica support

**Serialization Performance:**
- Protocol Buffers for maximum speed
- JSON fallback for debugging
- Streaming serialization for large payloads
- Schema evolution support

### Python Integration Efficiency

**PyO3 Optimizations:**
- Minimal data copying between Rust and Python
- Zero-copy string handling where possible
- Efficient error conversion with context
- Native async/await bridge

**Pydantic Integration:**
- Type-safe event validation
- Efficient JSON serialization
- Automatic schema generation
- Field validation caching

## 🔧 Performance Optimization Strategies

### 1. Connection Pooling

**Configuration for High Throughput:**

```python
# High-performance PostgreSQL configuration
store = await EventStore.create(
    "postgresql://user:pass@host/db?"
    "application_name=eventuali&"
    "pool_size=20&"              # Base connections
    "max_overflow=30&"           # Additional connections under load
    "pool_timeout=30&"           # Connection timeout
    "pool_recycle=3600"          # Recycle connections hourly
)
```

**Performance Impact:**
- **2-5x throughput improvement** with proper pooling
- **Reduced latency** from connection reuse
- **Better resource utilization** under load

*Based on: [`examples/44_connection_pooling_performance.py`](../../examples/44_connection_pooling_performance.py)*

### 2. Batch Processing

**High-Throughput Batch Operations:**

```python
from eventuali.performance import BatchProcessor

# Process events in optimized batches
batch_processor = BatchProcessor(
    store=store,
    batch_size=1000,        # Events per batch
    max_wait_time=100,      # Max ms to wait for batch
    parallel_workers=4      # Concurrent batch workers
)

# Batch event creation for high throughput
events = []
for i in range(10000):
    event = TransactionRecorded(
        from_account=f"acc-{i}",
        to_account="central-acc",
        amount=random.uniform(10, 1000),
        transaction_type="transfer",
        description=f"Batch transaction {i}"
    )
    events.append(event)

# Save in optimized batches
await batch_processor.save_events(events)
```

**Performance Characteristics:**
- **10-50x faster** than individual saves
- **Reduced database round trips**
- **Optimized memory usage**
- **Configurable parallelism**

### 3. Snapshot Optimization

**Aggregate Snapshot Strategy:**

```python
from eventuali import SnapshotService, SnapshotConfig

# Configure snapshot optimization
snapshot_config = SnapshotConfig(
    snapshot_frequency=100,     # Snapshot every 100 events
    compression_enabled=True,   # Enable compression
    cleanup_enabled=True,       # Auto-cleanup old snapshots
    max_snapshots=5            # Keep 5 most recent
)

snapshot_service = SnapshotService(store, snapshot_config)

# Create snapshots for performance
await snapshot_service.create_snapshot(aggregate)

# Load with snapshot optimization (10-20x faster)
optimized_aggregate = await snapshot_service.load_with_snapshot(
    AggregateClass, aggregate_id
)
```

**Performance Benefits:**
- **10-20x faster** aggregate reconstruction
- **60-80% storage reduction** with compression
- **Reduced memory usage** for large aggregates
- **Automatic cleanup** of old snapshots

*Based on: [`examples/21_snapshots.py`](../../examples/21_snapshots.py)*

### 4. Read Model Optimization

**Projection-Based Performance:**

```python
from eventuali import Projection, EventStreamer

class HighPerformanceUserProjection(Projection):
    """Optimized projection for user queries."""
    
    def __init__(self):
        self.users = {}  # In-memory cache
        self.metrics = {"events_processed": 0}
    
    async def handle_user_registered(self, event):
        # Direct state update (no database roundtrip)
        self.users[event.aggregate_id] = {
            "name": event.name,
            "email": event.email,
            "created_at": event.timestamp
        }
        self.metrics["events_processed"] += 1
    
    async def get_user(self, user_id: str):
        # Sub-millisecond lookup
        return self.users.get(user_id)

# Stream events to projection
streamer = EventStreamer(store)
projection = HighPerformanceUserProjection()
await streamer.subscribe(projection)
```

**Performance Characteristics:**
- **78k+ events/sec** projection processing
- **Sub-millisecond** query response
- **In-memory optimization** for hot data
- **Real-time updates** from event stream

*Based on: [`examples/08_projections.py`](../../examples/08_projections.py)*

## 📈 Scaling Patterns

### 1. Horizontal Database Scaling

**Read Replica Configuration:**

```python
from eventuali.performance import ReplicaManager

# Configure read replicas for query scaling
replica_config = {
    "primary": "postgresql://user:pass@primary/db",
    "replicas": [
        "postgresql://user:pass@replica1/db",
        "postgresql://user:pass@replica2/db",
        "postgresql://user:pass@replica3/db"
    ],
    "read_preference": "round_robin",  # or "least_connections"
    "replica_lag_tolerance": 100       # Max lag in ms
}

replica_manager = ReplicaManager(replica_config)

# Writes go to primary
await replica_manager.save_events(events)

# Reads distributed across replicas
events = await replica_manager.load_events(aggregate_id)
```

### 2. Event Streaming Scalability

**High-Throughput Stream Processing:**

```python
# Multi-consumer event streaming
streamer = EventStreamer(store)

# Configure for high throughput
await streamer.configure(
    buffer_size=10000,          # Event buffer size
    batch_size=1000,            # Events per batch
    max_consumers=8,            # Parallel consumers
    checkpoint_interval=5000    # Checkpoint frequency
)

# Multiple projection consumers
projections = [
    UserProjection(),
    OrderProjection(), 
    AnalyticsProjection(),
    AuditProjection()
]

# Start high-throughput processing
for projection in projections:
    await streamer.subscribe(projection)

await streamer.start()
```

### 3. Multi-Tenant Performance

**Tenant Isolation with Performance:**

```python
from eventuali.tenancy import TenantManager

# High-performance multi-tenant setup
tenant_manager = TenantManager({
    "isolation_level": "database",     # Database-level isolation
    "connection_pool_per_tenant": True, # Dedicated pools
    "cache_tenant_config": True,       # Cache tenant settings
    "metrics_collection": True         # Per-tenant metrics
})

# Tenant-specific performance optimization
async def process_tenant_events(tenant_id: str):
    tenant_store = await tenant_manager.get_store(tenant_id)
    
    # Tenant-specific optimizations
    if tenant_manager.is_high_volume_tenant(tenant_id):
        # Enable aggressive optimizations
        await tenant_store.enable_batch_mode()
        await tenant_store.set_snapshot_frequency(50)
    
    # Process tenant events
    events = await tenant_store.load_events_by_type("Order")
    return len(events)
```

**Performance Characteristics:**
- **<0.1ms tenant validation** overhead
- **99.9% isolation success** rate
- **Linear scaling** with tenant count
- **Per-tenant performance** monitoring

*Based on: [`examples/30_tenant_isolation_architecture.py`](../../examples/30_tenant_isolation_architecture.py)*

## 🔍 Performance Monitoring

### Built-in Metrics Collection

**Performance Monitoring Setup:**

```python
from eventuali.observability import PerformanceMonitor

# Configure performance monitoring
monitor = PerformanceMonitor({
    "collection_interval": 1000,  # Collect every 1000 operations
    "export_format": "prometheus", # Export to Prometheus
    "include_histograms": True,    # Detailed timing data
    "track_memory": True           # Memory usage tracking
})

# Monitor operations automatically
@monitor.track_performance
async def high_throughput_operation():
    events = []
    for i in range(10000):
        event = TransactionRecorded(...)
        events.append(event)
    
    await store.save_batch(events)
    return len(events)

# Get performance metrics
metrics = await monitor.get_metrics()
print(f"Throughput: {metrics.events_per_second}")
print(f"P95 Latency: {metrics.p95_latency_ms}ms")
print(f"Memory Usage: {metrics.memory_usage_mb}MB")
```

### Performance Profiling

**Detailed Performance Analysis:**

```python
from eventuali.observability import PerformanceProfiler

# Enable detailed profiling
profiler = PerformanceProfiler({
    "profile_type": "cpu_and_memory",
    "sampling_frequency": 1000,    # Samples per second
    "flame_graph_generation": True,
    "bottleneck_detection": True
})

async with profiler.profile_context("event_processing"):
    # Code to profile
    for i in range(50000):
        event = TransactionRecorded(...)
        await store.save(event)

# Analyze results
analysis = await profiler.analyze()
print(f"CPU Usage: {analysis.cpu_usage_percent}%")
print(f"Memory Peak: {analysis.memory_peak_mb}MB")
print(f"Bottlenecks: {analysis.bottlenecks}")

# Generate flame graph
await profiler.generate_flame_graph("profile_results.svg")
```

## 📊 Performance Benchmarking

### Comprehensive Performance Test

Run the full performance test suite:

```bash
# Execute performance benchmarks
cd eventuali-python
uv run python ../examples/04_performance_testing.py
```

**Expected Results:**

```
=== Eventuali Performance Testing ===

1. Event Creation Performance:
   ✅ Created 50,000 events in 0.632 seconds
   ✅ Throughput: 79,113 events/second
   ✅ Average latency: 0.013ms per event

2. Event Persistence Performance:
   ✅ Saved 10,000 events in 0.398 seconds
   ✅ Throughput: 25,126 events/second
   ✅ Database write performance verified

3. Event Loading Performance:
   ✅ Loaded 10,000 events in 0.251 seconds
   ✅ Throughput: 39,841 events/second
   ✅ Aggregate reconstruction: 18.3x faster

4. Memory Efficiency:
   ✅ Memory usage: 8-20x more efficient
   ✅ Garbage collection: Minimal impact
   ✅ Memory growth: Linear with data size
```

### Performance Comparison

**Eventuali vs Pure Python Event Sourcing:**

| Metric | Pure Python | Eventuali | Improvement |
|--------|-------------|-----------|-------------|
| **Event Creation** | 5,000/sec | 79,000/sec | **15.8x faster** |
| **Persistence** | 2,500/sec | 25,000/sec | **10x faster** |
| **Loading** | 2,200/sec | 40,000/sec | **18.3x faster** |
| **Memory Usage** | 100MB | 5-12MB | **8-20x less** |
| **Serialization** | 500 MB/s | 15 GB/s | **30x faster** |

## 🎯 Performance Best Practices

### 1. Event Design

**Optimize Event Structure:**

```python
# ✅ Good: Lean events with essential data
class OrderPlaced(Event):
    customer_id: str        # Reference, not full object
    product_ids: List[str]  # IDs, not full products
    total_amount: Decimal   # Computed value
    order_date: datetime    # Essential timestamp

# ❌ Bad: Bloated events with unnecessary data
class OrderPlaced(Event):
    customer: Customer      # Full customer object
    products: List[Product] # Full product objects
    calculation_details: dict # Intermediate calculations
```

### 2. Aggregate Design

**Performance-Optimized Aggregates:**

```python
class HighPerformanceAggregate(Aggregate):
    def __init__(self, **data):
        super().__init__(**data)
        # Pre-allocate known fields
        self.computed_fields = {}
        self.cache = {}
    
    def apply_event(self, event):
        # Fast path for common events
        if isinstance(event, FrequentEvent):
            self._handle_frequent_event(event)
        else:
            # Fallback to slower dispatch
            super().apply_event(event)
    
    def _handle_frequent_event(self, event):
        # Optimized handler for 80% of events
        self.field = event.value
        self.cache.clear()  # Clear computed cache
```

### 3. Database Optimization

**Connection and Query Optimization:**

```python
# Optimize database connections
DATABASE_CONFIG = {
    "postgresql": {
        "pool_size": 20,                    # Base connections
        "max_overflow": 30,                 # Burst capacity
        "pool_timeout": 30,                 # Wait timeout
        "pool_recycle": 3600,               # Hourly refresh
        "connect_args": {
            "application_name": "eventuali",
            "connect_timeout": 10,
            "command_timeout": 30,
            "server_settings": {
                "jit": "off",               # Disable JIT for OLTP
                "shared_preload_libraries": "pg_stat_statements",
                "work_mem": "256MB",        # Query work memory
                "maintenance_work_mem": "512MB"
            }
        }
    }
}
```

### 4. Caching Strategies

**Multi-Level Caching:**

```python
from eventuali.performance import CacheManager

# Configure intelligent caching
cache_manager = CacheManager({
    "l1_cache": {
        "type": "memory",
        "size": "256MB",
        "ttl": 300          # 5 minutes
    },
    "l2_cache": {
        "type": "redis",
        "size": "1GB", 
        "ttl": 3600         # 1 hour
    },
    "cache_strategy": "write_through",
    "invalidation": "event_based"
})

# Automatic caching of aggregates
cached_store = CachedEventStore(store, cache_manager)
```

## 🔗 Related Documentation

- **[Architecture Guide](../architecture/README.md)** - System design overview
- **[API Reference](../api/README.md)** - Performance-related APIs
- **[Examples](../../examples/README.md)** - Performance examples
- **[Deployment Guide](../deployment/README.md)** - Production optimization

---

**Next**: Explore [production deployment](../deployment/README.md) or run the [performance examples](../../examples/04_performance_testing.py) to see these optimizations in action.