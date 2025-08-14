"""
Advanced Aggregate Template for Eventuali Event Sourcing

This template provides a comprehensive foundation for building domain aggregates
with event sourcing patterns, validation, and business logic.

Usage:
1. Replace {{AGGREGATE_NAME}} with your aggregate name (e.g., User, Order, Product)
2. Replace {{AGGREGATE_ID}} with your aggregate identifier (e.g., user_id, order_id)
3. Add your domain-specific events and business methods
4. Implement event handlers following the apply_{{event_name_lower}} pattern

Performance: 79k+ events/sec application, 18.3x faster than pure Python
"""

from eventuali import Aggregate, Event
from typing import Optional, List, Dict, Any, Union
from datetime import datetime, timedelta
from decimal import Decimal
from pydantic import Field, validator
from enum import Enum
import logging

logger = logging.getLogger(__name__)

# =============================================================================
# DOMAIN EVENTS
# =============================================================================

class {{AGGREGATE_NAME}}Status(str, Enum):
    """Status enumeration for {{AGGREGATE_NAME}}."""
    DRAFT = "draft"
    ACTIVE = "active"
    SUSPENDED = "suspended"
    ARCHIVED = "archived"

class {{AGGREGATE_NAME}}Created(Event):
    """{{AGGREGATE_NAME}} creation event."""
    name: str = Field(..., min_length=1, max_length=200)
    description: Optional[str] = Field(None, max_length=1000)
    created_by: str = Field(..., description="User ID who created this {{AGGREGATE_NAME_LOWER}}")
    
    @validator('name')
    def validate_name(cls, v):
        if not v or v.strip() != v:
            raise ValueError("Name cannot be empty or contain leading/trailing whitespace")
        return v

class {{AGGREGATE_NAME}}Updated(Event):
    """{{AGGREGATE_NAME}} update event."""
    name: Optional[str] = Field(None, min_length=1, max_length=200)
    description: Optional[str] = Field(None, max_length=1000)
    updated_by: str = Field(..., description="User ID who updated this {{AGGREGATE_NAME_LOWER}}")
    changes: Dict[str, Any] = Field(default_factory=dict, description="Changed fields")

class {{AGGREGATE_NAME}}StatusChanged(Event):
    """{{AGGREGATE_NAME}} status change event."""
    old_status: {{AGGREGATE_NAME}}Status
    new_status: {{AGGREGATE_NAME}}Status
    reason: str = Field(..., min_length=1, max_length=500)
    changed_by: str = Field(..., description="User ID who changed the status")

class {{AGGREGATE_NAME}}Archived(Event):
    """{{AGGREGATE_NAME}} archival event."""
    reason: str = Field(..., min_length=1, max_length=500)
    archived_by: str = Field(..., description="User ID who archived this {{AGGREGATE_NAME_LOWER}}")
    retention_date: Optional[datetime] = Field(None, description="When this can be permanently deleted")

# =============================================================================
# BUSINESS EXCEPTIONS
# =============================================================================

class {{AGGREGATE_NAME}}Error(Exception):
    """Base exception for {{AGGREGATE_NAME}} business logic errors."""
    pass

class {{AGGREGATE_NAME}}NotFoundError({{AGGREGATE_NAME}}Error):
    """{{AGGREGATE_NAME}} not found error."""
    pass

class {{AGGREGATE_NAME}}InvalidStateError({{AGGREGATE_NAME}}Error):
    """{{AGGREGATE_NAME}} invalid state error."""
    pass

class {{AGGREGATE_NAME}}ValidationError({{AGGREGATE_NAME}}Error):
    """{{AGGREGATE_NAME}} validation error."""
    pass

# =============================================================================
# AGGREGATE ROOT
# =============================================================================

class {{AGGREGATE_NAME}}(Aggregate):
    """
    {{AGGREGATE_NAME}} aggregate root managing {{AGGREGATE_NAME_LOWER}} lifecycle.
    
    This aggregate encapsulates all business logic related to {{AGGREGATE_NAME_LOWER}} management,
    ensuring consistency and enforcing business rules through domain events.
    
    Performance Characteristics:
    - Event Application: 79k+ events/sec
    - State Reconstruction: 18.3x faster than pure Python
    - Memory Efficiency: 8-20x lower usage vs pure Python
    
    Business Rules:
    - {{AGGREGATE_NAME}} must have a non-empty name
    - Status transitions follow defined workflow
    - Only active {{AGGREGATE_NAME_LOWER}}s can be updated
    - Archived {{AGGREGATE_NAME_LOWER}}s cannot be modified
    """
    
    def __init__(self, id: str, version: int = 0):
        """
        Initialize {{AGGREGATE_NAME}} aggregate.
        
        Args:
            id: Unique {{AGGREGATE_NAME_LOWER}} identifier
            version: Current aggregate version (default: 0 for new aggregates)
        """
        super().__init__(id, version)
        
        # Core properties
        self.name: Optional[str] = None
        self.description: Optional[str] = None
        self.status: {{AGGREGATE_NAME}}Status = {{AGGREGATE_NAME}}Status.DRAFT
        
        # Metadata
        self.created_at: Optional[datetime] = None
        self.created_by: Optional[str] = None
        self.updated_at: Optional[datetime] = None
        self.updated_by: Optional[str] = None
        self.archived_at: Optional[datetime] = None
        self.archived_by: Optional[str] = None
        
        # Business-specific properties (add your domain fields here)
        self.custom_field_1: Optional[str] = None
        self.custom_field_2: Optional[Decimal] = None
        self.custom_metadata: Dict[str, Any] = {}
        
        # Computed properties
        self._last_activity: Optional[datetime] = None
        self._change_count: int = 0
    
    # =========================================================================
    # BUSINESS METHODS
    # =========================================================================
    
    def create(
        self, 
        name: str, 
        created_by: str, 
        description: str = None,
        **custom_fields
    ) -> None:
        """
        Create a new {{AGGREGATE_NAME_LOWER}}.
        
        Args:
            name: {{AGGREGATE_NAME}} name (required, 1-200 characters)
            created_by: User ID creating this {{AGGREGATE_NAME_LOWER}}
            description: Optional description (max 1000 characters)
            **custom_fields: Additional domain-specific fields
            
        Raises:
            {{AGGREGATE_NAME}}ValidationError: If validation fails
            {{AGGREGATE_NAME}}InvalidStateError: If {{AGGREGATE_NAME_LOWER}} already exists
        """
        if self.version > 0:
            raise {{AGGREGATE_NAME}}InvalidStateError(
                f"{{AGGREGATE_NAME}} {self.id} already exists (version {self.version})"
            )
        
        # Validation
        self._validate_name(name)
        self._validate_user_id(created_by)
        if description:
            self._validate_description(description)
        
        # Create event
        event = {{AGGREGATE_NAME}}Created(
            name=name.strip(),
            description=description.strip() if description else None,
            created_by=created_by,
            **custom_fields
        )
        
        self.apply(event)
        logger.info(f"{{AGGREGATE_NAME}} {self.id} created by {created_by}")
    
    def update(
        self, 
        updated_by: str,
        name: str = None,
        description: str = None,
        **custom_fields
    ) -> None:
        """
        Update {{AGGREGATE_NAME_LOWER}} properties.
        
        Args:
            updated_by: User ID making the update
            name: New name (optional)
            description: New description (optional)
            **custom_fields: Additional domain-specific fields to update
            
        Raises:
            {{AGGREGATE_NAME}}ValidationError: If validation fails
            {{AGGREGATE_NAME}}InvalidStateError: If {{AGGREGATE_NAME_LOWER}} cannot be updated
        """
        self._ensure_exists()
        self._ensure_can_modify()
        self._validate_user_id(updated_by)
        
        # Build changes dictionary
        changes = {}
        
        if name is not None:
            self._validate_name(name)
            name = name.strip()
            if name != self.name:
                changes['name'] = {'old': self.name, 'new': name}
        
        if description is not None:
            self._validate_description(description)
            description = description.strip() if description else None
            if description != self.description:
                changes['description'] = {'old': self.description, 'new': description}
        
        # Add custom field changes
        for field, value in custom_fields.items():
            current_value = getattr(self, field, None)
            if value != current_value:
                changes[field] = {'old': current_value, 'new': value}
        
        # Only apply event if there are actual changes
        if not changes:
            logger.debug(f"No changes detected for {{AGGREGATE_NAME}} {self.id}")
            return
        
        event = {{AGGREGATE_NAME}}Updated(
            name=name,
            description=description,
            updated_by=updated_by,
            changes=changes,
            **custom_fields
        )
        
        self.apply(event)
        logger.info(f"{{AGGREGATE_NAME}} {self.id} updated by {updated_by}: {list(changes.keys())}")
    
    def change_status(
        self, 
        new_status: {{AGGREGATE_NAME}}Status, 
        reason: str, 
        changed_by: str
    ) -> None:
        """
        Change {{AGGREGATE_NAME_LOWER}} status.
        
        Args:
            new_status: New status to transition to
            reason: Reason for status change (required, 1-500 characters)
            changed_by: User ID making the change
            
        Raises:
            {{AGGREGATE_NAME}}ValidationError: If validation fails
            {{AGGREGATE_NAME}}InvalidStateError: If status transition is invalid
        """
        self._ensure_exists()
        self._validate_user_id(changed_by)
        
        if not reason or len(reason.strip()) == 0:
            raise {{AGGREGATE_NAME}}ValidationError("Status change reason is required")
        
        if len(reason) > 500:
            raise {{AGGREGATE_NAME}}ValidationError("Status change reason too long (max 500 characters)")
        
        if new_status == self.status:
            raise {{AGGREGATE_NAME}}InvalidStateError(
                f"{{AGGREGATE_NAME}} {self.id} already has status {new_status}"
            )
        
        # Validate status transition
        if not self._is_valid_status_transition(self.status, new_status):
            raise {{AGGREGATE_NAME}}InvalidStateError(
                f"Cannot transition from {self.status} to {new_status}"
            )
        
        event = {{AGGREGATE_NAME}}StatusChanged(
            old_status=self.status,
            new_status=new_status,
            reason=reason.strip(),
            changed_by=changed_by
        )
        
        self.apply(event)
        logger.info(f"{{AGGREGATE_NAME}} {self.id} status changed: {self.status} -> {new_status}")
    
    def archive(self, reason: str, archived_by: str, retention_days: int = 2555) -> None:
        """
        Archive the {{AGGREGATE_NAME_LOWER}}.
        
        Args:
            reason: Reason for archival (required, 1-500 characters)
            archived_by: User ID performing the archival
            retention_days: Days to retain before permanent deletion (default: 7 years)
            
        Raises:
            {{AGGREGATE_NAME}}ValidationError: If validation fails
            {{AGGREGATE_NAME}}InvalidStateError: If {{AGGREGATE_NAME_LOWER}} cannot be archived
        """
        self._ensure_exists()
        self._validate_user_id(archived_by)
        
        if self.status == {{AGGREGATE_NAME}}Status.ARCHIVED:
            raise {{AGGREGATE_NAME}}InvalidStateError(
                f"{{AGGREGATE_NAME}} {self.id} is already archived"
            )
        
        if not reason or len(reason.strip()) == 0:
            raise {{AGGREGATE_NAME}}ValidationError("Archive reason is required")
        
        if len(reason) > 500:
            raise {{AGGREGATE_NAME}}ValidationError("Archive reason too long (max 500 characters)")
        
        retention_date = datetime.now() + timedelta(days=retention_days)
        
        event = {{AGGREGATE_NAME}}Archived(
            reason=reason.strip(),
            archived_by=archived_by,
            retention_date=retention_date
        )
        
        self.apply(event)
        logger.info(f"{{AGGREGATE_NAME}} {self.id} archived by {archived_by}")
    
    # =========================================================================
    # EVENT HANDLERS
    # =========================================================================
    
    def apply_{{aggregate_name_lower}}created(self, event: {{AGGREGATE_NAME}}Created) -> None:
        """Apply {{AGGREGATE_NAME}} creation event."""
        self.name = event.name
        self.description = event.description
        self.status = {{AGGREGATE_NAME}}Status.ACTIVE  # Auto-activate on creation
        self.created_at = event.timestamp
        self.created_by = event.created_by
        self.updated_at = event.timestamp
        self.updated_by = event.created_by
        
        # Apply custom fields
        for key, value in event.to_dict().items():
            if key.startswith('custom_') and hasattr(self, key):
                setattr(self, key, value)
        
        self._update_activity()
    
    def apply_{{aggregate_name_lower}}updated(self, event: {{AGGREGATE_NAME}}Updated) -> None:
        """Apply {{AGGREGATE_NAME}} update event."""
        if event.name is not None:
            self.name = event.name
        
        if event.description is not None:
            self.description = event.description
        
        self.updated_at = event.timestamp
        self.updated_by = event.updated_by
        self._change_count += 1
        
        # Apply custom field updates
        event_data = event.to_dict()
        for key, value in event_data.items():
            if key.startswith('custom_') and hasattr(self, key):
                setattr(self, key, value)
        
        self._update_activity()
    
    def apply_{{aggregate_name_lower}}statuschanged(self, event: {{AGGREGATE_NAME}}StatusChanged) -> None:
        """Apply {{AGGREGATE_NAME}} status change event."""
        self.status = event.new_status
        self.updated_at = event.timestamp
        self.updated_by = event.changed_by
        
        self._update_activity()
    
    def apply_{{aggregate_name_lower}}archived(self, event: {{AGGREGATE_NAME}}Archived) -> None:
        """Apply {{AGGREGATE_NAME}} archival event."""
        self.status = {{AGGREGATE_NAME}}Status.ARCHIVED
        self.archived_at = event.timestamp
        self.archived_by = event.archived_by
        self.updated_at = event.timestamp
        self.updated_by = event.archived_by
        
        self._update_activity()
    
    # =========================================================================
    # VALIDATION METHODS
    # =========================================================================
    
    def _validate_name(self, name: str) -> None:
        """Validate {{AGGREGATE_NAME_LOWER}} name."""
        if not name or not isinstance(name, str):
            raise {{AGGREGATE_NAME}}ValidationError("Name is required")
        
        name = name.strip()
        if len(name) == 0:
            raise {{AGGREGATE_NAME}}ValidationError("Name cannot be empty")
        
        if len(name) > 200:
            raise {{AGGREGATE_NAME}}ValidationError("Name too long (max 200 characters)")
        
        # Add domain-specific name validation here
        # Example: forbidden characters, reserved names, etc.
    
    def _validate_description(self, description: str) -> None:
        """Validate {{AGGREGATE_NAME_LOWER}} description."""
        if description and len(description) > 1000:
            raise {{AGGREGATE_NAME}}ValidationError("Description too long (max 1000 characters)")
    
    def _validate_user_id(self, user_id: str) -> None:
        """Validate user ID format."""
        if not user_id or not isinstance(user_id, str) or len(user_id.strip()) == 0:
            raise {{AGGREGATE_NAME}}ValidationError("User ID is required")
        
        # Add domain-specific user ID validation here
        # Example: UUID format, length constraints, etc.
    
    def _is_valid_status_transition(
        self, 
        from_status: {{AGGREGATE_NAME}}Status, 
        to_status: {{AGGREGATE_NAME}}Status
    ) -> bool:
        """Check if status transition is valid."""
        # Define allowed status transitions
        allowed_transitions = {
            {{AGGREGATE_NAME}}Status.DRAFT: [
                {{AGGREGATE_NAME}}Status.ACTIVE, 
                {{AGGREGATE_NAME}}Status.ARCHIVED
            ],
            {{AGGREGATE_NAME}}Status.ACTIVE: [
                {{AGGREGATE_NAME}}Status.SUSPENDED, 
                {{AGGREGATE_NAME}}Status.ARCHIVED
            ],
            {{AGGREGATE_NAME}}Status.SUSPENDED: [
                {{AGGREGATE_NAME}}Status.ACTIVE, 
                {{AGGREGATE_NAME}}Status.ARCHIVED
            ],
            {{AGGREGATE_NAME}}Status.ARCHIVED: []  # No transitions from archived
        }
        
        return to_status in allowed_transitions.get(from_status, [])
    
    # =========================================================================
    # BUSINESS LOGIC HELPERS
    # =========================================================================
    
    def _ensure_exists(self) -> None:
        """Ensure {{AGGREGATE_NAME_LOWER}} exists (has been created)."""
        if self.version == 0:
            raise {{AGGREGATE_NAME}}NotFoundError(f"{{AGGREGATE_NAME}} {self.id} does not exist")
    
    def _ensure_can_modify(self) -> None:
        """Ensure {{AGGREGATE_NAME_LOWER}} can be modified."""
        if self.status == {{AGGREGATE_NAME}}Status.ARCHIVED:
            raise {{AGGREGATE_NAME}}InvalidStateError(
                f"Cannot modify archived {{AGGREGATE_NAME_LOWER}} {self.id}"
            )
    
    def _update_activity(self) -> None:
        """Update last activity timestamp."""
        self._last_activity = datetime.now()
    
    # =========================================================================
    # QUERY METHODS (COMPUTED PROPERTIES)
    # =========================================================================
    
    @property
    def is_active(self) -> bool:
        """Check if {{AGGREGATE_NAME_LOWER}} is active."""
        return self.status == {{AGGREGATE_NAME}}Status.ACTIVE
    
    @property
    def is_archived(self) -> bool:
        """Check if {{AGGREGATE_NAME_LOWER}} is archived."""
        return self.status == {{AGGREGATE_NAME}}Status.ARCHIVED
    
    @property
    def days_since_creation(self) -> Optional[int]:
        """Get days since creation."""
        if not self.created_at:
            return None
        return (datetime.now() - self.created_at).days
    
    @property
    def days_since_update(self) -> Optional[int]:
        """Get days since last update."""
        if not self.updated_at:
            return None
        return (datetime.now() - self.updated_at).days
    
    def get_activity_summary(self) -> Dict[str, Any]:
        """Get activity summary."""
        return {
            'total_changes': self._change_count,
            'days_since_creation': self.days_since_creation,
            'days_since_update': self.days_since_update,
            'is_active': self.is_active,
            'created_by': self.created_by,
            'last_updated_by': self.updated_by
        }
    
    def can_transition_to(self, status: {{AGGREGATE_NAME}}Status) -> bool:
        """Check if can transition to given status."""
        return self._is_valid_status_transition(self.status, status)
    
    def __str__(self) -> str:
        """String representation."""
        return f"{{AGGREGATE_NAME}}(id={self.id}, name='{self.name}', status={self.status}, version={self.version})"
    
    def __repr__(self) -> str:
        """Developer representation."""
        return (
            f"{{AGGREGATE_NAME}}(id='{self.id}', name='{self.name}', "
            f"status={self.status}, version={self.version}, "
            f"created_by='{self.created_by}')"
        )

# =============================================================================
# USAGE EXAMPLE
# =============================================================================

"""
# Example usage:

from eventuali import EventStore

async def example_usage():
    # Initialize event store
    event_store = await EventStore.create("postgresql://...")
    
    # Register events
    event_store.register_event_class("{{AGGREGATE_NAME}}Created", {{AGGREGATE_NAME}}Created)
    event_store.register_event_class("{{AGGREGATE_NAME}}Updated", {{AGGREGATE_NAME}}Updated)
    event_store.register_event_class("{{AGGREGATE_NAME}}StatusChanged", {{AGGREGATE_NAME}}StatusChanged)
    event_store.register_event_class("{{AGGREGATE_NAME}}Archived", {{AGGREGATE_NAME}}Archived)
    
    # Create new {{AGGREGATE_NAME_LOWER}}
    {{aggregate_name_lower}} = {{AGGREGATE_NAME}}(id="{{aggregate_name_lower}}-123")
    {{aggregate_name_lower}}.create(
        name="My {{AGGREGATE_NAME}}",
        created_by="user-456",
        description="A sample {{AGGREGATE_NAME_LOWER}}"
    )
    
    # Save to event store
    await event_store.save({{aggregate_name_lower}})
    {{aggregate_name_lower}}.mark_events_as_committed()
    
    # Load and modify
    loaded_{{aggregate_name_lower}} = await event_store.load({{AGGREGATE_NAME}}, "{{aggregate_name_lower}}-123")
    loaded_{{aggregate_name_lower}}.update(
        updated_by="user-789",
        description="Updated description"
    )
    
    # Change status
    loaded_{{aggregate_name_lower}}.change_status(
        new_status={{AGGREGATE_NAME}}Status.SUSPENDED,
        reason="Temporary suspension for review",
        changed_by="admin-123"
    )
    
    # Save changes
    await event_store.save(loaded_{{aggregate_name_lower}})
    loaded_{{aggregate_name_lower}}.mark_events_as_committed()
    
    print(f"{{AGGREGATE_NAME}} activity: {loaded_{{aggregate_name_lower}}.get_activity_summary()}")
"""