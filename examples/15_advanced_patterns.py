#!/usr/bin/env python3
"""
Advanced Patterns Example

This example demonstrates advanced event sourcing patterns:
- Event versioning and schema evolution
- Snapshot optimization with compression
- Complex aggregate relationships and references
- Event sourcing with temporal queries
- Advanced projection patterns with materializers
- Multi-tenant event sourcing architecture
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any, Union, TypeVar, Generic
from datetime import datetime, timezone, timedelta
from enum import Enum
from dataclasses import dataclass, field
from abc import ABC, abstractmethod
import json
import uuid
import time
import gzip
import base64
import hashlib
from collections import defaultdict

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User
from eventuali.event import UserRegistered, Event

# Event Versioning and Schema Evolution
class EventVersion:
    """Event versioning metadata."""
    
    def __init__(self, version: int, schema_hash: str):
        self.version = version
        self.schema_hash = schema_hash
        self.migration_path: List[int] = []

class VersionedEvent(Event):
    """Base class for versioned events."""
    event_version: int = 1
    schema_hash: str = ""
    
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Set event version from class if available, otherwise use default
        if hasattr(self.__class__, 'event_version'):
            self.event_version = self.__class__.event_version
        else:
            self.event_version = 1
        self.schema_hash = self._compute_schema_hash()
    
    def _compute_schema_hash(self) -> str:
        """Compute hash of event schema for versioning."""
        schema_data = {
            "class_name": self.__class__.__name__,
            "fields": sorted([attr for attr in dir(self) if not attr.startswith('_')])
        }
        return hashlib.md5(json.dumps(schema_data, sort_keys=True).encode()).hexdigest()[:8]

# V1 Events
class CustomerRegisteredV1(VersionedEvent):
    """Customer registered event - Version 1."""
    event_version: int = 1
    
    customer_id: str
    email: str
    name: str

class OrderPlacedV1(VersionedEvent):
    """Order placed event - Version 1."""
    event_version: int = 1
    
    order_id: str
    customer_id: str
    total_amount: float

# V2 Events (Schema Evolution)
class CustomerRegisteredV2(VersionedEvent):
    """Customer registered event - Version 2 (added phone and preferences)."""
    
    customer_id: str
    email: str
    name: str
    phone: Optional[str] = None
    preferences: Dict[str, Any] = field(default_factory=dict)
    registration_source: str = "web"

# Define event_version as a class variable after the class is created
CustomerRegisteredV2.event_version = 2

class OrderPlacedV2(VersionedEvent):
    """Order placed event - Version 2 (added items detail and metadata)."""
    
    order_id: str
    customer_id: str
    total_amount: float
    items: List[Dict[str, Any]] = field(default_factory=list)
    currency: str = "USD"
    order_metadata: Dict[str, Any] = field(default_factory=dict)

# Define event_version as a class variable after the class is created
OrderPlacedV2.event_version = 2

# Event Migration System
class EventMigrator:
    """Handles event schema migrations."""
    
    def __init__(self):
        self.migration_rules: Dict[str, Dict[int, callable]] = {}
    
    def register_migration(self, event_type: str, from_version: int, migration_func: callable):
        """Register a migration function."""
        if event_type not in self.migration_rules:
            self.migration_rules[event_type] = {}
        self.migration_rules[event_type][from_version] = migration_func
    
    def migrate_event(self, event_data: Dict[str, Any], target_version: int) -> Dict[str, Any]:
        """Migrate event to target version."""
        event_type = event_data.get("event_type")
        current_version = event_data.get("event_version", 1)
        
        if current_version == target_version:
            return event_data
        
        # Apply migrations step by step
        migrated_data = event_data.copy()
        while migrated_data.get("event_version", 1) < target_version:
            current_v = migrated_data.get("event_version", 1)
            if event_type in self.migration_rules and current_v in self.migration_rules[event_type]:
                migration_func = self.migration_rules[event_type][current_v]
                migrated_data = migration_func(migrated_data)
                migrated_data["event_version"] = current_v + 1
            else:
                break
        
        return migrated_data

# Snapshot System with Compression
@dataclass
class AggregateSnapshot:
    """Compressed aggregate snapshot."""
    aggregate_id: str
    aggregate_type: str
    version: int
    timestamp: str
    compressed_state: str  # Base64 encoded compressed JSON
    checksum: str
    
    @classmethod
    def create(cls, aggregate_id: str, aggregate_type: str, version: int, state: Dict[str, Any]) -> 'AggregateSnapshot':
        """Create compressed snapshot from state."""
        state_json = json.dumps(state, sort_keys=True)
        compressed = gzip.compress(state_json.encode())
        encoded = base64.b64encode(compressed).decode()
        checksum = hashlib.sha256(state_json.encode()).hexdigest()[:16]
        
        return cls(
            aggregate_id=aggregate_id,
            aggregate_type=aggregate_type,
            version=version,
            timestamp=datetime.now(timezone.utc).isoformat(),
            compressed_state=encoded,
            checksum=checksum
        )
    
    def decompress_state(self) -> Dict[str, Any]:
        """Decompress and return state."""
        compressed = base64.b64decode(self.compressed_state.encode())
        decompressed = gzip.decompress(compressed)
        return json.loads(decompressed.decode())
    
    def verify_checksum(self) -> bool:
        """Verify snapshot integrity."""
        state = self.decompress_state()
        state_json = json.dumps(state, sort_keys=True)
        checksum = hashlib.sha256(state_json.encode()).hexdigest()[:16]
        return checksum == self.checksum

class SnapshotStore:
    """Store for aggregate snapshots."""
    
    def __init__(self):
        self.snapshots: Dict[str, List[AggregateSnapshot]] = {}
        self.snapshot_interval = 10  # Take snapshot every 10 events
    
    def save_snapshot(self, snapshot: AggregateSnapshot):
        """Save aggregate snapshot."""
        key = f"{snapshot.aggregate_type}_{snapshot.aggregate_id}"
        if key not in self.snapshots:
            self.snapshots[key] = []
        
        self.snapshots[key].append(snapshot)
        # Keep only last 5 snapshots
        self.snapshots[key] = self.snapshots[key][-5:]
    
    def get_latest_snapshot(self, aggregate_type: str, aggregate_id: str) -> Optional[AggregateSnapshot]:
        """Get latest snapshot for aggregate."""
        key = f"{aggregate_type}_{aggregate_id}"
        snapshots = self.snapshots.get(key, [])
        return snapshots[-1] if snapshots else None
    
    def should_create_snapshot(self, current_version: int) -> bool:
        """Determine if snapshot should be created."""
        return current_version % self.snapshot_interval == 0

# Complex Aggregate with References
class ComplexCustomer:
    """Complex customer aggregate with relationships."""
    
    def __init__(self, customer_id: str = None):
        self.id = customer_id or f"cust-{uuid.uuid4().hex[:8]}"
        self.email: str = ""
        self.name: str = ""
        self.phone: Optional[str] = None
        self.preferences: Dict[str, Any] = {}
        self.registration_source: str = "web"
        
        # Complex relationships
        self.addresses: List[Dict[str, Any]] = []
        self.payment_methods: List[Dict[str, Any]] = []
        self.loyalty_profile: Dict[str, Any] = {}
        self.communication_preferences: Dict[str, Any] = {}
        
        # Aggregate references
        self.order_references: List[str] = []
        self.support_ticket_references: List[str] = []
        
        # Temporal tracking
        self.state_history: List[Dict[str, Any]] = []
        self.version = 0
        self.events: List[Event] = []
    
    def register(self, email: str, name: str, phone: str = None, 
                source: str = "web", preferences: Dict[str, Any] = None):
        """Register customer with V2 schema."""
        self.email = email
        self.name = name
        self.phone = phone
        self.registration_source = source
        self.preferences = preferences or {}
        
        # Initialize defaults
        self.loyalty_profile = {
            "points": 0,
            "tier": "bronze",
            "joined_date": datetime.now(timezone.utc).isoformat()
        }
        
        self.communication_preferences = {
            "email": True,
            "sms": phone is not None,
            "push": False
        }
        
        event = CustomerRegisteredV2(
            customer_id=self.id,
            email=email,
            name=name,
            phone=phone,
            preferences=self.preferences,
            registration_source=source
        )
        
        self._apply_event(event)
        return event
    
    def add_address(self, address_type: str, street: str, city: str, 
                   postal_code: str, country: str, is_default: bool = False):
        """Add customer address."""
        address = {
            "id": f"addr-{uuid.uuid4().hex[:8]}",
            "type": address_type,
            "street": street,
            "city": city,
            "postal_code": postal_code,
            "country": country,
            "is_default": is_default,
            "created_at": datetime.now(timezone.utc).isoformat()
        }
        
        # Set as default if it's the first address or explicitly requested
        if is_default or len(self.addresses) == 0:
            for addr in self.addresses:
                addr["is_default"] = False
            address["is_default"] = True
        
        self.addresses.append(address)
        self._record_state_change("address_added", {"address_id": address["id"]})
        return address
    
    def update_loyalty_points(self, points_change: int, reason: str):
        """Update loyalty points."""
        self.loyalty_profile["points"] += points_change
        
        # Update tier based on points
        points = self.loyalty_profile["points"]
        if points >= 10000:
            new_tier = "platinum"
        elif points >= 5000:
            new_tier = "gold"
        elif points >= 1000:
            new_tier = "silver"
        else:
            new_tier = "bronze"
        
        tier_changed = self.loyalty_profile["tier"] != new_tier
        self.loyalty_profile["tier"] = new_tier
        self.loyalty_profile["last_updated"] = datetime.now(timezone.utc).isoformat()
        
        self._record_state_change("loyalty_updated", {
            "points_change": points_change,
            "new_total": points,
            "tier_changed": tier_changed,
            "new_tier": new_tier,
            "reason": reason
        })
    
    def add_order_reference(self, order_id: str):
        """Add reference to an order."""
        if order_id not in self.order_references:
            self.order_references.append(order_id)
    
    def create_snapshot(self) -> AggregateSnapshot:
        """Create compressed snapshot of current state."""
        state = {
            "id": self.id,
            "email": self.email,
            "name": self.name,
            "phone": self.phone,
            "preferences": self.preferences,
            "registration_source": self.registration_source,
            "addresses": self.addresses,
            "payment_methods": self.payment_methods,
            "loyalty_profile": self.loyalty_profile,
            "communication_preferences": self.communication_preferences,
            "order_references": self.order_references,
            "support_ticket_references": self.support_ticket_references,
            "version": self.version
        }
        
        return AggregateSnapshot.create(self.id, "ComplexCustomer", self.version, state)
    
    def restore_from_snapshot(self, snapshot: AggregateSnapshot):
        """Restore state from snapshot."""
        if not snapshot.verify_checksum():
            raise ValueError("Snapshot checksum verification failed")
        
        state = snapshot.decompress_state()
        self.id = state["id"]
        self.email = state["email"]
        self.name = state["name"]
        self.phone = state.get("phone")
        self.preferences = state.get("preferences", {})
        self.registration_source = state.get("registration_source", "web")
        self.addresses = state.get("addresses", [])
        self.payment_methods = state.get("payment_methods", [])
        self.loyalty_profile = state.get("loyalty_profile", {})
        self.communication_preferences = state.get("communication_preferences", {})
        self.order_references = state.get("order_references", [])
        self.support_ticket_references = state.get("support_ticket_references", [])
        self.version = state.get("version", 0)
    
    def _apply_event(self, event: Event):
        """Apply event to aggregate."""
        event.aggregate_id = self.id
        event.aggregate_version = self.version + 1
        event.event_type = event.__class__.__name__
        
        self.events.append(event)
        self.version += 1
    
    def _record_state_change(self, change_type: str, details: Dict[str, Any]):
        """Record state change for temporal queries."""
        change = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "change_type": change_type,
            "version": self.version,
            "details": details
        }
        self.state_history.append(change)

# Temporal Query Engine
class TemporalQueryEngine:
    """Engine for temporal queries on event sourced data."""
    
    def __init__(self):
        self.aggregate_states: Dict[str, Dict[int, Dict[str, Any]]] = {}
        self.temporal_index: Dict[str, List[Dict[str, Any]]] = {}
    
    def index_aggregate_state(self, aggregate_id: str, version: int, 
                            timestamp: str, state_snapshot: Dict[str, Any]):
        """Index aggregate state for temporal queries."""
        if aggregate_id not in self.aggregate_states:
            self.aggregate_states[aggregate_id] = {}
        
        self.aggregate_states[aggregate_id][version] = {
            "timestamp": timestamp,
            "state": state_snapshot.copy()
        }
        
        # Create temporal index
        if aggregate_id not in self.temporal_index:
            self.temporal_index[aggregate_id] = []
        
        self.temporal_index[aggregate_id].append({
            "version": version,
            "timestamp": timestamp
        })
        
        # Keep sorted by version
        self.temporal_index[aggregate_id].sort(key=lambda x: x["version"])
    
    def query_at_time(self, aggregate_id: str, target_time: datetime) -> Optional[Dict[str, Any]]:
        """Query aggregate state at specific time."""
        if aggregate_id not in self.temporal_index:
            return None
        
        target_timestamp = target_time.isoformat()
        
        # Find latest version before target time
        latest_version = None
        for entry in reversed(self.temporal_index[aggregate_id]):
            if entry["timestamp"] <= target_timestamp:
                latest_version = entry["version"]
                break
        
        if latest_version is None:
            return None
        
        return self.aggregate_states[aggregate_id][latest_version]["state"]
    
    def query_changes_in_period(self, aggregate_id: str, start_time: datetime, 
                              end_time: datetime) -> List[Dict[str, Any]]:
        """Query all changes in time period."""
        if aggregate_id not in self.temporal_index:
            return []
        
        start_timestamp = start_time.isoformat()
        end_timestamp = end_time.isoformat()
        
        changes = []
        for entry in self.temporal_index[aggregate_id]:
            if start_timestamp <= entry["timestamp"] <= end_timestamp:
                state_data = self.aggregate_states[aggregate_id][entry["version"]]
                changes.append({
                    "version": entry["version"],
                    "timestamp": entry["timestamp"],
                    "state": state_data["state"]
                })
        
        return changes

# Advanced Projection with Materialization
class MaterializedView(ABC):
    """Base class for materialized views."""
    
    def __init__(self, view_name: str):
        self.view_name = view_name
        self.last_processed_version = 0
        self.materialized_data: Dict[str, Any] = {}
        self.refresh_strategy = "incremental"  # or "full"
    
    @abstractmethod
    def project_event(self, event: Event, aggregate_id: str):
        """Project event into materialized view."""
        pass
    
    @abstractmethod
    def get_view_data(self) -> Dict[str, Any]:
        """Get current materialized view data."""
        pass
    
    def needs_refresh(self, current_version: int) -> bool:
        """Check if view needs refresh."""
        return current_version > self.last_processed_version

class CustomerAnalyticsView(MaterializedView):
    """Advanced customer analytics materialized view."""
    
    def __init__(self):
        super().__init__("customer_analytics")
        self.customer_metrics: Dict[str, Dict[str, Any]] = {}
        self.segment_data: Dict[str, List[str]] = defaultdict(list)
        self.cohort_analysis: Dict[str, Dict[str, Any]] = {}
    
    def project_event(self, event: Event, aggregate_id: str):
        """Project customer events into analytics."""
        if isinstance(event, CustomerRegisteredV2):
            self._init_customer_metrics(event)
        elif hasattr(event, 'customer_id'):
            self._update_customer_activity(event)
        
        self.last_processed_version = getattr(event, 'aggregate_version', 0)
    
    def _init_customer_metrics(self, event: CustomerRegisteredV2):
        """Initialize customer metrics."""
        self.customer_metrics[event.customer_id] = {
            "customer_id": event.customer_id,
            "registration_date": getattr(event, 'timestamp', datetime.now(timezone.utc)).isoformat() if hasattr(event, 'timestamp') else datetime.now(timezone.utc).isoformat(),
            "registration_source": event.registration_source,
            "total_orders": 0,
            "total_spent": 0.0,
            "last_activity": getattr(event, 'timestamp', datetime.now(timezone.utc)).isoformat() if hasattr(event, 'timestamp') else datetime.now(timezone.utc).isoformat(),
            "preferences": event.preferences,
            "communication_channels": self._extract_channels(event),
            "lifecycle_stage": "new"
        }
        
        # Segment by registration source
        source = event.registration_source
        if event.customer_id not in self.segment_data[f"source_{source}"]:
            self.segment_data[f"source_{source}"].append(event.customer_id)
    
    def _update_customer_activity(self, event: Event):
        """Update customer activity metrics."""
        customer_id = getattr(event, 'customer_id', None)
        if customer_id and customer_id in self.customer_metrics:
            metrics = self.customer_metrics[customer_id]
            metrics["last_activity"] = getattr(event, 'timestamp', datetime.now(timezone.utc)).isoformat() if hasattr(event, 'timestamp') else datetime.now(timezone.utc).isoformat()
            
            # Update lifecycle stage based on activity
            self._update_lifecycle_stage(customer_id)
    
    def _extract_channels(self, event: CustomerRegisteredV2) -> List[str]:
        """Extract communication channels."""
        channels = ["email"]  # Always have email
        if event.phone:
            channels.append("sms")
        return channels
    
    def _update_lifecycle_stage(self, customer_id: str):
        """Update customer lifecycle stage."""
        metrics = self.customer_metrics[customer_id]
        
        # Simple lifecycle logic
        if metrics["total_orders"] == 0:
            metrics["lifecycle_stage"] = "new"
        elif metrics["total_orders"] < 3:
            metrics["lifecycle_stage"] = "active"
        else:
            metrics["lifecycle_stage"] = "loyal"
    
    def get_view_data(self) -> Dict[str, Any]:
        """Get customer analytics view."""
        total_customers = len(self.customer_metrics)
        
        # Calculate segments
        segments = {
            "by_source": {source: len(customers) for source, customers in self.segment_data.items()},
            "by_lifecycle": defaultdict(int)
        }
        
        for customer in self.customer_metrics.values():
            segments["by_lifecycle"][customer["lifecycle_stage"]] += 1
        
        return {
            "total_customers": total_customers,
            "segments": dict(segments["by_lifecycle"]),
            "source_breakdown": segments["by_source"],
            "last_updated": datetime.now(timezone.utc).isoformat(),
            "view_version": self.last_processed_version
        }

# Multi-tenant Architecture
class TenantContext:
    """Tenant context for multi-tenant event sourcing."""
    
    def __init__(self, tenant_id: str, tenant_name: str):
        self.tenant_id = tenant_id
        self.tenant_name = tenant_name
        self.created_at = datetime.now(timezone.utc)
        self.settings: Dict[str, Any] = {}
        self.resource_limits: Dict[str, Any] = {
            "max_events_per_day": 100000,
            "max_aggregates": 10000,
            "max_storage_mb": 1000
        }

class MultiTenantEventStore:
    """Multi-tenant event store wrapper."""
    
    def __init__(self, base_event_store: EventStore):
        self.base_event_store = base_event_store
        self.tenants: Dict[str, TenantContext] = {}
        self.tenant_metrics: Dict[str, Dict[str, Any]] = {}
    
    def create_tenant(self, tenant_id: str, tenant_name: str) -> TenantContext:
        """Create new tenant."""
        if tenant_id in self.tenants:
            raise ValueError(f"Tenant {tenant_id} already exists")
        
        tenant = TenantContext(tenant_id, tenant_name)
        self.tenants[tenant_id] = tenant
        self.tenant_metrics[tenant_id] = {
            "events_stored": 0,
            "aggregates_created": 0,
            "storage_used_mb": 0,
            "last_activity": datetime.now(timezone.utc).isoformat()
        }
        
        return tenant
    
    def get_tenant_context(self, tenant_id: str) -> TenantContext:
        """Get tenant context."""
        if tenant_id not in self.tenants:
            raise ValueError(f"Tenant {tenant_id} not found")
        return self.tenants[tenant_id]
    
    async def store_tenant_event(self, tenant_id: str, event: Event, aggregate_id: str):
        """Store event with tenant isolation."""
        tenant = self.get_tenant_context(tenant_id)
        
        # Add tenant prefix to aggregate ID for isolation
        prefixed_aggregate_id = f"{tenant_id}_{aggregate_id}"
        event.aggregate_id = prefixed_aggregate_id
        
        # Check tenant limits
        metrics = self.tenant_metrics[tenant_id]
        if metrics["events_stored"] >= tenant.resource_limits["max_events_per_day"]:
            raise ValueError(f"Tenant {tenant_id} has exceeded daily event limit")
        
        # Store event (would use actual event store in real implementation)
        metrics["events_stored"] += 1
        metrics["last_activity"] = datetime.now(timezone.utc).isoformat()
        
        return event
    
    def get_tenant_metrics(self, tenant_id: str) -> Dict[str, Any]:
        """Get tenant usage metrics."""
        if tenant_id not in self.tenant_metrics:
            return {}
        
        metrics = self.tenant_metrics[tenant_id].copy()
        tenant = self.tenants[tenant_id]
        
        # Add utilization percentages
        metrics["event_utilization"] = (metrics["events_stored"] / tenant.resource_limits["max_events_per_day"]) * 100
        metrics["storage_utilization"] = (metrics["storage_used_mb"] / tenant.resource_limits["max_storage_mb"]) * 100
        
        return metrics

async def demonstrate_advanced_patterns():
    """Demonstrate advanced event sourcing patterns."""
    print("=== Advanced Patterns Example ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    
    print("1. Event versioning and schema evolution...")
    
    # Set up event migration
    migrator = EventMigrator()
    
    def migrate_customer_v1_to_v2(event_data: Dict[str, Any]) -> Dict[str, Any]:
        """Migrate CustomerRegistered from V1 to V2."""
        migrated = event_data.copy()
        migrated["phone"] = None
        migrated["preferences"] = {}
        migrated["registration_source"] = "web"
        return migrated
    
    migrator.register_migration("CustomerRegisteredV1", 1, migrate_customer_v1_to_v2)
    
    # Create V1 event and migrate to V2
    v1_event_data = {
        "event_type": "CustomerRegisteredV1",
        "event_version": 1,
        "customer_id": "cust-123",
        "email": "alice@example.com",
        "name": "Alice Johnson"
    }
    
    migrated_event = migrator.migrate_event(v1_event_data, 2)
    print(f"   ✓ Migrated V1 event to V2:")
    print(f"     Original fields: {len(v1_event_data)}")
    print(f"     Migrated fields: {len(migrated_event)}")
    print(f"     New fields: phone={migrated_event.get('phone')}, source={migrated_event.get('registration_source')}")
    
    print("\n2. Snapshot optimization with compression...")
    
    # Create complex customer with rich state
    customer = ComplexCustomer("advanced-customer")
    customer.register("john@example.com", "John Smith", "+1234567890", "mobile", 
                     {"newsletter": True, "promotions": False})
    
    # Add complex state
    customer.add_address("home", "123 Main St", "Cityville", "12345", "US", True)
    customer.add_address("work", "456 Business Ave", "Corporate City", "67890", "US", False)
    customer.update_loyalty_points(1500, "Welcome bonus")
    customer.update_loyalty_points(250, "Purchase reward")
    
    for i in range(5):
        customer.add_order_reference(f"order-{i+1}")
    
    # Create and test snapshot
    snapshot = customer.create_snapshot()
    print(f"   ✓ Created compressed snapshot:")
    print(f"     Aggregate: {snapshot.aggregate_type}")
    print(f"     Version: {snapshot.version}")
    print(f"     Compressed size: {len(snapshot.compressed_state)} bytes")
    print(f"     Checksum: {snapshot.checksum}")
    print(f"     Integrity check: {'✅' if snapshot.verify_checksum() else '❌'}")
    
    # Test snapshot restoration
    restored_customer = ComplexCustomer()
    restored_customer.restore_from_snapshot(snapshot)
    
    print(f"   ✓ Restored from snapshot:")
    print(f"     Customer: {restored_customer.name} ({restored_customer.email})")
    print(f"     Addresses: {len(restored_customer.addresses)}")
    print(f"     Loyalty points: {restored_customer.loyalty_profile['points']}")
    print(f"     Order references: {len(restored_customer.order_references)}")
    
    print("\n3. Complex aggregate relationships...")
    
    # Demonstrate complex relationships and state tracking
    print(f"   📊 Complex Customer State:")
    print(f"     Registration source: {customer.registration_source}")
    print(f"     Communication preferences: {customer.communication_preferences}")
    print(f"     Loyalty tier: {customer.loyalty_profile['tier']}")
    print(f"     State changes tracked: {len(customer.state_history)}")
    
    for change in customer.state_history[-3:]:  # Show last 3 changes
        print(f"       [{change['timestamp'][:19]}] {change['change_type']}")
    
    print("\n4. Temporal queries and time travel...")
    
    # Set up temporal query engine
    temporal_engine = TemporalQueryEngine()
    
    # Index customer states at different points
    base_time = datetime.now(timezone.utc)
    for i, version in enumerate([1, 3, 5, customer.version]):
        timestamp = (base_time + timedelta(minutes=i*10)).isoformat()
        
        # Simulate state at different versions
        simulated_state = {
            "version": version,
            "name": customer.name,
            "loyalty_points": min(version * 300, customer.loyalty_profile['points']),
            "addresses": len(customer.addresses) if version >= 3 else 0,
            "orders": min(version, len(customer.order_references))
        }
        
        temporal_engine.index_aggregate_state(customer.id, version, timestamp, simulated_state)
    
    # Query at specific time
    query_time = base_time + timedelta(minutes=25)
    state_at_time = temporal_engine.query_at_time(customer.id, query_time)
    
    print(f"   🕒 Temporal Query Results:")
    if state_at_time:
        print(f"     State at {query_time.strftime('%H:%M')}:")
        print(f"       Version: {state_at_time['version']}")
        print(f"       Loyalty points: {state_at_time['loyalty_points']}")
        print(f"       Addresses: {state_at_time['addresses']}")
    
    # Query changes in period
    period_start = base_time
    period_end = base_time + timedelta(minutes=30)
    changes = temporal_engine.query_changes_in_period(customer.id, period_start, period_end)
    
    print(f"     Changes in 30-minute period: {len(changes)}")
    for change in changes:
        print(f"       v{change['version']}: {change['state']['loyalty_points']} points")
    
    print("\n5. Advanced projection with materialization...")
    
    # Set up materialized view
    analytics_view = CustomerAnalyticsView()
    
    # Project events into view
    customer_events = [
        CustomerRegisteredV2(customer_id="alice", email="alice@example.com", name="Alice", 
                           registration_source="web", preferences={"newsletter": True}),
        CustomerRegisteredV2(customer_id="bob", email="bob@example.com", name="Bob", 
                           registration_source="mobile", preferences={"promotions": True}),
        CustomerRegisteredV2(customer_id="charlie", email="charlie@example.com", name="Charlie", 
                           registration_source="social", preferences={})
    ]
    
    for event in customer_events:
        analytics_view.project_event(event, event.customer_id)
    
    view_data = analytics_view.get_view_data()
    print(f"   📈 Customer Analytics View:")
    print(f"     Total customers: {view_data['total_customers']}")
    print(f"     Lifecycle segments: {view_data['segments']}")
    print(f"     Source breakdown: {view_data['source_breakdown']}")
    print(f"     View version: {view_data['view_version']}")
    
    print("\n6. Multi-tenant architecture...")
    
    # Set up multi-tenant event store
    mt_store = MultiTenantEventStore(event_store)
    
    # Create tenants
    tenants = [
        ("tenant-corp", "Corporate Inc"),
        ("tenant-startup", "Startup Co"),
        ("tenant-enterprise", "Enterprise Ltd")
    ]
    
    for tenant_id, tenant_name in tenants:
        tenant = mt_store.create_tenant(tenant_id, tenant_name)
        print(f"   ✓ Created tenant: {tenant_name} (ID: {tenant_id})")
        
        # Store some tenant-specific events
        for i in range(5):
            event = CustomerRegisteredV2(
                customer_id=f"customer-{i}",
                email=f"user{i}@{tenant_id}.com",
                name=f"User {i}",
                registration_source="web"
            )
            
            await mt_store.store_tenant_event(tenant_id, event, f"customer-{i}")
    
    # Show tenant metrics
    print(f"\n   📊 Multi-tenant Metrics:")
    for tenant_id, _ in tenants:
        metrics = mt_store.get_tenant_metrics(tenant_id)
        print(f"     {tenant_id}:")
        print(f"       Events stored: {metrics['events_stored']}")
        print(f"       Event utilization: {metrics['event_utilization']:.1f}%")
        print(f"       Last activity: {metrics['last_activity'][:19]}")
    
    print("\n7. Snapshot store performance analysis...")
    
    # Set up snapshot store and test performance
    snapshot_store = SnapshotStore()
    
    # Create multiple snapshots to test compression
    customers_data = []
    for i in range(10):
        test_customer = ComplexCustomer(f"perf-test-{i}")
        test_customer.register(f"test{i}@example.com", f"Test User {i}", f"+123456789{i}")
        
        # Add realistic data
        for j in range(3):
            test_customer.add_address("home", f"{j+1}00 Test St", "Test City", f"1234{j}", "US")
        
        test_customer.update_loyalty_points(i * 500, "Test points")
        
        for j in range(i + 1):
            test_customer.add_order_reference(f"order-{i}-{j}")
        
        customers_data.append(test_customer)
    
    # Create and analyze snapshots
    snapshot_sizes = []
    for customer in customers_data:
        snapshot = customer.create_snapshot()
        snapshot_store.save_snapshot(snapshot)
        
        # Analyze compression
        original_state = customer.__dict__.copy()
        original_json = json.dumps(original_state, default=str)
        original_size = len(original_json.encode())
        compressed_size = len(snapshot.compressed_state)
        
        compression_ratio = (1 - compressed_size / original_size) * 100
        snapshot_sizes.append({
            "customer_id": customer.id,
            "original_size": original_size,
            "compressed_size": compressed_size,
            "compression_ratio": compression_ratio
        })
    
    avg_compression = sum(s["compression_ratio"] for s in snapshot_sizes) / len(snapshot_sizes)
    total_original = sum(s["original_size"] for s in snapshot_sizes)
    total_compressed = sum(s["compressed_size"] for s in snapshot_sizes)
    
    print(f"   💾 Snapshot Performance Analysis:")
    print(f"     Snapshots created: {len(snapshot_sizes)}")
    print(f"     Total original size: {total_original:,} bytes")
    print(f"     Total compressed size: {total_compressed:,} bytes")
    print(f"     Average compression ratio: {avg_compression:.1f}%")
    print(f"     Space saved: {total_original - total_compressed:,} bytes")
    
    return {
        "migrator": migrator,
        "customer": customer,
        "snapshot": snapshot,
        "temporal_engine": temporal_engine,
        "analytics_view": analytics_view,
        "mt_store": mt_store,
        "snapshot_store": snapshot_store,
        "compression_ratio": avg_compression,
        "tenants_created": len(tenants)
    }

async def main():
    result = await demonstrate_advanced_patterns()
    
    print(f"\n✅ SUCCESS! Advanced patterns demonstrated!")
    
    print(f"\nAdvanced patterns covered:")
    print(f"- ✓ Event versioning and schema evolution with migrations")
    print(f"- ✓ Snapshot optimization with compression and integrity checks")
    print(f"- ✓ Complex aggregate relationships and state tracking")
    print(f"- ✓ Temporal queries and time travel capabilities")
    print(f"- ✓ Advanced projection patterns with materialized views")
    print(f"- ✓ Multi-tenant event sourcing architecture")
    print(f"- ✓ Performance optimization with compressed snapshots")
    
    print(f"\nSystem performance:")
    print(f"- Event migrations: Schema evolution handled")
    print(f"- Snapshot compression: {result['compression_ratio']:.1f}% average reduction")
    print(f"- Multi-tenancy: {result['tenants_created']} tenants with isolation")
    print(f"- Temporal queries: Historical state reconstruction")
    print(f"- Complex relationships: Cross-aggregate references managed")
    print(f"- Materialized views: Real-time analytics projections")

if __name__ == "__main__":
    asyncio.run(main())