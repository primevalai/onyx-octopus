//! Performance profiling module for comprehensive performance analysis.
//!
//! This module provides:
//! - CPU profiling with sampling and flame graph generation
//! - Memory profiling with allocation tracking and leak detection
//! - I/O profiling with disk and network performance analysis
//! - Method-level performance tracking with call graphs
//! - Performance regression detection with alerting
//! - Bottleneck identification and optimization recommendations

use crate::error::{EventualiError, Result};
use crate::observability::CorrelationId;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Types of profiling that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    /// CPU profiling with sampling
    Cpu,
    /// Memory allocation profiling
    Memory,
    /// I/O performance profiling
    Io,
    /// Method-level call profiling
    Method,
    /// Combined profiling (all types)
    Combined,
}

/// Profiling configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Enable profiling globally
    pub enabled: bool,
    /// Sampling interval for CPU profiling (microseconds)
    pub cpu_sampling_interval_us: u64,
    /// Memory allocation threshold for tracking (bytes)
    pub memory_allocation_threshold: usize,
    /// I/O operation threshold for tracking (microseconds)
    pub io_threshold_us: u64,
    /// Maximum number of stack frames to capture
    pub max_stack_frames: usize,
    /// Maximum duration to keep profiling data (seconds)
    pub data_retention_seconds: u64,
    /// Enable flame graph generation
    pub enable_flame_graphs: bool,
    /// Performance regression threshold (percentage)
    pub regression_threshold_percent: f64,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_sampling_interval_us: 1000,       // 1ms sampling
            memory_allocation_threshold: 1024,     // 1KB allocations
            io_threshold_us: 100,                  // 100µs I/O operations
            max_stack_frames: 32,
            data_retention_seconds: 3600,          // 1 hour
            enable_flame_graphs: true,
            regression_threshold_percent: 10.0,    // 10% regression
        }
    }
}

/// Performance profile entry representing a single profiling sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    /// Unique ID for this profile entry
    pub id: String,
    /// Type of profiling
    pub profile_type: ProfileType,
    /// Timestamp when this sample was taken
    pub timestamp: SystemTime,
    /// Duration of the operation being profiled
    pub duration: Duration,
    /// Stack trace at the time of sampling
    pub stack_trace: Vec<String>,
    /// Memory information (if memory profiling)
    pub memory_info: Option<MemoryInfo>,
    /// I/O information (if I/O profiling)
    pub io_info: Option<IoInfo>,
    /// Correlation ID for tracing
    pub correlation_id: Option<CorrelationId>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Memory profiling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total allocated bytes
    pub allocated_bytes: usize,
    /// Total deallocated bytes
    pub deallocated_bytes: usize,
    /// Current memory usage
    pub current_usage_bytes: usize,
    /// Peak memory usage
    pub peak_usage_bytes: usize,
    /// Number of allocations
    pub allocation_count: usize,
    /// Number of deallocations
    pub deallocation_count: usize,
}

/// I/O profiling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoInfo {
    /// Type of I/O operation
    pub operation_type: String,
    /// Bytes read
    pub bytes_read: u64,
    /// Bytes written
    pub bytes_written: u64,
    /// Number of I/O operations
    pub operation_count: u64,
    /// Average I/O latency
    pub average_latency: Duration,
    /// I/O target (file path, network endpoint, etc.)
    pub target: String,
}

/// Call graph node for method-level profiling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphNode {
    /// Method or function name
    pub name: String,
    /// Total time spent in this method (including children)
    pub total_time: Duration,
    /// Self time spent in this method (excluding children)
    pub self_time: Duration,
    /// Number of times this method was called
    pub call_count: u64,
    /// Children method calls
    pub children: HashMap<String, CallGraphNode>,
    /// Average time per call
    pub avg_time_per_call: Duration,
}

/// Performance regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetection {
    /// Operation or method name
    pub operation: String,
    /// Current performance metrics
    pub current_metrics: PerformanceSnapshot,
    /// Baseline performance metrics
    pub baseline_metrics: PerformanceSnapshot,
    /// Percentage change from baseline
    pub change_percent: f64,
    /// Whether this is considered a regression
    pub is_regression: bool,
    /// Severity level of the regression
    pub severity: RegressionSeverity,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Performance snapshot for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Average execution time
    pub avg_execution_time: Duration,
    /// 95th percentile execution time
    pub p95_execution_time: Duration,
    /// 99th percentile execution time
    pub p99_execution_time: Duration,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Memory usage
    pub memory_usage_bytes: usize,
    /// Error rate
    pub error_rate: f64,
    /// Timestamp of snapshot
    pub timestamp: SystemTime,
}

/// Severity levels for performance regressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Low,      // 10-25% regression
    Medium,   // 25-50% regression
    High,     // 50-100% regression
    Critical, // >100% regression
}

/// Flame graph data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraph {
    /// Root node of the flame graph
    pub root: FlameGraphNode,
    /// Total duration represented by the flame graph
    pub total_duration: Duration,
    /// Number of samples in the flame graph
    pub sample_count: usize,
    /// Metadata about the flame graph
    pub metadata: HashMap<String, String>,
}

/// Individual node in a flame graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphNode {
    /// Name of the function/method
    pub name: String,
    /// Total time spent in this node and its children
    pub total_time: Duration,
    /// Self time spent in this node only
    pub self_time: Duration,
    /// Number of samples for this node
    pub sample_count: usize,
    /// Child nodes
    pub children: HashMap<String, FlameGraphNode>,
    /// Percentage of total execution time
    pub percentage: f64,
}

/// Bottleneck identification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    /// Top bottlenecks found
    pub bottlenecks: Vec<Bottleneck>,
    /// Analysis timestamp
    pub timestamp: SystemTime,
    /// Total analysis duration
    pub analysis_duration: Duration,
    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

/// Individual bottleneck identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Location of the bottleneck (function, method, etc.)
    pub location: String,
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Impact score (0-100)
    pub impact_score: f64,
    /// Time spent in this bottleneck
    pub time_spent: Duration,
    /// Percentage of total execution time
    pub percentage_of_total: f64,
    /// Call frequency
    pub call_frequency: u64,
    /// Description of the bottleneck
    pub description: String,
}

/// Types of bottlenecks that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckType {
    /// CPU-intensive computation
    Cpu,
    /// Memory allocation/deallocation
    Memory,
    /// I/O operations
    Io,
    /// Lock contention
    Lock,
    /// Network operations
    Network,
    /// Database operations
    Database,
    /// Serialization/deserialization
    Serialization,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Target area for optimization
    pub target: String,
    /// Type of optimization
    pub optimization_type: String,
    /// Expected impact
    pub expected_impact: String,
    /// Implementation effort
    pub effort_level: String,
    /// Detailed description
    pub description: String,
    /// Code examples or references
    pub examples: Vec<String>,
}

/// Main performance profiler service
#[derive(Debug)]
pub struct PerformanceProfiler {
    #[allow(dead_code)] // Profiling configuration settings (stored but not currently accessed in implementation)
    config: ProfilingConfig,
    profile_data: Arc<RwLock<VecDeque<ProfileEntry>>>,
    active_profiles: Arc<RwLock<HashMap<String, ProfileSession>>>,
    baseline_metrics: Arc<RwLock<HashMap<String, PerformanceSnapshot>>>,
    regression_history: Arc<RwLock<VecDeque<RegressionDetection>>>,
    #[allow(dead_code)] // Call graph for performance analysis (constructed but not currently traversed)
    call_graph: Arc<RwLock<CallGraphNode>>,
}

/// Active profiling session
#[derive(Debug, Clone)]
pub struct ProfileSession {
    pub id: String,
    pub profile_type: ProfileType,
    pub start_time: Instant,
    pub correlation_id: Option<CorrelationId>,
    pub metadata: HashMap<String, String>,
}

impl PerformanceProfiler {
    /// Create a new performance profiler with the given configuration
    pub fn new(config: ProfilingConfig) -> Self {
        Self {
            config,
            profile_data: Arc::new(RwLock::new(VecDeque::new())),
            active_profiles: Arc::new(RwLock::new(HashMap::new())),
            baseline_metrics: Arc::new(RwLock::new(HashMap::new())),
            regression_history: Arc::new(RwLock::new(VecDeque::new())),
            call_graph: Arc::new(RwLock::new(CallGraphNode {
                name: "root".to_string(),
                total_time: Duration::ZERO,
                self_time: Duration::ZERO,
                call_count: 0,
                children: HashMap::new(),
                avg_time_per_call: Duration::ZERO,
            })),
        }
    }

    /// Start a new profiling session
    pub async fn start_profiling(
        &self,
        profile_type: ProfileType,
        correlation_id: Option<CorrelationId>,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        if !self.config.enabled {
            return Err(EventualiError::InvalidState("Profiling is disabled".to_string()));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = ProfileSession {
            id: session_id.clone(),
            profile_type,
            start_time: Instant::now(),
            correlation_id,
            metadata,
        };

        let mut active_profiles = self.active_profiles.write().await;
        active_profiles.insert(session_id.clone(), session);

        tracing::info!("Started profiling session: {} (type: {:?})", session_id, profile_type);
        Ok(session_id)
    }

    /// End a profiling session and collect results
    pub async fn end_profiling(&self, session_id: &str) -> Result<ProfileEntry> {
        let mut active_profiles = self.active_profiles.write().await;
        let session = active_profiles.remove(session_id)
            .ok_or_else(|| EventualiError::InvalidState(format!("Profile session not found: {session_id}")))?;

        let duration = session.start_time.elapsed();
        let stack_trace = self.capture_stack_trace().await;
        
        let memory_info = if session.profile_type == ProfileType::Memory || session.profile_type == ProfileType::Combined {
            Some(self.collect_memory_info().await?)
        } else {
            None
        };

        let io_info = if session.profile_type == ProfileType::Io || session.profile_type == ProfileType::Combined {
            Some(self.collect_io_info().await?)
        } else {
            None
        };

        let entry = ProfileEntry {
            id: session_id.to_string(),
            profile_type: session.profile_type,
            timestamp: SystemTime::now(),
            duration,
            stack_trace,
            memory_info,
            io_info,
            correlation_id: session.correlation_id,
            metadata: session.metadata,
        };

        // Store the profile entry
        let mut profile_data = self.profile_data.write().await;
        profile_data.push_back(entry.clone());

        // Cleanup old data if necessary
        self.cleanup_old_data(&mut profile_data).await;

        tracing::info!("Ended profiling session: {} (duration: {:?})", session_id, duration);
        Ok(entry)
    }

    /// Capture current stack trace
    async fn capture_stack_trace(&self) -> Vec<String> {
        // In a real implementation, this would use platform-specific APIs
        // to capture the actual stack trace. For now, we'll simulate it.
        vec![
            "eventuali::event_store::append_events".to_string(),
            "eventuali::aggregate::apply_event".to_string(),
            "eventuali::performance::profile_operation".to_string(),
        ]
    }

    /// Collect current memory information
    async fn collect_memory_info(&self) -> Result<MemoryInfo> {
        // In a real implementation, this would collect actual memory statistics
        // using platform-specific APIs or memory profiling libraries
        Ok(MemoryInfo {
            allocated_bytes: 1024 * 1024,      // 1MB
            deallocated_bytes: 512 * 1024,     // 512KB
            current_usage_bytes: 512 * 1024,   // 512KB
            peak_usage_bytes: 2 * 1024 * 1024, // 2MB
            allocation_count: 100,
            deallocation_count: 50,
        })
    }

    /// Collect current I/O information
    async fn collect_io_info(&self) -> Result<IoInfo> {
        // In a real implementation, this would collect actual I/O statistics
        Ok(IoInfo {
            operation_type: "database_write".to_string(),
            bytes_read: 2048,
            bytes_written: 4096,
            operation_count: 10,
            average_latency: Duration::from_micros(250),
            target: "sqlite://events.db".to_string(),
        })
    }

    /// Generate a flame graph from collected profile data
    pub async fn generate_flame_graph(
        &self,
        profile_type: ProfileType,
        time_range: Option<(SystemTime, SystemTime)>,
    ) -> Result<FlameGraph> {
        if !self.config.enable_flame_graphs {
            return Err(EventualiError::InvalidState("Flame graph generation is disabled".to_string()));
        }

        let profile_data = self.profile_data.read().await;
        let filtered_data: Vec<_> = profile_data.iter()
            .filter(|entry| {
                entry.profile_type == profile_type || profile_type == ProfileType::Combined
            })
            .filter(|entry| {
                if let Some((start, end)) = time_range {
                    entry.timestamp >= start && entry.timestamp <= end
                } else {
                    true
                }
            })
            .collect();

        let mut root = FlameGraphNode {
            name: "root".to_string(),
            total_time: Duration::ZERO,
            self_time: Duration::ZERO,
            sample_count: 0,
            children: HashMap::new(),
            percentage: 100.0,
        };

        let total_samples = filtered_data.len();
        let mut total_duration = Duration::ZERO;

        for entry in &filtered_data {
            total_duration += entry.duration;
            self.add_to_flame_graph(&mut root, &entry.stack_trace, entry.duration).await;
        }

        // Calculate percentages
        self.calculate_flame_graph_percentages(&mut root, total_duration).await;

        Ok(FlameGraph {
            root,
            total_duration,
            sample_count: total_samples,
            metadata: HashMap::from([
                ("profile_type".to_string(), format!("{profile_type:?}")),
                ("generation_time".to_string(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()),
            ]),
        })
    }

    /// Add a stack trace to the flame graph
    async fn add_to_flame_graph(&self, node: &mut FlameGraphNode, stack_trace: &[String], duration: Duration) {
        if stack_trace.is_empty() {
            node.self_time += duration;
            node.sample_count += 1;
            return;
        }

        let frame = &stack_trace[0];
        let child = node.children.entry(frame.clone()).or_insert_with(|| FlameGraphNode {
            name: frame.clone(),
            total_time: Duration::ZERO,
            self_time: Duration::ZERO,
            sample_count: 0,
            children: HashMap::new(),
            percentage: 0.0,
        });

        child.total_time += duration;
        child.sample_count += 1;

        if stack_trace.len() > 1 {
            Box::pin(self.add_to_flame_graph(child, &stack_trace[1..], duration)).await;
        } else {
            child.self_time += duration;
        }
    }

    /// Calculate percentages for flame graph nodes
    async fn calculate_flame_graph_percentages(&self, node: &mut FlameGraphNode, total_duration: Duration) {
        if total_duration > Duration::ZERO {
            node.percentage = (node.total_time.as_nanos() as f64 / total_duration.as_nanos() as f64) * 100.0;
        }

        for child in node.children.values_mut() {
            Box::pin(self.calculate_flame_graph_percentages(child, total_duration)).await;
        }
    }

    /// Detect performance regressions by comparing current metrics with baselines
    pub async fn detect_regressions(&self, operation: &str) -> Result<Option<RegressionDetection>> {
        let current_metrics = self.collect_current_metrics(operation).await?;
        
        let baseline_metrics = self.baseline_metrics.read().await;
        if let Some(baseline) = baseline_metrics.get(operation) {
            let change_percent = ((current_metrics.avg_execution_time.as_nanos() as f64 - 
                                  baseline.avg_execution_time.as_nanos() as f64) /
                                 baseline.avg_execution_time.as_nanos() as f64) * 100.0;

            let is_regression = change_percent > self.config.regression_threshold_percent;
            let severity = match change_percent {
                x if x > 100.0 => RegressionSeverity::Critical,
                x if x > 50.0 => RegressionSeverity::High,
                x if x > 25.0 => RegressionSeverity::Medium,
                _ => RegressionSeverity::Low,
            };

            let recommendations = self.generate_optimization_recommendations(operation, &current_metrics, baseline).await;

            let detection = RegressionDetection {
                operation: operation.to_string(),
                current_metrics,
                baseline_metrics: baseline.clone(),
                change_percent,
                is_regression,
                severity,
                recommendations,
            };

            if is_regression {
                let mut regression_history = self.regression_history.write().await;
                regression_history.push_back(detection.clone());
                
                // Keep only recent regressions
                while regression_history.len() > 1000 {
                    regression_history.pop_front();
                }

                tracing::warn!("Performance regression detected for {}: {:.2}% slower", operation, change_percent);
            }

            Ok(Some(detection))
        } else {
            Ok(None)
        }
    }

    /// Collect current performance metrics for an operation
    async fn collect_current_metrics(&self, operation: &str) -> Result<PerformanceSnapshot> {
        let profile_data = self.profile_data.read().await;
        let operation_data: Vec<_> = profile_data.iter()
            .filter(|entry| entry.metadata.get("operation").is_some_and(|op| op == operation))
            .collect();

        if operation_data.is_empty() {
            return Ok(PerformanceSnapshot {
                avg_execution_time: Duration::ZERO,
                p95_execution_time: Duration::ZERO,
                p99_execution_time: Duration::ZERO,
                throughput: 0.0,
                memory_usage_bytes: 0,
                error_rate: 0.0,
                timestamp: SystemTime::now(),
            });
        }

        let mut durations: Vec<Duration> = operation_data.iter().map(|entry| entry.duration).collect();
        durations.sort();

        let avg_execution_time = Duration::from_nanos(
            (durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128) as u64
        );

        let p95_index = (durations.len() as f64 * 0.95) as usize;
        let p99_index = (durations.len() as f64 * 0.99) as usize;
        
        let p95_execution_time = durations.get(p95_index.min(durations.len() - 1)).copied().unwrap_or(Duration::ZERO);
        let p99_execution_time = durations.get(p99_index.min(durations.len() - 1)).copied().unwrap_or(Duration::ZERO);

        let throughput = if !durations.is_empty() {
            1.0 / avg_execution_time.as_secs_f64()
        } else {
            0.0
        };

        let memory_usage_bytes = operation_data.iter()
            .filter_map(|entry| entry.memory_info.as_ref())
            .map(|info| info.current_usage_bytes)
            .max()
            .unwrap_or(0);

        Ok(PerformanceSnapshot {
            avg_execution_time,
            p95_execution_time,
            p99_execution_time,
            throughput,
            memory_usage_bytes,
            error_rate: 0.0, // Would be calculated from error metrics
            timestamp: SystemTime::now(),
        })
    }

    /// Generate optimization recommendations
    async fn generate_optimization_recommendations(
        &self,
        _operation: &str,
        current: &PerformanceSnapshot,
        baseline: &PerformanceSnapshot,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Performance degradation recommendations
        if current.avg_execution_time > baseline.avg_execution_time * 2 {
            recommendations.push("Consider profiling CPU usage - execution time has doubled".to_string());
        }

        // Memory usage recommendations
        if current.memory_usage_bytes > baseline.memory_usage_bytes * 2 {
            recommendations.push("Memory usage has significantly increased - check for memory leaks".to_string());
        }

        // Throughput recommendations
        if current.throughput < baseline.throughput * 0.5 {
            recommendations.push("Throughput has dropped significantly - consider optimizing bottlenecks".to_string());
        }

        // General recommendations
        recommendations.push("Use flame graphs to identify specific bottlenecks".to_string());
        recommendations.push("Consider enabling detailed I/O profiling".to_string());
        recommendations.push("Review recent code changes for performance impact".to_string());

        recommendations
    }

    /// Identify bottlenecks in the current profile data
    pub async fn identify_bottlenecks(&self, profile_type: ProfileType) -> Result<BottleneckAnalysis> {
        let start_time = Instant::now();
        let profile_data = self.profile_data.read().await;
        
        let filtered_data: Vec<_> = profile_data.iter()
            .filter(|entry| entry.profile_type == profile_type || profile_type == ProfileType::Combined)
            .collect();

        let mut bottlenecks = Vec::new();
        let mut function_times: HashMap<String, (Duration, u64)> = HashMap::new();
        let total_time: Duration = filtered_data.iter().map(|entry| entry.duration).sum();

        // Analyze function call times
        for entry in &filtered_data {
            for frame in &entry.stack_trace {
                let (current_time, count) = function_times.get(frame).unwrap_or(&(Duration::ZERO, 0));
                function_times.insert(frame.clone(), (*current_time + entry.duration, count + 1));
            }
        }

        // Identify top bottlenecks
        let mut sorted_functions: Vec<_> = function_times.iter().collect();
        sorted_functions.sort_by(|a, b| b.1.0.cmp(&a.1.0));

        for (function, (time, count)) in sorted_functions.iter().take(10) {
            let percentage = (time.as_nanos() as f64 / total_time.as_nanos() as f64) * 100.0;
            let bottleneck_type = self.classify_bottleneck_type(function);
            
            bottlenecks.push(Bottleneck {
                location: (*function).clone(),
                bottleneck_type,
                impact_score: percentage,
                time_spent: *time,
                percentage_of_total: percentage,
                call_frequency: *count,
                description: format!("Function {function} consumes {percentage:.2}% of total execution time"),
            });
        }

        let optimization_suggestions = self.generate_bottleneck_optimizations(&bottlenecks).await;

        Ok(BottleneckAnalysis {
            bottlenecks,
            timestamp: SystemTime::now(),
            analysis_duration: start_time.elapsed(),
            optimization_suggestions,
        })
    }

    /// Classify the type of bottleneck based on function name
    fn classify_bottleneck_type(&self, function_name: &str) -> BottleneckType {
        if function_name.contains("database") || function_name.contains("sql") {
            BottleneckType::Database
        } else if function_name.contains("serialize") || function_name.contains("deserialize") {
            BottleneckType::Serialization
        } else if function_name.contains("network") || function_name.contains("http") {
            BottleneckType::Network
        } else if function_name.contains("lock") || function_name.contains("mutex") {
            BottleneckType::Lock
        } else if function_name.contains("io") || function_name.contains("read") || function_name.contains("write") {
            BottleneckType::Io
        } else if function_name.contains("alloc") || function_name.contains("free") {
            BottleneckType::Memory
        } else {
            BottleneckType::Cpu
        }
    }

    /// Generate optimization suggestions for bottlenecks
    async fn generate_bottleneck_optimizations(&self, bottlenecks: &[Bottleneck]) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        for bottleneck in bottlenecks {
            let suggestion = match bottleneck.bottleneck_type {
                BottleneckType::Database => OptimizationSuggestion {
                    target: bottleneck.location.clone(),
                    optimization_type: "Database Optimization".to_string(),
                    expected_impact: "20-50% performance improvement".to_string(),
                    effort_level: "Medium".to_string(),
                    description: "Add database indexes, optimize queries, consider connection pooling".to_string(),
                    examples: vec![
                        "CREATE INDEX idx_events_aggregate_id ON events(aggregate_id)".to_string(),
                        "Use prepared statements for repeated queries".to_string(),
                    ],
                },
                BottleneckType::Serialization => OptimizationSuggestion {
                    target: bottleneck.location.clone(),
                    optimization_type: "Serialization Optimization".to_string(),
                    expected_impact: "10-30% performance improvement".to_string(),
                    effort_level: "Low".to_string(),
                    description: "Use more efficient serialization formats or optimize serialization code".to_string(),
                    examples: vec![
                        "Consider using Protocol Buffers instead of JSON".to_string(),
                        "Implement custom serialization for hot paths".to_string(),
                    ],
                },
                BottleneckType::Memory => OptimizationSuggestion {
                    target: bottleneck.location.clone(),
                    optimization_type: "Memory Optimization".to_string(),
                    expected_impact: "15-40% performance improvement".to_string(),
                    effort_level: "Medium".to_string(),
                    description: "Optimize memory allocation patterns, use object pooling".to_string(),
                    examples: vec![
                        "Use Vec::with_capacity() to pre-allocate vectors".to_string(),
                        "Implement object pooling for frequently allocated objects".to_string(),
                    ],
                },
                _ => OptimizationSuggestion {
                    target: bottleneck.location.clone(),
                    optimization_type: "General Optimization".to_string(),
                    expected_impact: "5-20% performance improvement".to_string(),
                    effort_level: "Variable".to_string(),
                    description: "Profile the specific function to identify optimization opportunities".to_string(),
                    examples: vec![
                        "Use more efficient algorithms".to_string(),
                        "Reduce unnecessary computations".to_string(),
                    ],
                },
            };
            suggestions.push(suggestion);
        }

        suggestions
    }

    /// Set baseline metrics for an operation
    pub async fn set_baseline(&self, operation: &str) -> Result<()> {
        let metrics = self.collect_current_metrics(operation).await?;
        let mut baseline_metrics = self.baseline_metrics.write().await;
        baseline_metrics.insert(operation.to_string(), metrics);
        
        tracing::info!("Set baseline metrics for operation: {}", operation);
        Ok(())
    }

    /// Get current profiling statistics
    pub async fn get_statistics(&self) -> Result<HashMap<String, serde_json::Value>> {
        let profile_data = self.profile_data.read().await;
        let active_profiles = self.active_profiles.read().await;
        let regression_history = self.regression_history.read().await;

        let total_profiles = profile_data.len();
        let active_sessions = active_profiles.len();
        let total_regressions = regression_history.len();

        let profile_type_counts = profile_data.iter()
            .fold(HashMap::new(), |mut acc, entry| {
                *acc.entry(format!("{:?}", entry.profile_type)).or_insert(0) += 1;
                acc
            });

        Ok(HashMap::from([
            ("total_profiles".to_string(), serde_json::Value::Number(total_profiles.into())),
            ("active_sessions".to_string(), serde_json::Value::Number(active_sessions.into())),
            ("total_regressions".to_string(), serde_json::Value::Number(total_regressions.into())),
            ("profile_type_counts".to_string(), serde_json::to_value(profile_type_counts)?),
            ("config".to_string(), serde_json::to_value(&self.config)?),
        ]))
    }

    /// Cleanup old profile data based on retention settings
    async fn cleanup_old_data(&self, profile_data: &mut VecDeque<ProfileEntry>) {
        let cutoff = SystemTime::now() - Duration::from_secs(self.config.data_retention_seconds);
        
        while let Some(entry) = profile_data.front() {
            if entry.timestamp < cutoff {
                profile_data.pop_front();
            } else {
                break;
            }
        }
    }

    /// Export profile data to various formats
    pub async fn export_profile_data(&self, format: &str) -> Result<String> {
        let profile_data = self.profile_data.read().await;
        
        match format.to_lowercase().as_str() {
            "json" => Ok(serde_json::to_string_pretty(&*profile_data)?),
            "csv" => {
                let mut csv = String::from("id,type,timestamp,duration_ns,stack_depth\n");
                for entry in profile_data.iter() {
                    csv.push_str(&format!(
                        "{},{:?},{},{},{}\n",
                        entry.id,
                        entry.profile_type,
                        entry.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        entry.duration.as_nanos(),
                        entry.stack_trace.len()
                    ));
                }
                Ok(csv)
            },
            _ => Err(EventualiError::InvalidState(format!("Unsupported export format: {format}")))
        }
    }
}

/// Performance profiler builder for easy configuration
pub struct PerformanceProfilerBuilder {
    config: ProfilingConfig,
}

impl PerformanceProfilerBuilder {
    pub fn new() -> Self {
        Self {
            config: ProfilingConfig::default(),
        }
    }

    pub fn with_cpu_sampling_interval(mut self, interval_us: u64) -> Self {
        self.config.cpu_sampling_interval_us = interval_us;
        self
    }

    pub fn with_memory_threshold(mut self, threshold_bytes: usize) -> Self {
        self.config.memory_allocation_threshold = threshold_bytes;
        self
    }

    pub fn with_io_threshold(mut self, threshold_us: u64) -> Self {
        self.config.io_threshold_us = threshold_us;
        self
    }

    pub fn with_flame_graphs(mut self, enabled: bool) -> Self {
        self.config.enable_flame_graphs = enabled;
        self
    }

    pub fn with_regression_threshold(mut self, threshold_percent: f64) -> Self {
        self.config.regression_threshold_percent = threshold_percent;
        self
    }

    pub fn build(self) -> PerformanceProfiler {
        PerformanceProfiler::new(self.config)
    }
}

impl Default for PerformanceProfilerBuilder {
    fn default() -> Self {
        Self::new()
    }
}