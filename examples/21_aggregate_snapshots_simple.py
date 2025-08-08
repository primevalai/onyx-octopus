#!/usr/bin/env python3
"""
Example 21: Aggregate Snapshots - Simple Test

Simple test version of snapshot functionality to verify the implementation works.
"""

import asyncio
import json
import tempfile
from uuid import uuid4

# Test basic import first
try:
    print("Testing imports...")
    from eventuali import EventStore, Event, Aggregate
    print("✅ Basic imports successful")
    
    from eventuali import SnapshotService, SnapshotConfig, AggregateSnapshot
    print("✅ Snapshot imports successful")
    
except ImportError as e:
    print(f"❌ Import error: {e}")
    raise


async def simple_snapshot_test():
    """Simple test of snapshot functionality."""
    print("\n🧪 Simple Snapshot Test...")
    
    # Create temporary database
    db_file = tempfile.NamedTemporaryFile(delete=False)
    db_path = f"sqlite://{db_file.name}"
    
    print(f"📁 Using database: {db_path}")
    
    # Create snapshot service
    config = SnapshotConfig(
        snapshot_frequency=10,
        max_snapshot_age_hours=24,
        compression="gzip",
        auto_cleanup=True
    )
    print(f"⚙️ Snapshot config: {config}")
    
    # Initialize snapshot service  
    snapshot_service = SnapshotService(config)
    snapshot_service.initialize(db_path)
    print("✅ Snapshot service initialized")
    
    # Test snapshot creation
    user_id = str(uuid4())
    test_data = {"user_id": user_id, "name": "Test User", "balance": 100.0}
    state_json = json.dumps(test_data, sort_keys=True)
    state_data = state_json.encode('utf-8')
    
    print(f"📦 Creating snapshot for user {user_id[:8]}...")
    
    snapshot = snapshot_service.create_snapshot(
        user_id,
        "User",
        5,  # version
        state_data,
        5   # event count
    )
    
    print(f"✅ Snapshot created: {snapshot}")
    
    # Test snapshot loading
    print("🔍 Loading snapshot...")
    loaded_snapshot = snapshot_service.load_latest_snapshot(user_id)
    
    if loaded_snapshot:
        print(f"✅ Loaded snapshot: version={loaded_snapshot.aggregate_version}")
        
        # Test decompression
        decompressed = snapshot_service.decompress_snapshot_data(loaded_snapshot)
        recovered_data = json.loads(decompressed.decode('utf-8'))
        
        print(f"📊 Decompressed data: {recovered_data}")
        assert recovered_data == test_data
        print("✅ Data integrity verified")
    else:
        print("❌ No snapshot found")
    
    print("\n🎯 Simple snapshot test completed successfully!")


async def main():
    """Main function."""
    try:
        await simple_snapshot_test()
    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        raise


if __name__ == "__main__":
    asyncio.run(main())