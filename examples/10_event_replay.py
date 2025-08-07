#!/usr/bin/env python3
"""
Event Replay Example

This example demonstrates event replay and time travel patterns:
- Historical state reconstruction at any point in time
- Event replay from specific positions
- Time-based queries and point-in-time snapshots
- Debugging and audit capabilities through replay
- Performance optimizations for large-scale replay operations
"""

import asyncio
import sys
import os
from typing import ClassVar, Optional, Dict, List, Any, Union
from datetime import datetime, timezone, timedelta
from dataclasses import dataclass
import json
import time

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import User
from eventuali.event import UserRegistered, UserEmailChanged, UserDeactivated, Event

# Banking Domain Events for Replay Example
class BankAccountOpened(Event):
    """Bank account opened event."""
    account_holder: str
    initial_deposit: float
    account_type: str

class DepositMade(Event):
    """Deposit made event."""
    amount: float
    source: str
    description: str

class WithdrawalMade(Event):
    """Withdrawal made event."""
    amount: float
    destination: str
    description: str

class InterestCredited(Event):
    """Interest credited event."""
    amount: float
    interest_rate: float
    period: str

class AccountFrozen(Event):
    """Account frozen event."""
    reason: str
    frozen_by: str

class AccountUnfrozen(Event):
    """Account unfrozen event."""
    unfrozen_by: str
    notes: str

# Banking Aggregate for Replay
class BankAccount:
    """Bank account aggregate with comprehensive event tracking."""
    
    CHECKING = "checking"
    SAVINGS = "savings"
    FROZEN = "frozen"
    ACTIVE = "active"
    
    def __init__(self, account_id: str = None):
        self.id = account_id or f"acc-{int(time.time())}"
        self.account_holder: str = ""
        self.account_type: str = self.CHECKING
        self.balance: float = 0.0
        self.status: str = self.ACTIVE
        self.transaction_count: int = 0
        self.total_deposits: float = 0.0
        self.total_withdrawals: float = 0.0
        self.total_interest: float = 0.0
        self.opened_date: Optional[str] = None
        self.last_transaction_date: Optional[str] = None
        
        # Event tracking
        self.events: List[Event] = []
        self.version = 0
        self.creation_timestamp = datetime.now(timezone.utc)
    
    def open_account(self, holder: str, initial_deposit: float, account_type: str = CHECKING):
        """Open a new account."""
        if self.account_holder:
            raise ValueError("Account already opened")
        
        self.account_holder = holder
        self.account_type = account_type
        self.balance = initial_deposit
        self.total_deposits = initial_deposit
        self.opened_date = datetime.now(timezone.utc).isoformat()
        self.last_transaction_date = self.opened_date
        
        event = BankAccountOpened(
            account_holder=holder,
            initial_deposit=initial_deposit,
            account_type=account_type
        )
        self._apply_event(event)
        return event
    
    def deposit(self, amount: float, source: str, description: str = ""):
        """Make a deposit."""
        if self.status == self.FROZEN:
            raise ValueError("Cannot deposit to frozen account")
        if amount <= 0:
            raise ValueError("Deposit amount must be positive")
        
        self.balance += amount
        self.total_deposits += amount
        self.transaction_count += 1
        self.last_transaction_date = datetime.now(timezone.utc).isoformat()
        
        event = DepositMade(amount=amount, source=source, description=description)
        self._apply_event(event)
        return event
    
    def withdraw(self, amount: float, destination: str, description: str = ""):
        """Make a withdrawal."""
        if self.status == self.FROZEN:
            raise ValueError("Cannot withdraw from frozen account")
        if amount <= 0:
            raise ValueError("Withdrawal amount must be positive")
        if amount > self.balance:
            raise ValueError("Insufficient funds")
        
        self.balance -= amount
        self.total_withdrawals += amount
        self.transaction_count += 1
        self.last_transaction_date = datetime.now(timezone.utc).isoformat()
        
        event = WithdrawalMade(amount=amount, destination=destination, description=description)
        self._apply_event(event)
        return event
    
    def credit_interest(self, interest_rate: float, period: str):
        """Credit interest to account."""
        if self.status == self.FROZEN:
            return  # No interest on frozen accounts
        
        interest_amount = self.balance * (interest_rate / 100)
        if interest_amount <= 0:
            return
        
        self.balance += interest_amount
        self.total_interest += interest_amount
        self.transaction_count += 1
        self.last_transaction_date = datetime.now(timezone.utc).isoformat()
        
        event = InterestCredited(
            amount=interest_amount,
            interest_rate=interest_rate,
            period=period
        )
        self._apply_event(event)
        return event
    
    def freeze_account(self, reason: str, frozen_by: str):
        """Freeze the account."""
        if self.status == self.FROZEN:
            return
        
        self.status = self.FROZEN
        event = AccountFrozen(reason=reason, frozen_by=frozen_by)
        self._apply_event(event)
        return event
    
    def unfreeze_account(self, unfrozen_by: str, notes: str = ""):
        """Unfreeze the account."""
        if self.status != self.FROZEN:
            return
        
        self.status = self.ACTIVE
        event = AccountUnfrozen(unfrozen_by=unfrozen_by, notes=notes)
        self._apply_event(event)
        return event
    
    def _apply_event(self, event: Event):
        """Apply event to aggregate."""
        event.aggregate_id = self.id
        event.aggregate_version = self.version + 1
        event.event_type = event.__class__.__name__
        
        self.events.append(event)
        self.version += 1
    
    def get_state_summary(self) -> Dict[str, Any]:
        """Get current state summary."""
        return {
            "account_id": self.id,
            "account_holder": self.account_holder,
            "account_type": self.account_type,
            "balance": round(self.balance, 2),
            "status": self.status,
            "transaction_count": self.transaction_count,
            "total_deposits": round(self.total_deposits, 2),
            "total_withdrawals": round(self.total_withdrawals, 2),
            "total_interest": round(self.total_interest, 2),
            "opened_date": self.opened_date,
            "last_transaction_date": self.last_transaction_date,
            "version": self.version,
            "events_count": len(self.events)
        }

@dataclass
class ReplayCheckpoint:
    """Checkpoint for replay operations."""
    position: int
    timestamp: str
    aggregate_state: Dict[str, Any]
    events_processed: int

class EventReplayEngine:
    """Engine for replaying events and time travel queries."""
    
    def __init__(self, event_store: EventStore):
        self.event_store = event_store
        self.checkpoints: List[ReplayCheckpoint] = []
    
    def reconstruct_aggregate_at_version(self, aggregate_events: List[Event], target_version: int) -> BankAccount:
        """Reconstruct aggregate state at specific version."""
        account = BankAccount()
        
        events_to_apply = [e for e in aggregate_events if hasattr(e, 'aggregate_version') and e.aggregate_version <= target_version]
        events_to_apply.sort(key=lambda x: getattr(x, 'aggregate_version', 0))
        
        for event in events_to_apply:
            if isinstance(event, BankAccountOpened):
                account.account_holder = event.account_holder
                account.account_type = event.account_type
                account.balance = event.initial_deposit
                account.total_deposits = event.initial_deposit
                account.opened_date = getattr(event, 'timestamp', datetime.now(timezone.utc)).isoformat() if hasattr(event, 'timestamp') else datetime.now(timezone.utc).isoformat()
                
            elif isinstance(event, DepositMade):
                account.balance += event.amount
                account.total_deposits += event.amount
                account.transaction_count += 1
                
            elif isinstance(event, WithdrawalMade):
                account.balance -= event.amount
                account.total_withdrawals += event.amount
                account.transaction_count += 1
                
            elif isinstance(event, InterestCredited):
                account.balance += event.amount
                account.total_interest += event.amount
                account.transaction_count += 1
                
            elif isinstance(event, AccountFrozen):
                account.status = BankAccount.FROZEN
                
            elif isinstance(event, AccountUnfrozen):
                account.status = BankAccount.ACTIVE
            
            account.version = getattr(event, 'aggregate_version', account.version + 1)
        
        return account
    
    def replay_events_from_position(self, all_events: List[Event], start_position: int) -> List[Dict[str, Any]]:
        """Replay events starting from specific position."""
        replay_results = []
        
        events_from_position = all_events[start_position:]
        
        for i, event in enumerate(events_from_position):
            result = {
                "position": start_position + i,
                "event_type": type(event).__name__,
                "event_data": self._extract_event_data(event),
                "aggregate_id": getattr(event, 'aggregate_id', 'unknown'),
                "aggregate_version": getattr(event, 'aggregate_version', 0)
            }
            replay_results.append(result)
        
        return replay_results
    
    def create_checkpoint(self, position: int, account: BankAccount) -> ReplayCheckpoint:
        """Create a checkpoint for replay optimization."""
        checkpoint = ReplayCheckpoint(
            position=position,
            timestamp=datetime.now(timezone.utc).isoformat(),
            aggregate_state=account.get_state_summary(),
            events_processed=position + 1
        )
        
        self.checkpoints.append(checkpoint)
        return checkpoint
    
    def find_nearest_checkpoint(self, target_position: int) -> Optional[ReplayCheckpoint]:
        """Find nearest checkpoint before target position."""
        suitable_checkpoints = [cp for cp in self.checkpoints if cp.position <= target_position]
        return max(suitable_checkpoints, key=lambda x: x.position) if suitable_checkpoints else None
    
    def replay_with_checkpoints(self, all_events: List[Event], target_position: int) -> Dict[str, Any]:
        """Replay events using checkpoints for optimization."""
        nearest_checkpoint = self.find_nearest_checkpoint(target_position)
        
        if nearest_checkpoint:
            start_position = nearest_checkpoint.position + 1
            base_state = nearest_checkpoint.aggregate_state
            print(f"   📍 Using checkpoint at position {nearest_checkpoint.position}")
        else:
            start_position = 0
            base_state = None
            print(f"   📍 No checkpoint found, replaying from beginning")
        
        events_to_replay = all_events[start_position:target_position + 1]
        
        replay_data = {
            "start_position": start_position,
            "target_position": target_position,
            "events_replayed": len(events_to_replay),
            "checkpoint_used": nearest_checkpoint is not None,
            "base_state": base_state,
            "events": []
        }
        
        for i, event in enumerate(events_to_replay):
            event_data = {
                "position": start_position + i,
                "event_type": type(event).__name__,
                "data": self._extract_event_data(event),
                "version": getattr(event, 'aggregate_version', 0)
            }
            replay_data["events"].append(event_data)
        
        return replay_data
    
    def analyze_event_history(self, all_events: List[Event]) -> Dict[str, Any]:
        """Analyze event history for insights."""
        analysis = {
            "total_events": len(all_events),
            "event_types": {},
            "timeline": [],
            "aggregates": set(),
            "version_range": {"min": float('inf'), "max": 0}
        }
        
        for event in all_events:
            event_type = type(event).__name__
            analysis["event_types"][event_type] = analysis["event_types"].get(event_type, 0) + 1
            
            if hasattr(event, 'aggregate_id'):
                analysis["aggregates"].add(event.aggregate_id)
            
            if hasattr(event, 'aggregate_version'):
                version = event.aggregate_version
                analysis["version_range"]["min"] = min(analysis["version_range"]["min"], version)
                analysis["version_range"]["max"] = max(analysis["version_range"]["max"], version)
        
        analysis["unique_aggregates"] = len(analysis["aggregates"])
        if analysis["version_range"]["min"] == float('inf'):
            analysis["version_range"]["min"] = 0
        
        return analysis
    
    def _extract_event_data(self, event: Event) -> Dict[str, Any]:
        """Extract event data for replay analysis."""
        data = {}
        
        # Use Pydantic model fields to avoid deprecated attribute access
        if hasattr(event.__class__, 'model_fields'):
            # Get field names from the model class, not instance
            field_names = event.__class__.model_fields.keys()
            for field_name in field_names:
                try:
                    value = getattr(event, field_name)
                    if isinstance(value, (str, int, float, bool, type(None))):
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
                        if isinstance(value, (str, int, float, bool, type(None))):
                            data[attr_name] = value
                    except:
                        continue
        
        return data

async def demonstrate_event_replay():
    """Demonstrate event replay and time travel capabilities."""
    print("=== Event Replay Example ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    replay_engine = EventReplayEngine(event_store)
    
    print("1. Creating bank account with transaction history...")
    
    # Create account and generate rich transaction history
    account = BankAccount("replay-demo-account")
    account.open_account("Alice Johnson", 1000.0, BankAccount.CHECKING)
    
    # Simulate 6 months of banking activity
    transactions = [
        ("deposit", {"amount": 500.0, "source": "payroll", "description": "Salary deposit"}),
        ("withdraw", {"amount": 200.0, "destination": "rent", "description": "Monthly rent"}),
        ("deposit", {"amount": 100.0, "source": "freelance", "description": "Contract work"}),
        ("withdraw", {"amount": 50.0, "destination": "groceries", "description": "Food shopping"}),
        ("credit_interest", {"interest_rate": 2.5, "period": "monthly"}),
        ("deposit", {"amount": 750.0, "source": "payroll", "description": "Salary deposit"}),
        ("withdraw", {"amount": 300.0, "destination": "utilities", "description": "Electric & gas"}),
        ("freeze_account", {"reason": "Suspicious activity", "frozen_by": "fraud_detection"}),
        ("unfreeze_account", {"unfrozen_by": "customer_service", "notes": "False positive cleared"}),
        ("deposit", {"amount": 250.0, "source": "refund", "description": "Purchase refund"}),
        ("withdraw", {"amount": 100.0, "destination": "atm", "description": "Cash withdrawal"}),
        ("credit_interest", {"interest_rate": 2.5, "period": "monthly"}),
        ("deposit", {"amount": 600.0, "source": "payroll", "description": "Salary deposit"}),
        ("withdraw", {"amount": 400.0, "destination": "insurance", "description": "Auto insurance"}),
        ("deposit", {"amount": 150.0, "source": "gift", "description": "Birthday gift"}),
    ]
    
    all_events = [account.events[0]]  # Start with account opening
    
    for i, (operation, params) in enumerate(transactions):
        try:
            if operation == "deposit":
                event = account.deposit(**params)
            elif operation == "withdraw":
                event = account.withdraw(**params)
            elif operation == "credit_interest":
                event = account.credit_interest(**params)
            elif operation == "freeze_account":
                event = account.freeze_account(**params)
            elif operation == "unfreeze_account":
                event = account.unfreeze_account(**params)
            
            if event:
                all_events.extend(account.events[-1:])  # Add new events
            
            # Create checkpoints at key intervals
            if (i + 1) % 5 == 0:
                replay_engine.create_checkpoint(len(all_events) - 1, account)
                
        except Exception as e:
            print(f"   ⚠️  Transaction {i+1} failed: {e}")
    
    final_state = account.get_state_summary()
    print(f"   ✓ Account created with {final_state['transaction_count']} transactions")
    print(f"   ✓ Final balance: ${final_state['balance']}")
    print(f"   ✓ Total events: {len(all_events)}")
    print(f"   ✓ Checkpoints created: {len(replay_engine.checkpoints)}")
    
    # Demonstrate Time Travel Queries
    print("\n2. Demonstrating time travel queries...")
    
    # Query state at different points in time
    time_points = [2, 5, 8, 12, len(all_events) - 1]
    
    for version in time_points:
        if version < len(all_events):
            historical_account = replay_engine.reconstruct_aggregate_at_version(all_events, version + 1)
            state = historical_account.get_state_summary()
            
            print(f"   📅 State at version {version + 1}:")
            print(f"      Balance: ${state['balance']}")
            print(f"      Transactions: {state['transaction_count']}")
            print(f"      Status: {state['status']}")
    
    # Demonstrate Event Replay from Position
    print("\n3. Replaying events from specific positions...")
    
    replay_positions = [0, 5, 10]
    
    for start_pos in replay_positions:
        if start_pos < len(all_events):
            replay_results = replay_engine.replay_events_from_position(all_events, start_pos)
            
            print(f"   🔄 Replay from position {start_pos}:")
            print(f"      Events replayed: {len(replay_results)}")
            
            for i, result in enumerate(replay_results[:3]):  # Show first 3
                print(f"      [{result['position']}] {result['event_type']}")
    
    # Demonstrate Checkpoint-Optimized Replay
    print("\n4. Demonstrating checkpoint-optimized replay...")
    
    checkpoint_targets = [7, 12, len(all_events) - 1]
    
    for target in checkpoint_targets:
        if target < len(all_events):
            print(f"\n   🎯 Replaying to position {target}:")
            
            start_time = time.perf_counter()
            replay_data = replay_engine.replay_with_checkpoints(all_events, target)
            replay_time = (time.perf_counter() - start_time) * 1000
            
            print(f"      Events replayed: {replay_data['events_replayed']}")
            print(f"      Checkpoint used: {replay_data['checkpoint_used']}")
            print(f"      Replay time: {replay_time:.2f}ms")
            
            if replay_data['events_replayed'] > 0:
                print(f"      Last event: {replay_data['events'][-1]['event_type']}")
    
    # Historical Analysis
    print("\n5. Analyzing event history...")
    
    history_analysis = replay_engine.analyze_event_history(all_events)
    
    print(f"   📊 Event History Analysis:")
    print(f"      Total events: {history_analysis['total_events']}")
    print(f"      Unique aggregates: {history_analysis['unique_aggregates']}")
    print(f"      Version range: {history_analysis['version_range']['min']} - {history_analysis['version_range']['max']}")
    
    print(f"      Event type distribution:")
    for event_type, count in sorted(history_analysis['event_types'].items()):
        percentage = (count / history_analysis['total_events']) * 100
        print(f"        - {event_type}: {count} ({percentage:.1f}%)")
    
    # Debug Scenario - Finding Specific Transactions
    print("\n6. Debugging scenario: Finding large withdrawals...")
    
    large_withdrawals = []
    for i, event in enumerate(all_events):
        if isinstance(event, WithdrawalMade) and event.amount > 250.0:
            # Reconstruct state just before this withdrawal
            state_before = replay_engine.reconstruct_aggregate_at_version(all_events, i)
            
            large_withdrawals.append({
                "position": i,
                "amount": event.amount,
                "destination": event.destination,
                "balance_before": round(state_before.balance, 2),
                "balance_after": round(state_before.balance - event.amount, 2)
            })
    
    print(f"   🔍 Found {len(large_withdrawals)} large withdrawals (>$250):")
    for withdrawal in large_withdrawals:
        print(f"      Position {withdrawal['position']}: ${withdrawal['amount']} to {withdrawal['destination']}")
        print(f"        Balance: ${withdrawal['balance_before']} → ${withdrawal['balance_after']}")
    
    # Audit Trail
    print("\n7. Generating audit trail...")
    
    audit_events = ["AccountFrozen", "AccountUnfrozen", "WithdrawalMade"]
    audit_trail = []
    
    for i, event in enumerate(all_events):
        if type(event).__name__ in audit_events:
            audit_trail.append({
                "position": i,
                "event_type": type(event).__name__,
                "data": replay_engine._extract_event_data(event),
                "aggregate_version": getattr(event, 'aggregate_version', 0)
            })
    
    print(f"   📋 Audit trail ({len(audit_trail)} events):")
    for audit_event in audit_trail:
        print(f"      [{audit_event['position']}] {audit_event['event_type']} (v{audit_event['aggregate_version']})")
        
        # Show relevant data
        data = audit_event['data']
        if 'amount' in data:
            print(f"        Amount: ${data['amount']}")
        if 'reason' in data:
            print(f"        Reason: {data['reason']}")
        if 'destination' in data:
            print(f"        Destination: {data['destination']}")
    
    return {
        "replay_engine": replay_engine,
        "final_state": final_state,
        "total_events": len(all_events),
        "checkpoints": len(replay_engine.checkpoints),
        "history_analysis": history_analysis,
        "large_withdrawals": large_withdrawals,
        "audit_trail": audit_trail
    }

async def main():
    result = await demonstrate_event_replay()
    
    print(f"\n✅ SUCCESS! Event replay patterns demonstrated!")
    
    print(f"\nEvent replay patterns covered:")
    print(f"- ✓ Historical state reconstruction at any version")
    print(f"- ✓ Event replay from specific positions")
    print(f"- ✓ Checkpoint-optimized replay for performance")
    print(f"- ✓ Time travel queries and point-in-time analysis")
    print(f"- ✓ Event history analysis and insights")
    print(f"- ✓ Debugging support through event replay")
    print(f"- ✓ Audit trail generation and compliance")
    
    print(f"\nReplay capabilities demonstrated:")
    print(f"- Total events processed: {result['total_events']}")
    print(f"- Checkpoints created: {result['checkpoints']}")
    print(f"- Large withdrawals found: {len(result['large_withdrawals'])}")
    print(f"- Audit events tracked: {len(result['audit_trail'])}")
    print(f"- Event types analyzed: {len(result['history_analysis']['event_types'])}")

if __name__ == "__main__":
    asyncio.run(main())