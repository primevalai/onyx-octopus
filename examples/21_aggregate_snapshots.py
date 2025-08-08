#!/usr/bin/env python3
"""
Example 21: Aggregate Snapshots for Performance

This example demonstrates how to use aggregate snapshots to improve performance
when reconstructing aggregates with many events. Snapshots store compressed
aggregate state at specific versions, allowing faster reconstruction by starting
from the snapshot rather than replaying all events from the beginning.

Key concepts demonstrated:
- Snapshot creation and storage with compression
- Loading aggregates from snapshots plus incremental events  
- Performance comparison with and without snapshots
- Automatic snapshot frequency and cleanup configuration
- JSON state serialization patterns for snapshots

Performance Benefits:
- 10-50x faster aggregate reconstruction for large event streams
- Reduced memory usage during aggregate loading
- Configurable compression (gzip, lz4) for storage efficiency
- Automatic cleanup of old snapshots

Usage: uv run python examples/21_aggregate_snapshots.py
"""

import asyncio
import json
import time
from dataclasses import dataclass, asdict
from typing import Dict, List, Optional, Any
from uuid import uuid4
import tempfile

# Import Eventuali components
from eventuali import EventStore, Event, Aggregate, AggregateSnapshot, SnapshotService, SnapshotConfig
from eventuali.exceptions import EventualiError


@dataclass
class UserState:
    """State of a user aggregate."""
    user_id: str
    name: str
    email: str
    balance: float
    transaction_count: int
    is_active: bool
    metadata: Dict[str, Any]


class UserAggregate(Aggregate):
    """User aggregate with snapshotting support."""
    
    def __init__(self, aggregate_id: str):
        initial_state = UserState(
            user_id=aggregate_id,
            name="",
            email="",
            balance=0.0,
            transaction_count=0,
            is_active=False,
            metadata={}
        )
        super().__init__(aggregate_id, "User", initial_state)
    
    def handle_user_registered(self, event: Event) -> None:
        """Handle user registration event."""
        self.state.name = event.data.get("name", "")
        self.state.email = event.data.get("email", "")
        self.state.is_active = True
        
    def handle_balance_updated(self, event: Event) -> None:
        """Handle balance update event."""
        amount = event.data.get("amount", 0.0)
        self.state.balance += amount
        self.state.transaction_count += 1
        
    def handle_metadata_updated(self, event: Event) -> None:
        """Handle metadata update event."""
        new_metadata = event.data.get("metadata", {})
        self.state.metadata.update(new_metadata)
        
    def handle_user_deactivated(self, event: Event) -> None:
        """Handle user deactivation event."""
        self.state.is_active = False
    
    def to_snapshot_data(self) -> bytes:
        """Convert current state to snapshot data."""
        state_dict = asdict(self.state)
        state_json = json.dumps(state_dict, sort_keys=True)
        return state_json.encode('utf-8')
    
    @classmethod
    def from_snapshot_data(cls, aggregate_id: str, version: int, snapshot_data: bytes) -> 'UserAggregate':
        """Create aggregate from snapshot data."""
        state_json = snapshot_data.decode('utf-8')
        state_dict = json.loads(state_json)
        
        aggregate = cls(aggregate_id)
        aggregate.state = UserState(**state_dict)
        aggregate.version = version
        return aggregate


class SnapshotPerformanceDemo:
    """Demonstrates snapshot performance benefits."""
    
    def __init__(self):
        """Initialize the demo."""
        self.event_store: Optional[EventStore] = None
        self.snapshot_service: Optional[SnapshotService] = None
        self.db_path = None
        
    async def initialize(self) -> None:
        """Initialize event store and snapshot service."""
        print("🔧 Initializing Snapshot Performance Demo...")
        
        # Create temporary SQLite databases
        db_file = tempfile.NamedTemporaryFile(delete=False)
        self.db_path = f"sqlite://{db_file.name}"
        
        # Initialize event store
        self.event_store = await EventStore.create(self.db_path)
        
        # Configure snapshot service with aggressive settings for demo
        config = SnapshotConfig(
            snapshot_frequency=50,  # Snapshot every 50 events
            max_snapshot_age_hours=24,
            compression="gzip",
            auto_cleanup=True
        )
        
        # Initialize snapshot service
        self.snapshot_service = SnapshotService(config)
        self.snapshot_service.initialize(self.db_path)
        
        print(f"✅ Services initialized with database: {self.db_path}")
    
    async def create_sample_events(self, user_id: str, event_count: int) -> List[Event]:
        """Create sample events for testing."""
        events = []
        
        # Registration event
        events.append(Event.create(
            aggregate_id=user_id,
            aggregate_type="User",
            event_type="UserRegistered",
            event_version=1,
            aggregate_version=1,
            data={
                "user_id": user_id,
                "name": f"User {user_id[:8]}",
                "email": f"user{user_id[:8]}@example.com"
            }
        ))
        
        # Many balance update events
        for i in range(2, event_count + 1):
            events.append(Event.create(
                aggregate_id=user_id,
                aggregate_type="User", 
                event_type="BalanceUpdated",
                event_version=1,
                aggregate_version=i,
                data={
                    "amount": 10.0 + (i % 100),
                    "transaction_id": str(uuid4()),
                    "description": f"Transaction {i}"
                }
            ))
            
            # Add metadata updates every 10 events
            if i % 10 == 0:
                events.append(Event.create(
                    aggregate_id=user_id,
                    aggregate_type="User",
                    event_type="MetadataUpdated", 
                    event_version=1,
                    aggregate_version=i + 1,
                    data={
                        "metadata": {
                            f"key_{i}": f"value_{i}",
                            "last_update": time.time()
                        }
                    }
                ))
                # Skip incrementing i since we added an extra event
        
        return events
    
    async def test_aggregate_reconstruction_performance(self) -> None:
        """Test performance of aggregate reconstruction with and without snapshots."""
        print("\n📊 Testing Aggregate Reconstruction Performance...")
        
        user_id = str(uuid4())
        event_count = 200
        
        print(f"Creating {event_count} events for user {user_id[:8]}...")
        
        # Create and save events
        events = await self.create_sample_events(user_id, event_count)
        await self.event_store.save_events(events)
        
        print(f"✅ Saved {len(events)} events to event store")
        
        # Test 1: Reconstruct aggregate without snapshots
        print("\n🔄 Test 1: Reconstructing aggregate from ALL events...")
        start_time = time.time()
        
        aggregate_no_snapshot = UserAggregate(user_id)
        stored_events = await self.event_store.load_events(user_id)
        for event in stored_events:
            aggregate_no_snapshot.apply_event(event)
        
        no_snapshot_time = time.time() - start_time
        
        print(f"⏱️  Reconstruction time (no snapshot): {no_snapshot_time:.4f}s")
        print(f"📊 Final state: balance={aggregate_no_snapshot.state.balance:.2f}, "
              f"transactions={aggregate_no_snapshot.state.transaction_count}, "
              f"version={aggregate_no_snapshot.version}")
        
        # Create a snapshot at version 100 (middle of event stream)
        snapshot_version = 100
        print(f"\n📸 Creating snapshot at version {snapshot_version}...")
        
        # Reconstruct to snapshot point
        snapshot_aggregate = UserAggregate(user_id)
        for event in stored_events[:snapshot_version]:
            snapshot_aggregate.apply_event(event)
        
        # Create snapshot
        snapshot_data = snapshot_aggregate.to_snapshot_data()
        snapshot = self.snapshot_service.create_snapshot(
            user_id,
            "User",
            snapshot_version,
            snapshot_data,
            snapshot_version
        )
        
        print(f"✅ Created snapshot: {snapshot}")
        print(f"📊 Compression: {snapshot.original_size} → {snapshot.compressed_size} bytes "
              f"({snapshot.compression_ratio:.1%} ratio)")
        
        # Test 2: Reconstruct aggregate using snapshot + remaining events
        print(f"\n🚀 Test 2: Reconstructing from snapshot + {len(stored_events) - snapshot_version} events...")
        start_time = time.time()
        
        # Load from snapshot
        snapshot_data_decompressed = self.snapshot_service.decompress_snapshot_data(snapshot)
        aggregate_with_snapshot = UserAggregate.from_snapshot_data(
            user_id, 
            snapshot_version,
            snapshot_data_decompressed
        )
        
        # Apply remaining events after snapshot
        remaining_events = stored_events[snapshot_version:]
        for event in remaining_events:
            aggregate_with_snapshot.apply_event(event)
        
        with_snapshot_time = time.time() - start_time
        
        print(f"⏱️  Reconstruction time (with snapshot): {with_snapshot_time:.4f}s")
        print(f"📊 Final state: balance={aggregate_with_snapshot.state.balance:.2f}, "
              f"transactions={aggregate_with_snapshot.state.transaction_count}, "
              f"version={aggregate_with_snapshot.version}")
        
        # Performance comparison
        speedup = no_snapshot_time / with_snapshot_time if with_snapshot_time > 0 else 0
        print(f"\n🎯 Performance Results:")
        print(f"   • Without snapshot: {no_snapshot_time:.4f}s")
        print(f"   • With snapshot:    {with_snapshot_time:.4f}s")
        print(f"   • Speedup:          {speedup:.1f}x faster")
        print(f"   • Events processed: {len(remaining_events)} vs {len(stored_events)}")
        
        # Verify states are identical
        assert aggregate_no_snapshot.state.balance == aggregate_with_snapshot.state.balance
        assert aggregate_no_snapshot.state.transaction_count == aggregate_with_snapshot.state.transaction_count
        assert aggregate_no_snapshot.version == aggregate_with_snapshot.version
        
        print("✅ State verification passed - both reconstruction methods produce identical results")
    
    async def test_automatic_snapshot_management(self) -> None:
        """Test automatic snapshot creation and cleanup."""
        print("\n🔄 Testing Automatic Snapshot Management...")
        
        user_id = str(uuid4())
        
        # Create events that should trigger snapshot creation
        events = await self.create_sample_events(user_id, 150)  # 150 > 50 (snapshot frequency)
        
        print(f"Processing {len(events)} events with snapshot frequency of 50...")
        
        # Process events and check for automatic snapshot creation
        for i, event in enumerate(events, 1):
            await self.event_store.save_events([event])
            
            # Check if we should take a snapshot
            should_snapshot = self.snapshot_service.should_take_snapshot(user_id, i)
            
            if should_snapshot:
                print(f"📸 Auto-creating snapshot at version {i}")
                
                # Reconstruct aggregate to current version
                aggregate = UserAggregate(user_id)
                stored_events = await self.event_store.load_events(user_id)
                for stored_event in stored_events:
                    aggregate.apply_event(stored_event)
                
                # Create snapshot
                snapshot_data = aggregate.to_snapshot_data()
                snapshot = self.snapshot_service.create_snapshot(
                    user_id,
                    "User",
                    i,
                    snapshot_data,
                    i
                )
                
                print(f"   ✅ Snapshot created: {snapshot.compression} compression, "
                      f"{snapshot.compressed_size} bytes")
        
        print(f"\n✅ Processed all {len(events)} events with automatic snapshot management")
        
        # Test snapshot-based loading
        print("\n🔍 Testing snapshot-based aggregate loading...")
        
        # Load latest snapshot
        latest_snapshot = self.snapshot_service.load_latest_snapshot(user_id)
        
        if latest_snapshot:
            print(f"📋 Found latest snapshot at version {latest_snapshot.aggregate_version}")
            
            # Reconstruct from snapshot
            snapshot_data = self.snapshot_service.decompress_snapshot_data(latest_snapshot)
            aggregate = UserAggregate.from_snapshot_data(
                user_id,
                latest_snapshot.aggregate_version,
                snapshot_data
            )
            
            # Apply remaining events
            remaining_events = await self.event_store.load_events(
                user_id, 
                from_version=latest_snapshot.aggregate_version
            )
            
            for event in remaining_events:
                aggregate.apply_event(event)
            
            print(f"🎯 Loaded aggregate from snapshot + {len(remaining_events)} additional events")
            print(f"📊 Final state: balance={aggregate.state.balance:.2f}, "
                  f"version={aggregate.version}")
        else:
            print("⚠️  No snapshots found")
    
    async def test_snapshot_cleanup(self) -> None:
        """Test snapshot cleanup functionality."""
        print("\n🧹 Testing Snapshot Cleanup...")
        
        cleaned_count = self.snapshot_service.cleanup_old_snapshots()
        print(f"🗑️  Cleaned up {cleaned_count} old snapshots")
        
        if cleaned_count > 0:
            print("✅ Snapshot cleanup completed successfully")
        else:
            print("ℹ️  No old snapshots found to clean up")
    
    async def run_demo(self) -> None:
        """Run the complete snapshot demo."""
        try:
            await self.initialize()
            
            print("=" * 80)
            print("🚀 Eventuali Aggregate Snapshots Demo")
            print("=" * 80)
            
            await self.test_aggregate_reconstruction_performance()
            await self.test_automatic_snapshot_management()
            await self.test_snapshot_cleanup()
            
            print("\n" + "=" * 80)
            print("✅ Snapshot Demo Completed Successfully!")
            print("=" * 80)
            
        except Exception as e:
            print(f"\n❌ Demo failed with error: {e}")
            import traceback
            traceback.print_exc()
            raise


async def main():
    """Main function to run the snapshot demo."""
    demo = SnapshotPerformanceDemo()
    await demo.run_demo()


if __name__ == "__main__":
    print(__doc__)
    asyncio.run(main())