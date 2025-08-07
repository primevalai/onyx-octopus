"""
High-level Python interface for the event store.
"""

import asyncio
from typing import Optional, List, Type, TypeVar, Union
from ._eventuali import PyEventStore
from .event import Event
from .aggregate import Aggregate

T = TypeVar('T', bound=Aggregate)


class EventStore:
    """High-performance event store supporting PostgreSQL and SQLite."""
    
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
    
    def _ensure_initialized(self):
        """Ensure the event store has been initialized."""
        if not self._initialized:
            raise RuntimeError("EventStore not initialized. Use EventStore.create() instead of EventStore()")
    
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
            
            # Dynamically import the event class based on event_type
            event_type = event_dict['event_type']
            try:
                # Try to import from the event module
                from . import event as event_module
                event_class = getattr(event_module, event_type, None)
                if event_class is None:
                    # Fall back to generic Event
                    event_class = Event
                
                # Create Python event from dict
                python_event = event_class.from_dict(event_dict)
                events.append(python_event)
                
            except (ImportError, AttributeError):
                # Fall back to generic Event
                python_event = Event.from_dict(event_dict)
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
            
            # Dynamically import the event class based on event_type
            event_type = event_dict['event_type']
            try:
                from . import event as event_module
                event_class = getattr(event_module, event_type, Event)
                python_event = event_class.from_dict(event_dict)
                events.append(python_event)
            except (ImportError, AttributeError):
                python_event = Event.from_dict(event_dict)
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
            
            # Dynamically import the event class based on event_type
            event_type = event_dict['event_type']
            try:
                from . import event as event_module
                event_class = getattr(event_module, event_type, Event)
                python_event = event_class.from_dict(event_dict)
                events.append(python_event)
            except (ImportError, AttributeError):
                python_event = Event.from_dict(event_dict)
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