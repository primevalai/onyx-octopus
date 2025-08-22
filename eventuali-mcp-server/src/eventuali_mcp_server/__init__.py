"""Eventuali MCP Server - Event sourcing integration for Model Context Protocol."""

from .client import EventAPIClient
from .models import (
    AgentEventRequest,
    WorkflowEventRequest,
    SystemEventRequest,
    StartAgentRequest,
    CompleteAgentRequest,
    StartWorkflowRequest,
    CompleteWorkflowRequest,
    EventResponse,
    EventItem,
    EventsResponse,
    GetEventsRequest,
    StreamEventItem,
    HealthResponse,
)

__version__ = "0.1.0"
__author__ = "Primeval AI"
__email__ = "noreply@primevalai.com"

__all__ = [
    "EventAPIClient",
    "AgentEventRequest",
    "WorkflowEventRequest",
    "SystemEventRequest",
    "StartAgentRequest",
    "CompleteAgentRequest",
    "StartWorkflowRequest",
    "CompleteWorkflowRequest",
    "EventResponse",
    "EventItem",
    "EventsResponse",
    "GetEventsRequest",
    "StreamEventItem",
    "HealthResponse",
]