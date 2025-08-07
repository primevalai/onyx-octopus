"""
Comprehensive test suite for Eventuali Python functionality.
"""

import pytest
import asyncio
import json
from uuid import uuid4
from datetime import datetime
from eventuali import EventStore
from eventuali.event import Event, UserRegistered, UserEmailChanged, UserDeactivated
from eventuali.aggregate import User


class TestEventCreation:
    """Test event creation and serialization."""
    
    def test_event_creation_with_defaults(self):
        """Test creating events with default values."""
        event = UserRegistered(name="Alice Smith", email="alice@example.com")
        
        assert event.name == "Alice Smith"
        assert event.email == "alice@example.com"
        assert event.get_event_type() == "UserRegistered"
        assert event.event_id is not None
        assert event.timestamp is not None
        assert event.event_version == 1
    
    def test_event_serialization_roundtrip(self):
        """Test event serialization and deserialization."""
        original = UserEmailChanged(
            old_email="old@example.com",
            new_email="new@example.com"
        )
        
        # Serialize to JSON
        json_str = original.to_json()
        assert len(json_str) > 0
        assert "old@example.com" in json_str
        assert "new@example.com" in json_str
        
        # Deserialize back
        restored = UserEmailChanged.from_json(json_str)
        assert restored.old_email == original.old_email
        assert restored.new_email == original.new_email
        assert restored.event_id == original.event_id
    
    def test_event_dict_conversion(self):
        """Test event dictionary conversion."""
        event = UserDeactivated(reason="Testing")
        event_dict = event.to_dict()
        
        assert isinstance(event_dict, dict)
        assert event_dict["reason"] == "Testing"
        assert "event_id" in event_dict
        assert "timestamp" in event_dict
        
        # Test from dict
        restored = UserDeactivated.from_dict(event_dict)
        assert restored.reason == event.reason
    
    def test_event_metadata(self):
        """Test event metadata handling."""
        event = UserRegistered(name="Bob", email="bob@example.com")
        event.user_id = "test-user-123"
        event.causation_id = uuid4()
        
        assert event.user_id == "test-user-123"
        assert event.causation_id is not None
        
        # Test serialization preserves metadata
        json_str = event.to_json()
        restored = UserRegistered.from_json(json_str)
        assert restored.user_id == "test-user-123"
        assert restored.causation_id == event.causation_id


class TestAggregateOperations:
    """Test aggregate functionality."""
    
    def test_aggregate_creation(self):
        """Test basic aggregate creation."""
        user = User()
        
        assert user.id is not None
        assert len(user.id) > 0
        assert user.version == 0
        assert user.is_new()
        assert not user.has_uncommitted_events()
        assert user.name == ""
        assert user.email == ""
        assert user.is_active
    
    def test_aggregate_with_custom_id(self):
        """Test creating aggregate with custom ID."""
        custom_id = str(uuid4())
        user = User(id=custom_id)
        
        assert user.id == custom_id
        assert user.version == 0
        assert user.is_new()
    
    def test_event_application(self):
        """Test applying events to aggregates."""
        user = User()
        register_event = UserRegistered(name="Carol", email="carol@example.com")
        
        # Apply event
        user.apply(register_event)
        
        assert user.name == "Carol"
        assert user.email == "carol@example.com"
        assert user.version == 1
        assert user.has_uncommitted_events()
        
        events = user.get_uncommitted_events()
        assert len(events) == 1
        assert events[0].name == "Carol"
    
    def test_multiple_event_application(self):
        """Test applying multiple events."""
        user = User()
        
        # Register user
        register_event = UserRegistered(name="Dave", email="dave@example.com")
        user.apply(register_event)
        
        # Change email
        email_change = UserEmailChanged(
            old_email="dave@example.com",
            new_email="david@newcompany.com"
        )
        user.apply(email_change)
        
        # Deactivate
        deactivate = UserDeactivated(reason="User request")
        user.apply(deactivate)
        
        assert user.name == "Dave"
        assert user.email == "david@newcompany.com"
        assert not user.is_active
        assert user.version == 3
        assert len(user.get_uncommitted_events()) == 3
    
    def test_business_methods(self):
        """Test aggregate business methods."""
        user = User()
        
        # Register through business method (if available)
        register_event = UserRegistered(name="Eve", email="eve@example.com")
        user.apply(register_event)
        
        # Use business method
        user.change_email("eve.smith@example.com")
        
        assert user.email == "eve.smith@example.com"
        assert user.version == 2
        
        events = user.get_uncommitted_events()
        assert len(events) == 2
        assert isinstance(events[1], UserEmailChanged)
    
    def test_event_replay(self):
        """Test aggregate reconstruction from events."""
        # Create events sequence
        events = [
            UserRegistered(name="Frank", email="frank@example.com"),
            UserEmailChanged(old_email="frank@example.com", new_email="franklin@example.com"),
            UserDeactivated(reason="Testing")
        ]
        
        # Set metadata (simulate what event store would do)
        aggregate_id = str(uuid4())
        for i, event in enumerate(events, 1):
            event.aggregate_id = aggregate_id
            event.aggregate_type = "User"
            event.aggregate_version = i
        
        # Reconstruct aggregate
        user = User.from_events(events)
        
        assert user.id == aggregate_id
        assert user.name == "Frank"
        assert user.email == "franklin@example.com"
        assert not user.is_active
        assert user.version == 3
        assert not user.is_new()
        assert not user.has_uncommitted_events()
    
    def test_aggregate_state_consistency(self):
        """Test state consistency between original and reconstructed aggregates."""
        # Create original aggregate
        original = User()
        events = [
            UserRegistered(name="Grace", email="grace@example.com"),
            UserEmailChanged(old_email="grace@example.com", new_email="grace.jones@example.com")
        ]
        
        for event in events:
            original.apply(event)
        
        # Prepare events for replay
        replay_events = original.get_uncommitted_events().copy()
        for i, event in enumerate(replay_events, 1):
            event.aggregate_id = original.id
            event.aggregate_type = "User"
            event.aggregate_version = i
        
        # Reconstruct
        reconstructed = User.from_events(replay_events)
        
        # Compare states (excluding internal fields)
        original_dict = original.to_dict()
        reconstructed_dict = reconstructed.to_dict()
        
        assert original_dict == reconstructed_dict
        assert original.name == reconstructed.name
        assert original.email == reconstructed.email
        assert original.is_active == reconstructed.is_active


class TestEventStoreIntegration:
    """Test event store integration."""
    
    @pytest.mark.asyncio
    async def test_event_store_creation(self):
        """Test creating event store instances."""
        # SQLite in-memory
        store = await EventStore.create("sqlite://:memory:")
        assert store is not None
        assert store._initialized
    
    @pytest.mark.asyncio
    async def test_event_store_error_handling(self):
        """Test event store error conditions."""
        store = EventStore()
        
        # Should raise error when not initialized
        with pytest.raises(RuntimeError, match="EventStore not initialized"):
            store._ensure_initialized()
    
    @pytest.mark.asyncio
    async def test_event_store_connection_strings(self):
        """Test different connection string formats."""
        # In-memory SQLite
        store1 = await EventStore.create("sqlite://:memory:")
        assert store1._initialized
        
        # Try file-based SQLite (might fail due to permissions)
        try:
            store2 = await EventStore.create("sqlite://test.db")
            assert store2._initialized
        except Exception:
            # Expected in restricted environments
            pass


class TestErrorHandling:
    """Test error handling and edge cases."""
    
    def test_invalid_event_data(self):
        """Test handling of invalid event data."""
        # Missing required fields should raise validation error
        with pytest.raises(Exception):
            UserRegistered(name="Test")  # Missing email
    
    def test_aggregate_invariants(self):
        """Test aggregate invariant enforcement."""
        user = User()
        
        # Try applying event without registration first
        email_change = UserEmailChanged(
            old_email="",
            new_email="test@example.com"
        )
        
        # Should still apply (no invariant enforcement in this simple example)
        user.apply(email_change)
        assert user.version == 1
    
    def test_event_version_handling(self):
        """Test event version handling."""
        event = UserRegistered(name="Test", email="test@example.com")
        
        # Default version should be 1
        assert event.event_version == 1
        
        # Should be able to modify version
        event.event_version = 2
        assert event.event_version == 2


class TestPerformance:
    """Test performance characteristics."""
    
    def test_event_creation_performance(self):
        """Test event creation performance."""
        import time
        
        event_count = 1000
        start_time = time.time()
        
        events = []
        for i in range(event_count):
            event = UserRegistered(
                name=f"User {i}",
                email=f"user{i}@example.com"
            )
            events.append(event)
        
        end_time = time.time()
        duration = end_time - start_time
        events_per_sec = event_count / duration
        
        print(f"Created {event_count} events in {duration:.3f}s ({events_per_sec:.0f} events/sec)")
        assert events_per_sec > 1000  # Should be very fast for pure Python objects
    
    def test_event_serialization_performance(self):
        """Test event serialization performance."""
        import time
        
        # Create test events
        events = []
        for i in range(100):
            event = UserRegistered(name=f"User {i}", email=f"user{i}@example.com")
            events.append(event)
        
        # Test JSON serialization
        start_time = time.time()
        json_strings = []
        for event in events:
            json_str = event.to_json()
            json_strings.append(json_str)
        end_time = time.time()
        
        serialize_duration = end_time - start_time
        
        # Test deserialization
        start_time = time.time()
        for json_str in json_strings:
            restored = UserRegistered.from_json(json_str)
        end_time = time.time()
        
        deserialize_duration = end_time - start_time
        
        print(f"Serialized 100 events in {serialize_duration:.3f}s")
        print(f"Deserialized 100 events in {deserialize_duration:.3f}s")
        
        # Should be reasonably fast
        assert serialize_duration < 1.0
        assert deserialize_duration < 1.0
    
    def test_aggregate_replay_performance(self):
        """Test aggregate replay performance."""
        import time
        
        # Create many events
        events = []
        for i in range(100):
            if i == 0:
                event = UserRegistered(name=f"User {i}", email=f"user{i}@example.com")
            elif i % 2 == 0:
                event = UserEmailChanged(
                    old_email=f"user{i-1}@example.com",
                    new_email=f"user{i}@example.com"
                )
            else:
                event = UserDeactivated(reason=f"Test {i}")
            
            event.aggregate_id = "test-user"
            event.aggregate_version = i + 1
            events.append(event)
        
        # Test replay performance
        start_time = time.time()
        user = User.from_events(events)
        end_time = time.time()
        
        replay_duration = end_time - start_time
        events_per_sec = len(events) / replay_duration
        
        print(f"Replayed {len(events)} events in {replay_duration:.3f}s ({events_per_sec:.0f} events/sec)")
        
        assert user.version == 100
        assert events_per_sec > 1000  # Should replay very quickly


@pytest.mark.asyncio
async def test_async_workflow():
    """Test async workflow integration."""
    store = await EventStore.create("sqlite://:memory:")
    
    # This tests that the async store creation works
    # Full integration tests would require the Rust backend to be complete
    assert store._initialized
    
    print("✓ Async workflow test passed")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])