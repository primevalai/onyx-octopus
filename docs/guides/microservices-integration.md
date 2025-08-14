# Microservices Integration Guide

**Building distributed systems with Eventuali event sourcing**

This guide demonstrates how to architect and implement microservices using Eventuali for event sourcing, cross-service communication, and distributed transaction coordination.

## 🎯 What You'll Learn

- ✅ Microservices architecture with event sourcing
- ✅ Cross-service communication patterns
- ✅ Distributed transaction management (Sagas)
- ✅ Event-driven service coordination
- ✅ Service boundaries and data consistency
- ✅ Inter-service event streaming
- ✅ Deployment and scaling strategies

## 📋 Prerequisites

```bash
# Install dependencies
uv add eventuali fastapi uvicorn
uv add httpx aiohttp  # For service-to-service communication
uv add celery redis  # For background processing
uv add docker-compose  # For local development
uv add prometheus-client  # For monitoring
```

## 🏗️ Architecture Overview

### Service Topology

```
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway                              │
│                 (FastAPI + NGINX)                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
          ┌───────────┼───────────┐
          │           │           │
    ┌─────▼─────┐ ┌───▼────┐ ┌────▼─────┐
    │   User    │ │ Order  │ │ Inventory│
    │ Service   │ │Service │ │ Service  │
    └─────┬─────┘ └───┬────┘ └────┬─────┘
          │           │           │
          └───────────┼───────────┘
                      │
         ┌────────────▼────────────┐
         │     Event Streaming     │
         │    (EventStreamer)      │
         └─────────────────────────┘
```

## 👥 User Service

### Domain Model

Create `services/user_service/domain/events.py`:

```python
from eventuali import Event
from pydantic import EmailStr, Field
from datetime import datetime
from typing import Optional

class UserRegistered(Event):
    """User registration event."""
    username: str = Field(..., min_length=3, max_length=50)
    email: EmailStr
    first_name: Optional[str] = None
    last_name: Optional[str] = None

class UserEmailChanged(Event):
    """User email change event."""
    old_email: EmailStr
    new_email: EmailStr

class UserActivated(Event):
    """User account activation event."""
    activated_by: str
    activation_reason: str

class UserDeactivated(Event):
    """User account deactivation event."""
    deactivated_by: str
    deactivation_reason: str

class UserProfileUpdated(Event):
    """User profile update event."""
    first_name: Optional[str] = None
    last_name: Optional[str] = None
    phone: Optional[str] = None
```

Create `services/user_service/domain/aggregate.py`:

```python
from eventuali import Aggregate
from typing import Optional
from datetime import datetime
from .events import (
    UserRegistered, UserEmailChanged, UserActivated, 
    UserDeactivated, UserProfileUpdated
)

class User(Aggregate):
    """User aggregate managing user lifecycle."""
    
    def __init__(self, id: str, version: int = 0):
        super().__init__(id, version)
        self.username: Optional[str] = None
        self.email: Optional[str] = None
        self.first_name: Optional[str] = None
        self.last_name: Optional[str] = None
        self.phone: Optional[str] = None
        self.is_active: bool = False
        self.registration_date: Optional[datetime] = None
        self.activation_date: Optional[datetime] = None
        self.deactivation_date: Optional[datetime] = None
    
    def register(self, username: str, email: str, first_name: str = None, last_name: str = None):
        """Register a new user."""
        if self.version > 0:
            raise ValueError("User already registered")
        
        event = UserRegistered(
            username=username,
            email=email,
            first_name=first_name,
            last_name=last_name
        )
        self.apply(event)
    
    def change_email(self, new_email: str):
        """Change user's email address."""
        if not self.is_active:
            raise ValueError("Cannot change email for inactive user")
        if new_email == self.email:
            raise ValueError("New email same as current")
        
        event = UserEmailChanged(
            old_email=self.email,
            new_email=new_email
        )
        self.apply(event)
    
    def activate(self, activated_by: str, reason: str = "Account activated"):
        """Activate user account."""
        if self.is_active:
            raise ValueError("User already active")
        
        event = UserActivated(
            activated_by=activated_by,
            activation_reason=reason
        )
        self.apply(event)
    
    def deactivate(self, deactivated_by: str, reason: str):
        """Deactivate user account."""
        if not self.is_active:
            raise ValueError("User already inactive")
        
        event = UserDeactivated(
            deactivated_by=deactivated_by,
            deactivation_reason=reason
        )
        self.apply(event)
    
    def update_profile(self, first_name: str = None, last_name: str = None, phone: str = None):
        """Update user profile."""
        if not self.is_active:
            raise ValueError("Cannot update profile for inactive user")
        
        event = UserProfileUpdated(
            first_name=first_name,
            last_name=last_name,
            phone=phone
        )
        self.apply(event)
    
    # Event handlers
    def apply_userregistered(self, event: UserRegistered):
        self.username = event.username
        self.email = event.email
        self.first_name = event.first_name
        self.last_name = event.last_name
        self.registration_date = event.timestamp
        self.is_active = True  # Auto-activate on registration
    
    def apply_useremailchanged(self, event: UserEmailChanged):
        self.email = event.new_email
    
    def apply_useractivated(self, event: UserActivated):
        self.is_active = True
        self.activation_date = event.timestamp
    
    def apply_userdeactivated(self, event: UserDeactivated):
        self.is_active = False
        self.deactivation_date = event.timestamp
    
    def apply_userprofileupdated(self, event: UserProfileUpdated):
        if event.first_name is not None:
            self.first_name = event.first_name
        if event.last_name is not None:
            self.last_name = event.last_name
        if event.phone is not None:
            self.phone = event.phone
```

### Service Implementation

Create `services/user_service/app.py`:

```python
from fastapi import FastAPI, HTTPException, BackgroundTasks, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, EmailStr
from typing import Optional, List
from uuid import uuid4
import asyncio
import logging

from eventuali import EventStore, EventStreamer
from eventuali.streaming import Subscription
from .domain.aggregate import User
from .domain.events import *
from .infrastructure.event_bus import EventBus
from .infrastructure.projections import UserProjectionHandler

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(
    title="User Service",
    description="User management microservice with event sourcing",
    version="1.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global components
event_store: Optional[EventStore] = None
event_streamer: Optional[EventStreamer] = None
event_bus: Optional[EventBus] = None

# Request/Response models
class UserRegistrationRequest(BaseModel):
    username: str
    email: EmailStr
    first_name: Optional[str] = None
    last_name: Optional[str] = None

class UserEmailChangeRequest(BaseModel):
    new_email: EmailStr

class UserProfileUpdateRequest(BaseModel):
    first_name: Optional[str] = None
    last_name: Optional[str] = None
    phone: Optional[str] = None

class UserResponse(BaseModel):
    user_id: str
    username: str
    email: str
    first_name: Optional[str]
    last_name: Optional[str]
    phone: Optional[str]
    is_active: bool
    registration_date: Optional[str]
    version: int

# Startup/Shutdown
@app.on_event("startup")
async def startup_event():
    """Initialize service components."""
    global event_store, event_streamer, event_bus
    
    try:
        # Initialize event store
        connection_string = "postgresql://user:pass@postgres:5432/user_service"
        event_store = await EventStore.create(connection_string)
        
        # Register events
        event_store.register_event_class("UserRegistered", UserRegistered)
        event_store.register_event_class("UserEmailChanged", UserEmailChanged)
        event_store.register_event_class("UserActivated", UserActivated)
        event_store.register_event_class("UserDeactivated", UserDeactivated)
        event_store.register_event_class("UserProfileUpdated", UserProfileUpdated)
        
        # Initialize event streamer
        event_streamer = EventStreamer(capacity=5000)
        
        # Initialize event bus for cross-service communication
        event_bus = EventBus(event_streamer)
        
        # Start projection handler
        asyncio.create_task(start_projection_handler())
        
        logger.info("User service started successfully")
        
    except Exception as e:
        logger.error(f"Failed to start user service: {e}")
        raise

async def get_event_store() -> EventStore:
    """Dependency for event store."""
    if event_store is None:
        raise HTTPException(status_code=503, detail="Event store not initialized")
    return event_store

async def get_event_bus() -> EventBus:
    """Dependency for event bus."""
    if event_bus is None:
        raise HTTPException(status_code=503, detail="Event bus not initialized")
    return event_bus

# API Endpoints
@app.post("/users", response_model=UserResponse, status_code=201)
async def register_user(
    request: UserRegistrationRequest,
    background_tasks: BackgroundTasks,
    store: EventStore = Depends(get_event_store),
    bus: EventBus = Depends(get_event_bus)
):
    """Register a new user."""
    try:
        # Create user aggregate
        user_id = str(uuid4())
        user = User(id=user_id)
        
        # Register user
        user.register(
            username=request.username,
            email=request.email,
            first_name=request.first_name,
            last_name=request.last_name
        )
        
        # Save to event store
        await store.save(user)
        
        # Publish events to other services
        for event in user.get_uncommitted_events():
            await bus.publish_cross_service_event(event)
        
        user.mark_events_as_committed()
        
        # Schedule background tasks
        background_tasks.add_task(send_welcome_email, user.email, user.username)
        
        return UserResponse(
            user_id=user.id,
            username=user.username,
            email=user.email,
            first_name=user.first_name,
            last_name=user.last_name,
            phone=user.phone,
            is_active=user.is_active,
            registration_date=user.registration_date.isoformat() if user.registration_date else None,
            version=user.version
        )
        
    except Exception as e:
        logger.error(f"User registration failed: {e}")
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/users/{user_id}", response_model=UserResponse)
async def get_user(
    user_id: str,
    store: EventStore = Depends(get_event_store)
):
    """Get user by ID."""
    try:
        user = await store.load(User, user_id)
        if not user:
            raise HTTPException(status_code=404, detail="User not found")
        
        return UserResponse(
            user_id=user.id,
            username=user.username,
            email=user.email,
            first_name=user.first_name,
            last_name=user.last_name,
            phone=user.phone,
            is_active=user.is_active,
            registration_date=user.registration_date.isoformat() if user.registration_date else None,
            version=user.version
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting user: {e}")
        raise HTTPException(status_code=500, detail="Internal server error")

@app.put("/users/{user_id}/email")
async def change_user_email(
    user_id: str,
    request: UserEmailChangeRequest,
    store: EventStore = Depends(get_event_store),
    bus: EventBus = Depends(get_event_bus)
):
    """Change user's email address."""
    try:
        user = await store.load(User, user_id)
        if not user:
            raise HTTPException(status_code=404, detail="User not found")
        
        # Change email
        user.change_email(request.new_email)
        
        # Save changes
        await store.save(user)
        
        # Publish events
        for event in user.get_uncommitted_events():
            await bus.publish_cross_service_event(event)
        
        user.mark_events_as_committed()
        
        return {"message": "Email updated successfully"}
        
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Error changing email: {e}")
        raise HTTPException(status_code=500, detail="Internal server error")

# Background tasks
async def send_welcome_email(email: str, username: str):
    """Send welcome email to new user."""
    logger.info(f"Sending welcome email to {username} <{email}>")
    # Implementation would integrate with email service

async def start_projection_handler():
    """Start projection handler for read models."""
    try:
        projection_handler = UserProjectionHandler(event_store)
        subscription = Subscription(
            id="user-projection-handler",
            aggregate_type_filter="User"
        )
        
        receiver = await event_streamer.subscribe(subscription)
        
        async for stream_event in receiver:
            await projection_handler.handle_event(stream_event.event)
        
    except Exception as e:
        logger.error(f"Projection handler error: {e}")

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8001)
```

## 🛍️ Order Service

### Order Domain Model

Create `services/order_service/domain/events.py`:

```python
from eventuali import Event
from pydantic import Field
from decimal import Decimal
from typing import List, Dict, Any
from datetime import datetime

class OrderPlaced(Event):
    """Order placement event."""
    customer_id: str
    items: List[Dict[str, Any]]
    total_amount: Decimal
    shipping_address: Dict[str, str]

class OrderConfirmed(Event):
    """Order confirmation event."""
    confirmed_by: str
    estimated_delivery: datetime

class OrderShipped(Event):
    """Order shipping event."""
    tracking_number: str
    carrier: str
    shipped_at: datetime

class OrderDelivered(Event):
    """Order delivery event."""
    delivered_at: datetime
    delivered_to: str

class OrderCancelled(Event):
    """Order cancellation event."""
    reason: str
    cancelled_by: str
    refund_amount: Decimal

class PaymentProcessed(Event):
    """Payment processing event."""
    payment_id: str
    amount: Decimal
    payment_method: str
    status: str  # 'success', 'failed', 'pending'
```

Create `services/order_service/domain/aggregate.py`:

```python
from eventuali import Aggregate
from typing import List, Dict, Any, Optional
from datetime import datetime
from decimal import Decimal
from .events import *

class Order(Aggregate):
    """Order aggregate managing order lifecycle."""
    
    def __init__(self, id: str, version: int = 0):
        super().__init__(id, version)
        self.customer_id: Optional[str] = None
        self.items: List[Dict[str, Any]] = []
        self.total_amount: Decimal = Decimal('0.00')
        self.shipping_address: Dict[str, str] = {}
        self.status: str = 'draft'  # draft, placed, confirmed, shipped, delivered, cancelled
        self.payment_status: str = 'pending'  # pending, processed, failed
        self.tracking_number: Optional[str] = None
        self.carrier: Optional[str] = None
        self.placed_at: Optional[datetime] = None
        self.shipped_at: Optional[datetime] = None
        self.delivered_at: Optional[datetime] = None
    
    def place_order(
        self, 
        customer_id: str, 
        items: List[Dict[str, Any]], 
        shipping_address: Dict[str, str]
    ):
        """Place a new order."""
        if self.version > 0:
            raise ValueError("Order already placed")
        
        if not items:
            raise ValueError("Order must have at least one item")
        
        # Calculate total
        total = sum(Decimal(str(item['price'])) * item['quantity'] for item in items)
        
        event = OrderPlaced(
            customer_id=customer_id,
            items=items,
            total_amount=total,
            shipping_address=shipping_address
        )
        self.apply(event)
    
    def confirm_order(self, confirmed_by: str, estimated_delivery: datetime):
        """Confirm the order."""
        if self.status != 'placed':
            raise ValueError(f"Cannot confirm order in status: {self.status}")
        
        event = OrderConfirmed(
            confirmed_by=confirmed_by,
            estimated_delivery=estimated_delivery
        )
        self.apply(event)
    
    def ship_order(self, tracking_number: str, carrier: str):
        """Ship the order."""
        if self.status != 'confirmed':
            raise ValueError(f"Cannot ship order in status: {self.status}")
        
        event = OrderShipped(
            tracking_number=tracking_number,
            carrier=carrier,
            shipped_at=datetime.now()
        )
        self.apply(event)
    
    def deliver_order(self, delivered_to: str):
        """Mark order as delivered."""
        if self.status != 'shipped':
            raise ValueError(f"Cannot deliver order in status: {self.status}")
        
        event = OrderDelivered(
            delivered_at=datetime.now(),
            delivered_to=delivered_to
        )
        self.apply(event)
    
    def cancel_order(self, reason: str, cancelled_by: str):
        """Cancel the order."""
        if self.status in ['delivered', 'cancelled']:
            raise ValueError(f"Cannot cancel order in status: {self.status}")
        
        refund_amount = self.total_amount if self.payment_status == 'processed' else Decimal('0.00')
        
        event = OrderCancelled(
            reason=reason,
            cancelled_by=cancelled_by,
            refund_amount=refund_amount
        )
        self.apply(event)
    
    def process_payment(self, payment_id: str, payment_method: str, status: str):
        """Process payment for the order."""
        event = PaymentProcessed(
            payment_id=payment_id,
            amount=self.total_amount,
            payment_method=payment_method,
            status=status
        )
        self.apply(event)
    
    # Event handlers
    def apply_orderplaced(self, event: OrderPlaced):
        self.customer_id = event.customer_id
        self.items = event.items
        self.total_amount = event.total_amount
        self.shipping_address = event.shipping_address
        self.status = 'placed'
        self.placed_at = event.timestamp
    
    def apply_orderconfirmed(self, event: OrderConfirmed):
        self.status = 'confirmed'
    
    def apply_ordershipped(self, event: OrderShipped):
        self.status = 'shipped'
        self.tracking_number = event.tracking_number
        self.carrier = event.carrier
        self.shipped_at = event.shipped_at
    
    def apply_orderdelivered(self, event: OrderDelivered):
        self.status = 'delivered'
        self.delivered_at = event.delivered_at
    
    def apply_ordercancelled(self, event: OrderCancelled):
        self.status = 'cancelled'
    
    def apply_paymentprocessed(self, event: PaymentProcessed):
        self.payment_status = event.status
```

## 🔄 Cross-Service Communication

### Event Bus Implementation

Create `services/shared/infrastructure/event_bus.py`:

```python
import asyncio
import json
import aiohttp
from typing import Dict, Any, Optional, List
from eventuali import Event
from eventuali.streaming import EventStreamer, Subscription
import logging

logger = logging.getLogger(__name__)

class ServiceRegistry:
    """Registry of microservices and their endpoints."""
    
    def __init__(self):
        self.services = {
            'user-service': 'http://user-service:8001',
            'order-service': 'http://order-service:8002',
            'inventory-service': 'http://inventory-service:8003',
            'notification-service': 'http://notification-service:8004',
        }
    
    def get_service_url(self, service_name: str) -> Optional[str]:
        """Get service URL by name."""
        return self.services.get(service_name)
    
    def register_service(self, service_name: str, url: str):
        """Register a new service."""
        self.services[service_name] = url
    
    def get_all_services(self) -> Dict[str, str]:
        """Get all registered services."""
        return self.services.copy()

class EventBus:
    """Event bus for cross-service communication."""
    
    def __init__(self, event_streamer: EventStreamer, service_registry: ServiceRegistry = None):
        self.event_streamer = event_streamer
        self.service_registry = service_registry or ServiceRegistry()
        self.subscriptions: Dict[str, Subscription] = {}
        self.event_handlers: Dict[str, List[callable]] = {}
    
    async def publish_cross_service_event(self, event: Event):
        """Publish event to other services."""
        try:
            # Publish to local event stream
            await self.event_streamer.publish_event(
                event=event,
                stream_position=event.aggregate_version,
                global_position=0  # Would need proper position tracking
            )
            
            # Send HTTP notifications to interested services
            await self._notify_services_via_http(event)
            
        except Exception as e:
            logger.error(f"Error publishing cross-service event: {e}")
    
    async def _notify_services_via_http(self, event: Event):
        """Send HTTP notifications to other services."""
        event_data = {
            'event_type': event.event_type,
            'aggregate_id': event.aggregate_id,
            'aggregate_type': event.aggregate_type,
            'aggregate_version': event.aggregate_version,
            'timestamp': event.timestamp.isoformat() if event.timestamp else None,
            'data': event.to_dict()
        }
        
        # Determine which services should receive this event
        interested_services = self._get_interested_services(event.event_type)
        
        # Send to each interested service
        async with aiohttp.ClientSession() as session:
            tasks = []
            for service_name in interested_services:
                service_url = self.service_registry.get_service_url(service_name)
                if service_url:
                    webhook_url = f"{service_url}/webhooks/events"
                    task = self._send_webhook(session, webhook_url, event_data)
                    tasks.append(task)
            
            if tasks:
                await asyncio.gather(*tasks, return_exceptions=True)
    
    async def _send_webhook(self, session: aiohttp.ClientSession, url: str, data: Dict[str, Any]):
        """Send webhook to a service."""
        try:
            async with session.post(
                url, 
                json=data, 
                timeout=aiohttp.ClientTimeout(total=10)
            ) as response:
                if response.status >= 400:
                    logger.warning(f"Webhook failed: {url} returned {response.status}")
                else:
                    logger.debug(f"Webhook sent successfully: {url}")
        except Exception as e:
            logger.error(f"Error sending webhook to {url}: {e}")
    
    def _get_interested_services(self, event_type: str) -> List[str]:
        """Determine which services are interested in an event type."""
        # This could be configuration-driven or use service discovery
        event_routing = {
            'UserRegistered': ['order-service', 'notification-service'],
            'UserEmailChanged': ['notification-service'],
            'OrderPlaced': ['inventory-service', 'notification-service'],
            'OrderShipped': ['notification-service'],
            'PaymentProcessed': ['order-service', 'notification-service'],
        }
        
        return event_routing.get(event_type, [])
    
    async def subscribe_to_cross_service_events(
        self, 
        event_types: List[str], 
        handler: callable
    ):
        """Subscribe to events from other services."""
        subscription_id = f"cross-service-{len(self.subscriptions)}"
        
        subscription = Subscription(
            id=subscription_id,
            event_type_filter=None  # Listen to all events, filter in handler
        )
        
        self.subscriptions[subscription_id] = subscription
        
        # Register handler
        for event_type in event_types:
            if event_type not in self.event_handlers:
                self.event_handlers[event_type] = []
            self.event_handlers[event_type].append(handler)
        
        # Start listening
        receiver = await self.event_streamer.subscribe(subscription)
        asyncio.create_task(self._handle_cross_service_events(receiver))
    
    async def _handle_cross_service_events(self, receiver):
        """Handle incoming cross-service events."""
        async for stream_event in receiver:
            event = stream_event.event
            event_type = event.event_type
            
            # Call registered handlers
            handlers = self.event_handlers.get(event_type, [])
            for handler in handlers:
                try:
                    await handler(event)
                except Exception as e:
                    logger.error(f"Error in event handler for {event_type}: {e}")

class SagaCoordinator:
    """Coordinator for distributed transactions (Sagas)."""
    
    def __init__(self, event_bus: EventBus):
        self.event_bus = event_bus
        self.active_sagas: Dict[str, Dict[str, Any]] = {}
    
    async def start_order_processing_saga(self, order_id: str, customer_id: str):
        """Start order processing saga."""
        saga_id = f"order-{order_id}-saga"
        
        self.active_sagas[saga_id] = {
            'saga_id': saga_id,
            'order_id': order_id,
            'customer_id': customer_id,
            'status': 'started',
            'steps_completed': [],
            'compensation_needed': []
        }
        
        # Step 1: Reserve inventory
        await self._reserve_inventory(saga_id, order_id)
    
    async def _reserve_inventory(self, saga_id: str, order_id: str):
        """Reserve inventory for order."""
        try:
            # Call inventory service
            async with aiohttp.ClientSession() as session:
                inventory_url = self.event_bus.service_registry.get_service_url('inventory-service')
                url = f"{inventory_url}/inventory/reserve"
                
                async with session.post(url, json={'order_id': order_id}) as response:
                    if response.status == 200:
                        # Success - proceed to next step
                        self.active_sagas[saga_id]['steps_completed'].append('inventory_reserved')
                        await self._process_payment(saga_id, order_id)
                    else:
                        # Failed - start compensation
                        await self._compensate_saga(saga_id, 'inventory_reservation_failed')
        
        except Exception as e:
            logger.error(f"Error reserving inventory for saga {saga_id}: {e}")
            await self._compensate_saga(saga_id, 'inventory_service_error')
    
    async def _process_payment(self, saga_id: str, order_id: str):
        """Process payment for order."""
        try:
            # Payment processing logic
            self.active_sagas[saga_id]['steps_completed'].append('payment_processed')
            await self._confirm_order(saga_id, order_id)
            
        except Exception as e:
            logger.error(f"Error processing payment for saga {saga_id}: {e}")
            await self._compensate_saga(saga_id, 'payment_processing_failed')
    
    async def _confirm_order(self, saga_id: str, order_id: str):
        """Confirm order after successful payment."""
        try:
            # Confirm order
            self.active_sagas[saga_id]['steps_completed'].append('order_confirmed')
            self.active_sagas[saga_id]['status'] = 'completed'
            
            logger.info(f"Saga {saga_id} completed successfully")
            
        except Exception as e:
            logger.error(f"Error confirming order for saga {saga_id}: {e}")
            await self._compensate_saga(saga_id, 'order_confirmation_failed')
    
    async def _compensate_saga(self, saga_id: str, reason: str):
        """Compensate for failed saga."""
        saga = self.active_sagas[saga_id]
        saga['status'] = 'compensating'
        saga['failure_reason'] = reason
        
        # Reverse completed steps
        steps_completed = saga['steps_completed']
        
        if 'payment_processed' in steps_completed:
            await self._reverse_payment(saga_id)
        
        if 'inventory_reserved' in steps_completed:
            await self._release_inventory(saga_id)
        
        saga['status'] = 'failed'
        logger.error(f"Saga {saga_id} failed and compensated: {reason}")
    
    async def _reverse_payment(self, saga_id: str):
        """Reverse payment as compensation."""
        logger.info(f"Reversing payment for saga {saga_id}")
        # Payment reversal logic
    
    async def _release_inventory(self, saga_id: str):
        """Release reserved inventory as compensation."""
        logger.info(f"Releasing inventory for saga {saga_id}")
        # Inventory release logic
```

## 📊 Service Monitoring and Health Checks

Create `services/shared/monitoring/health.py`:

```python
from fastapi import FastAPI, status
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import Dict, Any, Optional
from datetime import datetime
import asyncio
import psutil
import logging

logger = logging.getLogger(__name__)

class HealthStatus(BaseModel):
    """Health status model."""
    status: str  # 'healthy', 'degraded', 'unhealthy'
    timestamp: str
    service_name: str
    version: str
    checks: Dict[str, Any]

class HealthChecker:
    """Health checker for microservices."""
    
    def __init__(self, service_name: str, version: str = "1.0.0"):
        self.service_name = service_name
        self.version = version
        self.checks = {}
        
    def add_check(self, name: str, check_func: callable):
        """Add a health check."""
        self.checks[name] = check_func
    
    async def get_health_status(self) -> HealthStatus:
        """Get current health status."""
        check_results = {}
        overall_status = "healthy"
        
        for check_name, check_func in self.checks.items():
            try:
                result = await check_func()
                check_results[check_name] = result
                
                if result.get('status') != 'healthy':
                    overall_status = 'degraded'
                    
            except Exception as e:
                check_results[check_name] = {
                    'status': 'unhealthy',
                    'error': str(e)
                }
                overall_status = 'unhealthy'
        
        return HealthStatus(
            status=overall_status,
            timestamp=datetime.now().isoformat(),
            service_name=self.service_name,
            version=self.version,
            checks=check_results
        )

def create_health_endpoints(app: FastAPI, health_checker: HealthChecker):
    """Add health check endpoints to FastAPI app."""
    
    @app.get("/health")
    async def health_check():
        """Basic health check."""
        health_status = await health_checker.get_health_status()
        
        status_code = status.HTTP_200_OK
        if health_status.status == 'degraded':
            status_code = status.HTTP_200_OK  # Still return 200 for degraded
        elif health_status.status == 'unhealthy':
            status_code = status.HTTP_503_SERVICE_UNAVAILABLE
        
        return JSONResponse(
            content=health_status.dict(),
            status_code=status_code
        )
    
    @app.get("/health/ready")
    async def readiness_check():
        """Readiness check for Kubernetes."""
        health_status = await health_checker.get_health_status()
        
        if health_status.status == 'unhealthy':
            return JSONResponse(
                content={"status": "not ready"},
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE
            )
        
        return {"status": "ready"}
    
    @app.get("/health/live")
    async def liveness_check():
        """Liveness check for Kubernetes."""
        return {"status": "alive"}

# Common health checks
async def database_health_check(event_store) -> Dict[str, Any]:
    """Check database connectivity."""
    try:
        # Try to get global position as a simple connectivity test
        position = await event_store.get_global_position()
        return {
            'status': 'healthy',
            'global_position': position,
            'response_time_ms': 10  # Would measure actual response time
        }
    except Exception as e:
        return {
            'status': 'unhealthy',
            'error': str(e)
        }

async def memory_health_check() -> Dict[str, Any]:
    """Check memory usage."""
    memory = psutil.virtual_memory()
    memory_percent = memory.percent
    
    status = 'healthy'
    if memory_percent > 90:
        status = 'unhealthy'
    elif memory_percent > 80:
        status = 'degraded'
    
    return {
        'status': status,
        'memory_percent': memory_percent,
        'memory_available_mb': memory.available // (1024 * 1024)
    }

async def event_stream_health_check(event_streamer) -> Dict[str, Any]:
    """Check event stream connectivity."""
    try:
        # Check if we can get current position
        position = await event_streamer.get_global_position()
        return {
            'status': 'healthy',
            'global_position': position
        }
    except Exception as e:
        return {
            'status': 'unhealthy',
            'error': str(e)
        }
```

## 🐳 Docker and Deployment

### Docker Compose Setup

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  # Infrastructure
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: eventuali
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - eventuali-network

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    networks:
      - eventuali-network

  # API Gateway
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - user-service
      - order-service
      - inventory-service
    networks:
      - eventuali-network

  # Microservices
  user-service:
    build:
      context: ./services/user_service
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/eventuali
      - REDIS_URL=redis://redis:6379/0
      - SERVICE_NAME=user-service
    ports:
      - "8001:8000"
    depends_on:
      - postgres
      - redis
    networks:
      - eventuali-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  order-service:
    build:
      context: ./services/order_service
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/eventuali
      - REDIS_URL=redis://redis:6379/1
      - SERVICE_NAME=order-service
    ports:
      - "8002:8000"
    depends_on:
      - postgres
      - redis
      - user-service
    networks:
      - eventuali-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  inventory-service:
    build:
      context: ./services/inventory_service
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/eventuali
      - REDIS_URL=redis://redis:6379/2
      - SERVICE_NAME=inventory-service
    ports:
      - "8003:8000"
    depends_on:
      - postgres
      - redis
    networks:
      - eventuali-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  notification-service:
    build:
      context: ./services/notification_service
      dockerfile: Dockerfile
    environment:
      - REDIS_URL=redis://redis:6379/3
      - EMAIL_SERVICE_URL=http://mock-email-service:8080
      - SERVICE_NAME=notification-service
    ports:
      - "8004:8000"
    depends_on:
      - redis
    networks:
      - eventuali-network

  # Monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
    networks:
      - eventuali-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
    networks:
      - eventuali-network

volumes:
  postgres_data:
  grafana_data:

networks:
  eventuali-network:
    driver: bridge
```

### NGINX Configuration

Create `nginx/nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    upstream user-service {
        server user-service:8000;
    }
    
    upstream order-service {
        server order-service:8000;
    }
    
    upstream inventory-service {
        server inventory-service:8000;
    }
    
    upstream notification-service {
        server notification-service:8000;
    }

    server {
        listen 80;
        server_name localhost;

        # User Service
        location /api/users/ {
            proxy_pass http://user-service/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Order Service
        location /api/orders/ {
            proxy_pass http://order-service/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Inventory Service
        location /api/inventory/ {
            proxy_pass http://inventory-service/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Notification Service
        location /api/notifications/ {
            proxy_pass http://notification-service/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Health checks
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
}
```

## 🔧 Development Scripts

Create `scripts/start-dev.sh`:

```bash
#!/bin/bash

echo "🚀 Starting Eventuali Microservices Development Environment"

# Create networks
docker network create eventuali-network 2>/dev/null || true

# Start infrastructure
echo "Starting infrastructure services..."
docker-compose up -d postgres redis

# Wait for databases
echo "Waiting for databases to be ready..."
sleep 10

# Start services
echo "Starting microservices..."
docker-compose up -d user-service order-service inventory-service notification-service

# Start API gateway
echo "Starting API gateway..."
docker-compose up -d nginx

# Start monitoring
echo "Starting monitoring..."
docker-compose up -d prometheus grafana

echo "✅ All services started!"
echo ""
echo "Service URLs:"
echo "  API Gateway: http://localhost"
echo "  User Service: http://localhost:8001"
echo "  Order Service: http://localhost:8002"
echo "  Inventory Service: http://localhost:8003"
echo "  Notification Service: http://localhost:8004"
echo "  Prometheus: http://localhost:9090"
echo "  Grafana: http://localhost:3000 (admin/admin)"
echo ""
echo "Health checks:"
echo "  curl http://localhost/health"
echo "  curl http://localhost:8001/health"
echo "  curl http://localhost:8002/health"
echo ""
echo "View logs: docker-compose logs -f [service-name]"
```

## 🔗 Related Documentation

- **[FastAPI Integration](fastapi-integration.md)** - REST API patterns
- **[Deployment Guide](../deployment/README.md)** - Production deployment
- **[Performance Guide](../performance/README.md)** - Optimization strategies
- **[Examples](../../examples/12_microservices_integration.py)** - Complete microservices example

---

**Next**: Explore the [deployment patterns](../deployment/README.md) or see the [complete microservices example](../../examples/12_microservices_integration.py).