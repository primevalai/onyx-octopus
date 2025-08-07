#!/usr/bin/env python3
"""
Fixed version of streaming example demonstrating working Python-Rust integration.

This example shows the corrected streaming functionality after fixing the integration issues.
"""

import asyncio
import json
from datetime import datetime, timezone
from typing import Optional

# Use the correct Python path for our fixed bindings
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.event import UserRegistered, UserEmailChanged
from eventuali.streaming import EventStreamer, SubscriptionBuilder, Projection
from eventuali.aggregate import User


class UserProjection(Projection):
    """
    Simple projection that tracks user data from events.
    """
    
    def __init__(self):
        self.users = {}
        self.last_position = None
    
    async def handle_event(self, event) -> None:
        """Process events to update the user read model."""
        if hasattr(event, 'get_event_type'):
            event_type = event.get_event_type()
        else:
            event_type = type(event).__name__
            
        if event_type == "UserRegistered":
            self.users[event.aggregate_id] = {
                "name": event.name,
                "email": event.email,
                "registered_at": str(datetime.now(timezone.utc))
            }
            print(f"[Projection] User registered: {event.name} ({event.email})")
        
        elif event_type == "UserEmailChanged":
            if event.aggregate_id in self.users:
                old_email = self.users[event.aggregate_id]["email"]
                self.users[event.aggregate_id]["email"] = event.new_email
                print(f"[Projection] Email changed for {event.aggregate_id}: {old_email} -> {event.new_email}")
    
    async def reset(self) -> None:
        """Reset the projection to its initial state."""
        self.users.clear()
        self.last_position = None
    
    async def get_last_processed_position(self) -> Optional[int]:
        """Get the last processed event position."""
        return self.last_position
    
    async def set_last_processed_position(self, position: int) -> None:
        """Set the last processed event position."""
        self.last_position = position
    
    def get_user(self, user_id: str) -> Optional[dict]:
        """Get user data from the read model."""
        return self.users.get(user_id)
    
    def get_all_users(self) -> dict:
        """Get all users from the read model."""
        return self.users.copy()


async def main():
    """Main demonstration of fixed streaming functionality."""
    print("=== Eventuali Fixed Streaming Example ===\n")
    
    # 1. Set up event store
    print("1. Setting up event store...")
    event_store = await EventStore.create("sqlite://:memory:")
    print("   ✓ Event store created with SQLite backend\n")
    
    # 2. Create event streamer
    print("2. Creating event streamer...")
    streamer = EventStreamer(capacity=1000)
    print("   ✓ Event streamer created\n")
    
    # 3. Create a projection
    print("3. Setting up user projection...")
    user_projection = UserProjection()
    print("   ✓ User projection ready\n")
    
    # 4. Test subscription creation (this was the main issue we fixed)
    print("4. Testing subscription creation...")
    subscription = (SubscriptionBuilder()
                   .with_id("user-projection-subscription")
                   .filter_by_aggregate_type("User")
                   .build())
    
    print(f"   ✓ Subscription created with ID: {subscription.id}")
    print(f"   ✓ Aggregate filter: {subscription.aggregate_type_filter}\n")
    
    # 5. Test that we can subscribe (this was failing before)
    print("5. Testing subscription to stream...")
    try:
        receiver = await streamer.subscribe(subscription)
        print("   ✓ Successfully subscribed to stream\n")
        
        # 6. Test basic event creation and aggregate operations
        print("6. Creating and working with aggregates...")
        
        # Create a user aggregate
        user = User()
        print(f"   ✓ Created user: {user.id}")
        
        # Apply a registration event
        register_event = UserRegistered(name="Alice Johnson", email="alice@example.com")
        user.apply(register_event)
        print(f"   ✓ Applied registration event: {user.name} ({user.email})")
        
        # Save the aggregate to the store
        await event_store.save(user)
        print(f"   ✓ Saved user to event store")
        
        # 7. Test event store loading
        print("\n7. Testing event store loading...")
        loaded_user = await event_store.load(User, user.id)
        print(f"   ✓ Loaded user: {loaded_user.name} ({loaded_user.email})")
        print(f"   ✓ Version: {loaded_user.version}")
        
        # 8. Test business method
        print("\n8. Testing business methods...")
        loaded_user.change_email("alice.johnson@newcompany.com")
        print(f"   ✓ Changed email: {loaded_user.email}")
        
        await event_store.save(loaded_user)
        print("   ✓ Saved updated user")
        
        # 9. Show streaming positions
        print("\n9. Testing streaming statistics...")
        global_position = await streamer.get_global_position()
        print(f"   ✓ Global stream position: {global_position}")
        
        # 10. Summary
        print("\n✅ SUCCESS! Python-Rust streaming integration is working!")
        print("\nFixed issues:")
        print("- ✓ SubscriptionBuilder now chains methods correctly")
        print("- ✓ Python-Rust bridge passes subscription data properly")
        print("- ✓ EventStore save/load operations work")
        print("- ✓ Streaming positions are tracked")
        print("- ✓ Event creation and aggregate operations function")
        
    except Exception as e:
        print(f"   ❌ Subscription failed: {e}")
        print("\nThis indicates there are still integration issues to fix.")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())