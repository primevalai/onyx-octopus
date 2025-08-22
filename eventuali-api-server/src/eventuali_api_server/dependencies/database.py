"""Database dependencies for eventuali integration."""

import asyncio
import logging
from pathlib import Path
from typing import AsyncGenerator, List, Dict, Any, Optional

from eventuali import EventStore, Event

from ..config import get_config

logger = logging.getLogger(__name__)


class DatabaseManager:
    """Manager for eventuali EventStore connection."""
    
    def __init__(self) -> None:
        self._store: EventStore | None = None
        self._lock = asyncio.Lock()
        self.config = get_config()
    
    async def get_store(self) -> EventStore:
        """Get or create EventStore instance."""
        if self._store is None:
            async with self._lock:
                if self._store is None:
                    # Ensure events directory exists
                    events_dir = Path(self.config.data_dir)
                    events_dir.mkdir(exist_ok=True)
                    
                    # Create SQLite EventStore
                    db_path = events_dir / "events.db"
                    self._store = await EventStore.create(
                        f"sqlite:///{db_path.absolute()}"
                    )
                    
                    # Register custom event classes
                    from ..routes.events import AgentEvent, WorkflowEvent, SystemEvent
                    
                    EventStore.register_event_class("AgentEvent", AgentEvent)
                    EventStore.register_event_class("WorkflowEvent", WorkflowEvent)
                    EventStore.register_event_class("SystemEvent", SystemEvent)
                    
                    logger.info("EventStore initialized and custom event classes registered")
        
        return self._store
    
    async def close(self) -> None:
        """Close the EventStore connection."""
        if self._store is not None:
            # EventStore cleanup if needed
            self._store = None
            logger.info("EventStore connection closed")
    
    async def get_recent_events(
        self, 
        limit: int = 100, 
        offset: int = 0,
        aggregate_type: Optional[str] = None,
        event_type: Optional[str] = None,
        since: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get recent events from the event store."""
        store = await self.get_store()
        
        try:
            # Load events from specified aggregate types or all
            if aggregate_type:
                events = await asyncio.wait_for(
                    store.load_events_by_type(aggregate_type), 
                    timeout=self.config.database_timeout
                )
            else:
                # Load from all aggregate types
                tasks = [
                    store.load_events_by_type("agent_aggregate"),
                    store.load_events_by_type("workflow_aggregate"), 
                    store.load_events_by_type("system_aggregate"),
                    store.load_events_by_type("event_aggregate"),  # Legacy
                ]
                
                try:
                    results = await asyncio.wait_for(
                        asyncio.gather(*tasks, return_exceptions=True),
                        timeout=self.config.database_timeout
                    )
                    
                    all_events = []
                    for result in results:
                        if isinstance(result, Exception):
                            logger.warning(f"Failed to load events: {result}")
                            continue
                        all_events.extend(list(result))
                    
                    events = all_events
                    
                except asyncio.TimeoutError:
                    logger.error("Timeout loading events")
                    return []
            
            # Convert events to dictionaries
            event_dicts = []
            for event in events:
                event_dict = event.to_dict()
                
                # Add metadata fields
                event_dict.update({
                    'event_id': str(event.event_id),
                    'aggregate_id': event.aggregate_id,
                    'aggregate_type': event.aggregate_type,
                    'event_type': event.event_type,
                    'aggregate_version': event.aggregate_version,
                    'timestamp': event.timestamp.isoformat() if event.timestamp else None,
                    'user_id': event.user_id,
                    'causation_id': str(event.causation_id) if event.causation_id else None,
                    'correlation_id': str(event.correlation_id) if event.correlation_id else None,
                    'attributes': getattr(event, 'attributes', {}),
                    'agent_name': getattr(event, 'agent_name', ''),
                    'agent_id': getattr(event, 'agent_id', ''),
                    'parent_agent_id': getattr(event, 'parent_agent_id', ''),
                    'workflow_id': getattr(event, 'workflow_id', ''),
                    'event_name': getattr(event, 'event_name', ''),
                })
                
                # Override correlation/causation with agent-specific fields
                if event_dict.get('workflow_id'):
                    event_dict['correlation_id'] = event_dict['workflow_id']
                if event_dict.get('parent_agent_id'):
                    event_dict['causation_id'] = event_dict['parent_agent_id']
                
                # Filter by event type if specified
                if event_type is None or event.event_type == event_type:
                    event_dicts.append(event_dict)
            
            # Filter by timestamp if specified
            if since:
                event_dicts = [
                    e for e in event_dicts 
                    if e.get('timestamp', '') > since
                ]
            
            # Sort by timestamp (most recent first)
            event_dicts.sort(
                key=lambda x: x.get('timestamp', ''), 
                reverse=True
            )
            
            # Apply pagination
            start_idx = offset
            end_idx = offset + limit
            return event_dicts[start_idx:end_idx]
            
        except Exception as e:
            logger.error(f"Error retrieving events: {e}")
            return []
    
    async def get_agent_events(self, agent_id: str, limit: int = 100) -> List[Dict[str, Any]]:
        """Get all events for a specific agent aggregate."""
        return await self.get_recent_events(
            limit=limit,
            aggregate_type="agent_aggregate"
        )
    
    async def get_workflow_events(self, workflow_id: str, limit: int = 100) -> List[Dict[str, Any]]:
        """Get all events for a specific workflow aggregate."""
        return await self.get_recent_events(
            limit=limit,
            aggregate_type="workflow_aggregate"
        )
    
    async def get_system_events(self, session_id: str, limit: int = 100) -> List[Dict[str, Any]]:
        """Get all events for a specific system/session aggregate."""
        return await self.get_recent_events(
            limit=limit,
            aggregate_type="system_aggregate"
        )
    
    async def get_workflow_agents(self, workflow_id: str) -> List[Dict[str, Any]]:
        """Get all agents that participated in a specific workflow."""
        store = await self.get_store()
        
        try:
            events = await store.load_events_by_type("agent_aggregate")
            
            workflow_agents = []
            seen_agents = set()
            
            for event in events:
                event_dict = event.to_dict()
                
                correlation_id = event_dict.get('correlation_id')
                if correlation_id == workflow_id:
                    agent_id = event_dict.get('aggregate_id')
                    
                    if agent_id and agent_id not in seen_agents:
                        seen_agents.add(agent_id)
                        
                        agent_name = agent_id.split('-')[0] if '-' in agent_id else 'unknown'
                        
                        workflow_agents.append({
                            'agent_id': agent_id,
                            'agent_name': agent_name,
                            'workflow_id': workflow_id,
                            'first_event_time': event_dict.get('timestamp'),
                            'event_type': event_dict.get('event_type')
                        })
            
            workflow_agents.sort(key=lambda x: x.get('first_event_time', ''))
            return workflow_agents
            
        except Exception as e:
            logger.error(f"Error retrieving workflow agents: {e}")
            return []

    async def health_check(self) -> bool:
        """Check if database connection is healthy."""
        try:
            store = await self.get_store()
            # Try a simple query to verify connection
            await asyncio.wait_for(
                store.load_events_by_type("system_aggregate", limit=1),
                timeout=5.0
            )
            return True
        except Exception as e:
            logger.error(f"Database health check failed: {e}")
            return False


# Global database manager instance
db_manager = DatabaseManager()


async def get_event_store() -> AsyncGenerator[EventStore, None]:
    """Dependency to get EventStore instance."""
    store = await db_manager.get_store()
    try:
        yield store
    except Exception:
        raise