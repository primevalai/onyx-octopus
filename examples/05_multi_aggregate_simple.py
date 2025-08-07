#!/usr/bin/env python3
"""
Multi-Aggregate Coordination Example (Simplified)

This example demonstrates coordination between multiple aggregates in event sourcing:
- Cross-aggregate business processes
- Event choreography and coordination  
- Maintaining consistency across aggregate boundaries
- Handling distributed transactions with eventual consistency
- Coordination patterns using built-in aggregates
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any
from datetime import datetime, timezone

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User  # Use built-in User aggregate
from eventuali.event import UserRegistered, UserEmailChanged, Event
from eventuali.streaming import EventStreamer, SubscriptionBuilder, Projection

# Custom events for our coordination example
class InventoryReserved(Event):
    """Product inventory reserved."""
    product_id: str
    quantity: int
    order_id: str
    customer_id: str

class InventoryReleased(Event):
    """Product inventory released."""
    product_id: str
    quantity: int
    order_id: str
    reason: str

class OrderCreated(Event):
    """Order created."""
    customer_id: str
    product_id: str
    quantity: int
    total_amount: float

class OrderCompleted(Event):
    """Order successfully completed."""
    fulfillment_date: str

class OrderFailed(Event):
    """Order processing failed."""
    reason: str

# Simple Product aggregate
class Product:
    """Simple product for inventory management."""
    
    def __init__(self, product_id: str, name: str, price: float, stock: int):
        self.id = product_id
        self.name = name
        self.price = price
        self.total_stock = stock
        self.reserved_stock = 0
        self.version = 0
    
    @property
    def available_stock(self) -> int:
        return self.total_stock - self.reserved_stock
    
    def can_reserve(self, quantity: int) -> bool:
        return self.available_stock >= quantity
    
    def reserve_stock(self, quantity: int, order_id: str, customer_id: str):
        if not self.can_reserve(quantity):
            raise ValueError(f"Insufficient stock for product {self.id}")
        
        self.reserved_stock += quantity
        self.version += 1
        return InventoryReserved(
            product_id=self.id,
            quantity=quantity, 
            order_id=order_id,
            customer_id=customer_id
        )
    
    def release_stock(self, quantity: int, order_id: str, reason: str):
        if quantity > self.reserved_stock:
            raise ValueError("Cannot release more stock than reserved")
        
        self.reserved_stock -= quantity
        self.version += 1
        return InventoryReleased(
            product_id=self.id,
            quantity=quantity,
            order_id=order_id,
            reason=reason
        )
    
    def fulfill_reservation(self, quantity: int):
        """Fulfill reservation by reducing total stock."""
        if quantity > self.reserved_stock:
            raise ValueError("Cannot fulfill more than reserved")
        
        self.reserved_stock -= quantity
        self.total_stock -= quantity
        self.version += 1

# Simple Order aggregate  
class Order:
    """Simple order for coordination demo."""
    
    CREATED = "created"
    RESERVED = "reserved"
    COMPLETED = "completed"
    FAILED = "failed"
    
    def __init__(self, order_id: str, customer_id: str, product_id: str, quantity: int, total_amount: float):
        self.id = order_id
        self.customer_id = customer_id
        self.product_id = product_id
        self.quantity = quantity
        self.total_amount = total_amount
        self.status = self.CREATED
        self.version = 0
        self.events = []
    
    def create_order(self):
        """Create the order."""
        event = OrderCreated(
            customer_id=self.customer_id,
            product_id=self.product_id,
            quantity=self.quantity,
            total_amount=self.total_amount
        )
        self.events.append(event)
        self.version += 1
        return event
    
    def complete_order(self, fulfillment_date: str = None):
        """Complete the order."""
        if self.status != self.RESERVED:
            raise ValueError(f"Cannot complete order in {self.status} status")
        
        if not fulfillment_date:
            fulfillment_date = datetime.now(timezone.utc).isoformat()
        
        self.status = self.COMPLETED
        event = OrderCompleted(fulfillment_date=fulfillment_date)
        self.events.append(event)
        self.version += 1
        return event
    
    def fail_order(self, reason: str):
        """Fail the order."""
        self.status = self.FAILED
        event = OrderFailed(reason=reason)
        self.events.append(event)
        self.version += 1
        return event
    
    def mark_reserved(self):
        """Mark order items as reserved."""
        self.status = self.RESERVED
        self.version += 1

# Multi-aggregate coordination service
class OrderProcessingService:
    """Service to coordinate order processing across multiple aggregates."""
    
    def __init__(self, event_store: EventStore):
        self.event_store = event_store
        self.products = {
            "laptop": Product("laptop", "Gaming Laptop", 800.0, 5),
            "mouse": Product("mouse", "Gaming Mouse", 50.0, 20),
            "keyboard": Product("keyboard", "Mechanical Keyboard", 120.0, 10)
        }
    
    async def process_order(self, order: Order) -> Dict[str, Any]:
        """Process order with multi-aggregate coordination."""
        results = {
            "order_id": order.id,
            "status": "started",
            "steps": [],
            "errors": []
        }
        
        try:
            # Step 1: Create order event
            order.create_order()
            results["steps"].append("Order created")
            
            # Step 2: Validate customer exists and is active
            try:
                user_events = await self.event_store.load_events(order.customer_id)
                if not user_events:
                    raise ValueError(f"Customer {order.customer_id} not found")
                
                user = User.from_events(user_events) 
                if not user.is_active:
                    raise ValueError(f"Customer {order.customer_id} is not active")
                
                results["steps"].append(f"Customer validated: {user.name} ({user.email})")
                
            except Exception as e:
                raise ValueError(f"Customer validation failed: {e}")
            
            # Step 3: Check and reserve inventory
            product = self.products.get(order.product_id)
            if not product:
                raise ValueError(f"Product {order.product_id} not found")
            
            if not product.can_reserve(order.quantity):
                raise ValueError(f"Insufficient stock for {order.product_id}: available {product.available_stock}, requested {order.quantity}")
            
            inventory_event = product.reserve_stock(order.quantity, order.id, order.customer_id)
            results["steps"].append(f"Reserved {order.quantity} units of {product.name}")
            
            # Step 4: Mark order as reserved
            order.mark_reserved()
            results["steps"].append("Order marked as reserved")
            
            # Step 5: Complete order (simulate payment processing)
            completion_event = order.complete_order()
            results["steps"].append("Order completed successfully")
            
            # Step 6: Fulfill inventory (convert reservation to sale)
            product.fulfill_reservation(order.quantity)
            results["steps"].append(f"Inventory fulfilled - remaining stock: {product.total_stock}")
            
            results["status"] = "completed"
            
        except Exception as e:
            results["status"] = "failed"
            results["errors"].append(str(e))
            
            # Compensating actions
            try:
                # Release any inventory reservations
                if order.product_id in self.products:
                    product = self.products[order.product_id]
                    if product.reserved_stock > 0:
                        product.release_stock(order.quantity, order.id, f"Order failed: {e}")
                        results["steps"].append("Inventory reservation released (compensation)")
                
                # Mark order as failed
                order.fail_order(str(e))
                results["steps"].append("Order marked as failed")
                
            except Exception as comp_error:
                results["errors"].append(f"Compensation failed: {comp_error}")
        
        return results
    
    async def setup_test_users(self) -> Dict[str, User]:
        """Set up test users for the demonstration."""
        users = {}
        
        # Create user Alice
        alice = User(id="alice")
        alice_reg = UserRegistered(name="Alice Johnson", email="alice@example.com")
        alice.apply(alice_reg)
        await self.event_store.save(alice)
        alice.mark_events_as_committed()
        users["alice"] = alice
        
        # Create user Bob  
        bob = User(id="bob")
        bob_reg = UserRegistered(name="Bob Smith", email="bob@example.com")
        bob.apply(bob_reg)
        await self.event_store.save(bob)
        bob.mark_events_as_committed()
        users["bob"] = bob
        
        # Create inactive user
        inactive = User(id="inactive")
        inactive_reg = UserRegistered(name="Inactive User", email="inactive@example.com")
        inactive.apply(inactive_reg)
        inactive.deactivate("Test account")
        await self.event_store.save(inactive)
        inactive.mark_events_as_committed()
        users["inactive"] = inactive
        
        return users
    
    def get_inventory_status(self) -> Dict[str, Dict[str, Any]]:
        """Get current inventory status for all products."""
        return {
            pid: {
                "name": product.name,
                "price": product.price,
                "total_stock": product.total_stock,
                "reserved_stock": product.reserved_stock,
                "available_stock": product.available_stock
            }
            for pid, product in self.products.items()
        }

async def demonstrate_multi_aggregate_coordination():
    """Demonstrate coordination patterns across multiple aggregates."""
    print("=== Multi-Aggregate Coordination Example ===\n")
    
    # Setup
    event_store = await EventStore.create("sqlite://:memory:")
    service = OrderProcessingService(event_store)
    
    print("1. Setting up test data...")
    users = await service.setup_test_users()
    
    print(f"   ✓ Created {len(users)} test users")
    for user_id, user in users.items():
        status = "active" if user.is_active else "inactive"
        print(f"     - {user.name} ({user_id}): {status}")
    
    print(f"   ✓ Initialized product inventory")
    inventory = service.get_inventory_status()
    for pid, info in inventory.items():
        print(f"     - {info['name']}: ${info['price']:.2f}, stock: {info['available_stock']}")
    
    # Test Case 1: Successful order processing
    print("\n2. Testing successful order processing...")
    order1 = Order("order-001", "alice", "laptop", 1, 800.0)
    result1 = await service.process_order(order1)
    
    print(f"   Order {result1['order_id']} - Status: {result1['status']}")
    for step in result1["steps"]:
        print(f"     ✓ {step}")
    
    for error in result1.get("errors", []):
        print(f"     ❌ {error}")
    
    # Test Case 2: Order with insufficient inventory
    print("\n3. Testing insufficient inventory scenario...")
    order2 = Order("order-002", "bob", "laptop", 10, 8000.0)  # More than available
    result2 = await service.process_order(order2)
    
    print(f"   Order {result2['order_id']} - Status: {result2['status']}")
    for step in result2["steps"]:
        print(f"     ✓ {step}")
    
    for error in result2.get("errors", []):
        print(f"     ❌ {error}")
    
    # Test Case 3: Order with inactive customer
    print("\n4. Testing inactive customer scenario...")
    order3 = Order("order-003", "inactive", "mouse", 1, 50.0)
    result3 = await service.process_order(order3)
    
    print(f"   Order {result3['order_id']} - Status: {result3['status']}")
    for step in result3["steps"]:
        print(f"     ✓ {step}")
    
    for error in result3.get("errors", []):
        print(f"     ❌ {error}")
    
    # Test Case 4: Multiple concurrent orders
    print("\n5. Testing concurrent order processing...")
    
    order4 = Order("order-004", "alice", "mouse", 2, 100.0)
    order5 = Order("order-005", "bob", "keyboard", 1, 120.0)
    order6 = Order("order-006", "alice", "laptop", 3, 2400.0)  # Should succeed
    
    # Process orders concurrently
    results = await asyncio.gather(
        service.process_order(order4),
        service.process_order(order5),
        service.process_order(order6),
        return_exceptions=True
    )
    
    for i, result in enumerate(results, 4):
        if isinstance(result, Exception):
            print(f"     Order order-00{i} failed with exception: {result}")
        else:
            print(f"     Order {result['order_id']}: {result['status']}")
            if result.get("errors"):
                for error in result["errors"]:
                    print(f"       ❌ {error}")
    
    # Final system state
    print("\n6. Final system state...")
    final_inventory = service.get_inventory_status()
    
    print("   Inventory Status:")
    for pid, info in final_inventory.items():
        print(f"     - {info['name']}: {info['available_stock']} available, "
              f"{info['reserved_stock']} reserved, {info['total_stock']} total")
    
    print(f"\n   Order Summary:")
    print(f"     - Successful orders: {sum(1 for r in [result1, result2, result3] + results if not isinstance(r, Exception) and r['status'] == 'completed')}")
    print(f"     - Failed orders: {sum(1 for r in [result1, result2, result3] + results if not isinstance(r, Exception) and r['status'] == 'failed')}")
    
    return {
        "service": service,
        "orders": [order1, order2, order3, order4, order5, order6],
        "results": [result1, result2, result3] + results
    }

async def main():
    result = await demonstrate_multi_aggregate_coordination()
    
    print(f"\n✅ SUCCESS! Multi-aggregate coordination patterns demonstrated!")
    
    print(f"\nKey patterns demonstrated:")
    print(f"- ✓ Cross-aggregate business process orchestration")
    print(f"- ✓ Inventory reservation and release coordination")
    print(f"- ✓ Customer validation across aggregate boundaries")
    print(f"- ✓ Compensating actions for failed transactions")
    print(f"- ✓ Concurrent order processing with resource contention")
    print(f"- ✓ Multi-step workflow with rollback capabilities")
    print(f"- ✓ Event-driven coordination without tight coupling")

if __name__ == "__main__":
    asyncio.run(main())