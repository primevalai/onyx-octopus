# Event API Reference

The `Event` class is the base class for all domain events in Eventuali, providing immutable records of business occurrences with rich metadata support.

## Class Hierarchy

```python
Event (ABC)
├── DomainEvent     # Business logic events
└── SystemEvent     # Infrastructure events
```

## Base Event Class

### `Event(BaseModel, ABC)`

Abstract base class for all events in the system.

**Key Principles:**
- **Immutable**: Events represent facts that cannot be changed
- **Past Tense Naming**: Events should be named as completed actions (e.g., `UserRegistered`, `OrderShipped`)
- **Rich Metadata**: Automatic correlation, causation, and timing information
- **Type Safety**: Full Pydantic validation and type hints

---

## Core Fields

### Identification Fields

```python
event_id: Optional[UUID]          # Unique event identifier (auto-generated)
aggregate_id: Optional[str]       # ID of the aggregate that generated this event
aggregate_type: Optional[str]     # Type of the aggregate (e.g., "User", "Order")
event_type: Optional[str]         # Type of the event (auto-derived from class name)
```

### Versioning Fields

```python
event_version: Optional[int] = 1      # Schema version of the event
aggregate_version: Optional[int]      # Version of aggregate after this event
timestamp: Optional[datetime]         # When the event occurred (auto-generated)
```

### Correlation Fields

```python
causation_id: Optional[UUID]      # ID of the event that caused this event
correlation_id: Optional[UUID]    # ID correlating related events across aggregates
user_id: Optional[str]           # ID of the user who triggered this event
```

---

## Core Methods

### `get_event_type() -> str` {classmethod}

Returns the event type name, typically the class name.

```python
class OrderCreated(DomainEvent):
    customer_id: str
    amount: Decimal

assert OrderCreated.get_event_type() == "OrderCreated"
```

### Serialization Methods

#### `to_json() -> str`

Converts the event to a JSON string with proper encoding.

```python
event = UserRegistered(name="John", email="john@example.com")
json_str = event.to_json()
```

#### `from_json(json_str: str) -> Event` {classmethod}

Creates an event instance from a JSON string.

```python
event = UserRegistered.from_json(json_str)
```

#### `to_dict() -> Dict[str, Any]`

Converts the event to a dictionary.

```python
event_dict = event.to_dict()
```

#### `from_dict(data: Dict[str, Any]) -> Event` {classmethod}

Creates an event instance from a dictionary.

```python
event = UserRegistered.from_dict(event_dict)
```

---

## Event Types

### DomainEvent

Base class for business logic events that represent meaningful business occurrences.

```python
class DomainEvent(Event):
    """
    Base class for domain events that are part of the business logic.
    
    Domain events represent meaningful business occurrences and are typically
    the result of executing a command on an aggregate.
    """
```

**Examples:**

```python
class OrderPlaced(DomainEvent):
    customer_id: str
    items: List[OrderItem]
    total_amount: Decimal

class PaymentProcessed(DomainEvent):
    order_id: str
    payment_method: str
    amount: Decimal
    transaction_id: str

class InventoryReserved(DomainEvent):
    product_id: str
    quantity: int
    reservation_id: str
```

### SystemEvent

Base class for infrastructure events that are not part of core business logic.

```python
class SystemEvent(Event):
    """
    Base class for system events that are not part of the core business logic.
    
    System events represent technical occurrences like aggregate snapshots,
    migrations, or other infrastructure concerns.
    """
```

**Examples:**

```python
class AggregateSnapshotCreated(SystemEvent):
    aggregate_id: str
    snapshot_data: dict
    compression_ratio: float

class EventMigrationCompleted(SystemEvent):
    from_version: int
    to_version: int
    events_migrated: int
```

---

## Built-in Domain Events

Eventuali provides several built-in domain events for common patterns:

### UserRegistered

```python
class UserRegistered(DomainEvent):
    """Event fired when a user registers."""
    name: str
    email: str
```

### UserEmailChanged

```python
class UserEmailChanged(DomainEvent):
    """Event fired when a user changes their email."""
    old_email: str
    new_email: str
```

### UserDeactivated

```python
class UserDeactivated(DomainEvent):
    """Event fired when a user account is deactivated."""
    reason: Optional[str] = None
```

---

## Event Creation Patterns

### Simple Event Creation

```python
# Create event with required fields
event = UserRegistered(name="John Doe", email="john@example.com")

# Metadata is automatically generated
assert event.event_id is not None
assert event.timestamp is not None
assert event.event_type == "UserRegistered"
```

### Event with Correlation

```python
# Create correlated events
correlation_id = uuid4()

# First event in a workflow
payment_event = PaymentRequested(
    order_id="order-123",
    amount=Decimal("99.99"),
    correlation_id=correlation_id
)

# Related event in the same workflow
fulfillment_event = OrderShipped(
    order_id="order-123",
    tracking_number="TRK-456",
    correlation_id=correlation_id,
    causation_id=payment_event.event_id  # This event caused by payment
)
```

### Event with User Context

```python
# Event with user attribution
event = UserEmailChanged(
    old_email="john@example.com",
    new_email="john.doe@company.com",
    user_id="user-123"  # Track who made the change
)
```

---

## Custom Event Creation

### Simple Custom Event

```python
from eventuali import DomainEvent
from decimal import Decimal
from typing import List

class ProductAddedToCart(DomainEvent):
    """Event fired when a product is added to shopping cart."""
    product_id: str
    quantity: int
    price: Decimal
    cart_id: str

# Usage
event = ProductAddedToCart(
    product_id="prod-123",
    quantity=2,
    price=Decimal("29.99"),
    cart_id="cart-456"
)
```

### Complex Custom Event

```python
from pydantic import Field, validator
from typing import Optional, List
from enum import Enum

class OrderStatus(str, Enum):
    PENDING = "pending"
    CONFIRMED = "confirmed"
    SHIPPED = "shipped"
    DELIVERED = "delivered"

class ShippingAddress(BaseModel):
    street: str
    city: str
    state: str
    zip_code: str
    country: str = "US"

class OrderStatusChanged(DomainEvent):
    """Event fired when order status changes."""
    order_id: str
    previous_status: OrderStatus
    new_status: OrderStatus
    shipping_address: Optional[ShippingAddress] = None
    tracking_number: Optional[str] = None
    notes: Optional[str] = None

    @validator('new_status')
    def validate_status_transition(cls, v, values):
        """Ensure valid status transitions."""
        previous = values.get('previous_status')
        if previous == OrderStatus.DELIVERED and v != OrderStatus.DELIVERED:
            raise ValueError("Cannot change status from delivered")
        return v

# Usage
event = OrderStatusChanged(
    order_id="order-789",
    previous_status=OrderStatus.CONFIRMED,
    new_status=OrderStatus.SHIPPED,
    shipping_address=ShippingAddress(
        street="123 Main St",
        city="Anytown",
        state="CA",
        zip_code="12345"
    ),
    tracking_number="1Z999AA1234567890"
)
```

---

## Event Registration

### Automatic Registration

Events are automatically registered with the EventStore when they inherit from `Event`:

```python
# Automatically registered by class name
class OrderCancelled(DomainEvent):
    order_id: str
    reason: str

# Can be loaded automatically
events = await store.load_events("order-123")
```

### Manual Registration

For custom event type names or complex scenarios:

```python
from eventuali import EventStore

# Register with custom event type name
EventStore.register_event_class("order.cancelled", OrderCancelled)
EventStore.register_event_class("order.cancelled.v2", OrderCancelledV2)

# Check registrations
registered = EventStore.get_registered_event_classes()
print(registered["order.cancelled"])  # <class 'OrderCancelled'>
```

---

## Serialization and JSON Handling

### JSON Configuration

Events use custom JSON encoders for proper serialization:

```python
# Automatic handling of special types
event = OrderCreated(
    customer_id="cust-123",
    order_date=datetime.utcnow(),
    order_id=uuid4(),
    total=Decimal("199.99")
)

# Serializes to clean JSON
json_str = event.to_json()
# {
#   "customer_id": "cust-123",
#   "order_date": "2024-01-15T10:30:00",
#   "order_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
#   "total": "199.99"
# }
```

### Extra Fields Preservation

Events preserve unknown fields during deserialization for forward compatibility:

```python
# Event with extra field from future version
event_data = {
    "name": "John Doe",
    "email": "john@example.com",
    "future_field": "some_value"  # Unknown field
}

# Loads successfully, preserving extra field
event = UserRegistered.from_dict(event_data)
assert hasattr(event, 'future_field')
```

---

## Best Practices

### Event Naming

```python
# ✅ Good: Past tense, specific
class UserEmailVerified(DomainEvent):
    pass

class PaymentAuthorized(DomainEvent):
    pass

# ❌ Bad: Present tense, vague
class UserEmailVerify(DomainEvent):  # Should be past tense
    pass

class UserUpdate(DomainEvent):  # Too vague
    pass
```

### Event Granularity

```python
# ✅ Good: Specific, focused events
class UserEmailChanged(DomainEvent):
    old_email: str
    new_email: str

class UserNameChanged(DomainEvent):
    old_name: str
    new_name: str

# ❌ Bad: Generic, hard to handle
class UserUpdated(DomainEvent):
    changes: dict  # What changed?
```

### Event Data

```python
# ✅ Good: Include relevant business data
class OrderPlaced(DomainEvent):
    customer_id: str
    items: List[OrderItem]
    total_amount: Decimal
    shipping_address: Address
    payment_method: str

# ❌ Bad: Missing context
class OrderPlaced(DomainEvent):
    order_id: str  # Too minimal, requires additional queries
```

---

## Error Handling

### Validation Errors

```python
from pydantic import ValidationError

try:
    event = UserRegistered(
        name="",  # Invalid: empty name
        email="invalid-email"  # Invalid: bad email format
    )
except ValidationError as e:
    print(f"Validation error: {e}")
```

### Serialization Errors

```python
from eventuali.exceptions import SerializationError

try:
    # Malformed JSON
    event = UserRegistered.from_json('{"name": "John"')  # Missing closing brace
except SerializationError as e:
    print(f"Failed to deserialize: {e}")
```

---

## Performance Considerations

### Event Size

- **Keep events focused**: Include only essential business data
- **Avoid large payloads**: Use references for large objects
- **Consider compression**: Built-in support for large event compression

```python
# ✅ Good: Focused event
class UserAvatarChanged(DomainEvent):
    user_id: str
    avatar_url: str  # Reference to stored image
    previous_avatar_url: Optional[str]

# ❌ Bad: Large payload
class UserAvatarChanged(DomainEvent):
    user_id: str
    avatar_data: bytes  # Don't embed large binary data
```

### Event Creation Performance

- **79,000+ events/sec** creation speed
- **Lazy evaluation** of computed fields
- **Minimal memory allocation** for high-throughput scenarios

---

## Related Documentation

- **[EventStore API](event-store.md)** - Event persistence and retrieval
- **[Aggregate API](aggregate.md)** - Event application to aggregates
- **[Streaming API](streaming/event-streamer.md)** - Real-time event processing
- **[Examples](../../examples/README.md)** - Working event examples

---

**Next**: Explore the [Aggregate API](aggregate.md) or see [event examples](../../examples/01_basic_event_store_simple.py).