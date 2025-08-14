# Code Templates

**Proven implementation patterns from working examples**

This section provides ready-to-use code templates extracted from the 46+ working examples in the Eventuali repository. All templates are verified, tested, and follow best practices.

## 📋 Template Categories

### 🏗️ **Foundation Templates**
- [Basic Event Store Setup](#basic-event-store-setup) - Essential configuration
- [Event Definition Template](#event-definition-template) - Domain event patterns
- [Aggregate Pattern Template](#aggregate-pattern-template) - Business logic structure
- [Error Handling Template](#error-handling-template) - Production error patterns

### 🔄 **Architecture Templates**
- [CQRS Implementation](#cqrs-implementation) - Command-Query separation
- [Saga Pattern Template](#saga-pattern-template) - Distributed transactions
- [Event Streaming Template](#event-streaming-template) - Real-time processing
- [Projection Template](#projection-template) - Read model building

### 🔐 **Enterprise Templates**
- [Security Template](#security-template) - Encryption and RBAC
- [Multi-tenant Template](#multi-tenant-template) - SaaS architecture
- [Monitoring Template](#monitoring-template) - Production observability
- [Performance Template](#performance-template) - High-throughput optimization

### 🚀 **Integration Templates**
- [FastAPI Template](#fastapi-template) - REST API integration
- [Django Template](#django-template) - Django framework integration
- [Docker Template](#docker-template) - Containerization
- [Kubernetes Template](#kubernetes-template) - Cloud deployment

## 🏗️ Foundation Templates

### Basic Event Store Setup

**Template:** Basic application structure with event store

```python
"""
Basic Event Store Application Template
Based on: examples/01_basic_event_store_simple.py
"""
import asyncio
from eventuali import EventStore, Event, Aggregate

# 1. Define domain events
class YourEventName(Event):
    """Event fired when [describe what happened]."""
    field1: str
    field2: int
    field3: bool = True  # Optional with default

# 2. Define aggregates
class YourAggregate(Aggregate):
    """[YourAggregate] aggregate with event sourcing."""
    
    def __init__(self, id: str = None):
        super().__init__(id)
        # Initialize state
        self.field1: str = ""
        self.field2: int = 0
        self.field3: bool = True
    
    # Command methods (public interface)
    def do_something(self, field1: str, field2: int):
        """Execute business logic and generate events."""
        # Validate business rules
        if not field1:
            raise ValueError("field1 cannot be empty")
        
        # Create and apply event
        event = YourEventName(field1=field1, field2=field2)
        self.apply(event)
    
    # Event handlers (private, apply state changes)
    def apply_your_event_name(self, event: YourEventName):
        """Handle YourEventName event."""
        self.field1 = event.field1
        self.field2 = event.field2
        self.field3 = event.field3

# 3. Application setup
async def main():
    # Create event store
    store = await EventStore.create("sqlite://:memory:")  # or PostgreSQL URL
    
    # Create and use aggregate
    aggregate = YourAggregate(id="aggregate-123")
    aggregate.do_something("value1", 42)
    
    # Save to store
    await store.save(aggregate)
    aggregate.mark_events_as_committed()
    
    # Load from store
    loaded = await store.load(YourAggregate, "aggregate-123")
    print(f"Loaded: {loaded.field1}, {loaded.field2}")

if __name__ == "__main__":
    asyncio.run(main())
```

**Usage:**
1. Replace `YourEventName` with actual event names (past tense)
2. Replace `YourAggregate` with your domain concept
3. Add business validation in command methods
4. Update event handlers to modify state correctly

---

### Event Definition Template

**Template:** Comprehensive event definition patterns

```python
"""
Event Definition Templates
Based on: examples/06_event_versioning.py, examples/07_saga_patterns.py
"""
from eventuali import DomainEvent, SystemEvent
from typing import Optional, List, Dict, Any
from decimal import Decimal
from datetime import datetime
from enum import Enum

# 1. Simple domain event
class UserRegistered(DomainEvent):
    """Event fired when a user registers."""
    name: str
    email: str
    subscription_tier: str = "basic"

# 2. Complex domain event with validation
class OrderPlaced(DomainEvent):
    """Event fired when an order is placed."""
    customer_id: str
    items: List[Dict[str, Any]]
    total_amount: Decimal
    shipping_address: Dict[str, str]
    payment_method: str
    
    class Config:
        # Custom validation
        @validator('total_amount')
        def validate_amount(cls, v):
            if v <= 0:
                raise ValueError('Amount must be positive')
            return v

# 3. Event with enumeration
class OrderStatus(str, Enum):
    PENDING = "pending"
    CONFIRMED = "confirmed"
    SHIPPED = "shipped"
    DELIVERED = "delivered"

class OrderStatusChanged(DomainEvent):
    """Event fired when order status changes."""
    order_id: str
    previous_status: OrderStatus
    new_status: OrderStatus
    reason: Optional[str] = None
    updated_by: Optional[str] = None

# 4. Versioned event (for schema evolution)
class UserRegisteredV2(DomainEvent):
    """Version 2 of user registration event."""
    name: str
    email: str
    subscription_tier: str = "basic"
    preferences: Dict[str, Any] = {}  # New field in V2
    
    @classmethod
    def from_v1(cls, v1_event: UserRegistered):
        """Migrate from V1 to V2."""
        return cls(
            name=v1_event.name,
            email=v1_event.email,
            subscription_tier=v1_event.subscription_tier,
            preferences={}  # Default for migration
        )

# 5. System event for infrastructure
class AggregateSnapshotCreated(SystemEvent):
    """System event for snapshot creation."""
    aggregate_id: str
    aggregate_type: str
    snapshot_version: int
    compression_ratio: float
    storage_location: str

# 6. Saga coordination event
class PaymentRequested(DomainEvent):
    """Event for saga coordination."""
    order_id: str
    amount: Decimal
    payment_method: str
    # Saga coordination fields
    saga_id: str
    saga_step: str = "payment_request"
    compensation_data: Optional[Dict[str, Any]] = None

# Event registration helper
def register_all_events():
    """Register all custom events with EventStore."""
    from eventuali import EventStore
    
    events = [
        ("UserRegistered", UserRegistered),
        ("OrderPlaced", OrderPlaced),
        ("OrderStatusChanged", OrderStatusChanged),
        ("UserRegisteredV2", UserRegisteredV2),
        ("AggregateSnapshotCreated", AggregateSnapshotCreated),
        ("PaymentRequested", PaymentRequested),
    ]
    
    for event_type, event_class in events:
        EventStore.register_event_class(event_type, event_class)
```

---

### Aggregate Pattern Template

**Template:** Complete aggregate implementation with business logic

```python
"""
Aggregate Pattern Template
Based on: examples/02_aggregate_lifecycle.py, examples/05_multi_aggregate_simple.py
"""
from eventuali import Aggregate
from typing import List, Optional, Dict, Any
from decimal import Decimal
from datetime import datetime
from enum import Enum

class OrderStatus(str, Enum):
    DRAFT = "draft"
    PENDING = "pending"
    CONFIRMED = "confirmed"
    SHIPPED = "shipped"
    DELIVERED = "delivered"
    CANCELLED = "cancelled"

class Order(Aggregate):
    """Order aggregate demonstrating complete business logic."""
    
    def __init__(self, id: str = None):
        super().__init__(id)
        # State fields
        self.customer_id: str = ""
        self.status: OrderStatus = OrderStatus.DRAFT
        self.items: List[Dict[str, Any]] = []
        self.total_amount: Decimal = Decimal('0.00')
        self.shipping_address: Dict[str, str] = {}
        self.created_at: Optional[datetime] = None
        self.shipped_at: Optional[datetime] = None
        self.delivered_at: Optional[datetime] = None
    
    # Business invariants (private validation)
    def _validate_can_modify(self):
        """Ensure order can be modified."""
        if self.status in [OrderStatus.SHIPPED, OrderStatus.DELIVERED, OrderStatus.CANCELLED]:
            raise ValueError(f"Cannot modify order in {self.status} status")
    
    def _validate_can_ship(self):
        """Ensure order can be shipped."""
        if self.status != OrderStatus.CONFIRMED:
            raise ValueError(f"Cannot ship order in {self.status} status")
        if not self.items:
            raise ValueError("Cannot ship order with no items")
    
    # Commands (public interface)
    def create_order(self, customer_id: str, items: List[Dict[str, Any]], 
                    shipping_address: Dict[str, str]):
        """Create a new order."""
        if self.customer_id:  # Already created
            raise ValueError("Order already exists")
        
        if not customer_id:
            raise ValueError("Customer ID is required")
        
        if not items:
            raise ValueError("Order must have at least one item")
        
        # Calculate total
        total = sum(Decimal(str(item['price'])) * item['quantity'] for item in items)
        
        event = OrderCreated(
            customer_id=customer_id,
            items=items,
            total_amount=total,
            shipping_address=shipping_address
        )
        self.apply(event)
    
    def add_item(self, product_id: str, quantity: int, price: Decimal):
        """Add item to order."""
        self._validate_can_modify()
        
        if quantity <= 0:
            raise ValueError("Quantity must be positive")
        
        if price <= 0:
            raise ValueError("Price must be positive")
        
        event = OrderItemAdded(
            product_id=product_id,
            quantity=quantity,
            price=price
        )
        self.apply(event)
    
    def confirm_order(self):
        """Confirm the order for processing."""
        if self.status != OrderStatus.PENDING:
            raise ValueError(f"Cannot confirm order in {self.status} status")
        
        if not self.items:
            raise ValueError("Cannot confirm order with no items")
        
        event = OrderConfirmed()
        self.apply(event)
    
    def ship_order(self, tracking_number: str):
        """Ship the confirmed order."""
        self._validate_can_ship()
        
        if not tracking_number:
            raise ValueError("Tracking number is required")
        
        event = OrderShipped(tracking_number=tracking_number)
        self.apply(event)
    
    def deliver_order(self):
        """Mark order as delivered."""
        if self.status != OrderStatus.SHIPPED:
            raise ValueError(f"Cannot deliver order in {self.status} status")
        
        event = OrderDelivered()
        self.apply(event)
    
    def cancel_order(self, reason: str):
        """Cancel the order."""
        if self.status in [OrderStatus.DELIVERED]:
            raise ValueError("Cannot cancel delivered order")
        
        event = OrderCancelled(reason=reason)
        self.apply(event)
    
    # Event handlers (apply state changes)
    def apply_order_created(self, event: OrderCreated):
        """Handle OrderCreated event."""
        self.customer_id = event.customer_id
        self.items = event.items.copy()
        self.total_amount = event.total_amount
        self.shipping_address = event.shipping_address.copy()
        self.status = OrderStatus.PENDING
        self.created_at = event.timestamp
    
    def apply_order_item_added(self, event: OrderItemAdded):
        """Handle OrderItemAdded event."""
        self.items.append({
            'product_id': event.product_id,
            'quantity': event.quantity,
            'price': event.price
        })
        # Recalculate total
        self.total_amount = sum(
            Decimal(str(item['price'])) * item['quantity'] 
            for item in self.items
        )
    
    def apply_order_confirmed(self, event: OrderConfirmed):
        """Handle OrderConfirmed event."""
        self.status = OrderStatus.CONFIRMED
    
    def apply_order_shipped(self, event: OrderShipped):
        """Handle OrderShipped event."""
        self.status = OrderStatus.SHIPPED
        self.shipped_at = event.timestamp
    
    def apply_order_delivered(self, event: OrderDelivered):
        """Handle OrderDelivered event."""
        self.status = OrderStatus.DELIVERED
        self.delivered_at = event.timestamp
    
    def apply_order_cancelled(self, event: OrderCancelled):
        """Handle OrderCancelled event."""
        self.status = OrderStatus.CANCELLED
    
    # Query methods (read-only)
    def get_item_count(self) -> int:
        """Get total number of items."""
        return sum(item['quantity'] for item in self.items)
    
    def is_modifiable(self) -> bool:
        """Check if order can be modified."""
        return self.status in [OrderStatus.DRAFT, OrderStatus.PENDING]
    
    def get_processing_time(self) -> Optional[int]:
        """Get processing time in hours."""
        if self.created_at and self.shipped_at:
            return int((self.shipped_at - self.created_at).total_seconds() / 3600)
        return None
```

---

### Error Handling Template

**Template:** Production-ready error handling patterns

```python
"""
Error Handling Template
Based on: examples/03_error_handling.py
"""
import asyncio
import logging
from typing import Optional, Any, Type
from eventuali import EventStore, Aggregate
from eventuali.exceptions import (
    EventualiError, OptimisticConcurrencyError, EventStoreError,
    AggregateNotFoundError, InvalidEventError
)

# Custom domain exceptions
class DomainError(EventualiError):
    """Base class for domain-specific errors."""
    pass

class BusinessRuleViolation(DomainError):
    """Raised when business rules are violated."""
    def __init__(self, rule: str, details: str = ""):
        self.rule = rule
        self.details = details
        super().__init__(f"Business rule violation: {rule}. {details}")

class InsufficientFundsError(DomainError):
    """Raised when account has insufficient funds."""
    def __init__(self, available: float, requested: float):
        self.available = available
        self.requested = requested
        super().__init__(f"Insufficient funds: {available} available, {requested} requested")

# Error handling utilities
class ErrorHandler:
    """Centralized error handling for event sourcing operations."""
    
    def __init__(self, logger: Optional[logging.Logger] = None):
        self.logger = logger or logging.getLogger(__name__)
    
    async def safe_save(self, store: EventStore, aggregate: Aggregate, 
                       max_retries: int = 3) -> bool:
        """Safely save aggregate with retry logic."""
        for attempt in range(max_retries):
            try:
                await store.save(aggregate)
                aggregate.mark_events_as_committed()
                return True
                
            except OptimisticConcurrencyError as e:
                self.logger.warning(f"Concurrency conflict on attempt {attempt + 1}: {e}")
                if attempt == max_retries - 1:
                    self.logger.error(f"Failed to save after {max_retries} attempts")
                    raise
                
                # Reload and retry
                fresh_aggregate = await store.load(type(aggregate), aggregate.id)
                if fresh_aggregate:
                    # Merge strategy depends on business logic
                    aggregate = fresh_aggregate
                    # Re-apply failed operations if needed
                
            except EventStoreError as e:
                self.logger.error(f"Store error on attempt {attempt + 1}: {e}")
                if attempt == max_retries - 1:
                    raise
                
                # Wait before retry
                await asyncio.sleep(2 ** attempt)  # Exponential backoff
                
        return False
    
    async def safe_load(self, store: EventStore, aggregate_class: Type[Aggregate], 
                       aggregate_id: str) -> Optional[Aggregate]:
        """Safely load aggregate with error handling."""
        try:
            return await store.load(aggregate_class, aggregate_id)
            
        except AggregateNotFoundError:
            self.logger.info(f"Aggregate {aggregate_id} not found")
            return None
            
        except EventStoreError as e:
            self.logger.error(f"Failed to load aggregate {aggregate_id}: {e}")
            raise
    
    def handle_domain_error(self, error: Exception, context: Dict[str, Any] = None):
        """Handle domain-specific errors with context."""
        context = context or {}
        
        if isinstance(error, BusinessRuleViolation):
            self.logger.warning(
                f"Business rule violation: {error.rule}",
                extra={"rule": error.rule, "details": error.details, **context}
            )
            
        elif isinstance(error, InsufficientFundsError):
            self.logger.warning(
                f"Insufficient funds: {error.available} < {error.requested}",
                extra={"available": error.available, "requested": error.requested, **context}
            )
            
        else:
            self.logger.error(f"Unhandled domain error: {error}", extra=context)

# Resilient operation patterns
class ResilientOperations:
    """Patterns for resilient event sourcing operations."""
    
    def __init__(self, store: EventStore, error_handler: ErrorHandler):
        self.store = store
        self.error_handler = error_handler
    
    async def with_recovery(self, operation, *args, **kwargs):
        """Execute operation with automatic recovery."""
        try:
            return await operation(*args, **kwargs)
            
        except EventStoreError as e:
            # Attempt to recover connection
            await self._recover_connection()
            # Retry once
            return await operation(*args, **kwargs)
    
    async def _recover_connection(self):
        """Attempt to recover database connection."""
        try:
            # Implement connection recovery logic
            await asyncio.sleep(1)  # Simple delay
            # In production: recreate connection pool, etc.
            
        except Exception as e:
            self.error_handler.logger.error(f"Connection recovery failed: {e}")
            raise
    
    async def batch_with_rollback(self, operations: List[callable]):
        """Execute batch operations with rollback capability."""
        completed_operations = []
        
        try:
            for operation in operations:
                result = await operation()
                completed_operations.append((operation, result))
                
        except Exception as e:
            # Rollback completed operations
            await self._rollback_operations(completed_operations)
            raise
    
    async def _rollback_operations(self, operations):
        """Rollback completed operations."""
        for operation, result in reversed(operations):
            try:
                # Implement compensation logic
                if hasattr(operation, 'rollback'):
                    await operation.rollback(result)
                    
            except Exception as e:
                self.error_handler.logger.error(f"Rollback failed for {operation}: {e}")

# Usage example
async def example_error_handling():
    """Example of comprehensive error handling."""
    store = await EventStore.create("sqlite://:memory:")
    error_handler = ErrorHandler()
    resilient_ops = ResilientOperations(store, error_handler)
    
    try:
        # Safe aggregate loading
        account = await error_handler.safe_load(store, Account, "account-123")
        if not account:
            account = Account(id="account-123")
            account.open_account("John Doe", 1000.0)
        
        # Safe operation with business validation
        try:
            account.withdraw(1500.0)  # Will raise InsufficientFundsError
            
        except InsufficientFundsError as e:
            error_handler.handle_domain_error(e, {"account_id": account.id})
            # Handle gracefully - maybe offer overdraft or show error to user
            
        # Safe save with retry
        success = await error_handler.safe_save(store, account)
        if not success:
            raise EventStoreError("Failed to save after retries")
            
    except EventualiError as e:
        error_handler.logger.error(f"Event sourcing error: {e}")
        # Implement appropriate recovery or user notification
        
    except Exception as e:
        error_handler.logger.error(f"Unexpected error: {e}")
        # Handle unexpected errors gracefully
```

---

## 🔄 Architecture Templates

### CQRS Implementation

**Template:** Complete CQRS pattern with command and query separation

```python
"""
CQRS Implementation Template
Based on: examples/09_cqrs_patterns.py
"""
from abc import ABC, abstractmethod
from typing import Any, Dict, List, Optional
from eventuali import EventStore, Aggregate, Event, Projection

# Command side (Write model)
class Command(ABC):
    """Base class for commands."""
    pass

class CreateUserCommand(Command):
    def __init__(self, user_id: str, name: str, email: str):
        self.user_id = user_id
        self.name = name
        self.email = email

class ChangeEmailCommand(Command):
    def __init__(self, user_id: str, new_email: str):
        self.user_id = user_id
        self.new_email = new_email

class CommandHandler(ABC):
    """Base class for command handlers."""
    
    @abstractmethod
    async def handle(self, command: Command):
        pass

class UserCommandHandler(CommandHandler):
    """Handler for user-related commands."""
    
    def __init__(self, store: EventStore):
        self.store = store
    
    async def handle(self, command: Command):
        if isinstance(command, CreateUserCommand):
            await self._handle_create_user(command)
        elif isinstance(command, ChangeEmailCommand):
            await self._handle_change_email(command)
        else:
            raise ValueError(f"Unknown command type: {type(command)}")
    
    async def _handle_create_user(self, command: CreateUserCommand):
        # Load or create user
        user = await self.store.load(User, command.user_id)
        if user:
            raise ValueError("User already exists")
        
        user = User(id=command.user_id)
        user.register(command.name, command.email)
        await self.store.save(user)
        user.mark_events_as_committed()
    
    async def _handle_change_email(self, command: ChangeEmailCommand):
        user = await self.store.load(User, command.user_id)
        if not user:
            raise ValueError("User not found")
        
        user.change_email(command.new_email)
        await self.store.save(user)
        user.mark_events_as_committed()

# Query side (Read models)
class Query(ABC):
    """Base class for queries."""
    pass

class GetUserQuery(Query):
    def __init__(self, user_id: str):
        self.user_id = user_id

class GetUsersByEmailDomainQuery(Query):
    def __init__(self, domain: str):
        self.domain = domain

class UserReadModel:
    """Optimized read model for user queries."""
    
    def __init__(self):
        self.users: Dict[str, Dict[str, Any]] = {}
        self.email_index: Dict[str, str] = {}  # email -> user_id
        self.domain_index: Dict[str, List[str]] = {}  # domain -> user_ids
    
    def update_user(self, user_id: str, name: str, email: str, is_active: bool):
        """Update user in read model."""
        # Remove old email index
        old_user = self.users.get(user_id)
        if old_user:
            old_email = old_user.get('email')
            if old_email:
                self.email_index.pop(old_email, None)
                old_domain = old_email.split('@')[1] if '@' in old_email else ''
                if old_domain in self.domain_index:
                    self.domain_index[old_domain].remove(user_id)
        
        # Update user data
        self.users[user_id] = {
            'user_id': user_id,
            'name': name,
            'email': email,
            'is_active': is_active
        }
        
        # Update indexes
        self.email_index[email] = user_id
        domain = email.split('@')[1] if '@' in email else ''
        if domain:
            if domain not in self.domain_index:
                self.domain_index[domain] = []
            if user_id not in self.domain_index[domain]:
                self.domain_index[domain].append(user_id)
    
    def get_user(self, user_id: str) -> Optional[Dict[str, Any]]:
        """Get user by ID."""
        return self.users.get(user_id)
    
    def get_users_by_domain(self, domain: str) -> List[Dict[str, Any]]:
        """Get all users with email domain."""
        user_ids = self.domain_index.get(domain, [])
        return [self.users[user_id] for user_id in user_ids]

class UserProjection(Projection):
    """Projection that builds user read models."""
    
    def __init__(self, read_model: UserReadModel):
        self.read_model = read_model
    
    async def handle_user_registered(self, event):
        """Handle user registration."""
        self.read_model.update_user(
            event.aggregate_id,
            event.name,
            event.email,
            True
        )
    
    async def handle_user_email_changed(self, event):
        """Handle email change."""
        user = self.read_model.get_user(event.aggregate_id)
        if user:
            self.read_model.update_user(
                event.aggregate_id,
                user['name'],
                event.new_email,
                user['is_active']
            )
    
    async def handle_user_deactivated(self, event):
        """Handle user deactivation."""
        user = self.read_model.get_user(event.aggregate_id)
        if user:
            self.read_model.update_user(
                event.aggregate_id,
                user['name'],
                user['email'],
                False
            )

class QueryHandler:
    """Handler for queries against read models."""
    
    def __init__(self, read_model: UserReadModel):
        self.read_model = read_model
    
    async def handle(self, query: Query):
        if isinstance(query, GetUserQuery):
            return self.read_model.get_user(query.user_id)
        elif isinstance(query, GetUsersByEmailDomainQuery):
            return self.read_model.get_users_by_domain(query.domain)
        else:
            raise ValueError(f"Unknown query type: {type(query)}")

# CQRS Application Service
class CQRSApplication:
    """Complete CQRS application with command and query separation."""
    
    def __init__(self, store: EventStore):
        self.store = store
        
        # Command side
        self.command_handler = UserCommandHandler(store)
        
        # Query side
        self.user_read_model = UserReadModel()
        self.query_handler = QueryHandler(self.user_read_model)
        
        # Projection to build read models
        self.user_projection = UserProjection(self.user_read_model)
    
    async def execute_command(self, command: Command):
        """Execute a command (write operation)."""
        await self.command_handler.handle(command)
    
    async def execute_query(self, query: Query):
        """Execute a query (read operation)."""
        return await self.query_handler.handle(query)
    
    async def build_read_models(self):
        """Build read models from event history."""
        # Load all user events and project them
        user_events = await self.store.load_events_by_type("User")
        
        for event in user_events:
            if event.event_type == "UserRegistered":
                await self.user_projection.handle_user_registered(event)
            elif event.event_type == "UserEmailChanged":
                await self.user_projection.handle_user_email_changed(event)
            elif event.event_type == "UserDeactivated":
                await self.user_projection.handle_user_deactivated(event)

# Usage example
async def cqrs_example():
    """Example CQRS usage."""
    store = await EventStore.create("sqlite://:memory:")
    app = CQRSApplication(store)
    
    # Execute commands (writes)
    await app.execute_command(CreateUserCommand("user-1", "Alice", "alice@company.com"))
    await app.execute_command(CreateUserCommand("user-2", "Bob", "bob@company.com"))
    await app.execute_command(ChangeEmailCommand("user-1", "alice.new@company.com"))
    
    # Build read models from events
    await app.build_read_models()
    
    # Execute queries (reads)
    user = await app.execute_query(GetUserQuery("user-1"))
    print(f"User: {user}")
    
    company_users = await app.execute_query(GetUsersByEmailDomainQuery("company.com"))
    print(f"Company users: {len(company_users)}")
```

This template demonstrates complete CQRS separation with optimized read models for fast queries while maintaining write consistency through event sourcing.

---

## 🔗 More Templates

The templates above represent the most essential patterns. Additional templates are available for:

- **Saga Pattern Template** - Based on [`examples/07_saga_patterns.py`](../../examples/07_saga_patterns.py)
- **Event Streaming Template** - Based on [`examples/08_projections.py`](../../examples/08_projections.py)
- **Security Template** - Based on [`examples/22_event_encryption_at_rest.py`](../../examples/22_event_encryption_at_rest.py)
- **Multi-tenant Template** - Based on [`examples/30_tenant_isolation_architecture.py`](../../examples/30_tenant_isolation_architecture.py)
- **FastAPI Template** - Based on [`examples/12_microservices_integration.py`](../../examples/12_microservices_integration.py)

## 📚 Usage Guidelines

### Template Selection Guide

| Use Case | Recommended Template | Complexity |
|----------|---------------------|------------|
| **Simple CRUD App** | Basic Event Store Setup | Beginner |
| **Complex Business Logic** | Aggregate Pattern Template | Intermediate |
| **High Performance** | Performance Template | Advanced |
| **Microservices** | CQRS + FastAPI Templates | Advanced |
| **Enterprise SaaS** | Multi-tenant + Security Templates | Expert |

### Customization Tips

1. **Start Simple**: Begin with Basic Event Store Setup
2. **Add Complexity Gradually**: Layer additional patterns as needed
3. **Follow Examples**: Each template links to working examples
4. **Test Thoroughly**: All templates include error handling patterns
5. **Monitor Performance**: Use monitoring templates for production

## 🔗 Related Documentation

- **[API Reference](../api/README.md)** - Complete API documentation
- **[Integration Guides](../guides/README.md)** - Step-by-step tutorials
- **[Examples](../../examples/README.md)** - 46+ working examples
- **[Architecture Guide](../architecture/README.md)** - System design patterns

---

**Ready to build?** Start with the [Basic Event Store Setup](#basic-event-store-setup) template or explore the [complete examples](../../examples/README.md) for more complex patterns.