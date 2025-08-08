# Eventuali Examples

This directory contains comprehensive examples demonstrating Eventuali's event sourcing capabilities, from basic concepts to advanced patterns. All examples use **UV** for Python dependency management and execution.

## 📖 Example Index

### 📚 Basic Examples (01-04)
- **[01 - Basic Event Store Simple](#01---basic-event-store-simple)** - Event persistence, aggregate reconstruction
- **[02 - Aggregate Lifecycle](#02---aggregate-lifecycle)** - Complex business logic, state transitions
- **[03 - Error Handling](#03---error-handling)** - Domain exceptions, validation patterns
- **[04 - Performance Testing](#04---performance-testing)** - Benchmarking, throughput measurement

### 🔄 Intermediate Examples (05-08)
- **[05 - Multi-Aggregate Coordination](#05---multi-aggregate-coordination)** - Cross-aggregate workflows
- **[06 - Event Versioning](#06---event-versioning)** - Schema evolution, backward compatibility
- **[07 - Saga Patterns](#07---saga-patterns)** - Distributed transactions, orchestration
- **[08 - Projections](#08---projections)** - Read models, real-time analytics

### 🚀 Advanced Examples (09-16)
- **[09 - CQRS Patterns](#09---cqrs-patterns)** - Command-Query separation, read models
- **[10 - Event Replay](#10---event-replay)** - Historical state reconstruction, time travel
- **[11 - Distributed Events](#11---distributed-events)** - Multi-node coordination, consensus
- **[12 - Microservices Integration](#12---microservices-integration)** - Service boundaries, coordination
- **[13 - Real-time Dashboards](#13---real-time-dashboards)** - Live data visualization, streaming
- **[14 - Production Monitoring](#14---production-monitoring)** - Health checks, SLA monitoring
- **[15 - Advanced Patterns](#15---advanced-patterns)** - Event versioning, snapshots, temporal queries
- **[16 - Enterprise Features](#16---enterprise-features)** - Security, compliance, HA/DR

### 🔧 CLI Examples (17-20)
- **[17 - CLI Basic Operations](#17---cli-basic-operations)** - CLI fundamentals, configuration management
- **[18 - CLI Database Management](#18---cli-database-management)** - Database workflows, migrations
- **[19 - CLI Performance Monitoring](#19---cli-performance-monitoring)** - Performance analysis via CLI
- **[20 - CLI Production Workflow](#20---cli-production-workflow)** - End-to-end production deployment

### 📸 Snapshot Examples (21)
- **[21 - Snapshots](#21---snapshots)** - Aggregate snapshots for performance optimization

### 🔐 Phase 2 Security & Compliance (22-29)
- **[22 - Event Encryption at Rest](#22---event-encryption-at-rest)** - AES-256-GCM encryption with key management

### 🏢 Phase 2 Multi-tenancy (30-36)
- **[30 - Tenant Isolation Architecture](#30---tenant-isolation-architecture)** - Tenant data isolation patterns
- **[31 - Tenant-aware Event Storage](#31---tenant-aware-event-storage)** - Isolated event storage per tenant
- **[32 - Tenant-scoped Projections](#32---tenant-scoped-projections)** - Tenant-specific read models
- **[33 - Tenant Management API](#33---tenant-management-api)** - Tenant CRUD and configuration

### 📊 Phase 2 Observability (37-43)
- **[37 - OpenTelemetry Integration](#37---opentelemetry-integration)** - Distributed tracing with correlation IDs
- **[38 - Prometheus Metrics Export](#38---prometheus-metrics-export)** - Comprehensive metrics collection
- **[39 - Grafana Dashboard Creation](#39---grafana-dashboard-creation)** - Custom dashboards and alerting
- **[40 - Structured Logging](#40---structured-logging)** - Correlation IDs and log aggregation
- **[41 - Observability Performance Analysis](#41---observability-performance-analysis)** - Performance impact analysis

### ⚡ Phase 2 Performance (44-49)
- **[44 - Connection Pooling Performance](#44---connection-pooling-performance)** - Database connection pool management

---

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

## CLI Examples

### 17 - CLI Basic Operations
**File**: `17_cli_basic_operations.py`
**Concepts**: CLI fundamentals, configuration management, workflow integration

```bash
uv run python ../examples/17_cli_basic_operations.py
```

**What you'll learn:**
- CLI help system and command discovery
- Configuration management and persistent storage
- Database initialization with different backends
- Event querying and inspection workflows
- Schema migration operations
- Error handling and validation patterns
- Development workflow integration

**Expected Output:**
- Comprehensive CLI command demonstrations
- Configuration management workflows
- Database initialization and migration
- Event querying with different output formats
- Error handling and recovery examples
- Complete development workflow patterns

**Performance**: Demonstrates CLI functionality with comprehensive deprecation warnings and improvement roadmap

---

### 18 - CLI Database Management
**File**: `18_cli_database_management.py`
**Concepts**: Advanced database workflows, multi-environment management

```bash
uv run python ../examples/18_cli_database_management.py
```

**What you'll learn:**
- Multi-backend database support (SQLite, PostgreSQL)
- Schema migration workflows and versioning
- Database health monitoring and validation
- Multi-environment configuration management
- Backup and recovery simulation patterns
- Database performance monitoring via CLI

**Expected Output:**
- Database backend compatibility testing
- Migration workflows with version management
- Health check and validation systems
- Multi-environment deployment patterns
- Backup/recovery simulation (100% success rate)
- Performance monitoring and optimization

**Performance**: Production-ready database management with 100% success rate on core workflows

---

### 19 - CLI Performance Monitoring
**File**: `19_cli_performance_monitoring.py`
**Concepts**: Performance analysis, resource monitoring, benchmarking

```bash
uv run python ../examples/19_cli_performance_monitoring.py
```

**What you'll learn:**
- CLI performance baseline establishment
- Benchmark command analysis with timeout protection
- Query performance scaling analysis
- Configuration system performance testing
- Resource usage monitoring patterns
- Performance bottleneck identification

**Expected Output:**
- Baseline performance measurements for CLI operations
- Benchmark command analysis (with known infinite loop protection)
- Query scaling performance across different limit sizes
- Configuration access performance metrics
- Resource usage monitoring (with fallback for missing psutil)
- Critical findings and performance improvement recommendations

**Performance**: CLI performance analysis with timeout protection and known limitations documentation

---

### 20 - CLI Production Workflow
**File**: `20_cli_production_workflow.py`  
**Concepts**: End-to-end production deployment, health monitoring, disaster recovery

```bash
uv run python ../examples/20_cli_production_workflow.py
```

**What you'll learn:**
- Complete production deployment pipeline automation
- Health monitoring and system status checking
- Disaster recovery procedures and validation
- Multi-environment deployment strategies
- Production readiness assessment patterns
- BFB improvement identification and prioritization

**Expected Output:**
- End-to-end deployment pipeline simulation
- Health monitoring with comprehensive status checks
- Disaster recovery workflow demonstrations
- Multi-environment deployment coordination
- Production readiness assessment with detailed findings
- Comprehensive BFB improvement roadmap with 17 specific improvements

**Performance**: Production workflow simulation with comprehensive assessment and improvement roadmap

---

## Phase 2 Examples

### 22 - Event Encryption at Rest
**File**: `22_event_encryption_at_rest.py`
**Concepts**: AES-256-GCM encryption, key management, PBKDF2 derivation

```bash
uv run python ../examples/22_event_encryption_at_rest.py
```

**What you'll learn:**
- AES-256-GCM authenticated encryption for event data
- Secure key generation and password-based key derivation
- Multi-key management with rotation capabilities
- Performance benchmarking showing <5% overhead
- Real-world scenarios: financial, healthcare, e-commerce data

**Expected Output:**
- Comprehensive encryption/decryption demonstrations
- Key management workflows with multiple keys
- Security validation and tampering detection
- Performance benchmarks: 666K+ ops/sec throughput
- Real-world scenario testing with integrity verification

**Performance**: <1ms encryption/decryption, industry-standard security properties

---

### 30 - Tenant Isolation Architecture
**File**: `30_tenant_isolation_architecture.py`
**Concepts**: Multi-tenant data isolation, database-level separation, resource quotas

```bash
uv run python ../examples/30_tenant_isolation_architecture.py
```

**What you'll learn:**
- Database-level tenant isolation with namespace prefixing
- Resource quota management and enforcement
- Tenant validation and security boundaries
- Performance monitoring with <0.1ms overhead
- Multi-tenant architecture patterns

**Expected Output:**
- Tenant creation and validation workflows
- Resource quota enforcement demonstrations
- Isolation validation with 99.9% success rate
- Performance metrics showing minimal overhead
- Security boundary testing

**Performance**: <0.1ms validation overhead, 99.9% isolation success rate

---

### 31 - Tenant-aware Event Storage
**File**: `31_tenant_aware_event_storage.py`
**Concepts**: Isolated event storage, tenant-scoped operations, batch processing

```bash
uv run python ../examples/31_tenant_aware_event_storage.py
```

**What you'll learn:**
- Tenant-isolated event storage with performance optimization
- Batch processing capabilities for high-throughput scenarios
- Tenant-scoped event querying and filtering
- Performance monitoring and metrics collection
- Data integrity and isolation verification

**Expected Output:**
- Tenant-isolated event operations
- Batch processing with <1ms operations
- Performance monitoring and statistics
- Data isolation verification
- Comprehensive tenant storage workflows

**Performance**: <0.1ms single saves, <1ms batch operations, complete tenant isolation

---

### 32 - Tenant-scoped Projections
**File**: `32_tenant_scoped_projections.py`
**Concepts**: Tenant-specific read models, real-time projection processing, analytics

```bash
uv run python ../examples/32_tenant_scoped_projections.py
```

**What you'll learn:**
- Real-time tenant-scoped projection processing
- Analytics projections with tenant isolation
- Performance optimization for multi-tenant projections
- Tenant-specific business intelligence
- Projection isolation and data security

**Expected Output:**
- Real-time projection processing demonstrations
- Tenant-specific analytics and reporting
- Performance metrics: 2.04ms average processing
- Business intelligence projections
- Complete tenant data isolation verification

**Performance**: 2.04ms average processing, 99th percentile 5.01ms, complete isolation

---

### 33 - Tenant Management API
**File**: `33_tenant_management_api.py`
**Concepts**: Tenant CRUD operations, administrative interface, health monitoring

```bash
uv run python ../examples/33_tenant_management_api.py
```

**What you'll learn:**
- Complete tenant lifecycle management
- Administrative API for tenant operations
- Health monitoring and status checking
- Configuration management per tenant
- Audit logging and compliance tracking

**Expected Output:**
- Comprehensive tenant management workflows
- Administrative interface demonstrations
- Health monitoring with detailed metrics
- Configuration management examples
- Audit trail and compliance reporting

**Performance**: 17 operations with 100% success rate, comprehensive administrative interface

---

### 34 - Tenant-Specific Configuration
**File**: `34_tenant_specific_configuration.py`
**Concepts**: Dynamic per-tenant configuration, hot-reload, multi-environment support, validation

```bash
uv run python ../examples/34_tenant_specific_configuration.py
```

**What you'll learn:**
- Dynamic per-tenant configuration with real-time hot-reload capabilities
- Multi-environment configuration management (dev/staging/production/testing)
- Configuration validation and type safety with custom schema enforcement
- Template-based configuration for rapid tenant onboarding
- Comprehensive change tracking and audit logging
- Configuration import/export for backup and migration workflows
- Performance-optimized caching with intelligent cache invalidation
- Configuration rollback capabilities for recovery scenarios

**Expected Output:**
- Dynamic configuration management across multiple environments
- Hot-reload demonstrations with zero-downtime updates
- Template application for rapid tenant provisioning
- Change tracking with full audit trails and history
- Performance metrics showing sub-millisecond retrieval times
- Export/import workflows for configuration backup and migration
- Cross-tenant configuration analysis and comparison

**Performance**: Zero-downtime configuration updates, sub-millisecond retrieval, 90% faster onboarding with templates

---

### 36 - Tenant Metrics
**File**: `36_tenant_metrics.py`
**Concepts**: Advanced observability, SLA monitoring, health scoring, anomaly detection

```bash
uv run python ../examples/36_tenant_metrics.py
```

**What you'll learn:**
- Real-time tenant metrics collection with time-series storage
- Advanced statistical analysis including percentiles, trends, and forecasting
- Intelligent anomaly detection using statistical and ML-based methods
- Custom dashboard creation with flexible widget systems
- SLA monitoring and compliance tracking with automated alerting
- Multi-dimensional health scoring with actionable recommendations
- Cross-tenant performance comparison and benchmarking
- Export capabilities for external monitoring system integration

**Expected Output:**
- Real-time monitoring dashboards with live metric updates
- SLA compliance tracking with violation detection and alerting
- Comprehensive health scoring across multiple performance dimensions
- Anomaly detection with statistical analysis and pattern recognition
- Custom dashboard creation with real-time data visualization
- Cross-tenant analysis showing performance rankings and insights
- Metrics export in multiple formats (JSON, CSV, Prometheus)

**Performance**: Real-time metrics processing, sub-second dashboard updates, comprehensive analytics platform

---

### 37 - OpenTelemetry Integration
**File**: `37_opentelemetry_integration.py`
**Concepts**: Distributed tracing, correlation IDs, span relationships

```bash
uv run python ../examples/37_opentelemetry_integration.py
```

**What you'll learn:**
- Distributed tracing with parent-child span relationships
- Correlation ID tracking across service boundaries
- Event attribution and performance monitoring
- Cross-service trace correlation
- Structured logging with trace context

**Expected Output:**
- Distributed tracing demonstrations
- Correlation ID propagation across operations
- Span relationship visualization
- Performance attribution analysis
- Cross-service tracing patterns

**Performance**: Comprehensive tracing with minimal overhead, production-ready correlation

---

### 38 - Prometheus Metrics Export
**File**: `38_prometheus_metrics_export.py`
**Concepts**: Metrics collection, Prometheus format export, HTTP endpoints

```bash
uv run python ../examples/38_prometheus_metrics_export.py
```

**What you'll learn:**
- Counter, gauge, and histogram metrics collection
- Prometheus-compatible text format export
- HTTP endpoints for metrics scraping
- Business and system health metrics
- Performance metrics with timing distributions

**Expected Output:**
- Comprehensive metrics collection demonstration
- Prometheus format export examples
- HTTP endpoint configuration for scraping
- Business metrics tracking
- System health and performance monitoring

**Performance**: Production-ready metrics with HTTP endpoints, comprehensive coverage

---

### 39 - Grafana Dashboard Creation
**File**: `39_grafana_dashboard_creation.py`
**Concepts**: Dashboard JSON generation, alerting rules, Docker deployment

```bash
uv run python ../examples/39_grafana_dashboard_creation.py
```

**What you'll learn:**
- Complete dashboard JSON configuration generation
- Prometheus alerting rules creation
- Docker Compose stack for monitoring deployment
- Template variables for dynamic filtering
- Advanced query optimization

**Expected Output:**
- Complete Grafana dashboard configurations
- Prometheus alerting rules (4 comprehensive alerts)
- Docker Compose monitoring stack
- Dashboard JSON ready for import
- Production deployment configurations

**Performance**: Complete monitoring stack deployment, production-ready dashboards

---

### 40 - Structured Logging
**File**: `40_structured_logging.py`
**Concepts**: JSON logging, correlation IDs, log aggregation, error tracking

```bash
uv run python ../examples/40_structured_logging.py
```

**What you'll learn:**
- JSON-structured logs with consistent schema
- Correlation ID tracking across distributed operations
- Context propagation through async operations
- Multi-service correlation analysis
- Performance-aware logging with sampling

**Expected Output:**
- Structured JSON logging demonstrations
- Correlation ID propagation examples
- Context tracking across operations
- Business operation logging
- Error tracking with full context

**Performance**: Performance-aware logging, comprehensive context tracking

---

### 41 - Observability Performance Analysis
**File**: `41_observability_performance_analysis.py`
**Concepts**: Performance impact measurement, sampling strategies, optimization

```bash
uv run python ../examples/41_observability_performance_analysis.py
```

**What you'll learn:**
- Observability performance impact analysis
- Sampling strategies for production deployment
- Memory overhead measurement and optimization
- Stress testing with full observability
- Performance tuning recommendations

**Expected Output:**
- Comprehensive performance impact analysis
- Event Creation: 2.20% overhead (with sampling: 1.28%)
- Event Loading: 0.43% overhead (well under target)
- Memory scaling analysis
- Production tuning recommendations

**Performance**: <2% overhead with sampling, production-optimized observability

---

### 44 - Connection Pooling Performance
**File**: `44_connection_pooling_performance.py`
**Concepts**: Database connection pools, automatic scaling, health monitoring

```bash
uv run python ../examples/44_connection_pooling_performance.py
```

**What you'll learn:**
- Connection pool configuration and optimization
- Automatic pool sizing based on load
- Connection health monitoring
- Performance statistics tracking
- Multiple configuration presets

**Expected Output:**
- Connection pool performance demonstrations
- Automatic scaling based on load patterns
- Health monitoring with detailed statistics
- Configuration preset comparisons
- Performance optimization recommendations

**Performance**: Optimized connection management, automatic scaling capabilities

---

### 21 - Snapshots
**File**: `21_snapshots.py`  
**Concepts**: Aggregate snapshots, compression, performance optimization
```bash
# Run from eventuali-python directory
uv run python ../examples/21_snapshots.py
```
**What you'll learn:**
- Creating compressed snapshots of aggregate state  
- Performance comparison: full replay vs snapshot + incremental events
- Automatic snapshot frequency management and cleanup
- Data integrity verification with checksums
- Storage optimization with compression algorithms (gzip)
**Expected Output:**
- Snapshot creation demonstration with compression metrics
- Performance comparison showing 10-20x faster reconstruction
- Compression ratio reporting (60-80% storage reduction)
- Automatic snapshot management simulation
- Data integrity verification with checksum validation
**Performance**: 10-20x faster aggregate reconstruction, 60-80% storage reduction

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

### 🔧 CLI Examples (17-20) - **4/4 COMPLETED**
Comprehensive CLI system demonstration with production workflow patterns:
- CLI basic operations with comprehensive functionality testing
- Advanced database management with multi-backend support
- Performance monitoring with resource usage analysis
- Production workflow automation with deployment pipeline patterns

### 🔐 Phase 2 Security & Compliance (22-29) - **1/8 COMPLETED** ✅ **IN PROGRESS**
Enterprise-grade security and compliance features:
- ✅ Event encryption at rest with AES-256-GCM
- 🚧 Digital signatures for event integrity (in development)
- 🚧 Comprehensive audit trail system (in development)
- 🚧 Production-ready RBAC (in development)
- 🚧 Data retention policies (in development)
- 🚧 GDPR compliance features (in development)
- 🚧 Security scanning and monitoring (in development)
- 🚧 Compliance reporting tools (in development)

### 🏢 Phase 2 Multi-tenancy (30-36) - **6/7 COMPLETED** ✅ **NEARLY COMPLETE**
Complete multi-tenant architecture with enterprise isolation:
- ✅ Tenant isolation architecture with database-level separation
- ✅ Tenant-aware event storage with performance optimization
- ✅ Tenant-scoped projections and real-time analytics
- ✅ Tenant management API with comprehensive administration
- ✅ Tenant-specific configuration system with hot-reload capabilities
- ✅ Tenant metrics and advanced observability dashboard
- 🚧 Resource quotas per tenant (in development)

### 📊 Phase 2 Observability (37-43) - **5/7 COMPLETED** ✅ **NEARLY COMPLETE**
Production-ready observability and monitoring stack:
- ✅ OpenTelemetry integration with distributed tracing
- ✅ Prometheus metrics export with HTTP endpoints
- ✅ Grafana dashboard creation with alerting rules
- ✅ Structured logging with correlation IDs
- ✅ Observability performance analysis (<2% overhead)
- 🚧 Performance profiling capabilities (in development)
- 🚧 Health check endpoints (in development)

### ⚡ Phase 2 Performance (44-49) - **2/6 COMPLETED** ✅ **IN PROGRESS**
Advanced performance optimizations for enterprise deployment:
- ✅ Connection pooling with automatic scaling
- ✅ WAL optimization for database performance (Rust implementation)
- 🚧 Batch processing for high-throughput (framework ready)
- 🚧 Read replicas for query scaling (framework ready)
- 🚧 Multi-level caching strategies (framework ready)
- 🚧 Advanced compression algorithms (framework ready)

### 📊 Phase 2 Progress Summary
**Overall Phase 2 Status: 14/28 features completed (50%)**
- 🔐 Security: 1/8 features (12.5%)
- 🏢 Multi-tenancy: 6/7 features (85.7%) 
- 📊 Observability: 5/7 features (71.4%)
- ⚡ Performance: 2/6 features (33.3%)

### Future Expansion Areas (Phase 3)

**Message Broker Integration (Planned)**:
- **50 - Kafka Integration**: Native streaming with Schema Registry
- **51 - RabbitMQ Integration**: AMQP patterns and routing
- **52 - Redis Streams**: Lightweight streaming patterns

**Cloud Platform Support (Planned)**:
- **53 - AWS Integration**: EventBridge, Lambda, RDS optimization
- **54 - Azure Integration**: Event Hubs, Functions, Cosmos DB
- **55 - GCP Integration**: Pub/Sub, Cloud Functions, Firestore

**Data Pipeline Integration (Planned)**:
- **56 - Apache Spark**: Batch processing integration
- **57 - Apache Flink**: Streaming integration
- **58 - dbt Integration**: Data build tool patterns

*This README is maintained automatically as new examples are added to ensure current and accurate instructions.*