"""Tests for the EventAPIClient."""

import pytest
from unittest.mock import Mock, patch, AsyncMock
import httpx

from eventuali_mcp_server import EventAPIClient
from eventuali_mcp_server.models import EventResponse


@pytest.fixture
def client():
    """Create a test client."""
    return EventAPIClient("http://test-api.com")


@pytest.mark.asyncio
class TestEventAPIClient:
    """Test cases for EventAPIClient."""
    
    async def test_init(self, client):
        """Test client initialization."""
        assert client.base_url == "http://test-api.com"
        assert client.events_url == "http://test-api.com/events"
        assert client._client is None
        assert client._closed is False
    
    async def test_context_manager(self):
        """Test async context manager."""
        with patch.object(httpx.AsyncClient, 'aclose', new_callable=AsyncMock):
            async with EventAPIClient("http://test.com") as client:
                assert client._client is not None
                assert not client._closed
    
    async def test_emit_event_success(self, client):
        """Test successful event emission."""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "success": True,
            "event_id": "test-123",
            "message": "Event created"
        }
        
        with patch.object(httpx.AsyncClient, 'post', new_callable=AsyncMock, return_value=mock_response):
            response = await client.emit_event("test.event", {"key": "value"})
            
            assert isinstance(response, EventResponse)
            assert response.success is True
            assert response.event_id == "test-123"
    
    async def test_emit_event_with_ids(self, client):
        """Test event emission with correlation and causation IDs."""
        mock_response = Mock()
        mock_response.raise_for_status.return_value = None
        mock_response.json.return_value = {
            "success": True,
            "event_id": "test-123",
            "message": "Event created"
        }
        
        with patch.object(httpx.AsyncClient, 'post', new_callable=AsyncMock, return_value=mock_response) as mock_post:
            await client.emit_event(
                "test.event",
                {"key": "value"},
                aggregate_id="agent-123",
                correlation_id="workflow-456",
                causation_id="parent-789"
            )
            
            # Check that the correct data was posted
            call_args = mock_post.call_args
            posted_data = call_args[1]['json']
            assert posted_data['aggregate_id'] == "agent-123"
            assert posted_data['correlation_id'] == "workflow-456"
            assert posted_data['causation_id'] == "parent-789"
    
    async def test_emit_event_http_error(self, client):
        """Test event emission with HTTP error."""
        with patch.object(httpx.AsyncClient, 'post', new_callable=AsyncMock) as mock_post:
            mock_post.side_effect = httpx.HTTPError("Connection failed")
            
            with pytest.raises(httpx.HTTPError):
                await client.emit_event("test.event")
    
    async def test_close(self, client):
        """Test client closure."""
        mock_client = Mock()
        mock_client.aclose = AsyncMock()
        mock_client.is_closed = False  # Mock that client is not closed
        client._client = mock_client
        
        await client.close()
        
        assert client._closed is True
        mock_client.aclose.assert_called_once()
    
    async def test_closed_client_raises_error(self, client):
        """Test that closed client raises errors."""
        client._closed = True
        
        with pytest.raises(RuntimeError, match="Client is closed"):
            await client.emit_event("test.event")