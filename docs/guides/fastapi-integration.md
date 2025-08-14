# FastAPI Integration Guide

**Building REST APIs with Eventuali event sourcing**

This guide demonstrates how to build production-ready REST APIs using FastAPI with Eventuali for event sourcing, including dependency injection, error handling, and real-time features.

## 🎯 What You'll Learn

- ✅ FastAPI dependency injection with EventStore
- ✅ REST endpoint patterns for event sourcing
- ✅ Request/response handling with aggregates
- ✅ Error handling and validation
- ✅ Real-time WebSocket integration
- ✅ Background task processing
- ✅ Production deployment patterns

## 📋 Prerequisites

```bash
# Install dependencies
uv add fastapi uvicorn eventuali
uv add websockets  # For real-time features
uv add python-multipart  # For file uploads
```

## 🚀 Basic Setup

### Project Structure

```
fastapi-eventuali-app/
├── app/
│   ├── __init__.py
│   ├── main.py              # FastAPI application
│   ├── dependencies.py     # Dependency injection
│   ├── models.py           # Pydantic request/response models
│   ├── routers/
│   │   ├── __init__.py
│   │   ├── users.py        # User endpoints
│   │   └── orders.py       # Order endpoints
│   ├── domain/
│   │   ├── __init__.py
│   │   ├── events.py       # Domain events
│   │   └── aggregates.py   # Domain aggregates
│   ├── services/
│   │   ├── __init__.py
│   │   └── user_service.py # Business services
│   └── websocket/
│       ├── __init__.py
│       └── events.py       # Real-time event broadcasting
├── tests/
├── requirements.txt
└── docker-compose.yml
```

### Core Application Setup

Create `app/main.py`:

```python
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from contextlib import asynccontextmanager
import logging

from .dependencies import get_event_store, get_event_streamer
from .routers import users, orders
from .websocket import events as ws_events

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan management."""
    # Startup
    logger.info("Starting Eventuali FastAPI application")
    
    # Initialize event store and streaming
    store = await get_event_store()
    streamer = await get_event_streamer()
    
    # Start background event processing
    await ws_events.start_event_broadcasting(streamer)
    
    yield
    
    # Shutdown
    logger.info("Shutting down Eventuali FastAPI application")

# Create FastAPI application
app = FastAPI(
    title="Eventuali FastAPI Example",
    description="Event sourcing REST API with real-time features",
    version="1.0.0",
    lifespan=lifespan
)

# CORS middleware for frontend integration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(users.router, prefix="/api/v1/users", tags=["users"])
app.include_router(orders.router, prefix="/api/v1/orders", tags=["orders"])
app.include_router(ws_events.router, prefix="/ws", tags=["websockets"])

@app.get("/")
async def root():
    """Health check endpoint."""
    return {"message": "Eventuali FastAPI application is running"}

@app.get("/health")
async def health_check():
    """Detailed health check."""
    try:
        store = await get_event_store()
        # Could add more health checks here
        return {
            "status": "healthy",
            "components": {
                "event_store": "operational",
                "api": "operational"
            }
        }
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Service unhealthy: {str(e)}")
```

## 🔧 Dependency Injection

Create `app/dependencies.py`:

```python
from functools import lru_cache
from typing import Annotated
from fastapi import Depends, HTTPException
import os

from eventuali import EventStore
from eventuali.streaming import EventStreamer
from eventuali.exceptions import EventualiError

# Global instances (initialized once)
_event_store = None
_event_streamer = None

@lru_cache()
def get_database_url() -> str:
    """Get database URL from environment."""
    url = os.getenv("DATABASE_URL", "sqlite:///events.db")
    return url

async def get_event_store() -> EventStore:
    """Dependency for EventStore instance."""
    global _event_store
    
    if _event_store is None:
        try:
            database_url = get_database_url()
            _event_store = await EventStore.create(database_url)
        except Exception as e:
            raise HTTPException(
                status_code=503, 
                detail=f"Failed to initialize event store: {str(e)}"
            )
    
    return _event_store

async def get_event_streamer() -> EventStreamer:
    """Dependency for EventStreamer instance."""
    global _event_streamer
    
    if _event_streamer is None:
        _event_streamer = EventStreamer(capacity=5000)
    
    return _event_streamer

# Type aliases for cleaner dependency injection
EventStoreDep = Annotated[EventStore, Depends(get_event_store)]
EventStreamerDep = Annotated[EventStreamer, Depends(get_event_streamer)]

class ErrorHandler:
    """Centralized error handling for API endpoints."""
    
    @staticmethod
    def handle_eventuali_error(e: EventualiError) -> HTTPException:
        """Convert Eventuali errors to HTTP exceptions."""
        if "not found" in str(e).lower():
            return HTTPException(status_code=404, detail=str(e))
        elif "already exists" in str(e).lower():
            return HTTPException(status_code=409, detail=str(e))
        elif "invalid" in str(e).lower() or "validation" in str(e).lower():
            return HTTPException(status_code=400, detail=str(e))
        else:
            return HTTPException(status_code=500, detail=str(e))
```

## 📝 Request/Response Models

Create `app/models.py`:

```python
from pydantic import BaseModel, EmailStr, Field
from typing import Optional, List
from datetime import datetime
from decimal import Decimal

# User models
class UserCreateRequest(BaseModel):
    """Request model for creating users."""
    name: str = Field(..., min_length=1, max_length=100, description="User's full name")
    email: EmailStr = Field(..., description="User's email address")

class UserUpdateEmailRequest(BaseModel):
    """Request model for updating user email."""
    new_email: EmailStr = Field(..., description="New email address")

class UserResponse(BaseModel):
    """Response model for user data."""
    user_id: str = Field(..., description="Unique user identifier")
    name: str = Field(..., description="User's full name")
    email: str = Field(..., description="User's email address")
    is_active: bool = Field(..., description="Whether user is active")
    version: int = Field(..., description="Current aggregate version")
    created_at: Optional[datetime] = Field(None, description="When user was created")

class UserListResponse(BaseModel):
    """Response model for user lists."""
    users: List[UserResponse]
    total: int
    page: int
    page_size: int

# Order models
class OrderItemRequest(BaseModel):
    """Request model for order items."""
    product_id: str = Field(..., description="Product identifier")
    quantity: int = Field(..., gt=0, description="Quantity to order")
    price: Decimal = Field(..., gt=0, description="Price per item")

class OrderCreateRequest(BaseModel):
    """Request model for creating orders."""
    customer_id: str = Field(..., description="Customer identifier")
    items: List[OrderItemRequest] = Field(..., min_items=1, description="Order items")
    shipping_address: dict = Field(..., description="Shipping address")

class OrderResponse(BaseModel):
    """Response model for order data."""
    order_id: str
    customer_id: str
    status: str
    total_amount: Decimal
    items: List[dict]
    created_at: datetime
    version: int

# Error models
class ErrorResponse(BaseModel):
    """Standardized error response."""
    error: str = Field(..., description="Error type")
    message: str = Field(..., description="Error message")
    details: Optional[dict] = Field(None, description="Additional error details")

# Event models for WebSocket
class EventNotification(BaseModel):
    """WebSocket event notification."""
    event_type: str
    aggregate_id: str
    aggregate_type: str
    timestamp: datetime
    data: dict
```

## 👥 User Management Endpoints

Create `app/routers/users.py`:

```python
from fastapi import APIRouter, HTTPException, Query, BackgroundTasks
from typing import List, Optional
from uuid import uuid4

from ..dependencies import EventStoreDep, ErrorHandler
from ..models import (
    UserCreateRequest, UserUpdateEmailRequest, UserResponse,
    UserListResponse, ErrorResponse
)
from ..domain.aggregates import User
from ..domain.events import UserRegistered
from ..services.user_service import UserService

router = APIRouter()

@router.post("/", response_model=UserResponse, status_code=201)
async def create_user(
    request: UserCreateRequest,
    background_tasks: BackgroundTasks,
    store: EventStoreDep
):
    """Create a new user."""
    try:
        # Create user aggregate
        user = User(id=str(uuid4()))
        
        # Apply registration event
        event = UserRegistered(
            name=request.name,
            email=request.email
        )
        user.apply(event)
        
        # Save to event store
        await store.save(user)
        user.mark_events_as_committed()
        
        # Schedule background tasks (e.g., send welcome email)
        background_tasks.add_task(send_welcome_email, user.email, user.name)
        
        return UserResponse(
            user_id=user.id,
            name=user.name,
            email=user.email,
            is_active=user.is_active,
            version=user.version,
            created_at=user.get_uncommitted_events()[0].timestamp if user.get_uncommitted_events() else None
        )
        
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.get("/{user_id}", response_model=UserResponse)
async def get_user(user_id: str, store: EventStoreDep):
    """Get user by ID."""
    try:
        user = await store.load(User, user_id)
        if not user:
            raise HTTPException(status_code=404, detail="User not found")
        
        return UserResponse(
            user_id=user.id,
            name=user.name,
            email=user.email,
            is_active=user.is_active,
            version=user.version
        )
        
    except HTTPException:
        raise
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.get("/", response_model=UserListResponse)
async def list_users(
    page: int = Query(1, ge=1, description="Page number"),
    page_size: int = Query(10, ge=1, le=100, description="Items per page"),
    active_only: bool = Query(True, description="Return only active users"),
    store: EventStoreDep
):
    """List users with pagination."""
    try:
        # In a real application, you'd use a projection or read model
        # This is simplified for demonstration
        user_service = UserService(store)
        users = await user_service.list_users(
            page=page,
            page_size=page_size,
            active_only=active_only
        )
        
        user_responses = [
            UserResponse(
                user_id=user.id,
                name=user.name,
                email=user.email,
                is_active=user.is_active,
                version=user.version
            )
            for user in users.items
        ]
        
        return UserListResponse(
            users=user_responses,
            total=users.total,
            page=page,
            page_size=page_size
        )
        
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.put("/{user_id}/email", response_model=dict)
async def update_user_email(
    user_id: str,
    request: UserUpdateEmailRequest,
    store: EventStoreDep
):
    """Update user's email address."""
    try:
        user = await store.load(User, user_id)
        if not user:
            raise HTTPException(status_code=404, detail="User not found")
        
        # Update email (business logic in aggregate)
        user.change_email(request.new_email)
        
        # Save changes
        await store.save(user)
        user.mark_events_as_committed()
        
        return {"message": "Email updated successfully"}
        
    except HTTPException:
        raise
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.delete("/{user_id}", status_code=204)
async def deactivate_user(
    user_id: str,
    reason: Optional[str] = Query(None, description="Reason for deactivation"),
    store: EventStoreDep
):
    """Deactivate user account."""
    try:
        user = await store.load(User, user_id)
        if not user:
            raise HTTPException(status_code=404, detail="User not found")
        
        # Deactivate user
        user.deactivate(reason or "Deactivated via API")
        
        # Save changes
        await store.save(user)
        user.mark_events_as_committed()
        
        # 204 No Content - successful deletion
        
    except HTTPException:
        raise
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.get("/{user_id}/events", response_model=List[dict])
async def get_user_events(user_id: str, store: EventStoreDep):
    """Get event history for user (admin/debugging endpoint)."""
    try:
        events = await store.load_events(user_id)
        
        return [
            {
                "event_type": event.event_type,
                "aggregate_version": event.aggregate_version,
                "timestamp": event.timestamp,
                "data": event.to_dict()
            }
            for event in events
        ]
        
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

# Background task functions
async def send_welcome_email(email: str, name: str):
    """Send welcome email (background task)."""
    # Implementation would integrate with email service
    print(f"Sending welcome email to {name} <{email}>")
    # await email_service.send_welcome_email(email, name)
```

## 🛒 Order Management Endpoints

Create `app/routers/orders.py`:

```python
from fastapi import APIRouter, HTTPException, BackgroundTasks
from uuid import uuid4
from decimal import Decimal

from ..dependencies import EventStoreDep, ErrorHandler
from ..models import OrderCreateRequest, OrderResponse
from ..domain.aggregates import Order
from ..domain.events import OrderCreated

router = APIRouter()

@router.post("/", response_model=OrderResponse, status_code=201)
async def create_order(
    request: OrderCreateRequest,
    background_tasks: BackgroundTasks,
    store: EventStoreDep
):
    """Create a new order."""
    try:
        # Create order aggregate
        order = Order(id=str(uuid4()))
        
        # Calculate total
        items = [item.dict() for item in request.items]
        total = sum(Decimal(str(item['price'])) * item['quantity'] for item in items)
        
        # Apply order creation event
        event = OrderCreated(
            customer_id=request.customer_id,
            items=items,
            total_amount=total,
            shipping_address=request.shipping_address
        )
        order.apply(event)
        
        # Save to event store
        await store.save(order)
        order.mark_events_as_committed()
        
        # Schedule background processing
        background_tasks.add_task(process_order_workflow, order.id)
        
        return OrderResponse(
            order_id=order.id,
            customer_id=order.customer_id,
            status=order.status,
            total_amount=order.total_amount,
            items=order.items,
            created_at=order.created_at,
            version=order.version
        )
        
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.get("/{order_id}", response_model=OrderResponse)
async def get_order(order_id: str, store: EventStoreDep):
    """Get order by ID."""
    try:
        order = await store.load(Order, order_id)
        if not order:
            raise HTTPException(status_code=404, detail="Order not found")
        
        return OrderResponse(
            order_id=order.id,
            customer_id=order.customer_id,
            status=order.status,
            total_amount=order.total_amount,
            items=order.items,
            created_at=order.created_at,
            version=order.version
        )
        
    except HTTPException:
        raise
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

@router.post("/{order_id}/ship", response_model=dict)
async def ship_order(
    order_id: str,
    tracking_number: str,
    store: EventStoreDep
):
    """Ship an order."""
    try:
        order = await store.load(Order, order_id)
        if not order:
            raise HTTPException(status_code=404, detail="Order not found")
        
        # Ship order (business logic in aggregate)
        order.ship(tracking_number)
        
        # Save changes
        await store.save(order)
        order.mark_events_as_committed()
        
        return {
            "message": "Order shipped successfully",
            "tracking_number": tracking_number
        }
        
    except HTTPException:
        raise
    except Exception as e:
        raise ErrorHandler.handle_eventuali_error(e)

async def process_order_workflow(order_id: str):
    """Background order processing workflow."""
    # Implementation would handle:
    # - Inventory reservation
    # - Payment processing
    # - Fulfillment scheduling
    print(f"Processing order workflow for {order_id}")
```

## 🔄 Real-time WebSocket Integration

Create `app/websocket/events.py`:

```python
from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from typing import Set
import json
import asyncio

from eventuali.streaming import EventStreamer, Subscription
from ..dependencies import get_event_streamer

router = APIRouter()

# Active WebSocket connections
active_connections: Set[WebSocket] = set()

class ConnectionManager:
    """Manages WebSocket connections for real-time updates."""
    
    def __init__(self):
        self.active_connections: Set[WebSocket] = set()
    
    async def connect(self, websocket: WebSocket):
        """Accept new WebSocket connection."""
        await websocket.accept()
        self.active_connections.add(websocket)
        print(f"WebSocket connected. Total connections: {len(self.active_connections)}")
    
    def disconnect(self, websocket: WebSocket):
        """Remove WebSocket connection."""
        self.active_connections.discard(websocket)
        print(f"WebSocket disconnected. Total connections: {len(self.active_connections)}")
    
    async def broadcast(self, message: dict):
        """Broadcast message to all connected clients."""
        if not self.active_connections:
            return
        
        message_str = json.dumps(message)
        disconnected = set()
        
        for connection in self.active_connections:
            try:
                await connection.send_text(message_str)
            except Exception:
                disconnected.add(connection)
        
        # Remove disconnected clients
        for connection in disconnected:
            self.disconnect(connection)

# Global connection manager
manager = ConnectionManager()

@router.websocket("/events")
async def websocket_events(websocket: WebSocket):
    """WebSocket endpoint for real-time event updates."""
    await manager.connect(websocket)
    
    try:
        # Keep connection alive and handle client messages
        while True:
            # You can receive filters/preferences from client
            data = await websocket.receive_text()
            client_message = json.loads(data)
            
            # Handle client requests (e.g., filter preferences)
            if client_message.get("type") == "subscribe":
                await websocket.send_text(json.dumps({
                    "type": "subscribed",
                    "message": "Subscribed to real-time events"
                }))
            
    except WebSocketDisconnect:
        manager.disconnect(websocket)

async def start_event_broadcasting(streamer: EventStreamer):
    """Start background task to broadcast events to WebSocket clients."""
    
    # Subscribe to all events for broadcasting
    subscription = Subscription(id="websocket-broadcast")
    receiver = await streamer.subscribe(subscription)
    
    async def broadcast_events():
        """Background task to broadcast events."""
        async for stream_event in receiver:
            event = stream_event.event
            
            # Create WebSocket message
            message = {
                "type": "event",
                "event_type": event.event_type,
                "aggregate_id": event.aggregate_id,
                "aggregate_type": event.aggregate_type,
                "timestamp": event.timestamp.isoformat() if event.timestamp else None,
                "data": {
                    # Include relevant event data
                    "aggregate_version": event.aggregate_version,
                    "global_position": stream_event.global_position
                }
            }
            
            # Broadcast to all connected clients
            await manager.broadcast(message)
    
    # Start background broadcasting
    asyncio.create_task(broadcast_events())
```

## 🧪 Testing

Create `tests/test_users.py`:

```python
import pytest
from httpx import AsyncClient
from fastapi.testclient import TestClient

from app.main import app
from app.dependencies import get_event_store
from eventuali import EventStore

# Test database override
async def get_test_event_store():
    return await EventStore.create("sqlite://:memory:")

@pytest.fixture
async def client():
    """Create test client with test database."""
    app.dependency_overrides[get_event_store] = get_test_event_store
    
    async with AsyncClient(app=app, base_url="http://test") as ac:
        yield ac
    
    app.dependency_overrides.clear()

@pytest.mark.asyncio
async def test_create_user(client: AsyncClient):
    """Test user creation endpoint."""
    response = await client.post(
        "/api/v1/users/",
        json={
            "name": "John Doe",
            "email": "john@example.com"
        }
    )
    
    assert response.status_code == 201
    data = response.json()
    assert data["name"] == "John Doe"
    assert data["email"] == "john@example.com"
    assert data["is_active"] is True
    assert "user_id" in data

@pytest.mark.asyncio
async def test_get_user(client: AsyncClient):
    """Test user retrieval endpoint."""
    # Create user first
    create_response = await client.post(
        "/api/v1/users/",
        json={
            "name": "Jane Doe",
            "email": "jane@example.com"
        }
    )
    user_id = create_response.json()["user_id"]
    
    # Get user
    response = await client.get(f"/api/v1/users/{user_id}")
    
    assert response.status_code == 200
    data = response.json()
    assert data["user_id"] == user_id
    assert data["name"] == "Jane Doe"

@pytest.mark.asyncio
async def test_update_user_email(client: AsyncClient):
    """Test email update endpoint."""
    # Create user
    create_response = await client.post(
        "/api/v1/users/",
        json={
            "name": "Bob Smith",
            "email": "bob@example.com"
        }
    )
    user_id = create_response.json()["user_id"]
    
    # Update email
    response = await client.put(
        f"/api/v1/users/{user_id}/email",
        json={"new_email": "bob.smith@company.com"}
    )
    
    assert response.status_code == 200
    assert response.json()["message"] == "Email updated successfully"
    
    # Verify update
    get_response = await client.get(f"/api/v1/users/{user_id}")
    assert get_response.json()["email"] == "bob.smith@company.com"

@pytest.mark.asyncio
async def test_user_not_found(client: AsyncClient):
    """Test 404 handling."""
    response = await client.get("/api/v1/users/nonexistent")
    assert response.status_code == 404
    assert "not found" in response.json()["detail"].lower()
```

## 🚀 Production Deployment

### Docker Configuration

Create `Dockerfile`:

```dockerfile
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \\
    build-essential \\
    curl \\
    && rm -rf /var/lib/apt/lists/*

# Install Rust (for Eventuali)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install UV
RUN pip install uv

# Set working directory
WORKDIR /app

# Copy requirements and install dependencies
COPY requirements.txt .
RUN uv pip sync requirements.txt --system

# Copy application code
COPY . .

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \\
    CMD curl -f http://localhost:8000/health || exit 1

# Run application
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8000:8000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/eventuali
    depends_on:
      - db
      - redis
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: eventuali
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

### Running the Application

```bash
# Development
uv run uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

# Production with Docker
docker-compose up -d

# Check health
curl http://localhost:8000/health
```

## 📊 Performance Considerations

### Production Optimizations

1. **Connection Pooling**: Configure appropriate database pool sizes
2. **Background Tasks**: Use Celery or similar for heavy processing
3. **Caching**: Implement Redis caching for read models
4. **Load Balancing**: Use multiple API instances behind load balancer

### API Performance Monitoring

```python
from fastapi import Request
import time

@app.middleware("http")
async def add_process_time_header(request: Request, call_next):
    """Add processing time to response headers."""
    start_time = time.time()
    response = await call_next(request)
    process_time = time.time() - start_time
    response.headers["X-Process-Time"] = str(process_time)
    return response
```

## 🔗 Related Documentation

- **[API Reference](../api/README.md)** - Eventuali API documentation
- **[Performance Guide](../performance/README.md)** - Optimization strategies
- **[Deployment Guide](../deployment/README.md)** - Production deployment
- **[Examples](../../examples/12_microservices_integration.py)** - Microservices patterns

---

**Next**: Try the [Django Integration Guide](django-integration.md) or explore [microservices patterns](../../examples/12_microservices_integration.py).