"""
Basic tests for Eventuali functionality.
"""

import pytest
import asyncio
from eventuali import EventStore
from eventuali.event import UserRegistered, UserEmailChanged
from eventuali.aggregate import User


class TestEventSourcing:
    """Test basic event sourcing functionality."""
    
    def test_event_creation(self):
        """Test event creation and serialization."""
        event = UserRegistered(name="John Doe", email="john@example.com")
        
        assert event.name == "John Doe"
        assert event.email == "john@example.com"
        assert event.get_event_type() == "UserRegistered"
        
        # Test JSON serialization
        json_str = event.to_json()
        assert "John Doe" in json_str
        assert "john@example.com" in json_str
        
        # Test deserialization
        event2 = UserRegistered.from_json(json_str)
        assert event2.name == event.name
        assert event2.email == event.email
    
    def test_aggregate_creation(self):
        """Test aggregate creation and event application."""
        user = User()
        assert user.is_new()
        assert user.version == 0
        assert not user.has_uncommitted_events()
        
        # Apply an event
        event = UserRegistered(name="John Doe", email="john@example.com")
        user.apply(event)
        
        assert user.name == "John Doe"
        assert user.email == "john@example.com"
        assert user.version == 1
        assert user.has_uncommitted_events()
        
        events = user.get_uncommitted_events()
        assert len(events) == 1
        assert events[0].name == "John Doe"
    
    def test_aggregate_business_methods(self):
        """Test aggregate business methods."""
        user = User()
        
        # Register user
        register_event = UserRegistered(name="John Doe", email="john@example.com")
        user.apply(register_event)
        
        # Change email
        user.change_email("john.doe@example.com")
        
        assert user.email == "john.doe@example.com"
        assert user.version == 2
        
        events = user.get_uncommitted_events()
        assert len(events) == 2
        assert isinstance(events[1], UserEmailChanged)
        assert events[1].new_email == "john.doe@example.com"
        assert events[1].old_email == "john@example.com"
    
    def test_aggregate_from_events(self):
        """Test reconstructing aggregate from events."""
        # Create events
        event1 = UserRegistered(name="John Doe", email="john@example.com")
        event1.aggregate_id = "user-123"
        event1.aggregate_version = 1
        
        event2 = UserEmailChanged(old_email="john@example.com", new_email="john.doe@example.com")
        event2.aggregate_id = "user-123"
        event2.aggregate_version = 2
        
        events = [event1, event2]
        
        # Reconstruct aggregate
        user = User.from_events(events)
        
        assert user.id == "user-123"
        assert user.name == "John Doe"
        assert user.email == "john.doe@example.com"
        assert user.version == 2
        assert not user.has_uncommitted_events()
        assert not user.is_new()
    
    @pytest.mark.asyncio
    async def test_event_store_creation(self):
        """Test event store creation."""
        # Test SQLite event store creation
        store = await EventStore.create("sqlite://:memory:")
        assert store is not None
        assert store._initialized
    
    def test_event_store_not_initialized(self):
        """Test that uninitialized event store raises error."""
        store = EventStore()
        
        with pytest.raises(RuntimeError, match="EventStore not initialized"):
            store._ensure_initialized()


if __name__ == "__main__":
    pytest.main([__file__])