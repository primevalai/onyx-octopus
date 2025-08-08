"""
Eventuali - High-performance event sourcing for Python, powered by Rust.

This package provides a high-performance event sourcing library that combines
the performance and memory safety of Rust with the ease of use of Python.
"""

from ._eventuali import (
    PyEventStore as _PyEventStore, 
    PyEvent as _PyEvent, 
    PyAggregate as _PyAggregate,
    SnapshotService as _PySnapshotService,
    SnapshotConfig as _PySnapshotConfig,
    AggregateSnapshot as _PyAggregateSnapshot
)
from .event_store import EventStore
from .event import Event  
from .aggregate import Aggregate
from .streaming import (
    EventStreamer, EventStreamReceiver, StreamEvent, Subscription,
    SubscriptionBuilder, Projection, SagaHandler
)
from .snapshot import SnapshotService, SnapshotConfig, AggregateSnapshot
from .exceptions import *

__version__ = "0.1.0"
__author__ = "PrimevalAI"
__email__ = "info@primeval.ai"

__all__ = [
    "EventStore",
    "Event", 
    "Aggregate",
    # Streaming
    "EventStreamer",
    "EventStreamReceiver",
    "StreamEvent",
    "Subscription",
    "SubscriptionBuilder",
    "Projection",
    "SagaHandler",
    # Snapshots
    "SnapshotService",
    "SnapshotConfig",
    "AggregateSnapshot",
    # Exceptions
    "EventualiError",
    "OptimisticConcurrencyError",
    "EventStoreError",
    "SerializationError",
    "AggregateNotFoundError",
    "InvalidEventError",
    "DatabaseError",
    "ConfigurationError",
    "ProjectionError",
    "SnapshotError",
    "StreamingError",
]