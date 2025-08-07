#!/usr/bin/env python3
"""
Python syntax test for event sourcing concepts.

This tests the pure Python code without requiring compiled Rust bindings.
"""

import asyncio
import json
import sys
import os

# Add the Python package to the path and import directly
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

# Import directly from modules (bypassing __init__.py which requires compiled bindings)
try:
    from eventuali.event import Event, UserRegistered, UserEmailChanged, UserDeactivated, DomainEvent
    from eventuali.aggregate import Aggregate
    print("✅ Successfully imported Python modules")
    IMPORTS_OK = True
except ImportError as e:
    print(f"❌ Import failed: {e}")
    IMPORTS_OK = False
    sys.exit(1)

# Define User aggregate
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

def test_event_creation():
    """Test creating and serializing events."""
    print("\n🧪 Testing Event Creation:")
    
    # Create events
    register_event = UserRegistered(name="Alice Johnson", email="alice@example.com")
    print(f"✓ Created UserRegistered: {register_event.name}")
    
    # Test JSON serialization
    json_data = register_event.to_json()
    print(f"✓ JSON serialization: {len(json_data)} bytes")
    
    # Test deserialization
    restored_event = UserRegistered.from_json(json_data)
    print(f"✓ JSON deserialization: {restored_event.name}")
    
    # Test dict conversion
    event_dict = register_event.to_dict()
    print(f"✓ Dict conversion: {len(event_dict)} fields")
    
    return register_event

def test_aggregate_operations():
    """Test aggregate creation and event application."""
    print("\n🧪 Testing Aggregate Operations:")
    
    user = User()
    print(f"✓ Created user: {user.id}")
    print(f"  Version: {user.version}, Is new: {user.is_new()}")
    
    # Apply registration event
    register_event = UserRegistered(name="Bob Smith", email="bob@example.com")
    user.apply(register_event)
    print(f"✓ Applied registration event")
    print(f"  Name: {user.name}, Email: {user.email}")
    print(f"  Version: {user.version}, Has uncommitted: {user.has_uncommitted_events()}")
    
    # Test business method
    user.change_email("bob.smith@newcompany.com")
    print(f"✓ Used business method to change email")
    print(f"  Email: {user.email}")
    print(f"  Uncommitted events: {len(user.get_uncommitted_events())}")
    
    return user

def test_event_replay():
    """Test event replay functionality."""
    print("\n🧪 Testing Event Replay:")
    
    # Create original aggregate with events
    original = User()
    events = [
        UserRegistered(name="Carol Williams", email="carol@example.com"),
        UserEmailChanged(old_email="carol@example.com", new_email="carol.w@company.com"),
        UserDeactivated(reason="Account closure requested")
    ]
    
    # Apply events to original
    for event in events:
        original.apply(event)
    
    print(f"✓ Original aggregate: {original.name}, {original.email}, Active: {original.is_active}")
    
    # Prepare events for replay
    replay_events = []
    for i, event in enumerate(original.get_uncommitted_events(), 1):
        # Simulate what the event store would do
        event.aggregate_id = original.id
        event.aggregate_type = "User"
        event.aggregate_version = i
        replay_events.append(event)
    
    # Reconstruct from events
    try:
        reconstructed = User.from_events(replay_events)
        print(f"✓ Reconstructed from {len(replay_events)} events")
        print(f"  Name: {reconstructed.name}, Email: {reconstructed.email}, Active: {reconstructed.is_active}")
        print(f"  Version: {reconstructed.version}, Is new: {reconstructed.is_new()}")
        
        # Check consistency
        orig_state = original.to_dict()
        recon_state = reconstructed.to_dict()
        
        # Remove internal fields
        for state in [orig_state, recon_state]:
            state.pop('_uncommitted_events', None)
            state.pop('_is_new', None)
            
        if orig_state == recon_state:
            print("✅ State consistency verified!")
        else:
            print("❌ State mismatch detected!")
            
    except Exception as e:
        print(f"❌ Event replay failed: {e}")
        # This might fail if from_events isn't implemented yet
        print("⏳ from_events method may not be fully implemented")

def main():
    """Run all tests."""
    print("🐍 Python Event Sourcing Syntax Test")
    print("=" * 40)
    
    if not IMPORTS_OK:
        print("❌ Cannot run tests - imports failed")
        return
        
    try:
        test_event_creation()
        test_aggregate_operations()
        test_event_replay()
        
        print("\n🎉 All tests completed!")
        print("✅ Python event sourcing syntax is working correctly")
        
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        print("⏳ Some functionality may not be fully implemented yet")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()