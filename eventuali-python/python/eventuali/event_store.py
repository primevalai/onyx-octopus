"""
High-level Python interface for the event store.
"""

import asyncio
from typing import Optional, List, Type, TypeVar, Union, Dict
from ._eventuali import PyEventStore
from .event import Event
from .aggregate import Aggregate

T = TypeVar('T', bound=Aggregate)


class EventStore:
    """High-performance event store supporting PostgreSQL and SQLite."""
    
    # Class-level event registry to map event types to Python classes
    _event_registry: Dict[str, Type[Event]] = {}
    
    def __init__(self):
        self._inner = PyEventStore()
        self._initialized = False
    
    @classmethod
    async def create(cls, connection_string: str) -> 'EventStore':
        """
        Create and initialize an event store.
        
        Args:
            connection_string: Database connection string
                - PostgreSQL: "postgresql://user:password@host:port/database"
                - SQLite: "sqlite://path/to/database.db" or just "database.db"
        
        Returns:
            Initialized EventStore instance
        
        Examples:
            >>> # SQLite for development
            >>> store = await EventStore.create("sqlite://events.db")
            
            >>> # PostgreSQL for production  
            >>> store = await EventStore.create("postgresql://user:pass@localhost/events")
        """
        store = cls()
        await store._inner.create(connection_string)
        store._initialized = True
        return store
    
    @classmethod
    def register_event_class(cls, event_type: str, event_class: Type[Event]) -> None:
        """
        Register a custom event class for deserialization.
        
        Args:
            event_type: The event type string that identifies this event class
            event_class: The Python class to use for deserializing events of this type
            
        Examples:
            >>> EventStore.register_event_class("AgentEvent", AgentEvent)
            >>> EventStore.register_event_class("agent.simonSays.commandReceived", AgentEvent)
        """
        if not issubclass(event_class, Event):
            raise ValueError(f"event_class must be a subclass of Event, got {event_class}")
        
        cls._event_registry[event_type] = event_class
    
    @classmethod
    def unregister_event_class(cls, event_type: str) -> None:
        """
        Unregister a custom event class.
        
        Args:
            event_type: The event type string to unregister
        """
        cls._event_registry.pop(event_type, None)
    
    @classmethod
    def get_registered_event_classes(cls) -> Dict[str, Type[Event]]:
        """
        Get a copy of all registered event classes.
        
        Returns:
            Dictionary mapping event types to their registered classes
        """
        return cls._event_registry.copy()
    
    def _ensure_initialized(self):
        """Ensure the event store has been initialized."""
        if not self._initialized:
            raise RuntimeError("EventStore not initialized. Use EventStore.create() instead of EventStore()")
    
    def _deserialize_event(self, event_dict: dict) -> Event:
        """
        Deserialize an event dictionary to the appropriate Python Event class.
        
        Args:
            event_dict: Dictionary containing event data from the database
            
        Returns:
            Event instance of the appropriate class
        """
        event_type = event_dict.get('event_type', '')
        
        # 1. First check the event registry for registered custom classes
        if event_type in self._event_registry:
            event_class = self._event_registry[event_type]
            try:
                return event_class.from_dict(event_dict)
            except Exception:
                # If deserialization fails, fall through to other methods
                pass
        
        # 2. Try to find the class in the eventuali.event module (existing behavior)
        try:
            from . import event as event_module
            event_class = getattr(event_module, event_type, None)
            if event_class is not None and issubclass(event_class, Event):
                return event_class.from_dict(event_dict)
        except (ImportError, AttributeError, TypeError):
            pass
        
        # 3. Fall back to base Event class but preserve all fields
        # This ensures data is not lost even if the exact class isn't available
        try:
            return Event.from_dict(event_dict)
        except Exception:
            # Final fallback - create a minimal Event with just metadata
            minimal_data = {
                'event_id': event_dict.get('event_id'),
                'aggregate_id': event_dict.get('aggregate_id'),
                'aggregate_type': event_dict.get('aggregate_type'),
                'event_type': event_dict.get('event_type'),
                'event_version': event_dict.get('event_version', 1),
                'aggregate_version': event_dict.get('aggregate_version'),
                'timestamp': event_dict.get('timestamp'),
                'causation_id': event_dict.get('causation_id'),
                'correlation_id': event_dict.get('correlation_id'),
                'user_id': event_dict.get('user_id'),
            }
            return Event.from_dict(minimal_data)
    
    async def save(self, aggregate: Aggregate) -> None:
        """
        Save an aggregate and its uncommitted events to the event store.
        
        Args:
            aggregate: The aggregate to save
            
        Raises:
            OptimisticConcurrencyError: If the aggregate has been modified by another process
        """
        self._ensure_initialized()
        
        if not aggregate.has_uncommitted_events():
            return  # Nothing to save
        
        # Convert uncommitted events to the format expected by Rust backend
        events = []
        for event in aggregate.get_uncommitted_events():
            # Ensure event has correct aggregate metadata
            event.aggregate_id = aggregate.id
            event.aggregate_type = aggregate.get_aggregate_type()
            event.event_type = event.get_event_type()
            
            # Convert to dict for Rust backend
            event_dict = event.model_dump()
            events.append(event_dict)
        
        try:
            # Save events through Rust backend
            await self._inner.save_events(events)
            
            # Mark events as committed
            aggregate.mark_events_as_committed()
            
        except Exception as e:
            # Check if this is an optimistic concurrency error
            if "OptimisticConcurrency" in str(e):
                from .exceptions import OptimisticConcurrencyError
                raise OptimisticConcurrencyError(
                    f"Aggregate {aggregate.id} has been modified by another process"
                ) from e
            raise
    
    async def load(self, aggregate_class: Type[T], aggregate_id: str) -> Optional[T]:
        """
        Load an aggregate from the event store by ID.
        
        Args:
            aggregate_class: The aggregate class to instantiate
            aggregate_id: The unique identifier of the aggregate
            
        Returns:
            The loaded aggregate, or None if not found
        """
        self._ensure_initialized()
        
        # Load events from Rust backend
        rust_events = await self._inner.load_events(aggregate_id)
        if not rust_events:
            return None
        
        # Convert Rust events back to Python events
        events = []
        for rust_event in rust_events:
            # Convert the Rust event back to Python Event
            event_dict = rust_event.to_dict()
            
            # Use the new deserialization helper
            python_event = self._deserialize_event(event_dict)
            events.append(python_event)
        
        # Reconstruct aggregate from events
        try:
            return aggregate_class.from_events(events)
        except ValueError:
            return None
    
    async def load_events(
        self, 
        aggregate_id: str, 
        from_version: Optional[int] = None
    ) -> List[Event]:
        """
        Load events for a specific aggregate.
        
        Args:
            aggregate_id: The aggregate identifier
            from_version: Optional version to start loading from
            
        Returns:
            List of events ordered by version
        """
        self._ensure_initialized()
        
        # Load events from Rust backend
        rust_events = await self._inner.load_events(aggregate_id, from_version)
        
        # Convert Rust events back to Python events
        events = []
        for rust_event in rust_events:
            event_dict = rust_event.to_dict()
            
            # Use the new deserialization helper
            python_event = self._deserialize_event(event_dict)
            events.append(python_event)
        
        return events
    
    async def load_events_by_type(
        self,
        aggregate_type: str,
        from_version: Optional[int] = None
    ) -> List[Event]:
        """
        Load all events for a specific aggregate type.
        
        Args:
            aggregate_type: The type of aggregate
            from_version: Optional version to start loading from
            
        Returns:
            List of events ordered by timestamp
        """
        self._ensure_initialized()
        
        # Load events from Rust backend
        rust_events = await self._inner.load_events_by_type(aggregate_type, from_version)
        
        # Convert Rust events back to Python events
        events = []
        for rust_event in rust_events:
            event_dict = rust_event.to_dict()
            
            # Use the new deserialization helper
            python_event = self._deserialize_event(event_dict)
            events.append(python_event)
        
        return events
    
    async def get_aggregate_version(self, aggregate_id: str) -> Optional[int]:
        """
        Get the current version of an aggregate.
        
        Args:
            aggregate_id: The aggregate identifier
            
        Returns:
            The current version, or None if aggregate doesn't exist
        """
        self._ensure_initialized()
        return await self._inner.get_aggregate_version(aggregate_id)