#!/usr/bin/env python3
"""
Debug script to examine event serialization issues.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali.event import UserRegistered
from eventuali.aggregate import User
import json

def main():
    print("=== Event Serialization Debug ===\n")
    
    # 1. Create a user and event
    user = User()
    event = UserRegistered(name="Alice Johnson", email="alice@example.com")
    
    print("1. Event before applying to aggregate:")
    print(f"   Event type: {type(event).__name__}")
    print(f"   Event data: {event}")
    print()
    
    # 2. Apply event to aggregate
    user.apply(event)
    print("2. After applying to aggregate:")
    print(f"   User: {user.name} ({user.email})")
    print(f"   Uncommitted events: {len(user.get_uncommitted_events())}")
    print()
    
    # 3. Check event serialization
    events = user.get_uncommitted_events()
    for i, evt in enumerate(events):
        print(f"3. Event {i} serialization:")
        print(f"   Type: {type(evt).__name__}")
        print(f"   Has model_dump: {hasattr(evt, 'model_dump')}")
        if hasattr(evt, 'model_dump'):
            try:
                event_dict = evt.model_dump()
                print(f"   model_dump(): {json.dumps(event_dict, indent=2, default=str)}")
            except Exception as e:
                print(f"   model_dump() error: {e}")
        
        print(f"   aggregate_id: {getattr(evt, 'aggregate_id', 'NOT SET')}")
        print(f"   aggregate_type: {getattr(evt, 'aggregate_type', 'NOT SET')}")
        print(f"   event_type: {getattr(evt, 'event_type', 'NOT SET')}")
        print()

if __name__ == "__main__":
    main()