# Eventuali Examples

This directory contains comprehensive examples demonstrating Eventuali's event sourcing capabilities, from basic concepts to advanced patterns. All examples use **UV** for Python dependency management and execution.

## Prerequisites

Before running any examples, ensure you have the development environment set up:

```bash
# Navigate to the Python project directory
cd eventuali/eventuali-python

# Install dependencies and tools
uv sync
uv tool install maturin
uv tool install patchelf

# Build Python bindings
uv run maturin develop
```

## Example Categories

### 📚 **Basic Examples (01-04)**
Learn fundamental event sourcing concepts with practical implementations.

### 🔄 **Intermediate Examples (05-08)**
Advanced patterns for production-ready event sourcing systems.

### 🚀 **Advanced Examples (09-14)** 
*[Coming Soon]* Enterprise patterns and distributed system architectures.

---

## Basic Examples

### 01 - Basic Event Store Simple
**File**: `01_basic_event_store_simple.py`
**Concepts**: Event persistence, aggregate reconstruction, basic CRUD operations

```bash
uv run python ../examples/01_basic_event_store_simple.py
```

**What you'll learn:**
- Creating and persisting domain events
- Loading aggregates from event streams  
- Event store save/load operations
- State reconstruction from events
- Using built-in User aggregate

**Expected Output:**
- User creation and state management
- Email changes and event persistence
- Account deactivation workflows
- Final state verification

**Performance**: Demonstrates basic event store operations with SQLite backend

---

### 02 - Aggregate Lifecycle
**File**: `02_aggregate_lifecycle.py`
**Concepts**: Complex business logic, state transitions, business rule enforcement

```bash
uv run python ../examples/02_aggregate_lifecycle.py
```

**What you'll learn:**
- Complex aggregate design patterns
- Order processing with multiple states
- Business rule validation
- State transition management
- Event-driven business logic

**Expected Output:**
- Complete order lifecycle from creation to fulfillment
- State transitions: Created → Processing → Shipped → Delivered
- Business rule enforcement and validation
- Event sequence demonstration

**Performance**: Shows aggregate complexity handling with proper encapsulation

---

### 03 - Error Handling
**File**: `03_error_handling.py`
**Concepts**: Domain exceptions, validation patterns, recovery strategies

```bash
uv run python ../examples/03_error_handling.py
```

**What you'll learn:**
- Custom domain exception hierarchies
- Input validation and business rule enforcement
- Error recovery strategies (retry, graceful degradation)
- Event store connection error handling
- Compensating transaction patterns

**Expected Output:**
- Validation error demonstrations
- Business rule violation handling
- Recovery strategy examples
- Connection failure resilience

**Performance**: Error handling adds minimal overhead while ensuring data integrity

---

### 04 - Performance Testing
**File**: `04_performance_testing.py`
**Concepts**: Benchmarking, throughput measurement, performance optimization

```bash
uv run python ../examples/04_performance_testing.py
```

**What you'll learn:**
- Event creation and serialization benchmarking
- Aggregate processing throughput measurement
- Memory usage optimization patterns
- Batch processing techniques
- Performance monitoring strategies

**Expected Output:**
- **64k+ events/sec** creation speed
- **30k+ events/sec** persistence rate  
- **49k+ events/sec** loading speed
- Memory efficiency metrics
- Performance comparison baselines

**Performance**: Demonstrates the 10-60x speed improvements over pure Python implementations

---

## Intermediate Examples

### 05 - Multi-Aggregate Coordination
**File**: `05_multi_aggregate_simple.py`
**Concepts**: Cross-aggregate workflows, coordination patterns, eventual consistency

```bash
uv run python ../examples/05_multi_aggregate_simple.py
```

**What you'll learn:**
- Coordinating multiple aggregates in business processes
- Inventory management with reservation patterns
- Customer validation across aggregate boundaries
- Compensating actions for failed transactions
- Event-driven coordination without tight coupling

**Expected Output:**
- Order processing with inventory coordination
- Customer validation workflows
- Success and failure scenarios
- Compensating transaction demonstrations
- Concurrent processing examples

**Performance**: Multi-aggregate coordination with automatic rollback capabilities

---

### 06 - Event Versioning  
**File**: `06_event_versioning.py`
**Concepts**: Schema evolution, backward compatibility, event migration

```bash
uv run python ../examples/06_event_versioning.py
```

**What you'll learn:**
- Event schema versioning strategies (V1 → V2 → V3)
- Backward and forward compatibility patterns
- Event upcasting and migration services
- Breaking change management
- Gradual system evolution techniques

**Expected Output:**
- Multiple event version demonstrations
- Automatic event upcasting examples
- Migration scenario testing
- Compatibility verification
- Version evolution patterns

**Performance**: Seamless schema migration with zero downtime patterns

---

### 07 - Saga Patterns
**File**: `07_saga_patterns.py` 
**Concepts**: Distributed transactions, orchestration, compensation actions

```bash
uv run python ../examples/07_saga_patterns.py
```

**What you'll learn:**
- Orchestrated saga coordination patterns
- Multi-step distributed transaction management
- Compensating actions for failure recovery
- State machine-based saga implementation
- Concurrent saga execution patterns

**Expected Output:**
- Payment → Inventory → Shipping workflow
- Success and failure scenarios with compensation
- **~214ms average** saga execution time
- Concurrent saga processing
- Event analysis and statistics

**Performance**: High-throughput saga processing with reliable compensation handling

---

### 08 - Projections
**File**: `08_projections.py`
**Concepts**: Read models, real-time analytics, event-driven projections

```bash
uv run python ../examples/08_projections.py
```

**What you'll learn:**
- Building optimized read models from event streams
- Real-time projection updates and analytics
- User activity tracking patterns
- Category-level sales analytics
- Dashboard and reporting projections

**Expected Output:**
- **78k+ events/sec** projection processing
- Real-time user activity analytics
- Category sales reporting
- Conversion funnel metrics
- Projection rebuilding demonstrations

**Performance**: Ultra-fast projection building with real-time analytics capabilities

---

## Legacy Examples

### Basic Usage (Standalone)
**File**: `basic_usage.py`
**Status**: Legacy - demonstrates core concepts without event store integration

```bash
uv run python ../examples/basic_usage.py
```

**Purpose**: Shows fundamental event sourcing concepts in isolation

---

### Streaming Example
**File**: `streaming_example.py`
**Status**: Legacy - early streaming implementation  

```bash
uv run python ../examples/streaming_example.py
```

**Purpose**: Demonstrates event streaming concepts (superseded by examples 07-08)

---

## Running All Examples

### Quick Verification (Basic Examples)
Test core functionality with the essential examples:

```bash
cd eventuali-python

# Run basic examples in sequence
uv run python ../examples/01_basic_event_store_simple.py
uv run python ../examples/02_aggregate_lifecycle.py  
uv run python ../examples/03_error_handling.py
uv run python ../examples/04_performance_testing.py
```

### Full Example Suite
Run all examples to see the complete feature set:

```bash
cd eventuali-python

# Basic examples
for example in 01 02 03 04; do
    echo "=== Running Example ${example} ==="
    uv run python ../examples/${example}_*.py
done

# Intermediate examples  
for example in 05 06 07 08; do
    echo "=== Running Example ${example} ==="
    uv run python ../examples/${example}_*.py
done
```

### Batch Execution Script
```bash
#!/bin/bash
cd eventuali-python

examples=(
    "01_basic_event_store_simple.py"
    "02_aggregate_lifecycle.py"
    "03_error_handling.py"
    "04_performance_testing.py"
    "05_multi_aggregate_simple.py"
    "06_event_versioning.py"
    "07_saga_patterns.py"
    "08_projections.py"
)

for example in "${examples[@]}"; do
    echo "🚀 Running $example"
    uv run python "../examples/$example"
    echo "✅ Completed $example"
    echo "----------------------------------------"
done
```

---

## Performance Benchmarks

| Example | Metric | Performance |
|---------|--------|-------------|
| 01 - Basic Event Store | Event persistence | Standard SQLite speeds |
| 02 - Aggregate Lifecycle | State transitions | Complex business logic handling |
| 03 - Error Handling | Exception handling | Minimal overhead with validation |
| 04 - Performance Testing | **Throughput** | **64k+ events/sec creation** |
| 05 - Multi-Aggregate | Coordination | Multi-step workflows with rollback |
| 06 - Event Versioning | Schema migration | Seamless version evolution |
| 07 - Saga Patterns | **Distributed TX** | **~214ms average execution** |
| 08 - Projections | **Analytics** | **78k+ events/sec processing** |

---

## Development Guidelines

### Adding New Examples

When creating new examples, follow this structure:

1. **File Naming**: Use sequential numbering: `09_feature_name.py`
2. **Header Documentation**: Include comprehensive docstring explaining concepts
3. **UV Commands**: All execution must use `uv run python`
4. **Performance Metrics**: Include expected execution times/throughput
5. **Error Handling**: Demonstrate proper error scenarios
6. **Documentation**: Update this README with atomic instructions

### Code Quality Standards

All examples must pass these quality checks:

```bash
# Format code
uv run black ../examples/

# Lint check  
uv run ruff check ../examples/ --fix

# Type checking (if enabled)
uv run mypy ../examples/
```

### Testing Examples

Before committing new examples, verify they work:

```bash
# Build bindings
uv run maturin develop

# Test specific example
uv run python ../examples/your_new_example.py

# Verify performance expectations
# Ensure clean execution without warnings
```

---

## Troubleshooting

### Common Issues

**Import Errors**
```bash
# Rebuild bindings if import errors occur
cd eventuali-python
uv run maturin develop
```

**Performance Issues**
```bash
# Ensure release build for performance testing
cargo build --release
uv run maturin develop --release
```

**UV Command Not Found**
```bash
# Install UV if not available
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Example Execution Failures**
```bash
# Verify working directory
cd eventuali-python  # Must be in this directory
uv run python ../examples/example_name.py
```

### Getting Help

- Check the main [README](../README.md) for setup instructions
- Review [CLAUDE.md](../CLAUDE.md) for development standards  
- Examine working examples for implementation patterns
- Verify UV and maturin installations are current

---

## Future Examples

### Advanced Examples (Planned)

- **09 - CQRS Patterns**: Command-Query Responsibility Segregation
- **10 - Event Replay**: Historical event processing and reconstruction  
- **11 - Distributed Events**: Multi-node event sourcing patterns
- **12 - Microservices Integration**: Service coordination via events
- **13 - Real-time Dashboards**: Live analytics and monitoring
- **14 - Monitoring & Observability**: Metrics, tracing, and debugging

### Enterprise Examples (Future)

- **15 - Security & Authorization**: Event-level security patterns
- **16 - Multi-tenancy**: Tenant isolation in event sourcing
- **17 - Cloud Integration**: AWS/GCP/Azure event sourcing
- **18 - Event Streaming**: Kafka/Pulsar integration patterns

*This README is maintained automatically as new examples are added to ensure current and accurate instructions.*