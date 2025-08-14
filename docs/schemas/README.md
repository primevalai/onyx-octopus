# AI Agent Documentation & Schemas

**Machine-readable specifications for AI agents and automated tools**

This section provides structured, machine-readable documentation specifically designed for AI agents, code generation tools, and automated systems working with Eventuali.

## 📋 Schema Categories

### 1. **API Schemas**
- [EventStore API Schema](api-schemas/event-store.json) - Complete EventStore interface
- [Event Schema](api-schemas/event.json) - Event class definitions
- [Aggregate Schema](api-schemas/aggregate.json) - Aggregate pattern specifications
- [Streaming Schema](api-schemas/streaming.json) - Real-time processing APIs

### 2. **Data Schemas**
- [Event Data Schema](data-schemas/event-data.json) - Event structure validation
- [Aggregate State Schema](data-schemas/aggregate-state.json) - Aggregate state format
- [Configuration Schema](data-schemas/configuration.json) - System configuration
- [Metadata Schema](data-schemas/metadata.json) - Event metadata structure

### 3. **Pattern Schemas**
- [CQRS Pattern](pattern-schemas/cqrs.json) - Command-Query implementation
- [Saga Pattern](pattern-schemas/saga.json) - Distributed transaction pattern
- [Projection Pattern](pattern-schemas/projection.json) - Read model building
- [Streaming Pattern](pattern-schemas/streaming.json) - Event streaming setup

### 4. **Integration Schemas**
- [FastAPI Integration](integration-schemas/fastapi.json) - Web API patterns
- [Django Integration](integration-schemas/django.json) - Django framework patterns
- [Database Integration](integration-schemas/database.json) - Database configuration
- [Security Integration](integration-schemas/security.json) - Security patterns

## 🤖 AI Agent Quick Reference

### Core Concepts for AI Agents

```json
{
  "eventuali_patterns": {
    "basic_usage": {
      "imports": ["from eventuali import EventStore, Event, Aggregate"],
      "store_creation": "store = await EventStore.create(connection_string)",
      "event_definition": "class EventName(Event): field: type",
      "aggregate_pattern": "class AggregateName(Aggregate): apply_event_name(self, event)",
      "save_pattern": "await store.save(aggregate)",
      "load_pattern": "aggregate = await store.load(AggregateClass, id)"
    },
    "performance_characteristics": {
      "event_creation": "79000+ events/sec",
      "event_persistence": "25000+ events/sec", 
      "event_loading": "40000+ events/sec",
      "memory_efficiency": "8-20x better than pure Python"
    },
    "database_support": {
      "sqlite": "sqlite://path/to/db.db or sqlite://:memory:",
      "postgresql": "postgresql://user:pass@host:port/db"
    }
  }
}
```

### Common Implementation Patterns

```json
{
  "implementation_patterns": {
    "event_sourcing_aggregate": {
      "pattern": "command_method -> create_event -> apply(event) -> event_handler",
      "example": "user.register(name, email) -> UserRegistered -> apply() -> apply_user_registered()",
      "validation": "Command methods validate business rules before creating events"
    },
    "cqrs_separation": {
      "commands": "Handled by aggregates, modify state through events",
      "queries": "Handled by projections, read-only optimized views",
      "example_files": ["examples/09_cqrs_patterns.py"]
    },
    "event_streaming": {
      "setup": "EventStreamer(store) -> subscribe_to_events()",
      "projection": "class ProjectionName(Projection) -> handle_event_type()",
      "example_files": ["examples/08_projections.py"]
    }
  }
}
```

### Decision Trees for AI Agents

```json
{
  "decision_trees": {
    "choose_database": {
      "development": "sqlite://:memory: - Fast, no setup",
      "testing": "sqlite://test.db - Isolated, reproducible", 
      "production_single": "sqlite://prod.db - Simple deployment",
      "production_scale": "postgresql://... - Horizontal scaling"
    },
    "choose_pattern": {
      "simple_crud": "Basic EventStore + Aggregate",
      "complex_business_logic": "CQRS with multiple projections",
      "real_time_updates": "Event Streaming with projections",
      "distributed_systems": "Saga patterns with compensation",
      "multi_tenant": "Tenant isolation with namespace prefixing"
    },
    "performance_optimization": {
      "high_throughput": "Batch processing, connection pooling",
      "low_latency": "In-memory caching, read replicas",
      "large_aggregates": "Snapshot optimization",
      "analytics": "Projection-based read models"
    }
  }
}
```

## 📊 Event Schema Specification

### Core Event Structure

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Eventuali Event",
  "type": "object",
  "required": ["event_type"],
  "properties": {
    "event_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique event identifier (auto-generated)"
    },
    "aggregate_id": {
      "type": "string",
      "description": "ID of the aggregate that generated this event"
    },
    "aggregate_type": {
      "type": "string",
      "description": "Type of the aggregate (e.g., 'User', 'Order')"
    },
    "event_type": {
      "type": "string",
      "description": "Type of the event (e.g., 'UserRegistered')"
    },
    "event_version": {
      "type": "integer",
      "default": 1,
      "description": "Schema version of the event"
    },
    "aggregate_version": {
      "type": "integer",
      "description": "Version of aggregate after this event"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "When the event occurred (ISO 8601)"
    },
    "causation_id": {
      "type": "string",
      "format": "uuid",
      "description": "ID of the event that caused this event"
    },
    "correlation_id": {
      "type": "string", 
      "format": "uuid",
      "description": "ID correlating related events"
    },
    "user_id": {
      "type": "string",
      "description": "ID of the user who triggered this event"
    }
  },
  "additionalProperties": true
}
```

### Built-in Event Types

```json
{
  "built_in_events": {
    "UserRegistered": {
      "type": "object",
      "required": ["name", "email"],
      "properties": {
        "name": {"type": "string"},
        "email": {"type": "string", "format": "email"}
      }
    },
    "UserEmailChanged": {
      "type": "object", 
      "required": ["old_email", "new_email"],
      "properties": {
        "old_email": {"type": "string", "format": "email"},
        "new_email": {"type": "string", "format": "email"}
      }
    },
    "UserDeactivated": {
      "type": "object",
      "properties": {
        "reason": {"type": "string", "description": "Optional deactivation reason"}
      }
    }
  }
}
```

## 🏗️ Aggregate Schema Specification

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Eventuali Aggregate",
  "type": "object",
  "required": ["id"],
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique aggregate identifier"
    },
    "version": {
      "type": "integer",
      "minimum": 0,
      "description": "Current aggregate version"
    },
    "uncommitted_events": {
      "type": "array",
      "items": {"$ref": "#/definitions/Event"},
      "description": "Events not yet persisted to store"
    }
  },
  "definitions": {
    "Event": {
      "type": "object",
      "required": ["event_type"],
      "properties": {
        "event_type": {"type": "string"},
        "event_data": {"type": "object"}
      }
    }
  }
}
```

## 🔧 Code Generation Templates

### Event Class Template

```json
{
  "event_template": {
    "class_definition": "class {event_name}(DomainEvent):",
    "docstring": '"""Event fired when {description}."""',
    "fields": [
      "{field_name}: {field_type}",
      "{field_name}: {field_type} = {default_value}"
    ],
    "example": {
      "input": {
        "event_name": "OrderPlaced",
        "description": "an order is placed by a customer",
        "fields": [
          {"name": "customer_id", "type": "str"},
          {"name": "total_amount", "type": "Decimal"},
          {"name": "items", "type": "List[OrderItem]"}
        ]
      },
      "output": "class OrderPlaced(DomainEvent):\\n    \\\"\\\"\\\"Event fired when an order is placed by a customer.\\\"\\\"\\\"\\n    customer_id: str\\n    total_amount: Decimal\\n    items: List[OrderItem]"
    }
  }
}
```

### Aggregate Class Template

```json
{
  "aggregate_template": {
    "class_definition": "class {aggregate_name}(Aggregate):",
    "docstring": '"""{aggregate_name} aggregate with event sourcing."""',
    "init_method": "def __init__(self, id: str = None):\\n    super().__init__(id)\\n    {state_initialization}",
    "command_method": "def {command_name}(self, {parameters}):\\n    # Validate business rules\\n    event = {event_name}({event_data})\\n    self.apply(event)",
    "event_handler": "def apply_{event_name_lower}(self, event: {event_name}):\\n    {state_updates}",
    "example": {
      "aggregate_name": "Order",
      "commands": [
        {"name": "place_order", "event": "OrderPlaced"},
        {"name": "ship_order", "event": "OrderShipped"}
      ],
      "state_fields": ["customer_id", "status", "total_amount"]
    }
  }
}
```

## 🧪 Validation Rules for AI Agents

### Event Validation

```json
{
  "event_validation_rules": {
    "naming": {
      "pattern": "^[A-Z][a-zA-Z]*$",
      "tense": "past_tense",
      "examples": ["UserRegistered", "OrderShipped", "PaymentProcessed"],
      "anti_examples": ["UserRegister", "user_registered", "UpdateUser"]
    },
    "fields": {
      "required_metadata": ["event_type"],
      "auto_generated": ["event_id", "timestamp"],
      "business_data": "Include all context needed to understand what happened",
      "immutability": "Events must not be modified after creation"
    }
  }
}
```

### Aggregate Validation

```json
{
  "aggregate_validation_rules": {
    "structure": {
      "inheritance": "Must inherit from Aggregate base class",
      "id_field": "Must have id field for identification",
      "state_fields": "All state as instance variables",
      "no_public_setters": "State only modified through events"
    },
    "methods": {
      "command_pattern": "Public methods for commands, validate before events",
      "event_handler_pattern": "apply_{event_name_lower} methods for each event",
      "business_logic": "Commands contain business rules, handlers only update state"
    }
  }
}
```

## 📈 Performance Optimization Schemas

### Connection Configuration

```json
{
  "connection_optimization": {
    "sqlite": {
      "development": "sqlite://:memory:",
      "testing": "sqlite://test.db",
      "production": "sqlite://app.db?cache=shared&mode=rwc"
    },
    "postgresql": {
      "basic": "postgresql://user:pass@host/db",
      "pooled": "postgresql://user:pass@host/db?application_name=eventuali&pool_size=20",
      "optimized": "postgresql://user:pass@host/db?application_name=eventuali&pool_size=20&max_overflow=30&pool_timeout=30"
    }
  }
}
```

### Performance Patterns

```json
{
  "performance_patterns": {
    "batch_processing": {
      "pattern": "Collect events in batches, save together",
      "code": "events = []; events.append(event); await store.save_batch(events)",
      "benefit": "Reduced database round trips"
    },
    "connection_pooling": {
      "pattern": "Reuse database connections across requests",
      "configuration": "Pool size based on expected concurrent load",
      "monitoring": "Track connection utilization metrics"
    },
    "read_replicas": {
      "pattern": "Route reads to replicas, writes to primary",
      "setup": "Configure replica connection strings",
      "load_balancing": "Round-robin or least-connections"
    }
  }
}
```

## 🔗 Integration Specifications

### FastAPI Integration Schema

```json
{
  "fastapi_integration": {
    "dependency_injection": {
      "pattern": "async def get_event_store() -> EventStore",
      "usage": "store: EventStore = Depends(get_event_store)",
      "lifecycle": "Create once per request or singleton"
    },
    "error_handling": {
      "eventuali_exceptions": "Convert to HTTP status codes",
      "concurrency_errors": "Return 409 Conflict",
      "validation_errors": "Return 400 Bad Request"
    },
    "response_patterns": {
      "command_endpoint": "Return aggregate ID and version",
      "query_endpoint": "Return projection data",
      "event_streaming": "WebSocket or Server-Sent Events"
    }
  }
}
```

## 📚 Example Code Repository

AI agents can reference these verified examples:

```json
{
  "example_mappings": {
    "basic_concepts": "examples/01_basic_event_store_simple.py",
    "aggregate_lifecycle": "examples/02_aggregate_lifecycle.py", 
    "error_handling": "examples/03_error_handling.py",
    "performance_testing": "examples/04_performance_testing.py",
    "cqrs_patterns": "examples/09_cqrs_patterns.py",
    "saga_patterns": "examples/07_saga_patterns.py",
    "event_streaming": "examples/08_projections.py",
    "multi_tenancy": "examples/30_tenant_isolation_architecture.py",
    "security": "examples/22_event_encryption_at_rest.py",
    "monitoring": "examples/37_opentelemetry_integration.py"
  }
}
```

## 🚀 AI Agent Quickstart

For AI agents generating Eventuali code:

1. **Always use UV**: `uv run python` for all Python execution
2. **Import patterns**: `from eventuali import EventStore, Event, Aggregate`
3. **Database strings**: SQLite for dev, PostgreSQL for production
4. **Event naming**: Past tense, descriptive (UserRegistered, OrderShipped)
5. **Aggregate pattern**: Commands validate → create events → apply → handlers update state
6. **Performance**: Built-in 79k+ events/sec, use examples for optimization

## 🔗 Related Documentation

- **[API Reference](../api/README.md)** - Human-readable API docs
- **[Integration Guides](../guides/README.md)** - Step-by-step tutorials
- **[Examples](../../examples/README.md)** - 46+ working examples
- **[Architecture](../architecture/README.md)** - System design

---

**For AI Agents**: Use these schemas for code generation, validation, and integration patterns. All examples are tested and verified working code.