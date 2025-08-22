# Eventuali Implementation Guide

## Overview

Eventuali is a high-performance event sourcing library that combines Rust's performance with Python's ease of use. The implementation uses a hybrid architecture where the performance-critical parts are written in Rust and exposed to Python through PyO3 bindings.

## Architecture

### Core Components

```
┌─────────────────────────────────────────┐
│              Python Layer               │
├─────────────────────────────────────────┤
│  • Pydantic Models (Events/Aggregates)  │
│  • Business Logic                       │
│  • High-level APIs                      │
│  • AsyncIO Integration                  │
└─────────────────────────────────────────┘
                    │
                    │ PyO3 Bindings
                    ▼
┌─────────────────────────────────────────┐
│               Rust Core                 │
├─────────────────────────────────────────┤
│  • Event Storage (SQLx)                 │
│  • Serialization (JSON/ProtoBuf)        │
│  • Database Backends                    │
│  • High-performance Operations          │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│              Databases                  │
├─────────────────────────────────────────┤
│  • PostgreSQL (Production)             │
│  • SQLite (Development)                │
└─────────────────────────────────────────┘
```

## Implementation Status

### ✅ Completed Components

#### 1. Rust Core Library (`eventuali-core`)

**Event System**:
- `Event` struct with metadata, versioning, and timestamps
- `EventData` enum supporting JSON and Protocol Buffers
- `EventMetadata` for causation/correlation tracking

**Aggregate System**:
- `Aggregate` struct with ID, version, and type
- Basic aggregate operations (create, increment version)

**Database Abstraction**:
- `EventStore` and `EventStoreBackend` traits
- Factory function for creating event stores
- Database-agnostic configuration

**PostgreSQL Backend**:
- Full CRUD operations for events
- Optimistic concurrency control
- JSON and Protocol Buffer storage
- Connection pooling with SQLx
- Proper indexing for performance

**SQLite Backend**:
- Full CRUD operations for events
- WAL mode for better concurrency
- File-based and in-memory support
- Compatible API with PostgreSQL backend

#### 2. PyO3 Bindings (`eventuali-python/src`)

**Python-Rust Bridge**:
- `PyEventStore` class exposing Rust functionality
- `PyEvent` and `PyAggregate` wrapper classes  
- Async support through `pyo3-asyncio`
- Error handling across language boundaries

#### 3. Python Package (`eventuali-python/python`)

**Pydantic Models**:
- `Event` base class with automatic serialization
- Domain event examples (`UserRegistered`, `UserEmailChanged`)
- `Aggregate` base class with event application
- Automatic method dispatch (`apply_*` methods)

**High-level APIs**:
- `EventStore` with connection string parsing
- Type-safe aggregate loading/saving interfaces
- Async/await support throughout

**Developer Experience**:
- Comprehensive type hints
- Pydantic validation and serialization
- Familiar Python patterns and idioms

#### 4. Testing & Examples

**Rust Tests**:
- Integration tests for both database backends
- Event serialization/deserialization tests
- Aggregate creation and manipulation tests

**Python Examples**:
- Basic usage demonstration
- Event creation and application
- Aggregate reconstruction from events
- State consistency verification

### ⏳ Partially Implemented

#### Event Store Integration
- ✅ Rust backend fully functional
- ✅ Python interface defined
- ⏳ Python-to-Rust operation bridging (in progress)

#### Serialization
- ✅ JSON serialization working
- ⏳ Protocol Buffers support (placeholder created)

### 🔮 Future Enhancements

#### Performance Features
- [ ] Protocol Buffers implementation
- [ ] Event compression
- [ ] Batch operations
- [ ] Connection pooling optimization
- [ ] Snapshot support

#### Advanced Features
- [ ] Projections/Read models
- [ ] Event streaming
- [ ] Sagas/Process managers
- [ ] Event upcasting/schema evolution
- [ ] Multi-tenant support

## Performance Characteristics

Based on research and initial implementation:

### Expected Performance Improvements
- **Serialization**: 20-30x faster than pure Python
- **Database operations**: 2-10x faster
- **Memory usage**: 8-20x more efficient
- **Concurrent throughput**: 2x better under load

### Benchmarking Strategy
```python
# Performance test structure
async def benchmark_event_storage():
    # Test event creation and serialization
    # Test database write performance
    # Test database read performance
    # Compare with pure Python implementation
```

## Database Schema

### Events Table
```sql
CREATE TABLE events (
    id UUID PRIMARY KEY,
    aggregate_id VARCHAR NOT NULL,
    aggregate_type VARCHAR NOT NULL,
    event_type VARCHAR NOT NULL,
    event_version INTEGER NOT NULL,
    aggregate_version BIGINT NOT NULL,
    event_data JSONB NOT NULL,          -- PostgreSQL
    event_data TEXT NOT NULL,           -- SQLite
    event_data_type VARCHAR NOT NULL DEFAULT 'json',
    metadata JSONB NOT NULL,            -- PostgreSQL  
    metadata TEXT NOT NULL,             -- SQLite
    timestamp TIMESTAMPTZ NOT NULL,     -- PostgreSQL
    timestamp TEXT NOT NULL,            -- SQLite
    UNIQUE(aggregate_id, aggregate_version)
);

-- Performance indexes
CREATE INDEX idx_events_aggregate_id ON events (aggregate_id);
CREATE INDEX idx_events_aggregate_type ON events (aggregate_type);
CREATE INDEX idx_events_timestamp ON events (timestamp);
```

## Configuration

### Database Connection Strings

```python
# PostgreSQL (production)
store = await EventStore.create(
    "postgresql://user:password@localhost:5432/events"
)

# SQLite (development)
store = await EventStore.create("sqlite://events.db")
store = await EventStore.create("sqlite://:memory:")  # In-memory
```

### Advanced Configuration
```python
# Custom table names and connection pooling
config = EventStoreConfig.postgres("postgresql://...")
config = config.with_table_name("custom_events")
config = config.with_max_connections(20)
```

## Usage Patterns

### 1. Define Domain Events
```python
from eventuali.event import DomainEvent

class OrderPlaced(DomainEvent):
    customer_id: str
    items: List[OrderItem]
    total_amount: Decimal
```

### 2. Create Aggregates
```python
from eventuali.aggregate import Aggregate

class Order(Aggregate):
    status: OrderStatus = OrderStatus.PENDING
    items: List[OrderItem] = []
    
    def apply_order_placed(self, event: OrderPlaced) -> None:
        self.status = OrderStatus.PLACED
        self.items = event.items
```

### 3. Business Operations
```python
async def place_order(store: EventStore, customer_id: str, items: List[OrderItem]):
    order = Order()
    event = OrderPlaced(customer_id=customer_id, items=items, total_amount=calculate_total(items))
    order.apply(event)
    await store.save(order)
    return order.id
```

## Development Workflow

### Building the Project
```bash
# Build Rust core
cargo build --release

# Build Python package with Maturin
cd eventuali-python
maturin develop  # Development
maturin build --release  # Production wheels
```

### Running Tests
```bash
# Rust tests
cargo test

# Python tests (once fully connected)
cd eventuali-python
pytest tests/
```

### Publishing
```bash
# Build wheels for multiple platforms
maturin build --release --target x86_64-unknown-linux-gnu
maturin build --release --target aarch64-apple-darwin
maturin build --release --target x86_64-pc-windows-msvc

# Publish to PyPI
maturin publish
```

## Design Decisions

### Why Hybrid Architecture?
1. **Performance**: Rust handles performance-critical operations
2. **Ecosystem**: Python provides rich ecosystem and ease of use  
3. **Type Safety**: Rust's type system catches errors at compile time
4. **Memory Safety**: No garbage collection pauses or memory leaks
5. **Developer Experience**: Python's dynamic nature for business logic

### Database Strategy
- **Multi-backend**: Supports different environments (dev vs prod)
- **SQLx**: Type-safe database operations with async support
- **Schema flexibility**: JSON for development, ProtoBuf for performance

### Serialization Strategy
- **JSON**: Human-readable, great tooling, good for development
- **Protocol Buffers**: Binary format, excellent performance and schema evolution

## Migration Path

For teams using pure Python event sourcing:

### Phase 1: Drop-in Replacement
1. Install Eventuali
2. Replace existing event store with `EventStore.create()`
3. Migrate events to inherit from `eventuali.Event`
4. Migrate aggregates to inherit from `eventuali.Aggregate`

### Phase 2: Performance Optimization  
1. Switch to Protocol Buffers for high-throughput events
2. Enable connection pooling
3. Add projections for read models

### Phase 3: Advanced Features
1. Implement event streaming
2. Add snapshot support
3. Set up multi-tenant architecture

This implementation provides a solid foundation for high-performance event sourcing in Python while maintaining familiar development patterns and offering significant performance improvements over pure Python solutions.