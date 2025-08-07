"""
Event streaming and subscription functionality for Eventuali.

This module provides high-performance event streaming capabilities that allow
applications to subscribe to event streams and build projections, sagas, and
other event-driven components.
"""

import asyncio
from typing import Optional, Dict, Any, Callable, AsyncIterator
from datetime import datetime

from ._eventuali import PyEventStreamer, PyEventStreamReceiver, PySubscriptionBuilder, PyProjection
from .event import Event


class EventStreamer:
    """
    High-performance event streamer for publishing and subscribing to events.
    
    The EventStreamer provides real-time event streaming capabilities using
    Rust's high-performance broadcast channels under the hood.
    """
    
    def __init__(self, capacity: int = 1000):
        """
        Initialize the event streamer.
        
        Args:
            capacity: Maximum number of events to buffer in memory (default: 1000)
        """
        self._streamer = PyEventStreamer(capacity)
    
    async def subscribe(self, subscription: 'Subscription') -> 'EventStreamReceiver':
        """
        Subscribe to events matching the given criteria.
        
        Args:
            subscription: Subscription configuration defining which events to receive
            
        Returns:
            EventStreamReceiver for consuming events
        """
        receiver = await self._streamer.subscribe(subscription.to_dict())
        return EventStreamReceiver(receiver)
    
    async def unsubscribe(self, subscription_id: str) -> None:
        """
        Unsubscribe from an event stream.
        
        Args:
            subscription_id: ID of the subscription to remove
        """
        await self._streamer.unsubscribe(subscription_id)
    
    async def get_stream_position(self, stream_id: str) -> Optional[int]:
        """
        Get the current position for a specific stream.
        
        Args:
            stream_id: ID of the stream
            
        Returns:
            Current stream position, or None if stream doesn't exist
        """
        return await self._streamer.get_stream_position(stream_id)
    
    async def get_global_position(self) -> int:
        """
        Get the current global position across all streams.
        
        Returns:
            Current global event position
        """
        return await self._streamer.get_global_position()
    
    async def publish_event(self, event: Event, stream_position: int, global_position: int) -> None:
        """
        Publish an event to the stream with position information.
        
        Args:
            event: Event to publish
            stream_position: Position in the event stream
            global_position: Global position across all streams
        """
        await self._streamer.publish_event(event._inner, stream_position, global_position)


class EventStreamReceiver:
    """
    Receiver for consuming events from a subscription.
    """
    
    def __init__(self, receiver: PyEventStreamReceiver):
        self._receiver = receiver
    
    async def recv(self) -> 'StreamEvent':
        """
        Receive the next event from the stream.
        
        Returns:
            StreamEvent containing the event and position information
            
        Raises:
            RuntimeError: If the channel is closed or no more events are available
        """
        event_data = await self._receiver.recv()
        return StreamEvent(
            event=event_data['event'],
            stream_position=event_data['stream_position'],
            global_position=event_data['global_position']
        )
    
    async def __aiter__(self) -> AsyncIterator['StreamEvent']:
        """
        Iterate over events in the stream.
        """
        try:
            while True:
                yield await self.recv()
        except RuntimeError:
            # Channel closed, stop iteration
            return


class StreamEvent:
    """
    Event wrapper containing position information for streaming.
    """
    
    def __init__(self, event: Event, stream_position: int, global_position: int):
        self.event = event
        self.stream_position = stream_position
        self.global_position = global_position
    
    def __repr__(self) -> str:
        return f"StreamEvent(event={self.event!r}, stream_pos={self.stream_position}, global_pos={self.global_position})"


class Subscription:
    """
    Subscription configuration for filtering events.
    """
    
    def __init__(
        self,
        id: Optional[str] = None,
        aggregate_type_filter: Optional[str] = None,
        event_type_filter: Optional[str] = None,
        from_timestamp: Optional[datetime] = None
    ):
        """
        Create a new subscription.
        
        Args:
            id: Optional subscription ID (auto-generated if not provided)
            aggregate_type_filter: Only receive events from aggregates of this type
            event_type_filter: Only receive events of this type
            from_timestamp: Only receive events from this timestamp onwards
        """
        self.id = id
        self.aggregate_type_filter = aggregate_type_filter
        self.event_type_filter = event_type_filter
        self.from_timestamp = from_timestamp
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert subscription to dictionary for Rust interop."""
        result = {}
        if self.id:
            result['id'] = self.id
        if self.aggregate_type_filter:
            result['aggregate_type_filter'] = self.aggregate_type_filter
        if self.event_type_filter:
            result['event_type_filter'] = self.event_type_filter
        if self.from_timestamp:
            result['from_timestamp'] = self.from_timestamp.isoformat()
        return result


class SubscriptionBuilder:
    """
    Builder for creating subscriptions with a fluent interface.
    """
    
    def __init__(self):
        self._builder = PySubscriptionBuilder()
    
    def with_id(self, id: str) -> 'SubscriptionBuilder':
        """Set the subscription ID."""
        self._builder.with_id(id)
        return self
    
    def filter_by_aggregate_type(self, aggregate_type: str) -> 'SubscriptionBuilder':
        """Filter events by aggregate type."""
        self._builder.filter_by_aggregate_type(aggregate_type)
        return self
    
    def filter_by_event_type(self, event_type: str) -> 'SubscriptionBuilder':
        """Filter events by event type."""
        self._builder.filter_by_event_type(event_type)
        return self
    
    def from_timestamp(self, timestamp: datetime) -> 'SubscriptionBuilder':
        """Only receive events from this timestamp onwards."""
        # For now, just store this - full implementation would handle timestamps
        return self
    
    def build(self) -> Subscription:
        """Build the subscription."""
        sub_dict = self._builder.build()
        return Subscription(
            id=sub_dict.get('id'),
            aggregate_type_filter=sub_dict.get('aggregate_type_filter'),
            event_type_filter=sub_dict.get('event_type_filter'),
            from_timestamp=None  # Simplified for now
        )


class Projection:
    """
    Base class for building read models from event streams.
    
    Projections allow you to build optimized read models by processing
    events as they arrive, maintaining eventual consistency with the
    event store.
    """
    
    async def handle_event(self, event: Event) -> None:
        """
        Handle an event and update the projection.
        
        Args:
            event: The event to process
        """
        raise NotImplementedError("Subclasses must implement handle_event")
    
    async def reset(self) -> None:
        """
        Reset the projection to its initial state.
        """
        raise NotImplementedError("Subclasses must implement reset")
    
    async def get_last_processed_position(self) -> Optional[int]:
        """
        Get the last processed event position.
        
        Returns:
            Last processed position, or None if no events have been processed
        """
        raise NotImplementedError("Subclasses must implement get_last_processed_position")
    
    async def set_last_processed_position(self, position: int) -> None:
        """
        Set the last processed event position.
        
        Args:
            position: Position to set
        """
        raise NotImplementedError("Subclasses must implement set_last_processed_position")


class SagaHandler:
    """
    Base class for handling events in long-running workflows (sagas).
    
    Sagas coordinate complex business processes that span multiple
    aggregates by reacting to events and issuing commands.
    """
    
    async def handle_event(self, event: Event) -> None:
        """
        Handle an event in the saga.
        
        Args:
            event: The event to process
        """
        raise NotImplementedError("Subclasses must implement handle_event")