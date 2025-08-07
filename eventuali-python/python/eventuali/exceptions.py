"""
Custom exceptions for Eventuali.
"""


class EventualiError(Exception):
    """Base exception for all Eventuali errors."""
    pass


class OptimisticConcurrencyError(EventualiError):
    """
    Raised when an optimistic concurrency conflict is detected.
    
    This happens when trying to save an aggregate that has been
    modified by another process since it was loaded.
    """
    
    def __init__(self, message: str, expected_version: int = None, actual_version: int = None):
        super().__init__(message)
        self.expected_version = expected_version
        self.actual_version = actual_version


class EventStoreError(EventualiError):
    """Raised when there's an error with the event store."""
    pass


class SerializationError(EventualiError):
    """Raised when there's an error serializing or deserializing events."""
    pass


class AggregateNotFoundError(EventualiError):
    """Raised when trying to load an aggregate that doesn't exist."""
    
    def __init__(self, aggregate_id: str, aggregate_type: str = None):
        self.aggregate_id = aggregate_id
        self.aggregate_type = aggregate_type
        
        message = f"Aggregate '{aggregate_id}' not found"
        if aggregate_type:
            message += f" (type: {aggregate_type})"
        
        super().__init__(message)


class InvalidEventError(EventualiError):
    """Raised when an event is invalid or malformed."""
    pass


class DatabaseError(EventualiError):
    """Raised when there's a database-related error."""
    pass


class ConfigurationError(EventualiError):
    """Raised when there's a configuration error."""
    pass


class ProjectionError(EventualiError):
    """Raised when there's an error with projections."""
    pass


class SnapshotError(EventualiError):
    """Raised when there's an error with snapshots."""
    pass


class StreamingError(EventualiError):
    """Raised when there's an error with event streaming."""
    pass