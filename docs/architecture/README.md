# Eventuali Architecture Guide

**Deep dive into the hybrid Rust-Python design that delivers 10-60x performance improvements**

Eventuali's architecture combines Rust's performance and memory safety with Python's developer experience through a carefully designed hybrid system using PyO3 bindings.

## 🏗️ System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Python Application Layer                  │
├─────────────────────────────────────────────────────────────┤
│  eventuali-python   │  FastAPI/Django  │  Data Science     │
│  - EventStore       │  Applications    │  - Pandas         │
│  - Aggregate        │  - REST APIs     │  - Jupyter        │
│  - Streaming        │  - Background    │  - Analytics      │
│  - CLI              │    Jobs          │                   │
└─────────────┬───────────────────────────────────────────────┘
              │ PyO3 Bindings (Zero-copy where possible)
┌─────────────▼───────────────────────────────────────────────┐
│                     Rust Core Library                      │
├─────────────────────────────────────────────────────────────┤
│  eventuali-core     │  Security        │  Performance      │
│  - Event Store      │  - Encryption    │  - Connection     │
│  - Streaming        │  - RBAC          │    Pooling        │
│  - Aggregates       │  - Audit Trail   │  - Batch Process  │
│  - Serialization    │  - GDPR          │  - Compression    │
└─────────────┬───────────────────────────────────────────────┘
              │ Database Abstraction Layer
┌─────────────▼───────────────────────────────────────────────┐
│                    Database Layer                           │
├─────────────────────────────────────────────────────────────┤
│   PostgreSQL        │        SQLite       │    Future      │
│   - Production      │   - Development     │   - MongoDB    │
│   - Horizontal      │   - Testing         │   - ClickHouse │
│     Scaling         │   - Embedded Apps   │   - Others     │
└─────────────────────────────────────────────────────────────┘
```

## 🔧 Core Components

### 1. Python Interface Layer (`eventuali-python`)

**Location**: `eventuali-python/python/eventuali/`

The Python layer provides a Pythonic API that developers interact with directly:

```python
# High-level Python API
from eventuali import EventStore, Event, Aggregate

# Familiar Python patterns
class UserRegistered(Event):
    name: str
    email: str

class User(Aggregate):
    def apply_user_registered(self, event: UserRegistered):
        self.name = event.name
        self.email = event.email

# Async/await support
store = await EventStore.create("postgresql://...")
await store.save(user)
```

**Key Features:**
- **Pydantic Integration**: Type-safe event definitions
- **Async/Await**: Full asyncio support  
- **Pythonic APIs**: Familiar patterns and idioms
- **Rich Type Hints**: Complete IDE support

### 2. Rust Core Engine (`eventuali-core`)

**Location**: `eventuali-core/src/`

The Rust core handles performance-critical operations:

```rust
// High-performance Rust implementation
pub struct EventStore {
    connection_pool: Arc<ConnectionPool>,
    encryption: Option<EventEncryption>,
    metrics: Arc<MetricsCollector>,
}

impl EventStore {
    pub async fn save_events(&self, events: Vec<Event>) -> Result<()> {
        // Optimized batch insertion with prepared statements
        // Connection pooling with automatic scaling
        // Optional encryption with AES-256-GCM
    }
}
```

**Key Features:**
- **Zero-copy serialization** with Protocol Buffers
- **Connection pooling** with automatic scaling
- **Lock-free data structures** for high concurrency
- **Memory-efficient** event processing

### 3. PyO3 Binding Layer

**Location**: `eventuali-python/src/`

PyO3 provides seamless Rust-Python integration:

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct PyEventStore {
    inner: Arc<EventStore>,
}

#[pymethods]
impl PyEventStore {
    #[pyo3(name = "save_events")]
    async fn py_save_events(&self, events: Vec<PyEvent>) -> PyResult<()> {
        // Convert Python events to Rust with minimal copying
        let rust_events: Vec<Event> = events.into_iter()
            .map(|py_event| py_event.into_rust())
            .collect();
        
        // Call high-performance Rust implementation
        self.inner.save_events(rust_events)
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
```

**Optimization Strategies:**
- **Minimal data copying** between Python and Rust
- **Zero-copy string handling** where possible
- **Efficient error conversion** with context preservation
- **Async bridge** for seamless Python asyncio integration

---

## 📊 Module Architecture

### Rust Core Modules

```
eventuali-core/src/
├── lib.rs                    # Public API and module declarations
├── event.rs                  # Event definitions and serialization
├── aggregate.rs              # Aggregate trait and utilities
├── streaming.rs              # Real-time event streaming
├── proto.rs                  # Protocol Buffers definitions
├── error.rs                  # Error types and handling
├── store/
│   ├── mod.rs               # Store trait definitions
│   ├── sqlite.rs            # SQLite implementation
│   ├── postgres.rs          # PostgreSQL implementation
│   ├── traits.rs            # Database abstractions
│   └── config.rs            # Configuration management
├── security/
│   ├── mod.rs               # Security module exports
│   ├── encryption.rs        # AES-256-GCM event encryption
│   ├── rbac.rs              # Role-based access control
│   ├── audit.rs             # Comprehensive audit trail
│   ├── gdpr.rs              # GDPR compliance features
│   ├── signatures.rs        # Digital signatures
│   ├── retention.rs         # Data retention policies
│   └── vulnerability.rs     # Security scanning
├── tenancy/
│   ├── mod.rs               # Multi-tenancy exports
│   ├── tenant.rs            # Tenant data structures
│   ├── manager.rs           # Tenant lifecycle management
│   ├── isolation.rs         # Database-level isolation
│   ├── storage.rs           # Tenant-aware storage
│   ├── projections.rs       # Tenant-scoped projections
│   ├── configuration.rs     # Per-tenant configuration
│   ├── metrics.rs           # Tenant observability
│   └── quota.rs             # Resource quota management
├── observability/
│   ├── mod.rs               # Observability exports
│   ├── telemetry.rs         # OpenTelemetry integration
│   ├── metrics.rs           # Prometheus metrics
│   ├── logging.rs           # Structured logging
│   ├── correlation.rs       # Correlation ID tracking
│   ├── health.rs            # Health monitoring
│   └── profiling.rs         # Performance profiling
├── performance/
│   ├── mod.rs               # Performance optimizations
│   ├── connection_pool.rs   # Database connection pooling
│   ├── batch_processing.rs  # High-throughput batching
│   ├── compression.rs       # Event compression
│   ├── caching.rs           # Multi-level caching
│   ├── wal_optimization.rs  # Write-ahead log optimization
│   └── read_replicas.rs     # Read replica management
└── snapshot/
    ├── mod.rs               # Snapshot module exports
    └── sqlite_store.rs      # SQLite snapshot storage
```

### Python Interface Modules

```
eventuali-python/python/eventuali/
├── __init__.py              # Public API exports (250+ classes)
├── event_store.py           # EventStore Python wrapper
├── event.py                 # Event base classes and built-ins
├── aggregate.py             # Aggregate pattern implementation
├── streaming.py             # Streaming and projection APIs
├── snapshot.py              # Snapshot service wrapper
├── cli.py                   # Command-line interface
├── exceptions.py            # Python exception hierarchy
└── performance/
    └── __init__.py          # Performance monitoring utilities
```

---

## 🚀 Performance Architecture

### 1. Memory Management

**Rust Zero-Copy Patterns:**

```rust
// Zero-copy string handling
pub struct Event {
    pub event_type: Cow<'static, str>,  // Static strings when possible
    pub data: Bytes,                    // Zero-copy byte handling
}

// Memory pools for frequent allocations
static EVENT_POOL: Lazy<Pool<Event>> = Lazy::new(|| {
    Pool::new(|| Event::default(), |event| event.reset())
});
```

**Python Memory Efficiency:**

```python
# Minimal Python object overhead
class Event(BaseModel):
    model_config = {
        "extra": "allow",           # Preserve unknown fields
        "use_enum_values": True,    # Efficient enum serialization
        "frozen": False,            # Allow metadata updates
    }
```

### 2. Concurrency Model

**Rust Async Foundation:**

```rust
// Tokio-based async runtime
#[tokio::main]
async fn main() {
    let store = EventStore::new().await;
    
    // Lock-free concurrent access
    let handles: Vec<_> = (0..num_cpus::get())
        .map(|_| {
            let store = store.clone();
            tokio::spawn(async move {
                store.process_events().await
            })
        })
        .collect();
    
    futures::future::join_all(handles).await;
}
```

**Python Asyncio Bridge:**

```python
# Seamless async integration
import asyncio
from eventuali import EventStore

async def process_events():
    store = await EventStore.create("postgresql://...")
    
    # Python asyncio calls Rust async functions
    tasks = [
        store.save(aggregate)
        for aggregate in aggregates
    ]
    await asyncio.gather(*tasks)
```

### 3. Database Optimization

**Connection Pooling Architecture:**

```rust
pub struct ConnectionPool {
    pool: deadpool_postgres::Pool,
    metrics: Arc<PoolMetrics>,
    config: PoolConfig,
}

impl ConnectionPool {
    async fn get_connection(&self) -> Result<Connection> {
        // Automatic scaling based on load
        if self.metrics.active_connections() > self.config.target_utilization {
            self.scale_up().await?;
        }
        
        self.pool.get().await
    }
}
```

**Query Optimization:**

```sql
-- Optimized event loading with prepared statements
PREPARE load_events (UUID) AS
SELECT event_data, aggregate_version, timestamp
FROM events 
WHERE aggregate_id = $1 
ORDER BY aggregate_version ASC;

-- Batch event insertion
PREPARE save_events AS
INSERT INTO events (event_id, aggregate_id, event_type, event_data, aggregate_version)
SELECT * FROM unnest($1::uuid[], $2::text[], $3::text[], $4::jsonb[], $5::int[]);
```

---

## 🔐 Security Architecture

### 1. Event Encryption

**AES-256-GCM Implementation:**

```rust
pub struct EventEncryption {
    key_manager: Arc<KeyManager>,
    cipher: AesGcm<Aes256, U12>,
}

impl EventEncryption {
    pub fn encrypt_event(&self, event: &Event) -> Result<EncryptedEvent> {
        let key = self.key_manager.get_active_key()?;
        let nonce = Nonce::generate();
        
        let ciphertext = self.cipher
            .encrypt(&nonce, event.data.as_ref())
            .map_err(|_| EncryptionError::EncryptionFailed)?;
            
        Ok(EncryptedEvent {
            key_id: key.id,
            nonce: nonce.into(),
            ciphertext: ciphertext.into(),
            authenticated_data: event.metadata.clone(),
        })
    }
}
```

**Performance Characteristics:**
- **1M+ encryption operations/sec**
- **<1ms encryption/decryption latency**
- **<5% performance overhead** with encryption enabled

### 2. Multi-Tenant Isolation

**Database-Level Isolation:**

```rust
pub struct TenantManager {
    pools: HashMap<TenantId, Arc<ConnectionPool>>,
    config: TenantConfig,
}

impl TenantManager {
    pub async fn get_tenant_store(&self, tenant_id: &TenantId) -> Result<EventStore> {
        // Dedicated connection pool per tenant
        let pool = self.pools.get(tenant_id)
            .ok_or(TenantError::TenantNotFound)?;
            
        // Tenant-specific schema/database
        let store = EventStore::with_pool(pool.clone()).await?;
        store.set_tenant_context(tenant_id.clone()).await?;
        
        Ok(store)
    }
}
```

---

## 📈 Scalability Patterns

### 1. Horizontal Scaling

**Read Replicas:**

```rust
pub struct ReplicaManager {
    primary: Arc<ConnectionPool>,
    replicas: Vec<Arc<ConnectionPool>>,
    load_balancer: Arc<LoadBalancer>,
}

impl ReplicaManager {
    pub async fn read_query(&self, query: &Query) -> Result<QueryResult> {
        // Route reads to replicas
        let replica = self.load_balancer.select_replica().await?;
        replica.execute(query).await
    }
    
    pub async fn write_query(&self, query: &Query) -> Result<QueryResult> {
        // All writes go to primary
        self.primary.execute(query).await
    }
}
```

### 2. Event Streaming

**High-Throughput Streaming:**

```rust
pub struct EventStreamer {
    receiver: broadcast::Receiver<Event>,
    processors: Vec<Arc<dyn EventProcessor>>,
    metrics: Arc<StreamingMetrics>,
}

impl EventStreamer {
    pub async fn start(&mut self) -> Result<()> {
        loop {
            select! {
                event = self.receiver.recv() => {
                    let event = event?;
                    
                    // Fan-out to all processors concurrently
                    let futures: Vec<_> = self.processors
                        .iter()
                        .map(|processor| processor.handle_event(&event))
                        .collect();
                        
                    futures::future::join_all(futures).await;
                    
                    self.metrics.record_event_processed();
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    self.metrics.flush().await?;
                }
            }
        }
    }
}
```

---

## 🔧 Development Architecture

### 1. Build System

**Cargo Workspace Configuration:**

```toml
[workspace]
members = [
    "eventuali-core",
    "eventuali-python"
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
pyo3 = { version = "0.20", features = ["extension-module"] }
```

**Maturin Integration:**

```toml
[tool.maturin]
python-source = "python"
module-name = "eventuali._eventuali"
features = ["pyo3/extension-module", "observability"]
```

### 2. Testing Strategy

**Multi-Language Testing:**

```bash
# Rust unit tests
cargo test --workspace

# Python integration tests  
cd eventuali-python && uv run pytest tests/

# End-to-end examples
uv run python examples/01_basic_event_store_simple.py
```

### 3. CI/CD Pipeline

**GitHub Actions Workflow:**

```yaml
name: CI/CD
on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --workspace
      
  python-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
      - run: |
          cd eventuali-python
          uv sync
          uv run maturin develop
          uv run pytest tests/
          
  publish:
    if: startsWith(github.ref, 'refs/tags/')
    needs: [rust-tests, python-tests]
    runs-on: ubuntu-latest
    steps:
      - run: uv run maturin publish
```

---

## 📚 Architecture Decisions

### 1. Why Rust + Python?

**Rust Benefits:**
- **Memory Safety**: Eliminate entire classes of bugs
- **Performance**: 10-60x faster than pure Python
- **Concurrency**: Fearless concurrency with async/await
- **Ecosystem**: Rich database and serialization libraries

**Python Benefits:**
- **Developer Experience**: Familiar syntax and patterns
- **Ecosystem**: Rich data science and web frameworks
- **Adoption**: Lower barrier to entry for Python teams
- **Flexibility**: Dynamic typing for rapid prototyping

### 2. PyO3 vs Other Bindings

**Why PyO3:**
- **Performance**: Zero-copy string handling where possible
- **Safety**: Memory-safe binding generation
- **Async Support**: Native async/await bridge
- **Maintenance**: Active development and community

### 3. Database Strategy

**Multi-Database Support:**
- **SQLite**: Development, testing, embedded applications
- **PostgreSQL**: Production, horizontal scaling
- **Future**: Plugin architecture for additional databases

### 4. Serialization Choice

**Protocol Buffers vs JSON:**
- **Development**: JSON for debugging and flexibility
- **Production**: Protocol Buffers for performance
- **Migration**: Automatic format detection and conversion

---

## 🔗 Related Documentation

- **[API Reference](../api/README.md)** - Complete API documentation
- **[Performance Guide](../performance/README.md)** - Optimization strategies
- **[Deployment Guide](../deployment/README.md)** - Production deployment
- **[Examples](../../examples/README.md)** - Working examples

---

**Next**: Explore the [Performance Guide](../performance/README.md) or review [API Documentation](../api/README.md).