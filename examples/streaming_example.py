#!/usr/bin/env python3
"""
Example demonstrating event streaming and subscriptions with Eventuali.

This example shows how to:
1. Set up an event store
2. Create an event streamer
3. Subscribe to specific events
4. Publish events and receive them through the stream
5. Build projections that update as events arrive
"""

import asyncio
from datetime import datetime, timezone
from typing import Optional

from eventuali import EventStore, Event, EventStreamer, SubscriptionBuilder, Projection


class UserRegistered(Event):
    """Domain event for user registration."""

    def __init__(self, user_id: str, email: str, name: str):
        super().__init__(
            aggregate_id=user_id,
            aggregate_type="User",
            event_type="UserRegistered",
            data={
                "user_id": user_id,
                "email": email,
                "name": name,
                "registered_at": datetime.now(timezone.utc).isoformat(),
            },
        )


class UserEmailChanged(Event):
    """Domain event for email address changes."""

    def __init__(self, user_id: str, old_email: str, new_email: str):
        super().__init__(
            aggregate_id=user_id,
            aggregate_type="User",
            event_type="UserEmailChanged",
            data={
                "user_id": user_id,
                "old_email": old_email,
                "new_email": new_email,
                "changed_at": datetime.now(timezone.utc).isoformat(),
            },
        )


class UserProjection(Projection):
    """
    Projection that maintains a read model of user data.

    This demonstrates how projections can be used to build
    optimized read models from event streams.
    """

    def __init__(self):
        self.users = {}
        self.last_position: Optional[int] = None

    async def handle_event(self, event: Event) -> None:
        """Process events to update the user read model."""
        if event.event_type == "UserRegistered":
            self.users[event.data["user_id"]] = {
                "user_id": event.data["user_id"],
                "email": event.data["email"],
                "name": event.data["name"],
                "registered_at": event.data["registered_at"],
            }
            print(
                f"[Projection] User registered: {event.data['name']} ({event.data['email']})"
            )

        elif event.event_type == "UserEmailChanged":
            user_id = event.data["user_id"]
            if user_id in self.users:
                old_email = self.users[user_id]["email"]
                self.users[user_id]["email"] = event.data["new_email"]
                print(
                    f"[Projection] Email changed for user {user_id}: {old_email} -> {event.data['new_email']}"
                )

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
    """Main example function."""
    print("=== Eventuali Streaming Example ===\n")

    # 1. Set up event store
    print("1. Setting up event store...")
    event_store = EventStore()
    await event_store.create("sqlite://:memory:")
    print("   Event store created with SQLite backend\n")

    # 2. Create event streamer
    print("2. Creating event streamer...")
    streamer = EventStreamer(capacity=1000)
    print("   Event streamer created with 1000 event capacity\n")

    # 3. Create a projection
    print("3. Setting up user projection...")
    user_projection = UserProjection()
    print("   User projection ready\n")

    # 4. Subscribe to user events
    print("4. Creating subscription for User events...")
    subscription = (
        SubscriptionBuilder()
        .with_id("user-projection-subscription")
        .filter_by_aggregate_type("User")
        .build()
    )

    receiver = await streamer.subscribe(subscription)
    print(f"   Subscribed with ID: {subscription.id}\n")

    # 5. Start background task to process events
    async def process_events():
        """Background task to process events from the stream."""
        print("[Background] Starting event processor...")
        try:
            async for stream_event in receiver:
                await user_projection.handle_event(stream_event.event)
                await user_projection.set_last_processed_position(
                    stream_event.global_position
                )
                print(
                    f"[Background] Processed event at position {stream_event.global_position}"
                )
        except Exception as e:
            print(f"[Background] Event processing stopped: {e}")

    # Start the background processor
    processor_task = asyncio.create_task(process_events())

    # Give the processor a moment to start
    await asyncio.sleep(0.1)

    # 6. Create and save some events
    print("5. Creating and saving events...")

    # Register some users
    events = [
        UserRegistered("user-1", "alice@example.com", "Alice Smith"),
        UserRegistered("user-2", "bob@example.com", "Bob Johnson"),
        UserRegistered("user-3", "carol@example.com", "Carol Williams"),
    ]

    print("   Saving user registration events...")
    for event in events:
        await event_store.save_events([event])
        print(f"   Saved: {event.event_type} for {event.data['name']}")
        # Small delay to see the streaming in action
        await asyncio.sleep(0.1)

    # Change an email address
    email_change_event = UserEmailChanged(
        "user-1", "alice@example.com", "alice.smith@newcompany.com"
    )
    print("   Saving email change event...")
    await event_store.save_events([email_change_event])
    print(f"   Saved: {email_change_event.event_type} for user-1")

    # Give events time to be processed
    await asyncio.sleep(0.2)

    # 7. Query the projection
    print("\n6. Querying the user projection...")
    all_users = user_projection.get_all_users()
    print(f"   Total users in projection: {len(all_users)}")

    for user_id, user_data in all_users.items():
        print(
            f"   - {user_data['name']} ({user_data['email']}) - registered: {user_data['registered_at']}"
        )

    # 8. Show streaming statistics
    print("\n7. Streaming statistics...")
    global_position = await streamer.get_global_position()
    print(f"   Global stream position: {global_position}")

    user_stream_pos = await streamer.get_stream_position("user-1")
    print(f"   Stream position for user-1: {user_stream_pos}")

    projection_position = await user_projection.get_last_processed_position()
    print(f"   Last position processed by projection: {projection_position}")

    # 9. Test event filtering
    print("\n8. Testing event type filtering...")
    email_subscription = (
        SubscriptionBuilder()
        .with_id("email-changes-only")
        .filter_by_event_type("UserEmailChanged")
        .build()
    )

    email_receiver = await streamer.subscribe(email_subscription)
    print("   Subscribed to email change events only")

    # Create another email change
    another_email_change = UserEmailChanged(
        "user-2", "bob@example.com", "bob.johnson@newcompany.com"
    )
    await event_store.save_events([another_email_change])
    print("   Created another email change event")

    # Try to receive the filtered event
    try:
        filtered_event = await asyncio.wait_for(email_receiver.recv(), timeout=1.0)
        print(
            f"   Received filtered event: {filtered_event.event.event_type} for {filtered_event.event.data['user_id']}"
        )
    except asyncio.TimeoutError:
        print("   No filtered events received (this might indicate a timing issue)")

    # 10. Clean up
    print("\n9. Cleaning up...")
    await streamer.unsubscribe(subscription.id)
    await streamer.unsubscribe(email_subscription.id)
    processor_task.cancel()

    try:
        await processor_task
    except asyncio.CancelledError:
        pass

    print("   Unsubscribed and cleaned up\n")

    print("=== Example completed successfully! ===")
    print("\nThis example demonstrated:")
    print("- Event streaming with real-time subscriptions")
    print("- Building projections from event streams")
    print("- Event filtering by aggregate and event type")
    print("- Position tracking for reliable processing")
    print("- Integration between EventStore and EventStreamer")


if __name__ == "__main__":
    asyncio.run(main())
