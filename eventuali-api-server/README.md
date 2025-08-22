# Eventuali API Server

A FastAPI-based REST and Server-Sent Events (SSE) API server for the Eventuali event sourcing system.

## Features

- **Event Management**: Emit and query events across agent, workflow, and system aggregates
- **Real-time Streaming**: Server-Sent Events (SSE) for live event monitoring
- **RESTful API**: Complete REST API with OpenAPI documentation
- **Health Monitoring**: Built-in health checks for API and database
- **CORS Support**: Configurable cross-origin resource sharing
- **CLI Interface**: Simple command-line interface for server management

## Installation

```bash
pip install eventuali-api-server
```

## Quick Start

### Start the server with defaults:

```bash
eventuali-api-server
```

The server will start on `http://127.0.0.1:8765` with Swagger UI available at `/docs`.

### Start with custom configuration:

```bash
eventuali-api-server --host 0.0.0.0 --port 9000 --reload
```

### Environment Variables

All CLI options can also be set via environment variables:

```bash
export HOST=0.0.0.0
export PORT=9000
export RELOAD=true
export LOG_LEVEL=debug
export DATA_DIR=/path/to/events
export CORS_ORIGINS="http://localhost:3000,https://app.example.com"
eventuali-api-server
```

## API Endpoints

### Events

- `POST /events/emit/agent` - Emit agent lifecycle events
- `POST /events/emit/workflow` - Emit workflow lifecycle events  
- `POST /events/emit/system` - Emit system lifecycle events
- `GET /events/` - Query events with pagination and filters
- `GET /events/agents/{agent_id}` - Get events for specific agent
- `GET /events/workflows/{workflow_id}` - Get events for specific workflow
- `GET /events/workflows/{workflow_id}/agents` - Get agents in workflow
- `GET /events/stream` - Real-time event stream via SSE

### Health

- `GET /health/` - Overall health check
- `GET /health/database` - Database connection health

### Documentation

- `GET /docs` - Swagger UI documentation
- `GET /openapi.json` - OpenAPI schema

## Event Types

### Agent Events

```json
{
  "name": "agent.started",
  "attributes": {
    "agent_name": "url-cacher",
    "message": "Agent started successfully"
  },
  "aggregate_id": "url-cacher-abc123",
  "correlation_id": "workflow-def456",
  "causation_id": "parent-agent-789"
}
```

### Workflow Events

```json
{
  "name": "workflow.started", 
  "attributes": {
    "user_prompt": "Cache the documentation page"
  },
  "aggregate_id": "workflow-def456"
}
```

### System Events

```json
{
  "name": "session.started",
  "attributes": {
    "session_type": "interactive"
  },
  "aggregate_id": "session-ghi789"
}
```

## Configuration

The server can be configured via CLI options, environment variables, or programmatically:

```python
from eventuali_api_server import APIServerConfig, set_config

config = APIServerConfig(
    host="0.0.0.0",
    port=8765,
    data_dir="/custom/events/path",
    cors_origins=["https://myapp.com"],
    log_level="debug"
)

set_config(config)
```

## Development

### Install for development:

```bash
git clone <repository>
cd eventuali-api-server
pip install -e ".[dev]"
```

### Run tests:

```bash
pytest
```

### Code quality:

```bash
black src/ tests/
flake8 src/ tests/
mypy src/
```

## Integration

The API server integrates with:

- **Eventuali Event Store**: SQLite-based event storage
- **eventuali-mcp-server**: MCP server for Claude Code integration  
- **@eventuali/dashboard**: React-based UI dashboard

## License

MIT License - see LICENSE file for details.