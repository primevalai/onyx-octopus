"""Event routes for the API server."""

import asyncio
import logging
from datetime import datetime
from typing import Dict, Any, List, Optional
from uuid import uuid4

from eventuali import Event
from fastapi import APIRouter, Depends, HTTPException, Query
from sse_starlette.sse import EventSourceResponse

from ..dependencies.database import get_event_store, db_manager
from ..models.events import (
    EventRequest,
    EventResponse,
    EventsResponse,
    EventItem,
    GetEventsRequest,
)

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/events", tags=["events"])


class AgentEvent(Event):
    """Agent lifecycle events."""
    
    def __init__(
        self,
        event_name: str,
        agent_name: str,
        agent_id: str,
        workflow_id: Optional[str] = None,
        parent_agent_id: Optional[str] = None,
        attributes: Optional[Dict[str, Any]] = None,
        **kwargs
    ):
        super().__init__(
            aggregate_id=agent_id,
            aggregate_type="agent_aggregate",
            event_type="AgentEvent",
            **kwargs
        )
        self.event_name = event_name
        self.agent_name = agent_name
        self.agent_id = agent_id
        self.workflow_id = workflow_id
        self.parent_agent_id = parent_agent_id
        self.attributes = attributes or {}


class WorkflowEvent(Event):
    """Workflow lifecycle events."""
    
    def __init__(
        self,
        event_name: str,
        workflow_id: str,
        user_prompt: Optional[str] = None,
        attributes: Optional[Dict[str, Any]] = None,
        **kwargs
    ):
        super().__init__(
            aggregate_id=workflow_id,
            aggregate_type="workflow_aggregate",
            event_type="WorkflowEvent",
            **kwargs
        )
        self.event_name = event_name
        self.workflow_id = workflow_id
        self.user_prompt = user_prompt
        self.attributes = attributes or {}


class SystemEvent(Event):
    """System lifecycle events."""
    
    def __init__(
        self,
        event_name: str,
        session_id: Optional[str] = None,
        attributes: Optional[Dict[str, Any]] = None,
        **kwargs
    ):
        aggregate_id = session_id or "system"
        super().__init__(
            aggregate_id=aggregate_id,
            aggregate_type="system_aggregate",
            event_type="SystemEvent",
            **kwargs
        )
        self.event_name = event_name
        self.session_id = session_id
        self.attributes = attributes or {}


@router.post("/emit/agent", response_model=EventResponse)
async def emit_agent_event(
    request: EventRequest,
    store=Depends(get_event_store)
) -> EventResponse:
    """Emit an agent event."""
    try:
        event = AgentEvent(
            event_name=request.name,
            agent_name=request.attributes.get("agent_name", ""),
            agent_id=request.aggregate_id or str(uuid4()),
            workflow_id=request.correlation_id,
            parent_agent_id=request.causation_id,
            attributes=request.attributes,
            timestamp=request.timestamp or datetime.utcnow()
        )
        
        await store.append_events([event])
        
        return EventResponse(
            success=True,
            event_id=str(event.event_id),
            message=f"Agent event '{request.name}' emitted successfully",
            timestamp=event.timestamp
        )
        
    except Exception as e:
        logger.error(f"Error emitting agent event: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/emit/workflow", response_model=EventResponse)
async def emit_workflow_event(
    request: EventRequest,
    store=Depends(get_event_store)
) -> EventResponse:
    """Emit a workflow event."""
    try:
        event = WorkflowEvent(
            event_name=request.name,
            workflow_id=request.aggregate_id or str(uuid4()),
            user_prompt=request.attributes.get("user_prompt"),
            attributes=request.attributes,
            timestamp=request.timestamp or datetime.utcnow()
        )
        
        await store.append_events([event])
        
        return EventResponse(
            success=True,
            event_id=str(event.event_id),
            message=f"Workflow event '{request.name}' emitted successfully",
            timestamp=event.timestamp
        )
        
    except Exception as e:
        logger.error(f"Error emitting workflow event: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/emit/system", response_model=EventResponse)
async def emit_system_event(
    request: EventRequest,
    store=Depends(get_event_store)
) -> EventResponse:
    """Emit a system event."""
    try:
        event = SystemEvent(
            event_name=request.name,
            session_id=request.aggregate_id,
            attributes=request.attributes,
            timestamp=request.timestamp or datetime.utcnow()
        )
        
        await store.append_events([event])
        
        return EventResponse(
            success=True,
            event_id=str(event.event_id),
            message=f"System event '{request.name}' emitted successfully",
            timestamp=event.timestamp
        )
        
    except Exception as e:
        logger.error(f"Error emitting system event: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/", response_model=EventsResponse)
async def get_events(
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    event_type: Optional[str] = Query(None),
    aggregate_type: Optional[str] = Query(None),
    since: Optional[str] = Query(None)
) -> EventsResponse:
    """Get recent events."""
    try:
        events_data = await db_manager.get_recent_events(
            limit=limit,
            offset=offset,
            aggregate_type=aggregate_type,
            event_type=event_type,
            since=since
        )
        
        events = [EventItem(**event_data) for event_data in events_data]
        
        return EventsResponse(
            events=events,
            total=None,
            limit=limit,
            offset=offset
        )
        
    except Exception as e:
        logger.error(f"Error retrieving events: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/agents/{agent_id}")
async def get_agent_events(
    agent_id: str,
    limit: int = Query(100, ge=1, le=1000)
):
    """Get events for a specific agent."""
    try:
        events = await db_manager.get_agent_events(agent_id, limit)
        return {"events": events}
    except Exception as e:
        logger.error(f"Error retrieving agent events: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/workflows/{workflow_id}")
async def get_workflow_events(
    workflow_id: str,
    limit: int = Query(100, ge=1, le=1000)
):
    """Get events for a specific workflow."""
    try:
        events = await db_manager.get_workflow_events(workflow_id, limit)
        return {"events": events}
    except Exception as e:
        logger.error(f"Error retrieving workflow events: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/workflows/{workflow_id}/agents")
async def get_workflow_agents(workflow_id: str):
    """Get all agents that participated in a workflow."""
    try:
        agents = await db_manager.get_workflow_agents(workflow_id)
        return {"agents": agents}
    except Exception as e:
        logger.error(f"Error retrieving workflow agents: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/stream")
async def stream_events(
    event_type: Optional[str] = Query(None),
    aggregate_type: Optional[str] = Query(None)
):
    """Stream events using Server-Sent Events."""
    
    async def event_generator():
        last_check = datetime.utcnow().isoformat()
        
        while True:
            try:
                events = await db_manager.get_recent_events(
                    limit=50,
                    since=last_check,
                    event_type=event_type,
                    aggregate_type=aggregate_type
                )
                
                for event in events:
                    yield {
                        "event": "event",
                        "data": event
                    }
                
                last_check = datetime.utcnow().isoformat()
                await asyncio.sleep(1)
                
            except Exception as e:
                logger.error(f"Error in event stream: {e}")
                yield {
                    "event": "error",
                    "data": {"error": str(e)}
                }
                await asyncio.sleep(5)
    
    return EventSourceResponse(event_generator())