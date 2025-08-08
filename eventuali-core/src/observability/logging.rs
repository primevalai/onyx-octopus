//! Structured logging module with correlation ID support
//!
//! Provides comprehensive logging capabilities with structured output and correlation tracking.

use crate::error::{EventualiError, Result};
use crate::observability::{
    correlation::{CorrelationId, CorrelationContext, TraceCorrelation},
    telemetry::TraceContext,
    ObservabilityConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::FmtSpan, MakeWriter},
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
    Layer, Registry,
};

/// Log levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl From<&tracing::Level> for LogLevel {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        }
    }
}

/// Context for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub correlation_id: Option<CorrelationId>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub service_name: String,
    pub operation: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl LogContext {
    /// Create a new log context
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            correlation_id: None,
            trace_id: None,
            span_id: None,
            user_id: None,
            session_id: None,
            request_id: None,
            service_name: service_name.into(),
            operation: None,
            attributes: HashMap::new(),
        }
    }

    /// Create log context from correlation context
    pub fn from_correlation_context(context: &CorrelationContext) -> Self {
        Self {
            correlation_id: Some(context.correlation_id.clone()),
            trace_id: context.trace_id.clone(),
            span_id: context.span_id.clone(),
            user_id: context.user_id.clone(),
            session_id: context.session_id.clone(),
            request_id: context.request_id.clone(),
            service_name: context.service.clone(),
            operation: Some(context.operation.clone()),
            attributes: context.attributes.clone(),
        }
    }

    /// Create log context from trace context
    pub fn from_trace_context(trace_context: &TraceContext, service_name: impl Into<String>) -> Self {
        Self {
            correlation_id: Some(trace_context.correlation_id.clone()),
            trace_id: None, // Could extract from span
            span_id: None,  // Could extract from span
            user_id: None,
            session_id: None,
            request_id: None,
            service_name: service_name.into(),
            operation: Some(trace_context.operation.clone()),
            attributes: trace_context.attributes.clone(),
        }
    }

    /// Add an attribute to the context
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set operation
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub context: LogContext,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub target: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>, context: LogContext) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message: message.into(),
            context,
            module: None,
            file: None,
            line: None,
            target: None,
            fields: HashMap::new(),
        }
    }

    /// Add metadata to the log entry
    pub fn with_metadata(
        mut self,
        module: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
        target: Option<&str>,
    ) -> Self {
        self.module = module.map(|s| s.to_string());
        self.file = file.map(|s| s.to_string());
        self.line = line;
        self.target = target.map(|s| s.to_string());
        self
    }

    /// Add a field to the log entry
    pub fn with_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| EventualiError::ObservabilityError(format!("Failed to serialize log entry: {}", e)))
    }

    /// Convert to pretty JSON string
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| EventualiError::ObservabilityError(format!("Failed to serialize log entry: {}", e)))
    }
}

/// Main structured logger
#[derive(Debug)]
pub struct StructuredLogger {
    config: ObservabilityConfig,
    entries: Arc<RwLock<Vec<LogEntry>>>,
    correlation_logger: CorrelationLogger,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(config: &ObservabilityConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            entries: Arc::new(RwLock::new(Vec::new())),
            correlation_logger: CorrelationLogger::new(config.service_name.clone()),
        })
    }

    /// Initialize the structured logger
    pub async fn initialize(&self) -> Result<()> {
        // Set up tracing subscriber if structured logging is enabled
        if self.config.structured_logging {
            let subscriber = Registry::default()
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_span_events(FmtSpan::CLOSE)
                )
                .with(ObservabilityLayer::new(self.entries.clone()));

            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| EventualiError::ObservabilityError(format!("Failed to set tracing subscriber: {}", e)))?;
        }

        tracing::info!(
            structured_logging = self.config.structured_logging,
            service_name = %self.config.service_name,
            "Structured logger initialized"
        );

        Ok(())
    }

    /// Log a message with full context
    pub fn log_with_context(&self, level: LogLevel, message: &str, trace_context: &TraceContext) {
        let context = LogContext::from_trace_context(trace_context, &self.config.service_name);
        let entry = LogEntry::new(level, message, context);
        
        // Store the entry
        if let Ok(mut entries) = self.entries.try_write() {
            entries.push(entry.clone());
        }

        // Also log through tracing
        match level {
            LogLevel::Error => tracing::error!(
                correlation_id = %trace_context.correlation_id,
                operation = %trace_context.operation,
                message = message
            ),
            LogLevel::Warn => tracing::warn!(
                correlation_id = %trace_context.correlation_id,
                operation = %trace_context.operation,
                message = message
            ),
            LogLevel::Info => tracing::info!(
                correlation_id = %trace_context.correlation_id,
                operation = %trace_context.operation,
                message = message
            ),
            LogLevel::Debug => tracing::debug!(
                correlation_id = %trace_context.correlation_id,
                operation = %trace_context.operation,
                message = message
            ),
            LogLevel::Trace => tracing::trace!(
                correlation_id = %trace_context.correlation_id,
                operation = %trace_context.operation,
                message = message
            ),
        }
    }

    /// Log with correlation context
    pub fn log_with_correlation(&self, level: LogLevel, message: &str, context: &CorrelationContext) {
        let log_context = LogContext::from_correlation_context(context);
        let entry = LogEntry::new(level, message, log_context);
        
        if let Ok(mut entries) = self.entries.try_write() {
            entries.push(entry);
        }

        self.correlation_logger.log(level, message, Some(context));
    }

    /// Log a simple message
    pub fn log(&self, level: LogLevel, message: &str) {
        let context = LogContext::new(&self.config.service_name);
        let entry = LogEntry::new(level, message, context);
        
        if let Ok(mut entries) = self.entries.try_write() {
            entries.push(entry);
        }

        self.correlation_logger.log(level, message, None);
    }

    /// Get recent log entries
    pub async fn get_recent_entries(&self, limit: usize) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        let start = if entries.len() > limit { entries.len() - limit } else { 0 };
        entries[start..].to_vec()
    }

    /// Clear stored log entries
    pub async fn clear_entries(&self) {
        self.entries.write().await.clear();
    }

    /// Get all log entries
    pub async fn get_all_entries(&self) -> Vec<LogEntry> {
        self.entries.read().await.clone()
    }

    /// Export logs to JSON file
    pub async fn export_logs(&self, file_path: &str) -> Result<()> {
        let entries = self.get_all_entries().await;
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| EventualiError::ObservabilityError(format!("Failed to serialize logs: {}", e)))?;

        tokio::fs::write(file_path, json).await
            .map_err(|e| EventualiError::ObservabilityError(format!("Failed to write log file: {}", e)))?;

        Ok(())
    }

    /// Shutdown the logger
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Structured logger shut down successfully");
        Ok(())
    }
}

/// Logger specifically for correlation tracking
#[derive(Debug)]
pub struct CorrelationLogger {
    service_name: String,
}

impl CorrelationLogger {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Log with optional correlation context
    pub fn log(&self, level: LogLevel, message: &str, context: Option<&CorrelationContext>) {
        if let Some(ctx) = context {
            match level {
                LogLevel::Error => tracing::error!(
                    correlation_id = %ctx.correlation_id,
                    operation = %ctx.operation,
                    service = %ctx.service,
                    user_id = ?ctx.user_id,
                    session_id = ?ctx.session_id,
                    message = message
                ),
                LogLevel::Warn => tracing::warn!(
                    correlation_id = %ctx.correlation_id,
                    operation = %ctx.operation,
                    service = %ctx.service,
                    user_id = ?ctx.user_id,
                    session_id = ?ctx.session_id,
                    message = message
                ),
                LogLevel::Info => tracing::info!(
                    correlation_id = %ctx.correlation_id,
                    operation = %ctx.operation,
                    service = %ctx.service,
                    user_id = ?ctx.user_id,
                    session_id = ?ctx.session_id,
                    message = message
                ),
                LogLevel::Debug => tracing::debug!(
                    correlation_id = %ctx.correlation_id,
                    operation = %ctx.operation,
                    service = %ctx.service,
                    user_id = ?ctx.user_id,
                    session_id = ?ctx.session_id,
                    message = message
                ),
                LogLevel::Trace => tracing::trace!(
                    correlation_id = %ctx.correlation_id,
                    operation = %ctx.operation,
                    service = %ctx.service,
                    user_id = ?ctx.user_id,
                    session_id = ?ctx.session_id,
                    message = message
                ),
            }
        } else {
            match level {
                LogLevel::Error => tracing::error!(service = %self.service_name, message = message),
                LogLevel::Warn => tracing::warn!(service = %self.service_name, message = message),
                LogLevel::Info => tracing::info!(service = %self.service_name, message = message),
                LogLevel::Debug => tracing::debug!(service = %self.service_name, message = message),
                LogLevel::Trace => tracing::trace!(service = %self.service_name, message = message),
            }
        }
    }
}

/// Observability-aware logger that combines structured logging with telemetry
#[derive(Debug)]
pub struct ObservabilityLogger {
    structured_logger: Arc<StructuredLogger>,
    correlation_logger: CorrelationLogger,
}

impl ObservabilityLogger {
    pub fn new(structured_logger: Arc<StructuredLogger>, service_name: String) -> Self {
        Self {
            structured_logger,
            correlation_logger: CorrelationLogger::new(service_name),
        }
    }

    /// Log with full observability context
    pub async fn log_with_observability(
        &self,
        level: LogLevel,
        message: &str,
        trace_context: Option<&TraceContext>,
        correlation_context: Option<&CorrelationContext>,
    ) {
        if let Some(trace_ctx) = trace_context {
            self.structured_logger.log_with_context(level, message, trace_ctx);
        } else if let Some(corr_ctx) = correlation_context {
            self.structured_logger.log_with_correlation(level, message, corr_ctx);
        } else {
            self.structured_logger.log(level, message);
        }
    }
}

/// Log aggregator for collecting and analyzing logs
#[derive(Debug)]
pub struct LogAggregator {
    entries: Arc<RwLock<Vec<LogEntry>>>,
}

impl LogAggregator {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a log entry
    pub async fn add_entry(&self, entry: LogEntry) {
        self.entries.write().await.push(entry);
    }

    /// Get entries by correlation ID
    pub async fn get_entries_by_correlation(&self, correlation_id: &CorrelationId) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|entry| {
                entry.context.correlation_id.as_ref() == Some(correlation_id)
            })
            .cloned()
            .collect()
    }

    /// Get entries by operation
    pub async fn get_entries_by_operation(&self, operation: &str) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|entry| {
                entry.context.operation.as_deref() == Some(operation)
            })
            .cloned()
            .collect()
    }

    /// Get error entries
    pub async fn get_error_entries(&self) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|entry| entry.level == LogLevel::Error)
            .cloned()
            .collect()
    }

    /// Generate aggregated statistics
    pub async fn get_statistics(&self) -> LogStatistics {
        let entries = self.entries.read().await;
        
        let mut stats = LogStatistics::default();
        stats.total_entries = entries.len() as u64;

        for entry in entries.iter() {
            match entry.level {
                LogLevel::Error => stats.error_count += 1,
                LogLevel::Warn => stats.warn_count += 1,
                LogLevel::Info => stats.info_count += 1,
                LogLevel::Debug => stats.debug_count += 1,
                LogLevel::Trace => stats.trace_count += 1,
            }

            if let Some(operation) = &entry.context.operation {
                *stats.operations.entry(operation.clone()).or_insert(0) += 1;
            }

            if let Some(correlation_id) = &entry.context.correlation_id {
                stats.unique_correlations.insert(correlation_id.to_string());
            }
        }

        stats.unique_correlation_count = stats.unique_correlations.len() as u64;
        stats
    }
}

impl Default for LogAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about collected logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatistics {
    pub total_entries: u64,
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
    pub debug_count: u64,
    pub trace_count: u64,
    pub unique_correlation_count: u64,
    pub operations: HashMap<String, u64>,
    #[serde(skip)]
    pub unique_correlations: std::collections::HashSet<String>,
}

impl Default for LogStatistics {
    fn default() -> Self {
        Self {
            total_entries: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            debug_count: 0,
            trace_count: 0,
            unique_correlation_count: 0,
            operations: HashMap::new(),
            unique_correlations: std::collections::HashSet::new(),
        }
    }
}

/// Custom tracing layer for capturing structured logs
struct ObservabilityLayer {
    entries: Arc<RwLock<Vec<LogEntry>>>,
}

impl ObservabilityLayer {
    fn new(entries: Arc<RwLock<Vec<LogEntry>>>) -> Self {
        Self { entries }
    }
}

impl<S> Layer<S> for ObservabilityLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = LogLevel::from(metadata.level());
        
        // Extract message and fields from the event
        let mut visitor = EventVisitor::new();
        event.record(&mut visitor);
        
        let context = LogContext::new("eventuali-core")
            .with_attribute("target", metadata.target())
            .with_attribute("module", metadata.module_path().unwrap_or("unknown"));

        let entry = LogEntry::new(level, visitor.message.unwrap_or_default(), context)
            .with_metadata(
                metadata.module_path(),
                metadata.file(),
                metadata.line(),
                Some(metadata.target()),
            );

        if let Ok(mut entries) = self.entries.try_write() {
            entries.push(entry);
        }
    }
}

/// Visitor for extracting event data
struct EventVisitor {
    message: Option<String>,
    fields: HashMap<String, serde_json::Value>,
}

impl EventVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: HashMap::new(),
        }
    }
}

impl tracing::field::Visit for EventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            self.fields.insert(field.name().to_string(), serde_json::Value::String(format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(tracing::Level::from(LogLevel::Error), tracing::Level::ERROR);
        assert_eq!(tracing::Level::from(LogLevel::Info), tracing::Level::INFO);
        assert_eq!(LogLevel::from(&tracing::Level::WARN), LogLevel::Warn);
    }

    #[test]
    fn test_log_context_creation() {
        let context = LogContext::new("test-service")
            .with_attribute("key1", "value1")
            .with_user_id("user123")
            .with_operation("test_operation");

        assert_eq!(context.service_name, "test-service");
        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.operation, Some("test_operation".to_string()));
        assert_eq!(context.attributes.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_log_entry_creation() {
        let context = LogContext::new("test-service");
        let entry = LogEntry::new(LogLevel::Info, "Test message", context)
            .with_field("field1", serde_json::Value::String("value1".to_string()));

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.fields.get("field1"), Some(&serde_json::Value::String("value1".to_string())));
    }

    #[tokio::test]
    async fn test_structured_logger_creation() {
        let config = ObservabilityConfig {
            structured_logging: false, // Disable to avoid global subscriber conflicts
            ..ObservabilityConfig::default()
        };
        let logger = StructuredLogger::new(&config).unwrap();
        
        assert_eq!(logger.config.service_name, "eventuali");
    }

    #[tokio::test]
    async fn test_log_aggregator() {
        let aggregator = LogAggregator::new();
        let context = LogContext::new("test-service").with_operation("test_op");
        let entry1 = LogEntry::new(LogLevel::Info, "Message 1", context.clone());
        let entry2 = LogEntry::new(LogLevel::Error, "Message 2", context);

        aggregator.add_entry(entry1).await;
        aggregator.add_entry(entry2).await;

        let stats = aggregator.get_statistics().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.operations.get("test_op"), Some(&2));
    }
}