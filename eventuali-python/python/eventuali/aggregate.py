"""
Aggregate base classes and utilities.
"""

from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional, Type, TypeVar, get_type_hints
from pydantic import BaseModel, Field
from uuid import UUID, uuid4
from .event import Event

T = TypeVar('T', bound='Aggregate')


class Aggregate(BaseModel, ABC):
    """
    Base class for all aggregates in the domain.
    
    An aggregate is a cluster of domain objects that can be treated as a single unit.
    It maintains business invariants and generates events when its state changes.
    """
    
    # Core aggregate fields
    id: str = Field(default_factory=lambda: str(uuid4()), description="Unique aggregate identifier")
    version: int = Field(default=0, description="Current version of the aggregate")
    
    # Private fields for event sourcing
    uncommitted_events: List[Event] = Field(default_factory=list, exclude=True, alias='_uncommitted_events')
    is_new_flag: bool = Field(default=True, exclude=True, alias='_is_new')
    
    model_config = {
        "validate_assignment": True,
        "use_enum_values": True,
        "json_encoders": {
            UUID: lambda v: str(v),
        }
    }
    
    @classmethod
    def get_aggregate_type(cls) -> str:
        """Get the aggregate type name."""
        return cls.__name__
    
    def mark_events_as_committed(self) -> None:
        """Mark all uncommitted events as committed."""
        self.uncommitted_events.clear()
        self.is_new_flag = False
    
    def get_uncommitted_events(self) -> List[Event]:
        """Get all uncommitted events."""
        return self.uncommitted_events.copy()
    
    def has_uncommitted_events(self) -> bool:
        """Check if the aggregate has uncommitted events."""
        return len(self.uncommitted_events) > 0
    
    def is_new(self) -> bool:
        """Check if this is a new aggregate (never persisted)."""
        return self.is_new_flag and self.version == 0
    
    def _apply_event(self, event: Event) -> None:
        """
        Apply an event to the aggregate without adding it to uncommitted events.
        
        This is used when loading from the event store.
        """
        # Set event metadata
        event.aggregate_id = self.id
        event.aggregate_type = self.get_aggregate_type()
        event.event_type = event.get_event_type()
        event.aggregate_version = self.version + 1
        
        # Find and call the appropriate apply method
        method_name = f"apply_{self._get_method_name(event.get_event_type())}"
        if hasattr(self, method_name):
            getattr(self, method_name)(event)
            self.version += 1
        else:
            raise NotImplementedError(
                f"No apply method found for event {event.get_event_type()}. "
                f"Expected method: {method_name}"
            )
    
    def apply(self, event: Event) -> None:
        """
        Apply an event to the aggregate and add it to uncommitted events.
        
        This is used when generating new events.
        """
        self._apply_event(event)
        self.uncommitted_events.append(event)
    
    def _get_method_name(self, event_type: str) -> str:
        """Convert event type to method name (e.g., UserRegistered -> user_registered)."""
        # Convert PascalCase to snake_case
        result = []
        for i, char in enumerate(event_type):
            if char.isupper() and i > 0:
                result.append('_')
            result.append(char.lower())
        return ''.join(result)
    
    @classmethod
    def from_events(cls: Type[T], events: List[Event]) -> T:
        """
        Create an aggregate by replaying events.
        
        Args:
            events: List of events to replay, ordered by version
            
        Returns:
            Aggregate with state reconstructed from events
        """
        if not events:
            raise ValueError("Cannot create aggregate from empty event list")
        
        # Get aggregate ID from first event
        aggregate_id = events[0].aggregate_id
        if not aggregate_id:
            raise ValueError("First event must have aggregate_id set")
        
        # Create new aggregate instance
        aggregate = cls(id=aggregate_id, version=0)
        aggregate._is_new = False
        
        # Apply all events
        for event in events:
            aggregate._apply_event(event)
        
        return aggregate
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert aggregate to dictionary (excluding private fields)."""
        return self.model_dump(exclude={'uncommitted_events', 'is_new_flag'})
    
    @classmethod
    def from_dict(cls: Type[T], data: Dict[str, Any]) -> T:
        """Create aggregate from dictionary."""
        aggregate = cls.model_validate(data)
        aggregate.is_new_flag = False
        return aggregate


# Example aggregate for demonstration
class User(Aggregate):
    """Example user aggregate."""
    
    name: str = ""
    email: str = ""
    is_active: bool = True
    
    def apply_user_registered(self, event: 'UserRegistered') -> None:
        """Apply UserRegistered event."""
        self.name = event.name
        self.email = event.email
        self.is_active = True
    
    def apply_user_email_changed(self, event: 'UserEmailChanged') -> None:
        """Apply UserEmailChanged event."""
        self.email = event.new_email
    
    def apply_user_deactivated(self, event: 'UserDeactivated') -> None:
        """Apply UserDeactivated event."""
        self.is_active = False
    
    # Business methods
    def change_email(self, new_email: str) -> None:
        """Change user's email address."""
        if not new_email or '@' not in new_email:
            raise ValueError("Invalid email address")
        
        if new_email == self.email:
            return  # No change needed
        
        from .event import UserEmailChanged
        event = UserEmailChanged(old_email=self.email, new_email=new_email)
        self.apply(event)
    
    def deactivate(self, reason: Optional[str] = None) -> None:
        """Deactivate user account."""
        if not self.is_active:
            return  # Already deactivated
        
        from .event import UserDeactivated
        event = UserDeactivated(reason=reason)
        self.apply(event)