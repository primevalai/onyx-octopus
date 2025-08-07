#!/usr/bin/env python3
"""
Basic Event Store Example

This example demonstrates the fundamentals of event sourcing with Eventuali:
- Creating aggregates and events
- Saving and loading events from the event store
- Rebuilding aggregate state from events
- Basic event versioning
"""

import asyncio
import sys
import os

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import Aggregate
from eventuali.event import Event

class AccountCreated(Event):
    """Event fired when a new account is created."""
    owner_name: str
    initial_balance: float
    
class DepositMade(Event):
    """Event fired when money is deposited to an account."""
    amount: float
    description: str = ""
    
class WithdrawalMade(Event):
    """Event fired when money is withdrawn from an account."""
    amount: float
    description: str = ""
    
class AccountFrozen(Event):
    """Event fired when an account is frozen."""
    reason: str

class BankAccount(Aggregate):
    """Simple bank account aggregate demonstrating basic event sourcing."""
    
    owner_name: str = ""
    balance: float = 0.0
    is_frozen: bool = False
    transaction_count: int = 0
    
    def __init__(self, **data):
        super().__init__(**data)
    
    # Business methods (commands)
    def create_account(self, owner_name: str, initial_balance: float = 0.0):
        """Create a new bank account."""
        if initial_balance < 0:
            raise ValueError("Initial balance cannot be negative")
        
        event = AccountCreated(owner_name=owner_name, initial_balance=initial_balance)
        self.apply(event)
    
    def deposit(self, amount: float, description: str = "Deposit"):
        """Deposit money to the account."""
        if amount <= 0:
            raise ValueError("Deposit amount must be positive")
        if self.is_frozen:
            raise ValueError("Cannot deposit to frozen account")
        
        event = DepositMade(amount=amount, description=description)
        self.apply(event)
    
    def withdraw(self, amount: float, description: str = "Withdrawal"):
        """Withdraw money from the account."""
        if amount <= 0:
            raise ValueError("Withdrawal amount must be positive")
        if self.is_frozen:
            raise ValueError("Cannot withdraw from frozen account")
        if amount > self.balance:
            raise ValueError("Insufficient funds")
        
        event = WithdrawalMade(amount=amount, description=description)
        self.apply(event)
    
    def freeze_account(self, reason: str):
        """Freeze the account."""
        if self.is_frozen:
            return  # Already frozen
        
        event = AccountFrozen(reason=reason)
        self.apply(event)
    
    # Event handlers (projections)
    def apply_account_created(self, event: AccountCreated):
        """Apply AccountCreated event."""
        self.owner_name = event.owner_name
        self.balance = event.initial_balance
        
    def apply_deposit_made(self, event: DepositMade):
        """Apply DepositMade event."""
        self.balance += event.amount
        self.transaction_count += 1
        
    def apply_withdrawal_made(self, event: WithdrawalMade):
        """Apply WithdrawalMade event."""
        self.balance -= event.amount
        self.transaction_count += 1
        
    def apply_account_frozen(self, event: AccountFrozen):
        """Apply AccountFrozen event."""
        self.is_frozen = True

async def main():
    print("=== Basic Event Store Example ===\n")
    
    # 1. Create event store
    print("1. Creating event store...")
    event_store = await EventStore.create("sqlite://:memory:")
    print("   ✓ Event store created with in-memory SQLite database")
    
    # 2. Create a new bank account
    print("\n2. Creating a new bank account...")
    account = BankAccount(id="account-123")
    account.create_account("John Doe", 1000.0)
    
    print(f"   ✓ Account created for {account.owner_name} with ${account.balance:.2f}")
    print(f"   ✓ Aggregate version: {account.version}")
    print(f"   ✓ Uncommitted events: {len(account.get_uncommitted_events())}")
    
    # 3. Save the account to event store
    print("\n3. Saving account to event store...")
    await event_store.save(account)
    account.mark_events_as_committed()
    print("   ✓ Account saved successfully")
    print(f"   ✓ Uncommitted events after save: {len(account.get_uncommitted_events())}")
    
    # 4. Perform some transactions
    print("\n4. Performing transactions...")
    account.deposit(500.0, "Salary payment")
    account.deposit(200.0, "Freelance work")
    account.withdraw(150.0, "Groceries")
    account.withdraw(75.0, "Gas")
    
    print(f"   ✓ Made 4 transactions")
    print(f"   ✓ Current balance: ${account.balance:.2f}")
    print(f"   ✓ Transaction count: {account.transaction_count}")
    print(f"   ✓ Aggregate version: {account.version}")
    print(f"   ✓ Uncommitted events: {len(account.get_uncommitted_events())}")
    
    # 5. Save transactions
    print("\n5. Saving transactions...")
    await event_store.save(account)
    account.mark_events_as_committed()
    print("   ✓ All transactions saved")
    
    # 6. Load account from event store (event sourcing in action!)
    print("\n6. Loading account from event store...")
    loaded_events = await event_store.load_events(account.id)
    print(f"   ✓ Loaded {len(loaded_events)} events from store")
    
    # 7. Rebuild aggregate from events
    print("\n7. Rebuilding aggregate state from events...")
    restored_account = BankAccount.from_events(loaded_events)
    
    print(f"   ✓ Account restored: {restored_account.owner_name}")
    print(f"   ✓ Balance: ${restored_account.balance:.2f}")
    print(f"   ✓ Transaction count: {restored_account.transaction_count}")
    print(f"   ✓ Version: {restored_account.version}")
    print(f"   ✓ Frozen status: {restored_account.is_frozen}")
    
    # 8. Verify state consistency
    print("\n8. Verifying state consistency...")
    assert account.balance == restored_account.balance, "Balance mismatch!"
    assert account.transaction_count == restored_account.transaction_count, "Transaction count mismatch!"
    assert account.version == restored_account.version, "Version mismatch!"
    assert account.owner_name == restored_account.owner_name, "Owner name mismatch!"
    print("   ✓ All state matches perfectly!")
    
    # 9. Test account freezing
    print("\n9. Testing account freezing...")
    account.freeze_account("Suspicious activity detected")
    await event_store.save(account)
    
    try:
        account.withdraw(50.0, "Attempted withdrawal")
        print("   ❌ Should not be able to withdraw from frozen account")
    except ValueError as e:
        print(f"   ✓ Correctly blocked transaction: {e}")
    
    # 10. Load and verify final state
    print("\n10. Final verification...")
    final_events = await event_store.load_events(account.id)
    final_account = BankAccount.from_events(final_events)
    
    print(f"   ✓ Final state loaded from {len(final_events)} events")
    print(f"   ✓ Owner: {final_account.owner_name}")
    print(f"   ✓ Balance: ${final_account.balance:.2f}")
    print(f"   ✓ Transactions: {final_account.transaction_count}")
    print(f"   ✓ Frozen: {final_account.is_frozen}")
    print(f"   ✓ Version: {final_account.version}")
    
    print(f"\n✅ SUCCESS! Event store fundamentals demonstrated!")
    
    print(f"\nKey concepts demonstrated:")
    print(f"- ✓ Aggregate creation and business logic")
    print(f"- ✓ Event generation and application")
    print(f"- ✓ Event persistence and loading")
    print(f"- ✓ State reconstruction from events")
    print(f"- ✓ Business rule enforcement")
    print(f"- ✓ Event versioning and consistency")

if __name__ == "__main__":
    asyncio.run(main())