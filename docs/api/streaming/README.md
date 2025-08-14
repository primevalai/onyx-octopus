# Streaming API Reference

**Real-time event processing and subscription system**

Eventuali's streaming API provides high-performance, real-time event processing capabilities built on Rust's async runtime for maximum throughput and minimal latency.

## 🚀 Core Components

### [EventStreamer](event-streamer.md)
High-performance event publisher/subscriber system with broadcast channels

### [EventStreamReceiver](event-stream-receiver.md)  
Consumer for receiving events from subscriptions

### [Projection](projection.md)
Base class for building read models from event streams

### [SagaHandler](saga-handler.md)
Distributed transaction coordination with compensation

### [Subscription](subscription.md)
Configuration for event filtering and routing

## 📊 Performance Characteristics

Based on verified measurements from working examples:

| Component | Throughput | Latency | Use Case |
|-----------|------------|---------|----------|
| **EventStreamer** | 78k+ events/sec | <1ms | Real-time processing |
| **Projections** | 78k+ events/sec | <1ms | Read model building |
| **Sagas** | ~214ms avg | - | Distributed transactions |
| **Subscriptions** | 1.1 updates/sec | <100ms | Dashboard updates |

*Performance data from [`examples/08_projections.py`](../../../examples/08_projections.py)*

## 🔄 Quick Start

### Basic Event Streaming

```python
from eventuali import EventStore, EventStreamer, Projection

# Setup
store = await EventStore.create("postgresql://...")
streamer = EventStreamer()

# Create projection
class UserProjection(Projection):
    def __init__(self):
        self.users = {}
    
    async def handle_user_registered(self, event):
        self.users[event.aggregate_id] = {
            'name': event.name,
            'email': event.email
        }

# Subscribe to events
projection = UserProjection()
await streamer.subscribe(projection)
await streamer.start()
```

### Real-time Dashboard Updates

```python
class DashboardProjection(Projection):
    def __init__(self):
        self.metrics = {
            'total_users': 0,
            'active_sessions': 0,
            'revenue': 0.0
        }
    
    async def handle_user_registered(self, event):
        self.metrics['total_users'] += 1
        await self.broadcast_update()
    
    async def broadcast_update(self):
        # Send to WebSocket clients
        await websocket_broadcast(self.metrics)

# 1.1 updates/sec real-time performance
```

## 📋 Component Overview

### EventStreamer Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   EventStreamer                         │
├─────────────────────────────────────────────────────────┤
│  Rust Broadcast     │  Subscription    │  Position      │
│  Channels          │  Management      │  Tracking      │
│  - High throughput │  - Event filtering│  - Stream pos  │
│  - Low latency     │  - Type routing  │  - Global pos  │
│  - Memory efficient│  - Pattern match │  - Checkpoints │
└─────────────────────────────────────────────────────────┘
```

### Projection Pipeline

```
Event → Filter → Transform → Update State → Notify Subscribers
  ↓       ↓         ↓           ↓              ↓
78k/sec  Filter   Business   Read Model    WebSocket
        by type   Logic      Update        Broadcast
```

## 🔗 Related Documentation

- **[Architecture Guide](../../architecture/README.md)** - Streaming system design
- **[Performance Guide](../../performance/README.md)** - Optimization strategies
- **[Examples](../../../examples/08_projections.py)** - Working streaming examples
- **[Integration Guides](../../guides/README.md)** - Framework integration

---

**Next**: Explore specific components or see the [projection example](../../../examples/08_projections.py) in action.