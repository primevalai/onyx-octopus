#!/usr/bin/env python3
"""
Custom Event Subclasses Example

This example demonstrates how to define and use custom Event subclasses with 
additional fields, and how to register them with the EventStore for proper
deserialization. This solves the issue where custom fields are lost during
event retrieval.
"""

import asyncio
import sys
import os
from typing import Dict, Any, Optional
from datetime import datetime

# Add the python package to the path
sys.path.insert(
    0, os.path.join(os.path.dirname(__file__), "..", "eventuali-python", "python")
)

from eventuali import EventStore, Event, Aggregate


# Define custom event subclasses with additional fields
class AgentEvent(Event):
    """Event class specifically for agent lifecycle and actions."""
    
    # Agent-specific fields
    agent_name: str = ""
    agent_id: str = ""
    parent_agent_id: str = ""
    workflow_id: str = ""
    event_name: str = ""
    attributes: Dict[str, Any] = {}
    
    def get_event_type(self) -> str:
        return "AgentEvent"


class OrderEvent(Event):
    """Event class for e-commerce order operations."""
    
    # Order-specific fields
    order_id: str = ""
    customer_id: str = ""
    order_total: float = 0.0
    currency: str = "USD"
    items: list = []
    shipping_address: Dict[str, str] = {}
    
    def get_event_type(self) -> str:
        return "OrderEvent"


class PaymentEvent(Event):
    """Event class for payment processing."""
    
    # Payment-specific fields
    payment_id: str = ""
    order_id: str = ""
    amount: float = 0.0
    currency: str = "USD"
    payment_method: str = ""
    status: str = ""
    transaction_id: Optional[str] = None
    
    def get_event_type(self) -> str:
        return "PaymentEvent"


# Define custom aggregates that use these events
class AgentAggregate(Aggregate):
    """Aggregate for managing agent state."""
    
    # Additional fields for the aggregate
    agent_name: str = ""
    status: str = "inactive"
    current_workflow: str = ""
    
    def get_aggregate_type(self) -> str:
        return "agent_aggregate"
    
    def command_received(self, command: str, workflow_id: str, agent_name: str):
        """Handle a command received by the agent."""
        event = AgentEvent(
            aggregate_id=self.id,
            aggregate_type=self.get_aggregate_type(),
            agent_name=agent_name,
            agent_id=self.id,
            workflow_id=workflow_id,
            event_name="agent.commandReceived",
            attributes={
                "command": command,
                "received_at": datetime.utcnow().isoformat()
            }
        )
        self.apply(event)
    
    def apply_agent_event(self, event: Event):
        """Apply AgentEvent events to update aggregate state."""
        if isinstance(event, AgentEvent):
            self.agent_name = event.agent_name
            self.status = "active"
            self.current_workflow = event.workflow_id


class OrderAggregate(Aggregate):
    """Aggregate for managing order state."""
    
    # Additional fields for the aggregate
    customer_id: str = ""
    total: float = 0.0
    status: str = "pending"
    
    def get_aggregate_type(self) -> str:
        return "order_aggregate"
    
    def place_order(self, customer_id: str, total: float, items: list):
        """Place a new order."""
        event = OrderEvent(
            aggregate_id=self.id,
            aggregate_type=self.get_aggregate_type(),
            order_id=self.id,
            customer_id=customer_id,
            order_total=total,
            items=items,
            shipping_address={
                "street": "123 Main St",
                "city": "Anytown",
                "state": "CA",
                "zip": "12345"
            }
        )
        self.apply(event)
    
    def apply_order_event(self, event: Event):
        """Apply OrderEvent events to update aggregate state."""
        if isinstance(event, OrderEvent):
            self.customer_id = event.customer_id
            self.total = event.order_total
            self.status = "placed"


async def main():
    print("=== Custom Event Subclasses Example ===\n")

    # 1. Create event store
    print("1. Creating event store...")
    event_store = await EventStore.create("sqlite://:memory:")
    print("   ✓ Event store created with in-memory SQLite database")

    # 2. Register custom event classes
    print("\n2. Registering custom event classes...")
    EventStore.register_event_class("AgentEvent", AgentEvent)
    EventStore.register_event_class("OrderEvent", OrderEvent)
    EventStore.register_event_class("PaymentEvent", PaymentEvent)
    
    registered_classes = EventStore.get_registered_event_classes()
    print(f"   ✓ Registered {len(registered_classes)} event classes:")
    for event_type, event_class in registered_classes.items():
        print(f"     - {event_type} -> {event_class.__name__}")

    # 3. Create and save agent events
    print("\n3. Creating agent events...")
    agent = AgentAggregate(id="agent-123")
    agent.command_received(
        command="analyze market trends",
        workflow_id="workflow-456",
        agent_name="MarketAnalyzer"
    )
    
    # Add another event with different attributes
    enhanced_event = AgentEvent(
        aggregate_id=agent.id,
        aggregate_type=agent.get_aggregate_type(),
        agent_name="MarketAnalyzer",
        agent_id="agent-123",
        workflow_id="workflow-456",
        event_name="agent.taskCompleted",
        attributes={
            "task": "market analysis",
            "result": "bullish trend detected",
            "confidence": 0.85,
            "timestamp": datetime.utcnow().isoformat(),
            "metadata": {
                "model_version": "v2.1",
                "execution_time_ms": 1250
            }
        }
    )
    agent.apply(enhanced_event)
    
    print(f"   ✓ Agent '{agent.agent_name}' status: {agent.status}")
    print(f"   ✓ Current workflow: {agent.current_workflow}")
    print(f"   ✓ Created {len(agent.get_uncommitted_events())} events")

    # 4. Create and save order events
    print("\n4. Creating order events...")
    order = OrderAggregate(id="order-789")
    order.place_order(
        customer_id="customer-001",
        total=249.99,
        items=[
            {"sku": "WIDGET-001", "name": "Super Widget", "price": 99.99, "quantity": 2},
            {"sku": "GADGET-002", "name": "Amazing Gadget", "price": 149.99, "quantity": 1}
        ]
    )
    
    print(f"   ✓ Order for customer '{order.customer_id}' total: ${order.total}")
    print(f"   ✓ Order status: {order.status}")

    # 5. Save all aggregates
    print("\n5. Saving aggregates to event store...")
    await event_store.save(agent)
    await event_store.save(order)
    print("   ✓ All aggregates saved successfully")

    # 6. Load events by aggregate type and verify custom fields are preserved
    print("\n6. Loading agent events and verifying custom fields...")
    agent_events = await event_store.load_events_by_type("agent_aggregate")
    
    print(f"   ✓ Loaded {len(agent_events)} agent events")
    
    for i, event in enumerate(agent_events, 1):
        print(f"\n   Event {i}:")
        print(f"     Type: {type(event).__name__}")
        print(f"     Is AgentEvent: {isinstance(event, AgentEvent)}")
        print(f"     Event type: {event.event_type}")
        
        if isinstance(event, AgentEvent):
            print(f"     Agent name: {event.agent_name}")
            print(f"     Agent ID: {event.agent_id}")
            print(f"     Workflow ID: {event.workflow_id}")
            print(f"     Event name: {event.event_name}")
            print(f"     Attributes: {event.attributes}")
        else:
            # Show what fields are available even if not the right class
            event_dict = event.to_dict()
            print(f"     Available fields: {list(event_dict.keys())}")
            print(f"     Has agent_name: {'agent_name' in event_dict}")
            print(f"     Has attributes: {'attributes' in event_dict}")

    # 7. Load order events and verify custom fields
    print("\n7. Loading order events and verifying custom fields...")
    order_events = await event_store.load_events_by_type("order_aggregate")
    
    print(f"   ✓ Loaded {len(order_events)} order events")
    
    for i, event in enumerate(order_events, 1):
        print(f"\n   Event {i}:")
        print(f"     Type: {type(event).__name__}")
        print(f"     Is OrderEvent: {isinstance(event, OrderEvent)}")
        
        if isinstance(event, OrderEvent):
            print(f"     Order ID: {event.order_id}")
            print(f"     Customer ID: {event.customer_id}")
            print(f"     Order total: ${event.order_total}")
            print(f"     Currency: {event.currency}")
            print(f"     Items count: {len(event.items)}")
            print(f"     Shipping address: {event.shipping_address}")

    # 8. Test loading specific aggregates and verify state reconstruction
    print("\n8. Testing aggregate reconstruction...")
    
    loaded_agent = await event_store.load(AgentAggregate, "agent-123")
    if loaded_agent:
        print(f"   ✓ Agent loaded: {loaded_agent.agent_name}")
        print(f"   ✓ Agent status: {loaded_agent.status}")
        print(f"   ✓ Current workflow: {loaded_agent.current_workflow}")
        print(f"   ✓ Version: {loaded_agent.version}")
    
    loaded_order = await event_store.load(OrderAggregate, "order-789")
    if loaded_order:
        print(f"   ✓ Order loaded: customer {loaded_order.customer_id}")
        print(f"   ✓ Order total: ${loaded_order.total}")
        print(f"   ✓ Order status: {loaded_order.status}")
        print(f"   ✓ Version: {loaded_order.version}")

    # 9. Demonstrate what happens with unregistered event types
    print("\n9. Testing unregistered event type handling...")
    
    # Create a payment event but don't register it properly
    payment = PaymentEvent(
        aggregate_id="payment-001",
        aggregate_type="payment_aggregate",
        payment_id="pay-001",
        order_id="order-789",
        amount=249.99,
        payment_method="credit_card",
        status="completed",
        transaction_id="txn-abc123"
    )
    
    # Save it manually by creating a minimal aggregate
    class PaymentAggregate(Aggregate):
        def get_aggregate_type(self):
            return "payment_aggregate"
        def apply_payment_event(self, event):
            pass
        def apply_payment_completed(self, event):
            pass
    
    payment_agg = PaymentAggregate(id="payment-001")
    payment.event_type = "payment.completed"  # Use unregistered type
    payment_agg.apply(payment)
    await event_store.save(payment_agg)
    
    # Now load it back
    payment_events = await event_store.load_events_by_type("payment_aggregate")
    
    for event in payment_events:
        print(f"   Event type: {type(event).__name__}")
        print(f"   Is PaymentEvent: {isinstance(event, PaymentEvent)}")
        print(f"   Event type string: {event.event_type}")
        
        # Check if custom fields are preserved even without registration
        event_dict = event.to_dict()
        has_payment_fields = all(field in event_dict for field in ['payment_id', 'amount', 'payment_method'])
        print(f"   Custom fields preserved: {has_payment_fields}")
        
        if has_payment_fields:
            print(f"   Payment ID: {event_dict.get('payment_id')}")
            print(f"   Amount: ${event_dict.get('amount')}")
            print(f"   Payment method: {event_dict.get('payment_method')}")

    print("\n=== Example Complete ===")
    print("\nKey takeaways:")
    print("✓ Custom event classes can be registered with EventStore.register_event_class()")
    print("✓ Registered events are deserialized to the correct class with all custom fields")
    print("✓ Unregistered events fall back to base Event but preserve custom fields as extra attributes")
    print("✓ This solves the original issue where custom fields were lost during deserialization")


if __name__ == "__main__":
    asyncio.run(main())