# Quick Start Guide

**Get up and running with Eventuali in 5 minutes**

This guide walks you through the fundamentals of event sourcing with Eventuali using our proven basic example.

## 🎯 What You'll Learn

By the end of this guide, you'll understand:
- ✅ Event sourcing concepts with practical examples
- ✅ Creating and persisting domain events
- ✅ Building aggregates with business logic
- ✅ Loading and reconstructing aggregate state
- ✅ Performance characteristics (79k+ events/sec)

## 📋 Prerequisites

```bash
# Install UV (modern Python package manager)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Verify Python 3.8+ is available
python --version  # Should be 3.8 or higher
```

## 🚀 Installation

### Option 1: Use Published Package (Recommended)

```bash
# Create new project
mkdir my-event-sourced-app
cd my-event-sourced-app

# Initialize with UV
uv init
uv add eventuali

# Verify installation
uv run python -c "import eventuali; print('✅ Eventuali ready!')"
```

### Option 2: Build from Source

```bash
# Clone repository
git clone https://github.com/primevalai/onyx-octopus.git
cd onyx-octopus/eventuali-python

# Install dependencies and tools
uv sync
uv tool install maturin

# Build Python bindings
uv run maturin develop

# Test the installation
uv run python ../examples/01_basic_event_store_simple.py
```

## 🏗️ Your First Event Store

Let's build a simple user management system with event sourcing.

### Step 1: Define Domain Events

Create `events.py`:

```python
from eventuali import DomainEvent
from typing import Optional

class UserRegistered(DomainEvent):
    """Event fired when a user registers."""
    name: str
    email: str

class UserEmailChanged(DomainEvent):
    """Event fired when a user changes their email."""
    old_email: str
    new_email: str

class UserDeactivated(DomainEvent):
    """Event fired when a user account is deactivated."""
    reason: Optional[str] = None
```

**Key Concepts:**
- **Past Tense Naming**: Events represent facts that already happened
- **Immutable Data**: Events cannot be changed once created
- **Rich Context**: Include all relevant business information

### Step 2: Create an Aggregate

Create `user.py`:

```python
from eventuali import Aggregate
from events import UserRegistered, UserEmailChanged, UserDeactivated

class User(Aggregate):
    """User aggregate with event sourcing."""
    
    def __init__(self, id: str = None):
        super().__init__(id)
        self.name: str = ""
        self.email: str = ""
        self.is_active: bool = True
    
    # Business methods that generate events
    def register(self, name: str, email: str):
        """Register a new user."""
        if self.name:  # Already registered
            raise ValueError("User already registered")
            
        event = UserRegistered(name=name, email=email)
        self.apply(event)
    
    def change_email(self, new_email: str):
        """Change user's email address."""
        if not self.is_active:
            raise ValueError("Cannot change email for inactive user")
            
        if self.email == new_email:
            return  # No change needed
            
        event = UserEmailChanged(old_email=self.email, new_email=new_email)
        self.apply(event)
    
    def deactivate(self, reason: str = None):
        """Deactivate user account."""
        if not self.is_active:
            return  # Already inactive
            
        event = UserDeactivated(reason=reason)
        self.apply(event)
    
    # Event handlers that update state
    def apply_user_registered(self, event: UserRegistered):
        """Handle UserRegistered event."""
        self.name = event.name
        self.email = event.email
        self.is_active = True
    
    def apply_user_email_changed(self, event: UserEmailChanged):
        """Handle UserEmailChanged event."""
        self.email = event.new_email
    
    def apply_user_deactivated(self, event: UserDeactivated):
        """Handle UserDeactivated event."""
        self.is_active = False
```

**Key Concepts:**
- **Aggregate Root**: Central entity that enforces business rules
- **Command Methods**: Public methods that validate and generate events
- **Event Handlers**: Private methods that apply events to state
- **Consistency**: All state changes happen through events

### Step 3: Use the Event Store

Create `main.py`:

```python
import asyncio
from eventuali import EventStore
from user import User

async def main():
    print("=== Eventuali Quick Start ===\\n")
    
    # 1. Create event store (SQLite for simplicity)
    print("1. Creating event store...")
    store = await EventStore.create("sqlite://:memory:")
    print("   ✅ Event store created with in-memory SQLite")
    
    # 2. Create and register a new user
    print("\\n2. Creating a new user...")
    user = User(id="user-123")
    user.register(name="Alice Johnson", email="alice@example.com")
    
    print(f"   ✅ User created: {user.name} ({user.email})")
    print(f"   ✅ User is active: {user.is_active}")
    print(f"   ✅ Uncommitted events: {len(user.get_uncommitted_events())}")
    
    # 3. Save to event store
    print("\\n3. Saving user to event store...")
    await store.save(user)
    user.mark_events_as_committed()
    print("   ✅ User saved successfully")
    print(f"   ✅ Current version: {user.version}")
    
    # 4. Update the user
    print("\\n4. Updating user email...")
    user.change_email("alice.johnson@company.com")
    await store.save(user)
    user.mark_events_as_committed()
    print(f"   ✅ Email updated to: {user.email}")
    
    # 5. Load user from event store
    print("\\n5. Loading user from event store...")
    loaded_user = await store.load(User, "user-123")
    
    if loaded_user:
        print(f"   ✅ Loaded user: {loaded_user.name}")
        print(f"   ✅ Email: {loaded_user.email}")
        print(f"   ✅ Active: {loaded_user.is_active}")
        print(f"   ✅ Version: {loaded_user.version}")
    
    # 6. Deactivate user
    print("\\n6. Deactivating user...")
    loaded_user.deactivate("Account closed by user request")
    await store.save(loaded_user)
    loaded_user.mark_events_as_committed()
    print(f"   ✅ User deactivated: {not loaded_user.is_active}")
    
    # 7. View event history
    print("\\n7. Event history:")
    events = await store.load_events("user-123")
    for i, event in enumerate(events, 1):
        print(f"   {i}. {event.event_type} (v{event.aggregate_version})")
        if hasattr(event, 'name'):
            print(f"      → Registered: {event.name}")
        elif hasattr(event, 'new_email'):
            print(f"      → Email: {event.old_email} → {event.new_email}")
        elif hasattr(event, 'reason'):
            print(f"      → Reason: {event.reason}")
    
    print("\\n✅ Quick start completed successfully!")

if __name__ == "__main__":
    asyncio.run(main())
```

### Step 4: Run Your First Example

```bash
# Run the quick start example
uv run python main.py
```

**Expected Output:**

```
=== Eventuali Quick Start ===

1. Creating event store...
   ✅ Event store created with in-memory SQLite

2. Creating a new user...
   ✅ User created: Alice Johnson (alice@example.com)
   ✅ User is active: True
   ✅ Uncommitted events: 1

3. Saving user to event store...
   ✅ User saved successfully
   ✅ Current version: 1

4. Updating user email...
   ✅ Email updated to: alice.johnson@company.com

5. Loading user from event store...
   ✅ Loaded user: Alice Johnson
   ✅ Email: alice.johnson@company.com
   ✅ Active: True
   ✅ Version: 2

6. Deactivating user...
   ✅ User deactivated: True

7. Event history:
   1. UserRegistered (v1)
      → Registered: Alice Johnson
   2. UserEmailChanged (v2)
      → Email: alice@example.com → alice.johnson@company.com
   3. UserDeactivated (v3)
      → Reason: Account closed by user request

✅ Quick start completed successfully!
```

## 🔧 Understanding the Flow

### 1. Event Creation

```python
# Command creates an event
user.register(name="Alice", email="alice@example.com")

# Event is applied to aggregate
event = UserRegistered(name="Alice", email="alice@example.com")
user.apply(event)  # Calls apply_user_registered()
```

### 2. State Reconstruction

```python
# When loading from store:
# 1. Load all events for aggregate
events = await store.load_events("user-123")

# 2. Replay events to rebuild state
user = User(id="user-123")
for event in events:
    user.apply(event)  # State is rebuilt
```

### 3. Event Store Persistence

```python
# Events are persisted as immutable records
await store.save(user)  # Saves uncommitted events

# Events can be queried and analyzed
all_events = await store.load_events("user-123")
user_events = await store.load_events_by_type("User")
```

## 🚀 Performance Characteristics

Your quick start example demonstrates Eventuali's performance advantages:

| Operation | Performance | Comparison |
|-----------|-------------|------------|
| **Event Creation** | 79,000+ events/sec | 10-60x faster than pure Python |
| **Event Persistence** | 25,000+ events/sec | Rust-optimized database operations |
| **Aggregate Loading** | 40,000+ aggregates/sec | 18.3x faster reconstruction |
| **Memory Usage** | 8-20x more efficient | Minimal Python object overhead |

## 🔄 Next Steps

Now that you understand the basics, explore these areas:

### 1. Database Configuration

```python
# PostgreSQL for production
store = await EventStore.create(
    "postgresql://user:password@localhost:5432/events"
)

# SQLite with file persistence
store = await EventStore.create("sqlite://events.db")
```

### 2. Advanced Event Patterns

```python
# Event with correlation
import uuid
correlation_id = uuid.uuid4()

event = UserRegistered(
    name="Bob",
    email="bob@example.com",
    correlation_id=correlation_id,
    user_id="admin-456"  # Who triggered this event
)
```

### 3. Error Handling

```python
from eventuali.exceptions import OptimisticConcurrencyError

try:
    await store.save(user)
except OptimisticConcurrencyError:
    # Handle concurrent modification
    fresh_user = await store.load(User, user.id)
    # Merge changes or retry
```

### 4. Custom Events

```python
class OrderPlaced(DomainEvent):
    customer_id: str
    items: List[dict]
    total_amount: float
    shipping_address: dict

# Register custom events
EventStore.register_event_class("OrderPlaced", OrderPlaced)
```

## 📚 Learning Path

1. **✅ You're Here**: Quick Start - Basic concepts
2. **Next**: [Performance Testing](../../examples/04_performance_testing.py) - Benchmarking
3. **Then**: [Error Handling](../../examples/03_error_handling.py) - Production patterns
4. **Advanced**: [CQRS Patterns](../../examples/09_cqrs_patterns.py) - Architecture

## 🔗 Related Resources

- **[Working Examples](../../examples/README.md)** - 46+ proven examples
- **[API Reference](../api/README.md)** - Complete API documentation
- **[Architecture Guide](../architecture/README.md)** - System design
- **[Migration Guide](migration-guide.md)** - Moving from other libraries

---

**Congratulations!** 🎉 You've built your first event-sourced application with Eventuali. The patterns you've learned here scale to enterprise applications with millions of events.

**Next**: Try the [FastAPI Integration Guide](fastapi-integration.md) to build a REST API, or explore the [Performance Guide](../performance/README.md) for optimization techniques.