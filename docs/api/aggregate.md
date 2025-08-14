# Aggregate API Reference

The `Aggregate` class is the foundational base class for all domain aggregates in Eventuali, implementing the aggregate root pattern with event sourcing capabilities.

## Class Definition

```python
class Aggregate(BaseModel, ABC):
    """
    Base class for all aggregates in the domain.
    
    An aggregate is a cluster of domain objects that can be treated as a single unit.
    It maintains business invariants and generates events when its state changes.
    """
```

## Core Concepts

### Aggregate Root Pattern

An aggregate is a cluster of related objects that are treated as a single unit for data changes. The aggregate root is the only member of the aggregate that outside objects are allowed to hold references to.

**Key Principles:**
- **Consistency Boundary**: Aggregates enforce business invariants
- **Event Generation**: State changes produce domain events
- **Encapsulation**: Business logic is contained within the aggregate
- **Identity**: Each aggregate has a unique identifier

---

## Core Fields

### Identity and Versioning

```python
id: str                           # Unique aggregate identifier (auto-generated UUID)
version: int = 0                  # Current version (incremented with each event)
```

### Event Sourcing Fields

```python
uncommitted_events: List[Event]  # Events not yet persisted (private)
is_new_flag: bool = True         # Whether aggregate is new (private)
```

---

## Class Methods

### `get_aggregate_type() -> str` {classmethod}

Returns the aggregate type name, typically the class name.

```python
class Order(Aggregate):
    pass

assert Order.get_aggregate_type() == "Order"
```

### `from_events(events: List[Event]) -> Aggregate` {classmethod}

Creates an aggregate by replaying a list of events in order.

**Parameters:**
- `events: List[Event]` - Events ordered by version

**Returns:** Aggregate with state reconstructed from events

**Examples:**

```python
# Load aggregate from event history
events = await store.load_events("user-123")
user = User.from_events(events)

# Verify state reconstruction
assert user.id == "user-123"
assert user.version == len(events)
```

**Raises:**
- `ValueError` - If events list is empty or first event lacks aggregate_id

---

## Instance Methods

### Event Management

#### `apply(event: Event)`

Applies an event to the aggregate and adds it to uncommitted events. Used when generating new events.

**Parameters:**
- `event: Event` - The event to apply

**Examples:**

```python
# Generate and apply new event
user = User(id="user-123")
event = UserRegistered(name="John", email="john@example.com")
user.apply(event)

# Event is now uncommitted
assert len(user.get_uncommitted_events()) == 1
assert user.version == 1
```

#### `mark_events_as_committed()`

Marks all uncommitted events as committed and clears the uncommitted events list.

```python
# After saving to store
await store.save(user)
user.mark_events_as_committed()

assert len(user.get_uncommitted_events()) == 0
assert not user.has_uncommitted_events()
```

#### `get_uncommitted_events() -> List[Event]`

Returns a copy of all uncommitted events.

```python
events = user.get_uncommitted_events()
for event in events:
    print(f"Uncommitted: {event.event_type}")
```

#### `has_uncommitted_events() -> bool`

Checks if the aggregate has any uncommitted events.

```python
if user.has_uncommitted_events():
    await store.save(user)
    user.mark_events_as_committed()
```

### State Management

#### `is_new() -> bool`

Checks if this is a new aggregate that has never been persisted.

```python
new_user = User(id="user-456")
assert new_user.is_new()

# After loading from store
loaded_user = await store.load(User, "user-123")
assert not loaded_user.is_new()
```

### Serialization

#### `to_dict() -> Dict[str, Any]`

Converts the aggregate to a dictionary, excluding private event sourcing fields.

```python
user_data = user.to_dict()
# Contains: id, version, name, email, is_active
# Excludes: uncommitted_events, is_new_flag
```

#### `from_dict(data: Dict[str, Any]) -> Aggregate` {classmethod}

Creates an aggregate from a dictionary.

```python
user = User.from_dict({
    "id": "user-123",
    "version": 5,
    "name": "John Doe",
    "email": "john@example.com",
    "is_active": True
})
```

---

## Event Handler Pattern

### Automatic Event Dispatch

The aggregate automatically dispatches events to handler methods based on event type:

```python
class Order(Aggregate):
    status: str = "draft"
    
    def apply_order_created(self, event: OrderCreated):
        """Handler for OrderCreated event."""
        self.status = "pending"
        # Handler methods update state only
    
    def apply_order_shipped(self, event: OrderShipped):
        """Handler for OrderShipped event."""
        self.status = "shipped"
```

### Handler Method Convention

- **Pattern**: `apply_{event_name_in_snake_case}`
- **Event Type**: `OrderCreated` → Method: `apply_order_created`
- **Event Type**: `UserEmailChanged` → Method: `apply_user_email_changed`

### Event Name Conversion

The aggregate automatically converts PascalCase event names to snake_case method names:

```python
# Event type → Method name mappings
"UserRegistered"     → "apply_user_registered"
"OrderPlaced"        → "apply_order_placed"
"PaymentProcessed"   → "apply_payment_processed"
"InventoryReserved"  → "apply_inventory_reserved"
```

---

## Implementation Patterns

### Complete Aggregate Example

```python
class BankAccount(Aggregate):
    """Bank account aggregate with business logic."""
    
    def __init__(self, **data):
        super().__init__(**data)
        self.account_holder: str = ""
        self.balance: Decimal = Decimal('0.00')
        self.is_active: bool = False
        self.overdraft_limit: Decimal = Decimal('0.00')
    
    # Business methods (commands)
    def open_account(self, holder: str, initial_deposit: Decimal):
        """Open a new bank account."""
        if self.is_active:
            raise ValueError("Account already open")
        
        if initial_deposit < 0:
            raise ValueError("Initial deposit must be non-negative")
        
        event = AccountOpened(
            account_holder=holder,
            initial_deposit=initial_deposit
        )
        self.apply(event)
    
    def deposit(self, amount: Decimal, description: str = ""):
        """Deposit money into the account."""
        if not self.is_active:
            raise ValueError("Cannot deposit to inactive account")
        
        if amount <= 0:
            raise ValueError("Deposit amount must be positive")
        
        event = MoneyDeposited(
            amount=amount,
            description=description
        )
        self.apply(event)
    
    def withdraw(self, amount: Decimal, description: str = ""):
        """Withdraw money from the account."""
        if not self.is_active:
            raise ValueError("Cannot withdraw from inactive account")
        
        if amount <= 0:
            raise ValueError("Withdrawal amount must be positive")
        
        # Check business rules
        available_funds = self.balance + self.overdraft_limit
        if amount > available_funds:
            raise InsufficientFundsError(
                available=float(available_funds),
                requested=float(amount)
            )
        
        event = MoneyWithdrawn(
            amount=amount,
            description=description
        )
        self.apply(event)
    
    def close_account(self, reason: str = ""):
        """Close the account."""
        if not self.is_active:
            return  # Already closed
        
        if self.balance != 0:
            raise ValueError("Cannot close account with non-zero balance")
        
        event = AccountClosed(reason=reason)
        self.apply(event)
    
    # Event handlers (apply state changes)
    def apply_account_opened(self, event: AccountOpened):
        """Handle AccountOpened event."""
        self.account_holder = event.account_holder
        self.balance = event.initial_deposit
        self.is_active = True
    
    def apply_money_deposited(self, event: MoneyDeposited):
        """Handle MoneyDeposited event."""
        self.balance += event.amount
    
    def apply_money_withdrawn(self, event: MoneyWithdrawn):
        """Handle MoneyWithdrawn event."""
        self.balance -= event.amount
    
    def apply_account_closed(self, event: AccountClosed):
        """Handle AccountClosed event."""
        self.is_active = False
    
    # Query methods (read-only)
    def get_available_funds(self) -> Decimal:
        """Get total available funds including overdraft."""
        return self.balance + self.overdraft_limit
    
    def can_withdraw(self, amount: Decimal) -> bool:
        """Check if withdrawal amount is allowed."""
        return self.is_active and amount <= self.get_available_funds()
```

### Business Logic Patterns

```python
class User(Aggregate):
    """User aggregate with validation and business rules."""
    
    name: str = ""
    email: str = ""
    is_active: bool = True
    subscription_tier: str = "basic"
    
    # Command methods with validation
    def register(self, name: str, email: str):
        """Register a new user."""
        if self.name:  # Already registered
            raise ValueError("User already registered")
        
        # Validate inputs
        if not name or not name.strip():
            raise ValueError("Name is required")
        
        if not email or '@' not in email:
            raise ValueError("Valid email is required")
        
        # Generate event
        event = UserRegistered(name=name.strip(), email=email.lower())
        self.apply(event)
    
    def change_email(self, new_email: str):
        """Change user's email with validation."""
        if not self.is_active:
            raise ValueError("Cannot change email for inactive user")
        
        if not new_email or '@' not in new_email:
            raise ValueError("Valid email is required")
        
        new_email = new_email.lower().strip()
        if new_email == self.email:
            return  # No change needed
        
        event = UserEmailChanged(
            old_email=self.email,
            new_email=new_email
        )
        self.apply(event)
    
    def upgrade_subscription(self, new_tier: str):
        """Upgrade user subscription."""
        valid_tiers = ["basic", "premium", "enterprise"]
        if new_tier not in valid_tiers:
            raise ValueError(f"Invalid tier. Must be one of: {valid_tiers}")
        
        if new_tier == self.subscription_tier:
            return  # No change needed
        
        event = SubscriptionUpgraded(
            old_tier=self.subscription_tier,
            new_tier=new_tier
        )
        self.apply(event)
    
    # Event handlers
    def apply_user_registered(self, event: UserRegistered):
        """Handle user registration."""
        self.name = event.name
        self.email = event.email
        self.is_active = True
        self.subscription_tier = "basic"
    
    def apply_user_email_changed(self, event: UserEmailChanged):
        """Handle email change."""
        self.email = event.new_email
    
    def apply_subscription_upgraded(self, event: SubscriptionUpgraded):
        """Handle subscription upgrade."""
        self.subscription_tier = event.new_tier
    
    def apply_user_deactivated(self, event: UserDeactivated):
        """Handle user deactivation."""
        self.is_active = False
```

---

## Error Handling

### Common Exceptions

```python
# Business rule violations
class BusinessRuleViolation(Exception):
    pass

class InsufficientFundsError(BusinessRuleViolation):
    def __init__(self, available: float, requested: float):
        self.available = available
        self.requested = requested
        super().__init__(f"Insufficient funds: {available} available, {requested} requested")

# Usage in aggregate
def withdraw(self, amount: Decimal):
    if amount > self.balance:
        raise InsufficientFundsError(
            available=float(self.balance),
            requested=float(amount)
        )
```

### Missing Event Handler

```python
# Aggregate raises NotImplementedError if handler is missing
class Order(Aggregate):
    def apply_order_created(self, event):
        pass
    
    # Missing: apply_order_shipped method

order = Order()
try:
    order.apply(OrderShipped())  # Will raise NotImplementedError
except NotImplementedError as e:
    print(f"Missing handler: {e}")
    # "No apply method found for event OrderShipped. Expected method: apply_order_shipped"
```

---

## Performance Considerations

### Event Application Performance

- **Event Dispatch**: O(1) method lookup via `hasattr` and `getattr`
- **State Updates**: Direct field assignment for optimal performance
- **Memory Usage**: Minimal overhead with Pydantic optimization

### Large Aggregate Optimization

```python
class HighVolumeAggregate(Aggregate):
    """Optimized aggregate for high event volumes."""
    
    def __init__(self, **data):
        super().__init__(**data)
        # Pre-allocate known collections
        self.items = []
        self.computed_cache = {}
    
    def _clear_cache(self):
        """Clear computed values when state changes."""
        self.computed_cache.clear()
    
    def apply_item_added(self, event):
        """Optimized event handler."""
        self.items.append(event.item)
        self._clear_cache()  # Invalidate computed values
    
    @property
    def total_value(self) -> Decimal:
        """Cached computation for performance."""
        if 'total_value' not in self.computed_cache:
            self.computed_cache['total_value'] = sum(
                item.value for item in self.items
            )
        return self.computed_cache['total_value']
```

### Snapshot Integration

```python
# Aggregates work seamlessly with snapshots
from eventuali import SnapshotService

async def load_with_snapshots(store, aggregate_class, aggregate_id):
    """Load aggregate with snapshot optimization."""
    snapshot_service = SnapshotService(store)
    
    # Load from snapshot + recent events (10-20x faster)
    return await snapshot_service.load_with_snapshot(
        aggregate_class, aggregate_id
    )
```

---

## Testing Patterns

### Unit Testing Aggregates

```python
import pytest
from decimal import Decimal

def test_bank_account_deposit():
    """Test deposit functionality."""
    account = BankAccount(id="acc-123")
    account.open_account("John Doe", Decimal('100.00'))
    
    # Test deposit
    account.deposit(Decimal('50.00'), "Test deposit")
    
    # Verify state
    assert account.balance == Decimal('150.00')
    assert len(account.get_uncommitted_events()) == 2
    
    # Verify events
    events = account.get_uncommitted_events()
    assert events[0].event_type == "AccountOpened"
    assert events[1].event_type == "MoneyDeposited"
    assert events[1].amount == Decimal('50.00')

def test_insufficient_funds():
    """Test insufficient funds handling."""
    account = BankAccount(id="acc-456")
    account.open_account("Jane Doe", Decimal('50.00'))
    
    # Test insufficient funds
    with pytest.raises(InsufficientFundsError) as excinfo:
        account.withdraw(Decimal('100.00'))
    
    assert excinfo.value.available == 50.0
    assert excinfo.value.requested == 100.0

def test_event_replay():
    """Test aggregate reconstruction from events."""
    # Create events
    events = [
        AccountOpened(
            aggregate_id="acc-789",
            account_holder="Bob Smith",
            initial_deposit=Decimal('200.00')
        ),
        MoneyDeposited(
            aggregate_id="acc-789", 
            amount=Decimal('100.00')
        ),
        MoneyWithdrawn(
            aggregate_id="acc-789",
            amount=Decimal('50.00')
        )
    ]
    
    # Reconstruct aggregate
    account = BankAccount.from_events(events)
    
    # Verify final state
    assert account.id == "acc-789"
    assert account.account_holder == "Bob Smith"
    assert account.balance == Decimal('250.00')
    assert account.version == 3
    assert not account.is_new()
```

---

## Related Documentation

- **[EventStore API](event-store.md)** - Persistence and loading
- **[Event API](event.md)** - Event definitions and patterns
- **[Streaming API](streaming/event-streamer.md)** - Real-time processing
- **[Examples](../../examples/README.md)** - Working aggregate examples

---

**Next**: Explore the [EventStore API](event-store.md) for persistence patterns or see [aggregate examples](../../examples/02_aggregate_lifecycle.py) in action.