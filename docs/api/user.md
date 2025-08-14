# User Aggregate API Reference

The `User` class is a built-in aggregate that demonstrates event sourcing patterns and provides a ready-to-use user management implementation.

## Class Definition

```python
class User(Aggregate):
    """Example user aggregate with complete lifecycle management."""
```

## State Fields

```python
name: str = ""              # User's full name
email: str = ""             # User's email address  
is_active: bool = True      # Whether the user account is active
```

## Built-in Events

The User aggregate works with these predefined events:

- **`UserRegistered`** - When a user first registers
- **`UserEmailChanged`** - When user changes email address
- **`UserDeactivated`** - When user account is deactivated

## Business Methods

### `change_email(new_email: str)`

Changes the user's email address with validation.

**Parameters:**
- `new_email: str` - The new email address

**Validation:**
- Email must not be empty
- Email must contain '@' symbol
- Cannot change email for inactive users
- No change if email is the same

**Examples:**

```python
user = User(id="user-123")
# Assume user is already registered

# Change email
user.change_email("john.doe@company.com")

# Generates UserEmailChanged event
events = user.get_uncommitted_events()
assert len(events) == 1
assert events[0].event_type == "UserEmailChanged"
assert events[0].old_email == "john@example.com"
assert events[0].new_email == "john.doe@company.com"
```

**Raises:**
- `ValueError` - If email is invalid or user is inactive

### `deactivate(reason: Optional[str] = None)`

Deactivates the user account.

**Parameters:**
- `reason: Optional[str]` - Optional reason for deactivation

**Examples:**

```python
# Deactivate user
user.deactivate("Account closed by user request")

# Check state
assert not user.is_active

# Generates UserDeactivated event
events = user.get_uncommitted_events()
assert events[-1].event_type == "UserDeactivated"
assert events[-1].reason == "Account closed by user request"
```

**Behavior:**
- No-op if user is already inactive
- Sets `is_active` to `False`
- Generates `UserDeactivated` event

---

## Event Handlers

### `apply_user_registered(event: UserRegistered)`

Handles user registration event.

**State Changes:**
- Sets `name` from event
- Sets `email` from event  
- Sets `is_active` to `True`

### `apply_user_email_changed(event: UserEmailChanged)`

Handles email change event.

**State Changes:**
- Updates `email` to `new_email` from event

### `apply_user_deactivated(event: UserDeactivated)`

Handles user deactivation event.

**State Changes:**
- Sets `is_active` to `False`

---

## Usage Patterns

### Complete User Lifecycle

```python
import asyncio
from eventuali import EventStore
from eventuali.aggregate import User
from eventuali.event import UserRegistered

async def user_lifecycle_example():
    # Setup
    store = await EventStore.create("sqlite://:memory:")
    
    # 1. Create new user
    user = User(id="user-123")
    
    # 2. Register user
    registration_event = UserRegistered(
        name="John Doe", 
        email="john@example.com"
    )
    user.apply(registration_event)
    
    # 3. Save initial state
    await store.save(user)
    user.mark_events_as_committed()
    
    print(f"User registered: {user.name} ({user.email})")
    print(f"Active: {user.is_active}, Version: {user.version}")
    
    # 4. Change email
    user.change_email("john.doe@company.com")
    await store.save(user)
    user.mark_events_as_committed()
    
    print(f"Email updated to: {user.email}")
    
    # 5. Deactivate account
    user.deactivate("User requested account closure")
    await store.save(user)
    user.mark_events_as_committed()
    
    print(f"User deactivated: {not user.is_active}")
    
    # 6. Load user from store
    loaded_user = await store.load(User, "user-123")
    print(f"Loaded user: {loaded_user.name}, Active: {loaded_user.is_active}")
    
    # 7. View event history
    events = await store.load_events("user-123")
    print(f"Total events: {len(events)}")
    for i, event in enumerate(events, 1):
        print(f"  {i}. {event.event_type} (v{event.aggregate_version})")

asyncio.run(user_lifecycle_example())
```

**Expected Output:**
```
User registered: John Doe (john@example.com)
Active: True, Version: 1
Email updated to: john.doe@company.com
User deactivated: True
Loaded user: John Doe, Active: False
Total events: 3
  1. UserRegistered (v1)
  2. UserEmailChanged (v2)
  3. UserDeactivated (v3)
```

### User Registration Pattern

```python
async def register_user(store: EventStore, name: str, email: str) -> User:
    """Register a new user with validation."""
    
    # Validate inputs
    if not name or not name.strip():
        raise ValueError("Name is required")
    
    if not email or '@' not in email:
        raise ValueError("Valid email address is required")
    
    # Create user
    user = User(id=str(uuid4()))
    
    # Apply registration event
    event = UserRegistered(
        name=name.strip(),
        email=email.lower().strip()
    )
    user.apply(event)
    
    # Save to store
    await store.save(user)
    user.mark_events_as_committed()
    
    return user

# Usage
user = await register_user(store, "Alice Johnson", "alice@example.com")
```

### User Management Service

```python
class UserService:
    """Service for user management operations."""
    
    def __init__(self, store: EventStore):
        self.store = store
    
    async def get_user(self, user_id: str) -> Optional[User]:
        """Get user by ID."""
        return await self.store.load(User, user_id)
    
    async def update_email(self, user_id: str, new_email: str) -> bool:
        """Update user email with validation."""
        user = await self.store.load(User, user_id)
        if not user:
            raise ValueError(f"User {user_id} not found")
        
        if not user.is_active:
            raise ValueError("Cannot update email for inactive user")
        
        # Update email
        user.change_email(new_email)
        
        # Save changes
        await self.store.save(user)
        user.mark_events_as_committed()
        
        return True
    
    async def deactivate_user(self, user_id: str, reason: str = "") -> bool:
        """Deactivate user account."""
        user = await self.store.load(User, user_id)
        if not user:
            raise ValueError(f"User {user_id} not found")
        
        if not user.is_active:
            return False  # Already deactivated
        
        # Deactivate
        user.deactivate(reason)
        
        # Save changes
        await self.store.save(user)
        user.mark_events_as_committed()
        
        return True
    
    async def get_user_history(self, user_id: str) -> List[Event]:
        """Get complete event history for user."""
        return await self.store.load_events(user_id)

# Usage
service = UserService(store)
await service.update_email("user-123", "newemail@example.com")
await service.deactivate_user("user-123", "Policy violation")
```

---

## Integration Examples

### With FastAPI

```python
from fastapi import FastAPI, HTTPException, Depends
from pydantic import BaseModel

app = FastAPI()

class UserCreateRequest(BaseModel):
    name: str
    email: str

class UserUpdateEmailRequest(BaseModel):
    new_email: str

async def get_event_store() -> EventStore:
    return await EventStore.create("postgresql://...")

@app.post("/users", response_model=dict)
async def create_user(
    request: UserCreateRequest,
    store: EventStore = Depends(get_event_store)
):
    """Create a new user."""
    try:
        user = User(id=str(uuid4()))
        event = UserRegistered(name=request.name, email=request.email)
        user.apply(event)
        
        await store.save(user)
        user.mark_events_as_committed()
        
        return {
            "user_id": user.id,
            "name": user.name,
            "email": user.email,
            "is_active": user.is_active
        }
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/users/{user_id}")
async def get_user(
    user_id: str,
    store: EventStore = Depends(get_event_store)
):
    """Get user by ID."""
    user = await store.load(User, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    
    return {
        "user_id": user.id,
        "name": user.name,
        "email": user.email,
        "is_active": user.is_active,
        "version": user.version
    }

@app.put("/users/{user_id}/email")
async def update_user_email(
    user_id: str,
    request: UserUpdateEmailRequest,
    store: EventStore = Depends(get_event_store)
):
    """Update user email."""
    user = await store.load(User, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    
    try:
        user.change_email(request.new_email)
        await store.save(user)
        user.mark_events_as_committed()
        
        return {"message": "Email updated successfully"}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

@app.delete("/users/{user_id}")
async def deactivate_user(
    user_id: str,
    store: EventStore = Depends(get_event_store)
):
    """Deactivate user account."""
    user = await store.load(User, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")
    
    user.deactivate("Deactivated via API")
    await store.save(user)
    user.mark_events_as_committed()
    
    return {"message": "User deactivated successfully"}
```

### Event Sourcing Projection

```python
from eventuali import Projection

class UserSummaryProjection(Projection):
    """Projection for user summary statistics."""
    
    def __init__(self):
        self.total_users = 0
        self.active_users = 0
        self.total_registrations = 0
        self.total_deactivations = 0
    
    async def handle_user_registered(self, event: UserRegistered):
        """Handle user registration."""
        self.total_users += 1
        self.active_users += 1
        self.total_registrations += 1
    
    async def handle_user_deactivated(self, event: UserDeactivated):
        """Handle user deactivation."""
        self.active_users -= 1
        self.total_deactivations += 1
    
    def get_summary(self) -> dict:
        """Get user statistics summary."""
        return {
            "total_users": self.total_users,
            "active_users": self.active_users,
            "inactive_users": self.total_users - self.active_users,
            "total_registrations": self.total_registrations,
            "total_deactivations": self.total_deactivations,
            "activation_rate": (
                self.active_users / self.total_users 
                if self.total_users > 0 else 0
            )
        }

# Usage with EventStreamer
from eventuali import EventStreamer

async def setup_user_analytics():
    store = await EventStore.create("postgresql://...")
    streamer = EventStreamer(store)
    projection = UserSummaryProjection()
    
    # Subscribe to user events
    await streamer.subscribe(projection)
    await streamer.start()
    
    # Get real-time statistics
    summary = projection.get_summary()
    print(f"Active users: {summary['active_users']}")
    print(f"Activation rate: {summary['activation_rate']:.2%}")
```

---

## Testing the User Aggregate

```python
import pytest
from eventuali.aggregate import User
from eventuali.event import UserRegistered, UserEmailChanged, UserDeactivated

def test_user_registration():
    """Test user registration flow."""
    user = User(id="test-user")
    
    # Apply registration event
    event = UserRegistered(name="Test User", email="test@example.com")
    user.apply(event)
    
    # Verify state
    assert user.name == "Test User"
    assert user.email == "test@example.com"
    assert user.is_active is True
    assert user.version == 1
    
    # Verify event
    events = user.get_uncommitted_events()
    assert len(events) == 1
    assert events[0].event_type == "UserRegistered"

def test_email_change():
    """Test email change functionality."""
    user = User(id="test-user")
    
    # Register first
    registration = UserRegistered(name="Test", email="old@example.com")
    user.apply(registration)
    user.mark_events_as_committed()
    
    # Change email
    user.change_email("new@example.com")
    
    # Verify state
    assert user.email == "new@example.com"
    assert user.version == 2
    
    # Verify event
    events = user.get_uncommitted_events()
    assert len(events) == 1
    assert events[0].event_type == "UserEmailChanged"
    assert events[0].old_email == "old@example.com"
    assert events[0].new_email == "new@example.com"

def test_email_validation():
    """Test email validation."""
    user = User(id="test-user")
    
    # Register first
    registration = UserRegistered(name="Test", email="test@example.com")
    user.apply(registration)
    
    # Test invalid emails
    with pytest.raises(ValueError, match="Valid email is required"):
        user.change_email("")
    
    with pytest.raises(ValueError, match="Valid email is required"):
        user.change_email("invalid-email")
    
    # No change for same email
    user.change_email("test@example.com")
    assert len(user.get_uncommitted_events()) == 0

def test_deactivation():
    """Test user deactivation."""
    user = User(id="test-user")
    
    # Register and mark committed
    registration = UserRegistered(name="Test", email="test@example.com")
    user.apply(registration)
    user.mark_events_as_committed()
    
    # Deactivate
    user.deactivate("Test deactivation")
    
    # Verify state
    assert user.is_active is False
    assert user.version == 2
    
    # Verify event
    events = user.get_uncommitted_events()
    assert len(events) == 1
    assert events[0].event_type == "UserDeactivated"
    assert events[0].reason == "Test deactivation"

def test_cannot_change_email_when_inactive():
    """Test that inactive users cannot change email."""
    user = User(id="test-user")
    
    # Register and deactivate
    registration = UserRegistered(name="Test", email="test@example.com")
    user.apply(registration)
    user.deactivate()
    
    # Try to change email
    with pytest.raises(ValueError, match="Cannot change email for inactive user"):
        user.change_email("new@example.com")

def test_event_replay():
    """Test aggregate reconstruction from events."""
    events = [
        UserRegistered(
            aggregate_id="test-user",
            name="John Doe",
            email="john@example.com"
        ),
        UserEmailChanged(
            aggregate_id="test-user",
            old_email="john@example.com",
            new_email="john.doe@company.com"
        ),
        UserDeactivated(
            aggregate_id="test-user",
            reason="Account closure"
        )
    ]
    
    # Reconstruct user from events
    user = User.from_events(events)
    
    # Verify final state
    assert user.id == "test-user"
    assert user.name == "John Doe"
    assert user.email == "john.doe@company.com"
    assert user.is_active is False
    assert user.version == 3
    assert not user.is_new()
```

---

## Related Documentation

- **[Aggregate API](aggregate.md)** - Base aggregate pattern
- **[Event API](event.md)** - UserRegistered, UserEmailChanged, UserDeactivated events
- **[EventStore API](event-store.md)** - Persistence and loading
- **[Examples](../../examples/01_basic_event_store_simple.py)** - Working user examples

---

**Next**: See the User aggregate in action with the [basic example](../../examples/01_basic_event_store_simple.py) or explore the [EventStore API](event-store.md) for persistence patterns.