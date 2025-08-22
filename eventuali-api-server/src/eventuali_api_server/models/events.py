"""Event models for the API server."""

from datetime import datetime
from typing import Any, Dict, Optional, List
from uuid import uuid4

from pydantic import BaseModel, Field


class EventRequest(BaseModel):
    """Request model for creating events."""
    
    name: str = Field(..., description="Event name")
    attributes: Dict[str, Any] = Field(
        default_factory=dict, description="Event attributes"
    )
    timestamp: Optional[datetime] = Field(
        default=None, description="Event timestamp (UTC)"
    )
    aggregate_id: Optional[str] = Field(
        default=None, description="Aggregate ID (for agent instances)"
    )
    causation_id: Optional[str] = Field(
        default=None, description="Causation ID (parent-child relationships)"
    )
    correlation_id: Optional[str] = Field(
        default=None, description="Correlation ID (workflow grouping)"
    )


class EventResponse(BaseModel):
    """Response model for event operations."""
    
    success: bool = Field(..., description="Operation success status")
    event_id: str = Field(..., description="Generated event ID")
    message: str = Field(..., description="Response message")
    timestamp: datetime = Field(..., description="Event timestamp (UTC)")


class EventItem(BaseModel):
    """Model for individual event items."""
    
    event_id: str = Field(..., description="Unique event ID")
    aggregate_id: str = Field(..., description="Aggregate ID")
    aggregate_type: str = Field(..., description="Type of aggregate")
    event_type: str = Field(..., description="Type of event")
    event_name: Optional[str] = Field(None, description="Event name")
    timestamp: datetime = Field(..., description="Event timestamp")
    attributes: Dict[str, Any] = Field(default_factory=dict, description="Event attributes")
    
    # Correlation fields
    correlation_id: Optional[str] = Field(None, description="Correlation ID")
    causation_id: Optional[str] = Field(None, description="Causation ID")
    
    # Agent-specific fields
    agent_name: Optional[str] = Field(None, description="Agent name")
    agent_id: Optional[str] = Field(None, description="Agent ID")
    parent_agent_id: Optional[str] = Field(None, description="Parent agent ID")
    workflow_id: Optional[str] = Field(None, description="Workflow ID")
    
    # Workflow-specific fields
    user_prompt: Optional[str] = Field(None, description="User prompt")
    
    # System-specific fields
    session_id: Optional[str] = Field(None, description="Session ID")


class EventsResponse(BaseModel):
    """Response model for event listing."""
    
    events: List[EventItem] = Field(..., description="List of events")
    total: Optional[int] = Field(None, description="Total count if available")
    limit: int = Field(..., description="Query limit")
    offset: int = Field(..., description="Query offset")


class GetEventsRequest(BaseModel):
    """Request model for getting events."""
    
    limit: int = Field(100, description="Maximum events to return", ge=1, le=1000)
    offset: int = Field(0, description="Offset for pagination", ge=0)
    event_type: Optional[str] = Field(None, description="Filter by event type")
    aggregate_type: Optional[str] = Field(None, description="Filter by aggregate type")
    since: Optional[str] = Field(None, description="ISO timestamp for filtering")


class HealthResponse(BaseModel):
    """Response model for health check."""
    
    status: str = Field(..., description="Service status")
    timestamp: datetime = Field(..., description="Health check timestamp")
    api_connected: bool = Field(True, description="API connection status")
    database_connected: bool = Field(True, description="Database connection status")