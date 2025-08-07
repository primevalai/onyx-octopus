# Eventuali Python Package

High-performance event sourcing library for Python, powered by Rust.

## Development Setup

### Prerequisites

- Python 3.8+
- Rust (latest stable)
- Maturin for building Python extensions

```bash
pip install maturin[patchelf]
```

### Building the Package

```bash
# Development build (creates a virtual environment and installs the package)
maturin develop

# Production build (creates wheel files)
maturin build --release
```

### Running Tests

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest tests/
```

## Usage Example

```python
import asyncio
from eventuali import EventStore
from eventuali.event import UserRegistered, UserEmailChanged
from eventuali.aggregate import User

async def main():
    # Create event store (SQLite for development)
    store = await EventStore.create("sqlite://events.db")
    
    # Create a new user aggregate
    user = User()
    
    # Apply domain events
    register_event = UserRegistered(name="John Doe", email="john@example.com")
    user.apply(register_event)
    
    user.change_email("john.doe@example.com")
    
    # Save to event store (TODO: implement)
    # await store.save(user)
    
    print(f"User: {user.name} ({user.email})")
    print(f"Version: {user.version}")
    print(f"Uncommitted events: {len(user.get_uncommitted_events())}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Architecture

- **Rust Core**: High-performance event storage and serialization
- **Python API**: Pydantic-based domain modeling with familiar Python patterns
- **Multi-Database**: Supports both PostgreSQL and SQLite
- **Async**: Built with async/await throughout

## Features

- ✅ Event and Aggregate base classes with Pydantic
- ✅ JSON serialization/deserialization
- ✅ Event replay and aggregate reconstruction
- ✅ SQLite and PostgreSQL support (Rust backend)
- ⏳ Event store save/load operations (in progress)
- ⏳ Protocol Buffers for high-performance serialization
- ⏳ Projections and read models
- ⏳ Event streaming and subscriptions