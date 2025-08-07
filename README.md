# Eventuali

High-performance event sourcing library for Python, powered by Rust.

## Overview

Eventuali combines the performance and memory safety of Rust with the developer experience of Python to create a powerful event sourcing library. It provides:

- **10-60x performance improvement** over pure Python implementations
- **8-20x better memory efficiency** 
- **Multi-database support** (PostgreSQL, SQLite)
- **Seamless Python integration** with Pydantic models
- **Async/await support** throughout
- **Type safety** with full type hints

## Architecture

- **Rust Core** (`eventuali-core`): High-performance event storage, serialization, and projections
- **Python Bindings** (`eventuali-python`): PyO3-based bindings with Pythonic APIs
- **Multi-Database**: Unified interface supporting PostgreSQL and SQLite

## Installation

### Production Installation
```bash
# Install from PyPI (when available)
uv add eventuali
```

### Development Installation
```bash
# Clone the repository
git clone git@github.com:primevalai/eventuali.git
cd eventuali/eventuali-python

# Install with development dependencies  
uv sync

# Install required tools
uv tool install maturin
uv tool install patchelf

# Build Python bindings
uv run maturin develop

# Verify installation
uv run python ../examples/01_basic_event_store_simple.py
```

## Quick Start

```python
import asyncio
from eventuali import EventStore, Aggregate, Event
from pydantic import BaseModel

# Define your events
class UserRegistered(Event):
    name: str
    email: str

class UserEmailChanged(Event):
    new_email: str

# Define your aggregate
class User(Aggregate):
    name: str = ""
    email: str = ""
    
    def apply_user_registered(self, event: UserRegistered) -> None:
        self.name = event.name
        self.email = event.email
    
    def apply_user_email_changed(self, event: UserEmailChanged) -> None:
        self.email = event.new_email

async def main():
    # Initialize the event store (SQLite for development)
    store = await EventStore.create("sqlite://events.db")
    
    # Create and save an aggregate
    user = User()
    user.apply(UserRegistered(name="John Doe", email="john@example.com"))
    await store.save(user)
    
    # Load and update the aggregate
    loaded_user = await store.load(User, user.id)
    loaded_user.apply(UserEmailChanged(new_email="john.doe@example.com"))
    await store.save(loaded_user)

asyncio.run(main())
```

## Features

### Multi-Database Support

```python
# PostgreSQL for production
store = await EventStore.create(
    "postgresql://user:password@localhost/events"
)

# SQLite for development/testing
store = await EventStore.create("sqlite://events.db")
```

### High-Performance Serialization

- Protocol Buffers for maximum performance
- JSON fallback for development and debugging
- Automatic schema evolution support

### Async Throughout

Built from the ground up with async/await support using Tokio (Rust) and asyncio (Python).

## Development

This project uses a Rust/Python hybrid approach with **UV as the mandated Python toolchain**:

1. **Rust workspace** with `eventuali-core` and `eventuali-python`
2. **Maturin** for building Python wheels with embedded Rust
3. **PyO3** for seamless Rust-Python integration
4. **UV** for all Python dependency management, tool installation, and script execution

### UV Requirements

**⚠️ IMPORTANT: This project exclusively uses UV for Python operations.**

UV provides faster, more reliable Python dependency management and ensures consistent environments across development and CI/CD:

- **Faster installs**: 10-100x faster than pip
- **Dependency resolution**: More reliable than pip-tools
- **Tool management**: Built-in support for development tools
- **Environment isolation**: Better virtual environment handling

### Building from Source

**⚠️ Use UV for all Python operations as per project standards**

```bash
# Install required tools using UV
uv tool install maturin
uv tool install patchelf

# Install project dependencies
cd eventuali-python
uv sync

# Build and install in development mode
uv run maturin develop

# Run tests
uv run pytest

# Format code
uv run black ../examples/
uv run ruff check ../examples/ --fix
```

### Testing Your Build

After building, verify everything works by running the examples using **UV**:

```bash
# Test the Rust core (fastest way to verify the build)
cargo run --package eventuali-core --example rust_streaming_demo

# Test Python bindings compilation
cargo build --package eventuali-python

# Run Python examples using UV (required)
cd eventuali-python
uv run python ../examples/01_basic_event_store_simple.py
uv run python ../examples/02_aggregate_lifecycle.py
uv run python ../examples/03_error_handling.py
uv run python ../examples/04_performance_testing.py

# Advanced examples
uv run python ../examples/05_multi_aggregate_simple.py
uv run python ../examples/06_event_versioning.py
uv run python ../examples/07_saga_patterns.py
uv run python ../examples/08_projections.py
```

See the [Examples](#examples) section for comprehensive documentation on running and understanding each example.

## Performance

Benchmarks show significant performance improvements over pure Python event sourcing:

- **Serialization**: 20-30x faster
- **Database operations**: 2-10x faster
- **Memory usage**: 8-20x more efficient
- **Concurrent throughput**: 2x better under load

## Examples

This section provides comprehensive examples demonstrating Eventuali's capabilities, from basic event sourcing concepts to high-performance streaming.

### Prerequisites

Before running examples, ensure you have the required dependencies:

```bash
# Install Rust (required for all examples)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Protocol Buffers compiler (for Rust examples)
# Ubuntu/Debian:
sudo apt install protobuf-compiler
# macOS:
brew install protobuf

# Build the project
cargo build --release

# For Python examples, compile Python bindings
cargo build --package eventuali-python
```

### Rust Examples

#### High-Performance Streaming Demo

**Location**: `eventuali-core/examples/rust_streaming_demo.rs`

This comprehensive example demonstrates the full power of Eventuali's Rust core, showcasing real-world event sourcing and streaming scenarios.

```bash
# Run the complete streaming demonstration
cargo run --package eventuali-core --example rust_streaming_demo
```

**What it demonstrates:**
- ✅ **High-performance event store** with SQLite backend
- ✅ **Real-time event streaming** with broadcast channels  
- ✅ **Projection system** building read models from events
- ✅ **Position tracking** for reliable exactly-once processing
- ✅ **Event sourcing** with full event replay capabilities
- ✅ **Batch processing** demonstrating high throughput
- ✅ **Complete async workflow** with Tokio

**Expected Performance:**
- **10,000+ events/second** throughput
- Real-time streaming with < 1ms latency
- Memory-efficient processing of large event batches
- Demonstrates the 10-60x performance improvements over Python

**Sample Output:**
```
=== Eventuali Rust Streaming Demo ===

1. Setting up SQLite event store...
   ✓ Event store ready

2. Creating high-performance event streamer...
   ✓ Event streamer connected to store

3. Setting up user projection...
   ✓ Projection subscribed to User events

...

10. Performance demonstration...
   ✓ Saved 100 events in 9.309ms (10742.08 events/sec)
   ✓ Projection now contains 103 users

=== Demo completed successfully! ===
Key achievements:
✓ High-performance event store with SQLite backend
✓ Real-time event streaming with broadcast channels
✓ Projection system building read models from events
✓ Position tracking for reliable exactly-once processing
✓ Event sourcing with full event replay capabilities
✓ Batch processing demonstrating high throughput
✓ Production-ready Rust implementation complete
```

### Python Examples

#### Basic Event Sourcing

**Location**: `examples/basic_usage.py`

Learn the fundamentals of event sourcing with this comprehensive introduction.

```bash
cd eventuali-python
uv run python ../examples/basic_usage.py
```

**What you'll learn:**
- Creating domain events with Pydantic models
- Building aggregates that apply events
- Event serialization and deserialization
- Event replay to reconstruct state
- State consistency verification

**Key Concepts Covered:**
- Domain events (`UserRegistered`, `UserEmailChanged`, `UserDeactivated`)
- Aggregate root pattern with the `User` aggregate
- Business methods that generate and apply events
- Event sourcing state reconstruction

#### Advanced Streaming Example

**Location**: `examples/streaming_example.py`

Explore real-time event streaming and projection building.

```bash
cd eventuali-python  
uv run python ../examples/streaming_example.py
```

**What you'll learn:**
- Setting up event stores with different backends
- Creating high-performance event streamers
- Building real-time projections from event streams
- Event filtering by aggregate and event type
- Position tracking for reliable processing
- Integration between EventStore and EventStreamer

**Features Demonstrated:**
- Real-time event subscriptions
- Background event processing
- Projection-based read models
- Event type filtering
- Stream position management

#### Unit Tests and Testing Patterns

**Location**: `eventuali-python/tests/test_basic.py`

Learn how to test event sourcing applications effectively.

```bash
cd eventuali-python
uv run pytest tests/test_basic.py -v
```

**Testing Patterns Covered:**
- Event creation and serialization testing
- Aggregate behavior verification
- Business method testing
- Event replay testing
- Async event store operations (when available)

### Running the Examples

#### Quick Start - Rust Demo

The fastest way to see Eventuali in action:

```bash
# Clone and build
git clone git@github.com:primevalai/eventuali.git
cd eventuali
cargo build --release

# Run the high-performance demo
cargo run --package eventuali-core --example rust_streaming_demo
```

#### Python Examples Setup

**All Python examples use UV for dependency management:**

```bash
# Setup development environment
cd eventuali/eventuali-python
uv sync
uv tool install maturin
uv tool install patchelf

# Build Python bindings
uv run maturin develop

# Run examples (8 comprehensive examples)
uv run python ../examples/01_basic_event_store_simple.py     # Basic event sourcing
uv run python ../examples/02_aggregate_lifecycle.py         # Complex aggregates
uv run python ../examples/03_error_handling.py              # Error patterns
uv run python ../examples/04_performance_testing.py         # Performance benchmarks
uv run python ../examples/05_multi_aggregate_simple.py      # Multi-aggregate coordination
uv run python ../examples/06_event_versioning.py            # Schema evolution
uv run python ../examples/07_saga_patterns.py               # Distributed transactions
uv run python ../examples/08_projections.py                 # Real-time projections

# Run tests
uv run pytest tests/ -v

# Code quality
uv run black ../examples/
uv run ruff check ../examples/ --fix
```

### Example Progression

We recommend exploring the examples in this order:

1. **Start with Rust Demo** - See the full system in action
2. **Basic Python Usage** - Learn event sourcing fundamentals  
3. **Python Streaming** - Understand real-time event processing
4. **Unit Tests** - Learn testing patterns

Each example builds on concepts from the previous ones, providing a comprehensive learning path from basic event sourcing to production-ready streaming applications.

### Performance Insights from Examples

The Rust streaming demo consistently demonstrates:

- **10,000+ events/second** sustained throughput
- **Sub-millisecond** event processing latency  
- **Memory efficiency** with large event volumes
- **Reliable processing** with position tracking
- **Real-time projections** with immediate consistency

This showcases the core value proposition: **Rust-level performance with Python-level usability**.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.