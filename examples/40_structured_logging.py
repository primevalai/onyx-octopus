#!/usr/bin/env python3
"""
Example 40: Structured Logging with Correlation IDs and Log Aggregation Patterns

This example demonstrates:
1. Structured logging with correlation IDs for event sourcing operations
2. JSON-formatted logs for easy parsing and aggregation
3. Log correlation across distributed operations
4. Integration with logging frameworks (loguru, structlog-style formatting)
5. Log aggregation patterns for centralized monitoring
6. Context propagation through async operations
7. Performance-aware logging with sampling

The implementation shows comprehensive logging strategies for production
event sourcing systems with correlation tracking and centralized aggregation.
"""

import asyncio
import json
import time
import uuid
import logging
import sys
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, field, asdict
from contextlib import asynccontextmanager
from enum import Enum
import threading
from collections import defaultdict, deque
import random
import traceback

import eventuali


class LogLevel(Enum):
    """Log levels for structured logging"""
    TRACE = "TRACE"
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARN = "WARN"
    ERROR = "ERROR"
    FATAL = "FATAL"


@dataclass
class CorrelationContext:
    """Context for correlation tracking across operations"""
    correlation_id: str
    trace_id: Optional[str] = None
    span_id: Optional[str] = None
    parent_span_id: Optional[str] = None
    user_id: Optional[str] = None
    tenant_id: Optional[str] = None
    operation: Optional[str] = None
    session_id: Optional[str] = None
    request_id: Optional[str] = None
    
    def __post_init__(self):
        if self.trace_id is None:
            self.trace_id = str(uuid.uuid4())
        if self.span_id is None:
            self.span_id = str(uuid.uuid4())


@dataclass
class LogRecord:
    """Structured log record for event sourcing operations"""
    timestamp: datetime
    level: LogLevel
    message: str
    logger_name: str
    correlation_context: CorrelationContext
    event_data: Dict[str, Any] = field(default_factory=dict)
    performance_data: Dict[str, Any] = field(default_factory=dict)
    error_data: Optional[Dict[str, Any]] = None
    tags: List[str] = field(default_factory=list)
    source_location: Optional[Dict[str, Any]] = None
    
    def to_json(self) -> str:
        """Convert log record to JSON string for structured logging"""
        record_dict = {
            "timestamp": self.timestamp.isoformat(),
            "level": self.level.value,
            "message": self.message,
            "logger": self.logger_name,
            "correlation_id": self.correlation_context.correlation_id,
            "trace_id": self.correlation_context.trace_id,
            "span_id": self.correlation_context.span_id,
            "parent_span_id": self.correlation_context.parent_span_id,
            "user_id": self.correlation_context.user_id,
            "tenant_id": self.correlation_context.tenant_id,
            "operation": self.correlation_context.operation,
            "session_id": self.correlation_context.session_id,
            "request_id": self.correlation_context.request_id,
            "event_data": self.event_data,
            "performance_data": self.performance_data,
            "error_data": self.error_data,
            "tags": self.tags,
            "source_location": self.source_location
        }
        
        # Remove None values for cleaner JSON
        return json.dumps({k: v for k, v in record_dict.items() if v is not None})


class StructuredLogger:
    """High-performance structured logger with correlation ID support"""
    
    def __init__(self, name: str, min_level: LogLevel = LogLevel.INFO, 
                 enable_sampling: bool = True, sample_rate: float = 1.0):
        self.name = name
        self.min_level = min_level
        self.enable_sampling = enable_sampling
        self.sample_rate = sample_rate
        self.log_buffer = deque(maxlen=10000)  # Circular buffer for recent logs
        self.lock = threading.Lock()
        self.stats = defaultdict(int)
        
        # Performance tracking
        self.log_count = 0
        self.start_time = time.time()
        
    def _should_log(self, level: LogLevel) -> bool:
        """Determine if log should be written based on level and sampling"""
        level_values = {
            LogLevel.TRACE: 0,
            LogLevel.DEBUG: 1,
            LogLevel.INFO: 2,
            LogLevel.WARN: 3,
            LogLevel.ERROR: 4,
            LogLevel.FATAL: 5
        }
        
        if level_values[level] < level_values[self.min_level]:
            return False
        
        if self.enable_sampling and level in [LogLevel.DEBUG, LogLevel.TRACE]:
            return random.random() < self.sample_rate
        
        return True
    
    def _get_source_location(self) -> Dict[str, Any]:
        """Get source code location for debugging"""
        import inspect
        frame = inspect.currentframe()
        try:
            # Go up the stack to find the actual caller
            caller_frame = frame.f_back.f_back.f_back
            if caller_frame:
                return {
                    "file": caller_frame.f_code.co_filename,
                    "function": caller_frame.f_code.co_name,
                    "line": caller_frame.f_lineno
                }
        finally:
            del frame
        return None
    
    def _log(self, level: LogLevel, message: str, correlation_context: CorrelationContext,
             event_data: Dict[str, Any] = None, performance_data: Dict[str, Any] = None,
             error_data: Dict[str, Any] = None, tags: List[str] = None):
        """Internal logging method"""
        
        if not self._should_log(level):
            return
        
        # Create log record
        record = LogRecord(
            timestamp=datetime.now(timezone.utc),
            level=level,
            message=message,
            logger_name=self.name,
            correlation_context=correlation_context,
            event_data=event_data or {},
            performance_data=performance_data or {},
            error_data=error_data,
            tags=tags or [],
            source_location=self._get_source_location()
        )
        
        # Output to console (in production, this would go to proper log aggregation)
        log_json = record.to_json()
        print(log_json)
        
        # Store in buffer for analysis
        with self.lock:
            self.log_buffer.append(record)
            self.stats[level.value] += 1
            self.log_count += 1
    
    def trace(self, message: str, correlation_context: CorrelationContext, **kwargs):
        """Log trace level message"""
        self._log(LogLevel.TRACE, message, correlation_context, **kwargs)
    
    def debug(self, message: str, correlation_context: CorrelationContext, **kwargs):
        """Log debug level message"""
        self._log(LogLevel.DEBUG, message, correlation_context, **kwargs)
    
    def info(self, message: str, correlation_context: CorrelationContext, **kwargs):
        """Log info level message"""
        self._log(LogLevel.INFO, message, correlation_context, **kwargs)
    
    def warn(self, message: str, correlation_context: CorrelationContext, **kwargs):
        """Log warning level message"""
        self._log(LogLevel.WARN, message, correlation_context, **kwargs)
    
    def error(self, message: str, correlation_context: CorrelationContext, **kwargs):
        """Log error level message"""
        self._log(LogLevel.ERROR, message, correlation_context, **kwargs)
    
    def fatal(self, message: str, correlation_context: CorrelationContext, **kwargs):
        """Log fatal level message"""
        self._log(LogLevel.FATAL, message, correlation_context, **kwargs)
    
    def log_event_operation(self, operation: str, correlation_context: CorrelationContext,
                           event_type: str, aggregate_id: str, success: bool,
                           duration_ms: float, event_size: int = None):
        """Specialized logging for event operations"""
        level = LogLevel.INFO if success else LogLevel.ERROR
        
        event_data = {
            "operation_type": operation,
            "event_type": event_type,
            "aggregate_id": aggregate_id,
            "success": success,
            "event_size_bytes": event_size
        }
        
        performance_data = {
            "duration_ms": duration_ms,
            "throughput_events_per_second": 1000 / duration_ms if duration_ms > 0 else 0
        }
        
        self._log(
            level,
            f"Event operation {operation} {('completed' if success else 'failed')} for {event_type}",
            correlation_context,
            event_data=event_data,
            performance_data=performance_data,
            tags=["event_operation", operation, event_type]
        )
    
    def log_business_operation(self, operation: str, correlation_context: CorrelationContext,
                              success: bool, duration_ms: float, business_data: Dict[str, Any] = None):
        """Specialized logging for business operations"""
        level = LogLevel.INFO if success else LogLevel.ERROR
        
        event_data = {
            "business_operation": operation,
            "success": success
        }
        
        if business_data:
            event_data.update(business_data)
        
        performance_data = {
            "duration_ms": duration_ms
        }
        
        self._log(
            level,
            f"Business operation {operation} {('completed' if success else 'failed')}",
            correlation_context,
            event_data=event_data,
            performance_data=performance_data,
            tags=["business_operation", operation]
        )
    
    def log_performance_metrics(self, correlation_context: CorrelationContext,
                               metrics: Dict[str, float]):
        """Log performance metrics"""
        self._log(
            LogLevel.INFO,
            "Performance metrics collected",
            correlation_context,
            performance_data=metrics,
            tags=["performance_metrics"]
        )
    
    def log_exception(self, correlation_context: CorrelationContext, 
                     exception: Exception, operation: str = None):
        """Log exception with full context"""
        error_data = {
            "exception_type": type(exception).__name__,
            "exception_message": str(exception),
            "traceback": traceback.format_exc(),
            "operation": operation
        }
        
        self._log(
            LogLevel.ERROR,
            f"Exception in {operation or 'unknown operation'}: {str(exception)}",
            correlation_context,
            error_data=error_data,
            tags=["exception", type(exception).__name__]
        )
    
    def get_stats(self) -> Dict[str, Any]:
        """Get logging statistics"""
        runtime = time.time() - self.start_time
        with self.lock:
            return {
                "total_logs": self.log_count,
                "logs_per_second": self.log_count / runtime if runtime > 0 else 0,
                "level_distribution": dict(self.stats),
                "buffer_size": len(self.log_buffer),
                "runtime_seconds": runtime
            }
    
    def get_recent_logs(self, count: int = 10) -> List[LogRecord]:
        """Get recent log records for analysis"""
        with self.lock:
            return list(self.log_buffer)[-count:]


# Global context storage for async operations
_correlation_context: threading.local = threading.local()


def set_correlation_context(context: CorrelationContext):
    """Set correlation context for current thread/task"""
    _correlation_context.value = context


def get_correlation_context() -> Optional[CorrelationContext]:
    """Get correlation context for current thread/task"""
    return getattr(_correlation_context, 'value', None)


@asynccontextmanager
async def correlation_context(context: CorrelationContext):
    """Async context manager for correlation context"""
    old_context = get_correlation_context()
    set_correlation_context(context)
    try:
        yield context
    finally:
        set_correlation_context(old_context)


class ObservabilityEventStore:
    """Event store with comprehensive structured logging"""
    
    def __init__(self, connection_string: str, logger: StructuredLogger):
        self.connection_string = connection_string
        self.logger = logger
        self.operation_count = 0
        
        # Create base correlation context for the event store
        self.base_context = CorrelationContext(
            correlation_id=str(uuid.uuid4()),
            operation="event_store_init"
        )
        
        self.logger.info("EventStore initialized", self.base_context,
                        event_data={"connection_string": connection_string},
                        tags=["initialization"])
    
    async def create_event_with_logging(self, aggregate_id: str, event_type: str,
                                       event_data: dict, correlation_context: CorrelationContext) -> dict:
        """Create an event with comprehensive logging"""
        operation_start = time.time()
        
        # Create child context for this operation
        child_context = CorrelationContext(
            correlation_id=correlation_context.correlation_id,
            trace_id=correlation_context.trace_id,
            span_id=str(uuid.uuid4()),
            parent_span_id=correlation_context.span_id,
            user_id=correlation_context.user_id,
            tenant_id=correlation_context.tenant_id,
            operation="create_event"
        )
        
        self.logger.debug(
            "Starting event creation",
            child_context,
            event_data={
                "aggregate_id": aggregate_id,
                "event_type": event_type,
                "event_size_bytes": len(json.dumps(event_data))
            },
            tags=["event_creation", "start"]
        )
        
        try:
            # Simulate event creation
            await asyncio.sleep(random.uniform(0.005, 0.02))
            
            # Create the event
            self.operation_count += 1
            event_id = f"event-{self.operation_count:06d}"
            
            duration_ms = (time.time() - operation_start) * 1000
            event_size = len(json.dumps(event_data))
            
            # Log successful event creation
            self.logger.log_event_operation(
                "create_event",
                child_context,
                event_type,
                aggregate_id,
                True,
                duration_ms,
                event_size
            )
            
            # Additional detailed logging
            self.logger.info(
                "Event created successfully",
                child_context,
                event_data={
                    "event_id": event_id,
                    "aggregate_id": aggregate_id,
                    "event_type": event_type,
                    "version": self.operation_count
                },
                performance_data={
                    "duration_ms": duration_ms,
                    "event_size_bytes": event_size,
                    "throughput_events_per_second": 1000 / duration_ms
                },
                tags=["event_creation", "success"]
            )
            
            return {
                "event_id": event_id,
                "aggregate_id": aggregate_id,
                "event_type": event_type,
                "correlation_id": child_context.correlation_id,
                "trace_id": child_context.trace_id,
                "created_at": datetime.now(timezone.utc).isoformat(),
                "processing_duration_ms": duration_ms
            }
            
        except Exception as e:
            duration_ms = (time.time() - operation_start) * 1000
            
            # Log the exception
            self.logger.log_exception(child_context, e, "create_event")
            
            # Log failed event operation
            self.logger.log_event_operation(
                "create_event",
                child_context,
                event_type,
                aggregate_id,
                False,
                duration_ms
            )
            
            raise
    
    async def load_events_with_logging(self, aggregate_id: str, 
                                     correlation_context: CorrelationContext) -> List[dict]:
        """Load events with comprehensive logging"""
        operation_start = time.time()
        
        child_context = CorrelationContext(
            correlation_id=correlation_context.correlation_id,
            trace_id=correlation_context.trace_id,
            span_id=str(uuid.uuid4()),
            parent_span_id=correlation_context.span_id,
            user_id=correlation_context.user_id,
            tenant_id=correlation_context.tenant_id,
            operation="load_events"
        )
        
        self.logger.debug(
            "Starting event loading",
            child_context,
            event_data={"aggregate_id": aggregate_id},
            tags=["event_loading", "start"]
        )
        
        try:
            # Simulate loading events
            event_count = random.randint(1, 20)
            await asyncio.sleep(event_count * 0.001)
            
            # Create simulated events
            events = []
            for i in range(event_count):
                events.append({
                    "event_id": f"event-{i:03d}",
                    "aggregate_id": aggregate_id,
                    "event_type": "SimulatedEvent",
                    "version": i + 1,
                    "correlation_id": child_context.correlation_id
                })
            
            duration_ms = (time.time() - operation_start) * 1000
            
            # Log successful loading
            self.logger.log_event_operation(
                "load_events",
                child_context,
                "SimulatedEvent",
                aggregate_id,
                True,
                duration_ms
            )
            
            self.logger.info(
                f"Loaded {event_count} events",
                child_context,
                event_data={
                    "aggregate_id": aggregate_id,
                    "event_count": event_count,
                    "events_per_ms": event_count / duration_ms
                },
                performance_data={
                    "duration_ms": duration_ms,
                    "events_loaded": event_count,
                    "loading_rate_events_per_second": event_count / (duration_ms / 1000)
                },
                tags=["event_loading", "success"]
            )
            
            return events
            
        except Exception as e:
            duration_ms = (time.time() - operation_start) * 1000
            
            self.logger.log_exception(child_context, e, "load_events")
            
            self.logger.log_event_operation(
                "load_events",
                child_context,
                "SimulatedEvent",
                aggregate_id,
                False,
                duration_ms
            )
            
            raise


class LogAggregator:
    """Log aggregation and analysis for centralized monitoring"""
    
    def __init__(self):
        self.logs: List[LogRecord] = []
        self.correlation_chains: Dict[str, List[LogRecord]] = defaultdict(list)
        self.performance_metrics: Dict[str, List[float]] = defaultdict(list)
        
    def add_logs_from_logger(self, logger: StructuredLogger):
        """Aggregate logs from a structured logger"""
        recent_logs = logger.get_recent_logs(1000)
        
        for log in recent_logs:
            self.logs.append(log)
            
            # Group by correlation ID
            self.correlation_chains[log.correlation_context.correlation_id].append(log)
            
            # Extract performance metrics
            if log.performance_data:
                for metric_name, value in log.performance_data.items():
                    if isinstance(value, (int, float)):
                        self.performance_metrics[metric_name].append(value)
    
    def analyze_correlation_chains(self) -> Dict[str, Any]:
        """Analyze correlation chains for distributed operations"""
        analysis = {
            "total_chains": len(self.correlation_chains),
            "chain_lengths": [],
            "average_chain_length": 0,
            "longest_chains": [],
            "error_chains": []
        }
        
        chain_lengths = []
        
        for correlation_id, chain in self.correlation_chains.items():
            length = len(chain)
            chain_lengths.append(length)
            
            # Check for errors in chain
            has_errors = any(log.level in [LogLevel.ERROR, LogLevel.FATAL] for log in chain)
            if has_errors:
                analysis["error_chains"].append({
                    "correlation_id": correlation_id,
                    "length": length,
                    "error_count": sum(1 for log in chain if log.level in [LogLevel.ERROR, LogLevel.FATAL])
                })
            
            # Track longest chains
            if length > 5:
                operations = [log.correlation_context.operation for log in chain if log.correlation_context.operation]
                analysis["longest_chains"].append({
                    "correlation_id": correlation_id,
                    "length": length,
                    "operations": operations
                })
        
        analysis["chain_lengths"] = chain_lengths
        analysis["average_chain_length"] = sum(chain_lengths) / len(chain_lengths) if chain_lengths else 0
        
        return analysis
    
    def analyze_performance_patterns(self) -> Dict[str, Any]:
        """Analyze performance patterns from aggregated logs"""
        patterns = {}
        
        for metric_name, values in self.performance_metrics.items():
            if values:
                patterns[metric_name] = {
                    "count": len(values),
                    "min": min(values),
                    "max": max(values),
                    "avg": sum(values) / len(values),
                    "percentile_50": sorted(values)[len(values) // 2] if values else 0,
                    "percentile_95": sorted(values)[int(len(values) * 0.95)] if values else 0
                }
        
        return patterns
    
    def get_error_summary(self) -> Dict[str, Any]:
        """Get summary of errors from aggregated logs"""
        errors = [log for log in self.logs if log.level in [LogLevel.ERROR, LogLevel.FATAL]]
        
        error_types = defaultdict(int)
        error_operations = defaultdict(int)
        
        for error_log in errors:
            if error_log.error_data:
                error_type = error_log.error_data.get("exception_type", "Unknown")
                error_types[error_type] += 1
                
                operation = error_log.error_data.get("operation", "unknown")
                error_operations[operation] += 1
        
        return {
            "total_errors": len(errors),
            "error_types": dict(error_types),
            "error_operations": dict(error_operations),
            "error_rate": len(errors) / len(self.logs) * 100 if self.logs else 0
        }


async def demonstrate_structured_logging():
    """Demonstrate structured logging with correlation IDs"""
    print("=" * 80)
    print("📝 STRUCTURED LOGGING DEMONSTRATION")
    print("=" * 80)
    print()
    
    # Initialize structured logger
    logger = StructuredLogger("eventuali.demo", LogLevel.DEBUG, enable_sampling=True, sample_rate=0.8)
    
    # Initialize event store with logging
    event_store = ObservabilityEventStore("sqlite://:memory:", logger)
    
    print("🎯 Simulating correlated operations with structured logging...")
    print("(Each operation generates structured JSON logs with correlation IDs)")
    print()
    
    # Create root correlation context for a business workflow
    root_context = CorrelationContext(
        correlation_id=str(uuid.uuid4()),
        user_id="user-12345",
        tenant_id="tenant-retail",
        operation="order_processing_workflow",
        session_id=str(uuid.uuid4()),
        request_id=str(uuid.uuid4())
    )
    
    async with correlation_context(root_context):
        workflow_start = time.time()
        
        logger.info(
            "Starting order processing workflow",
            root_context,
            event_data={
                "user_id": root_context.user_id,
                "tenant_id": root_context.tenant_id,
                "workflow_type": "order_processing"
            },
            tags=["workflow", "start", "order_processing"]
        )
        
        try:
            # Step 1: Create order events
            order_events = []
            for i, event_type in enumerate(["OrderCreated", "PaymentProcessed", "InventoryReserved"]):
                event = await event_store.create_event_with_logging(
                    f"order-{uuid.uuid4()}",
                    event_type,
                    {
                        "order_id": f"order-{i:03d}",
                        "amount": random.uniform(50.0, 500.0),
                        "customer_id": root_context.user_id
                    },
                    root_context
                )
                order_events.append(event)
                
                # Small delay between events
                await asyncio.sleep(0.01)
            
            # Step 2: Load and verify events
            for event in order_events:
                loaded_events = await event_store.load_events_with_logging(
                    event["aggregate_id"],
                    root_context
                )
                
                logger.debug(
                    "Verified event persistence",
                    root_context,
                    event_data={
                        "aggregate_id": event["aggregate_id"],
                        "loaded_count": len(loaded_events),
                        "original_event_id": event["event_id"]
                    },
                    tags=["verification", "persistence"]
                )
            
            workflow_duration = (time.time() - workflow_start) * 1000
            
            # Log successful workflow completion
            logger.log_business_operation(
                "order_processing_workflow",
                root_context,
                True,
                workflow_duration,
                {
                    "events_created": len(order_events),
                    "total_amount": sum(random.uniform(50.0, 500.0) for _ in order_events),
                    "customer_id": root_context.user_id
                }
            )
            
        except Exception as e:
            workflow_duration = (time.time() - workflow_start) * 1000
            
            logger.log_exception(root_context, e, "order_processing_workflow")
            
            logger.log_business_operation(
                "order_processing_workflow",
                root_context,
                False,
                workflow_duration
            )
    
    # Show logging statistics
    stats = logger.get_stats()
    print(f"📊 Logging Statistics:")
    print(f"   Total Logs: {stats['total_logs']}")
    print(f"   Logs/Second: {stats['logs_per_second']:.2f}")
    print(f"   Level Distribution: {stats['level_distribution']}")
    print(f"   Runtime: {stats['runtime_seconds']:.2f}s")
    print()
    
    return logger


async def demonstrate_log_aggregation():
    """Demonstrate log aggregation and correlation analysis"""
    print("=" * 80)
    print("📊 LOG AGGREGATION AND CORRELATION ANALYSIS")
    print("=" * 80)
    print()
    
    # Create multiple loggers for different services
    loggers = {
        "user_service": StructuredLogger("user_service", LogLevel.DEBUG),
        "order_service": StructuredLogger("order_service", LogLevel.DEBUG), 
        "payment_service": StructuredLogger("payment_service", LogLevel.DEBUG),
        "inventory_service": StructuredLogger("inventory_service", LogLevel.DEBUG)
    }
    
    event_stores = {
        name: ObservabilityEventStore(f"sqlite://:memory:{name}", logger)
        for name, logger in loggers.items()
    }
    
    print("🔄 Simulating multi-service operations with shared correlation...")
    
    # Simulate multiple business workflows
    correlation_ids = []
    
    for workflow_num in range(3):
        # Create shared correlation context
        shared_context = CorrelationContext(
            correlation_id=str(uuid.uuid4()),
            user_id=f"user-{workflow_num:03d}",
            tenant_id=f"tenant-{workflow_num % 2}",
            operation="multi_service_workflow",
            request_id=str(uuid.uuid4())
        )
        correlation_ids.append(shared_context.correlation_id)
        
        # User service: Create user event
        await event_stores["user_service"].create_event_with_logging(
            f"user-{workflow_num:03d}",
            "UserProfileUpdated",
            {"email": f"user{workflow_num}@example.com"},
            shared_context
        )
        
        # Order service: Create order
        await event_stores["order_service"].create_event_with_logging(
            f"order-{workflow_num:03d}",
            "OrderCreated",
            {"amount": random.uniform(100.0, 1000.0)},
            shared_context
        )
        
        # Payment service: Process payment
        payment_success = random.random() > 0.2  # 80% success rate
        try:
            if not payment_success:
                raise Exception("Payment processing failed")
                
            await event_stores["payment_service"].create_event_with_logging(
                f"payment-{workflow_num:03d}",
                "PaymentProcessed",
                {"status": "completed"},
                shared_context
            )
        except Exception as e:
            loggers["payment_service"].log_exception(shared_context, e, "process_payment")
        
        # Inventory service: Update inventory
        await event_stores["inventory_service"].create_event_with_logging(
            f"inventory-{workflow_num:03d}",
            "InventoryUpdated", 
            {"quantity_reserved": random.randint(1, 5)},
            shared_context
        )
        
        await asyncio.sleep(0.01)  # Small delay between workflows
    
    print(f"✅ Completed {len(correlation_ids)} multi-service workflows")
    print()
    
    # Aggregate logs from all services
    print("📋 Aggregating logs from all services...")
    aggregator = LogAggregator()
    
    for service_name, logger in loggers.items():
        aggregator.add_logs_from_logger(logger)
        print(f"   {service_name}: {logger.get_stats()['total_logs']} logs")
    
    print()
    
    # Analyze correlation chains
    print("🔗 Analyzing correlation chains...")
    chain_analysis = aggregator.analyze_correlation_chains()
    
    print(f"   Total Chains: {chain_analysis['total_chains']}")
    print(f"   Average Chain Length: {chain_analysis['average_chain_length']:.2f}")
    print(f"   Error Chains: {len(chain_analysis['error_chains'])}")
    
    if chain_analysis['error_chains']:
        print("   Error Chain Details:")
        for error_chain in chain_analysis['error_chains']:
            print(f"     Correlation ID: {error_chain['correlation_id'][:8]}...")
            print(f"     Chain Length: {error_chain['length']}")
            print(f"     Error Count: {error_chain['error_count']}")
    
    print()
    
    # Analyze performance patterns
    print("⚡ Analyzing performance patterns...")
    performance_analysis = aggregator.analyze_performance_patterns()
    
    for metric_name, stats in performance_analysis.items():
        if "duration" in metric_name.lower():
            print(f"   {metric_name}:")
            print(f"     Count: {stats['count']}")
            print(f"     Avg: {stats['avg']:.2f}ms")
            print(f"     95th percentile: {stats['percentile_95']:.2f}ms")
    
    print()
    
    # Error summary
    print("🚨 Error Analysis:")
    error_summary = aggregator.get_error_summary()
    print(f"   Total Errors: {error_summary['total_errors']}")
    print(f"   Error Rate: {error_summary['error_rate']:.2f}%")
    
    if error_summary['error_types']:
        print("   Error Types:")
        for error_type, count in error_summary['error_types'].items():
            print(f"     {error_type}: {count}")
    
    return aggregator, chain_analysis


async def demonstrate_log_patterns():
    """Demonstrate advanced logging patterns"""
    print("\n" + "=" * 80)
    print("🎛️  ADVANCED LOGGING PATTERNS DEMONSTRATION")
    print("=" * 80)
    print()
    
    logger = StructuredLogger("patterns.demo", LogLevel.TRACE, enable_sampling=True, sample_rate=1.0)
    
    print("✨ Advanced logging patterns demonstrated:")
    print()
    
    # Pattern 1: Performance-aware logging with sampling
    print("1. 📊 Performance-aware logging with sampling")
    base_context = CorrelationContext(
        correlation_id=str(uuid.uuid4()),
        operation="performance_logging_demo"
    )
    
    # High-frequency debug logs with sampling
    for i in range(100):
        logger.debug(
            f"High-frequency debug log {i}",
            base_context,
            event_data={"iteration": i, "batch": "performance_test"},
            tags=["debug", "high_frequency"]
        )
    
    print("   Generated 100 debug logs with sampling (actual logs may be fewer)")
    
    # Pattern 2: Context inheritance through async operations
    print("\n2. 🔗 Context inheritance through async operations")
    
    async def nested_operation_1(context: CorrelationContext):
        child_context = CorrelationContext(
            correlation_id=context.correlation_id,
            trace_id=context.trace_id,
            span_id=str(uuid.uuid4()),
            parent_span_id=context.span_id,
            operation="nested_op_1"
        )
        
        logger.info("Nested operation 1 started", child_context, tags=["nested", "operation_1"])
        await asyncio.sleep(0.01)
        
        await nested_operation_2(child_context)
        
        logger.info("Nested operation 1 completed", child_context, tags=["nested", "operation_1"])
    
    async def nested_operation_2(context: CorrelationContext):
        child_context = CorrelationContext(
            correlation_id=context.correlation_id,
            trace_id=context.trace_id,
            span_id=str(uuid.uuid4()),
            parent_span_id=context.span_id,
            operation="nested_op_2"
        )
        
        logger.info("Nested operation 2 started", child_context, tags=["nested", "operation_2"])
        await asyncio.sleep(0.005)
        logger.info("Nested operation 2 completed", child_context, tags=["nested", "operation_2"])
    
    await nested_operation_1(base_context)
    print("   Demonstrated context inheritance through nested async operations")
    
    # Pattern 3: Structured error logging with context
    print("\n3. 🚨 Structured error logging with full context")
    
    error_context = CorrelationContext(
        correlation_id=str(uuid.uuid4()),
        user_id="user-error-demo",
        operation="error_handling_demo"
    )
    
    try:
        # Simulate an error
        raise ValueError("Demonstration error with context")
    except Exception as e:
        logger.log_exception(error_context, e, "error_demo_operation")
    
    print("   Logged structured exception with full context and traceback")
    
    # Pattern 4: Business metrics logging
    print("\n4. 💼 Business metrics integration")
    
    business_context = CorrelationContext(
        correlation_id=str(uuid.uuid4()),
        user_id="business-user",
        tenant_id="tenant-enterprise",
        operation="business_metrics_demo"
    )
    
    # Log business operation with comprehensive metrics
    business_duration = random.uniform(50.0, 200.0)
    logger.log_business_operation(
        "customer_onboarding",
        business_context,
        True,
        business_duration,
        {
            "customer_tier": "enterprise",
            "onboarding_steps": 5,
            "documents_uploaded": 3,
            "verification_method": "manual"
        }
    )
    
    # Log performance metrics
    metrics = {
        "database_query_time_ms": random.uniform(5.0, 50.0),
        "external_api_time_ms": random.uniform(100.0, 500.0),
        "memory_usage_mb": random.uniform(100.0, 300.0),
        "cpu_usage_percent": random.uniform(20.0, 80.0)
    }
    logger.log_performance_metrics(business_context, metrics)
    
    print("   Logged business operation with performance and business metrics")
    
    # Show final statistics
    final_stats = logger.get_stats()
    print(f"\n📈 Final Pattern Demo Statistics:")
    print(f"   Total Logs Generated: {final_stats['total_logs']}")
    print(f"   Average Logs/Second: {final_stats['logs_per_second']:.2f}")
    print(f"   Level Distribution: {final_stats['level_distribution']}")
    
    return logger


async def main():
    """Main demonstration function"""
    print("📝 Eventuali Structured Logging Example")
    print("=" * 80)
    print()
    print("This example demonstrates comprehensive structured logging")
    print("for event sourcing applications with correlation ID tracking.")
    print()
    print("Key concepts demonstrated:")
    print("• Structured JSON logging with correlation IDs")
    print("• Context propagation through async operations")
    print("• Log aggregation and correlation analysis")
    print("• Performance-aware logging with sampling")
    print("• Business and technical metrics integration")
    print("• Multi-service correlation tracking")
    print()
    
    try:
        # Run all demonstrations
        basic_logger = await demonstrate_structured_logging()
        aggregator, chains = await demonstrate_log_aggregation()
        advanced_logger = await demonstrate_log_patterns()
        
        print("\n" + "=" * 80)
        print("🎉 STRUCTURED LOGGING DEMONSTRATION COMPLETED!")
        print("=" * 80)
        print()
        print("📋 LOGGING CAPABILITIES DEMONSTRATED:")
        print("• JSON-structured logs with consistent schema")
        print("• Correlation ID tracking across distributed operations")
        print("• Performance metrics integration")
        print("• Error tracking with full context and tracebacks")
        print("• Multi-service correlation analysis")
        print("• High-performance logging with sampling")
        print("• Business operation tracking")
        print("• Context inheritance through async operations")
        print()
        print("🔍 PRODUCTION DEPLOYMENT CONSIDERATIONS:")
        print("• Configure log shipping to centralized aggregation (ELK, Splunk)")
        print("• Set appropriate log levels for different environments")
        print("• Implement log rotation and retention policies")
        print("• Configure structured log parsing in your log aggregator")
        print("• Set up alerts based on correlation ID patterns")
        print("• Implement log sampling for high-throughput scenarios")
        print("• Consider log compression for network efficiency")
        print("• Monitor logging performance impact (<1% overhead target)")
        print()
        print("📊 AGGREGATION INSIGHTS:")
        print(f"• Total correlation chains analyzed: {chains['total_chains']}")
        print(f"• Average operations per chain: {chains['average_chain_length']:.2f}")
        print(f"• Error chains identified: {len(chains['error_chains'])}")
        print("• All logs structured with consistent correlation tracking")
        
    except Exception as e:
        print(f"❌ Error during demonstration: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())