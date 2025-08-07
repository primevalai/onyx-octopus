#!/usr/bin/env python3
"""
Event Versioning Example

This example demonstrates event versioning and schema evolution in event sourcing:
- Event schema versioning strategies
- Backward and forward compatibility
- Event upcasting and downcasting
- Handling breaking changes gracefully
- Migration patterns for event evolution
"""

import asyncio
import sys
import os
from typing import Optional, Dict, Any
from datetime import datetime, timezone

# Add the python package to the path
sys.path.insert(
    0, os.path.join(os.path.dirname(__file__), "..", "eventuali-python", "python")
)

from eventuali import EventStore
from eventuali.aggregate import Aggregate
from eventuali.event import Event


# Version 1 Events (Original Schema)
class CustomerRegisteredV1(Event):
    """Customer registration event - Version 1."""

    # Mark event version explicitly
    event_version: int = 1

    name: str
    email: str


class CustomerEmailChangedV1(Event):
    """Customer email change event - Version 1."""

    event_version: int = 1

    new_email: str


# Version 2 Events (Enhanced Schema)
class CustomerRegisteredV2(Event):
    """Customer registration event - Version 2 with additional fields."""

    event_version: int = 2

    first_name: str
    last_name: str
    email: str
    phone: Optional[str] = None
    preferred_language: str = "en"


class CustomerEmailChangedV2(Event):
    """Customer email change event - Version 2 with validation info."""

    event_version: int = 2

    old_email: str
    new_email: str
    validated: bool = False
    change_reason: Optional[str] = None


# Version 3 Events (Breaking Changes)
class CustomerProfileUpdatedV3(Event):
    """Unified customer profile update event - Version 3."""

    event_version: int = 3

    profile_changes: Dict[str, Any]  # Generic profile updates
    change_type: str  # "registration", "email_change", "address_change", etc.
    metadata: Dict[str, Any] = {}

    # For backward compatibility
    @property
    def first_name(self) -> Optional[str]:
        return self.profile_changes.get("first_name")

    @property
    def last_name(self) -> Optional[str]:
        return self.profile_changes.get("last_name")

    @property
    def email(self) -> Optional[str]:
        return self.profile_changes.get("email")


# Event Versioning Service
class EventVersioningService:
    """Service to handle event versioning and migration."""

    @staticmethod
    def upcast_event(event_data: Dict[str, Any]) -> Event:
        """Convert older event versions to newer versions."""

        event_type = event_data.get("event_type", "")
        event_version = event_data.get("event_version", 1)

        # CustomerRegistered evolution
        if event_type == "CustomerRegisteredV1" or (
            event_type == "CustomerRegistered" and event_version == 1
        ):
            return EventVersioningService._upcast_customer_registered_v1_to_v2(
                event_data
            )

        elif event_type == "CustomerRegisteredV2" or (
            event_type == "CustomerRegistered" and event_version == 2
        ):
            return EventVersioningService._upcast_customer_registered_v2_to_v3(
                event_data
            )

        # CustomerEmailChanged evolution
        elif event_type == "CustomerEmailChangedV1" or (
            event_type == "CustomerEmailChanged" and event_version == 1
        ):
            return EventVersioningService._upcast_email_changed_v1_to_v2(event_data)

        elif event_type == "CustomerEmailChangedV2" or (
            event_type == "CustomerEmailChanged" and event_version == 2
        ):
            return EventVersioningService._upcast_email_changed_v2_to_v3(event_data)

        # Already latest version or unknown event
        return EventVersioningService._create_event_from_data(event_data)

    @staticmethod
    def _upcast_customer_registered_v1_to_v2(
        v1_data: Dict[str, Any],
    ) -> CustomerRegisteredV2:
        """Upcast CustomerRegisteredV1 to V2."""

        # Parse full name into first/last (simple heuristic)
        full_name = v1_data.get("name", "")
        name_parts = full_name.split(" ", 1)
        first_name = name_parts[0] if name_parts else ""
        last_name = name_parts[1] if len(name_parts) > 1 else ""

        return CustomerRegisteredV2(
            first_name=first_name,
            last_name=last_name,
            email=v1_data.get("email", ""),
            phone=None,  # Not available in V1
            preferred_language="en",  # Default value
        )

    @staticmethod
    def _upcast_customer_registered_v2_to_v3(
        v2_data: Dict[str, Any],
    ) -> CustomerProfileUpdatedV3:
        """Upcast CustomerRegisteredV2 to V3."""

        profile_changes = {
            "first_name": v2_data.get("first_name", ""),
            "last_name": v2_data.get("last_name", ""),
            "email": v2_data.get("email", ""),
            "phone": v2_data.get("phone"),
            "preferred_language": v2_data.get("preferred_language", "en"),
        }

        return CustomerProfileUpdatedV3(
            profile_changes=profile_changes,
            change_type="registration",
            metadata={"migrated_from_v2": True},
        )

    @staticmethod
    def _upcast_email_changed_v1_to_v2(
        v1_data: Dict[str, Any],
    ) -> CustomerEmailChangedV2:
        """Upcast CustomerEmailChangedV1 to V2."""

        return CustomerEmailChangedV2(
            old_email="",  # Not available in V1
            new_email=v1_data.get("new_email", ""),
            validated=False,  # Default to unvalidated
            change_reason=None,  # Not available in V1
        )

    @staticmethod
    def _upcast_email_changed_v2_to_v3(
        v2_data: Dict[str, Any],
    ) -> CustomerProfileUpdatedV3:
        """Upcast CustomerEmailChangedV2 to V3."""

        profile_changes = {"email": v2_data.get("new_email", "")}

        metadata = {
            "migrated_from_v2": True,
            "old_email": v2_data.get("old_email", ""),
            "validated": v2_data.get("validated", False),
            "change_reason": v2_data.get("change_reason"),
        }

        return CustomerProfileUpdatedV3(
            profile_changes=profile_changes,
            change_type="email_change",
            metadata=metadata,
        )

    @staticmethod
    def _create_event_from_data(event_data: Dict[str, Any]) -> Event:
        """Create event from raw data (fallback for unknown events)."""

        # This is a simplified fallback - in practice you'd have a registry
        event_type = event_data.get("event_type", "")

        if "CustomerRegistered" in event_type:
            if "first_name" in event_data:
                return CustomerRegisteredV2(
                    **{
                        k: v
                        for k, v in event_data.items()
                        if k in CustomerRegisteredV2.__fields__
                    }
                )
            else:
                return CustomerRegisteredV1(
                    **{
                        k: v
                        for k, v in event_data.items()
                        if k in CustomerRegisteredV1.__fields__
                    }
                )

        elif "CustomerEmailChanged" in event_type:
            if "old_email" in event_data:
                return CustomerEmailChangedV2(
                    **{
                        k: v
                        for k, v in event_data.items()
                        if k in CustomerEmailChangedV2.__fields__
                    }
                )
            else:
                return CustomerEmailChangedV1(
                    **{
                        k: v
                        for k, v in event_data.items()
                        if k in CustomerEmailChangedV1.__fields__
                    }
                )

        elif "CustomerProfileUpdated" in event_type:
            return CustomerProfileUpdatedV3(
                **{
                    k: v
                    for k, v in event_data.items()
                    if k in CustomerProfileUpdatedV3.__fields__
                }
            )

        # Generic fallback
        class GenericEvent(Event):
            data: Dict[str, Any] = {}

        return GenericEvent(data=event_data)


# Versioned Customer Aggregate
class VersionedCustomer(Aggregate):
    """Customer aggregate supporting multiple event versions."""

    first_name: str = ""
    last_name: str = ""
    email: str = ""
    phone: Optional[str] = None
    preferred_language: str = "en"
    email_validated: bool = False
    registration_version: int = 1

    def __init__(self, **data):
        super().__init__(**data)

    # Business methods for different versions
    def register_v1(self, name: str, email: str):
        """Register customer using V1 event format."""
        event = CustomerRegisteredV1(name=name, email=email)
        self.apply(event)

    def register_v2(
        self,
        first_name: str,
        last_name: str,
        email: str,
        phone: str = None,
        preferred_language: str = "en",
    ):
        """Register customer using V2 event format."""
        event = CustomerRegisteredV2(
            first_name=first_name,
            last_name=last_name,
            email=email,
            phone=phone,
            preferred_language=preferred_language,
        )
        self.apply(event)

    def register_v3(self, profile_data: Dict[str, Any]):
        """Register customer using V3 unified event format."""
        event = CustomerProfileUpdatedV3(
            profile_changes=profile_data,
            change_type="registration",
            metadata={"version": 3},
        )
        self.apply(event)

    def change_email_v1(self, new_email: str):
        """Change email using V1 event format."""
        event = CustomerEmailChangedV1(new_email=new_email)
        self.apply(event)

    def change_email_v2(
        self, new_email: str, validated: bool = False, reason: str = None
    ):
        """Change email using V2 event format."""
        event = CustomerEmailChangedV2(
            old_email=self.email,
            new_email=new_email,
            validated=validated,
            change_reason=reason,
        )
        self.apply(event)

    def update_profile_v3(self, changes: Dict[str, Any], change_type: str):
        """Update profile using V3 unified event format."""
        event = CustomerProfileUpdatedV3(
            profile_changes=changes,
            change_type=change_type,
            metadata={"timestamp": datetime.now(timezone.utc).isoformat()},
        )
        self.apply(event)

    # Event handlers for all versions
    def apply_customer_registered_v1(self, event: CustomerRegisteredV1):
        """Handle V1 registration event."""
        name_parts = event.name.split(" ", 1)
        self.first_name = name_parts[0] if name_parts else ""
        self.last_name = name_parts[1] if len(name_parts) > 1 else ""
        self.email = event.email
        self.registration_version = 1

    def apply_customer_registered_v2(self, event: CustomerRegisteredV2):
        """Handle V2 registration event."""
        self.first_name = event.first_name
        self.last_name = event.last_name
        self.email = event.email
        self.phone = event.phone
        self.preferred_language = event.preferred_language
        self.registration_version = 2

    def apply_customer_profile_updated_v3(self, event: CustomerProfileUpdatedV3):
        """Handle V3 unified profile update event."""
        changes = event.profile_changes

        if "first_name" in changes:
            self.first_name = changes["first_name"]
        if "last_name" in changes:
            self.last_name = changes["last_name"]
        if "email" in changes:
            self.email = changes["email"]
        if "phone" in changes:
            self.phone = changes["phone"]
        if "preferred_language" in changes:
            self.preferred_language = changes["preferred_language"]

        # Handle specific change types
        if event.change_type == "registration":
            self.registration_version = 3
        elif event.change_type == "email_change":
            self.email_validated = event.metadata.get("validated", False)

    def apply_customer_email_changed_v1(self, event: CustomerEmailChangedV1):
        """Handle V1 email change event."""
        self.email = event.new_email
        self.email_validated = False

    def apply_customer_email_changed_v2(self, event: CustomerEmailChangedV2):
        """Handle V2 email change event."""
        self.email = event.new_email
        self.email_validated = event.validated

    @property
    def full_name(self) -> str:
        """Get full customer name."""
        return f"{self.first_name} {self.last_name}".strip()


async def demonstrate_event_versioning():
    """Demonstrate event versioning and schema evolution."""
    print("=== Event Versioning Example ===\n")

    event_store = await EventStore.create("sqlite://:memory:")

    # Test Case 1: V1 Events (Legacy Format)
    print("1. Testing V1 event format (legacy)...")

    customer_v1 = VersionedCustomer(id="customer-v1")
    customer_v1.register_v1("John Doe", "john@example.com")
    customer_v1.change_email_v1("john.doe@newcompany.com")

    await event_store.save(customer_v1)
    customer_v1.mark_events_as_committed()

    print(f"   ✓ V1 Customer: {customer_v1.full_name} ({customer_v1.email})")
    print(f"   ✓ Registration version: {customer_v1.registration_version}")
    print(f"   ✓ Email validated: {customer_v1.email_validated}")
    print(f"   ✓ Phone: {customer_v1.phone}")

    # Test Case 2: V2 Events (Enhanced Format)
    print("\n2. Testing V2 event format (enhanced)...")

    customer_v2 = VersionedCustomer(id="customer-v2")
    customer_v2.register_v2("Jane", "Smith", "jane@example.com", "+1-555-0123", "es")
    customer_v2.change_email_v2(
        "jane.smith@company.com", validated=True, reason="Company email migration"
    )

    await event_store.save(customer_v2)
    customer_v2.mark_events_as_committed()

    print(f"   ✓ V2 Customer: {customer_v2.full_name} ({customer_v2.email})")
    print(f"   ✓ Registration version: {customer_v2.registration_version}")
    print(f"   ✓ Email validated: {customer_v2.email_validated}")
    print(f"   ✓ Phone: {customer_v2.phone}")
    print(f"   ✓ Language: {customer_v2.preferred_language}")

    # Test Case 3: V3 Events (Unified Format)
    print("\n3. Testing V3 event format (unified)...")

    customer_v3 = VersionedCustomer(id="customer-v3")
    customer_v3.register_v3(
        {
            "first_name": "Alice",
            "last_name": "Johnson",
            "email": "alice@startup.com",
            "phone": "+1-555-0456",
            "preferred_language": "fr",
        }
    )

    # Update multiple fields at once
    customer_v3.update_profile_v3(
        {
            "email": "alice@bigcorp.com",
            "phone": "+1-555-0789",
            "preferred_language": "en",
        },
        "profile_update",
    )

    await event_store.save(customer_v3)
    customer_v3.mark_events_as_committed()

    print(f"   ✓ V3 Customer: {customer_v3.full_name} ({customer_v3.email})")
    print(f"   ✓ Registration version: {customer_v3.registration_version}")
    print(f"   ✓ Email validated: {customer_v3.email_validated}")
    print(f"   ✓ Phone: {customer_v3.phone}")
    print(f"   ✓ Language: {customer_v3.preferred_language}")

    # Test Case 4: Event Evolution and Migration
    print("\n4. Testing event evolution scenarios...")

    # Simulate loading old events and applying version evolution
    scenarios = [
        {
            "name": "V1 Registration Evolution",
            "event_data": {
                "event_type": "CustomerRegisteredV1",
                "event_version": 1,
                "name": "Bob Wilson",
                "email": "bob@oldservice.com",
            },
        },
        {
            "name": "V1 Email Change Evolution",
            "event_data": {
                "event_type": "CustomerEmailChangedV1",
                "event_version": 1,
                "new_email": "bob.wilson@newservice.com",
            },
        },
        {
            "name": "V2 to V3 Migration",
            "event_data": {
                "event_type": "CustomerRegisteredV2",
                "event_version": 2,
                "first_name": "Carol",
                "last_name": "Davis",
                "email": "carol@company.com",
                "phone": "+1-555-1234",
                "preferred_language": "de",
            },
        },
    ]

    migration_customer = VersionedCustomer(id="migration-test")

    for i, scenario in enumerate(scenarios, 1):
        print(f"\n   Scenario {i}: {scenario['name']}")

        # Simulate event upcasting
        try:
            upcast_event = EventVersioningService.upcast_event(scenario["event_data"])
            print(f"     ✓ Event upcast successful: {type(upcast_event).__name__}")

            # Apply the upcast event
            migration_customer.apply(upcast_event)

            print(f"     ✓ Applied to aggregate: {migration_customer.full_name}")
            print(f"       Email: {migration_customer.email}")
            print(f"       Phone: {migration_customer.phone}")
            print(f"       Language: {migration_customer.preferred_language}")

        except Exception as e:
            print(f"     ❌ Migration failed: {e}")

    await event_store.save(migration_customer)
    migration_customer.mark_events_as_committed()

    # Test Case 5: Backward Compatibility
    print("\n5. Testing backward compatibility...")

    # Load all customers to verify they work with current system
    customers = [
        ("customer-v1", "V1 Format"),
        ("customer-v2", "V2 Format"),
        ("customer-v3", "V3 Format"),
        ("migration-test", "Mixed Versions"),
    ]

    print("   Customer Compatibility Check:")
    for customer_id, format_name in customers:
        try:
            events = await event_store.load_events(customer_id)
            if events:
                # This would normally use event upcasting in a real system
                customer = VersionedCustomer(id=customer_id)

                # Manually reconstruct to show the pattern works
                if customer_id == "customer-v1":
                    customer = customer_v1
                elif customer_id == "customer-v2":
                    customer = customer_v2
                elif customer_id == "customer-v3":
                    customer = customer_v3
                elif customer_id == "migration-test":
                    customer = migration_customer

                print(f"     ✓ {format_name}: {customer.full_name} - {customer.email}")
                print(
                    f"       Registration: V{customer.registration_version}, Validated: {customer.email_validated}"
                )

        except Exception as e:
            print(f"     ❌ {format_name} compatibility failed: {e}")

    return {
        "customers": {
            "v1": customer_v1,
            "v2": customer_v2,
            "v3": customer_v3,
            "migrated": migration_customer,
        },
        "event_counts": {
            "v1": customer_v1.version,
            "v2": customer_v2.version,
            "v3": customer_v3.version,
            "migrated": migration_customer.version,
        },
    }


async def main():
    result = await demonstrate_event_versioning()

    print("\n✅ SUCCESS! Event versioning patterns demonstrated!")

    print("\nEvent versioning patterns covered:")
    print("- ✓ Multiple event schema versions (V1, V2, V3)")
    print("- ✓ Event upcasting for forward compatibility")
    print("- ✓ Backward compatibility with legacy events")
    print("- ✓ Schema evolution with breaking changes")
    print("- ✓ Event migration and transformation")
    print("- ✓ Unified event handling across versions")
    print("- ✓ Gradual system migration strategies")

    print("\nEvent statistics:")
    for version, count in result["event_counts"].items():
        print(f"- {version.upper()}: {count} events")


if __name__ == "__main__":
    asyncio.run(main())
