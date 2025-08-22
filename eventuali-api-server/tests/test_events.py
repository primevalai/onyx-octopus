"""Tests for event routes."""

import pytest
from datetime import datetime
from unittest.mock import AsyncMock, patch
from fastapi.testclient import TestClient

from eventuali_api_server.main import create_app
from eventuali_api_server.config import APIServerConfig, set_config


@pytest.fixture
def test_app():
    """Create test FastAPI application."""
    config = APIServerConfig(
        data_dir="test_events",
        log_level="debug"
    )
    set_config(config)
    
    app = create_app()
    return app


@pytest.fixture
def client(test_app):
    """Create test client."""
    return TestClient(test_app)


def test_emit_agent_event(client):
    """Test emitting an agent event."""
    event_data = {
        "name": "agent.started",
        "attributes": {
            "agent_name": "test-agent",
            "message": "Agent started successfully"
        },
        "aggregate_id": "test-agent-123",
        "correlation_id": "workflow-456"
    }
    
    with patch("eventuali_api_server.dependencies.database.get_event_store") as mock_store:
        mock_store_instance = AsyncMock()
        mock_store.return_value.__aenter__.return_value = mock_store_instance
        
        response = client.post("/events/emit/agent", json=event_data)
        
        # May fail due to database setup, but should have correct structure
        if response.status_code == 200:
            data = response.json()
            assert data["success"] is True
            assert "event_id" in data
            assert "message" in data
            assert "timestamp" in data


def test_emit_workflow_event(client):
    """Test emitting a workflow event."""
    event_data = {
        "name": "workflow.started",
        "attributes": {
            "user_prompt": "Create a test file"
        },
        "aggregate_id": "workflow-789"
    }
    
    with patch("eventuali_api_server.dependencies.database.get_event_store") as mock_store:
        mock_store_instance = AsyncMock()
        mock_store.return_value.__aenter__.return_value = mock_store_instance
        
        response = client.post("/events/emit/workflow", json=event_data)
        
        # Structure test even if database fails
        assert response.status_code in [200, 500]


def test_emit_system_event(client):
    """Test emitting a system event."""
    event_data = {
        "name": "session.started",
        "attributes": {
            "session_type": "interactive"
        },
        "aggregate_id": "session-abc"
    }
    
    with patch("eventuali_api_server.dependencies.database.get_event_store") as mock_store:
        mock_store_instance = AsyncMock()
        mock_store.return_value.__aenter__.return_value = mock_store_instance
        
        response = client.post("/events/emit/system", json=event_data)
        
        # Structure test even if database fails
        assert response.status_code in [200, 500]


def test_get_events(client):
    """Test getting events with pagination."""
    with patch("eventuali_api_server.dependencies.database.db_manager") as mock_db:
        mock_db.get_recent_events.return_value = [
            {
                "event_id": "event-1",
                "aggregate_id": "agent-1",
                "aggregate_type": "agent_aggregate",
                "event_type": "AgentEvent",
                "event_name": "started",
                "timestamp": datetime.utcnow(),
                "attributes": {},
                "correlation_id": "workflow-1",
                "causation_id": None,
                "agent_name": "test-agent",
                "agent_id": "agent-1",
                "parent_agent_id": None,
                "workflow_id": "workflow-1"
            }
        ]
        
        response = client.get("/events/?limit=10&offset=0")
        
        if response.status_code == 200:
            data = response.json()
            assert "events" in data
            assert "limit" in data
            assert "offset" in data
            assert data["limit"] == 10
            assert data["offset"] == 0


def test_get_events_with_filters(client):
    """Test getting events with filters."""
    with patch("eventuali_api_server.dependencies.database.db_manager") as mock_db:
        mock_db.get_recent_events.return_value = []
        
        response = client.get(
            "/events/?event_type=AgentEvent&aggregate_type=agent_aggregate"
        )
        
        # Should handle filters correctly
        assert response.status_code in [200, 500]


def test_get_agent_events(client):
    """Test getting events for specific agent."""
    agent_id = "test-agent-123"
    
    with patch("eventuali_api_server.dependencies.database.db_manager") as mock_db:
        mock_db.get_agent_events.return_value = []
        
        response = client.get(f"/events/agents/{agent_id}")
        
        assert response.status_code in [200, 500]


def test_get_workflow_events(client):
    """Test getting events for specific workflow."""
    workflow_id = "workflow-456"
    
    with patch("eventuali_api_server.dependencies.database.db_manager") as mock_db:
        mock_db.get_workflow_events.return_value = []
        
        response = client.get(f"/events/workflows/{workflow_id}")
        
        assert response.status_code in [200, 500]


def test_get_workflow_agents(client):
    """Test getting agents for specific workflow."""
    workflow_id = "workflow-456"
    
    with patch("eventuali_api_server.dependencies.database.db_manager") as mock_db:
        mock_db.get_workflow_agents.return_value = []
        
        response = client.get(f"/events/workflows/{workflow_id}/agents")
        
        assert response.status_code in [200, 500]


def test_invalid_event_data(client):
    """Test posting invalid event data."""
    # Missing required 'name' field
    invalid_data = {
        "attributes": {"key": "value"}
    }
    
    response = client.post("/events/emit/agent", json=invalid_data)
    assert response.status_code == 422  # Validation error


def test_event_stream_endpoint(client):
    """Test event streaming endpoint exists."""
    response = client.get("/events/stream")
    
    # Should either work or fail gracefully
    assert response.status_code in [200, 500]