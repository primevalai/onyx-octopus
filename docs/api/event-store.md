# EventStore API Reference

The `EventStore` class is the central component for event persistence and retrieval in Eventuali, providing high-performance event sourcing with multi-database support.

## Class Definition

```python
class EventStore:
    """High-performance event store supporting PostgreSQL and SQLite."""
```

## Creation and Initialization

### `create(connection_string: str) -> EventStore` {async, classmethod}

Creates and initializes an event store instance.

**Parameters:**
- `connection_string: str` - Database connection string

**Connection String Formats:**
- **PostgreSQL**: `"postgresql://user:password@host:port/database"`
- **SQLite**: `"sqlite://path/to/database.db"` or `"sqlite://:memory:"` for in-memory

**Returns:** Initialized `EventStore` instance

**Examples:**

```python
# SQLite for development/testing
store = await EventStore.create("sqlite://:memory:")
store = await EventStore.create("sqlite://events.db")

# PostgreSQL for production
store = await EventStore.create("postgresql://user:pass@localhost/events")
```

**Raises:**
- `RuntimeError` - If initialization fails
- `DatabaseError` - If connection string is invalid

---

## Event Registration

### `register_event_class(event_type: str, event_class: Type[Event])` {classmethod}

Registers a custom event class for automatic deserialization.

**Parameters:**
- `event_type: str` - The event type identifier
- `event_class: Type[Event]` - Python class for this event type

**Examples:**

```python
# Register custom events
class OrderCreated(Event):
    customer_id: str
    amount: Decimal

EventStore.register_event_class("OrderCreated", OrderCreated)
EventStore.register_event_class("order.created", OrderCreated)
```

**Raises:**
- `ValueError` - If event_class is not a subclass of Event

### `unregister_event_class(event_type: str)` {classmethod}

Unregisters a custom event class.

**Parameters:**
- `event_type: str` - The event type to unregister

### `get_registered_event_classes() -> Dict[str, Type[Event]]` {classmethod}

Returns a copy of all registered event classes.

**Returns:** Dictionary mapping event types to their registered classes

---

## Aggregate Operations

### `save(aggregate: Aggregate)` {async}

Saves an aggregate and its uncommitted events to the event store.

**Parameters:**
- `aggregate: Aggregate` - The aggregate to save

**Examples:**

```python
# Create and save user
user = User(id="user-123")
user.apply(UserRegistered(name="John", email="john@example.com"))
await store.save(user)

# Update and save
user.apply(UserEmailChanged(new_email="john.doe@example.com"))
await store.save(user)
```

**Raises:**
- `OptimisticConcurrencyError` - If aggregate was modified by another process
- `EventStoreError` - If save operation fails

**Performance:**
- **25,000+ saves/sec** typical throughput
- Atomic transaction per aggregate
- Optimistic concurrency control

### `load(aggregate_class: Type[T], aggregate_id: str) -> Optional[T]` {async}

Loads an aggregate from the event store by ID.

**Parameters:**
- `aggregate_class: Type[T]` - The aggregate class to instantiate
- `aggregate_id: str` - Unique identifier of the aggregate

**Returns:** The loaded aggregate, or `None` if not found

**Examples:**

```python
# Load existing user
user = await store.load(User, "user-123")
if user:
    print(f"Loaded user: {user.name} ({user.email})")

# Type-safe loading
order: Optional[Order] = await store.load(Order, "order-456")
```

**Performance:**
- **40,000+ loads/sec** typical throughput
- Event replay with optimized reconstruction
- Automatic event deserialization

---

## Event Operations

### `load_events(aggregate_id: str, from_version: Optional[int] = None) -> List[Event]` {async}

Loads events for a specific aggregate.

**Parameters:**
- `aggregate_id: str` - The aggregate identifier
- `from_version: Optional[int]` - Optional version to start loading from

**Returns:** List of events ordered by version

**Examples:**

```python
# Load all events for an aggregate
events = await store.load_events("user-123")

# Load events from specific version (for incremental loading)
recent_events = await store.load_events("user-123", from_version=10)

# Event replay debugging
for event in events:
    print(f"Version {event.aggregate_version}: {event.event_type}")
```

### `load_events_by_type(aggregate_type: str, from_version: Optional[int] = None) -> List[Event]` {async}

Loads all events for a specific aggregate type.

**Parameters:**
- `aggregate_type: str` - The type of aggregate
- `from_version: Optional[int]` - Optional version to start loading from

**Returns:** List of events ordered by timestamp

**Examples:**

```python
# Load all user events across all users
user_events = await store.load_events_by_type("User")

# Load recent order events for analytics
recent_orders = await store.load_events_by_type("Order", from_version=1000)
```

### `get_aggregate_version(aggregate_id: str) -> Optional[int]` {async}

Gets the current version of an aggregate without loading events.

**Parameters:**
- `aggregate_id: str` - The aggregate identifier

**Returns:** Current version, or `None` if aggregate doesn't exist

**Examples:**

```python
# Check if aggregate exists and get version
version = await store.get_aggregate_version("user-123")
if version is not None:
    print(f"User exists at version {version}")

# Optimistic concurrency check
current_version = await store.get_aggregate_version(user.id)
if current_version != user.version:
    # Handle concurrency conflict
    pass
```

---

## Error Handling

### Exception Hierarchy

All EventStore operations may raise these exceptions:

```python
# Base exception
EventualiError
├── EventStoreError              # General store errors
├── OptimisticConcurrencyError   # Concurrent modification
├── DatabaseError                # Database connection issues
├── SerializationError           # Event serialization failures
└── AggregateNotFoundError       # Aggregate not found
```

### Error Handling Patterns

```python
from eventuali.exceptions import OptimisticConcurrencyError, EventStoreError

async def safe_save_with_retry(store: EventStore, aggregate: Aggregate, max_retries: int = 3):
    for attempt in range(max_retries):
        try:
            await store.save(aggregate)
            return  # Success
        except OptimisticConcurrencyError:
            if attempt == max_retries - 1:
                raise  # Final attempt failed
            # Reload and retry
            fresh_aggregate = await store.load(type(aggregate), aggregate.id)
            if fresh_aggregate:
                aggregate = fresh_aggregate
        except EventStoreError as e:
            logger.error(f"Store error on attempt {attempt + 1}: {e}")
            if attempt == max_retries - 1:
                raise
```

---

## Performance Characteristics

### Throughput Benchmarks

| Operation | Performance | Database |
|-----------|-------------|----------|
| **Event Creation** | 79,000+ events/sec | SQLite |
| **Event Persistence** | 25,000+ events/sec | SQLite |
| **Event Loading** | 40,000+ events/sec | SQLite |
| **Aggregate Reconstruction** | 18.3x faster | vs Pure Python |

### Memory Efficiency

- **8-20x lower memory usage** compared to pure Python event sourcing
- **Lazy loading** of events during aggregate reconstruction
- **Connection pooling** with automatic scaling

### Database Support

| Feature | SQLite | PostgreSQL |
|---------|--------|------------|
| **Development** | ✅ Recommended | ✅ Supported |
| **Production** | ✅ Single-node | ✅ Recommended |
| **Transactions** | ✅ ACID | ✅ ACID |
| **Performance** | ✅ High | ✅ Very High |
| **Scalability** | ⚠️ Single file | ✅ Horizontal |

---

## Thread Safety

The `EventStore` class is **async-safe** but not thread-safe. Use one instance per async context:

```python
# ✅ Correct: One store per async context
async def handle_request():
    store = await EventStore.create("postgresql://...")
    # Use store in this async context
    
# ✅ Correct: Shared store across async operations
store = await EventStore.create("postgresql://...")

async def operation_a():
    await store.save(aggregate_a)

async def operation_b():
    await store.save(aggregate_b)

# Run concurrently - this is safe
await asyncio.gather(operation_a(), operation_b())
```

---

## Integration Examples

### With FastAPI

```python
from fastapi import FastAPI, Depends
from eventuali import EventStore

app = FastAPI()

async def get_event_store() -> EventStore:
    """Dependency injection for EventStore."""
    return await EventStore.create("postgresql://user:pass@localhost/events")

@app.post("/users")
async def create_user(
    user_data: UserCreateRequest,
    store: EventStore = Depends(get_event_store)
):
    user = User(id=str(uuid4()))
    user.apply(UserRegistered(name=user_data.name, email=user_data.email))
    await store.save(user)
    return {"user_id": user.id}
```

### With Dependency Injection

```python
from dependency_injector import containers, providers
from eventuali import EventStore

class Container(containers.DeclarativeContainer):
    event_store = providers.Resource(
        EventStore.create,
        connection_string="postgresql://user:pass@localhost/events"
    )
```

---

## Related Documentation

- **[Event API](event.md)** - Event class documentation
- **[Aggregate API](aggregate.md)** - Aggregate pattern implementation
- **[Streaming API](streaming/event-streamer.md)** - Real-time event streaming
- **[Performance Guide](../performance/README.md)** - Optimization patterns
- **[Examples](../../examples/README.md)** - Working examples

---

**Next**: Explore the [Event API](event.md) or see [practical examples](../../examples/01_basic_event_store_simple.py).