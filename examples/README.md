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

### 🚀 **Advanced Examples (09-16)** ✅ **COMPLETED**
Enterprise-grade patterns and distributed system architectures for production deployments.

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

---

## Advanced Examples

### 09 - CQRS Patterns
**File**: `09_cqrs_patterns.py`
**Concepts**: Command-Query Responsibility Segregation, multiple read models, optimized queries

```bash
uv run python ../examples/09_cqrs_patterns.py
```

**What you'll learn:**
- Command-Query separation principles
- Multiple specialized read models for different query patterns
- Optimized query performance through CQRS
- Command validation and processing
- Read model synchronization patterns

**Expected Output:**
- Command processing with validation
- Multiple read models: user profiles, order summaries, analytics
- Query performance optimization demonstrations
- 20 commands processed across 3 specialized read models

**Performance**: Demonstrates query optimization through dedicated read models

---

### 10 - Event Replay
**File**: `10_event_replay.py`
**Concepts**: Historical state reconstruction, time travel queries, checkpoint optimization

```bash
uv run python ../examples/10_event_replay.py
```

**What you'll learn:**
- Event replay for state reconstruction
- Time travel queries to historical states
- Checkpoint creation for replay optimization
- Historical analysis and debugging
- Point-in-time aggregate recovery

**Expected Output:**
- Historical state reconstruction from events
- Time travel to specific points in aggregate history
- 16 events with 3 checkpoints for optimization
- State verification at different time points

**Performance**: Efficient historical state reconstruction with checkpoint optimization

---

### 11 - Distributed Events
**File**: `11_distributed_events.py`
**Concepts**: Multi-node coordination, consensus, failover, event deduplication

```bash
uv run python ../examples/11_distributed_events.py
```

**What you'll learn:**
- Multi-node event coordination and consensus
- Cross-region event replication patterns
- Node failure detection and automatic failover
- Event deduplication and idempotency guarantees
- Distributed system health monitoring
- Node recovery and synchronization

**Expected Output:**
- 5-node distributed cluster setup
- Event replication with 60% consensus threshold
- Node failure simulation and recovery
- 100% system availability with automatic failover
- 353.8% replication rate across nodes

**Performance**: Enterprise-grade distributed event coordination with consensus

---

### 12 - Microservices Integration
**File**: `12_microservices_integration.py`
**Concepts**: Service boundaries, event-driven communication, service coordination

```bash
uv run python ../examples/12_microservices_integration.py
```

**What you'll learn:**
- Event-driven microservices architecture
- Service boundary design patterns
- Inter-service communication via events
- Service coordination and choreography
- Distributed transaction patterns

**Expected Output:**
- 4 microservices: Order, Inventory, Payment, Notification
- Event-driven service communication
- Cross-service workflow coordination
- 100% event processing success rate
- Service health monitoring and metrics

**Performance**: High-throughput microservices coordination with reliable event delivery

---

### 13 - Real-time Dashboards
**File**: `13_realtime_dashboards.py`
**Concepts**: Live data visualization, streaming updates, real-time analytics

```bash
uv run python ../examples/13_realtime_dashboards.py
```

**What you'll learn:**
- Real-time dashboard updates from event streams
- Live data visualization patterns
- Streaming analytics and metrics
- WebSocket-style real-time updates
- Dashboard performance optimization

**Expected Output:**
- Live dashboard with streaming updates
- Real-time metrics: user activity, sales, performance
- 15-second simulation with 1.1 updates/second
- Dashboard responsiveness metrics
- Real-time alert generation

**Performance**: Sub-second dashboard updates with streaming analytics

---

### 14 - Production Monitoring
**File**: `14_production_monitoring.py`
**Concepts**: Health checks, metrics collection, SLA monitoring, incident management

```bash
uv run python ../examples/14_production_monitoring.py
```

**What you'll learn:**
- Production system health monitoring
- SLA tracking and alerting
- Performance metrics collection
- Incident detection and response
- Monitoring dashboard design

**Expected Output:**
- Comprehensive health check system
- SLA monitoring with 99.9% uptime target
- 6 health checks with detailed metrics
- Automated incident detection and response
- Performance monitoring and alerting

**Performance**: Production-grade monitoring with comprehensive health tracking

---

### 15 - Advanced Patterns
**File**: `15_advanced_patterns.py`
**Concepts**: Event versioning, snapshots, temporal queries, multi-tenancy

```bash
uv run python ../examples/15_advanced_patterns.py
```

**What you'll learn:**
- Advanced event versioning with schema evolution
- Snapshot creation and management
- Temporal queries across time ranges
- Multi-tenant data isolation
- Advanced performance optimization patterns

**Expected Output:**
- Event versioning V1→V2 with backward compatibility
- Snapshot creation with 73.4% compression ratio
- Temporal queries across multiple time periods
- Multi-tenant data isolation demonstration
- Advanced pattern performance metrics

**Performance**: Enterprise-grade patterns with optimal performance characteristics

---

### 16 - Enterprise Features
**File**: `16_enterprise_features.py`
**Concepts**: Security, compliance, HA/DR, business intelligence

```bash
uv run python ../examples/16_enterprise_features.py
```

**What you'll learn:**
- Enterprise security patterns and encryption
- Compliance and audit trail management
- High availability and disaster recovery
- Business intelligence and reporting
- Enterprise-grade performance optimization

**Expected Output:**
- Security framework with encryption and access control
- Compliance tracking with audit trails
- HA/DR setup with automatic failover
- Business intelligence dashboards and reporting
- Enterprise performance metrics and optimization

**Performance**: Enterprise-scale deployment with security, compliance, and HA/DR

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

# Basic examples (01-04)
for example in 01 02 03 04; do
    echo "=== Running Example ${example} ==="
    uv run python ../examples/${example}_*.py
done

# Intermediate examples (05-08)
for example in 05 06 07 08; do
    echo "=== Running Example ${example} ==="
    uv run python ../examples/${example}_*.py
done

# Advanced examples (09-16)
for example in 09 10 11 12 13 14 15 16; do
    echo "=== Running Example ${example} ==="
    uv run python ../examples/${example}_*.py
done
```

### Batch Execution Script
```bash
#!/bin/bash
cd eventuali-python

examples=(
    # Basic Examples
    "01_basic_event_store_simple.py"
    "02_aggregate_lifecycle.py"
    "03_error_handling.py"
    "04_performance_testing.py"
    # Intermediate Examples
    "05_multi_aggregate_simple.py"
    "06_event_versioning.py"
    "07_saga_patterns.py"
    "08_projections.py"
    # Advanced Examples
    "09_cqrs_patterns.py"
    "10_event_replay.py"
    "11_distributed_events.py"
    "12_microservices_integration.py"
    "13_realtime_dashboards.py"
    "14_production_monitoring.py"
    "15_advanced_patterns.py"
    "16_enterprise_features.py"
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

### Basic Examples (01-04)
| Example | Metric | Performance |
|---------|--------|-------------|
| 01 - Basic Event Store | Event persistence | Standard SQLite speeds |
| 02 - Aggregate Lifecycle | State transitions | Complex business logic handling |
| 03 - Error Handling | Exception handling | Minimal overhead with validation |
| 04 - Performance Testing | **Throughput** | **64k+ events/sec creation** |

### Intermediate Examples (05-08)
| Example | Metric | Performance |
|---------|--------|-------------|
| 05 - Multi-Aggregate | Coordination | Multi-step workflows with rollback |
| 06 - Event Versioning | Schema migration | Seamless version evolution |
| 07 - Saga Patterns | **Distributed TX** | **~214ms average execution** |
| 08 - Projections | **Analytics** | **78k+ events/sec processing** |

### Advanced Examples (09-16)
| Example | Metric | Performance |
|---------|--------|-------------|
| 09 - CQRS Patterns | **Read Models** | **3 optimized read models** |
| 10 - Event Replay | **Time Travel** | **16 events, 3 checkpoints** |
| 11 - Distributed Events | **Consensus** | **100% availability, 353.8% replication** |
| 12 - Microservices | **Integration** | **4 services, 100% success rate** |
| 13 - Real-time Dashboards | **Streaming** | **1.1 updates/sec live analytics** |
| 14 - Production Monitoring | **Health Checks** | **6 checks, 99.9% SLA** |
| 15 - Advanced Patterns | **Snapshots** | **73.4% compression ratio** |
| 16 - Enterprise Features | **Security & HA/DR** | **Enterprise-grade compliance** |

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

## Example Categories Summary

### 📚 Basic Examples (01-04) - **4/4 COMPLETED**
Foundational event sourcing concepts with practical implementations:
- Event persistence and aggregate reconstruction
- Complex business logic and state transitions  
- Error handling and recovery strategies
- Performance benchmarking and optimization

### 🔄 Intermediate Examples (05-08) - **4/4 COMPLETED**
Advanced patterns for production-ready event sourcing systems:
- Multi-aggregate coordination and workflows
- Event versioning and schema evolution
- Saga patterns for distributed transactions
- Real-time projections and analytics

### 🚀 Advanced Examples (09-16) - **8/8 COMPLETED**
Enterprise-grade patterns and distributed system architectures:
- CQRS patterns with multiple read models
- Event replay and time travel queries
- Distributed events with consensus and failover
- Microservices integration and coordination
- Real-time dashboards and streaming analytics
- Production monitoring and health checks
- Advanced patterns: snapshots, temporal queries, multi-tenancy
- Enterprise features: security, compliance, HA/DR, business intelligence

### Future Expansion Areas

**Cloud Integration (Planned)**:
- **17 - AWS Integration**: EventBridge, Lambda, RDS optimization
- **18 - Azure Integration**: Event Hubs, Functions, Cosmos DB
- **19 - GCP Integration**: Pub/Sub, Cloud Functions, Firestore

**Message Broker Integration (Planned)**:
- **20 - Kafka Integration**: Native streaming with Schema Registry
- **21 - RabbitMQ Integration**: AMQP patterns and routing
- **22 - Redis Streams**: Lightweight streaming patterns

*This README is maintained automatically as new examples are added to ensure current and accurate instructions.*