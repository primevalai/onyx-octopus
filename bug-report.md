# Eventuali Bug Report: Event Subclass Fields Lost During Deserialization

## Summary
When storing and retrieving custom Event subclasses with additional fields, eventuali deserializes all events as the base `Event` class, causing all custom fields to be lost. The data exists in the database but is inaccessible after retrieval.

## Environment
- **eventuali version**: Latest (as of 2025-08-13)
- **Python version**: 3.x (using UV package manager)
- **Database**: SQLite
- **OS**: Linux 6.8.0-71-generic

## Bug Description

### Expected Behavior
When defining custom Event subclasses with additional fields and storing them via `EventStore`, the events should be retrieved with all their custom fields intact.

### Actual Behavior
All events are deserialized as the base `Event` class, losing all custom fields defined in subclasses.

## Reproduction Steps

### 1. Define Custom Event Subclass
```python
from eventuali import Event
from typing import Dict, Any

class AgentEvent(Event):
    """Event class specifically for agent lifecycle and actions."""
    
    # Agent-specific fields
    agent_name: str = ""
    agent_id: str = ""
    parent_agent_id: str = ""
    workflow_id: str = ""
    event_name: str = ""
    attributes: Dict[str, Any] = {}
    
    def get_event_type(self) -> str:
        return self.event_name if self.event_name else "agent_event"
```

### 2. Create and Store Event
```python
from eventuali import EventStore, Aggregate
import uuid
from datetime import datetime, timezone

# Create event with custom fields
event = AgentEvent(
    event_id=str(uuid.uuid4()),
    aggregate_id="simonSays-test-123",
    aggregate_type="agent_aggregate",
    agent_name="simonSays",
    agent_id="simonSays-test-123",
    parent_agent_id="",
    workflow_id="workflow-test-456",
    event_name="agent.simonSays.commandReceived",
    attributes={
        "simon_command": "test the enhanced event display",
        "test_type": "verification"
    }
)

# Store via aggregate
aggregate = AgentAggregate(id="simonSays-test-123")
aggregate.apply(event)
await store.save(aggregate)
```

### 3. Retrieve Events
```python
# Load events by aggregate type
events = await store.load_events_by_type("agent_aggregate")

# Check the retrieved event
event = events[-1]
print(f"Event type: {type(event)}")  # Output: <class 'eventuali.event.Event'>
print(f"Event class: {event.__class__.__name__}")  # Output: Event
print(f"Is AgentEvent: {isinstance(event, AgentEvent)}")  # Output: False

# Try to access custom fields
print(f"attributes: {getattr(event, 'attributes', 'NOT FOUND')}")  # Output: NOT FOUND
print(f"agent_name: {getattr(event, 'agent_name', 'NOT FOUND')}")  # Output: NOT FOUND
```

## Evidence of Data Loss

### Database Contains Full Data
Direct SQLite query shows the data is stored correctly:
```sql
SELECT event_data FROM events WHERE aggregate_type = 'agent_aggregate' ORDER BY timestamp DESC LIMIT 1;
```
Result:
```json
{
  "agent_id": "simonSays-test-123",
  "agent_name": "simonSays",
  "attributes": {
    "simon_command": "test the enhanced event display",
    "test_type": "verification"
  },
  "event_name": "agent.simonSays.commandReceived",
  "parent_agent_id": "",
  "workflow_id": "workflow-test-456"
}
```

### Retrieved Event Missing Fields
Using `event.to_dict()` or `event.model_dump()`:
```json
{
  "event_id": "a67ad7b4-0ac9-4a82-a763-49ea08ebc897",
  "aggregate_id": "simonSays-test-123",
  "aggregate_type": "agent_aggregate",
  "event_type": "agent.simonSays.commandReceived",
  "event_version": 1,
  "aggregate_version": 1,
  "timestamp": "2025-08-13 22:28:50.446488+00:00",
  "causation_id": null,
  "correlation_id": null,
  "user_id": null
}
```

**Missing fields**: `attributes`, `agent_name`, `agent_id`, `parent_agent_id`, `workflow_id`, `event_name`

## Test Script to Reproduce
```python
#!/usr/bin/env python3
"""Test script to demonstrate event deserialization issue"""

import asyncio
import json
from pathlib import Path
from eventuali import EventStore
from routes.events import AgentEvent

async def test_event_retrieval():
    """Test how events are retrieved and what data they contain"""
    
    events_dir = Path(".events")
    db_path = events_dir / "events.db"
    store = await EventStore.create(f"sqlite:///{db_path.absolute()}")
    
    # Load agent events
    events = await store.load_events_by_type("agent_aggregate")
    print(f"Found {len(events)} agent events\n")
    
    if events:
        event = events[-1]
        
        print(f"Event type: {type(event)}")
        print(f"Event class: {event.__class__.__name__}")
        print(f"Is AgentEvent: {isinstance(event, AgentEvent)}")
        
        # Check for custom fields
        for attr in ['attributes', 'agent_name', 'agent_id', 'workflow_id']:
            value = getattr(event, attr, 'NOT FOUND')
            print(f"  {attr}: {value}")
        
        # Show what to_dict() returns
        print(f"\nto_dict() keys: {list(event.to_dict().keys())}")

if __name__ == "__main__":
    asyncio.run(test_event_retrieval())
```

## Root Cause Analysis

The issue appears to be in the event deserialization process:

1. Events are stored in the database with `event_data` as JSON containing all custom fields
2. The `event_data_type` column is set to "json" 
3. During retrieval, eventuali deserializes all events as the base `Event` class
4. There's no mechanism to determine which Event subclass should be used for deserialization
5. All custom fields in the stored JSON are ignored/lost

## Impact

This bug makes it impossible to:
- Use custom Event subclasses with additional fields effectively
- Implement domain-specific event sourcing patterns
- Access event metadata that's critical for business logic

In our specific case, we cannot display what command triggered our agent events, even though this data is stored in the database.

## Suggested Fixes

### Option 1: Store Class Information
Store the Python class path in the database (e.g., in metadata or a new column) and use it during deserialization.

### Option 2: Event Type Registry
Implement an event type registry that maps event_type strings to Python classes:
```python
EventStore.register_event_type("agent_aggregate", AgentEvent)
```

### Option 3: Custom Deserializer Hook
Provide a hook for custom deserialization logic:
```python
store = await EventStore.create(
    connection_string,
    event_deserializer=custom_deserializer
)
```

### Option 4: Preserve Unknown Fields
If the exact class can't be determined, preserve all JSON fields as a dict attribute on the Event object.

## Workaround Attempts

We tried several workarounds that all failed:
1. Setting fields after event creation - fields not persisted
2. Using getattr with defaults - fields don't exist on base Event
3. Checking hasattr for custom fields - always returns False
4. Accessing event.__dict__ - only contains base Event fields

## Additional Context

This issue is blocking our event telemetry system where we need to display:
- What commands triggered specific agent events
- Agent-specific metadata (agent_id, workflow_id)
- Custom attributes for debugging and monitoring

The data exists in the database but is completely inaccessible through the eventuali API, forcing us to consider bypassing eventuali entirely with direct SQL queries.

## Reproducible Example Repository

The full code demonstrating this issue is available at:
- Repository: https://github.com/primevalai/gold-grizzly
- Relevant files:
  - `.apps/api/routes/events.py` - Event class definitions
  - `.apps/api/dependencies/database.py` - Event retrieval code
  - `.apps/api/test_event_retrieval.py` - Test script demonstrating the issue

## Expected Resolution

Events stored with custom fields should be retrievable with those fields intact. The library should either:
1. Automatically detect and use the correct Event subclass
2. Provide a mechanism to register Event subclasses
3. At minimum, preserve all stored data even if the type is unknown

Thank you for your attention to this issue. This is a critical blocker for using eventuali in production systems that require custom event types with domain-specific fields.