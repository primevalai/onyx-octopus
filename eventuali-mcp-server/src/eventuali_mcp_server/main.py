"""FastMCP server for eventuali event system integration."""

import asyncio
import logging
import os
import subprocess
import signal
import threading
from collections import deque
from datetime import datetime, timezone
from typing import Dict, Any, List, Optional, AsyncGenerator

from mcp.server import FastMCP
from pydantic import ValidationError

from .client import EventAPIClient
from .models import (
    AgentEventRequest,
    WorkflowEventRequest,
    SystemEventRequest,
    StartAgentRequest,
    CompleteAgentRequest,
    StartWorkflowRequest,
    CompleteWorkflowRequest,
    GetEventsRequest,
    EventResponse,
    EventsResponse,
    HealthResponse,
    StreamEventItem,
)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastMCP server
mcp = FastMCP("eventuali-mcp")

# Global API client and event buffer
api_client: Optional[EventAPIClient] = None
event_buffer: deque = deque(maxlen=1000)  # Buffer for recent events
streaming_task: Optional[asyncio.Task] = None
event_subscribers: List[asyncio.Queue] = []
api_server_process: Optional[subprocess.Popen] = None


async def start_api_server() -> bool:
    """Start the eventuali API server if not already running."""
    global api_server_process
    
    if api_server_process and api_server_process.poll() is None:
        logger.info("API server already running")
        return True
    
    try:
        # Check if we should start the API server
        start_api = os.getenv("START_API_SERVER", "false").lower() == "true"
        if not start_api:
            logger.info("API server auto-start disabled (START_API_SERVER=false)")
            return False
        
        logger.info("Starting eventuali API server...")
        
        # Start the API server process
        api_server_process = subprocess.Popen(
            ["eventuali-api-server", "--host", "127.0.0.1", "--port", "8765"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        
        # Give it a moment to start
        await asyncio.sleep(2)
        
        # Check if it's still running
        if api_server_process.poll() is None:
            logger.info("API server started successfully")
            return True
        else:
            logger.error("API server failed to start")
            return False
            
    except Exception as e:
        logger.error(f"Failed to start API server: {e}")
        return False


async def stop_api_server():
    """Stop the API server if we started it."""
    global api_server_process
    
    if api_server_process and api_server_process.poll() is None:
        logger.info("Stopping API server...")
        api_server_process.terminate()
        try:
            api_server_process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            logger.warning("API server didn't stop gracefully, killing...")
            api_server_process.kill()
        api_server_process = None
        logger.info("API server stopped")


async def get_api_client() -> EventAPIClient:
    """Get or create the API client."""
    global api_client
    if api_client is None:
        base_url = os.getenv("EVENT_API_URL", "http://127.0.0.1:8765")
        api_client = EventAPIClient(base_url)
    return api_client


async def start_event_streaming():
    """Start the background event streaming task."""
    global streaming_task
    if streaming_task is None or streaming_task.done():
        streaming_task = asyncio.create_task(event_streaming_loop())
        logger.info("Event streaming task started")


async def event_streaming_loop():
    """Background task that streams events from the API and buffers them."""
    global event_buffer, event_subscribers
    
    while True:
        try:
            client = await get_api_client()
            logger.info("Starting event stream connection...")
            
            async for stream_item in client.stream_events():
                # Add to buffer
                event_buffer.append(stream_item)
                
                # Notify subscribers
                dead_queues = []
                for queue in event_subscribers:
                    try:
                        queue.put_nowait(stream_item)
                    except asyncio.QueueFull:
                        # Skip if queue is full
                        continue
                    except Exception:
                        # Mark dead queues for removal
                        dead_queues.append(queue)
                
                # Remove dead queues
                for dead_queue in dead_queues:
                    if dead_queue in event_subscribers:
                        event_subscribers.remove(dead_queue)
                
                logger.debug(f"Processed stream event: {stream_item.event}")
        
        except Exception as e:
            logger.error(f"Event streaming error: {e}")
            # Wait before retrying
            await asyncio.sleep(5)


async def subscribe_to_events() -> AsyncGenerator[StreamEventItem, None]:
    """Subscribe to real-time events."""
    global event_subscribers
    
    # Ensure streaming is started
    await start_event_streaming()
    
    # Create a queue for this subscriber
    queue = asyncio.Queue(maxsize=100)
    event_subscribers.append(queue)
    
    try:
        while True:
            # Wait for next event
            event = await queue.get()
            yield event
            queue.task_done()
    finally:
        # Remove from subscribers when done
        if queue in event_subscribers:
            event_subscribers.remove(queue)


# MCP Tool: Emit Agent Event
@mcp.tool()
async def emit_agent_event(request: AgentEventRequest) -> EventResponse:
    """
    Emit an agent lifecycle or action event.
    
    This tool emits events following the agent.<agentName>.<eventName> pattern
    for tracking agent activities within workflows.
    """
    try:
        client = await get_api_client()
        
        # Construct the full event name following the pattern
        event_name = f"agent.{request.agent_name}.{request.event_name}"
        
        # Prepare attributes with agent context
        attributes = {
            "agent_id": request.agent_id,
            "agent_name": request.agent_name,
            **request.attributes
        }
        
        response = await client.emit_agent_event(
            event_name=event_name,
            attributes=attributes,
            aggregate_id=request.agent_id,
            correlation_id=request.workflow_id,
            causation_id=request.parent_agent_id
        )
        
        logger.info(f"Emitted agent event: {event_name}")
        return response
        
    except Exception as e:
        logger.error(f"Failed to emit agent event: {e}")
        return EventResponse(
            success=False,
            message=f"Failed to emit agent event: {str(e)}",
            timestamp=datetime.now(timezone.utc)
        )


# MCP Tool: Start Agent
@mcp.tool()
async def start_agent(request: StartAgentRequest) -> EventResponse:
    """
    Start a new agent with proper event emission.
    
    Convenience tool that emits an 'agent.started' event with proper metadata.
    """
    agent_request = AgentEventRequest(
        agent_id=request.agent_id,
        agent_name=request.agent_name,
        event_name="started",
        parent_agent_id=request.parent_agent_id,
        workflow_id=request.workflow_id,
        attributes={
            "started_at": datetime.now(timezone.utc).isoformat()
        }
    )
    
    return await emit_agent_event(agent_request)


# MCP Tool: Complete Agent
@mcp.tool()
async def complete_agent(request: CompleteAgentRequest) -> EventResponse:
    """
    Complete an agent with success or failure status.
    
    Convenience tool that emits 'agent.completed' or 'agent.failed' event.
    """
    event_name = "completed" if request.success else "failed"
    
    attributes = {
        "success": request.success,
        "completed_at": datetime.now(timezone.utc).isoformat()
    }
    
    if request.message:
        attributes["message"] = request.message
    
    agent_request = AgentEventRequest(
        agent_id=request.agent_id,
        agent_name=request.agent_name,
        event_name=event_name,
        parent_agent_id=request.parent_agent_id,
        workflow_id=request.workflow_id,
        attributes=attributes
    )
    
    return await emit_agent_event(agent_request)


# MCP Tool: Emit Workflow Event
@mcp.tool()
async def emit_workflow_event(request: WorkflowEventRequest) -> EventResponse:
    """
    Emit a workflow lifecycle or coordination event.
    
    Events follow the workflow.<eventName> pattern for workflow management.
    """
    try:
        client = await get_api_client()
        
        event_name = f"workflow.{request.event_name}"
        
        attributes = {
            "workflow_id": request.workflow_id,
            **request.attributes
        }
        
        if request.user_prompt:
            attributes["user_prompt"] = request.user_prompt
        
        response = await client.emit_workflow_event(
            event_name=event_name,
            attributes=attributes,
            aggregate_id=request.workflow_id,
            correlation_id=request.workflow_id
        )
        
        logger.info(f"Emitted workflow event: {event_name}")
        return response
        
    except Exception as e:
        logger.error(f"Failed to emit workflow event: {e}")
        return EventResponse(
            success=False,
            message=f"Failed to emit workflow event: {str(e)}",
            timestamp=datetime.now(timezone.utc)
        )


# MCP Tool: Start Workflow
@mcp.tool()
async def start_workflow(request: StartWorkflowRequest) -> EventResponse:
    """
    Start a new workflow.
    
    Convenience tool that emits a 'workflow.started' event.
    """
    workflow_request = WorkflowEventRequest(
        workflow_id=request.workflow_id,
        event_name="started",
        user_prompt=request.user_prompt,
        attributes={
            "started_at": datetime.now(timezone.utc).isoformat()
        }
    )
    
    return await emit_workflow_event(workflow_request)


# MCP Tool: Complete Workflow
@mcp.tool()
async def complete_workflow(request: CompleteWorkflowRequest) -> EventResponse:
    """
    Complete a workflow.
    
    Convenience tool that emits 'workflow.completed' or 'workflow.failed' event.
    """
    event_name = "completed" if request.success else "failed"
    
    attributes = {
        "success": request.success,
        "completed_at": datetime.now(timezone.utc).isoformat()
    }
    
    if request.message:
        attributes["message"] = request.message
    
    workflow_request = WorkflowEventRequest(
        workflow_id=request.workflow_id,
        event_name=event_name,
        attributes=attributes
    )
    
    return await emit_workflow_event(workflow_request)


# MCP Tool: Emit System Event
@mcp.tool()
async def emit_system_event(request: SystemEventRequest) -> EventResponse:
    """
    Emit a system-level event.
    
    Events follow the system.<eventName> pattern for system operations.
    """
    try:
        client = await get_api_client()
        
        event_name = f"system.{request.event_name}"
        
        attributes = request.attributes.copy()
        if request.session_id:
            attributes["session_id"] = request.session_id
        
        response = await client.emit_system_event(
            event_name=event_name,
            attributes=attributes,
            aggregate_id=request.session_id or "system"
        )
        
        logger.info(f"Emitted system event: {event_name}")
        return response
        
    except Exception as e:
        logger.error(f"Failed to emit system event: {e}")
        return EventResponse(
            success=False,
            message=f"Failed to emit system event: {str(e)}",
            timestamp=datetime.now(timezone.utc)
        )


# MCP Tool: Get Events
@mcp.tool()
async def get_events(request: GetEventsRequest) -> EventsResponse:
    """
    Retrieve events with pagination and filtering.
    
    Returns a list of events matching the specified criteria.
    """
    try:
        client = await get_api_client()
        
        response = await client.get_events(
            limit=request.limit,
            offset=request.offset,
            event_type=request.event_type
        )
        
        logger.info(f"Retrieved {len(response.events)} events")
        return response
        
    except Exception as e:
        logger.error(f"Failed to get events: {e}")
        return EventsResponse(
            events=[],
            total=0,
            limit=request.limit,
            offset=request.offset
        )


# MCP Resource: Recent Events
@mcp.resource("eventuali://recent-events")
async def get_recent_events() -> str:
    """
    Get recent events from the buffer.
    
    Returns the most recent events that have been streamed and buffered.
    """
    global event_buffer
    
    if not event_buffer:
        return "No recent events available"
    
    events_text = []
    for item in list(event_buffer)[-50:]:  # Last 50 events
        if item.event == "event_created" and item.data:
            event_name = item.data.get("event_name", "unknown")
            event_id = item.data.get("event_id", "unknown")
            timestamp = item.data.get("timestamp", "unknown")
            events_text.append(f"- {timestamp}: {event_name} ({event_id[:8]}...)")
        else:
            events_text.append(f"- {item.event}: {item.data}")
    
    return "Recent Events:\n" + "\n".join(events_text)


# MCP Resource: Event Stream
@mcp.resource("eventuali://event-stream")
async def get_event_stream() -> AsyncGenerator[str, None]:
    """
    Real-time event stream.
    
    Provides a live stream of events as they are received from the API.
    """
    async for event in subscribe_to_events():
        if event.event == "event_created" and event.data:
            event_name = event.data.get("event_name", "unknown")
            event_id = event.data.get("event_id", "unknown")
            timestamp = event.data.get("timestamp", "unknown")
            yield f"{timestamp}: {event_name} ({event_id[:8]}...)"
        else:
            yield f"{event.event}: {event.data}"


# MCP Resource: Health Check
@mcp.resource("eventuali://health")
async def get_health() -> str:
    """
    Check the health of the event API connection.
    
    Returns the current health status of the eventuali API.
    """
    try:
        client = await get_api_client()
        health = await client.health_check()
        
        status_emoji = "✅" if health.api_connected else "❌"
        return f"{status_emoji} API Status: {health.status}\nConnected: {health.api_connected}\nChecked: {health.timestamp}"
        
    except Exception as e:
        return f"❌ Health check failed: {str(e)}"


def main():
    """Main entry point for the MCP server."""
    import sys
    
    logger.info("Starting Eventuali MCP Server v0.2.0")
    
    async def startup():
        """Startup tasks."""
        # Try to start API server if configured
        await start_api_server()
    
    async def shutdown():
        """Shutdown tasks."""
        # Stop API server if we started it
        await stop_api_server()
        
        # Close API client if open
        global api_client
        if api_client:
            await api_client.close()
    
    try:
        # Run startup tasks
        asyncio.run(startup())
        
        # Start the MCP server (runs via stdio by default)
        mcp.run()
    except KeyboardInterrupt:
        logger.info("Server stopped by user")
        asyncio.run(shutdown())
        sys.exit(0)
    except Exception as e:
        logger.error(f"Server error: {e}")
        asyncio.run(shutdown())
        sys.exit(1)


if __name__ == "__main__":
    main()