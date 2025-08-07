#!/usr/bin/env python3
"""
Projections Example

This example demonstrates projection patterns for building read models:
- Event-driven projection building
- Multiple projection types (single, composite, analytical)
- Real-time vs batch projection updates
- Projection versioning and rebuilding
- Query-optimized read models
"""

import asyncio
import sys
import os
from typing import Optional, Dict, List, Any, Set
from datetime import datetime, timezone
from collections import defaultdict

# Add the python package to the path
sys.path.insert(
    0, os.path.join(os.path.dirname(__file__), "..", "eventuali-python", "python")
)

from eventuali import EventStore
from eventuali.aggregate import User  # Use built-in User aggregate
from eventuali.event import UserRegistered, Event


# Domain Events for the example
class ProductViewed(Event):
    """Product viewed event."""

    user_id: str
    product_id: str
    category: str
    price: float
    view_duration_seconds: int


class ProductPurchased(Event):
    """Product purchased event."""

    user_id: str
    product_id: str
    category: str
    price: float
    quantity: int


class CartItemAdded(Event):
    """Item added to cart event."""

    user_id: str
    product_id: str
    price: float
    quantity: int


class CartItemRemoved(Event):
    """Item removed from cart event."""

    user_id: str
    product_id: str
    quantity: int


class OrderPlaced(Event):
    """Order placed event."""

    user_id: str
    order_id: str
    total_amount: float
    item_count: int


class OrderCompleted(Event):
    """Order completed event."""

    user_id: str
    order_id: str
    completion_date: str


# Simple Shopping Aggregate for generating events
class ShoppingSession:
    """Simple shopping session aggregate to generate test events."""

    def __init__(self, user_id: str):
        self.user_id = user_id
        self.id = f"session-{user_id}"
        self.cart_items: Dict[str, int] = {}
        self.events: List[Event] = []
        self.version = 0

    def view_product(self, product_id: str, category: str, price: float, duration: int):
        """User views a product."""
        event = ProductViewed(
            user_id=self.user_id,
            product_id=product_id,
            category=category,
            price=price,
            view_duration_seconds=duration,
        )
        self.events.append(event)
        self.version += 1
        return event

    def add_to_cart(self, product_id: str, price: float, quantity: int = 1):
        """Add item to cart."""
        self.cart_items[product_id] = self.cart_items.get(product_id, 0) + quantity

        event = CartItemAdded(
            user_id=self.user_id, product_id=product_id, price=price, quantity=quantity
        )
        self.events.append(event)
        self.version += 1
        return event

    def remove_from_cart(self, product_id: str, quantity: int = 1):
        """Remove item from cart."""
        if product_id in self.cart_items:
            self.cart_items[product_id] = max(0, self.cart_items[product_id] - quantity)
            if self.cart_items[product_id] == 0:
                del self.cart_items[product_id]

        event = CartItemRemoved(
            user_id=self.user_id, product_id=product_id, quantity=quantity
        )
        self.events.append(event)
        self.version += 1
        return event

    def purchase_product(
        self, product_id: str, category: str, price: float, quantity: int = 1
    ):
        """Purchase a product directly."""
        event = ProductPurchased(
            user_id=self.user_id,
            product_id=product_id,
            category=category,
            price=price,
            quantity=quantity,
        )
        self.events.append(event)
        self.version += 1
        return event

    def place_order(self, order_id: str, total_amount: float):
        """Place an order for cart items."""
        event = OrderPlaced(
            user_id=self.user_id,
            order_id=order_id,
            total_amount=total_amount,
            item_count=sum(self.cart_items.values()),
        )
        self.events.append(event)
        self.version += 1

        # Clear cart after order
        self.cart_items.clear()
        return event

    def complete_order(self, order_id: str, completion_date: str = None):
        """Complete an order."""
        if not completion_date:
            completion_date = datetime.now(timezone.utc).isoformat()

        event = OrderCompleted(
            user_id=self.user_id, order_id=order_id, completion_date=completion_date
        )
        self.events.append(event)
        self.version += 1
        return event


# User Activity Projection (Single Entity Projection)
class UserActivityProjection:
    """Projection tracking individual user activity."""

    def __init__(self, user_id: str):
        self.user_id = user_id
        self.total_views = 0
        self.total_purchases = 0
        self.total_spent = 0.0
        self.favorite_category = ""
        self.last_activity = ""
        self.view_duration_total = 0
        self.cart_additions = 0
        self.orders_placed = 0
        self.orders_completed = 0

        # Track categories
        self.category_views: Dict[str, int] = defaultdict(int)
        self.category_purchases: Dict[str, int] = defaultdict(int)

        # Track products
        self.viewed_products: Set[str] = set()
        self.purchased_products: Set[str] = set()

    def apply_event(self, event: Event) -> bool:
        """Apply an event to update the projection."""
        updated = False

        if hasattr(event, "user_id") and event.user_id != self.user_id:
            return False  # Not for this user

        if isinstance(event, ProductViewed):
            self.total_views += 1
            self.view_duration_total += event.view_duration_seconds
            self.category_views[event.category] += 1
            self.viewed_products.add(event.product_id)
            self.last_activity = "product_view"
            updated = True

        elif isinstance(event, ProductPurchased):
            self.total_purchases += 1
            self.total_spent += event.price * event.quantity
            self.category_purchases[event.category] += event.quantity
            self.purchased_products.add(event.product_id)
            self.last_activity = "product_purchase"
            updated = True

        elif isinstance(event, CartItemAdded):
            self.cart_additions += 1
            self.last_activity = "cart_addition"
            updated = True

        elif isinstance(event, OrderPlaced):
            self.orders_placed += 1
            self.last_activity = "order_placed"
            updated = True

        elif isinstance(event, OrderCompleted):
            self.orders_completed += 1
            self.last_activity = "order_completed"
            updated = True

        # Update favorite category
        if self.category_views:
            self.favorite_category = max(
                self.category_views.items(), key=lambda x: x[1]
            )[0]

        return updated

    def get_summary(self) -> Dict[str, Any]:
        """Get a summary of user activity."""
        avg_view_duration = self.view_duration_total / max(self.total_views, 1)
        conversion_rate = (self.total_purchases / max(self.total_views, 1)) * 100

        return {
            "user_id": self.user_id,
            "total_views": self.total_views,
            "total_purchases": self.total_purchases,
            "total_spent": round(self.total_spent, 2),
            "orders_placed": self.orders_placed,
            "orders_completed": self.orders_completed,
            "favorite_category": self.favorite_category,
            "avg_view_duration": round(avg_view_duration, 1),
            "conversion_rate": round(conversion_rate, 2),
            "unique_products_viewed": len(self.viewed_products),
            "unique_products_purchased": len(self.purchased_products),
            "last_activity": self.last_activity,
        }


# Category Sales Projection (Analytical Projection)
class CategorySalesProjection:
    """Projection for category-level sales analytics."""

    def __init__(self):
        self.categories: Dict[str, Dict[str, Any]] = defaultdict(
            lambda: {
                "total_views": 0,
                "total_purchases": 0,
                "total_revenue": 0.0,
                "unique_viewers": set(),
                "unique_buyers": set(),
                "avg_price": 0.0,
                "products": set(),
            }
        )
        self.last_updated = datetime.now(timezone.utc).isoformat()

    def apply_event(self, event: Event) -> bool:
        """Apply an event to update category analytics."""
        updated = False

        if isinstance(event, ProductViewed):
            cat_data = self.categories[event.category]
            cat_data["total_views"] += 1
            cat_data["unique_viewers"].add(event.user_id)
            cat_data["products"].add(event.product_id)
            updated = True

        elif isinstance(event, ProductPurchased):
            cat_data = self.categories[event.category]
            cat_data["total_purchases"] += event.quantity
            cat_data["total_revenue"] += event.price * event.quantity
            cat_data["unique_buyers"].add(event.user_id)
            cat_data["products"].add(event.product_id)

            # Update average price
            if cat_data["total_purchases"] > 0:
                cat_data["avg_price"] = (
                    cat_data["total_revenue"] / cat_data["total_purchases"]
                )

            updated = True

        if updated:
            self.last_updated = datetime.now(timezone.utc).isoformat()

        return updated

    def get_category_summary(self, category: str) -> Optional[Dict[str, Any]]:
        """Get summary for a specific category."""
        if category not in self.categories:
            return None

        cat_data = self.categories[category]

        return {
            "category": category,
            "total_views": cat_data["total_views"],
            "total_purchases": cat_data["total_purchases"],
            "total_revenue": round(cat_data["total_revenue"], 2),
            "unique_viewers": len(cat_data["unique_viewers"]),
            "unique_buyers": len(cat_data["unique_buyers"]),
            "unique_products": len(cat_data["products"]),
            "avg_price": round(cat_data["avg_price"], 2),
            "conversion_rate": round(
                (
                    len(cat_data["unique_buyers"])
                    / max(len(cat_data["unique_viewers"]), 1)
                )
                * 100,
                2,
            ),
        }

    def get_all_categories(self) -> List[Dict[str, Any]]:
        """Get summary for all categories."""
        return [self.get_category_summary(cat) for cat in self.categories.keys()]


# Real-time Dashboard Projection (Composite Projection)
class RealTimeDashboard:
    """Real-time dashboard combining multiple data sources."""

    def __init__(self):
        self.metrics = {
            "total_active_users": 0,
            "total_views_today": 0,
            "total_purchases_today": 0,
            "total_revenue_today": 0.0,
            "avg_order_value": 0.0,
            "top_categories": [],
            "recent_activities": [],
            "conversion_funnel": {
                "views": 0,
                "cart_additions": 0,
                "orders_placed": 0,
                "orders_completed": 0,
            },
        }

        self.active_users: Set[str] = set()
        self.today = datetime.now().date()
        self.recent_activity_limit = 10

    def apply_event(self, event: Event) -> bool:
        """Apply event to update dashboard metrics."""
        updated = False
        event_date = datetime.now().date()  # In real app, extract from event timestamp

        # Only process today's events for daily metrics
        if event_date != self.today:
            return False

        if hasattr(event, "user_id"):
            self.active_users.add(event.user_id)
            self.metrics["total_active_users"] = len(self.active_users)

        if isinstance(event, ProductViewed):
            self.metrics["total_views_today"] += 1
            self.metrics["conversion_funnel"]["views"] += 1
            self._add_recent_activity(event.user_id, "viewed", event.product_id)
            updated = True

        elif isinstance(event, ProductPurchased):
            self.metrics["total_purchases_today"] += event.quantity
            revenue = event.price * event.quantity
            self.metrics["total_revenue_today"] += revenue

            # Update average order value
            if self.metrics["total_purchases_today"] > 0:
                self.metrics["avg_order_value"] = (
                    self.metrics["total_revenue_today"]
                    / self.metrics["total_purchases_today"]
                )

            self._add_recent_activity(
                event.user_id, "purchased", event.product_id, revenue
            )
            updated = True

        elif isinstance(event, CartItemAdded):
            self.metrics["conversion_funnel"]["cart_additions"] += event.quantity
            self._add_recent_activity(event.user_id, "added_to_cart", event.product_id)
            updated = True

        elif isinstance(event, OrderPlaced):
            self.metrics["conversion_funnel"]["orders_placed"] += 1
            self._add_recent_activity(
                event.user_id, "placed_order", event.order_id, event.total_amount
            )
            updated = True

        elif isinstance(event, OrderCompleted):
            self.metrics["conversion_funnel"]["orders_completed"] += 1
            self._add_recent_activity(event.user_id, "completed_order", event.order_id)
            updated = True

        return updated

    def _add_recent_activity(
        self, user_id: str, action: str, item_id: str, value: float = None
    ):
        """Add to recent activities list."""
        activity = {
            "user_id": user_id,
            "action": action,
            "item_id": item_id,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }

        if value is not None:
            activity["value"] = round(value, 2)

        self.metrics["recent_activities"].insert(0, activity)

        # Keep only recent activities
        if len(self.metrics["recent_activities"]) > self.recent_activity_limit:
            self.metrics["recent_activities"] = self.metrics["recent_activities"][
                : self.recent_activity_limit
            ]

    def get_dashboard_data(self) -> Dict[str, Any]:
        """Get current dashboard data."""
        # Calculate conversion rates
        funnel = self.metrics["conversion_funnel"]
        view_to_cart_rate = (funnel["cart_additions"] / max(funnel["views"], 1)) * 100
        cart_to_order_rate = (
            funnel["orders_placed"] / max(funnel["cart_additions"], 1)
        ) * 100
        order_completion_rate = (
            funnel["orders_completed"] / max(funnel["orders_placed"], 1)
        ) * 100

        return {
            "summary": {
                "active_users": self.metrics["total_active_users"],
                "views": self.metrics["total_views_today"],
                "purchases": self.metrics["total_purchases_today"],
                "revenue": round(self.metrics["total_revenue_today"], 2),
                "avg_order_value": round(self.metrics["avg_order_value"], 2),
            },
            "conversion_rates": {
                "view_to_cart": round(view_to_cart_rate, 1),
                "cart_to_order": round(cart_to_order_rate, 1),
                "order_completion": round(order_completion_rate, 1),
            },
            "funnel": funnel.copy(),
            "recent_activities": self.metrics["recent_activities"].copy(),
        }


# Projection Manager
class ProjectionManager:
    """Manager for handling multiple projections."""

    def __init__(self):
        self.user_projections: Dict[str, UserActivityProjection] = {}
        self.category_projection = CategorySalesProjection()
        self.dashboard = RealTimeDashboard()
        self.processed_events = 0

    async def process_event(self, event: Event) -> Dict[str, bool]:
        """Process an event through all projections."""
        results = {}

        # Process user-specific projections
        if hasattr(event, "user_id"):
            user_id = event.user_id
            if user_id not in self.user_projections:
                self.user_projections[user_id] = UserActivityProjection(user_id)

            results["user_projection"] = self.user_projections[user_id].apply_event(
                event
            )

        # Process category projection
        results["category_projection"] = self.category_projection.apply_event(event)

        # Process dashboard
        results["dashboard"] = self.dashboard.apply_event(event)

        self.processed_events += 1
        return results

    def get_user_summary(self, user_id: str) -> Optional[Dict[str, Any]]:
        """Get user activity summary."""
        projection = self.user_projections.get(user_id)
        return projection.get_summary() if projection else None

    def get_category_summaries(self) -> List[Dict[str, Any]]:
        """Get all category summaries."""
        return self.category_projection.get_all_categories()

    def get_dashboard_data(self) -> Dict[str, Any]:
        """Get dashboard data."""
        return self.dashboard.get_dashboard_data()

    def get_system_stats(self) -> Dict[str, Any]:
        """Get projection system statistics."""
        return {
            "processed_events": self.processed_events,
            "user_projections": len(self.user_projections),
            "categories_tracked": len(self.category_projection.categories),
            "active_users_today": len(self.dashboard.active_users),
        }


async def demonstrate_projections():
    """Demonstrate projection patterns and read model building."""
    print("=== Projections Example ===\n")

    event_store = await EventStore.create("sqlite://:memory:")
    projection_manager = ProjectionManager()

    # Setup test users
    print("1. Setting up test users and data...")

    users = {}
    for user_id in ["alice", "bob", "charlie"]:
        user = User(id=user_id)
        user.apply(
            UserRegistered(
                name=f"{user_id.title()} Johnson", email=f"{user_id}@example.com"
            )
        )
        await event_store.save(user)
        user.mark_events_as_committed()
        users[user_id] = user

    print(f"   ✓ Created {len(users)} test users")

    # Generate test shopping activity
    print("\n2. Generating shopping activity...")

    # Alice - Heavy electronics shopper
    alice_session = ShoppingSession("alice")
    alice_events = [
        alice_session.view_product("laptop", "electronics", 1200.0, 45),
        alice_session.view_product("mouse", "electronics", 80.0, 15),
        alice_session.add_to_cart("laptop", 1200.0, 1),
        alice_session.add_to_cart("mouse", 80.0, 1),
        alice_session.place_order("order-001", 1280.0),
        alice_session.complete_order("order-001"),
        alice_session.view_product("keyboard", "electronics", 150.0, 20),
        alice_session.purchase_product("keyboard", "electronics", 150.0, 1),
    ]

    # Bob - Books and electronics
    bob_session = ShoppingSession("bob")
    bob_events = [
        bob_session.view_product("python-book", "books", 45.0, 30),
        bob_session.view_product("tablet", "electronics", 300.0, 25),
        bob_session.add_to_cart("python-book", 45.0, 2),
        bob_session.view_product("notebook", "office", 15.0, 10),
        bob_session.add_to_cart("notebook", 15.0, 3),
        bob_session.place_order("order-002", 135.0),
        bob_session.complete_order("order-002"),
    ]

    # Charlie - Browsing but not purchasing much
    charlie_session = ShoppingSession("charlie")
    charlie_events = [
        charlie_session.view_product("laptop", "electronics", 1200.0, 60),
        charlie_session.view_product("mouse", "electronics", 80.0, 10),
        charlie_session.view_product("desk", "furniture", 400.0, 20),
        charlie_session.add_to_cart("mouse", 80.0, 1),
        charlie_session.remove_from_cart("mouse", 1),
        charlie_session.view_product("chair", "furniture", 250.0, 15),
    ]

    all_events = alice_events + bob_events + charlie_events
    print(f"   ✓ Generated {len(all_events)} shopping events")

    # Process events through projections
    print("\n3. Processing events through projections...")

    start_time = datetime.now()

    for event in all_events:
        await projection_manager.process_event(event)
        # Optionally save events to event store for persistence
        # await event_store.save_event(event)

    processing_time = (datetime.now() - start_time).total_seconds() * 1000

    print(f"   ✓ Processed {len(all_events)} events in {processing_time:.1f}ms")
    print(
        f"   ✓ Processing rate: {(len(all_events) / (processing_time / 1000)):.0f} events/sec"
    )

    # Display projection results
    print("\n4. User activity projections...")

    for user_id in ["alice", "bob", "charlie"]:
        summary = projection_manager.get_user_summary(user_id)
        if summary:
            print(f"   {summary['user_id'].title()}:")
            print(
                f"     - Views: {summary['total_views']}, Purchases: {summary['total_purchases']}"
            )
            print(f"     - Total spent: ${summary['total_spent']}")
            print(f"     - Favorite category: {summary['favorite_category']}")
            print(f"     - Conversion rate: {summary['conversion_rate']}%")
            print(f"     - Avg view duration: {summary['avg_view_duration']}s")

    # Display category analytics
    print("\n5. Category analytics projection...")

    category_summaries = projection_manager.get_category_summaries()
    category_summaries.sort(key=lambda x: x["total_revenue"], reverse=True)

    for cat_summary in category_summaries:
        print(f"   {cat_summary['category'].title()}:")
        print(
            f"     - Views: {cat_summary['total_views']}, Purchases: {cat_summary['total_purchases']}"
        )
        print(f"     - Revenue: ${cat_summary['total_revenue']}")
        print(f"     - Avg price: ${cat_summary['avg_price']}")
        print(f"     - Conversion: {cat_summary['conversion_rate']}%")
        print(f"     - Unique products: {cat_summary['unique_products']}")

    # Display real-time dashboard
    print("\n6. Real-time dashboard projection...")

    dashboard_data = projection_manager.get_dashboard_data()

    print("   Summary:")
    summary = dashboard_data["summary"]
    for metric, value in summary.items():
        formatted_value = (
            f"${value}" if "revenue" in metric or "value" in metric else value
        )
        print(f"     - {metric.replace('_', ' ').title()}: {formatted_value}")

    print("   Conversion Funnel:")
    funnel = dashboard_data["funnel"]
    rates = dashboard_data["conversion_rates"]
    print(f"     - Views: {funnel['views']}")
    print(
        f"     - Cart Additions: {funnel['cart_additions']} ({rates['view_to_cart']}% conversion)"
    )
    print(
        f"     - Orders Placed: {funnel['orders_placed']} ({rates['cart_to_order']}% conversion)"
    )
    print(
        f"     - Orders Completed: {funnel['orders_completed']} ({rates['order_completion']}% conversion)"
    )

    print("   Recent Activities:")
    for activity in dashboard_data["recent_activities"][:5]:  # Show top 5
        action_desc = activity["action"].replace("_", " ").title()
        item = activity["item_id"]
        user = activity["user_id"]
        value_str = f" (${activity['value']})" if "value" in activity else ""
        print(f"     - {user} {action_desc.lower()}: {item}{value_str}")

    # System statistics
    print("\n7. Projection system statistics...")

    stats = projection_manager.get_system_stats()
    print("   System Performance:")
    print(f"     - Total events processed: {stats['processed_events']}")
    print(f"     - User projections created: {stats['user_projections']}")
    print(f"     - Categories tracked: {stats['categories_tracked']}")
    print(f"     - Processing time: {processing_time:.1f}ms")
    print(
        f"     - Events per second: {(stats['processed_events'] / (processing_time / 1000)):.0f}"
    )

    # Projection rebuild demonstration
    print("\n8. Testing projection rebuild...")

    # Simulate rebuilding a user projection
    original_alice = projection_manager.get_user_summary("alice")

    # Create new projection and replay events
    new_alice_projection = UserActivityProjection("alice")
    alice_specific_events = [
        e for e in all_events if hasattr(e, "user_id") and e.user_id == "alice"
    ]

    rebuild_start = datetime.now()
    for event in alice_specific_events:
        new_alice_projection.apply_event(event)
    rebuild_time = (datetime.now() - rebuild_start).total_seconds() * 1000

    rebuilt_alice = new_alice_projection.get_summary()

    print("   Rebuild Results for Alice:")
    print(f"     - Original total spent: ${original_alice['total_spent']}")
    print(f"     - Rebuilt total spent: ${rebuilt_alice['total_spent']}")
    print(
        f"     - Data consistency: {'✓ MATCH' if original_alice['total_spent'] == rebuilt_alice['total_spent'] else '❌ MISMATCH'}"
    )
    print(
        f"     - Rebuild time: {rebuild_time:.1f}ms for {len(alice_specific_events)} events"
    )

    return {
        "projection_manager": projection_manager,
        "users": users,
        "events": all_events,
        "stats": stats,
        "processing_time": processing_time,
        "dashboard_data": dashboard_data,
    }


async def main():
    result = await demonstrate_projections()

    print("\n✅ SUCCESS! Projection patterns demonstrated!")

    print("\nProjection patterns covered:")
    print("- ✓ Single entity projections (user activity)")
    print("- ✓ Analytical projections (category analytics)")
    print("- ✓ Composite projections (real-time dashboard)")
    print("- ✓ Event-driven projection updates")
    print("- ✓ Real-time metrics and conversion funnels")
    print("- ✓ Projection rebuilding from event replay")
    print("- ✓ Query-optimized read models")

    stats = result["stats"]
    processing_time = result["processing_time"]

    print("\nPerformance characteristics:")
    print(f"- Processed {stats['processed_events']} events in {processing_time:.1f}ms")
    print(
        f"- Processing rate: {(stats['processed_events'] / (processing_time / 1000)):.0f} events/sec"
    )
    print(f"- Created {stats['user_projections']} user projections")
    print(f"- Tracked {stats['categories_tracked']} product categories")
    print(f"- Real-time dashboard with {stats['active_users_today']} active users")


if __name__ == "__main__":
    asyncio.run(main())
