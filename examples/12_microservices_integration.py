#!/usr/bin/env python3
"""
Microservices Integration Example

This example demonstrates microservices integration patterns:
- Service boundaries and event-driven communication
- Inter-service event publishing and subscription
- Service choreography vs orchestration
- Event-driven saga patterns across services
- Service resilience and failure handling
- API gateway integration with event sourcing
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any, Callable
from datetime import datetime, timezone
from enum import Enum
from abc import ABC, abstractmethod
import json
import uuid
import time

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User
from eventuali.event import UserRegistered, Event

# Service-Specific Events
class UserServiceEvents:
    """Events from User Service."""
    
    class UserAccountCreated(Event):
        user_id: str
        email: str
        account_type: str
        
    class UserProfileUpdated(Event):
        user_id: str
        profile_data: Dict[str, Any]
        
    class UserPreferencesChanged(Event):
        user_id: str
        preferences: Dict[str, Any]

class OrderServiceEvents:
    """Events from Order Service."""
    
    class OrderCreated(Event):
        order_id: str
        user_id: str
        items: List[Dict[str, Any]]
        total_amount: float
        
    class OrderStatusChanged(Event):
        order_id: str
        old_status: str
        new_status: str
        reason: Optional[str] = None
        
    class OrderCancelled(Event):
        order_id: str
        user_id: str
        cancellation_reason: str

class PaymentServiceEvents:
    """Events from Payment Service."""
    
    class PaymentRequested(Event):
        payment_id: str
        order_id: str
        user_id: str
        amount: float
        payment_method: str
        
    class PaymentCompleted(Event):
        payment_id: str
        order_id: str
        transaction_id: str
        amount: float
        
    class PaymentFailed(Event):
        payment_id: str
        order_id: str
        failure_reason: str

class InventoryServiceEvents:
    """Events from Inventory Service."""
    
    class InventoryReserved(Event):
        reservation_id: str
        product_id: str
        quantity: int
        order_id: str
        
    class InventoryReleased(Event):
        reservation_id: str
        product_id: str
        quantity: int
        reason: str
        
    class StockLevelChanged(Event):
        product_id: str
        old_level: int
        new_level: int

class NotificationServiceEvents:
    """Events from Notification Service."""
    
    class NotificationSent(Event):
        notification_id: str
        user_id: str
        channel: str
        message: str
        
    class NotificationFailed(Event):
        notification_id: str
        user_id: str
        failure_reason: str

# Base Microservice
class MicroService(ABC):
    """Base class for microservices."""
    
    def __init__(self, service_name: str):
        self.service_name = service_name
        self.event_store: Optional[EventStore] = None
        self.event_handlers: Dict[str, Callable] = {}
        self.published_events: List[Event] = []
        self.processed_events: List[Event] = []
        self.service_health = "healthy"
        self.metrics = {
            "events_published": 0,
            "events_processed": 0,
            "errors": 0,
            "uptime": time.time()
        }
    
    async def initialize(self):
        """Initialize the microservice."""
        self.event_store = await EventStore.create("sqlite://:memory:")
        await self._register_event_handlers()
    
    @abstractmethod
    async def _register_event_handlers(self):
        """Register event handlers specific to this service."""
        pass
    
    def register_handler(self, event_type: str, handler: Callable):
        """Register an event handler."""
        self.event_handlers[event_type] = handler
    
    async def publish_event(self, event: Event):
        """Publish an event from this service."""
        event.aggregate_id = getattr(event, 'aggregate_id', f"{self.service_name}-{uuid.uuid4().hex[:8]}")
        event.event_type = type(event).__name__
        event.timestamp = datetime.now(timezone.utc)
        
        self.published_events.append(event)
        self.metrics["events_published"] += 1
        
        print(f"   📤 {self.service_name}: Published {type(event).__name__}")
        return event
    
    async def handle_event(self, event: Event) -> bool:
        """Handle an incoming event."""
        event_type = type(event).__name__
        
        if event_type in self.event_handlers:
            try:
                await self.event_handlers[event_type](event)
                self.processed_events.append(event)
                self.metrics["events_processed"] += 1
                print(f"   📥 {self.service_name}: Handled {event_type}")
                return True
            except Exception as e:
                print(f"   ❌ {self.service_name}: Error handling {event_type}: {e}")
                self.metrics["errors"] += 1
                return False
        
        return False
    
    def get_health_status(self) -> Dict[str, Any]:
        """Get service health status."""
        uptime = time.time() - self.metrics["uptime"]
        return {
            "service_name": self.service_name,
            "health": self.service_health,
            "uptime_seconds": round(uptime, 2),
            "events_published": self.metrics["events_published"],
            "events_processed": self.metrics["events_processed"],
            "error_count": self.metrics["errors"],
            "error_rate": (self.metrics["errors"] / max(self.metrics["events_processed"], 1)) * 100
        }

# Specific Microservice Implementations
class UserService(MicroService):
    """User management microservice."""
    
    def __init__(self):
        super().__init__("UserService")
        self.users: Dict[str, Dict[str, Any]] = {}
    
    async def _register_event_handlers(self):
        """Register handlers for events this service cares about."""
        self.register_handler("OrderCreated", self._handle_order_created)
        self.register_handler("PaymentCompleted", self._handle_payment_completed)
    
    async def create_user_account(self, user_id: str, email: str, account_type: str = "standard"):
        """Create a new user account."""
        if user_id in self.users:
            raise ValueError("User already exists")
        
        self.users[user_id] = {
            "user_id": user_id,
            "email": email,
            "account_type": account_type,
            "created_at": datetime.now(timezone.utc).isoformat(),
            "profile": {},
            "preferences": {}
        }
        
        event = UserServiceEvents.UserAccountCreated(
            user_id=user_id,
            email=email,
            account_type=account_type
        )
        
        return await self.publish_event(event)
    
    async def update_user_preferences(self, user_id: str, preferences: Dict[str, Any]):
        """Update user preferences."""
        if user_id not in self.users:
            raise ValueError("User not found")
        
        self.users[user_id]["preferences"].update(preferences)
        
        event = UserServiceEvents.UserPreferencesChanged(
            user_id=user_id,
            preferences=preferences
        )
        
        return await self.publish_event(event)
    
    async def _handle_order_created(self, event):
        """Handle order created events."""
        # Update user order history
        user_id = event.user_id
        if user_id in self.users:
            if "order_history" not in self.users[user_id]:
                self.users[user_id]["order_history"] = []
            
            self.users[user_id]["order_history"].append({
                "order_id": event.order_id,
                "total_amount": event.total_amount,
                "created_at": datetime.now(timezone.utc).isoformat()
            })
    
    async def _handle_payment_completed(self, event):
        """Handle payment completed events."""
        # Could trigger loyalty points, upgrade account type, etc.
        user_id = event.user_id
        if user_id in self.users:
            # Award loyalty points (simplified)
            points = int(event.amount * 0.1)  # 10 cents per dollar
            if "loyalty_points" not in self.users[user_id]:
                self.users[user_id]["loyalty_points"] = 0
            self.users[user_id]["loyalty_points"] += points

class OrderService(MicroService):
    """Order management microservice."""
    
    def __init__(self):
        super().__init__("OrderService")
        self.orders: Dict[str, Dict[str, Any]] = {}
    
    async def _register_event_handlers(self):
        """Register handlers for events this service cares about."""
        self.register_handler("UserAccountCreated", self._handle_user_created)
        self.register_handler("PaymentCompleted", self._handle_payment_completed)
        self.register_handler("PaymentFailed", self._handle_payment_failed)
        self.register_handler("InventoryReserved", self._handle_inventory_reserved)
    
    async def create_order(self, user_id: str, items: List[Dict[str, Any]]) -> str:
        """Create a new order."""
        order_id = f"order-{uuid.uuid4().hex[:8]}"
        total_amount = sum(item["price"] * item["quantity"] for item in items)
        
        self.orders[order_id] = {
            "order_id": order_id,
            "user_id": user_id,
            "items": items,
            "total_amount": total_amount,
            "status": "created",
            "created_at": datetime.now(timezone.utc).isoformat()
        }
        
        event = OrderServiceEvents.OrderCreated(
            order_id=order_id,
            user_id=user_id,
            items=items,
            total_amount=total_amount
        )
        
        await self.publish_event(event)
        return order_id
    
    async def change_order_status(self, order_id: str, new_status: str, reason: str = None):
        """Change order status."""
        if order_id not in self.orders:
            raise ValueError("Order not found")
        
        old_status = self.orders[order_id]["status"]
        self.orders[order_id]["status"] = new_status
        self.orders[order_id]["updated_at"] = datetime.now(timezone.utc).isoformat()
        
        event = OrderServiceEvents.OrderStatusChanged(
            order_id=order_id,
            old_status=old_status,
            new_status=new_status,
            reason=reason
        )
        
        return await self.publish_event(event)
    
    async def _handle_user_created(self, event):
        """Handle user created events."""
        # Could initialize user's order preferences, shopping cart, etc.
        pass
    
    async def _handle_payment_completed(self, event):
        """Handle payment completion."""
        order_id = event.order_id
        if order_id in self.orders:
            await self.change_order_status(order_id, "paid", "Payment successful")
    
    async def _handle_payment_failed(self, event):
        """Handle payment failure."""
        order_id = event.order_id
        if order_id in self.orders:
            await self.change_order_status(order_id, "payment_failed", event.failure_reason)
    
    async def _handle_inventory_reserved(self, event):
        """Handle inventory reservation."""
        order_id = event.order_id
        if order_id in self.orders:
            self.orders[order_id]["inventory_reserved"] = True

class PaymentService(MicroService):
    """Payment processing microservice."""
    
    def __init__(self):
        super().__init__("PaymentService")
        self.payments: Dict[str, Dict[str, Any]] = {}
        self.failure_rate = 0.1  # 10% failure rate for demo
    
    async def _register_event_handlers(self):
        """Register handlers for events this service cares about."""
        self.register_handler("OrderCreated", self._handle_order_created)
        self.register_handler("OrderCancelled", self._handle_order_cancelled)
    
    async def process_payment(self, order_id: str, user_id: str, amount: float, payment_method: str) -> str:
        """Process a payment."""
        payment_id = f"pay-{uuid.uuid4().hex[:8]}"
        
        # Request payment
        request_event = PaymentServiceEvents.PaymentRequested(
            payment_id=payment_id,
            order_id=order_id,
            user_id=user_id,
            amount=amount,
            payment_method=payment_method
        )
        await self.publish_event(request_event)
        
        # Simulate payment processing
        await asyncio.sleep(0.1)
        
        # Simulate random failures
        import random
        if random.random() < self.failure_rate:
            # Payment failed
            self.payments[payment_id] = {
                "payment_id": payment_id,
                "order_id": order_id,
                "status": "failed",
                "amount": amount,
                "failure_reason": "Insufficient funds"
            }
            
            failure_event = PaymentServiceEvents.PaymentFailed(
                payment_id=payment_id,
                order_id=order_id,
                failure_reason="Insufficient funds"
            )
            await self.publish_event(failure_event)
            return payment_id
        
        # Payment successful
        transaction_id = f"txn-{uuid.uuid4().hex[:8]}"
        self.payments[payment_id] = {
            "payment_id": payment_id,
            "order_id": order_id,
            "transaction_id": transaction_id,
            "status": "completed",
            "amount": amount,
            "processed_at": datetime.now(timezone.utc).isoformat()
        }
        
        success_event = PaymentServiceEvents.PaymentCompleted(
            payment_id=payment_id,
            order_id=order_id,
            transaction_id=transaction_id,
            amount=amount
        )
        await self.publish_event(success_event)
        return payment_id
    
    async def _handle_order_created(self, event):
        """Handle order created events."""
        # Automatically initiate payment processing
        await self.process_payment(
            event.order_id,
            event.user_id,
            event.total_amount,
            "credit_card"
        )
    
    async def _handle_order_cancelled(self, event):
        """Handle order cancellation."""
        # Could initiate refunds for paid orders
        pass

class NotificationService(MicroService):
    """Notification service."""
    
    def __init__(self):
        super().__init__("NotificationService")
        self.notifications: List[Dict[str, Any]] = []
        
    async def _register_event_handlers(self):
        """Register handlers for events this service cares about."""
        self.register_handler("UserAccountCreated", self._handle_user_created)
        self.register_handler("OrderCreated", self._handle_order_created)
        self.register_handler("PaymentCompleted", self._handle_payment_completed)
        self.register_handler("PaymentFailed", self._handle_payment_failed)
    
    async def send_notification(self, user_id: str, channel: str, message: str) -> str:
        """Send a notification."""
        notification_id = f"notif-{uuid.uuid4().hex[:8]}"
        
        # Simulate sending
        await asyncio.sleep(0.05)
        
        # Assume 95% success rate
        import random
        success = random.random() > 0.05
        
        if success:
            self.notifications.append({
                "notification_id": notification_id,
                "user_id": user_id,
                "channel": channel,
                "message": message,
                "status": "sent",
                "sent_at": datetime.now(timezone.utc).isoformat()
            })
            
            event = NotificationServiceEvents.NotificationSent(
                notification_id=notification_id,
                user_id=user_id,
                channel=channel,
                message=message
            )
            await self.publish_event(event)
        else:
            event = NotificationServiceEvents.NotificationFailed(
                notification_id=notification_id,
                user_id=user_id,
                failure_reason="Network timeout"
            )
            await self.publish_event(event)
        
        return notification_id
    
    async def _handle_user_created(self, event):
        """Send welcome notification."""
        await self.send_notification(
            event.user_id,
            "email",
            f"Welcome! Your {event.account_type} account has been created."
        )
    
    async def _handle_order_created(self, event):
        """Send order confirmation."""
        await self.send_notification(
            event.user_id,
            "email",
            f"Order {event.order_id} created for ${event.total_amount}"
        )
    
    async def _handle_payment_completed(self, event):
        """Send payment confirmation."""
        await self.send_notification(
            event.user_id,
            "sms",
            f"Payment of ${event.amount} completed for order {event.order_id}"
        )
    
    async def _handle_payment_failed(self, event):
        """Send payment failure notification."""
        await self.send_notification(
            event.user_id,
            "email",
            f"Payment failed for order {event.order_id}: {event.failure_reason}"
        )

# Event Bus for Microservices Communication
class EventBus:
    """Event bus for inter-service communication."""
    
    def __init__(self):
        self.services: Dict[str, MicroService] = {}
        self.event_log: List[Dict[str, Any]] = []
        self.subscriptions: Dict[str, List[str]] = {}  # event_type -> [service_names]
    
    def register_service(self, service: MicroService):
        """Register a service with the event bus."""
        self.services[service.service_name] = service
    
    def subscribe(self, service_name: str, event_type: str):
        """Subscribe a service to an event type."""
        if event_type not in self.subscriptions:
            self.subscriptions[event_type] = []
        
        if service_name not in self.subscriptions[event_type]:
            self.subscriptions[event_type].append(service_name)
    
    async def publish(self, event: Event, publisher_service: str):
        """Publish an event to all interested subscribers."""
        event_type = type(event).__name__
        
        # Log the event
        self.event_log.append({
            "event_id": f"{publisher_service}-{len(self.event_log)}",
            "event_type": event_type,
            "publisher": publisher_service,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "event_data": self._extract_event_data(event)
        })
        
        # Find subscribers
        subscribers = self.subscriptions.get(event_type, [])
        
        # Publish to subscribers (excluding the publisher)
        for service_name in subscribers:
            if service_name != publisher_service and service_name in self.services:
                service = self.services[service_name]
                await service.handle_event(event)
    
    def get_event_statistics(self) -> Dict[str, Any]:
        """Get event bus statistics."""
        event_types = {}
        publishers = {}
        
        for log_entry in self.event_log:
            event_type = log_entry["event_type"]
            publisher = log_entry["publisher"]
            
            event_types[event_type] = event_types.get(event_type, 0) + 1
            publishers[publisher] = publishers.get(publisher, 0) + 1
        
        return {
            "total_events": len(self.event_log),
            "event_types": event_types,
            "publishers": publishers,
            "registered_services": len(self.services),
            "total_subscriptions": sum(len(subs) for subs in self.subscriptions.values())
        }
    
    def _extract_event_data(self, event: Event) -> Dict[str, Any]:
        """Extract event data for logging."""
        data = {}
        
        # Use Pydantic model fields to avoid deprecated attribute access
        if hasattr(event.__class__, 'model_fields'):
            # Get field names from the model class, not instance
            field_names = event.__class__.model_fields.keys()
            for field_name in field_names:
                try:
                    value = getattr(event, field_name)
                    if isinstance(value, (str, int, float, bool, type(None), list, dict)):
                        data[field_name] = value
                except:
                    continue
        else:
            # Fallback for non-Pydantic events - filter out known problematic attributes
            excluded_attrs = {'model_fields', 'model_computed_fields', 'model_config'}
            for attr_name in dir(event):
                if (not attr_name.startswith('_') and 
                    attr_name not in excluded_attrs and 
                    not callable(getattr(event, attr_name))):
                    try:
                        value = getattr(event, attr_name)
                        if isinstance(value, (str, int, float, bool, type(None), list, dict)):
                            data[attr_name] = value
                    except:
                        continue
        
        return data

async def demonstrate_microservices_integration():
    """Demonstrate microservices integration patterns."""
    print("=== Microservices Integration Example ===\n")
    
    # Initialize event bus
    event_bus = EventBus()
    
    print("1. Setting up microservices architecture...")
    
    # Create microservices
    services = {
        "user": UserService(),
        "order": OrderService(),
        "payment": PaymentService(),
        "notification": NotificationService()
    }
    
    # Initialize services
    for service_name, service in services.items():
        await service.initialize()
        event_bus.register_service(service)
        print(f"   ✓ {service.service_name} initialized")
    
    # Configure subscriptions (choreography pattern)
    print("\n2. Configuring event subscriptions...")
    
    subscriptions = [
        ("UserService", "OrderCreated"),
        ("UserService", "PaymentCompleted"),
        ("OrderService", "UserAccountCreated"),
        ("OrderService", "PaymentCompleted"),
        ("OrderService", "PaymentFailed"),
        ("OrderService", "InventoryReserved"),
        ("PaymentService", "OrderCreated"),
        ("PaymentService", "OrderCancelled"),
        ("NotificationService", "UserAccountCreated"),
        ("NotificationService", "OrderCreated"),
        ("NotificationService", "PaymentCompleted"),
        ("NotificationService", "PaymentFailed"),
    ]
    
    for service_name, event_type in subscriptions:
        event_bus.subscribe(service_name, event_type)
        print(f"   ✓ {service_name} subscribed to {event_type}")
    
    print(f"   ✓ {len(subscriptions)} subscriptions configured")
    
    # Business scenario: User registration and order processing
    print("\n3. Executing business scenario: User registration and orders...")
    
    # User registration
    print(f"\n   👤 User Registration:")
    user_service = services["user"]
    user_event = await user_service.create_user_account("alice", "alice@example.com", "premium")
    await event_bus.publish(user_event, "UserService")
    
    # Create multiple orders
    print(f"\n   🛒 Order Processing:")
    order_service = services["order"]
    
    orders_data = [
        ([{"product_id": "laptop", "price": 1200.0, "quantity": 1}], "alice"),
        ([{"product_id": "mouse", "price": 80.0, "quantity": 2}], "alice"),
        ([{"product_id": "keyboard", "price": 150.0, "quantity": 1}], "alice")
    ]
    
    created_orders = []
    for items, user_id in orders_data:
        order_id = await order_service.create_order(user_id, items)
        created_orders.append(order_id)
        
        # Publish order created event through event bus
        order_event = order_service.published_events[-1]  # Get last published event
        await event_bus.publish(order_event, "OrderService")
        
        # Allow some processing time
        await asyncio.sleep(0.1)
    
    print(f"   ✓ Created {len(created_orders)} orders")
    
    # Update user preferences
    print(f"\n   ⚙️  User Preferences Update:")
    preferences_event = await user_service.update_user_preferences("alice", {
        "email_notifications": True,
        "sms_notifications": True,
        "preferred_payment_method": "credit_card"
    })
    await event_bus.publish(preferences_event, "UserService")
    
    # Service Health Check
    print("\n4. Service health monitoring...")
    
    print(f"   🏥 Service Health Status:")
    for service_name, service in services.items():
        health = service.get_health_status()
        status_icon = "🟢" if health["health"] == "healthy" else "🔴"
        print(f"      {status_icon} {health['service_name']}:")
        print(f"         Published: {health['events_published']} events")
        print(f"         Processed: {health['events_processed']} events")
        print(f"         Errors: {health['error_count']}")
        print(f"         Error Rate: {health['error_rate']:.1f}%")
        print(f"         Uptime: {health['uptime_seconds']:.1f}s")
    
    # Event Bus Analytics
    print("\n5. Event bus analytics...")
    
    stats = event_bus.get_event_statistics()
    
    print(f"   📊 Event Bus Statistics:")
    print(f"      Total Events: {stats['total_events']}")
    print(f"      Registered Services: {stats['registered_services']}")
    print(f"      Total Subscriptions: {stats['total_subscriptions']}")
    
    print(f"      Event Type Distribution:")
    for event_type, count in sorted(stats['event_types'].items()):
        print(f"        - {event_type}: {count}")
    
    print(f"      Publisher Distribution:")
    for publisher, count in sorted(stats['publishers'].items()):
        print(f"        - {publisher}: {count}")
    
    # Service Data Inspection
    print("\n6. Service data inspection...")
    
    # Check user service data
    print(f"   👤 User Service Data:")
    alice_data = user_service.users.get("alice", {})
    print(f"      User: {alice_data.get('email', 'N/A')}")
    print(f"      Account Type: {alice_data.get('account_type', 'N/A')}")
    print(f"      Order History: {len(alice_data.get('order_history', []))}")
    print(f"      Loyalty Points: {alice_data.get('loyalty_points', 0)}")
    
    # Check order service data  
    print(f"   🛒 Order Service Data:")
    for order_id in created_orders[:3]:  # Show first 3
        order = order_service.orders.get(order_id, {})
        print(f"      Order {order_id}: {order.get('status', 'unknown')} (${order.get('total_amount', 0)})")
    
    # Check payment service data
    print(f"   💳 Payment Service Data:")
    payment_service = services["payment"]
    successful_payments = len([p for p in payment_service.payments.values() if p["status"] == "completed"])
    failed_payments = len([p for p in payment_service.payments.values() if p["status"] == "failed"])
    print(f"      Successful Payments: {successful_payments}")
    print(f"      Failed Payments: {failed_payments}")
    print(f"      Success Rate: {(successful_payments / max(successful_payments + failed_payments, 1)) * 100:.1f}%")
    
    # Check notification service data
    print(f"   📢 Notification Service Data:")
    notification_service = services["notification"]
    sent_notifications = len([n for n in notification_service.notifications if n["status"] == "sent"])
    print(f"      Notifications Sent: {sent_notifications}")
    
    notification_channels = {}
    for notif in notification_service.notifications:
        channel = notif["channel"]
        notification_channels[channel] = notification_channels.get(channel, 0) + 1
    
    print(f"      Channel Distribution:")
    for channel, count in notification_channels.items():
        print(f"        - {channel}: {count}")
    
    # Event Flow Analysis
    print("\n7. Event flow analysis...")
    
    # Trace events for a specific order
    sample_order_id = created_orders[0] if created_orders else None
    if sample_order_id:
        print(f"   🔍 Event Flow for Order {sample_order_id}:")
        
        related_events = [
            log for log in event_bus.event_log 
            if log["event_data"].get("order_id") == sample_order_id
        ]
        
        for event_log in related_events:
            print(f"      [{event_log['timestamp'][:19]}] {event_log['publisher']} → {event_log['event_type']}")
            if event_log["event_data"].get("amount"):
                print(f"         Amount: ${event_log['event_data']['amount']}")
    
    # System Integration Health
    print("\n8. System integration health...")
    
    total_events_published = sum(s.metrics["events_published"] for s in services.values())
    total_events_processed = sum(s.metrics["events_processed"] for s in services.values())
    total_errors = sum(s.metrics["errors"] for s in services.values())
    
    print(f"   🔧 Integration Metrics:")
    print(f"      Events Published: {total_events_published}")
    print(f"      Events Processed: {total_events_processed}")
    print(f"      Event Bus Events: {stats['total_events']}")
    print(f"      Total Errors: {total_errors}")
    print(f"      Overall Error Rate: {(total_errors / max(total_events_processed, 1)) * 100:.1f}%")
    
    # Calculate service coupling
    coupling_score = stats['total_subscriptions'] / (len(services) * len(stats['event_types']))
    print(f"      Service Coupling Score: {coupling_score:.2f}")
    
    return {
        "event_bus": event_bus,
        "services": services,
        "created_orders": created_orders,
        "stats": stats,
        "integration_metrics": {
            "events_published": total_events_published,
            "events_processed": total_events_processed,
            "total_errors": total_errors,
            "coupling_score": coupling_score
        }
    }

async def main():
    result = await demonstrate_microservices_integration()
    
    print(f"\n✅ SUCCESS! Microservices integration patterns demonstrated!")
    
    print(f"\nMicroservices patterns covered:")
    print(f"- ✓ Service boundaries with event-driven communication")
    print(f"- ✓ Event choreography vs orchestration")
    print(f"- ✓ Inter-service event publishing and subscription")
    print(f"- ✓ Service health monitoring and metrics")
    print(f"- ✓ Event bus for service coordination")
    print(f"- ✓ Cross-service business process flows")
    print(f"- ✓ Service resilience and error handling")
    
    metrics = result["integration_metrics"]
    stats = result["stats"]
    
    print(f"\nSystem performance:")
    print(f"- Services deployed: {len(result['services'])}")
    print(f"- Events published: {metrics['events_published']}")
    print(f"- Events processed: {metrics['events_processed']}")
    print(f"- Event types: {len(stats['event_types'])}")
    print(f"- Service coupling: {metrics['coupling_score']:.2f}")
    print(f"- Error rate: {(metrics['total_errors'] / max(metrics['events_processed'], 1)) * 100:.1f}%")

if __name__ == "__main__":
    asyncio.run(main())