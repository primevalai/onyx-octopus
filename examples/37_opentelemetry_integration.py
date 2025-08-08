#!/usr/bin/env python3
"""
Example 37: OpenTelemetry Integration with Distributed Tracing and Correlation IDs

This example demonstrates:
1. Basic distributed tracing concepts with correlation IDs
2. Request correlation across operations
3. Structured logging with trace context
4. Performance monitoring integration
5. Cross-service correlation tracking

The implementation focuses on the foundation for OpenTelemetry integration,
showing how correlation IDs flow through operations and enable distributed tracing.
"""

import asyncio
import uuid
import time
import json
from datetime import datetime, timezone
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict

import eventuali


@dataclass
class TraceContext:
    """Represents a distributed tracing context"""
    trace_id: str
    span_id: str
    operation: str
    correlation_id: str
    parent_span_id: Optional[str] = None
    start_time: Optional[datetime] = None
    attributes: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.start_time is None:
            self.start_time = datetime.now(timezone.utc)
        if self.attributes is None:
            self.attributes = {}


@dataclass
class SpanEvent:
    """Represents an event within a span"""
    name: str
    timestamp: datetime
    attributes: Dict[str, Any]


class DistributedTracer:
    """Simplified distributed tracer for demonstration purposes"""
    
    def __init__(self, service_name: str):
        self.service_name = service_name
        self.active_spans: Dict[str, TraceContext] = {}
        self.span_events: Dict[str, List[SpanEvent]] = {}
        
    def start_span(self, operation: str, parent_context: Optional[TraceContext] = None) -> TraceContext:
        """Start a new span with optional parent context"""
        trace_id = parent_context.trace_id if parent_context else str(uuid.uuid4())
        span_id = str(uuid.uuid4())
        correlation_id = str(uuid.uuid4())
        
        context = TraceContext(
            trace_id=trace_id,
            span_id=span_id,
            operation=operation,
            correlation_id=correlation_id,
            parent_span_id=parent_context.span_id if parent_context else None,
            attributes={
                'service.name': self.service_name,
                'operation.type': 'child' if parent_context else 'root'
            }
        )
        
        self.active_spans[span_id] = context
        self.span_events[span_id] = []
        
        print(f"🎯 Started span: {operation}")
        print(f"   Trace ID: {trace_id}")
        print(f"   Span ID: {span_id}")
        print(f"   Correlation ID: {correlation_id}")
        if parent_context:
            print(f"   Parent Span ID: {parent_context.span_id}")
        print()
        
        return context
    
    def add_event(self, context: TraceContext, event_name: str, attributes: Dict[str, Any] = None):
        """Add an event to a span"""
        if attributes is None:
            attributes = {}
            
        event = SpanEvent(
            name=event_name,
            timestamp=datetime.now(timezone.utc),
            attributes=attributes
        )
        
        if context.span_id in self.span_events:
            self.span_events[context.span_id].append(event)
            
        print(f"📝 Event in span {context.operation}: {event_name}")
        if attributes:
            print(f"   Attributes: {attributes}")
    
    def add_attribute(self, context: TraceContext, key: str, value: Any):
        """Add an attribute to a span"""
        context.attributes[key] = value
        print(f"🏷️  Added attribute to {context.operation}: {key}={value}")
    
    def end_span(self, context: TraceContext, status: str = "ok"):
        """End a span and record its duration"""
        if context.span_id in self.active_spans:
            duration = datetime.now(timezone.utc) - context.start_time
            context.attributes['duration_ms'] = duration.total_seconds() * 1000
            context.attributes['status'] = status
            
            print(f"✅ Ended span: {context.operation}")
            print(f"   Duration: {duration.total_seconds() * 1000:.2f}ms")
            print(f"   Status: {status}")
            print()
            
            del self.active_spans[context.span_id]
    
    def get_trace_summary(self, trace_id: str) -> Dict[str, Any]:
        """Get a summary of all spans in a trace"""
        trace_spans = []
        for span_id, events in self.span_events.items():
            # Find the context (might be ended)
            context_data = {
                'span_id': span_id,
                'events': [asdict(event) for event in events]
            }
            trace_spans.append(context_data)
            
        return {
            'trace_id': trace_id,
            'service_name': self.service_name,
            'spans': trace_spans,
            'timestamp': datetime.now(timezone.utc).isoformat()
        }


class ObservabilityEventStore:
    """Event store wrapper with distributed tracing capabilities"""
    
    def __init__(self, connection_string: str, tracer: DistributedTracer):
        # For this demo, we'll simulate the event store since we need the actual connection setup
        self.connection_string = connection_string
        self.tracer = tracer
        print(f"🔗 Simulated EventStore connection to: {connection_string}")
    
    async def create_event_with_tracing(self, aggregate_id: str, event_type: str, 
                                      event_data: dict, trace_context: Optional[TraceContext] = None):
        """Create an event with full distributed tracing"""
        
        # Start a new span for this operation
        span = self.tracer.start_span("create_event", trace_context)
        
        try:
            # Add operation attributes
            self.tracer.add_attribute(span, "aggregate.id", aggregate_id)
            self.tracer.add_attribute(span, "event.type", event_type)
            self.tracer.add_attribute(span, "event.size_bytes", len(json.dumps(event_data)))
            
            # Record the operation start
            self.tracer.add_event(span, "event.creation.started", {
                "aggregate_id": aggregate_id,
                "event_type": event_type
            })
            
            # Simulate the actual event creation
            start_time = time.time()
            
            # Create the event (this would be the actual Eventuali call)
            # For this demo, we'll simulate it
            await asyncio.sleep(0.01)  # Simulate network/database delay
            
            operation_duration = time.time() - start_time
            
            # Record success event
            self.tracer.add_event(span, "event.creation.completed", {
                "duration_ms": operation_duration * 1000,
                "success": True
            })
            
            # Add performance metrics
            self.tracer.add_attribute(span, "operation.duration_ms", operation_duration * 1000)
            
            # End the span successfully
            self.tracer.end_span(span, "ok")
            
            return {
                "event_id": str(uuid.uuid4()),
                "aggregate_id": aggregate_id,
                "event_type": event_type,
                "trace_context": asdict(span),
                "created_at": datetime.now(timezone.utc).isoformat()
            }
            
        except Exception as e:
            # Record error event
            self.tracer.add_event(span, "event.creation.error", {
                "error": str(e),
                "error_type": type(e).__name__
            })
            
            # End span with error status
            self.tracer.end_span(span, "error")
            raise
    
    async def load_events_with_tracing(self, aggregate_id: str, 
                                     trace_context: Optional[TraceContext] = None) -> List[dict]:
        """Load events with distributed tracing"""
        
        span = self.tracer.start_span("load_events", trace_context)
        
        try:
            self.tracer.add_attribute(span, "aggregate.id", aggregate_id)
            
            # Record the operation start
            self.tracer.add_event(span, "events.loading.started", {
                "aggregate_id": aggregate_id
            })
            
            # Simulate loading events
            start_time = time.time()
            await asyncio.sleep(0.005)  # Simulate database query
            
            # Simulate some events
            events = [
                {
                    "event_id": str(uuid.uuid4()),
                    "aggregate_id": aggregate_id,
                    "event_type": "UserCreated",
                    "correlation_id": span.correlation_id,
                    "timestamp": datetime.now(timezone.utc).isoformat()
                },
                {
                    "event_id": str(uuid.uuid4()),
                    "aggregate_id": aggregate_id,
                    "event_type": "UserProfileUpdated",
                    "correlation_id": span.correlation_id,
                    "timestamp": datetime.now(timezone.utc).isoformat()
                }
            ]
            
            operation_duration = time.time() - start_time
            
            # Record success
            self.tracer.add_event(span, "events.loading.completed", {
                "duration_ms": operation_duration * 1000,
                "event_count": len(events)
            })
            
            self.tracer.add_attribute(span, "events.count", len(events))
            self.tracer.add_attribute(span, "operation.duration_ms", operation_duration * 1000)
            
            self.tracer.end_span(span, "ok")
            
            return events
            
        except Exception as e:
            self.tracer.add_event(span, "events.loading.error", {
                "error": str(e),
                "error_type": type(e).__name__
            })
            
            self.tracer.end_span(span, "error")
            raise


async def demonstrate_basic_tracing():
    """Demonstrate basic distributed tracing concepts"""
    print("=" * 80)
    print("🎯 BASIC DISTRIBUTED TRACING DEMONSTRATION")
    print("=" * 80)
    print()
    
    # Initialize the tracer
    tracer = DistributedTracer("eventuali-demo")
    
    # Create a root span for the entire operation
    root_span = tracer.start_span("user_registration_flow")
    
    # Add some context to the root span
    tracer.add_attribute(root_span, "user.id", "user-12345")
    tracer.add_attribute(root_span, "request.method", "POST")
    tracer.add_attribute(root_span, "request.path", "/api/users")
    
    try:
        # Step 1: Validate user data (child span)
        validation_span = tracer.start_span("validate_user_data", root_span)
        tracer.add_event(validation_span, "validation.started")
        
        await asyncio.sleep(0.002)  # Simulate validation
        
        tracer.add_event(validation_span, "validation.completed", {
            "validation_rules_applied": 5,
            "validation_passed": True
        })
        tracer.end_span(validation_span, "ok")
        
        # Step 2: Create user events (child span)
        event_creation_span = tracer.start_span("create_user_events", root_span)
        
        # Create multiple events within this span
        for i, event_type in enumerate(["UserCreated", "ProfileInitialized", "WelcomeEmailQueued"]):
            tracer.add_event(event_creation_span, f"event.created", {
                "event_type": event_type,
                "event_sequence": i + 1
            })
            await asyncio.sleep(0.001)  # Simulate event processing
        
        tracer.add_attribute(event_creation_span, "events.created_count", 3)
        tracer.end_span(event_creation_span, "ok")
        
        # Step 3: Send notifications (child span)
        notification_span = tracer.start_span("send_notifications", root_span)
        
        tracer.add_event(notification_span, "notification.email.sent", {
            "recipient": "user@example.com",
            "template": "welcome"
        })
        
        tracer.add_event(notification_span, "notification.sms.sent", {
            "recipient": "+1234567890",
            "message_type": "welcome"
        })
        
        tracer.end_span(notification_span, "ok")
        
        # Add final attributes to root span
        tracer.add_attribute(root_span, "operation.success", True)
        tracer.add_attribute(root_span, "user.registration.completed", True)
        
        tracer.end_span(root_span, "ok")
        
        print("✅ User registration flow completed successfully!")
        
    except Exception as e:
        tracer.add_event(root_span, "operation.error", {
            "error": str(e),
            "error_type": type(e).__name__
        })
        tracer.end_span(root_span, "error")
        raise
    
    # Print trace summary
    print("\n" + "=" * 60)
    print("📊 TRACE SUMMARY")
    print("=" * 60)
    trace_summary = tracer.get_trace_summary(root_span.trace_id)
    print(json.dumps(trace_summary, indent=2, default=str))


async def demonstrate_cross_service_correlation():
    """Demonstrate correlation IDs across multiple services"""
    print("\n" + "=" * 80)
    print("🔗 CROSS-SERVICE CORRELATION DEMONSTRATION")
    print("=" * 80)
    print()
    
    # Simulate multiple services
    user_service_tracer = DistributedTracer("user-service")
    email_service_tracer = DistributedTracer("email-service")
    audit_service_tracer = DistributedTracer("audit-service")
    
    # Start the initial request in user service
    user_span = user_service_tracer.start_span("process_user_update")
    user_service_tracer.add_attribute(user_span, "user.id", "user-67890")
    user_service_tracer.add_attribute(user_span, "service.version", "1.2.3")
    
    try:
        # User service processes the update
        user_service_tracer.add_event(user_span, "user.data.validated")
        user_service_tracer.add_event(user_span, "user.data.updated", {
            "fields_changed": ["email", "profile_picture"]
        })
        
        # Cross-service call to email service (propagate trace context)
        email_span = email_service_tracer.start_span("send_update_notification", user_span)
        email_service_tracer.add_attribute(email_span, "email.template", "profile_updated")
        email_service_tracer.add_attribute(email_span, "email.recipient", "user@example.com")
        
        email_service_tracer.add_event(email_span, "email.template.loaded")
        email_service_tracer.add_event(email_span, "email.sent", {
            "message_id": str(uuid.uuid4()),
            "delivery_status": "sent"
        })
        
        email_service_tracer.end_span(email_span, "ok")
        
        # Another cross-service call to audit service
        audit_span = audit_service_tracer.start_span("log_user_activity", user_span)
        audit_service_tracer.add_attribute(audit_span, "activity.type", "profile_update")
        audit_service_tracer.add_attribute(audit_span, "audit.category", "user_data_change")
        
        audit_service_tracer.add_event(audit_span, "audit.entry.created", {
            "audit_id": str(uuid.uuid4()),
            "change_summary": "User updated email and profile picture"
        })
        
        audit_service_tracer.end_span(audit_span, "ok")
        
        # Complete the user service span
        user_service_tracer.add_attribute(user_span, "cross_service.calls_made", 2)
        user_service_tracer.add_event(user_span, "user.update.completed")
        user_service_tracer.end_span(user_span, "ok")
        
        print("✅ Cross-service operation completed successfully!")
        print(f"🔗 All operations correlated under trace ID: {user_span.trace_id}")
        
    except Exception as e:
        user_service_tracer.end_span(user_span, "error")
        raise


async def demonstrate_observability_event_store():
    """Demonstrate event store operations with full observability"""
    print("\n" + "=" * 80)
    print("📚 EVENT STORE WITH OBSERVABILITY DEMONSTRATION")
    print("=" * 80)
    print()
    
    tracer = DistributedTracer("eventuali-event-store")
    
    # Initialize the observability-enhanced event store
    # Using in-memory SQLite for this demo
    obs_store = ObservabilityEventStore("sqlite://:memory:", tracer)
    
    # Start a business operation span
    business_span = tracer.start_span("order_processing_workflow")
    tracer.add_attribute(business_span, "order.id", "order-98765")
    tracer.add_attribute(business_span, "customer.id", "customer-45678")
    
    try:
        # Create multiple events as part of this business operation
        events_created = []
        
        # Event 1: Order Created
        event1 = await obs_store.create_event_with_tracing(
            "order-98765",
            "OrderCreated",
            {
                "customer_id": "customer-45678",
                "total_amount": 129.99,
                "items": [
                    {"product_id": "prod-1", "quantity": 2, "price": 49.99},
                    {"product_id": "prod-2", "quantity": 1, "price": 29.99}
                ]
            },
            business_span
        )
        events_created.append(event1)
        
        # Event 2: Payment Processed
        event2 = await obs_store.create_event_with_tracing(
            "order-98765",
            "PaymentProcessed",
            {
                "payment_method": "credit_card",
                "amount": 129.99,
                "transaction_id": "txn-" + str(uuid.uuid4()),
                "status": "successful"
            },
            business_span
        )
        events_created.append(event2)
        
        # Event 3: Order Shipped
        event3 = await obs_store.create_event_with_tracing(
            "order-98765",
            "OrderShipped",
            {
                "tracking_number": "TRK" + str(uuid.uuid4())[:8].upper(),
                "carrier": "FastShip Express",
                "estimated_delivery": "2024-01-15"
            },
            business_span
        )
        events_created.append(event3)
        
        # Now load all events for this aggregate with tracing
        loaded_events = await obs_store.load_events_with_tracing("order-98765", business_span)
        
        # Add summary information to the business span
        tracer.add_attribute(business_span, "events.created_count", len(events_created))
        tracer.add_attribute(business_span, "events.loaded_count", len(loaded_events))
        tracer.add_attribute(business_span, "order.status", "shipped")
        
        tracer.add_event(business_span, "order.workflow.completed", {
            "order_id": "order-98765",
            "final_status": "shipped",
            "processing_steps": 3
        })
        
        tracer.end_span(business_span, "ok")
        
        print("✅ Order processing workflow completed!")
        print(f"📦 Created {len(events_created)} events")
        print(f"📖 Loaded {len(loaded_events)} events")
        print()
        
        # Show the events with their correlation information
        print("📋 EVENTS WITH CORRELATION INFO:")
        print("-" * 50)
        for event in events_created:
            print(f"Event: {event['event_type']}")
            print(f"  Event ID: {event['event_id']}")
            print(f"  Correlation ID: {event['trace_context']['correlation_id']}")
            print(f"  Trace ID: {event['trace_context']['trace_id']}")
            print()
        
    except Exception as e:
        tracer.add_event(business_span, "order.workflow.error", {
            "error": str(e),
            "error_type": type(e).__name__
        })
        tracer.end_span(business_span, "error")
        raise


async def demonstrate_performance_monitoring():
    """Demonstrate performance monitoring with tracing"""
    print("\n" + "=" * 80)
    print("⚡ PERFORMANCE MONITORING DEMONSTRATION")
    print("=" * 80)
    print()
    
    tracer = DistributedTracer("performance-monitor")
    
    # Simulate a performance-sensitive operation
    perf_span = tracer.start_span("bulk_event_processing")
    tracer.add_attribute(perf_span, "operation.type", "bulk_processing")
    
    try:
        batch_size = 100
        processing_times = []
        
        tracer.add_event(perf_span, "bulk.processing.started", {
            "batch_size": batch_size
        })
        
        for batch_num in range(3):  # Process 3 batches
            batch_span = tracer.start_span(f"process_batch_{batch_num + 1}", perf_span)
            tracer.add_attribute(batch_span, "batch.number", batch_num + 1)
            tracer.add_attribute(batch_span, "batch.size", batch_size)
            
            start_time = time.time()
            
            # Simulate batch processing with varying performance
            base_delay = 0.01
            additional_delay = batch_num * 0.005  # Each batch gets slower
            await asyncio.sleep(base_delay + additional_delay)
            
            processing_time = time.time() - start_time
            processing_times.append(processing_time)
            
            # Record performance metrics
            tracer.add_attribute(batch_span, "processing.time_ms", processing_time * 1000)
            tracer.add_attribute(batch_span, "throughput.events_per_second", batch_size / processing_time)
            
            tracer.add_event(batch_span, "batch.completed", {
                "processing_time_ms": processing_time * 1000,
                "throughput_eps": batch_size / processing_time
            })
            
            tracer.end_span(batch_span, "ok")
        
        # Calculate overall performance metrics
        total_events = batch_size * 3
        total_time = sum(processing_times)
        average_time = total_time / len(processing_times)
        overall_throughput = total_events / total_time
        
        tracer.add_attribute(perf_span, "total.events_processed", total_events)
        tracer.add_attribute(perf_span, "total.processing_time_ms", total_time * 1000)
        tracer.add_attribute(perf_span, "average.batch_time_ms", average_time * 1000)
        tracer.add_attribute(perf_span, "overall.throughput_eps", overall_throughput)
        
        tracer.add_event(perf_span, "performance.analysis", {
            "total_events": total_events,
            "total_time_ms": total_time * 1000,
            "overall_throughput_eps": overall_throughput,
            "performance_degradation": processing_times[-1] > processing_times[0]
        })
        
        tracer.end_span(perf_span, "ok")
        
        print("📊 PERFORMANCE ANALYSIS RESULTS:")
        print("-" * 40)
        print(f"Total Events Processed: {total_events}")
        print(f"Total Processing Time: {total_time * 1000:.2f}ms")
        print(f"Average Batch Time: {average_time * 1000:.2f}ms")
        print(f"Overall Throughput: {overall_throughput:.2f} events/second")
        
        if processing_times[-1] > processing_times[0]:
            print("⚠️  Performance degradation detected!")
            print(f"   First batch: {processing_times[0] * 1000:.2f}ms")
            print(f"   Last batch: {processing_times[-1] * 1000:.2f}ms")
        else:
            print("✅ Stable performance maintained")
        
    except Exception as e:
        tracer.end_span(perf_span, "error")
        raise


async def main():
    """Main demonstration function"""
    print("🎯 Eventuali OpenTelemetry Integration Example")
    print("=" * 80)
    print()
    print("This example demonstrates distributed tracing concepts and correlation ID")
    print("management that form the foundation for OpenTelemetry integration.")
    print()
    print("Key concepts demonstrated:")
    print("• Distributed tracing with parent-child span relationships")
    print("• Correlation ID propagation across service boundaries")
    print("• Event attribution and performance monitoring")
    print("• Structured logging with trace context")
    print("• Cross-service operation correlation")
    print()
    
    try:
        # Run all demonstrations
        await demonstrate_basic_tracing()
        await demonstrate_cross_service_correlation()
        await demonstrate_observability_event_store()
        await demonstrate_performance_monitoring()
        
        print("\n" + "=" * 80)
        print("🎉 ALL OBSERVABILITY DEMONSTRATIONS COMPLETED SUCCESSFULLY!")
        print("=" * 80)
        print()
        print("Key takeaways:")
        print("• Correlation IDs enable end-to-end request tracking")
        print("• Span hierarchies provide detailed operation breakdown")
        print("• Performance metrics help identify bottlenecks")
        print("• Cross-service tracing enables system-wide visibility")
        print("• Event-driven architectures benefit greatly from distributed tracing")
        print()
        print("Next steps for full OpenTelemetry integration:")
        print("• Replace custom tracer with OpenTelemetry SDK")
        print("• Configure OTLP exporters for Jaeger/Zipkin")
        print("• Add automatic instrumentation for HTTP/database calls")
        print("• Implement trace sampling and rate limiting")
        print("• Set up centralized trace collection and analysis")
        
    except Exception as e:
        print(f"❌ Error during demonstration: {e}")
        raise


if __name__ == "__main__":
    asyncio.run(main())