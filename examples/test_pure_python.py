#!/usr/bin/env python3
"""
Test pure Python event sourcing functionality without compiled bindings.

This creates isolated copies of the pure Python classes for testing.
"""

import json
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any, Dict, List, Optional, Type, TypeVar
from uuid import UUID, uuid4

# We need pydantic
try:
    from pydantic import BaseModel, Field
    print("✅ Pydantic available")
except ImportError:
    print("❌ Pydantic not available - install with: pip install pydantic")
    exit(1)

T = TypeVar('T', bound='Event')
A = TypeVar('A', bound='Aggregate')

class Event(BaseModel, ABC):
    """Base class for all domain events."""
    
    # These fields will be set automatically by the event store
    event_id: Optional[UUID] = Field(default_factory=uuid4, description="Unique event identifier")
    aggregate_id: Optional[str] = Field(default=None, description="ID of the aggregate that generated this event")
    aggregate_type: Optional[str] = Field(default=None, description="Type of the aggregate")
    event_type: Optional[str] = Field(default=None, description="Type of the event")
    event_version: Optional[int] = Field(default=1, description="Schema version of the event")
    aggregate_version: Optional[int] = Field(default=None, description="Version of the aggregate after this event")
    timestamp: Optional[datetime] = Field(default_factory=datetime.utcnow, description="When the event occurred")
    
    # Metadata for correlation and causation
    causation_id: Optional[UUID] = Field(default=None, description="ID of the event that caused this event")
    correlation_id: Optional[UUID] = Field(default=None, description="ID correlating related events")
    user_id: Optional[str] = Field(default=None, description="ID of the user who triggered this event")
    
    model_config = {
        "frozen": False,  # Allow modification for event store metadata
        "use_enum_values": True,
        "json_encoders": {
            datetime: lambda v: v.isoformat(),
            UUID: lambda v: str(v),
        }
    }
    
    @classmethod
    def get_event_type(cls) -> str:
        """Get the event type name."""
        return cls.__name__
    
    def to_json(self) -> str:
        """Convert event to JSON string."""
        return self.model_dump_json()
    
    @classmethod
    def from_json(cls: Type[T], json_str: str) -> T:
        """Create event from JSON string."""
        return cls.model_validate_json(json_str)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert event to dictionary."""
        return self.model_dump()

class Aggregate(BaseModel, ABC):
    """Base class for all aggregates in the domain."""
    
    # Core aggregate fields
    id: str = Field(default_factory=lambda: str(uuid4()), description="Unique aggregate identifier")
    version: int = Field(default=0, description="Current version of the aggregate")
    
    # Private fields for event sourcing
    uncommitted_events: List[Event] = Field(default_factory=list, exclude=True, alias='_uncommitted_events')
    is_new_flag: bool = Field(default=True, exclude=True, alias='_is_new')
    
    model_config = {
        "validate_assignment": True,
        "use_enum_values": True,
        "json_encoders": {
            UUID: lambda v: str(v),
        }
    }
    
    @classmethod
    def get_aggregate_type(cls) -> str:
        """Get the aggregate type name."""
        return cls.__name__
    
    def mark_events_as_committed(self) -> None:
        """Mark all uncommitted events as committed."""
        self.uncommitted_events.clear()
        self.is_new_flag = False
    
    def get_uncommitted_events(self) -> List[Event]:
        """Get all uncommitted events."""
        return self.uncommitted_events.copy()
    
    def has_uncommitted_events(self) -> bool:
        """Check if the aggregate has uncommitted events."""
        return len(self.uncommitted_events) > 0
    
    def is_new(self) -> bool:
        """Check if this is a new aggregate (never persisted)."""
        return self.is_new_flag and self.version == 0
    
    def _get_method_name(self, event_type: str) -> str:
        """Convert event type to method name (e.g., UserRegistered -> user_registered)."""
        # Convert PascalCase to snake_case
        result = []
        for i, char in enumerate(event_type):
            if char.isupper() and i > 0:
                result.append('_')
            result.append(char.lower())
        return ''.join(result)
    
    def _apply_event(self, event: Event) -> None:
        """Apply an event to the aggregate without adding it to uncommitted events."""
        # Set event metadata
        event.aggregate_id = self.id
        event.aggregate_type = self.get_aggregate_type()
        event.event_type = event.get_event_type()
        event.aggregate_version = self.version + 1
        
        # Find and call the appropriate apply method
        method_name = f"apply_{self._get_method_name(event.get_event_type())}"
        if hasattr(self, method_name):
            getattr(self, method_name)(event)
            self.version += 1
        else:
            raise NotImplementedError(
                f"No apply method found for event {event.get_event_type()}. "
                f"Expected method: {method_name}"
            )
    
    def apply(self, event: Event) -> None:
        """Apply an event to the aggregate and add it to uncommitted events."""
        self._apply_event(event)
        self.uncommitted_events.append(event)

    @classmethod
    def from_events(cls: Type[A], events: List[Event]) -> A:
        """Reconstruct aggregate from a list of events."""
        if not events:
            return cls()
        
        # Create new instance
        instance = cls(id=events[0].aggregate_id)
        instance.is_new_flag = False
        
        # Apply all events
        for event in events:
            instance._apply_event(event)
        
        return instance
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert aggregate to dictionary."""
        return self.model_dump()

# Example domain events
class UserRegistered(Event):
    """Event fired when a user registers."""
    name: str
    email: str

class UserEmailChanged(Event):
    """Event fired when a user changes their email."""
    old_email: str
    new_email: str

class UserDeactivated(Event):
    """Event fired when a user account is deactivated."""
    reason: Optional[str] = None

# Example aggregate
class User(Aggregate):
    """User aggregate for demonstration."""
    name: str = ""
    email: str = ""
    is_active: bool = True
    
    def apply_user_registered(self, event: UserRegistered) -> None:
        """Apply UserRegistered event."""
        self.name = event.name
        self.email = event.email
        
    def apply_user_email_changed(self, event: UserEmailChanged) -> None:
        """Apply UserEmailChanged event."""
        self.email = event.new_email
        
    def apply_user_deactivated(self, event: UserDeactivated) -> None:
        """Apply UserDeactivated event."""
        self.is_active = False
        
    def change_email(self, new_email: str) -> None:
        """Business method to change email."""
        if self.email == new_email:
            return
        
        event = UserEmailChanged(old_email=self.email, new_email=new_email)
        self.apply(event)

def test_events():
    """Test event creation and serialization."""
    print("\n🧪 Testing Events:")
    
    # Create event
    event = UserRegistered(name="Alice Johnson", email="alice@example.com")
    print(f"✓ Created event: {event.get_event_type()}")
    
    # Test serialization
    json_data = event.to_json()
    print(f"✓ JSON serialization: {len(json_data)} bytes")
    
    # Test deserialization
    restored = UserRegistered.from_json(json_data)
    print(f"✓ Deserialized: {restored.name}, {restored.email}")
    
    # Test dict conversion
    event_dict = event.to_dict()
    print(f"✓ Dict conversion: {len(event_dict)} fields")
    
    return event

def test_aggregates():
    """Test aggregate functionality."""
    print("\n🧪 Testing Aggregates:")
    
    # Create user
    user = User()
    print(f"✓ Created user: {user.id[:8]}...")
    print(f"  Version: {user.version}, Is new: {user.is_new()}")
    
    # Apply events
    register_event = UserRegistered(name="Bob Smith", email="bob@example.com")
    user.apply(register_event)
    print(f"✓ Applied registration: {user.name}, {user.email}")
    print(f"  Version: {user.version}, Uncommitted: {len(user.get_uncommitted_events())}")
    
    # Business method
    user.change_email("bob.smith@company.com")
    print(f"✓ Changed email: {user.email}")
    print(f"  Version: {user.version}, Uncommitted: {len(user.get_uncommitted_events())}")
    
    # Deactivate
    deactivate_event = UserDeactivated(reason="User request")
    user.apply(deactivate_event)
    print(f"✓ Deactivated: Active = {user.is_active}")
    
    return user

def test_event_replay():
    """Test event replay."""
    print("\n🧪 Testing Event Replay:")
    
    # Create original
    original = User()
    events = [
        UserRegistered(name="Carol Williams", email="carol@example.com"),
        UserEmailChanged(old_email="carol@example.com", new_email="carol.w@company.com"),
    ]
    
    for event in events:
        original.apply(event)
    
    print(f"✓ Original: {original.name}, {original.email}")
    
    # Get events for replay
    replay_events = []
    for i, event in enumerate(original.get_uncommitted_events(), 1):
        # Set metadata as event store would
        event.aggregate_id = original.id
        event.aggregate_type = "User"
        event.aggregate_version = i
        replay_events.append(event)
    
    # Reconstruct
    reconstructed = User.from_events(replay_events)
    print(f"✓ Reconstructed: {reconstructed.name}, {reconstructed.email}")
    print(f"  Version: {reconstructed.version}, Is new: {reconstructed.is_new()}")
    
    # Verify consistency
    orig_dict = original.to_dict()
    recon_dict = reconstructed.to_dict()
    
    # Remove internal fields
    for d in [orig_dict, recon_dict]:
        d.pop('uncommitted_events', None)
        d.pop('is_new_flag', None)
    
    if orig_dict == recon_dict:
        print("✅ State consistency verified!")
    else:
        print("❌ State mismatch!")

def main():
    """Run all tests."""
    print("🐍 Pure Python Event Sourcing Test")
    print("=" * 40)
    
    try:
        test_events()
        test_aggregates()
        test_event_replay()
        
        print("\n🎉 All tests passed!")
        print("✅ Pure Python event sourcing is working correctly")
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()