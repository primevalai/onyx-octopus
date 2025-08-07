#!/usr/bin/env python3
"""
Saga Patterns Example

This example demonstrates saga patterns for managing long-running workflows:
- Orchestration vs Choreography patterns
- Compensating actions for failure scenarios  
- Distributed transaction management
- Event-driven workflow coordination
- Saga state machine implementation
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any, Union
from datetime import datetime, timezone
from enum import Enum
import json

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User  # Use built-in User aggregate
from eventuali.event import UserRegistered, Event
from eventuali.streaming import EventStreamer

# Saga State Enum
class SagaState(Enum):
    STARTED = "started"
    PAYMENT_REQUESTED = "payment_requested" 
    PAYMENT_COMPLETED = "payment_completed"
    INVENTORY_RESERVED = "inventory_reserved"
    SHIPPING_REQUESTED = "shipping_requested"
    COMPLETED = "completed"
    COMPENSATION_STARTED = "compensation_started"
    FAILED = "failed"

# Saga Events
class OrderSagaStarted(Event):
    """Order processing saga started."""
    order_id: str
    customer_id: str
    product_id: str
    quantity: int
    total_amount: float

class PaymentRequested(Event):
    """Payment processing requested."""
    payment_id: str
    amount: float
    customer_id: str

class PaymentCompleted(Event):
    """Payment successfully processed."""
    payment_id: str
    transaction_id: str
    amount: float

class PaymentFailed(Event):
    """Payment processing failed."""
    payment_id: str
    reason: str

class InventoryReservationRequested(Event):
    """Inventory reservation requested."""
    product_id: str
    quantity: int
    reservation_id: str

class InventoryReserved(Event):
    """Inventory successfully reserved."""
    reservation_id: str
    product_id: str
    quantity: int

class InventoryReservationFailed(Event):
    """Inventory reservation failed."""
    reservation_id: str
    product_id: str
    quantity: int
    reason: str

class ShippingRequested(Event):
    """Shipping process requested."""
    shipping_id: str
    order_id: str
    customer_id: str

class ShippingCompleted(Event):
    """Shipping completed."""
    shipping_id: str
    tracking_number: str

class ShippingFailed(Event):
    """Shipping process failed."""
    shipping_id: str
    reason: str

class OrderSagaCompleted(Event):
    """Order saga completed successfully."""
    completion_timestamp: str

class OrderSagaFailed(Event):
    """Order saga failed."""
    reason: str
    compensation_actions: List[str]

# Compensation Events
class PaymentReversalRequested(Event):
    """Payment reversal compensation."""
    original_payment_id: str
    reversal_id: str
    amount: float

class InventoryReleased(Event):
    """Inventory release compensation."""
    reservation_id: str
    product_id: str
    quantity: int

class ShippingCancelled(Event):
    """Shipping cancellation compensation."""
    shipping_id: str
    reason: str

# Order Processing Saga Aggregate
class OrderProcessingSaga:
    """Saga aggregate managing order processing workflow."""
    
    def __init__(self, saga_id: str = None):
        self.id = saga_id or f"saga-{datetime.now().strftime('%Y%m%d%H%M%S')}"
        self.state = SagaState.STARTED
        self.order_id: str = ""
        self.customer_id: str = ""
        self.product_id: str = ""
        self.quantity: int = 0
        self.total_amount: float = 0.0
        
        # Step tracking
        self.payment_id: Optional[str] = None
        self.reservation_id: Optional[str] = None
        self.shipping_id: Optional[str] = None
        self.transaction_id: Optional[str] = None
        self.tracking_number: Optional[str] = None
        
        # Compensation tracking
        self.compensation_actions: List[str] = []
        self.completed_steps: List[str] = []
        self.failed_steps: List[str] = []
        
        # Event tracking
        self.events: List[Event] = []
        self.version = 0
    
    def start_saga(self, order_id: str, customer_id: str, product_id: str, quantity: int, total_amount: float):
        """Start the order processing saga."""
        self.order_id = order_id
        self.customer_id = customer_id
        self.product_id = product_id
        self.quantity = quantity
        self.total_amount = total_amount
        
        event = OrderSagaStarted(
            order_id=order_id,
            customer_id=customer_id,
            product_id=product_id,
            quantity=quantity,
            total_amount=total_amount
        )
        self._apply_event(event)
    
    def request_payment(self, payment_id: str):
        """Request payment processing."""
        if self.state != SagaState.STARTED:
            raise ValueError(f"Cannot request payment in state {self.state}")
        
        self.payment_id = payment_id
        event = PaymentRequested(
            payment_id=payment_id,
            amount=self.total_amount,
            customer_id=self.customer_id
        )
        self._apply_event(event)
    
    def complete_payment(self, transaction_id: str):
        """Mark payment as completed."""
        if self.state != SagaState.PAYMENT_REQUESTED:
            raise ValueError(f"Cannot complete payment in state {self.state}")
        
        self.transaction_id = transaction_id
        self.completed_steps.append("payment")
        
        event = PaymentCompleted(
            payment_id=self.payment_id,
            transaction_id=transaction_id,
            amount=self.total_amount
        )
        self._apply_event(event)
    
    def fail_payment(self, reason: str):
        """Mark payment as failed."""
        self.failed_steps.append("payment")
        event = PaymentFailed(payment_id=self.payment_id, reason=reason)
        self._apply_event(event)
        self._start_compensation(f"Payment failed: {reason}")
    
    def request_inventory_reservation(self, reservation_id: str):
        """Request inventory reservation."""
        if self.state != SagaState.PAYMENT_COMPLETED:
            raise ValueError(f"Cannot request inventory reservation in state {self.state}")
        
        self.reservation_id = reservation_id
        event = InventoryReservationRequested(
            product_id=self.product_id,
            quantity=self.quantity,
            reservation_id=reservation_id
        )
        self._apply_event(event)
    
    def complete_inventory_reservation(self):
        """Mark inventory reservation as completed."""
        self.completed_steps.append("inventory")
        event = InventoryReserved(
            reservation_id=self.reservation_id,
            product_id=self.product_id,
            quantity=self.quantity
        )
        self._apply_event(event)
    
    def fail_inventory_reservation(self, reason: str):
        """Mark inventory reservation as failed."""
        self.failed_steps.append("inventory")
        event = InventoryReservationFailed(
            reservation_id=self.reservation_id,
            product_id=self.product_id,
            quantity=self.quantity,
            reason=reason
        )
        self._apply_event(event)
        self._start_compensation(f"Inventory reservation failed: {reason}")
    
    def request_shipping(self, shipping_id: str):
        """Request shipping processing."""
        if self.state != SagaState.INVENTORY_RESERVED:
            raise ValueError(f"Cannot request shipping in state {self.state}")
        
        self.shipping_id = shipping_id
        event = ShippingRequested(
            shipping_id=shipping_id,
            order_id=self.order_id,
            customer_id=self.customer_id
        )
        self._apply_event(event)
    
    def complete_shipping(self, tracking_number: str):
        """Mark shipping as completed."""
        self.tracking_number = tracking_number
        self.completed_steps.append("shipping")
        
        event = ShippingCompleted(
            shipping_id=self.shipping_id,
            tracking_number=tracking_number
        )
        self._apply_event(event)
        
        # Complete the saga
        completion_event = OrderSagaCompleted(
            completion_timestamp=datetime.now(timezone.utc).isoformat()
        )
        self._apply_event(completion_event)
    
    def fail_shipping(self, reason: str):
        """Mark shipping as failed."""
        self.failed_steps.append("shipping")
        event = ShippingFailed(shipping_id=self.shipping_id, reason=reason)
        self._apply_event(event)
        self._start_compensation(f"Shipping failed: {reason}")
    
    def _start_compensation(self, failure_reason: str):
        """Start compensation actions for failed saga."""
        self.state = SagaState.COMPENSATION_STARTED
        
        # Determine compensation actions needed
        if "shipping" in self.completed_steps:
            self._compensate_shipping()
        
        if "inventory" in self.completed_steps:
            self._compensate_inventory()
        
        if "payment" in self.completed_steps:
            self._compensate_payment()
        
        # Mark saga as failed
        event = OrderSagaFailed(
            reason=failure_reason,
            compensation_actions=self.compensation_actions
        )
        self._apply_event(event)
    
    def _compensate_payment(self):
        """Compensate payment by requesting reversal."""
        reversal_id = f"reversal-{self.payment_id}"
        event = PaymentReversalRequested(
            original_payment_id=self.payment_id,
            reversal_id=reversal_id,
            amount=self.total_amount
        )
        self._apply_event(event)
        self.compensation_actions.append("payment_reversal")
    
    def _compensate_inventory(self):
        """Compensate inventory by releasing reservation."""
        event = InventoryReleased(
            reservation_id=self.reservation_id,
            product_id=self.product_id,
            quantity=self.quantity
        )
        self._apply_event(event)
        self.compensation_actions.append("inventory_release")
    
    def _compensate_shipping(self):
        """Compensate shipping by cancelling."""
        event = ShippingCancelled(
            shipping_id=self.shipping_id,
            reason="Order processing failed"
        )
        self._apply_event(event)
        self.compensation_actions.append("shipping_cancellation")
    
    def _apply_event(self, event: Event):
        """Apply event and update state."""
        self.events.append(event)
        self.version += 1
        
        # State transitions
        if isinstance(event, OrderSagaStarted):
            self.state = SagaState.STARTED
        elif isinstance(event, PaymentRequested):
            self.state = SagaState.PAYMENT_REQUESTED
        elif isinstance(event, PaymentCompleted):
            self.state = SagaState.PAYMENT_COMPLETED
        elif isinstance(event, InventoryReserved):
            self.state = SagaState.INVENTORY_RESERVED
        elif isinstance(event, ShippingRequested):
            self.state = SagaState.SHIPPING_REQUESTED
        elif isinstance(event, OrderSagaCompleted):
            self.state = SagaState.COMPLETED
        elif isinstance(event, OrderSagaFailed):
            self.state = SagaState.FAILED
    
    @property
    def is_completed(self) -> bool:
        return self.state == SagaState.COMPLETED
    
    @property
    def is_failed(self) -> bool:
        return self.state == SagaState.FAILED
    
    @property
    def progress_percentage(self) -> float:
        total_steps = 3  # payment, inventory, shipping
        return (len(self.completed_steps) / total_steps) * 100 if total_steps > 0 else 0

# Saga Orchestrator
class SagaOrchestrator:
    """Orchestrator managing saga execution."""
    
    def __init__(self, event_store: EventStore):
        self.event_store = event_store
        self.active_sagas: Dict[str, OrderProcessingSaga] = {}
    
    async def start_order_processing_saga(self, order_id: str, customer_id: str, product_id: str, quantity: int, total_amount: float) -> OrderProcessingSaga:
        """Start a new order processing saga."""
        saga = OrderProcessingSaga()
        saga.start_saga(order_id, customer_id, product_id, quantity, total_amount)
        
        self.active_sagas[saga.id] = saga
        return saga
    
    async def execute_saga(self, saga: OrderProcessingSaga, simulate_failures: Dict[str, bool] = None) -> Dict[str, Any]:
        """Execute saga with optional failure simulation."""
        result = {
            "saga_id": saga.id,
            "order_id": saga.order_id,
            "steps": [],
            "compensation_actions": [],
            "final_state": None,
            "execution_time_ms": 0
        }
        
        start_time = datetime.now()
        simulate_failures = simulate_failures or {}
        
        try:
            # Step 1: Payment Processing
            payment_id = f"payment-{saga.order_id}"
            saga.request_payment(payment_id)
            result["steps"].append(f"Payment requested: {payment_id}")
            
            # Simulate payment processing
            await asyncio.sleep(0.1)  # Simulate async processing
            
            if simulate_failures.get("payment", False):
                saga.fail_payment("Simulated payment failure")
                result["steps"].append("Payment failed (simulated)")
            else:
                transaction_id = f"txn-{payment_id}"
                saga.complete_payment(transaction_id)
                result["steps"].append(f"Payment completed: {transaction_id}")
                
                # Step 2: Inventory Reservation
                reservation_id = f"res-{saga.order_id}"
                saga.request_inventory_reservation(reservation_id)
                result["steps"].append(f"Inventory reservation requested: {reservation_id}")
                
                await asyncio.sleep(0.1)
                
                if simulate_failures.get("inventory", False):
                    saga.fail_inventory_reservation("Simulated inventory shortage")
                    result["steps"].append("Inventory reservation failed (simulated)")
                else:
                    saga.complete_inventory_reservation()
                    result["steps"].append("Inventory reserved successfully")
                    
                    # Step 3: Shipping
                    shipping_id = f"ship-{saga.order_id}"
                    saga.request_shipping(shipping_id)
                    result["steps"].append(f"Shipping requested: {shipping_id}")
                    
                    await asyncio.sleep(0.1)
                    
                    if simulate_failures.get("shipping", False):
                        saga.fail_shipping("Simulated shipping failure")
                        result["steps"].append("Shipping failed (simulated)")
                    else:
                        tracking_number = f"TRK{saga.order_id.upper()}"
                        saga.complete_shipping(tracking_number)
                        result["steps"].append(f"Shipping completed: {tracking_number}")
        
        except Exception as e:
            result["steps"].append(f"Unexpected error: {e}")
        
        end_time = datetime.now()
        result["execution_time_ms"] = int((end_time - start_time).total_seconds() * 1000)
        result["final_state"] = saga.state.value
        result["compensation_actions"] = saga.compensation_actions
        result["completed_steps"] = saga.completed_steps
        result["failed_steps"] = saga.failed_steps
        result["progress"] = saga.progress_percentage
        
        return result
    
    def get_saga_status(self, saga_id: str) -> Optional[Dict[str, Any]]:
        """Get current status of a saga."""
        saga = self.active_sagas.get(saga_id)
        if not saga:
            return None
        
        return {
            "saga_id": saga.id,
            "order_id": saga.order_id,
            "state": saga.state.value,
            "progress": saga.progress_percentage,
            "completed_steps": saga.completed_steps,
            "failed_steps": saga.failed_steps,
            "compensation_actions": saga.compensation_actions,
            "total_events": len(saga.events)
        }

async def demonstrate_saga_patterns():
    """Demonstrate saga patterns for distributed transactions."""
    print("=== Saga Patterns Example ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    orchestrator = SagaOrchestrator(event_store)
    
    # Setup test customer
    print("1. Setting up test data...")
    customer = User(id="alice")
    customer.apply(UserRegistered(name="Alice Johnson", email="alice@example.com"))
    await event_store.save(customer)
    customer.mark_events_as_committed()
    print(f"   ✓ Created customer: {customer.name}")
    
    # Test Case 1: Successful Saga Execution
    print("\n2. Testing successful saga execution...")
    saga1 = await orchestrator.start_order_processing_saga("order-001", "alice", "laptop", 1, 1200.0)
    result1 = await orchestrator.execute_saga(saga1)
    
    print(f"   Saga {result1['saga_id']} - Final State: {result1['final_state']}")
    print(f"   Execution Time: {result1['execution_time_ms']}ms")
    print(f"   Progress: {result1['progress']:.1f}%")
    
    for step in result1["steps"]:
        print(f"     ✓ {step}")
    
    # Test Case 2: Payment Failure with Compensation
    print("\n3. Testing payment failure scenario...")
    saga2 = await orchestrator.start_order_processing_saga("order-002", "alice", "mouse", 2, 100.0)
    result2 = await orchestrator.execute_saga(saga2, {"payment": True})
    
    print(f"   Saga {result2['saga_id']} - Final State: {result2['final_state']}")
    print(f"   Execution Time: {result2['execution_time_ms']}ms")
    print(f"   Progress: {result2['progress']:.1f}%")
    
    for step in result2["steps"]:
        status = "✓" if "completed" in step else "❌" if "failed" in step else "→"
        print(f"     {status} {step}")
    
    if result2["compensation_actions"]:
        print("   Compensation Actions:")
        for action in result2["compensation_actions"]:
            print(f"     🔄 {action}")
    
    # Test Case 3: Inventory Failure with Compensation
    print("\n4. Testing inventory failure scenario...")
    saga3 = await orchestrator.start_order_processing_saga("order-003", "alice", "keyboard", 1, 150.0)
    result3 = await orchestrator.execute_saga(saga3, {"inventory": True})
    
    print(f"   Saga {result3['saga_id']} - Final State: {result3['final_state']}")
    print(f"   Execution Time: {result3['execution_time_ms']}ms")
    
    for step in result3["steps"]:
        status = "✓" if "completed" in step else "❌" if "failed" in step else "→"
        print(f"     {status} {step}")
    
    if result3["compensation_actions"]:
        print("   Compensation Actions:")
        for action in result3["compensation_actions"]:
            print(f"     🔄 {action}")
    
    # Test Case 4: Shipping Failure with Full Compensation
    print("\n5. Testing shipping failure scenario...")
    saga4 = await orchestrator.start_order_processing_saga("order-004", "alice", "monitor", 1, 300.0)
    result4 = await orchestrator.execute_saga(saga4, {"shipping": True})
    
    print(f"   Saga {result4['saga_id']} - Final State: {result4['final_state']}")
    print(f"   Execution Time: {result4['execution_time_ms']}ms")
    
    for step in result4["steps"]:
        status = "✓" if "completed" in step else "❌" if "failed" in step else "→"
        print(f"     {status} {step}")
    
    if result4["compensation_actions"]:
        print("   Compensation Actions:")
        for action in result4["compensation_actions"]:
            print(f"     🔄 {action}")
    
    # Test Case 5: Concurrent Saga Execution
    print("\n6. Testing concurrent saga execution...")
    
    # Start multiple sagas concurrently
    sagas = []
    for i in range(3):
        saga = await orchestrator.start_order_processing_saga(f"order-00{i+5}", "alice", "headset", 1, 80.0)
        sagas.append(saga)
    
    # Execute concurrently with different failure patterns
    failure_patterns = [
        {},  # Success
        {"payment": True},  # Payment failure
        {"inventory": True}  # Inventory failure
    ]
    
    results = await asyncio.gather(*[
        orchestrator.execute_saga(saga, failure_patterns[i])
        for i, saga in enumerate(sagas)
    ])
    
    print("   Concurrent Execution Results:")
    for i, result in enumerate(results):
        status = "SUCCESS" if result["final_state"] == "completed" else "FAILED"
        print(f"     - Order {result['order_id']}: {status} ({result['execution_time_ms']}ms)")
        if result["compensation_actions"]:
            print(f"       Compensations: {', '.join(result['compensation_actions'])}")
    
    # System State Summary
    print("\n7. Saga execution summary...")
    all_results = [result1, result2, result3, result4] + results
    
    successful = sum(1 for r in all_results if r["final_state"] == "completed")
    failed = sum(1 for r in all_results if r["final_state"] == "failed")
    total_compensation_actions = sum(len(r["compensation_actions"]) for r in all_results)
    avg_execution_time = sum(r["execution_time_ms"] for r in all_results) / len(all_results)
    
    print(f"   Total Sagas Executed: {len(all_results)}")
    print(f"   Successful: {successful}")
    print(f"   Failed: {failed}")
    print(f"   Total Compensation Actions: {total_compensation_actions}")
    print(f"   Average Execution Time: {avg_execution_time:.1f}ms")
    
    # Event Analysis
    print("\n8. Event analysis...")
    total_events = sum(len(saga.events) for saga in orchestrator.active_sagas.values())
    event_types = {}
    
    for saga in orchestrator.active_sagas.values():
        for event in saga.events:
            event_type = type(event).__name__
            event_types[event_type] = event_types.get(event_type, 0) + 1
    
    print(f"   Total Events Generated: {total_events}")
    print("   Event Type Distribution:")
    for event_type, count in sorted(event_types.items()):
        print(f"     - {event_type}: {count}")
    
    return {
        "orchestrator": orchestrator,
        "results": all_results,
        "event_types": event_types,
        "summary": {
            "successful": successful,
            "failed": failed,
            "compensation_actions": total_compensation_actions,
            "avg_execution_time": avg_execution_time
        }
    }

async def main():
    result = await demonstrate_saga_patterns()
    
    print(f"\n✅ SUCCESS! Saga patterns demonstrated!")
    
    print(f"\nSaga patterns covered:")
    print(f"- ✓ Orchestrated saga coordination")
    print(f"- ✓ Multi-step distributed transactions")
    print(f"- ✓ Compensating actions for failure recovery")
    print(f"- ✓ State machine-based saga management")
    print(f"- ✓ Concurrent saga execution")
    print(f"- ✓ Event-driven workflow coordination")
    print(f"- ✓ Failure isolation and recovery patterns")
    
    summary = result["summary"]
    print(f"\nExecution summary:")
    print(f"- Successful sagas: {summary['successful']}")
    print(f"- Failed sagas: {summary['failed']}")
    print(f"- Compensation actions: {summary['compensation_actions']}")
    print(f"- Average execution time: {summary['avg_execution_time']:.1f}ms")

if __name__ == "__main__":
    asyncio.run(main())