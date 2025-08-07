#!/usr/bin/env python3
"""
Aggregate Lifecycle Example

This example demonstrates the complete lifecycle of aggregates in event sourcing:
- Aggregate creation, modification, and state transitions
- Event application and business rule enforcement
- Optimistic concurrency and versioning
- Aggregate state snapshots and reconstruction
"""

import asyncio
import sys
import os
from typing import ClassVar

# Add the python package to the path
sys.path.insert(
    0, os.path.join(os.path.dirname(__file__), "..", "eventuali-python", "python")
)

from eventuali import EventStore
from eventuali.aggregate import Aggregate
from eventuali.event import Event


class OrderCreated(Event):
    """Event fired when an order is created."""

    customer_id: str
    product_name: str
    quantity: int
    unit_price: float


class OrderItemAdded(Event):
    """Event fired when an item is added to an order."""

    product_name: str
    quantity: int
    unit_price: float


class OrderShipped(Event):
    """Event fired when an order is shipped."""

    tracking_number: str
    shipping_address: str


class OrderCancelled(Event):
    """Event fired when an order is cancelled."""

    reason: str


class OrderCompleted(Event):
    """Event fired when an order is completed."""

    completion_date: str


class OrderItem:
    """Value object representing an order item."""

    def __init__(self, product_name: str, quantity: int, unit_price: float):
        self.product_name = product_name
        self.quantity = quantity
        self.unit_price = unit_price

    @property
    def total_price(self) -> float:
        return self.quantity * self.unit_price


class Order(Aggregate):
    """Order aggregate demonstrating complex lifecycle management."""

    # Order states
    CREATED: ClassVar[str] = "created"
    SHIPPED: ClassVar[str] = "shipped"
    CANCELLED: ClassVar[str] = "cancelled"
    COMPLETED: ClassVar[str] = "completed"

    customer_id: str = ""
    status: str = "created"
    items: list = []
    tracking_number: str = ""
    shipping_address: str = ""
    cancellation_reason: str = ""
    total_amount: float = 0.0

    def __init__(self, **data):
        super().__init__(**data)
        if "items" not in data:
            self.items = []

    # Business methods
    def create_order(
        self, customer_id: str, product_name: str, quantity: int, unit_price: float
    ):
        """Create a new order with initial item."""
        if quantity <= 0:
            raise ValueError("Quantity must be positive")
        if unit_price < 0:
            raise ValueError("Unit price cannot be negative")

        event = OrderCreated(
            customer_id=customer_id,
            product_name=product_name,
            quantity=quantity,
            unit_price=unit_price,
        )
        self.apply(event)

    def add_item(self, product_name: str, quantity: int, unit_price: float):
        """Add an item to the order."""
        if self.status != self.CREATED:
            raise ValueError(f"Cannot add items to order in {self.status} status")
        if quantity <= 0:
            raise ValueError("Quantity must be positive")
        if unit_price < 0:
            raise ValueError("Unit price cannot be negative")

        event = OrderItemAdded(
            product_name=product_name, quantity=quantity, unit_price=unit_price
        )
        self.apply(event)

    def ship_order(self, tracking_number: str, shipping_address: str):
        """Ship the order."""
        if self.status != self.CREATED:
            raise ValueError(f"Cannot ship order in {self.status} status")
        if not tracking_number:
            raise ValueError("Tracking number is required")
        if not shipping_address:
            raise ValueError("Shipping address is required")
        if len(self.items) == 0:
            raise ValueError("Cannot ship empty order")

        event = OrderShipped(
            tracking_number=tracking_number, shipping_address=shipping_address
        )
        self.apply(event)

    def cancel_order(self, reason: str):
        """Cancel the order."""
        if self.status in [self.CANCELLED, self.COMPLETED]:
            raise ValueError(f"Cannot cancel order in {self.status} status")

        event = OrderCancelled(reason=reason)
        self.apply(event)

    def complete_order(self, completion_date: str):
        """Complete the order."""
        if self.status != self.SHIPPED:
            raise ValueError(f"Cannot complete order in {self.status} status")

        event = OrderCompleted(completion_date=completion_date)
        self.apply(event)

    # Event handlers
    def apply_order_created(self, event: OrderCreated):
        """Apply OrderCreated event."""
        self.customer_id = event.customer_id
        self.status = self.CREATED
        self.items = [OrderItem(event.product_name, event.quantity, event.unit_price)]
        self._recalculate_total()

    def apply_order_item_added(self, event: OrderItemAdded):
        """Apply OrderItemAdded event."""
        self.items.append(
            OrderItem(event.product_name, event.quantity, event.unit_price)
        )
        self._recalculate_total()

    def apply_order_shipped(self, event: OrderShipped):
        """Apply OrderShipped event."""
        self.status = self.SHIPPED
        self.tracking_number = event.tracking_number
        self.shipping_address = event.shipping_address

    def apply_order_cancelled(self, event: OrderCancelled):
        """Apply OrderCancelled event."""
        self.status = self.CANCELLED
        self.cancellation_reason = event.reason

    def apply_order_completed(self, event: OrderCompleted):
        """Apply OrderCompleted event."""
        self.status = self.COMPLETED

    def _recalculate_total(self):
        """Recalculate order total."""
        self.total_amount = sum(item.total_price for item in self.items)

    # Query methods
    def get_item_count(self) -> int:
        """Get total number of items."""
        return sum(item.quantity for item in self.items)

    def can_be_modified(self) -> bool:
        """Check if order can be modified."""
        return self.status == self.CREATED

    def is_final_state(self) -> bool:
        """Check if order is in a final state."""
        return self.status in [self.CANCELLED, self.COMPLETED]


async def demonstrate_lifecycle():
    """Demonstrate complete aggregate lifecycle."""
    print("=== Order Lifecycle Demonstration ===\n")

    # Setup
    event_store = await EventStore.create("sqlite://:memory:")
    print("1. Event store created")

    # Create order
    print("\n2. Creating new order...")
    order = Order(id="order-456")
    order.create_order("customer-123", "Laptop", 1, 999.99)

    print(f"   ✓ Order created for customer {order.customer_id}")
    print(f"   ✓ Status: {order.status}")
    print(
        f"   ✓ Initial item: {order.items[0].product_name} x{order.items[0].quantity}"
    )
    print(f"   ✓ Total: ${order.total_amount:.2f}")
    print(f"   ✓ Version: {order.version}")

    # Add more items
    print("\n3. Adding more items...")
    order.add_item("Mouse", 1, 25.99)
    order.add_item("Keyboard", 1, 75.00)

    print("   ✓ Added mouse and keyboard")
    print(f"   ✓ Total items: {order.get_item_count()}")
    print(f"   ✓ New total: ${order.total_amount:.2f}")
    print(f"   ✓ Version: {order.version}")
    print(f"   ✓ Can be modified: {order.can_be_modified()}")

    # Save progress
    await event_store.save(order)
    order.mark_events_as_committed()
    print("   ✓ Order saved to event store")

    # Ship order
    print("\n4. Shipping order...")
    order.ship_order("TRK123456789", "123 Main St, Anytown, ST 12345")

    print("   ✓ Order shipped")
    print(f"   ✓ Status: {order.status}")
    print(f"   ✓ Tracking: {order.tracking_number}")
    print(f"   ✓ Address: {order.shipping_address}")
    print(f"   ✓ Can be modified: {order.can_be_modified()}")

    # Try to add item after shipping (should fail)
    print("\n5. Testing business rules...")
    try:
        order.add_item("Cable", 1, 15.00)
        print("   ❌ Should not be able to add items after shipping")
    except ValueError as e:
        print(f"   ✓ Correctly rejected item addition: {e}")

    # Complete order
    print("\n6. Completing order...")
    order.complete_order("2024-01-15")

    print("   ✓ Order completed")
    print(f"   ✓ Status: {order.status}")
    print(f"   ✓ Is final state: {order.is_final_state()}")

    # Save final state
    await event_store.save(order)
    order.mark_events_as_committed()

    # Demonstrate event reconstruction
    print("\n7. Reconstructing order from events...")
    events = await event_store.load_events(order.id)
    print(f"   ✓ Loaded {len(events)} events")

    # Print event sequence
    for i, event in enumerate(events, 1):
        print(f"     Event {i}: {event.event_type} (v{event.aggregate_version})")

    return order, events


async def demonstrate_alternative_lifecycle():
    """Demonstrate alternative lifecycle - cancellation."""
    print("\n=== Alternative Lifecycle - Cancellation ===\n")

    event_store = await EventStore.create("sqlite://:memory:")

    # Create and cancel order
    print("1. Creating order to be cancelled...")
    order = Order(id="order-789")
    order.create_order("customer-456", "Phone", 1, 599.99)
    order.add_item("Case", 1, 29.99)

    print(f"   ✓ Order created with total ${order.total_amount:.2f}")

    # Cancel instead of shipping
    print("\n2. Cancelling order...")
    order.cancel_order("Customer changed mind")

    print("   ✓ Order cancelled")
    print(f"   ✓ Status: {order.status}")
    print(f"   ✓ Reason: {order.cancellation_reason}")
    print(f"   ✓ Is final state: {order.is_final_state()}")

    # Try to ship cancelled order (should fail)
    print("\n3. Testing cancellation business rules...")
    try:
        order.ship_order("TRK999", "Some Address")
        print("   ❌ Should not be able to ship cancelled order")
    except ValueError as e:
        print(f"   ✓ Correctly rejected shipping: {e}")

    await event_store.save(order)
    return order


async def main():
    print("=== Aggregate Lifecycle Example ===\n")

    # Demonstrate normal lifecycle
    completed_order, events = await demonstrate_lifecycle()

    # Demonstrate alternative lifecycle
    cancelled_order = await demonstrate_alternative_lifecycle()

    print("\n=== Summary ===")
    print(
        f"Completed order: {completed_order.id} - ${completed_order.total_amount:.2f} ({completed_order.status})"
    )
    print(
        f"Cancelled order: {cancelled_order.id} - ${cancelled_order.total_amount:.2f} ({cancelled_order.status})"
    )

    print("\n✅ SUCCESS! Aggregate lifecycle patterns demonstrated!")

    print("\nKey patterns demonstrated:")
    print("- ✓ Complex aggregate state management")
    print("- ✓ Business rule enforcement at state transitions")
    print("- ✓ Event-driven state changes")
    print("- ✓ Aggregate versioning and consistency")
    print("- ✓ Multiple lifecycle paths (completion vs cancellation)")
    print("- ✓ Value objects and calculated fields")
    print("- ✓ Invariant protection")


if __name__ == "__main__":
    asyncio.run(main())
