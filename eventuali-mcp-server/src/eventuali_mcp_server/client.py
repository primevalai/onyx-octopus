"""Async HTTP client wrapper for the eventuali event API."""

import asyncio
import json
import logging
from datetime import datetime, timezone
from typing import Dict, Any, List, Optional, AsyncGenerator
from urllib.parse import urljoin

import httpx
from pydantic import ValidationError

from .models import (
    EventResponse,
    EventItem,
    EventsResponse,
    StreamEventItem,
    HealthResponse,
)

logger = logging.getLogger(__name__)


class EventAPIClient:
    """Async HTTP client for the eventuali event API."""
    
    def __init__(self, base_url: str = "http://127.0.0.1:8765"):
        """Initialize the client with base URL."""
        self.base_url = base_url.rstrip("/")
        self.events_url = f"{self.base_url}/events"
        self._client: Optional[httpx.AsyncClient] = None
        self._closed = False
    
    async def __aenter__(self):
        """Async context manager entry."""
        await self._ensure_client()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
    
    async def _ensure_client(self):
        """Ensure HTTP client is created."""
        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(
                timeout=httpx.Timeout(30.0),
                headers={"Content-Type": "application/json"}
            )
    
    async def close(self):
        """Close the HTTP client."""
        if self._client and not self._client.is_closed:
            await self._client.aclose()
        self._closed = True
    
    async def emit_agent_event(
        self,
        event_name: str,
        attributes: Dict[str, Any] = None,
        aggregate_id: Optional[str] = None,
        causation_id: Optional[str] = None,
        correlation_id: Optional[str] = None,
    ) -> EventResponse:
        """Emit an agent event to the API."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        event_data = {
            "name": event_name,
            "attributes": attributes or {},
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
        
        if aggregate_id:
            event_data["aggregate_id"] = aggregate_id
        if causation_id:
            event_data["causation_id"] = causation_id
        if correlation_id:
            event_data["correlation_id"] = correlation_id
        
        try:
            response = await self._client.post(
                f"{self.events_url}/emit/agent",
                json=event_data
            )
            response.raise_for_status()
            data = response.json()
            return EventResponse(**data)
        except httpx.HTTPError as e:
            logger.error(f"HTTP error emitting agent event: {e}")
            raise
        except ValidationError as e:
            logger.error(f"Response validation error: {e}")
            raise
    
    async def emit_workflow_event(
        self,
        event_name: str,
        attributes: Dict[str, Any] = None,
        aggregate_id: Optional[str] = None,
        causation_id: Optional[str] = None,
        correlation_id: Optional[str] = None,
    ) -> EventResponse:
        """Emit a workflow event to the API."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        event_data = {
            "name": event_name,
            "attributes": attributes or {},
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
        
        if aggregate_id:
            event_data["aggregate_id"] = aggregate_id
        if causation_id:
            event_data["causation_id"] = causation_id
        if correlation_id:
            event_data["correlation_id"] = correlation_id
        
        try:
            response = await self._client.post(
                f"{self.events_url}/emit/workflow",
                json=event_data
            )
            response.raise_for_status()
            data = response.json()
            return EventResponse(**data)
        except httpx.HTTPError as e:
            logger.error(f"HTTP error emitting workflow event: {e}")
            raise
        except ValidationError as e:
            logger.error(f"Response validation error: {e}")
            raise
    
    async def emit_system_event(
        self,
        event_name: str,
        attributes: Dict[str, Any] = None,
        aggregate_id: Optional[str] = None,
        causation_id: Optional[str] = None,
        correlation_id: Optional[str] = None,
    ) -> EventResponse:
        """Emit a system event to the API."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        event_data = {
            "name": event_name,
            "attributes": attributes or {},
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
        
        if aggregate_id:
            event_data["aggregate_id"] = aggregate_id
        if causation_id:
            event_data["causation_id"] = causation_id
        if correlation_id:
            event_data["correlation_id"] = correlation_id
        
        try:
            response = await self._client.post(
                f"{self.events_url}/emit/system",
                json=event_data
            )
            response.raise_for_status()
            data = response.json()
            return EventResponse(**data)
        except httpx.HTTPError as e:
            logger.error(f"HTTP error emitting system event: {e}")
            raise
        except ValidationError as e:
            logger.error(f"Response validation error: {e}")
            raise
    
    async def emit_event(
        self,
        event_name: str,
        attributes: Dict[str, Any] = None,
        aggregate_id: Optional[str] = None,
        causation_id: Optional[str] = None,
        correlation_id: Optional[str] = None,
    ) -> EventResponse:
        """Emit an event to the API (defaults to agent event for backward compatibility)."""
        return await self.emit_agent_event(
            event_name=event_name,
            attributes=attributes,
            aggregate_id=aggregate_id,
            causation_id=causation_id,
            correlation_id=correlation_id
        )
    
    async def get_events(
        self,
        limit: int = 100,
        offset: int = 0,
        event_type: Optional[str] = None,
    ) -> EventsResponse:
        """Get events with pagination and filtering."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        params = {
            "limit": limit,
            "offset": offset,
        }
        if event_type:
            params["event_type"] = event_type
        
        try:
            response = await self._client.get(
                f"{self.events_url}/",
                params=params
            )
            response.raise_for_status()
            response_data = response.json()
            
            # Convert raw events to EventItem models
            events = []
            events_data = response_data.get('events', [])
            for event_data in events_data:
                try:
                    events.append(EventItem(**event_data))
                except ValidationError as e:
                    logger.warning(f"Failed to parse event: {e}")
                    continue
            
            return EventsResponse(
                events=events,
                total=response_data.get('total', len(events_data)),
                limit=response_data.get('limit', limit),
                offset=response_data.get('offset', offset)
            )
        except httpx.HTTPError as e:
            logger.error(f"HTTP error getting events: {e}")
            raise
    
    async def get_agent_events(self, agent_id: str, limit: int = 100) -> EventsResponse:
        """Get all events for a specific agent."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        try:
            response = await self._client.get(
                f"{self.events_url}/agents/{agent_id}",
                params={"limit": limit}
            )
            response.raise_for_status()
            response_data = response.json()
            
            events = []
            events_data = response_data.get('events', [])
            for event_data in events_data:
                try:
                    events.append(EventItem(**event_data))
                except ValidationError as e:
                    logger.warning(f"Failed to parse agent event: {e}")
                    continue
            
            return EventsResponse(
                events=events,
                total=len(events_data),
                limit=limit,
                offset=0
            )
        except httpx.HTTPError as e:
            logger.error(f"HTTP error getting agent events: {e}")
            raise
    
    async def get_workflow_events(self, workflow_id: str, limit: int = 100) -> EventsResponse:
        """Get all events for a specific workflow."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        try:
            response = await self._client.get(
                f"{self.events_url}/workflows/{workflow_id}",
                params={"limit": limit}
            )
            response.raise_for_status()
            response_data = response.json()
            
            events = []
            events_data = response_data.get('events', [])
            for event_data in events_data:
                try:
                    events.append(EventItem(**event_data))
                except ValidationError as e:
                    logger.warning(f"Failed to parse workflow event: {e}")
                    continue
            
            return EventsResponse(
                events=events,
                total=len(events_data),
                limit=limit,
                offset=0
            )
        except httpx.HTTPError as e:
            logger.error(f"HTTP error getting workflow events: {e}")
            raise
    
    async def get_system_events(self, session_id: str, limit: int = 100) -> EventsResponse:
        """Get all system events for a specific session."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        try:
            # Use general events endpoint with filter for system events
            response = await self._client.get(
                f"{self.events_url}/",
                params={
                    "limit": limit,
                    "aggregate_type": "system_aggregate"
                }
            )
            response.raise_for_status()
            response_data = response.json()
            
            events = []
            events_data = response_data.get('events', [])
            for event_data in events_data:
                try:
                    events.append(EventItem(**event_data))
                except ValidationError as e:
                    logger.warning(f"Failed to parse system event: {e}")
                    continue
            
            return EventsResponse(
                events=events,
                total=len(events_data),
                limit=limit,
                offset=0
            )
        except httpx.HTTPError as e:
            logger.error(f"HTTP error getting system events: {e}")
            raise
    
    async def get_workflow_agents(self, workflow_id: str) -> List[Dict[str, Any]]:
        """Get all agents that participated in a specific workflow."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        try:
            response = await self._client.get(
                f"{self.events_url}/workflows/{workflow_id}/agents"
            )
            response.raise_for_status()
            response_data = response.json()
            return response_data.get('agents', [])
        except httpx.HTTPError as e:
            logger.error(f"HTTP error getting workflow agents: {e}")
            raise
    
    async def stream_events(self) -> AsyncGenerator[StreamEventItem, None]:
        """Stream events via Server-Sent Events."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        try:
            async with self._client.stream(
                "GET",
                f"{self.events_url}/stream",
                headers={"Accept": "text/event-stream"}
            ) as response:
                response.raise_for_status()
                
                async for line in response.aiter_lines():
                    if not line.strip():
                        continue
                    
                    # Parse SSE format
                    if line.startswith("event:"):
                        event_type = line[6:].strip()
                    elif line.startswith("data:"):
                        data_str = line[5:].strip()
                        try:
                            data = json.loads(data_str)
                            yield StreamEventItem(
                                event=event_type,
                                data=data,
                                id=data.get("event_id")
                            )
                        except json.JSONDecodeError as e:
                            logger.warning(f"Failed to parse SSE data: {e}")
                            continue
        except httpx.HTTPError as e:
            logger.error(f"HTTP error streaming events: {e}")
            raise
    
    async def health_check(self) -> HealthResponse:
        """Check API health status."""
        await self._ensure_client()
        
        if self._closed:
            raise RuntimeError("Client is closed")
        
        try:
            response = await self._client.get(f"{self.base_url}/health")
            response.raise_for_status()
            data = response.json()
            
            return HealthResponse(
                status=data.get("status", "unknown"),
                timestamp=datetime.fromisoformat(data.get("timestamp", datetime.now(timezone.utc).isoformat())),
                api_connected=True
            )
        except httpx.HTTPError as e:
            logger.error(f"HTTP error checking health: {e}")
            return HealthResponse(
                status="unhealthy",
                timestamp=datetime.now(timezone.utc),
                api_connected=False
            )
        except Exception as e:
            logger.error(f"Unexpected error checking health: {e}")
            return HealthResponse(
                status="error",
                timestamp=datetime.now(timezone.utc),
                api_connected=False
            )