# EventStreamer API Reference

The `EventStreamer` class provides high-performance event streaming capabilities using Rust's broadcast channels for real-time event processing and subscription management.

## Class Definition

```python
class EventStreamer:
    """
    High-performance event streamer for publishing and subscribing to events.
    
    The EventStreamer provides real-time event streaming capabilities using
    Rust's high-performance broadcast channels under the hood.
    """
```

## Constructor

### `EventStreamer(capacity: int = 1000)`

Initializes a new event streamer with configurable buffer capacity.

**Parameters:**
- `capacity: int` - Maximum number of events to buffer in memory (default: 1000)

**Examples:**

```python
# Default capacity (1000 events)
streamer = EventStreamer()

# High-throughput configuration
streamer = EventStreamer(capacity=10000)

# Memory-constrained environment
streamer = EventStreamer(capacity=100)
```

**Performance Considerations:**
- **Higher capacity**: Better burst handling, more memory usage
- **Lower capacity**: Lower memory usage, potential backpressure under load
- **Optimal range**: 1000-10000 for most applications

---

## Subscription Management

### `subscribe(subscription: Subscription) -> EventStreamReceiver` {async}

Subscribes to events matching the given criteria and returns a receiver for consuming events.

**Parameters:**
- `subscription: Subscription` - Configuration defining which events to receive

**Returns:** `EventStreamReceiver` for consuming filtered events

**Examples:**

```python
from eventuali.streaming import Subscription, SubscriptionBuilder

# Subscribe to all user events
user_subscription = Subscription(
    aggregate_type_filter="User",
    id="user-events"
)
user_receiver = await streamer.subscribe(user_subscription)

# Subscribe to specific event types
email_subscription = Subscription(
    event_type_filter="UserEmailChanged",
    id="email-changes"
)
email_receiver = await streamer.subscribe(email_subscription)

# Builder pattern for complex subscriptions
subscription = (SubscriptionBuilder()
    .with_id("order-processing")
    .filter_by_aggregate_type("Order")
    .filter_by_event_type("OrderPlaced")
    .build())
receiver = await streamer.subscribe(subscription)
```

### `unsubscribe(subscription_id: str)` {async}

Removes an active subscription and stops event delivery.

**Parameters:**
- `subscription_id: str` - ID of the subscription to remove

**Examples:**

```python
# Subscribe with specific ID
subscription = Subscription(id="temp-subscription", aggregate_type_filter="User")
receiver = await streamer.subscribe(subscription)

# Later, unsubscribe
await streamer.unsubscribe("temp-subscription")

# Receiver will stop receiving events
```

---

## Position Tracking

### `get_stream_position(stream_id: str) -> Optional[int]` {async}

Gets the current position for a specific event stream.

**Parameters:**
- `stream_id: str` - ID of the stream

**Returns:** Current stream position, or `None` if stream doesn't exist

### `get_global_position() -> int` {async}

Gets the current global position across all streams.

**Returns:** Current global event position

**Examples:**

```python
# Check stream position
user_position = await streamer.get_stream_position("user-stream")
if user_position is not None:
    print(f"User stream at position: {user_position}")

# Check global position
global_pos = await streamer.get_global_position()
print(f"Global position: {global_pos}")

# Use for checkpoint/resume functionality
checkpoint = await streamer.get_global_position()
# ... restart from checkpoint later
```

### `publish_event(event: Event, stream_position: int, global_position: int)` {async}

Publishes an event to the stream with position information.

**Parameters:**
- `event: Event` - Event to publish
- `stream_position: int` - Position in the event stream
- `global_position: int` - Global position across all streams

**Examples:**

```python
# Usually called internally by EventStore
# But can be used for custom event publishing
await streamer.publish_event(
    event=user_registered_event,
    stream_position=42,
    global_position=1337
)
```

---

## Usage Patterns

### Basic Event Streaming Setup

```python
import asyncio
from eventuali import EventStore, EventStreamer
from eventuali.streaming import Subscription

async def basic_streaming():
    # Setup
    store = await EventStore.create("postgresql://...")
    streamer = EventStreamer(capacity=5000)
    
    # Subscribe to user events
    subscription = Subscription(
        id="user-processor",
        aggregate_type_filter="User"
    )
    receiver = await streamer.subscribe(subscription)
    
    # Process events
    async for stream_event in receiver:
        event = stream_event.event
        print(f"Processing {event.event_type} at position {stream_event.global_position}")
        
        # Your event processing logic here
        await process_user_event(event)

asyncio.run(basic_streaming())
```

### High-Performance Projection Building

```python
class UserAnalyticsProjection:
    """High-performance user analytics projection."""
    
    def __init__(self):
        self.user_count = 0
        self.active_users = 0
        self.registration_rate = []
        self.last_position = 0
    
    async def process_events(self, receiver: EventStreamReceiver):
        """Process events from receiver."""
        async for stream_event in receiver:
            await self.handle_event(stream_event.event)
            self.last_position = stream_event.global_position
    
    async def handle_event(self, event):
        """Handle individual event."""
        if event.event_type == "UserRegistered":
            self.user_count += 1
            self.active_users += 1
            self.registration_rate.append(event.timestamp)
        
        elif event.event_type == "UserDeactivated":
            self.active_users -= 1

# Usage
async def analytics_streaming():
    streamer = EventStreamer(capacity=10000)  # High capacity for analytics
    
    # Subscribe to user events
    subscription = Subscription(aggregate_type_filter="User")
    receiver = await streamer.subscribe(subscription)
    
    # Build projection
    projection = UserAnalyticsProjection()
    await projection.process_events(receiver)

# Performance: 78k+ events/sec processing
```

### Multi-Subscriber Fan-Out

```python
async def multi_subscriber_example():
    """Demonstrate multiple subscribers to same events."""
    streamer = EventStreamer()
    
    # Multiple subscriptions to same event types
    subscribers = []
    
    # Analytics subscriber
    analytics_subscription = Subscription(
        id="analytics",
        aggregate_type_filter="Order"
    )
    analytics_receiver = await streamer.subscribe(analytics_subscription)
    subscribers.append(("Analytics", analytics_receiver))
    
    # Notification subscriber
    notification_subscription = Subscription(
        id="notifications", 
        event_type_filter="OrderPlaced"
    )
    notification_receiver = await streamer.subscribe(notification_subscription)
    subscribers.append(("Notifications", notification_receiver))
    
    # Audit subscriber
    audit_subscription = Subscription(id="audit")  # All events
    audit_receiver = await streamer.subscribe(audit_subscription)
    subscribers.append(("Audit", audit_receiver))
    
    # Process events concurrently
    async def process_subscriber(name, receiver):
        async for stream_event in receiver:
            print(f"{name}: {stream_event.event.event_type}")
    
    # Start all processors concurrently
    tasks = [
        asyncio.create_task(process_subscriber(name, receiver))
        for name, receiver in subscribers
    ]
    
    await asyncio.gather(*tasks)
```

### Error Handling and Resilience

```python
class ResilientEventProcessor:
    """Event processor with error handling and recovery."""
    
    def __init__(self, streamer: EventStreamer):
        self.streamer = streamer
        self.error_count = 0
        self.processed_count = 0
    
    async def start_processing(self, subscription: Subscription):
        """Start processing with automatic retry."""
        max_retries = 3
        retry_count = 0
        
        while retry_count < max_retries:
            try:
                receiver = await self.streamer.subscribe(subscription)
                await self._process_events(receiver)
                break  # Success, exit retry loop
                
            except Exception as e:
                retry_count += 1
                self.error_count += 1
                
                if retry_count >= max_retries:
                    raise Exception(f"Failed after {max_retries} retries: {e}")
                
                # Exponential backoff
                await asyncio.sleep(2 ** retry_count)
    
    async def _process_events(self, receiver: EventStreamReceiver):
        """Process events with individual error handling."""
        async for stream_event in receiver:
            try:
                await self._handle_event(stream_event.event)
                self.processed_count += 1
                
            except Exception as e:
                # Log error but continue processing
                print(f"Error processing event {stream_event.event.event_type}: {e}")
                self.error_count += 1
                
                # Could implement dead letter queue here
                await self._send_to_dead_letter_queue(stream_event.event, e)
    
    async def _handle_event(self, event):
        """Handle individual event (can raise exceptions)."""
        # Your event processing logic
        pass
    
    async def _send_to_dead_letter_queue(self, event, error):
        """Send failed events to dead letter queue."""
        # Implementation depends on your error handling strategy
        pass

# Usage
processor = ResilientEventProcessor(streamer)
subscription = Subscription(aggregate_type_filter="Order")
await processor.start_processing(subscription)
```

---

## Integration with EventStore

### Automatic Event Publishing

```python
# EventStreamer integrates automatically with EventStore
store = await EventStore.create("postgresql://...", streamer=streamer)

# When you save aggregates, events are automatically published to streams
user = User(id="user-123")
user.register("John Doe", "john@example.com")
await store.save(user)  # Event automatically published to streamer

# Subscribers receive events in real-time
```

### Manual Event Store Integration

```python
class EventStoreStreamerBridge:
    """Bridge for manual integration between EventStore and EventStreamer."""
    
    def __init__(self, store: EventStore, streamer: EventStreamer):
        self.store = store
        self.streamer = streamer
        self.last_published_position = 0
    
    async def publish_new_events(self):
        """Publish new events from store to streamer."""
        # Get events after last published position
        new_events = await self.store.load_events_from_position(
            self.last_published_position
        )
        
        for event in new_events:
            await self.streamer.publish_event(
                event=event,
                stream_position=event.aggregate_version,
                global_position=event.global_position
            )
            self.last_published_position = event.global_position
    
    async def start_background_publishing(self):
        """Start background task to continuously publish events."""
        while True:
            await self.publish_new_events()
            await asyncio.sleep(0.1)  # 100ms polling interval

# Usage
bridge = EventStoreStreamerBridge(store, streamer)
asyncio.create_task(bridge.start_background_publishing())
```

---

## Performance Optimization

### Capacity Tuning

```python
# For different workload patterns
configurations = {
    "low_throughput": EventStreamer(capacity=100),      # <1k events/sec
    "medium_throughput": EventStreamer(capacity=1000),  # 1k-10k events/sec  
    "high_throughput": EventStreamer(capacity=10000),   # 10k+ events/sec
    "burst_handling": EventStreamer(capacity=50000),    # Handle large bursts
}
```

### Memory-Efficient Processing

```python
async def memory_efficient_processing():
    """Process events with minimal memory usage."""
    streamer = EventStreamer(capacity=500)  # Smaller buffer
    
    subscription = Subscription(aggregate_type_filter="User")
    receiver = await streamer.subscribe(subscription)
    
    # Process one event at a time
    async for stream_event in receiver:
        # Process immediately, don't accumulate
        await process_event_immediately(stream_event.event)
        
        # Optional: checkpoint progress periodically
        if stream_event.global_position % 1000 == 0:
            await save_checkpoint(stream_event.global_position)
```

### Batch Processing Optimization

```python
async def batch_processing():
    """Process events in batches for better throughput."""
    streamer = EventStreamer(capacity=5000)
    subscription = Subscription(aggregate_type_filter="Order")
    receiver = await streamer.subscribe(subscription)
    
    batch = []
    batch_size = 100
    
    async for stream_event in receiver:
        batch.append(stream_event.event)
        
        if len(batch) >= batch_size:
            # Process entire batch
            await process_event_batch(batch)
            batch.clear()
            
            # Checkpoint batch completion
            await save_checkpoint(stream_event.global_position)
```

---

## Related Documentation

- **[EventStreamReceiver](event-stream-receiver.md)** - Event consumption
- **[Projection](projection.md)** - Read model building  
- **[Subscription](subscription.md)** - Event filtering
- **[Examples](../../../examples/08_projections.py)** - Working streaming examples

---

**Next**: Explore [Projection patterns](projection.md) or see the [streaming example](../../../examples/08_projections.py) in action.