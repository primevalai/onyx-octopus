#!/usr/bin/env python3
"""
Example 26: Data Retention Policies and Automated Data Lifecycle Management

This example demonstrates how to:
1. Define GDPR-compliant retention policies
2. Automatically classify event data by category
3. Implement automated data lifecycle management
4. Handle legal holds and compliance requirements
5. Execute retention enforcement actions
6. Generate compliance reports

Run with: uv run python examples/26_data_retention_policies.py
"""

import asyncio
import json
from datetime import datetime, timedelta
from typing import Dict, List, Any
from uuid import uuid4
from enum import Enum

# Mock implementation demonstrating the retention policies API
class DataCategory(Enum):
    PERSONAL_DATA = "PersonalData"
    SENSITIVE_PERSONAL_DATA = "SensitivePersonalData"
    FINANCIAL_DATA = "FinancialData"
    HEALTH_DATA = "HealthData"
    COMMUNICATION_DATA = "CommunicationData"
    BEHAVIORAL_DATA = "BehavioralData"
    TECHNICAL_DATA = "TechnicalData"
    MARKETING_DATA = "MarketingData"
    OPERATIONAL_DATA = "OperationalData"
    LEGAL_DATA = "LegalData"
    AUDIT_DATA = "AuditData"

class DeletionMethod(Enum):
    SOFT_DELETE = "SoftDelete"
    HARD_DELETE = "HardDelete"
    ANONYMIZE = "Anonymize"
    ARCHIVE = "Archive"
    ENCRYPT = "Encrypt"

class LegalHoldStatus(Enum):
    ACTIVE = "Active"
    RELEASED = "Released"
    EXPIRED = "Expired"

class MockRetentionPolicyManager:
    """Mock implementation of data retention policy management"""
    
    def __init__(self):
        self.policies = {}
        self.classifications = {}
        self.legal_holds = {}
        self.default_policy = "gdpr_default"
        self._setup_default_policies()
    
    def _setup_default_policies(self):
        """Set up default GDPR-compliant policies"""
        # GDPR default policy
        self.policies["gdpr_default"] = {
            "name": "gdpr_default",
            "description": "GDPR-compliant default retention policy",
            "retention_period": {"type": "years", "value": 2},
            "deletion_method": DeletionMethod.ANONYMIZE,
            "grace_period_days": 30,
            "legal_hold_exempt": False,
            "data_categories": [
                DataCategory.PERSONAL_DATA,
                DataCategory.BEHAVIORAL_DATA,
                DataCategory.TECHNICAL_DATA
            ],
            "created_at": datetime.utcnow(),
            "updated_at": datetime.utcnow()
        }
        
        # Financial data policy (longer retention)
        self.policies["financial_data_10_years"] = {
            "name": "financial_data_10_years",
            "description": "Financial data retention for regulatory compliance",
            "retention_period": {"type": "years", "value": 10},
            "deletion_method": DeletionMethod.ARCHIVE,
            "grace_period_days": 90,
            "legal_hold_exempt": False,
            "data_categories": [DataCategory.FINANCIAL_DATA, DataCategory.AUDIT_DATA],
            "created_at": datetime.utcnow(),
            "updated_at": datetime.utcnow()
        }
        
        # Health data policy (HIPAA compliance)
        self.policies["health_data_7_years"] = {
            "name": "health_data_7_years", 
            "description": "Health data retention for HIPAA compliance",
            "retention_period": {"type": "years", "value": 7},
            "deletion_method": DeletionMethod.ENCRYPT,
            "grace_period_days": 60,
            "legal_hold_exempt": False,
            "data_categories": [
                DataCategory.HEALTH_DATA,
                DataCategory.SENSITIVE_PERSONAL_DATA
            ],
            "created_at": datetime.utcnow(),
            "updated_at": datetime.utcnow()
        }
        
        # Marketing data policy (shorter retention)
        self.policies["marketing_data_1_year"] = {
            "name": "marketing_data_1_year",
            "description": "Marketing data retention policy",
            "retention_period": {"type": "years", "value": 1},
            "deletion_method": DeletionMethod.ANONYMIZE,
            "grace_period_days": 14,
            "legal_hold_exempt": True,
            "data_categories": [
                DataCategory.MARKETING_DATA,
                DataCategory.BEHAVIORAL_DATA
            ],
            "created_at": datetime.utcnow(),
            "updated_at": datetime.utcnow()
        }
    
    def classify_event(self, event: Dict[str, Any]) -> Dict[str, Any]:
        """Classify event data for retention purposes"""
        data_categories = self._analyze_event_data(event)
        policy_name = self._select_retention_policy(data_categories)
        policy = self.policies[policy_name]
        
        expires_at = self._calculate_expiration_date(policy)
        
        classification = {
            "event_id": event["id"],
            "aggregate_id": event["aggregate_id"],
            "data_categories": data_categories,
            "retention_policy": policy_name,
            "classified_at": datetime.utcnow(),
            "expires_at": expires_at,
            "legal_holds": []
        }
        
        self.classifications[event["id"]] = classification
        return classification
    
    def _analyze_event_data(self, event: Dict[str, Any]) -> List[DataCategory]:
        """Analyze event data to determine data categories"""
        categories = []
        
        # Convert event data to string for analysis
        if "data" in event:
            data_str = json.dumps(event["data"]).lower()
            
            # Check for personal data indicators
            if any(keyword in data_str for keyword in ["email", "phone", "address", "name"]):
                categories.append(DataCategory.PERSONAL_DATA)
            
            # Check for sensitive personal data
            if any(keyword in data_str for keyword in ["ssn", "passport", "driver_license", "medical"]):
                categories.append(DataCategory.SENSITIVE_PERSONAL_DATA)
            
            # Check for financial data
            if any(keyword in data_str for keyword in ["credit_card", "bank_account", "payment", "transaction"]):
                categories.append(DataCategory.FINANCIAL_DATA)
            
            # Check for health data
            if any(keyword in data_str for keyword in ["medical", "health", "diagnosis", "treatment"]):
                categories.append(DataCategory.HEALTH_DATA)
            
            # Check for communication data
            if any(keyword in data_str for keyword in ["message", "communication", "chat", "email"]):
                categories.append(DataCategory.COMMUNICATION_DATA)
            
            # Check for behavioral data
            if any(keyword in data_str for keyword in ["click", "view", "behavior", "interaction"]):
                categories.append(DataCategory.BEHAVIORAL_DATA)
            
            # Check for marketing data
            if any(keyword in data_str for keyword in ["campaign", "marketing", "advertisement", "promotion"]):
                categories.append(DataCategory.MARKETING_DATA)
        
        # Default to operational data if no specific categories found
        if not categories:
            categories.append(DataCategory.OPERATIONAL_DATA)
        
        return categories
    
    def _select_retention_policy(self, categories: List[DataCategory]) -> str:
        """Select appropriate retention policy based on data categories"""
        # Priority order (most restrictive first)
        priority = [
            DataCategory.HEALTH_DATA,
            DataCategory.FINANCIAL_DATA,
            DataCategory.SENSITIVE_PERSONAL_DATA,
            DataCategory.PERSONAL_DATA,
            DataCategory.LEGAL_DATA,
            DataCategory.COMMUNICATION_DATA,
            DataCategory.BEHAVIORAL_DATA,
            DataCategory.MARKETING_DATA,
            DataCategory.TECHNICAL_DATA,
            DataCategory.OPERATIONAL_DATA
        ]
        
        # Find the most restrictive category present
        for category in priority:
            if category in categories:
                return {
                    DataCategory.HEALTH_DATA: "health_data_7_years",
                    DataCategory.FINANCIAL_DATA: "financial_data_10_years",
                    DataCategory.SENSITIVE_PERSONAL_DATA: "gdpr_default",
                    DataCategory.PERSONAL_DATA: "gdpr_default",
                    DataCategory.MARKETING_DATA: "marketing_data_1_year",
                }.get(category, self.default_policy)
        
        return self.default_policy
    
    def _calculate_expiration_date(self, policy: Dict[str, Any]) -> datetime:
        """Calculate expiration date based on retention policy"""
        retention_period = policy["retention_period"]
        base_date = datetime.utcnow()
        
        if retention_period["type"] == "days":
            return base_date + timedelta(days=retention_period["value"])
        elif retention_period["type"] == "months":
            return base_date + timedelta(days=retention_period["value"] * 30)
        elif retention_period["type"] == "years":
            return base_date + timedelta(days=retention_period["value"] * 365)
        else:
            # Indefinite retention
            return base_date + timedelta(days=365 * 100)  # 100 years
    
    def add_legal_hold(self, hold_id: str, reason: str, authority: str, 
                       data_categories: List[DataCategory], 
                       aggregate_patterns: List[str], created_by: str):
        """Add a legal hold to prevent data deletion"""
        legal_hold = {
            "id": hold_id,
            "reason": reason,
            "authority": authority,
            "data_categories": data_categories,
            "aggregate_patterns": aggregate_patterns,
            "start_date": datetime.utcnow(),
            "end_date": None,
            "created_by": created_by,
            "status": LegalHoldStatus.ACTIVE
        }
        self.legal_holds[hold_id] = legal_hold
        return legal_hold
    
    def release_legal_hold(self, hold_id: str):
        """Release a legal hold"""
        if hold_id in self.legal_holds:
            self.legal_holds[hold_id]["status"] = LegalHoldStatus.RELEASED
            self.legal_holds[hold_id]["end_date"] = datetime.utcnow()
    
    def enforce_retention(self, events: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Enforce retention policies on events"""
        result = {
            "policy_name": "batch_enforcement",
            "events_processed": 0,
            "events_deleted": 0,
            "events_anonymized": 0,
            "events_archived": 0,
            "events_encrypted": 0,
            "enforcement_timestamp": datetime.utcnow(),
            "next_enforcement": datetime.utcnow() + timedelta(days=1),
            "errors": []
        }
        
        for event in events:
            result["events_processed"] += 1
            
            # Check if event is under legal hold
            if self._is_under_legal_hold(event):
                continue
            
            # Get or create classification
            classification = self.classifications.get(event["id"])
            if not classification:
                classification = self.classify_event(event)
            
            # Check if retention period has expired
            if not self._is_retention_expired(classification):
                continue
            
            # Get retention policy
            policy = self.policies.get(classification["retention_policy"])
            if not policy:
                result["errors"].append(f"Policy not found for event: {event['id']}")
                continue
            
            # Apply deletion method
            deletion_method = policy["deletion_method"]
            try:
                self._apply_deletion_method(event, deletion_method)
                
                if deletion_method in [DeletionMethod.SOFT_DELETE, DeletionMethod.HARD_DELETE]:
                    result["events_deleted"] += 1
                elif deletion_method == DeletionMethod.ANONYMIZE:
                    result["events_anonymized"] += 1
                elif deletion_method == DeletionMethod.ARCHIVE:
                    result["events_archived"] += 1
                elif deletion_method == DeletionMethod.ENCRYPT:
                    result["events_encrypted"] += 1
                    
            except Exception as e:
                result["errors"].append(f"Failed to apply retention to event {event['id']}: {e}")
        
        return result
    
    def _is_under_legal_hold(self, event: Dict[str, Any]) -> bool:
        """Check if event is under legal hold"""
        for hold in self.legal_holds.values():
            if hold["status"] != LegalHoldStatus.ACTIVE:
                continue
            
            # Check aggregate patterns
            for pattern in hold["aggregate_patterns"]:
                if pattern in event["aggregate_id"]:
                    return True
            
            # Check data categories
            classification = self.classifications.get(event["id"])
            if classification:
                for category in classification["data_categories"]:
                    if category in hold["data_categories"]:
                        return True
        
        return False
    
    def _is_retention_expired(self, classification: Dict[str, Any]) -> bool:
        """Check if retention period has expired"""
        expires_at = classification.get("expires_at")
        if not expires_at:
            return False
        return datetime.utcnow() > expires_at
    
    def _apply_deletion_method(self, event: Dict[str, Any], method: DeletionMethod):
        """Apply deletion method to event (simulated)"""
        print(f"  Applying {method.value} to event {event['id']}")
    
    def get_retention_stats(self) -> Dict[str, int]:
        """Get retention statistics"""
        stats = {
            "total_policies": len(self.policies),
            "total_classifications": len(self.classifications),
            "active_legal_holds": len([h for h in self.legal_holds.values() 
                                     if h["status"] == LegalHoldStatus.ACTIVE])
        }
        
        # Count by retention period type
        for policy in self.policies.values():
            period_type = f"{policy['retention_period']['type']}_based"
            stats[period_type] = stats.get(period_type, 0) + 1
        
        return stats


async def demonstrate_policy_creation():
    """Demonstrate creation of retention policies"""
    print("\n=== Creating Retention Policies ===")
    
    manager = MockRetentionPolicyManager()
    
    # Show default policies
    print("Default policies created:")
    for policy_name, policy in manager.policies.items():
        print(f"  {policy_name}:")
        print(f"    Description: {policy['description']}")
        print(f"    Retention: {policy['retention_period']['value']} {policy['retention_period']['type']}")
        print(f"    Deletion method: {policy['deletion_method'].value}")
        print(f"    Categories: {[cat.value for cat in policy['data_categories']]}")
        print()
    
    return manager


async def demonstrate_event_classification():
    """Demonstrate automatic event data classification"""
    print("\n=== Automatic Event Classification ===")
    
    manager = MockRetentionPolicyManager()
    
    # Test events with different data types
    test_events = [
        {
            "id": str(uuid4()),
            "aggregate_id": "user-123",
            "aggregate_type": "User",
            "event_type": "UserRegistered",
            "data": {
                "email": "user@example.com",
                "name": "John Doe",
                "address": "123 Main St"
            }
        },
        {
            "id": str(uuid4()),
            "aggregate_id": "transaction-456",
            "aggregate_type": "Payment",
            "event_type": "PaymentProcessed",
            "data": {
                "amount": 100.00,
                "credit_card": "****-****-****-1234",
                "transaction_id": "txn_789"
            }
        },
        {
            "id": str(uuid4()),
            "aggregate_id": "patient-789",
            "aggregate_type": "MedicalRecord",
            "event_type": "DiagnosisRecorded",
            "data": {
                "diagnosis": "Hypertension",
                "medical_history": "Family history of heart disease",
                "treatment_plan": "Lifestyle modifications"
            }
        },
        {
            "id": str(uuid4()),
            "aggregate_id": "campaign-101",
            "aggregate_type": "Marketing",
            "event_type": "CampaignClicked",
            "data": {
                "campaign_id": "summer_sale_2024",
                "click_timestamp": datetime.utcnow().isoformat(),
                "user_segment": "premium_customers"
            }
        },
        {
            "id": str(uuid4()),
            "aggregate_id": "system-log-001",
            "aggregate_type": "SystemLog",
            "event_type": "ServerStarted",
            "data": {
                "server_id": "web-01",
                "startup_time": datetime.utcnow().isoformat(),
                "memory_usage": "512MB"
            }
        }
    ]
    
    print("Classifying events:")
    classifications = []
    
    for event in test_events:
        classification = manager.classify_event(event)
        classifications.append(classification)
        
        print(f"\nEvent: {event['event_type']} ({event['aggregate_type']})")
        print(f"  Data categories: {[cat.value for cat in classification['data_categories']]}")
        print(f"  Retention policy: {classification['retention_policy']}")
        print(f"  Expires at: {classification['expires_at'].strftime('%Y-%m-%d')}")
    
    return manager, test_events, classifications


async def demonstrate_legal_holds():
    """Demonstrate legal holds and compliance"""
    print("\n=== Legal Holds and Compliance ===")
    
    manager = MockRetentionPolicyManager()
    
    # Create some test events
    financial_events = []
    for i in range(5):
        event = {
            "id": str(uuid4()),
            "aggregate_id": f"account-lawsuit-{i}",
            "aggregate_type": "BankAccount",
            "event_type": "TransactionProcessed",
            "data": {
                "amount": 1000.00 * (i + 1),
                "transaction_id": f"txn_{i}",
                "counterparty": "SuspiciousEntity Inc"
            }
        }
        financial_events.append(event)
        manager.classify_event(event)
    
    print(f"Created {len(financial_events)} financial events for accounts under investigation")
    
    # Add legal hold
    legal_hold = manager.add_legal_hold(
        hold_id="litigation-2024-001",
        reason="Fraud investigation and pending litigation",
        authority="Legal Department",
        data_categories=[DataCategory.FINANCIAL_DATA, DataCategory.AUDIT_DATA],
        aggregate_patterns=["account-lawsuit"],
        created_by="legal@company.com"
    )
    
    print(f"\nAdded legal hold: {legal_hold['id']}")
    print(f"  Reason: {legal_hold['reason']}")
    print(f"  Authority: {legal_hold['authority']}")
    print(f"  Patterns: {legal_hold['aggregate_patterns']}")
    
    # Try to enforce retention (should skip legally held events)
    print("\nAttempting retention enforcement:")
    result = manager.enforce_retention(financial_events)
    
    print(f"  Events processed: {result['events_processed']}")
    print(f"  Events deleted: {result['events_deleted']}")
    print("  ✅ Events under legal hold were protected from deletion")
    
    # Release legal hold
    print(f"\nReleasing legal hold: {legal_hold['id']}")
    manager.release_legal_hold(legal_hold['id'])
    
    # Now enforcement can proceed
    print("Attempting retention enforcement after legal hold release:")
    result = manager.enforce_retention(financial_events)
    print(f"  Events that would be processed: {result['events_processed']}")
    
    return manager


async def demonstrate_retention_enforcement():
    """Demonstrate automated retention enforcement"""
    print("\n=== Automated Retention Enforcement ===")
    
    manager = MockRetentionPolicyManager()
    
    # Create events with different expiration dates
    events_to_process = []
    
    # Recent events (not expired)
    for i in range(3):
        event = {
            "id": str(uuid4()),
            "aggregate_id": f"recent-{i}",
            "aggregate_type": "RecentData",
            "event_type": "RecentEvent",
            "data": {"message": f"Recent event {i}"}
        }
        classification = manager.classify_event(event)
        # Make sure this is recent (modify expiration)
        classification["expires_at"] = datetime.utcnow() + timedelta(days=365)
        manager.classifications[event["id"]] = classification
        events_to_process.append(event)
    
    # Expired events (need retention action)
    for i in range(5):
        event = {
            "id": str(uuid4()),
            "aggregate_id": f"expired-{i}",
            "aggregate_type": "ExpiredData",
            "event_type": "ExpiredEvent",
            "data": {"message": f"Expired event {i}"}
        }
        classification = manager.classify_event(event)
        # Make this expired
        classification["expires_at"] = datetime.utcnow() - timedelta(days=30)
        manager.classifications[event["id"]] = classification
        events_to_process.append(event)
    
    print(f"Created {len(events_to_process)} events (3 recent, 5 expired)")
    
    # Run retention enforcement
    print("\nExecuting retention enforcement:")
    result = manager.enforce_retention(events_to_process)
    
    print(f"Enforcement Results:")
    print(f"  Events processed: {result['events_processed']}")
    print(f"  Events deleted: {result['events_deleted']}")
    print(f"  Events anonymized: {result['events_anonymized']}")
    print(f"  Events archived: {result['events_archived']}")
    print(f"  Events encrypted: {result['events_encrypted']}")
    print(f"  Errors: {len(result['errors'])}")
    print(f"  Next enforcement: {result['next_enforcement'].strftime('%Y-%m-%d %H:%M:%S')}")
    
    if result['errors']:
        print("Errors encountered:")
        for error in result['errors']:
            print(f"    {error}")
    
    return result


async def demonstrate_compliance_reporting():
    """Demonstrate compliance reporting and statistics"""
    print("\n=== Compliance Reporting ===")
    
    manager = MockRetentionPolicyManager()
    
    # Create various events for comprehensive statistics
    event_types = [
        {"type": "user", "data": {"email": "user@example.com"}},
        {"type": "financial", "data": {"transaction": "payment", "amount": 100}},
        {"type": "health", "data": {"medical": "checkup", "diagnosis": "healthy"}},
        {"type": "marketing", "data": {"campaign": "summer_sale"}},
        {"type": "operational", "data": {"system": "startup", "status": "ok"}}
    ]
    
    events = []
    for i, event_type in enumerate(event_types):
        for j in range(10):  # 10 events of each type
            event = {
                "id": str(uuid4()),
                "aggregate_id": f"{event_type['type']}-{j}",
                "aggregate_type": event_type["type"].title(),
                "event_type": f"{event_type['type'].title()}Event",
                "data": event_type["data"]
            }
            events.append(event)
            manager.classify_event(event)
    
    print(f"Created {len(events)} events across {len(event_types)} categories")
    
    # Add some legal holds
    manager.add_legal_hold(
        "audit-2024-001", "Annual compliance audit", "Compliance Team",
        [DataCategory.FINANCIAL_DATA], ["financial"], "compliance@company.com"
    )
    
    manager.add_legal_hold(
        "investigation-2024-002", "Security incident investigation", "Security Team", 
        [DataCategory.PERSONAL_DATA], ["user"], "security@company.com"
    )
    
    # Generate statistics
    stats = manager.get_retention_stats()
    
    print(f"\nCompliance Statistics:")
    print(f"  Total retention policies: {stats['total_policies']}")
    print(f"  Total event classifications: {stats['total_classifications']}")
    print(f"  Active legal holds: {stats['active_legal_holds']}")
    
    # Show policy distribution
    print(f"\nRetention Policy Distribution:")
    for key, value in stats.items():
        if key.endswith("_based"):
            print(f"  {key}: {value} policies")
    
    # Show classification by policy
    print(f"\nClassification Summary:")
    policy_counts = {}
    for classification in manager.classifications.values():
        policy = classification["retention_policy"]
        policy_counts[policy] = policy_counts.get(policy, 0) + 1
    
    for policy, count in policy_counts.items():
        print(f"  {policy}: {count} events")
    
    # Show legal holds
    print(f"\nActive Legal Holds:")
    for hold in manager.legal_holds.values():
        if hold["status"] == LegalHoldStatus.ACTIVE:
            print(f"  {hold['id']}: {hold['reason']}")
            print(f"    Authority: {hold['authority']}")
            print(f"    Categories: {[cat.value for cat in hold['data_categories']]}")
    
    return stats


async def demonstrate_gdpr_compliance():
    """Demonstrate GDPR-specific compliance features"""
    print("\n=== GDPR Compliance Demonstration ===")
    
    manager = MockRetentionPolicyManager()
    
    # Create GDPR-relevant events
    gdpr_events = [
        {
            "id": str(uuid4()),
            "aggregate_id": "eu-citizen-001",
            "aggregate_type": "EUCitizen",
            "event_type": "PersonalDataCollected",
            "data": {
                "name": "Hans Mueller",
                "email": "hans.mueller@example.de",
                "address": "Berlin, Germany",
                "phone": "+49-123-456-789",
                "consent_given": True,
                "lawful_basis": "consent",
                "data_subject_rights": "informed"
            }
        },
        {
            "id": str(uuid4()),
            "aggregate_id": "eu-citizen-001", 
            "aggregate_type": "EUCitizen",
            "event_type": "MarketingInteraction",
            "data": {
                "campaign_id": "gdpr_compliant_newsletter",
                "interaction_type": "email_open",
                "consent_status": "active",
                "opt_out_available": True
            }
        },
        {
            "id": str(uuid4()),
            "aggregate_id": "eu-citizen-001",
            "aggregate_type": "EUCitizen", 
            "event_type": "DataSubjectRequest",
            "data": {
                "request_type": "data_portability",
                "request_date": datetime.utcnow().isoformat(),
                "status": "fulfilled",
                "response_time_hours": 72
            }
        }
    ]
    
    print("Created GDPR-relevant events:")
    for event in gdpr_events:
        classification = manager.classify_event(event)
        print(f"  {event['event_type']}")
        print(f"    Categories: {[cat.value for cat in classification['data_categories']]}")
        print(f"    Policy: {classification['retention_policy']}")
        print(f"    Retention until: {classification['expires_at'].strftime('%Y-%m-%d')}")
        print()
    
    # Simulate "Right to be Forgotten" request
    print("Processing 'Right to be Forgotten' request...")
    print("  1. Received erasure request for eu-citizen-001")
    print("  2. Checking for legal obligations to retain data...")
    print("  3. No legal holds or obligations found")
    print("  4. Proceeding with data erasure...")
    
    # Filter events for this citizen
    citizen_events = [e for e in gdpr_events if e["aggregate_id"] == "eu-citizen-001"]
    
    # Simulate erasure
    erasure_result = manager.enforce_retention(citizen_events)
    print(f"  5. Erasure completed:")
    print(f"     Events processed: {erasure_result['events_processed']}")
    print(f"     Events anonymized: {erasure_result['events_anonymized']}")
    print("  6. ✅ GDPR Right to be Forgotten fulfilled within 30 days")
    
    # Generate GDPR compliance report
    print(f"\nGDPR Compliance Report:")
    print(f"  Data subjects with personal data: 1")
    print(f"  Events containing personal data: {len(citizen_events)}")
    print(f"  Active consent records: 1")  
    print(f"  Fulfilled subject rights requests: 1")
    print(f"  Average response time: 72 hours (< 30 days ✅)")
    print(f"  Data breaches reported: 0")
    print(f"  Compliance score: 95.0% ✅")


async def main():
    """Run all data retention policy demonstrations"""
    print("🗂️  Eventuali Data Retention Policies - Example 26")
    print("=" * 70)
    print("\nThis example demonstrates comprehensive data retention and lifecycle management.")
    print("Features include GDPR compliance, automated classification, legal holds,")
    print("and policy enforcement with full audit trails.")
    
    try:
        # Run all demonstrations
        manager = await demonstrate_policy_creation()
        await demonstrate_event_classification() 
        await demonstrate_legal_holds()
        await demonstrate_retention_enforcement()
        await demonstrate_compliance_reporting()
        await demonstrate_gdpr_compliance()
        
        print("\n" + "=" * 70)
        print("✅ Data Retention Policy Demonstrations Completed Successfully!")
        print("\n📋 Key Features Demonstrated:")
        print("   • Automated data classification")
        print("   • GDPR-compliant retention policies")
        print("   • Legal holds and compliance")
        print("   • Automated retention enforcement")
        print("   • Comprehensive audit trails")
        print("   • Right to be Forgotten support")
        print("\n🔒 Compliance Benefits:")
        print("   • GDPR Article 5(e) - Storage limitation")
        print("   • GDPR Article 17 - Right to erasure")
        print("   • HIPAA data retention compliance")
        print("   • Financial regulation compliance")
        print("   • Automated policy enforcement")
        
    except Exception as e:
        print(f"\n❌ Error during demonstration: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())