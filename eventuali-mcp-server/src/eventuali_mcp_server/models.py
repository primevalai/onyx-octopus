"""Pydantic models for eventuali MCP server."""

from datetime import datetime, timezone
from typing import Dict, Any, Optional, List
from pydantic import BaseModel, Field


class AgentEventRequest(BaseModel):
    """Request model for emitting agent events."""
    agent_id: str = Field(..., description="Unique agent instance ID")
    event_name: str = Field(..., description="Event name (e.g., 'started', 'completed', 'failed')")
    agent_name: str = Field(..., description="Name of the agent type")
    parent_agent_id: Optional[str] = Field(None, description="Parent agent ID for causation")
    workflow_id: Optional[str] = Field(None, description="Workflow ID for correlation")
    attributes: Dict[str, Any] = Field(default_factory=dict, description="Additional event attributes")


class WorkflowEventRequest(BaseModel):
    """Request model for emitting workflow events."""
    workflow_id: str = Field(..., description="Unique workflow ID")
    event_name: str = Field(..., description="Event name (e.g., 'started', 'completed', 'agent_added')")
    user_prompt: Optional[str] = Field(None, description="Original user prompt that started the workflow")
    attributes: Dict[str, Any] = Field(default_factory=dict, description="Additional event attributes")


class SystemEventRequest(BaseModel):
    """Request model for emitting system events."""
    event_name: str = Field(..., description="Event name (e.g., 'session_started', 'session_ended')")
    session_id: Optional[str] = Field(None, description="Session ID for grouping system events")
    attributes: Dict[str, Any] = Field(default_factory=dict, description="Additional event attributes")


class StartAgentRequest(BaseModel):
    """Request model for starting a new agent."""
    agent_name: str = Field(..., description="Type/name of the agent (e.g., 'url-cacher', 'simon-says')")
    agent_id: str = Field(..., description="Unique agent instance ID")
    parent_agent_id: Optional[str] = Field(None, description="Parent agent ID if this is a sub-agent")
    workflow_id: Optional[str] = Field(None, description="Workflow ID this agent belongs to")


class CompleteAgentRequest(BaseModel):
    """Request model for completing an agent."""
    agent_id: str = Field(..., description="Agent instance ID to complete")
    agent_name: str = Field(..., description="Name of the agent type")
    success: bool = Field(True, description="Whether the agent completed successfully")
    message: Optional[str] = Field(None, description="Completion message or error details")
    workflow_id: Optional[str] = Field(None, description="Workflow ID this agent belongs to")
    parent_agent_id: Optional[str] = Field(None, description="Parent agent ID if this is a sub-agent")


class StartWorkflowRequest(BaseModel):
    """Request model for starting a new workflow."""
    workflow_id: str = Field(..., description="Unique workflow ID")
    user_prompt: str = Field(..., description="The user prompt that initiated the workflow")


class CompleteWorkflowRequest(BaseModel):
    """Request model for completing a workflow."""
    workflow_id: str = Field(..., description="Workflow ID to complete")
    success: bool = Field(True, description="Whether the workflow completed successfully")
    message: Optional[str] = Field(None, description="Completion message or error details")


class EventResponse(BaseModel):
    """Response model for event operations."""
    success: bool = Field(..., description="Whether the operation was successful")
    event_id: Optional[str] = Field(None, description="Generated event ID if successful")
    message: str = Field(..., description="Response message")
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class EventItem(BaseModel):
    """Model for individual event items."""
    event_id: str = Field(..., description="Unique event ID")
    aggregate_id: str = Field(..., description="Aggregate ID (agent, workflow, or system)")
    aggregate_type: str = Field(..., description="Type of aggregate")
    event_type: str = Field(..., description="Type of event")
    event_name: Optional[str] = Field(None, description="Event name for typed events")
    timestamp: datetime = Field(..., description="Event timestamp")
    attributes: Dict[str, Any] = Field(default_factory=dict, description="Event attributes")
    
    # Agent-specific fields
    agent_name: Optional[str] = Field(None, description="Agent name for agent events")
    agent_id: Optional[str] = Field(None, description="Agent ID for agent events")
    parent_agent_id: Optional[str] = Field(None, description="Parent agent ID")
    workflow_id: Optional[str] = Field(None, description="Workflow ID")
    
    # Workflow-specific fields
    user_prompt: Optional[str] = Field(None, description="User prompt for workflow events")
    
    # System-specific fields
    session_id: Optional[str] = Field(None, description="Session ID for system events")
    
    # Correlation fields
    correlation_id: Optional[str] = Field(None, description="Correlation ID")
    causation_id: Optional[str] = Field(None, description="Causation ID")


class EventsResponse(BaseModel):
    """Response model for event listing."""
    events: List[EventItem] = Field(..., description="List of events")
    total: Optional[int] = Field(None, description="Total count if available")
    limit: int = Field(..., description="Query limit")
    offset: int = Field(..., description="Query offset")


class GetEventsRequest(BaseModel):
    """Request model for getting events."""
    limit: int = Field(100, description="Maximum number of events to return", ge=1, le=1000)
    offset: int = Field(0, description="Offset for pagination", ge=0)
    event_type: Optional[str] = Field(None, description="Filter by event type")
    aggregate_type: Optional[str] = Field(None, description="Filter by aggregate type")


class StreamEventItem(BaseModel):
    """Model for streaming event items."""
    event: str = Field(..., description="Event type (event_created, heartbeat, error)")
    data: Dict[str, Any] = Field(..., description="Event data")
    id: Optional[str] = Field(None, description="Event ID for SSE")


class HealthResponse(BaseModel):
    """Health check response model."""
    status: str = Field(..., description="Health status")
    timestamp: datetime = Field(..., description="Health check timestamp")
    api_connected: bool = Field(..., description="Whether API connection is healthy")