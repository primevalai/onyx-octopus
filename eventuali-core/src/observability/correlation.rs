//! Correlation ID management for distributed tracing and request tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A unique identifier for correlating operations across distributed systems
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Create a new correlation ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create a correlation ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }

    /// Get the correlation ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to string
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CorrelationId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for CorrelationId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<Uuid> for CorrelationId {
    fn from(uuid: Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

/// Generate a new correlation ID
pub fn generate_correlation_id() -> CorrelationId {
    CorrelationId::from_uuid(Uuid::new_v4())
}

/// Context for tracking correlated operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationContext {
    pub correlation_id: CorrelationId,
    pub parent_id: Option<CorrelationId>,
    pub operation: String,
    pub service: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new(operation: impl Into<String>, service: impl Into<String>) -> Self {
        Self {
            correlation_id: generate_correlation_id(),
            parent_id: None,
            operation: operation.into(),
            service: service.into(),
            user_id: None,
            session_id: None,
            request_id: None,
            trace_id: None,
            span_id: None,
            attributes: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a child context with a new correlation ID but linked parent
    pub fn create_child(&self, operation: impl Into<String>) -> Self {
        Self {
            correlation_id: generate_correlation_id(),
            parent_id: Some(self.correlation_id.clone()),
            operation: operation.into(),
            service: self.service.clone(),
            user_id: self.user_id.clone(),
            session_id: self.session_id.clone(),
            request_id: self.request_id.clone(),
            trace_id: self.trace_id.clone(),
            span_id: None, // Child gets new span ID
            attributes: self.attributes.clone(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Set user ID for this context
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID for this context
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set request ID for this context
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set trace ID for this context
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set span ID for this context
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }

    /// Add an attribute to this context
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get an attribute from this context
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }
}

/// Tracker for managing correlation contexts across operations
#[derive(Debug)]
pub struct CorrelationTracker {
    contexts: Arc<RwLock<HashMap<CorrelationId, CorrelationContext>>>,
}

impl CorrelationTracker {
    /// Create a new correlation tracker
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new correlation ID
    pub fn register(&self, correlation_id: CorrelationId) {
        // This is a simple registration - we could extend with more metadata
        tracing::debug!(correlation_id = %correlation_id, "Registered correlation ID");
    }

    /// Store a correlation context
    pub async fn store_context(&self, context: CorrelationContext) {
        let correlation_id = context.correlation_id.clone();
        self.contexts.write().await.insert(correlation_id.clone(), context);
        
        tracing::debug!(
            correlation_id = %correlation_id,
            "Stored correlation context"
        );
    }

    /// Retrieve a correlation context
    pub async fn get_context(&self, correlation_id: &CorrelationId) -> Option<CorrelationContext> {
        self.contexts.read().await.get(correlation_id).cloned()
    }

    /// Remove a correlation context
    pub async fn remove_context(&self, correlation_id: &CorrelationId) -> Option<CorrelationContext> {
        let context = self.contexts.write().await.remove(correlation_id);
        
        if context.is_some() {
            tracing::debug!(
                correlation_id = %correlation_id,
                "Removed correlation context"
            );
        }
        
        context
    }

    /// Get all active correlation contexts
    pub async fn get_active_contexts(&self) -> Vec<CorrelationContext> {
        self.contexts.read().await.values().cloned().collect()
    }

    /// Clean up old contexts (should be called periodically)
    pub async fn cleanup_old_contexts(&self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;
        let mut contexts = self.contexts.write().await;
        
        let initial_count = contexts.len();
        contexts.retain(|_, context| context.created_at > cutoff);
        let final_count = contexts.len();
        
        if initial_count > final_count {
            tracing::info!(
                removed = initial_count - final_count,
                remaining = final_count,
                "Cleaned up old correlation contexts"
            );
        }
    }
}

impl Default for CorrelationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Request context that includes correlation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub correlation_context: CorrelationContext,
    pub request_path: Option<String>,
    pub http_method: Option<String>,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
    pub request_headers: HashMap<String, String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(operation: impl Into<String>, service: impl Into<String>) -> Self {
        Self {
            correlation_context: CorrelationContext::new(operation, service),
            request_path: None,
            http_method: None,
            user_agent: None,
            client_ip: None,
            request_headers: HashMap::new(),
            started_at: chrono::Utc::now(),
        }
    }

    /// Get the correlation ID for this request
    pub fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_context.correlation_id
    }

    /// Add request metadata
    pub fn with_request_metadata(
        mut self,
        path: Option<String>,
        method: Option<String>,
        user_agent: Option<String>,
        client_ip: Option<String>,
    ) -> Self {
        self.request_path = path;
        self.http_method = method;
        self.user_agent = user_agent;
        self.client_ip = client_ip;
        self
    }

    /// Add a request header
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.request_headers.insert(name.into(), value.into());
        self
    }

    /// Get request duration so far
    pub fn duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.started_at
    }
}

/// Correlation for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCorrelation {
    pub correlation_id: CorrelationId,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service_name: String,
    pub operation_name: String,
    pub baggage: HashMap<String, String>,
}

impl TraceCorrelation {
    /// Create a new trace correlation
    pub fn new(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        service_name: impl Into<String>,
        operation_name: impl Into<String>,
    ) -> Self {
        Self {
            correlation_id: generate_correlation_id(),
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            service_name: service_name.into(),
            operation_name: operation_name.into(),
            baggage: HashMap::new(),
        }
    }

    /// Create a child trace correlation
    pub fn create_child(
        &self,
        span_id: impl Into<String>,
        operation_name: impl Into<String>,
    ) -> Self {
        Self {
            correlation_id: generate_correlation_id(),
            trace_id: self.trace_id.clone(),
            span_id: span_id.into(),
            parent_span_id: Some(self.span_id.clone()),
            service_name: self.service_name.clone(),
            operation_name: operation_name.into(),
            baggage: self.baggage.clone(),
        }
    }

    /// Add baggage to this correlation
    pub fn with_baggage(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.baggage.insert(key.into(), value.into());
        self
    }

    /// Get baggage value
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_generation() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();
        
        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
        assert!(!id2.as_str().is_empty());
    }

    #[test]
    fn test_correlation_context_creation() {
        let context = CorrelationContext::new("test_operation", "test_service");
        
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.service, "test_service");
        assert!(context.parent_id.is_none());
        assert!(!context.correlation_id.as_str().is_empty());
    }

    #[test]
    fn test_child_context_creation() {
        let parent = CorrelationContext::new("parent_op", "test_service");
        let child = parent.create_child("child_op");
        
        assert_eq!(child.operation, "child_op");
        assert_eq!(child.service, "test_service");
        assert_eq!(child.parent_id, Some(parent.correlation_id));
        assert_ne!(child.correlation_id, parent.correlation_id);
    }

    #[tokio::test]
    async fn test_correlation_tracker() {
        let tracker = CorrelationTracker::new();
        let context = CorrelationContext::new("test_op", "test_service");
        let correlation_id = context.correlation_id.clone();
        
        tracker.store_context(context.clone()).await;
        
        let retrieved = tracker.get_context(&correlation_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().operation, "test_op");
        
        let removed = tracker.remove_context(&correlation_id).await;
        assert!(removed.is_some());
        
        let not_found = tracker.get_context(&correlation_id).await;
        assert!(not_found.is_none());
    }

    #[test]
    fn test_request_context() {
        let request_ctx = RequestContext::new("http_request", "web_service")
            .with_request_metadata(
                Some("/api/events".to_string()),
                Some("POST".to_string()),
                Some("test-agent/1.0".to_string()),
                Some("192.168.1.1".to_string()),
            )
            .with_header("Authorization", "Bearer token123");

        assert_eq!(request_ctx.request_path, Some("/api/events".to_string()));
        assert_eq!(request_ctx.http_method, Some("POST".to_string()));
        assert_eq!(request_ctx.request_headers.get("Authorization"), Some(&"Bearer token123".to_string()));
    }

    #[test]
    fn test_trace_correlation() {
        let trace = TraceCorrelation::new("trace123", "span456", "eventuali", "create_event")
            .with_baggage("user_id", "user789");

        assert_eq!(trace.trace_id, "trace123");
        assert_eq!(trace.span_id, "span456");
        assert_eq!(trace.service_name, "eventuali");
        assert_eq!(trace.operation_name, "create_event");
        assert_eq!(trace.get_baggage("user_id"), Some(&"user789".to_string()));

        let child = trace.create_child("child_span", "save_event");
        assert_eq!(child.trace_id, trace.trace_id);
        assert_eq!(child.parent_span_id, Some(trace.span_id));
        assert_eq!(child.get_baggage("user_id"), Some(&"user789".to_string()));
    }
}