#!/usr/bin/env python3
"""
Tests for custom event subclass deserialization functionality.

These tests verify that the EventStore properly handles custom event classes
with additional fields through the event registry system.
"""

import asyncio
import pytest
import sys
import os
from typing import Dict, Any
from datetime import datetime, timezone

# Add the python package to the path
sys.path.insert(
    0, os.path.join(os.path.dirname(__file__), "..", "python")
)

from eventuali import EventStore, Event, Aggregate


class TestEvent(Event):
    """Test event class with custom fields."""
    
    custom_field: str = ""
    numeric_field: int = 0
    dict_field: Dict[str, Any] = {}
    list_field: list = []
    
    def get_event_type(self) -> str:
        return "TestEvent"


class AnotherTestEvent(Event):
    """Another test event class with different fields."""
    
    agent_name: str = ""
    agent_id: str = ""
    attributes: Dict[str, Any] = {}
    
    def get_event_type(self) -> str:
        return "AnotherTestEvent"


class TestAggregate(Aggregate):
    """Test aggregate for testing events."""
    
    # Additional fields for the aggregate
    state: str = "initial"
    
    def get_aggregate_type(self) -> str:
        return "test_aggregate"
    
    def do_something(self, data: str):
        """Trigger a test event."""
        event = TestEvent(
            aggregate_id=self.id,
            aggregate_type=self.get_aggregate_type(),
            custom_field=data,
            numeric_field=42,
            dict_field={"key": "value", "nested": {"data": 123}},
            list_field=["item1", "item2", {"complex": "item"}]
        )
        self.apply(event)
    
    def apply_test_event(self, event: Event):
        """Apply TestEvent events to update state."""
        if isinstance(event, TestEvent):
            self.state = f"processed_{event.custom_field}"


@pytest.mark.asyncio
async def test_event_registration():
    """Test event class registration functionality."""
    
    # Clear any existing registrations from other tests
    EventStore._event_registry.clear()
    
    # Test registering a valid event class
    EventStore.register_event_class("test_event", TestEvent)
    
    registered = EventStore.get_registered_event_classes()
    assert "test_event" in registered
    assert registered["test_event"] == TestEvent
    
    # Test registering multiple classes
    EventStore.register_event_class("another_test_event", AnotherTestEvent)
    
    registered = EventStore.get_registered_event_classes()
    assert len(registered) == 2
    assert "another_test_event" in registered
    assert registered["another_test_event"] == AnotherTestEvent
    
    # Test unregistering
    EventStore.unregister_event_class("test_event")
    registered = EventStore.get_registered_event_classes()
    assert "test_event" not in registered
    assert "another_test_event" in registered


@pytest.mark.asyncio
async def test_invalid_event_registration():
    """Test that invalid event classes are rejected."""
    
    # Try to register a non-Event class
    class NotAnEvent:
        pass
    
    with pytest.raises(ValueError, match="must be a subclass of Event"):
        EventStore.register_event_class("invalid", NotAnEvent)


@pytest.mark.asyncio
async def test_custom_event_deserialization():
    """Test that registered custom events are properly deserialized."""
    
    # Clear registrations and create event store
    EventStore._event_registry.clear()
    event_store = await EventStore.create("sqlite://:memory:")
    
    # Register the test event class
    EventStore.register_event_class("TestEvent", TestEvent)
    
    # Create and save an aggregate with custom events
    aggregate = TestAggregate(id="test-123")
    aggregate.do_something("test_data")
    
    await event_store.save(aggregate)
    
    # Load events and verify they're deserialized as the correct class
    events = await event_store.load_events_by_type("test_aggregate")
    
    assert len(events) == 1
    event = events[0]
    
    # Verify it's the correct class
    assert isinstance(event, TestEvent)
    assert type(event).__name__ == "TestEvent"
    
    # Verify all custom fields are preserved
    assert event.custom_field == "test_data"
    assert event.numeric_field == 42
    assert event.dict_field == {"key": "value", "nested": {"data": 123}}
    assert event.list_field == ["item1", "item2", {"complex": "item"}]
    
    # Verify base Event fields are also present
    assert event.aggregate_id == "test-123"
    assert event.aggregate_type == "test_aggregate"
    assert event.event_type == "TestEvent"


@pytest.mark.asyncio
async def test_unregistered_event_fallback():
    """Test that unregistered events fall back to base Event but preserve fields."""
    
    # Clear registrations and create event store
    EventStore._event_registry.clear()
    event_store = await EventStore.create("sqlite://:memory:")
    
    # Create and save an event without registering its class
    aggregate = TestAggregate(id="test-456")
    
    # Create a custom event manually
    custom_event = TestEvent(
        aggregate_id=aggregate.id,
        aggregate_type=aggregate.get_aggregate_type(),
        custom_field="unregistered_data",
        numeric_field=999,
        dict_field={"unregistered": True},
        list_field=["unregistered", "fields"]
    )
    aggregate.apply(custom_event)
    
    await event_store.save(aggregate)
    
    # Load events - should fall back to base Event class
    events = await event_store.load_events_by_type("test_aggregate")
    
    assert len(events) == 1
    event = events[0]
    
    # Should be base Event class, not TestEvent
    assert type(event).__name__ == "Event"
    assert not isinstance(event, TestEvent)
    
    # But custom fields should be preserved as extra attributes (due to extra="allow")
    event_dict = event.to_dict()
    assert "custom_field" in event_dict
    assert "numeric_field" in event_dict
    assert "dict_field" in event_dict
    assert "list_field" in event_dict
    
    # Verify the values are correct
    assert event_dict["custom_field"] == "unregistered_data"
    assert event_dict["numeric_field"] == 999
    assert event_dict["dict_field"] == {"unregistered": True}
    assert event_dict["list_field"] == ["unregistered", "fields"]


@pytest.mark.asyncio
async def test_aggregate_reconstruction_with_custom_events(event_store):
    """Test that aggregates can be reconstructed from custom events."""
    
    # Register the test event class
    EventStore.register_event_class("TestEvent", TestEvent)
    
    # Create, modify, and save an aggregate
    original_aggregate = TestAggregate(id="test-789")
    original_aggregate.do_something("reconstruction_test")
    
    await event_store.save(original_aggregate)
    
    # Load the aggregate back from events
    loaded_aggregate = await event_store.load(TestAggregate, "test-789")
    
    assert loaded_aggregate is not None
    assert loaded_aggregate.id == "test-789"
    assert loaded_aggregate.state == "processed_reconstruction_test"
    assert loaded_aggregate.version == 1


@pytest.mark.asyncio
async def test_multiple_custom_event_types(event_store):
    """Test handling multiple different custom event types."""
    
    # Register multiple event classes
    EventStore.register_event_class("TestEvent", TestEvent)
    EventStore.register_event_class("AnotherTestEvent", AnotherTestEvent)
    
    # Create aggregate with multiple event types
    aggregate = TestAggregate(id="multi-test")
    
    # Add TestEvent
    test_event = TestEvent(
        aggregate_id=aggregate.id,
        aggregate_type=aggregate.get_aggregate_type(),
        custom_field="first_event",
        numeric_field=100
    )
    aggregate.apply(test_event)
    
    # Add AnotherTestEvent
    another_event = AnotherTestEvent(
        aggregate_id=aggregate.id,
        aggregate_type=aggregate.get_aggregate_type(),
        agent_name="test_agent",
        agent_id="agent-123",
        attributes={"action": "test", "timestamp": datetime.now(timezone.utc).isoformat()}
    )
    aggregate.apply(another_event)
    
    await event_store.save(aggregate)
    
    # Load events and verify correct deserialization
    events = await event_store.load_events("multi-test")
    
    assert len(events) == 2
    
    # First event should be TestEvent
    first_event = events[0]
    assert isinstance(first_event, TestEvent)
    assert first_event.custom_field == "first_event"
    assert first_event.numeric_field == 100
    
    # Second event should be AnotherTestEvent
    second_event = events[1]
    assert isinstance(second_event, AnotherTestEvent)
    assert second_event.agent_name == "test_agent"
    assert second_event.agent_id == "agent-123"
    assert "action" in second_event.attributes
    assert second_event.attributes["action"] == "test"


@pytest.mark.asyncio
async def test_event_type_mismatch_handling(event_store):
    """Test handling when event_type doesn't match the registered class."""
    
    # Register TestEvent for a different event_type
    EventStore.register_event_class("different_type", TestEvent)
    
    # Create an event with event_type that doesn't match any registration
    aggregate = TestAggregate(id="mismatch-test")
    
    event = TestEvent(
        aggregate_id=aggregate.id,
        aggregate_type=aggregate.get_aggregate_type(),
        custom_field="mismatch_test"
    )
    # Override the event_type to something not registered
    event.event_type = "unregistered_type"
    
    aggregate.apply(event)
    await event_store.save(aggregate)
    
    # Load events - should fall back to base Event
    events = await event_store.load_events_by_type("test_aggregate")
    
    assert len(events) == 1
    loaded_event = events[0]
    
    # Should be base Event, not TestEvent
    assert type(loaded_event).__name__ == "Event"
    assert loaded_event.event_type == "unregistered_type"
    
    # Custom fields should still be preserved
    event_dict = loaded_event.to_dict()
    assert event_dict["custom_field"] == "mismatch_test"


@pytest.mark.asyncio
async def test_complex_nested_data_preservation(event_store):
    """Test that complex nested data structures are preserved."""
    
    EventStore.register_event_class("complex_event", TestEvent)
    
    complex_data = {
        "level1": {
            "level2": {
                "level3": ["deep", "nested", "array"],
                "numbers": [1, 2, 3.14, 42],
                "mixed": {"strings": "text", "bools": True, "nulls": None}
            },
            "arrays": [
                {"object": "in_array"},
                ["nested", "array", "in", "array"],
                123
            ]
        },
        "unicode": "测试 🚀 العربية",
        "timestamps": datetime.now(timezone.utc).isoformat()
    }
    
    aggregate = TestAggregate(id="complex-test")
    event = TestEvent(
        aggregate_id=aggregate.id,
        aggregate_type=aggregate.get_aggregate_type(),
        custom_field="complex_test",
        dict_field=complex_data,
        list_field=[complex_data, "simple_string", 42, True, None]
    )
    aggregate.apply(event)
    
    await event_store.save(aggregate)
    
    # Load and verify complex data is preserved
    events = await event_store.load_events_by_type("test_aggregate")
    
    assert len(events) == 1
    loaded_event = events[0]
    
    assert isinstance(loaded_event, TestEvent)
    assert loaded_event.dict_field == complex_data
    assert loaded_event.list_field[0] == complex_data
    assert loaded_event.list_field[1] == "simple_string"
    assert loaded_event.list_field[2] == 42
    assert loaded_event.list_field[3] is True
    assert loaded_event.list_field[4] is None


if __name__ == "__main__":
    # Run the tests
    pytest.main([__file__, "-v"])