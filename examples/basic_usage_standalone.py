#!/usr/bin/env python3
"""
Basic usage example demonstrating Eventuali event sourcing concepts.

This standalone version runs without compiled Rust bindings for testing purposes.

This example shows how to:
1. Define domain events with Pydantic
2. Create aggregates that apply events
3. Use event replay to reconstruct state
4. Work with event stores (when fully implemented)
"""

import asyncio
import json
import sys
import os
from uuid import uuid4

# Add the Python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

try:
    # Try to import from the pure Python parts only
    from eventuali.event import Event, UserRegistered, UserEmailChanged, UserDeactivated
    from eventuali.aggregate import Aggregate
    IMPORTS_AVAILABLE = True
except ImportError as e:
    print(f"⚠️  Import error (expected for standalone testing): {e}")
    IMPORTS_AVAILABLE = False

# Define a simple User aggregate for testing
if IMPORTS_AVAILABLE:
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


def main():
    """Demonstrate basic event sourcing concepts."""
    if not IMPORTS_AVAILABLE:
        print("❌ Python packages not available for standalone testing")
        print("⏳ This would work once Python bindings are properly set up")
        return
        
    print("🚀 Eventuali Event Sourcing Demo (Standalone)")
    print("=" * 50)
    
    # 1. Create domain events
    print("\n📝 Creating Domain Events:")
    
    user_id = str(uuid4())
    register_event = UserRegistered(name="Alice Johnson", email="alice@example.com")
    print(f"✓ UserRegistered: {register_event.name} ({register_event.email})")
    
    email_change_event = UserEmailChanged(
        old_email="alice@example.com", 
        new_email="alice.johnson@company.com"
    )
    print(f"✓ UserEmailChanged: {email_change_event.old_email} → {email_change_event.new_email}")
    
    deactivate_event = UserDeactivated(reason="User requested account closure")
    print(f"✓ UserDeactivated: {deactivate_event.reason}")
    
    # 2. Create and modify an aggregate
    print("\n👤 Working with User Aggregate:")
    
    user = User(id=user_id)
    print(f"✓ Created user aggregate: {user.id}")
    print(f"  Version: {user.version}, Is new: {user.is_new()}")
    
    # Apply the registration event
    user.apply(register_event)
    print(f"✓ Applied registration event")
    print(f"  Name: {user.name}, Email: {user.email}")
    print(f"  Version: {user.version}, Uncommitted events: {len(user.get_uncommitted_events())}")
    
    # Use business method (which generates and applies events)
    user.change_email("alice.johnson@newcompany.com")
    print(f"✓ Changed email via business method")
    print(f"  Email: {user.email}")
    print(f"  Version: {user.version}, Uncommitted events: {len(user.get_uncommitted_events())}")
    
    # Apply deactivation
    user.apply(deactivate_event)
    print(f"✓ Applied deactivation event")
    print(f"  Active: {user.is_active}")
    print(f"  Version: {user.version}, Uncommitted events: {len(user.get_uncommitted_events())}")
    
    # 3. Show event serialization
    print("\n💾 Event Serialization:")
    
    events = user.get_uncommitted_events()
    for i, event in enumerate(events, 1):
        event_json = event.to_json()
        print(f"✓ Event {i} ({event.get_event_type()}): {len(event_json)} bytes")
        # Pretty print first event as example
        if i == 1:
            event_dict = json.loads(event_json)
            print(f"   Sample JSON: {json.dumps(event_dict, indent=2)[:200]}...")
    
    # 4. Demonstrate event replay
    print("\n🔄 Event Replay Demonstration:")
    
    # Prepare events for replay (simulate loading from event store)
    replay_events = []
    for i, event in enumerate(user.get_uncommitted_events(), 1):
        event.aggregate_id = user_id
        event.aggregate_type = "User"
        event.aggregate_version = i
        replay_events.append(event)
    
    # Create new aggregate from events
    reconstructed_user = User.from_events(replay_events)
    print(f"✓ Reconstructed user from {len(replay_events)} events")
    print(f"  ID: {reconstructed_user.id}")
    print(f"  Name: {reconstructed_user.name}")
    print(f"  Email: {reconstructed_user.email}")
    print(f"  Active: {reconstructed_user.is_active}")
    print(f"  Version: {reconstructed_user.version}")
    print(f"  Is new: {reconstructed_user.is_new()}")
    print(f"  Uncommitted events: {len(reconstructed_user.get_uncommitted_events())}")
    
    # 5. Demonstrate state consistency
    print("\n✅ State Consistency Check:")
    
    original_state = user.to_dict()
    reconstructed_state = reconstructed_user.to_dict()
    
    # Remove internal fields for comparison
    for state in [original_state, reconstructed_state]:
        state.pop('_uncommitted_events', None)
        state.pop('_is_new', None)
    
    if original_state == reconstructed_state:
        print("✓ Original and reconstructed states match perfectly!")
    else:
        print("❌ State mismatch detected!")
        print(f"Original: {original_state}")
        print(f"Reconstructed: {reconstructed_state}")
    
    print("\n🎉 Demo completed successfully!")
    print("\nNext steps:")
    print("- Implement event store save/load operations")
    print("- Add projection/read model capabilities")  
    print("- Set up event streaming and subscriptions")


async def async_demo():
    """Demonstrate async event store operations (when implemented)."""
    if not IMPORTS_AVAILABLE:
        return
        
    print("\n🔮 Future: Async Event Store Operations")
    print("=" * 40)
    
    try:
        from eventuali import EventStore
        
        # This will work once the Rust backend is fully connected
        print("✓ Importing EventStore...")
        store = await EventStore.create("sqlite://demo.db")
        print("✓ Created SQLite event store")
        print("⏳ Save/load operations coming soon...")
        
    except Exception as e:
        print(f"⏳ Async operations not fully implemented yet: {e}")


if __name__ == "__main__":
    # Run the main demo
    main()
    
    # Run async demo
    if IMPORTS_AVAILABLE:
        print("\n" + "=" * 60)
        asyncio.run(async_demo())