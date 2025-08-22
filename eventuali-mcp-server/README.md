# Eventuali MCP Server

A Model Context Protocol (MCP) server for the Eventuali event sourcing system, providing real-time event streaming, agent lifecycle management, and workflow orchestration capabilities with integrated API server support.

## Features

- **Real-time Event Streaming**: Continuous polling and streaming of events from Eventuali API
- **Agent Lifecycle Management**: Tools for starting, monitoring, and completing agent workflows
- **Workflow Orchestration**: Support for complex multi-agent workflows with proper correlation
- **Three-Aggregate Pattern**: Enforces proper event naming for Agent, Workflow, and System aggregates
- **Integrated API Server**: Optional auto-start of eventuali-api-server for self-contained operation
- **Type Safety**: Full Pydantic validation and type hints throughout
- **Async Performance**: Built on FastMCP 2.0 for high-performance async operations

## Installation

```bash
pip install eventuali-mcp-server
```

## Quick Start

### Running the MCP Server

```bash
# Start the server on default port 3333
eventuali-mcp-server

# Start with integrated API server (auto-starts eventuali-api-server)
export START_API_SERVER=true
eventuali-mcp-server

# Or specify custom configuration
export EVENT_API_URL=http://localhost:8765
export MCP_PORT=3333
export START_API_SERVER=false
eventuali-mcp-server
```

### Using the Event API Client

```python
from eventuali_mcp_server import EventAPIClient

async def main():
    async with EventAPIClient("http://localhost:8765") as client:
        # Emit an event
        await client.emit_event(
            event_name="agent.myAgent.started",
            attributes={"agent_id": "my-agent-123"},
            aggregate_id="my-agent-123"
        )
        
        # Stream events
        async for event in client.stream_events():
            if event.event == "event_created":
                print(f"Received: {event.data}")
```

### MCP Tools

The server provides several MCP tools for event management:

#### Start Agent
```python
await mcp_client.call_tool("start_agent", {
    "agent_name": "urlCacher",
    "agent_id": "cache-agent-123",
    "workflow_id": "workflow-456",
    "parent_agent_id": "orchestrator"
})
```

#### Emit Agent Event
```python
await mcp_client.call_tool("emit_agent_event", {
    "agent_id": "cache-agent-123",
    "agent_name": "urlCacher",
    "event_name": "processing_started",
    "attributes": {"url_count": 5}
})
```

#### Complete Agent
```python
await mcp_client.call_tool("complete_agent", {
    "agent_id": "cache-agent-123",
    "agent_name": "urlCacher", 
    "success": True,
    "message": "Successfully cached 5 URLs"
})
```

## Event Naming Convention

The server enforces a three-aggregate event naming pattern:

### Agent Events (`agent.<agentName>.*`)
- Format: `agent.<agentName>.<eventName>`
- Examples: `agent.urlCacher.started`, `agent.simonSays.completed`
- Required: `agent_id`, `agent_name`
- Optional: `workflow_id`, `parent_agent_id`

### Workflow Events (`workflow.*`)
- Format: `workflow.<eventName>`
- Examples: `workflow.started`, `workflow.completed`
- Required: `workflow_id`
- Optional: `user_prompt`

### System Events (`system.*`)
- Format: `system.<eventName>`
- Examples: `system.session_started`, `system.error`
- Optional: `session_id`

## Configuration

Environment variables:

- `EVENT_API_URL`: Eventuali API endpoint (default: `http://127.0.0.1:8765`)
- `MCP_PORT`: MCP server port (default: `3333`)
- `MCP_HOST`: MCP server host (default: `127.0.0.1`)
- `START_API_SERVER`: Auto-start eventuali-api-server (default: `false`)

## Development

### Setup

```bash
git clone https://github.com/primevalai/eventuali-mcp-server
cd eventuali-mcp-server
uv pip install -e ".[dev]"
```

### Running Tests

```bash
uv run pytest
```

### Code Quality

```bash
uv run black .
uv run flake8 .
uv run mypy .
```

## Architecture

The package provides:

- **EventAPIClient**: Async HTTP client for Eventuali API communication
- **MCP Server**: FastMCP-based server with event streaming and tools
- **Models**: Pydantic models for type-safe event handling
- **Tools**: MCP tools for agent and workflow management

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see LICENSE file for details.