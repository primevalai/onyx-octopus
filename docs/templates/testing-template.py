"""
Advanced Testing Template for Eventuali Event Sourcing

This template provides comprehensive testing patterns for event-sourced systems
including unit tests, integration tests, and property-based testing strategies.

Usage:
1. Replace {{AGGREGATE_NAME}} with your aggregate name (e.g., User, Order)
2. Replace {{SERVICE_NAME}} with your service name (e.g., UserService, OrderService)
3. Add your specific test scenarios and business rules
4. Implement domain-specific test data generators

Testing Patterns:
- Aggregate behavior testing with Given-When-Then
- Event sourcing scenarios with event replay
- Integration testing with real EventStore
- Performance testing and load testing
- Property-based testing for business invariants
"""

import pytest
import asyncio
from typing import List, Dict, Any, Optional, Type, Callable
from datetime import datetime, timedelta
from decimal import Decimal
from uuid import uuid4
import unittest.mock as mock
from hypothesis import given, strategies as st, settings, assume
from hypothesis.stateful import RuleBasedStateMachine, rule, initialize, precondition

# Import your domain classes
from eventuali import Event, Aggregate, EventStore, EventStreamer
from eventuali.streaming import Subscription, Projection
from eventuali.exceptions import EventualiError, OptimisticConcurrencyError

# Import your domain implementation (replace with actual imports)
# from your_domain.aggregates import {{AGGREGATE_NAME}}
# from your_domain.events import {{AGGREGATE_NAME}}Created, {{AGGREGATE_NAME}}Updated
# from your_domain.services import {{SERVICE_NAME}}

# =============================================================================
# TEST FIXTURES AND UTILITIES
# =============================================================================

@pytest.fixture
async def event_store():
    """Create in-memory event store for testing."""
    store = await EventStore.create("sqlite://:memory:")
    
    # Register your events here
    # store.register_event_class("{{AGGREGATE_NAME}}Created", {{AGGREGATE_NAME}}Created)
    # store.register_event_class("{{AGGREGATE_NAME}}Updated", {{AGGREGATE_NAME}}Updated)
    
    yield store
    
    # Cleanup
    # No explicit cleanup needed for in-memory SQLite

@pytest.fixture
async def event_streamer():
    """Create event streamer for testing."""
    return EventStreamer(capacity=1000)

@pytest.fixture
def sample_aggregate_id():
    """Generate unique aggregate ID for tests."""
    return f"test-{{aggregate_name_lower}}-{uuid4().hex[:8]}"

@pytest.fixture
def sample_user_id():
    """Generate unique user ID for tests."""
    return f"test-user-{uuid4().hex[:8]}"

class TestDataGenerator:
    """Generator for test data and scenarios."""
    
    @staticmethod
    def create_valid_{{aggregate_name_lower}}_data() -> Dict[str, Any]:
        """Create valid {{aggregate_name_lower}} creation data."""
        return {
            "name": f"Test {{AGGREGATE_NAME}} {uuid4().hex[:8]}",
            "description": "Test description for {{aggregate_name_lower}}",
            "custom_field_1": "test_value_1",
            "custom_field_2": Decimal("123.45")
        }
    
    @staticmethod
    def create_invalid_{{aggregate_name_lower}}_data() -> List[Dict[str, Any]]:
        """Create various invalid {{aggregate_name_lower}} data scenarios."""
        return [
            {"name": ""},  # Empty name
            {"name": None},  # None name
            {"name": "x" * 201},  # Too long name
            {},  # Missing required fields
            {"name": "  "},  # Whitespace only
        ]
    
    @staticmethod
    def create_{{aggregate_name_lower}}_update_data() -> Dict[str, Any]:
        """Create valid update data."""
        return {
            "name": f"Updated {{AGGREGATE_NAME}} {uuid4().hex[:8]}",
            "description": "Updated description",
            "custom_field_1": "updated_value"
        }
    
    @staticmethod
    def create_event_sequence() -> List[Dict[str, Any]]:
        """Create a sequence of events for scenario testing."""
        base_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        update_data = TestDataGenerator.create_{{aggregate_name_lower}}_update_data()
        
        return [
            {"event_type": "{{AGGREGATE_NAME}}Created", "data": base_data},
            {"event_type": "{{AGGREGATE_NAME}}Updated", "data": update_data},
            {"event_type": "{{AGGREGATE_NAME}}StatusChanged", "data": {"new_status": "active", "reason": "Test activation"}},
        ]

# =============================================================================
# GIVEN-WHEN-THEN TEST HELPERS
# =============================================================================

class GivenWhenThenTest:
    """Base class for Given-When-Then style testing."""
    
    def __init__(self):
        self.aggregate = None
        self.events = []
        self.exception = None
        self.result = None
    
    def given_new_{{aggregate_name_lower}}(self, aggregate_id: str = None):
        """Given a new {{aggregate_name_lower}} aggregate."""
        # self.aggregate = {{AGGREGATE_NAME}}(id=aggregate_id or str(uuid4()))
        return self
    
    def given_existing_{{aggregate_name_lower}}(self, events: List[Event]):
        """Given an existing {{aggregate_name_lower}} with event history."""
        # aggregate_id = events[0].aggregate_id if events else str(uuid4())
        # self.aggregate = {{AGGREGATE_NAME}}(id=aggregate_id)
        # self.aggregate.replay_events(events)
        return self
    
    def when_creating_{{aggregate_name_lower}}(self, **kwargs):
        """When creating a {{aggregate_name_lower}}."""
        try:
            # self.aggregate.create(**kwargs)
            # self.events = self.aggregate.get_uncommitted_events()
            pass
        except Exception as e:
            self.exception = e
        return self
    
    def when_updating_{{aggregate_name_lower}}(self, **kwargs):
        """When updating a {{aggregate_name_lower}}."""
        try:
            # self.aggregate.update(**kwargs)
            # self.events = self.aggregate.get_uncommitted_events()
            pass
        except Exception as e:
            self.exception = e
        return self
    
    def when_changing_status(self, new_status: str, reason: str, changed_by: str):
        """When changing {{aggregate_name_lower}} status."""
        try:
            # self.aggregate.change_status(new_status, reason, changed_by)
            # self.events = self.aggregate.get_uncommitted_events()
            pass
        except Exception as e:
            self.exception = e
        return self
    
    def then_should_succeed(self):
        """Then the operation should succeed."""
        assert self.exception is None, f"Expected success but got exception: {self.exception}"
        return self
    
    def then_should_fail_with(self, exception_type: Type[Exception]):
        """Then the operation should fail with specific exception."""
        assert self.exception is not None, "Expected exception but operation succeeded"
        assert isinstance(self.exception, exception_type), f"Expected {exception_type} but got {type(self.exception)}"
        return self
    
    def then_should_have_events(self, count: int):
        """Then should have specific number of events."""
        assert len(self.events) == count, f"Expected {count} events but got {len(self.events)}"
        return self
    
    def then_should_have_event_type(self, event_type: str):
        """Then should have event of specific type."""
        event_types = [event.event_type for event in self.events]
        assert event_type in event_types, f"Expected event type {event_type} but got {event_types}"
        return self
    
    def then_aggregate_should_have(self, **expected_values):
        """Then aggregate should have specific property values."""
        for prop, expected_value in expected_values.items():
            actual_value = getattr(self.aggregate, prop, None)
            assert actual_value == expected_value, f"Expected {prop}={expected_value} but got {actual_value}"
        return self

# =============================================================================
# UNIT TESTS - AGGREGATE BEHAVIOR
# =============================================================================

class Test{{AGGREGATE_NAME}}Aggregate:
    """Unit tests for {{AGGREGATE_NAME}} aggregate behavior."""
    
    def test_create_new_{{aggregate_name_lower}}_success(self, sample_aggregate_id, sample_user_id):
        """Test successful {{aggregate_name_lower}} creation."""
        data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        
        test = GivenWhenThenTest()
        (test
         .given_new_{{aggregate_name_lower}}(sample_aggregate_id)
         .when_creating_{{aggregate_name_lower}}(created_by=sample_user_id, **data)
         .then_should_succeed()
         .then_should_have_events(1)
         .then_should_have_event_type("{{AGGREGATE_NAME}}Created")
         .then_aggregate_should_have(
             name=data["name"],
             description=data["description"],
             version=1
         ))
    
    @pytest.mark.parametrize("invalid_data", TestDataGenerator.create_invalid_{{aggregate_name_lower}}_data())
    def test_create_{{aggregate_name_lower}}_validation_failures(self, sample_aggregate_id, sample_user_id, invalid_data):
        """Test {{aggregate_name_lower}} creation validation failures."""
        test = GivenWhenThenTest()
        (test
         .given_new_{{aggregate_name_lower}}(sample_aggregate_id)
         .when_creating_{{aggregate_name_lower}}(created_by=sample_user_id, **invalid_data)
         .then_should_fail_with(ValueError))  # Or your specific validation exception
    
    def test_update_existing_{{aggregate_name_lower}}(self, sample_aggregate_id, sample_user_id):
        """Test updating existing {{aggregate_name_lower}}."""
        # Create initial {{aggregate_name_lower}}
        create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        update_data = TestDataGenerator.create_{{aggregate_name_lower}}_update_data()
        
        test = GivenWhenThenTest()
        (test
         .given_new_{{aggregate_name_lower}}(sample_aggregate_id)
         .when_creating_{{aggregate_name_lower}}(created_by=sample_user_id, **create_data)
         .then_should_succeed())
        
        # Clear events and update
        test.events.clear()
        (test
         .when_updating_{{aggregate_name_lower}}(updated_by=sample_user_id, **update_data)
         .then_should_succeed()
         .then_should_have_events(1)
         .then_should_have_event_type("{{AGGREGATE_NAME}}Updated")
         .then_aggregate_should_have(
             name=update_data["name"],
             description=update_data["description"],
             version=2
         ))
    
    def test_cannot_update_non_existent_{{aggregate_name_lower}}(self, sample_aggregate_id, sample_user_id):
        """Test that updating non-existent {{aggregate_name_lower}} fails."""
        update_data = TestDataGenerator.create_{{aggregate_name_lower}}_update_data()
        
        test = GivenWhenThenTest()
        (test
         .given_new_{{aggregate_name_lower}}(sample_aggregate_id)
         .when_updating_{{aggregate_name_lower}}(updated_by=sample_user_id, **update_data)
         .then_should_fail_with(ValueError))  # Or your specific exception
    
    def test_status_transitions(self, sample_aggregate_id, sample_user_id):
        """Test valid status transitions."""
        create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        
        test = GivenWhenThenTest()
        (test
         .given_new_{{aggregate_name_lower}}(sample_aggregate_id)
         .when_creating_{{aggregate_name_lower}}(created_by=sample_user_id, **create_data)
         .then_should_succeed())
        
        # Test status change
        test.events.clear()
        (test
         .when_changing_status("suspended", "Test suspension", sample_user_id)
         .then_should_succeed()
         .then_should_have_events(1)
         .then_should_have_event_type("{{AGGREGATE_NAME}}StatusChanged"))

# =============================================================================
# INTEGRATION TESTS - EVENT STORE
# =============================================================================

class Test{{AGGREGATE_NAME}}Integration:
    """Integration tests with EventStore."""
    
    @pytest.mark.asyncio
    async def test_save_and_load_{{aggregate_name_lower}}(self, event_store, sample_aggregate_id, sample_user_id):
        """Test saving and loading {{aggregate_name_lower}} with EventStore."""
        # Create and save {{aggregate_name_lower}}
        # aggregate = {{AGGREGATE_NAME}}(id=sample_aggregate_id)
        # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        # aggregate.create(created_by=sample_user_id, **create_data)
        
        # await event_store.save(aggregate)
        # aggregate.mark_events_as_committed()
        
        # Load {{aggregate_name_lower}}
        # loaded_aggregate = await event_store.load({{AGGREGATE_NAME}}, sample_aggregate_id)
        
        # assert loaded_aggregate is not None
        # assert loaded_aggregate.id == sample_aggregate_id
        # assert loaded_aggregate.name == create_data["name"]
        # assert loaded_aggregate.version == 1
        pass
    
    @pytest.mark.asyncio
    async def test_optimistic_concurrency_control(self, event_store, sample_aggregate_id, sample_user_id):
        """Test optimistic concurrency control."""
        # Create initial {{aggregate_name_lower}}
        # aggregate1 = {{AGGREGATE_NAME}}(id=sample_aggregate_id)
        # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        # aggregate1.create(created_by=sample_user_id, **create_data)
        # await event_store.save(aggregate1)
        # aggregate1.mark_events_as_committed()
        
        # Load two instances
        # aggregate_a = await event_store.load({{AGGREGATE_NAME}}, sample_aggregate_id)
        # aggregate_b = await event_store.load({{AGGREGATE_NAME}}, sample_aggregate_id)
        
        # Modify both
        # aggregate_a.update(updated_by=sample_user_id, name="Updated by A")
        # aggregate_b.update(updated_by=sample_user_id, name="Updated by B")
        
        # Save first one should succeed
        # await event_store.save(aggregate_a)
        # aggregate_a.mark_events_as_committed()
        
        # Save second one should fail with concurrency error
        # with pytest.raises(OptimisticConcurrencyError):
        #     await event_store.save(aggregate_b)
        pass
    
    @pytest.mark.asyncio
    async def test_event_replay_reconstruction(self, event_store, sample_aggregate_id, sample_user_id):
        """Test aggregate reconstruction from event replay."""
        # Create and modify {{aggregate_name_lower}} multiple times
        # aggregate = {{AGGREGATE_NAME}}(id=sample_aggregate_id)
        # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        # update_data = TestDataGenerator.create_{{aggregate_name_lower}}_update_data()
        
        # aggregate.create(created_by=sample_user_id, **create_data)
        # await event_store.save(aggregate)
        # aggregate.mark_events_as_committed()
        
        # aggregate.update(updated_by=sample_user_id, **update_data)
        # await event_store.save(aggregate)
        # aggregate.mark_events_as_committed()
        
        # aggregate.change_status("suspended", "Test", sample_user_id)
        # await event_store.save(aggregate)
        # aggregate.mark_events_as_committed()
        
        # Load events and reconstruct
        # events = await event_store.load_events(sample_aggregate_id)
        # assert len(events) == 3
        
        # reconstructed = {{AGGREGATE_NAME}}(id=sample_aggregate_id)
        # reconstructed.replay_events(events)
        
        # assert reconstructed.name == update_data["name"]
        # assert reconstructed.version == 3
        # assert reconstructed.status == "suspended"
        pass

# =============================================================================
# PROJECTION TESTING
# =============================================================================

class Test{{AGGREGATE_NAME}}Projection:
    """Tests for {{AGGREGATE_NAME}} projections."""
    
    @pytest.mark.asyncio
    async def test_projection_handles_creation_event(self, event_streamer):
        """Test projection handles creation events correctly."""
        # from your_projections import {{AGGREGATE_NAME}}Projection
        # projection = {{AGGREGATE_NAME}}Projection(store=InMemoryReadModelStore())
        
        # Create and apply event
        # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        # event = {{AGGREGATE_NAME}}Created(
        #     aggregate_id="test-123",
        #     aggregate_type="{{AGGREGATE_NAME}}",
        #     aggregate_version=1,
        #     **create_data
        # )
        
        # await projection.handle_event(event)
        
        # Check read model was created
        # read_model = await projection.get_{{aggregate_name_lower}}("test-123")
        # assert read_model is not None
        # assert read_model["name"] == create_data["name"]
        # assert read_model["status"] == "active"
        pass
    
    @pytest.mark.asyncio
    async def test_projection_handles_update_event(self, event_streamer):
        """Test projection handles update events correctly."""
        # Similar to creation test but with update event
        pass
    
    @pytest.mark.asyncio
    async def test_projection_query_methods(self, event_streamer):
        """Test projection query methods."""
        # Test various query scenarios
        pass

# =============================================================================
# PERFORMANCE TESTING
# =============================================================================

class Test{{AGGREGATE_NAME}}Performance:
    """Performance tests for {{AGGREGATE_NAME}} operations."""
    
    @pytest.mark.asyncio
    @pytest.mark.performance
    async def test_aggregate_creation_performance(self, event_store):
        """Test aggregate creation performance."""
        num_aggregates = 1000
        start_time = datetime.now()
        
        for i in range(num_aggregates):
            # aggregate_id = f"perf-test-{i}"
            # aggregate = {{AGGREGATE_NAME}}(id=aggregate_id)
            # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
            # aggregate.create(created_by="perf-test", **create_data)
            pass
        
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        rate = num_aggregates / duration
        
        print(f"Created {num_aggregates} aggregates in {duration:.3f}s ({rate:.0f} ops/sec)")
        assert rate > 10000, f"Expected >10k ops/sec, got {rate:.0f}"  # Adjust threshold as needed
    
    @pytest.mark.asyncio
    @pytest.mark.performance
    async def test_event_store_save_performance(self, event_store):
        """Test EventStore save performance."""
        num_saves = 100
        aggregates = []
        
        # Create aggregates
        for i in range(num_saves):
            # aggregate_id = f"save-perf-{i}"
            # aggregate = {{AGGREGATE_NAME}}(id=aggregate_id)
            # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
            # aggregate.create(created_by="perf-test", **create_data)
            # aggregates.append(aggregate)
            pass
        
        # Time the saves
        start_time = datetime.now()
        
        for aggregate in aggregates:
            # await event_store.save(aggregate)
            # aggregate.mark_events_as_committed()
            pass
        
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        rate = num_saves / duration
        
        print(f"Saved {num_saves} aggregates in {duration:.3f}s ({rate:.0f} saves/sec)")
        # assert rate > 1000, f"Expected >1k saves/sec, got {rate:.0f}"  # Adjust threshold
    
    @pytest.mark.asyncio
    @pytest.mark.performance
    async def test_event_stream_processing_performance(self, event_streamer):
        """Test event stream processing performance."""
        num_events = 1000
        events_processed = 0
        
        # Create subscription
        subscription = Subscription(
            id="perf-test",
            aggregate_type_filter="{{AGGREGATE_NAME}}"
        )
        receiver = await event_streamer.subscribe(subscription)
        
        # Process events
        start_time = datetime.now()
        
        async def process_events():
            nonlocal events_processed
            async for stream_event in receiver:
                events_processed += 1
                if events_processed >= num_events:
                    break
        
        # Start processing
        process_task = asyncio.create_task(process_events())
        
        # Publish events
        for i in range(num_events):
            # create_data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
            # event = {{AGGREGATE_NAME}}Created(
            #     aggregate_id=f"perf-{i}",
            #     aggregate_type="{{AGGREGATE_NAME}}",
            #     aggregate_version=1,
            #     **create_data
            # )
            # await event_streamer.publish_event(event, 1, i)
            pass
        
        # Wait for processing to complete
        await process_task
        
        end_time = datetime.now()
        duration = (end_time - start_time).total_seconds()
        rate = events_processed / duration
        
        print(f"Processed {events_processed} events in {duration:.3f}s ({rate:.0f} events/sec)")
        # assert rate > 50000, f"Expected >50k events/sec, got {rate:.0f}"  # Adjust threshold

# =============================================================================
# PROPERTY-BASED TESTING
# =============================================================================

# Hypothesis strategies for generating test data
{{aggregate_name_lower}}_name_strategy = st.text(min_size=1, max_size=200).filter(lambda x: x.strip() != "")
{{aggregate_name_lower}}_description_strategy = st.one_of(st.none(), st.text(max_size=1000))
user_id_strategy = st.text(min_size=1, max_size=50).filter(lambda x: x.strip() != "")

class Test{{AGGREGATE_NAME}}Properties:
    """Property-based tests for {{AGGREGATE_NAME}} invariants."""
    
    @given(
        name={{aggregate_name_lower}}_name_strategy,
        description={{aggregate_name_lower}}_description_strategy,
        user_id=user_id_strategy
    )
    @settings(max_examples=100)
    def test_{{aggregate_name_lower}}_creation_properties(self, name, description, user_id):
        """Test {{aggregate_name_lower}} creation properties hold for all valid inputs."""
        assume(len(name.strip()) > 0)  # Ensure valid name
        
        # aggregate = {{AGGREGATE_NAME}}(id=str(uuid4()))
        # aggregate.create(name=name, description=description, created_by=user_id)
        
        # Invariants that should always hold
        # assert aggregate.name == name.strip()
        # assert aggregate.description == (description.strip() if description else None)
        # assert aggregate.version == 1
        # assert len(aggregate.get_uncommitted_events()) == 1
        # assert aggregate.get_uncommitted_events()[0].event_type == "{{AGGREGATE_NAME}}Created"
        pass
    
    @given(
        initial_name={{aggregate_name_lower}}_name_strategy,
        updated_name={{aggregate_name_lower}}_name_strategy,
        user_id=user_id_strategy
    )
    @settings(max_examples=50)
    def test_{{aggregate_name_lower}}_update_properties(self, initial_name, updated_name, user_id):
        """Test {{aggregate_name_lower}} update properties."""
        assume(len(initial_name.strip()) > 0)
        assume(len(updated_name.strip()) > 0)
        assume(initial_name != updated_name)  # Ensure actual change
        
        # Create initial {{aggregate_name_lower}}
        # aggregate = {{AGGREGATE_NAME}}(id=str(uuid4()))
        # aggregate.create(name=initial_name, created_by=user_id)
        # aggregate.mark_events_as_committed()
        
        # Update {{aggregate_name_lower}}
        # aggregate.update(name=updated_name, updated_by=user_id)
        
        # Invariants
        # assert aggregate.name == updated_name.strip()
        # assert aggregate.version == 2
        # assert len(aggregate.get_uncommitted_events()) == 1
        # assert aggregate.get_uncommitted_events()[0].event_type == "{{AGGREGATE_NAME}}Updated"
        pass

# =============================================================================
# STATEFUL TESTING
# =============================================================================

class {{AGGREGATE_NAME}}StateMachine(RuleBasedStateMachine):
    """Stateful testing for {{AGGREGATE_NAME}} aggregate."""
    
    def __init__(self):
        super().__init__()
        self.aggregate = None
        self.expected_version = 0
        self.is_created = False
    
    @initialize()
    def init_aggregate(self):
        """Initialize aggregate for testing."""
        # self.aggregate = {{AGGREGATE_NAME}}(id=str(uuid4()))
        self.expected_version = 0
        self.is_created = False
    
    @rule()
    @precondition(lambda self: not self.is_created)
    def create_{{aggregate_name_lower}}(self):
        """Create {{aggregate_name_lower}} if not already created."""
        # data = TestDataGenerator.create_valid_{{aggregate_name_lower}}_data()
        # self.aggregate.create(created_by="test-user", **data)
        self.is_created = True
        self.expected_version += 1
        
        # Invariants
        # assert self.aggregate.version == self.expected_version
        # assert len(self.aggregate.get_uncommitted_events()) == 1
    
    @rule()
    @precondition(lambda self: self.is_created)
    def update_{{aggregate_name_lower}}(self):
        """Update {{aggregate_name_lower}} if created."""
        # data = TestDataGenerator.create_{{aggregate_name_lower}}_update_data()
        # self.aggregate.update(updated_by="test-user", **data)
        self.expected_version += 1
        
        # Invariants
        # assert self.aggregate.version == self.expected_version
    
    @rule()
    @precondition(lambda self: self.is_created)
    def change_status(self):
        """Change {{aggregate_name_lower}} status."""
        # self.aggregate.change_status("suspended", "Test", "test-user")
        self.expected_version += 1
        
        # Invariants
        # assert self.aggregate.version == self.expected_version

# Run stateful tests
Test{{AGGREGATE_NAME}}StateMachine = {{AGGREGATE_NAME}}StateMachine.TestCase

# =============================================================================
# TEST UTILITIES AND HELPERS
# =============================================================================

class EventStoreTestHelper:
    """Helper utilities for event store testing."""
    
    @staticmethod
    async def create_test_events(
        event_store: EventStore,
        aggregate_id: str,
        event_sequence: List[Dict[str, Any]]
    ) -> List[Event]:
        """Create a sequence of test events."""
        events = []
        
        for i, event_data in enumerate(event_sequence):
            # Create event based on type
            event_type = event_data["event_type"]
            data = event_data["data"]
            
            # event = create_event_by_type(
            #     event_type,
            #     aggregate_id=aggregate_id,
            #     aggregate_version=i + 1,
            #     **data
            # )
            # events.append(event)
        
        return events
    
    @staticmethod
    async def assert_events_in_store(
        event_store: EventStore,
        aggregate_id: str,
        expected_event_types: List[str]
    ):
        """Assert that specific events exist in store."""
        events = await event_store.load_events(aggregate_id)
        actual_types = [event.event_type for event in events]
        
        assert actual_types == expected_event_types, f"Expected {expected_event_types}, got {actual_types}"

# =============================================================================
# CUSTOM PYTEST MARKERS
# =============================================================================

# Add to pytest.ini or conftest.py:
# [tool:pytest]
# markers =
#     unit: Unit tests
#     integration: Integration tests  
#     performance: Performance tests
#     slow: Slow-running tests

# =============================================================================
# MOCK UTILITIES
# =============================================================================

class MockEventStore:
    """Mock EventStore for isolated testing."""
    
    def __init__(self):
        self.saved_aggregates = {}
        self.events = {}
    
    async def save(self, aggregate):
        """Mock save operation."""
        self.saved_aggregates[aggregate.id] = aggregate
        events = aggregate.get_uncommitted_events()
        if aggregate.id not in self.events:
            self.events[aggregate.id] = []
        self.events[aggregate.id].extend(events)
    
    async def load(self, aggregate_class, aggregate_id):
        """Mock load operation."""
        if aggregate_id in self.events:
            # aggregate = aggregate_class(id=aggregate_id)
            # aggregate.replay_events(self.events[aggregate_id])
            # return aggregate
            pass
        return None
    
    async def load_events(self, aggregate_id):
        """Mock load events operation."""
        return self.events.get(aggregate_id, [])

# =============================================================================
# USAGE EXAMPLES
# =============================================================================

"""
# Running tests:

# Run all tests
pytest

# Run specific test categories
pytest -m unit
pytest -m integration
pytest -m performance

# Run with coverage
pytest --cov=your_module --cov-report=html

# Run property-based tests with more examples
pytest --hypothesis-show-statistics

# Run performance tests with timing
pytest -m performance -s -v

# Run specific test file
pytest test_{{aggregate_name_lower}}.py

# Run with parallel execution
pytest -n auto  # Requires pytest-xdist

# Example test execution:
python -m pytest tests/ -v --tb=short --cov=your_domain --cov-report=term-missing
"""