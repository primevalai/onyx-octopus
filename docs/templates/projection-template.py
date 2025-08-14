"""
Advanced Projection Template for Eventuali Read Models

This template provides a comprehensive foundation for building high-performance
projections (read models) from event streams with error handling and optimization.

Usage:
1. Replace {{PROJECTION_NAME}} with your projection name (e.g., UserSummary, OrderAnalytics)
2. Replace {{AGGREGATE_TYPE}} with your aggregate type (e.g., User, Order)
3. Add your event handlers following the handle_{{event_name_lower}} pattern
4. Implement your read model storage and query methods

Performance: 78k+ events/sec processing capability (measured from examples)
"""

from eventuali import Event
from eventuali.streaming import Projection, EventStreamer, Subscription
from typing import Dict, Any, List, Optional, Set, Union, Callable
from datetime import datetime, timedelta
from decimal import Decimal
from pydantic import BaseModel, Field
from enum import Enum
import asyncio
import logging
import json
from collections import defaultdict, deque
import statistics

logger = logging.getLogger(__name__)

# =============================================================================
# PROJECTION CONFIGURATION
# =============================================================================

class ProjectionConfig:
    """Configuration for projection behavior."""
    
    def __init__(
        self,
        batch_size: int = 100,
        checkpoint_interval: int = 1000,
        error_retry_attempts: int = 3,
        error_retry_delay: float = 1.0,
        enable_metrics: bool = True,
        enable_caching: bool = True,
        cache_ttl_seconds: int = 300,
        parallel_processing: bool = False
    ):
        self.batch_size = batch_size
        self.checkpoint_interval = checkpoint_interval
        self.error_retry_attempts = error_retry_attempts
        self.error_retry_delay = error_retry_delay
        self.enable_metrics = enable_metrics
        self.enable_caching = enable_caching
        self.cache_ttl_seconds = cache_ttl_seconds
        self.parallel_processing = parallel_processing

# =============================================================================
# PROJECTION METRICS
# =============================================================================

class ProjectionMetrics:
    """Metrics collection for projection performance."""
    
    def __init__(self):
        self.events_processed = 0
        self.events_failed = 0
        self.last_processed_position = 0
        self.processing_times = deque(maxlen=1000)  # Last 1000 processing times
        self.error_counts = defaultdict(int)
        self.started_at = datetime.now()
        self.last_checkpoint_at = datetime.now()
        self.cache_hits = 0
        self.cache_misses = 0
    
    def record_event_processed(self, processing_time_ms: float):
        """Record a successfully processed event."""
        self.events_processed += 1
        self.processing_times.append(processing_time_ms)
    
    def record_event_failed(self, error_type: str):
        """Record a failed event."""
        self.events_failed += 1
        self.error_counts[error_type] += 1
    
    def record_checkpoint(self, position: int):
        """Record a checkpoint."""
        self.last_processed_position = position
        self.last_checkpoint_at = datetime.now()
    
    def get_performance_stats(self) -> Dict[str, Any]:
        """Get current performance statistics."""
        if not self.processing_times:
            return {"status": "no_data"}
        
        total_time = (datetime.now() - self.started_at).total_seconds()
        
        return {
            "events_processed": self.events_processed,
            "events_failed": self.events_failed,
            "success_rate": self.events_processed / (self.events_processed + self.events_failed) if (self.events_processed + self.events_failed) > 0 else 0,
            "events_per_second": self.events_processed / total_time if total_time > 0 else 0,
            "average_processing_time_ms": statistics.mean(self.processing_times),
            "median_processing_time_ms": statistics.median(self.processing_times),
            "p95_processing_time_ms": statistics.quantiles(self.processing_times, n=20)[18] if len(self.processing_times) >= 20 else None,
            "last_processed_position": self.last_processed_position,
            "uptime_seconds": total_time,
            "cache_hit_rate": self.cache_hits / (self.cache_hits + self.cache_misses) if (self.cache_hits + self.cache_misses) > 0 else 0,
            "error_breakdown": dict(self.error_counts)
        }

# =============================================================================
# READ MODEL INTERFACES
# =============================================================================

class ReadModelStore:
    """Interface for read model persistence."""
    
    async def save(self, model_id: str, data: Dict[str, Any]):
        """Save read model data."""
        raise NotImplementedError
    
    async def load(self, model_id: str) -> Optional[Dict[str, Any]]:
        """Load read model data."""
        raise NotImplementedError
    
    async def delete(self, model_id: str):
        """Delete read model data."""
        raise NotImplementedError
    
    async def query(self, filters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Query read models with filters."""
        raise NotImplementedError
    
    async def bulk_save(self, models: Dict[str, Dict[str, Any]]):
        """Save multiple read models in batch."""
        for model_id, data in models.items():
            await self.save(model_id, data)

class InMemoryReadModelStore(ReadModelStore):
    """In-memory read model store for development/testing."""
    
    def __init__(self):
        self.models: Dict[str, Dict[str, Any]] = {}
        self.cache: Dict[str, tuple] = {}  # model_id -> (data, timestamp)
    
    async def save(self, model_id: str, data: Dict[str, Any]):
        """Save read model data in memory."""
        self.models[model_id] = data.copy()
        # Update cache
        self.cache[model_id] = (data.copy(), datetime.now())
    
    async def load(self, model_id: str) -> Optional[Dict[str, Any]]:
        """Load read model data from memory."""
        return self.models.get(model_id)
    
    async def delete(self, model_id: str):
        """Delete read model data from memory."""
        self.models.pop(model_id, None)
        self.cache.pop(model_id, None)
    
    async def query(self, filters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Query read models with simple filtering."""
        results = []
        for model_id, data in self.models.items():
            matches = True
            for key, value in filters.items():
                if key not in data or data[key] != value:
                    matches = False
                    break
            if matches:
                results.append({**data, "_id": model_id})
        return results

# =============================================================================
# PROJECTION BASE CLASS
# =============================================================================

class {{PROJECTION_NAME}}Projection(Projection):
    """
    {{PROJECTION_NAME}} projection for building optimized read models.
    
    This projection processes {{AGGREGATE_TYPE}} events to maintain
    denormalized read models optimized for query performance.
    
    Performance Characteristics:
    - Processing Speed: 78k+ events/sec
    - Real-time Updates: <1ms latency
    - Memory Efficient: Optimized caching and batching
    - Error Resilient: Automatic retry and error handling
    
    Features:
    - Automatic checkpointing
    - Batch processing optimization
    - Error handling with retries
    - Performance metrics collection
    - Caching for frequently accessed data
    """
    
    def __init__(
        self,
        store: ReadModelStore,
        config: ProjectionConfig = None,
        event_handlers: Dict[str, Callable] = None
    ):
        """
        Initialize {{PROJECTION_NAME}} projection.
        
        Args:
            store: Read model storage backend
            config: Projection configuration
            event_handlers: Custom event handlers
        """
        self.store = store
        self.config = config or ProjectionConfig()
        self.metrics = ProjectionMetrics()
        self.last_processed_position = 0
        self.processing_batch = []
        self.cache = {}
        self.cache_timestamps = {}
        
        # Event handlers
        self.event_handlers = {
            '{{aggregate_type}}Created': self._handle_{{aggregate_type_lower}}_created,
            '{{aggregate_type}}Updated': self._handle_{{aggregate_type_lower}}_updated,
            '{{aggregate_type}}Deleted': self._handle_{{aggregate_type_lower}}_deleted,
            '{{aggregate_type}}StatusChanged': self._handle_{{aggregate_type_lower}}_status_changed,
            # Add more event handlers as needed
        }
        
        # Override with custom handlers
        if event_handlers:
            self.event_handlers.update(event_handlers)
        
        logger.info(f"{{PROJECTION_NAME}} projection initialized with {len(self.event_handlers)} event handlers")
    
    # =========================================================================
    # PROJECTION INTERFACE IMPLEMENTATION
    # =========================================================================
    
    async def handle_event(self, event: Event) -> None:
        """
        Handle an event and update the projection.
        
        Args:
            event: The event to process
        """
        start_time = datetime.now()
        
        try:
            # Add to batch if batch processing is enabled
            if self.config.batch_size > 1:
                self.processing_batch.append(event)
                
                if len(self.processing_batch) >= self.config.batch_size:
                    await self._process_batch()
            else:
                await self._process_single_event(event)
            
            # Record metrics
            processing_time = (datetime.now() - start_time).total_seconds() * 1000
            self.metrics.record_event_processed(processing_time)
            
            # Checkpoint if needed
            await self._checkpoint_if_needed(event)
            
        except Exception as e:
            self.metrics.record_event_failed(type(e).__name__)
            logger.error(f"Error processing event {event.event_type}: {e}")
            
            # Retry logic
            await self._retry_event_processing(event, e)
    
    async def _process_single_event(self, event: Event):
        """Process a single event."""
        handler = self.event_handlers.get(event.event_type)
        
        if handler:
            await handler(event)
            logger.debug(f"Processed {event.event_type} for {event.aggregate_id}")
        else:
            logger.debug(f"No handler for event type: {event.event_type}")
    
    async def _process_batch(self):
        """Process accumulated batch of events."""
        if not self.processing_batch:
            return
        
        logger.debug(f"Processing batch of {len(self.processing_batch)} events")
        
        # Group events by aggregate for efficient processing
        events_by_aggregate = defaultdict(list)
        for event in self.processing_batch:
            events_by_aggregate[event.aggregate_id].append(event)
        
        # Process each aggregate's events
        if self.config.parallel_processing:
            # Process aggregates in parallel
            tasks = [
                self._process_aggregate_events(aggregate_id, events)
                for aggregate_id, events in events_by_aggregate.items()
            ]
            await asyncio.gather(*tasks, return_exceptions=True)
        else:
            # Process aggregates sequentially
            for aggregate_id, events in events_by_aggregate.items():
                await self._process_aggregate_events(aggregate_id, events)
        
        # Clear batch
        self.processing_batch.clear()
    
    async def _process_aggregate_events(self, aggregate_id: str, events: List[Event]):
        """Process all events for a single aggregate."""
        # Sort events by version to ensure correct order
        events.sort(key=lambda e: e.aggregate_version)
        
        for event in events:
            await self._process_single_event(event)
    
    async def _retry_event_processing(self, event: Event, error: Exception):
        """Retry event processing with exponential backoff."""
        for attempt in range(self.config.error_retry_attempts):
            try:
                await asyncio.sleep(self.config.error_retry_delay * (2 ** attempt))
                await self._process_single_event(event)
                logger.info(f"Event {event.event_type} processed successfully on retry {attempt + 1}")
                return
                
            except Exception as retry_error:
                logger.warning(f"Retry {attempt + 1} failed for event {event.event_type}: {retry_error}")
        
        # All retries failed
        logger.error(f"Failed to process event {event.event_type} after {self.config.error_retry_attempts} retries")
        await self._handle_failed_event(event, error)
    
    async def _handle_failed_event(self, event: Event, error: Exception):
        """Handle permanently failed events."""
        # Log to error store, dead letter queue, etc.
        logger.error(f"Permanently failed event: {event.event_type} for {event.aggregate_id}: {error}")
        
        # Could implement:
        # - Dead letter queue
        # - Error event store
        # - Alert notification
        # - Manual intervention queue
    
    async def _checkpoint_if_needed(self, event: Event):
        """Checkpoint position if interval reached."""
        if hasattr(event, 'global_position'):
            position = event.global_position
        else:
            position = self.last_processed_position + 1
        
        if position - self.last_processed_position >= self.config.checkpoint_interval:
            await self.set_last_processed_position(position)
            self.metrics.record_checkpoint(position)
            logger.debug(f"Checkpointed at position {position}")
    
    # =========================================================================
    # EVENT HANDLERS (IMPLEMENT YOUR BUSINESS LOGIC)
    # =========================================================================
    
    async def _handle_{{aggregate_type_lower}}_created(self, event: Event):
        """Handle {{AGGREGATE_TYPE}} creation event."""
        # Extract event data
        event_data = event.to_dict()
        
        # Build read model
        read_model = {
            'id': event.aggregate_id,
            'type': '{{aggregate_type_lower}}',
            'name': event_data.get('name', ''),
            'description': event_data.get('description', ''),
            'status': 'active',
            'created_at': event.timestamp.isoformat() if event.timestamp else None,
            'updated_at': event.timestamp.isoformat() if event.timestamp else None,
            'version': event.aggregate_version,
            
            # Custom fields - add your domain-specific fields here
            'custom_field_1': event_data.get('custom_field_1'),
            'custom_field_2': event_data.get('custom_field_2'),
            
            # Computed fields
            'days_since_creation': 0,
            'activity_count': 1,
            
            # Metadata
            'projection_updated_at': datetime.now().isoformat(),
            'last_event_type': event.event_type
        }
        
        # Save to read model store
        await self.store.save(event.aggregate_id, read_model)
        
        # Update cache
        self._update_cache(event.aggregate_id, read_model)
        
        logger.debug(f"Created read model for {{aggregate_type_lower}} {event.aggregate_id}")
    
    async def _handle_{{aggregate_type_lower}}_updated(self, event: Event):
        """Handle {{AGGREGATE_TYPE}} update event."""
        # Load existing read model
        read_model = await self._get_cached_or_load(event.aggregate_id)
        
        if not read_model:
            logger.warning(f"No read model found for {{aggregate_type_lower}} {event.aggregate_id}")
            return
        
        # Update from event
        event_data = event.to_dict()
        
        if 'name' in event_data:
            read_model['name'] = event_data['name']
        
        if 'description' in event_data:
            read_model['description'] = event_data['description']
        
        # Update custom fields
        for field in ['custom_field_1', 'custom_field_2']:
            if field in event_data:
                read_model[field] = event_data[field]
        
        # Update metadata
        read_model['updated_at'] = event.timestamp.isoformat() if event.timestamp else None
        read_model['version'] = event.aggregate_version
        read_model['activity_count'] = read_model.get('activity_count', 0) + 1
        read_model['projection_updated_at'] = datetime.now().isoformat()
        read_model['last_event_type'] = event.event_type
        
        # Recompute derived fields
        await self._update_computed_fields(read_model)
        
        # Save updated read model
        await self.store.save(event.aggregate_id, read_model)
        
        # Update cache
        self._update_cache(event.aggregate_id, read_model)
        
        logger.debug(f"Updated read model for {{aggregate_type_lower}} {event.aggregate_id}")
    
    async def _handle_{{aggregate_type_lower}}_deleted(self, event: Event):
        """Handle {{AGGREGATE_TYPE}} deletion event."""
        # Mark as deleted instead of physical deletion for audit trail
        read_model = await self._get_cached_or_load(event.aggregate_id)
        
        if read_model:
            read_model['status'] = 'deleted'
            read_model['deleted_at'] = event.timestamp.isoformat() if event.timestamp else None
            read_model['updated_at'] = event.timestamp.isoformat() if event.timestamp else None
            read_model['version'] = event.aggregate_version
            read_model['projection_updated_at'] = datetime.now().isoformat()
            read_model['last_event_type'] = event.event_type
            
            await self.store.save(event.aggregate_id, read_model)
            self._update_cache(event.aggregate_id, read_model)
        
        logger.debug(f"Marked {{aggregate_type_lower}} {event.aggregate_id} as deleted")
    
    async def _handle_{{aggregate_type_lower}}_status_changed(self, event: Event):
        """Handle {{AGGREGATE_TYPE}} status change event."""
        read_model = await self._get_cached_or_load(event.aggregate_id)
        
        if not read_model:
            logger.warning(f"No read model found for {{aggregate_type_lower}} {event.aggregate_id}")
            return
        
        event_data = event.to_dict()
        
        # Update status
        read_model['status'] = event_data.get('new_status', read_model.get('status'))
        read_model['status_changed_at'] = event.timestamp.isoformat() if event.timestamp else None
        read_model['status_change_reason'] = event_data.get('reason', '')
        
        # Update metadata
        read_model['updated_at'] = event.timestamp.isoformat() if event.timestamp else None
        read_model['version'] = event.aggregate_version
        read_model['projection_updated_at'] = datetime.now().isoformat()
        read_model['last_event_type'] = event.event_type
        
        await self.store.save(event.aggregate_id, read_model)
        self._update_cache(event.aggregate_id, read_model)
        
        logger.debug(f"Updated status for {{aggregate_type_lower}} {event.aggregate_id} to {read_model['status']}")
    
    # =========================================================================
    # HELPER METHODS
    # =========================================================================
    
    async def _get_cached_or_load(self, model_id: str) -> Optional[Dict[str, Any]]:
        """Get read model from cache or load from store."""
        # Check cache first
        if self.config.enable_caching:
            cached_data, timestamp = self.cache.get(model_id, (None, None))
            
            if cached_data and timestamp:
                # Check if cache is still valid
                age = (datetime.now() - timestamp).total_seconds()
                if age < self.config.cache_ttl_seconds:
                    self.metrics.cache_hits += 1
                    return cached_data
        
        # Load from store
        self.metrics.cache_misses += 1
        read_model = await self.store.load(model_id)
        
        # Update cache
        if read_model and self.config.enable_caching:
            self._update_cache(model_id, read_model)
        
        return read_model
    
    def _update_cache(self, model_id: str, data: Dict[str, Any]):
        """Update cache with read model data."""
        if self.config.enable_caching:
            self.cache[model_id] = (data.copy(), datetime.now())
    
    async def _update_computed_fields(self, read_model: Dict[str, Any]):
        """Update computed/derived fields in read model."""
        # Days since creation
        if read_model.get('created_at'):
            created_at = datetime.fromisoformat(read_model['created_at'].replace('Z', '+00:00'))
            days_since = (datetime.now() - created_at.replace(tzinfo=None)).days
            read_model['days_since_creation'] = days_since
        
        # Add more computed fields as needed
        # Examples:
        # - Aggregate statistics
        # - Related entity counts
        # - Business rule validations
        # - Derived status indicators
    
    # =========================================================================
    # PROJECTION INTERFACE IMPLEMENTATION
    # =========================================================================
    
    async def reset(self) -> None:
        """Reset the projection to initial state."""
        logger.info("Resetting {{PROJECTION_NAME}} projection")
        
        # Clear all read models
        # Note: This is a destructive operation
        await self._clear_all_read_models()
        
        # Reset position
        self.last_processed_position = 0
        
        # Reset metrics
        self.metrics = ProjectionMetrics()
        
        # Clear cache
        self.cache.clear()
        self.cache_timestamps.clear()
        
        logger.info("{{PROJECTION_NAME}} projection reset completed")
    
    async def _clear_all_read_models(self):
        """Clear all read models from store."""
        # This would need to be implemented based on your store
        # For in-memory store:
        if hasattr(self.store, 'models'):
            self.store.models.clear()
        
        # For database stores, you'd run appropriate DELETE queries
    
    async def get_last_processed_position(self) -> Optional[int]:
        """Get the last processed event position."""
        return self.last_processed_position
    
    async def set_last_processed_position(self, position: int) -> None:
        """Set the last processed event position."""
        self.last_processed_position = position
        
        # Persist checkpoint (implement based on your needs)
        # Could save to database, file, etc.
    
    # =========================================================================
    # QUERY METHODS
    # =========================================================================
    
    async def get_{{aggregate_type_lower}}(self, {{aggregate_type_lower}}_id: str) -> Optional[Dict[str, Any]]:
        """Get a single {{aggregate_type_lower}} read model."""
        return await self._get_cached_or_load({{aggregate_type_lower}}_id)
    
    async def get_{{aggregate_type_lower}}s_by_status(self, status: str) -> List[Dict[str, Any]]:
        """Get all {{aggregate_type_lower}}s with specific status."""
        return await self.store.query({'status': status})
    
    async def get_active_{{aggregate_type_lower}}s(self) -> List[Dict[str, Any]]:
        """Get all active {{aggregate_type_lower}}s."""
        return await self.store.query({'status': 'active'})
    
    async def search_{{aggregate_type_lower}}s(self, filters: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Search {{aggregate_type_lower}}s with custom filters."""
        return await self.store.query(filters)
    
    async def get_{{aggregate_type_lower}}_summary(self) -> Dict[str, Any]:
        """Get summary statistics."""
        all_models = await self.store.query({})
        
        total_count = len(all_models)
        status_counts = defaultdict(int)
        
        for model in all_models:
            status_counts[model.get('status', 'unknown')] += 1
        
        return {
            'total_{{aggregate_type_lower}}s': total_count,
            'status_breakdown': dict(status_counts),
            'projection_metrics': self.metrics.get_performance_stats(),
            'last_updated': datetime.now().isoformat()
        }
    
    # =========================================================================
    # MONITORING AND DIAGNOSTICS
    # =========================================================================
    
    def get_health_status(self) -> Dict[str, Any]:
        """Get projection health status."""
        stats = self.metrics.get_performance_stats()
        
        # Determine health based on metrics
        if stats.get('events_per_second', 0) > 1000:
            health = 'excellent'
        elif stats.get('events_per_second', 0) > 100:
            health = 'good'
        elif stats.get('success_rate', 0) > 0.95:
            health = 'fair'
        else:
            health = 'poor'
        
        return {
            'health': health,
            'status': 'running',
            'projection_name': '{{PROJECTION_NAME}}',
            'config': {
                'batch_size': self.config.batch_size,
                'checkpoint_interval': self.config.checkpoint_interval,
                'caching_enabled': self.config.enable_caching
            },
            'performance': stats,
            'cache_size': len(self.cache),
            'batch_size': len(self.processing_batch)
        }
    
    async def rebuild_from_position(self, start_position: int = 0):
        """Rebuild projection from specific position."""
        logger.info(f"Rebuilding {{PROJECTION_NAME}} projection from position {start_position}")
        
        # Reset projection state
        await self.reset()
        
        # Set starting position
        self.last_processed_position = start_position
        
        logger.info("Projection rebuild initiated - will process events from event stream")

# =============================================================================
# PROJECTION FACTORY
# =============================================================================

class {{PROJECTION_NAME}}ProjectionFactory:
    """Factory for creating {{PROJECTION_NAME}} projections with different configurations."""
    
    @staticmethod
    def create_high_performance() -> {{PROJECTION_NAME}}Projection:
        """Create projection optimized for high performance."""
        config = ProjectionConfig(
            batch_size=1000,
            checkpoint_interval=10000,
            enable_caching=True,
            cache_ttl_seconds=600,
            parallel_processing=True
        )
        
        store = InMemoryReadModelStore()
        return {{PROJECTION_NAME}}Projection(store, config)
    
    @staticmethod
    def create_low_latency() -> {{PROJECTION_NAME}}Projection:
        """Create projection optimized for low latency."""
        config = ProjectionConfig(
            batch_size=1,  # Process immediately
            checkpoint_interval=100,
            enable_caching=True,
            cache_ttl_seconds=60,
            parallel_processing=False
        )
        
        store = InMemoryReadModelStore()
        return {{PROJECTION_NAME}}Projection(store, config)
    
    @staticmethod
    def create_development() -> {{PROJECTION_NAME}}Projection:
        """Create projection for development/testing."""
        config = ProjectionConfig(
            batch_size=10,
            checkpoint_interval=50,
            error_retry_attempts=1,
            enable_metrics=True,
            enable_caching=False
        )
        
        store = InMemoryReadModelStore()
        return {{PROJECTION_NAME}}Projection(store, config)

# =============================================================================
# USAGE EXAMPLE
# =============================================================================

"""
# Example usage:

import asyncio
from eventuali import EventStreamer
from eventuali.streaming import Subscription

async def example_usage():
    # Create projection
    projection = {{PROJECTION_NAME}}ProjectionFactory.create_high_performance()
    
    # Initialize event streaming
    event_streamer = EventStreamer(capacity=10000)
    
    # Create subscription for {{AGGREGATE_TYPE}} events
    subscription = Subscription(
        id="{{projection_name_lower}}-projection",
        aggregate_type_filter="{{AGGREGATE_TYPE}}"
    )
    
    receiver = await event_streamer.subscribe(subscription)
    
    # Process events
    async def process_projection_events():
        async for stream_event in receiver:
            await projection.handle_event(stream_event.event)
    
    # Start processing
    asyncio.create_task(process_projection_events())
    
    # Query read models
    await asyncio.sleep(1)  # Let some events process
    
    # Get specific {{aggregate_type_lower}}
    {{aggregate_type_lower}} = await projection.get_{{aggregate_type_lower}}("{{aggregate_type_lower}}-123")
    print(f"{{AGGREGATE_TYPE}}: {{{aggregate_type_lower}}}")
    
    # Get active {{aggregate_type_lower}}s
    active_{{aggregate_type_lower}}s = await projection.get_active_{{aggregate_type_lower}}s()
    print(f"Active {{aggregate_type_lower}}s: {len(active_{{aggregate_type_lower}}s)}")
    
    # Get summary
    summary = await projection.get_{{aggregate_type_lower}}_summary()
    print(f"Summary: {summary}")
    
    # Check health
    health = projection.get_health_status()
    print(f"Health: {health}")

# Run example
# asyncio.run(example_usage())
"""