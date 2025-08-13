"""
Event base classes and utilities.
"""

import json
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any, Dict, Optional, Type, TypeVar
from pydantic import BaseModel, Field
from uuid import UUID, uuid4

T = TypeVar('T', bound='Event')


class Event(BaseModel, ABC):
    """
    Base class for all domain events.
    
    Events are immutable records of something that happened in the domain.
    They should be named in the past tense (e.g., UserRegistered, OrderPlaced).
    """
    
    # These fields will be set automatically by the event store
    event_id: Optional[UUID] = Field(default_factory=uuid4, description="Unique event identifier")
    aggregate_id: Optional[str] = Field(default=None, description="ID of the aggregate that generated this event")
    aggregate_type: Optional[str] = Field(default=None, description="Type of the aggregate")
    event_type: Optional[str] = Field(default=None, description="Type of the event")
    event_version: Optional[int] = Field(default=1, description="Schema version of the event")
    aggregate_version: Optional[int] = Field(default=None, description="Version of the aggregate after this event")
    timestamp: Optional[datetime] = Field(default_factory=datetime.utcnow, description="When the event occurred")
    
    # Metadata for correlation and causation
    causation_id: Optional[UUID] = Field(default=None, description="ID of the event that caused this event")
    correlation_id: Optional[UUID] = Field(default=None, description="ID correlating related events")
    user_id: Optional[str] = Field(default=None, description="ID of the user who triggered this event")
    
    model_config = {
        "frozen": False,  # Allow modification for event store metadata
        "use_enum_values": True,
        "extra": "allow",  # Allow extra fields to be preserved during deserialization
        "json_encoders": {
            datetime: lambda v: v.isoformat(),
            UUID: lambda v: str(v),
        }
    }
    
    @classmethod
    def get_event_type(cls) -> str:
        """Get the event type name."""
        return cls.__name__
    
    def to_json(self) -> str:
        """Convert event to JSON string."""
        return self.model_dump_json()
    
    @classmethod
    def from_json(cls: Type[T], json_str: str) -> T:
        """Create event from JSON string."""
        return cls.model_validate_json(json_str)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert event to dictionary."""
        return self.model_dump()
    
    @classmethod
    def from_dict(cls: Type[T], data: Dict[str, Any]) -> T:
        """Create event from dictionary."""
        return cls.model_validate(data)


class DomainEvent(Event):
    """
    Base class for domain events that are part of the business logic.
    
    Domain events represent meaningful business occurrences and are typically
    the result of executing a command on an aggregate.
    """
    pass


class SystemEvent(Event):
    """
    Base class for system events that are not part of the core business logic.
    
    System events represent technical occurrences like aggregate snapshots,
    migrations, or other infrastructure concerns.
    """
    pass


# Example domain events for demonstration
class UserRegistered(DomainEvent):
    """Event fired when a user registers."""
    name: str
    email: str
    

class UserEmailChanged(DomainEvent):
    """Event fired when a user changes their email."""
    old_email: str
    new_email: str


class UserDeactivated(DomainEvent):
    """Event fired when a user account is deactivated."""
    reason: Optional[str] = None