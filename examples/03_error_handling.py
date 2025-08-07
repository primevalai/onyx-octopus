#!/usr/bin/env python3
"""
Error Handling Example

This example demonstrates proper error handling patterns in event sourcing:
- Domain-specific exceptions and validation
- Optimistic concurrency conflict resolution
- Event store connection failures
- Aggregate invariant violations
- Recovery strategies and error logging
"""

import asyncio
import sys
import os
from typing import Optional, ClassVar

# Add the python package to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'eventuali-python', 'python'))

from eventuali import EventStore
from eventuali.aggregate import Aggregate
from eventuali.event import Event

# Custom exception classes
class DomainError(Exception):
    """Base class for domain-specific errors."""
    pass

class InsufficientFundsError(DomainError):
    """Raised when account has insufficient funds."""
    def __init__(self, balance: float, requested: float):
        self.balance = balance
        self.requested = requested
        super().__init__(f"Insufficient funds: balance ${balance:.2f}, requested ${requested:.2f}")

class AccountClosedError(DomainError):
    """Raised when trying to operate on a closed account."""
    pass

class InvalidTransactionError(DomainError):
    """Raised for invalid transaction parameters."""
    pass

# Events
class AccountOpened(Event):
    """Event fired when account is opened."""
    account_holder: str
    initial_deposit: float

class MoneyDeposited(Event):
    """Event fired when money is deposited."""
    amount: float
    reference: str

class MoneyWithdrawn(Event):
    """Event fired when money is withdrawn."""
    amount: float
    reference: str

class AccountClosed(Event):
    """Event fired when account is closed."""
    reason: str
    final_balance: float

class BankAccount(Aggregate):
    """Bank account with comprehensive error handling."""
    
    # Account states
    OPEN: ClassVar[str] = "open"
    CLOSED: ClassVar[str] = "closed"
    
    account_holder: str = ""
    balance: float = 0.0
    status: str = ""
    transaction_count: int = 0
    
    def __init__(self, **data):
        super().__init__(**data)
    
    # Business methods with error handling
    def open_account(self, account_holder: str, initial_deposit: float):
        """Open a new bank account."""
        # Validation
        if not account_holder or account_holder.strip() == "":
            raise InvalidTransactionError("Account holder name is required")
        
        if initial_deposit < 0:
            raise InvalidTransactionError("Initial deposit cannot be negative")
        
        if initial_deposit < 10.0:  # Minimum opening balance
            raise InvalidTransactionError("Minimum opening balance is $10.00")
        
        if self.status != "":  # Account already initialized
            raise DomainError("Account already exists")
        
        event = AccountOpened(
            account_holder=account_holder.strip(),
            initial_deposit=initial_deposit
        )
        self.apply(event)
    
    def deposit(self, amount: float, reference: str = ""):
        """Deposit money to account."""
        self._ensure_account_open()
        
        if amount <= 0:
            raise InvalidTransactionError("Deposit amount must be positive")
        
        if amount > 10000:  # Anti-money laundering limit
            raise InvalidTransactionError("Single deposit cannot exceed $10,000")
        
        event = MoneyDeposited(amount=amount, reference=reference)
        self.apply(event)
    
    def withdraw(self, amount: float, reference: str = ""):
        """Withdraw money from account."""
        self._ensure_account_open()
        
        if amount <= 0:
            raise InvalidTransactionError("Withdrawal amount must be positive")
        
        if amount > self.balance:
            raise InsufficientFundsError(self.balance, amount)
        
        # Daily withdrawal limit
        if amount > 1000:
            raise InvalidTransactionError("Daily withdrawal limit is $1,000")
        
        event = MoneyWithdrawn(amount=amount, reference=reference)
        self.apply(event)
    
    def close_account(self, reason: str = "Account closure requested"):
        """Close the account."""
        self._ensure_account_open()
        
        if self.balance > 0:
            raise DomainError(f"Cannot close account with positive balance of ${self.balance:.2f}")
        
        event = AccountClosed(reason=reason, final_balance=self.balance)
        self.apply(event)
    
    def _ensure_account_open(self):
        """Ensure account is open for operations."""
        if self.status == self.CLOSED:
            raise AccountClosedError("Account is closed")
        if self.status != self.OPEN:
            raise DomainError("Account not initialized")
    
    # Event handlers
    def apply_account_opened(self, event: AccountOpened):
        """Apply AccountOpened event."""
        self.account_holder = event.account_holder
        self.balance = event.initial_deposit
        self.status = self.OPEN
    
    def apply_money_deposited(self, event: MoneyDeposited):
        """Apply MoneyDeposited event."""
        self.balance += event.amount
        self.transaction_count += 1
    
    def apply_money_withdrawn(self, event: MoneyWithdrawn):
        """Apply MoneyWithdrawn event."""
        self.balance -= event.amount
        self.transaction_count += 1
    
    def apply_account_closed(self, event: AccountClosed):
        """Apply AccountClosed event."""
        self.status = self.CLOSED

async def demonstrate_validation_errors():
    """Demonstrate validation and business rule errors."""
    print("=== Validation Error Handling ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    
    # Test 1: Invalid account holder
    print("1. Testing invalid account holder...")
    account1 = BankAccount(id="error-test-1a")
    try:
        account1.open_account("", 100.0)
        print("   ❌ Should have failed")
    except InvalidTransactionError as e:
        print(f"   ✓ Correctly caught error: {e}")
    
    # Test 2: Insufficient opening balance
    print("\n2. Testing insufficient opening balance...")
    account2 = BankAccount(id="error-test-1b")
    try:
        account2.open_account("John Doe", 5.0)
        print("   ❌ Should have failed")
    except InvalidTransactionError as e:
        print(f"   ✓ Correctly caught error: {e}")
    
    # Test 3: Successful account opening
    print("\n3. Opening account correctly...")
    account = BankAccount(id="error-test-1c")
    account.open_account("John Doe", 100.0)
    print(f"   ✓ Account opened: {account.account_holder}, balance: ${account.balance:.2f}")
    
    # Test 4: Invalid deposit
    print("\n4. Testing invalid deposit...")
    try:
        account.deposit(-50.0)
        print("   ❌ Should have failed")
    except InvalidTransactionError as e:
        print(f"   ✓ Correctly caught error: {e}")
    
    # Test 5: Excessive deposit (AML)
    print("\n5. Testing excessive deposit...")
    try:
        account.deposit(15000.0)
        print("   ❌ Should have failed")
    except InvalidTransactionError as e:
        print(f"   ✓ Correctly caught error: {e}")
    
    # Test 6: Insufficient funds
    print("\n6. Testing insufficient funds...")
    try:
        account.withdraw(200.0)
        print("   ❌ Should have failed")
    except InsufficientFundsError as e:
        print(f"   ✓ Correctly caught error: {e}")
        print(f"     Balance: ${e.balance:.2f}, Requested: ${e.requested:.2f}")
    
    return account

async def demonstrate_business_rule_errors():
    """Demonstrate business rule violations."""
    print("\n=== Business Rule Error Handling ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    account = BankAccount(id="error-test-2")
    
    # Setup account
    account.open_account("Jane Smith", 500.0)
    print("1. Account opened with $500.00")
    
    # Test closed account operations
    print("\n2. Testing operations on closed account...")
    
    # Close account (withdraw funds first)
    account.withdraw(500.0, "Closing account")
    account.close_account("Customer request")
    
    print(f"   ✓ Account closed, status: {account.status}")
    
    # Try to deposit to closed account
    try:
        account.deposit(100.0)
        print("   ❌ Should have failed")
    except AccountClosedError as e:
        print(f"   ✓ Correctly blocked deposit: {e}")
    
    # Try to withdraw from closed account
    try:
        account.withdraw(50.0)
        print("   ❌ Should have failed")
    except AccountClosedError as e:
        print(f"   ✓ Correctly blocked withdrawal: {e}")
    
    return account

async def demonstrate_recovery_strategies():
    """Demonstrate error recovery strategies."""
    print("\n=== Recovery Strategy Examples ===\n")
    
    event_store = await EventStore.create("sqlite://:memory:")
    account = BankAccount(id="recovery-test")
    account.open_account("Recovery User", 1000.0)
    
    # Strategy 1: Retry with validation
    print("1. Retry strategy for user input errors...")
    withdrawal_amount = 1500.0  # Too much
    
    max_retries = 3
    for attempt in range(max_retries):
        try:
            account.withdraw(withdrawal_amount)
            print(f"   ✓ Withdrawal successful: ${withdrawal_amount:.2f}")
            break
        except InsufficientFundsError as e:
            print(f"   Attempt {attempt + 1}: {e}")
            # Adjust to available balance
            withdrawal_amount = e.balance * 0.8  # 80% of available
            if attempt == max_retries - 1:
                print("   ℹ️  Max retries reached, transaction cancelled")
    
    # Strategy 2: Graceful degradation
    print("\n2. Graceful degradation for service limits...")
    deposit_amount = 15000.0  # Exceeds AML limit
    
    try:
        account.deposit(deposit_amount)
    except InvalidTransactionError as e:
        print(f"   ⚠️  Large deposit blocked: {e}")
        # Split into smaller deposits
        chunk_size = 5000.0
        remaining = deposit_amount
        deposits_made = 0
        
        while remaining > 0 and deposits_made < 2:  # Limit splits
            chunk = min(chunk_size, remaining)
            try:
                account.deposit(chunk, f"Split deposit {deposits_made + 1}")
                print(f"   ✓ Deposited chunk: ${chunk:.2f}")
                remaining -= chunk
                deposits_made += 1
            except Exception as inner_e:
                print(f"   ❌ Chunk failed: {inner_e}")
                break
        
        if remaining > 0:
            print(f"   ℹ️  Remaining ${remaining:.2f} requires manual processing")

async def demonstrate_event_store_errors():
    """Demonstrate event store error handling."""
    print("\n=== Event Store Error Handling ===\n")
    
    # Test with invalid connection string
    print("1. Testing invalid database connection...")
    try:
        bad_store = await EventStore.create("invalid://bad-connection")
        print("   ❌ Should have failed")
    except Exception as e:
        print(f"   ✓ Correctly caught connection error: {type(e).__name__}")
    
    # Test with good connection
    print("\n2. Testing successful connection recovery...")
    good_store = await EventStore.create("sqlite://:memory:")
    print("   ✓ Successfully connected to database")
    
    # Test saving complex aggregate
    account = BankAccount(id="store-test")
    account.open_account("Store Test", 100.0)
    account.deposit(50.0)
    
    try:
        await good_store.save(account)
        account.mark_events_as_committed()
        print("   ✓ Successfully saved aggregate with 2 events")
        
        # Verify by loading
        loaded_events = await good_store.load_events(account.id)
        print(f"   ✓ Successfully loaded {len(loaded_events)} events")
        
    except Exception as e:
        print(f"   ❌ Unexpected error: {e}")

async def main():
    print("=== Error Handling Example ===\n")
    
    # Run all demonstrations
    account1 = await demonstrate_validation_errors()
    account2 = await demonstrate_business_rule_errors()
    await demonstrate_recovery_strategies()
    await demonstrate_event_store_errors()
    
    print(f"\n=== Summary ===")
    print(f"Test accounts created with various error scenarios")
    print(f"Account 1: {account1.account_holder} - ${account1.balance:.2f} ({account1.status})")
    print(f"Account 2: {account2.account_holder} - ${account2.balance:.2f} ({account2.status})")
    
    print(f"\n✅ SUCCESS! Error handling patterns demonstrated!")
    
    print(f"\nError handling patterns covered:")
    print(f"- ✓ Input validation with custom exceptions")
    print(f"- ✓ Business rule enforcement")
    print(f"- ✓ Domain-specific error types")
    print(f"- ✓ State-dependent operation validation")
    print(f"- ✓ Recovery strategies (retry, graceful degradation)")
    print(f"- ✓ Event store connection error handling")
    print(f"- ✓ Aggregate invariant protection")

if __name__ == "__main__":
    asyncio.run(main())