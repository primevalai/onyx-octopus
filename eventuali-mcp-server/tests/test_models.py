"""Tests for Pydantic models."""

import pytest
from datetime import datetime, timezone
from pydantic import ValidationError

from eventuali_mcp_server.models import (
    AgentEventRequest,
    WorkflowEventRequest,
    SystemEventRequest,
    StartAgentRequest,
    CompleteAgentRequest,
    EventResponse,
    EventItem,
)


class TestAgentEventRequest:
    """Test cases for AgentEventRequest model."""
    
    def test_valid_agent_event_request(self):
        """Test valid agent event request creation."""
        request = AgentEventRequest(
            agent_id="agent-123",
            event_name="started",
            agent_name="testAgent",
            workflow_id="workflow-456",
            attributes={"key": "value"}
        )
        
        assert request.agent_id == "agent-123"
        assert request.event_name == "started"
        assert request.agent_name == "testAgent"
        assert request.workflow_id == "workflow-456"
        assert request.attributes == {"key": "value"}
    
    def test_minimal_agent_event_request(self):
        """Test minimal agent event request."""
        request = AgentEventRequest(
            agent_id="agent-123",
            event_name="completed",
            agent_name="testAgent"
        )
        
        assert request.agent_id == "agent-123"
        assert request.event_name == "completed"
        assert request.agent_name == "testAgent"
        assert request.parent_agent_id is None
        assert request.workflow_id is None
        assert request.attributes == {}
    
    def test_missing_required_fields(self):
        """Test validation with missing required fields."""
        with pytest.raises(ValidationError):
            AgentEventRequest(agent_id="agent-123")  # Missing event_name and agent_name


class TestWorkflowEventRequest:
    """Test cases for WorkflowEventRequest model."""
    
    def test_valid_workflow_event_request(self):
        """Test valid workflow event request."""
        request = WorkflowEventRequest(
            workflow_id="workflow-123",
            event_name="started",
            user_prompt="Test workflow",
            attributes={"priority": "high"}
        )
        
        assert request.workflow_id == "workflow-123"
        assert request.event_name == "started"
        assert request.user_prompt == "Test workflow"
        assert request.attributes == {"priority": "high"}


class TestStartAgentRequest:
    """Test cases for StartAgentRequest model."""
    
    def test_valid_start_agent_request(self):
        """Test valid start agent request."""
        request = StartAgentRequest(
            agent_name="urlCacher",
            agent_id="cache-agent-123",
            workflow_id="workflow-456",
            parent_agent_id="orchestrator"
        )
        
        assert request.agent_name == "urlCacher"
        assert request.agent_id == "cache-agent-123"
        assert request.workflow_id == "workflow-456"
        assert request.parent_agent_id == "orchestrator"


class TestCompleteAgentRequest:
    """Test cases for CompleteAgentRequest model."""
    
    def test_successful_completion(self):
        """Test successful agent completion."""
        request = CompleteAgentRequest(
            agent_id="agent-123",
            agent_name="testAgent",
            success=True,
            message="Task completed successfully"
        )
        
        assert request.success is True
        assert request.message == "Task completed successfully"
    
    def test_failed_completion(self):
        """Test failed agent completion."""
        request = CompleteAgentRequest(
            agent_id="agent-123",
            agent_name="testAgent",
            success=False,
            message="Task failed with error"
        )
        
        assert request.success is False
        assert request.message == "Task failed with error"
    
    def test_default_success_value(self):
        """Test default success value."""
        request = CompleteAgentRequest(
            agent_id="agent-123",
            agent_name="testAgent"
        )
        
        assert request.success is True  # Default value


class TestEventResponse:
    """Test cases for EventResponse model."""
    
    def test_successful_response(self):
        """Test successful event response."""
        response = EventResponse(
            success=True,
            event_id="event-123",
            message="Event created successfully"
        )
        
        assert response.success is True
        assert response.event_id == "event-123"
        assert response.message == "Event created successfully"
        assert isinstance(response.timestamp, datetime)
    
    def test_failed_response(self):
        """Test failed event response."""
        response = EventResponse(
            success=False,
            message="Event creation failed"
        )
        
        assert response.success is False
        assert response.event_id is None
        assert response.message == "Event creation failed"


class TestEventItem:
    """Test cases for EventItem model."""
    
    def test_complete_event_item(self):
        """Test complete event item."""
        timestamp = datetime.now(timezone.utc)
        
        event = EventItem(
            event_id="event-123",
            aggregate_id="agent-456",
            aggregate_type="agent_aggregate",
            event_type="agent_event",
            event_name="agent.testAgent.started",
            timestamp=timestamp,
            attributes={"key": "value"},
            agent_name="testAgent",
            agent_id="agent-456",
            workflow_id="workflow-789"
        )
        
        assert event.event_id == "event-123"
        assert event.aggregate_id == "agent-456"
        assert event.event_name == "agent.testAgent.started"
        assert event.agent_name == "testAgent"
        assert event.workflow_id == "workflow-789"
    
    def test_minimal_event_item(self):
        """Test minimal event item."""
        timestamp = datetime.now(timezone.utc)
        
        event = EventItem(
            event_id="event-123",
            aggregate_id="system",
            aggregate_type="system_aggregate",
            event_type="system_event",
            timestamp=timestamp
        )
        
        assert event.event_id == "event-123"
        assert event.aggregate_id == "system"
        assert event.event_name is None
        assert event.attributes == {}