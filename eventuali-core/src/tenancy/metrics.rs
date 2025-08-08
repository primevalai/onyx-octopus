//! Advanced tenant metrics and observability
//!
//! This module provides comprehensive tenant metrics, analytics, and real-time monitoring:
//! - Real-time tenant performance monitoring
//! - Advanced analytics with trend detection
//! - Custom dashboards and alerting
//! - Multi-dimensional metrics collection
//! - Historical data analysis and reporting
//! - SLA monitoring and compliance tracking

use std::sync::{Arc, RwLock, Mutex};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

/// Type alias for hourly aggregation storage
pub type HourlyAggregations = Arc<RwLock<BTreeMap<DateTime<Utc>, HashMap<String, AggregatedMetric>>>>;

/// Type alias for metric record tuple
pub type MetricRecord = (String, f64, Option<HashMap<String, String>>);

use super::tenant::TenantId;
use super::quota::{UsagePattern, AlertType};
use crate::error::{EventualiError, Result};

/// Time-series data point for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

impl MetricDataPoint {
    pub fn new(value: f64) -> Self {
        MetricDataPoint {
            timestamp: Utc::now(),
            value,
            labels: HashMap::new(),
        }
    }

    pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }

    pub fn add_label(&mut self, key: String, value: String) {
        self.labels.insert(key, value);
    }
}

/// Time-series metric with rolling window
#[derive(Debug)]
pub struct TimeSeriesMetric {
    #[allow(dead_code)] // Metric name for identification (stored but not currently accessed in implementation)
    name: String,
    data_points: VecDeque<MetricDataPoint>,
    max_points: usize,
    retention_period: Duration,
}

impl TimeSeriesMetric {
    pub fn new(name: String, max_points: usize, retention_hours: u64) -> Self {
        TimeSeriesMetric {
            name,
            data_points: VecDeque::new(),
            max_points,
            retention_period: Duration::from_secs(retention_hours * 3600),
        }
    }

    pub fn add_point(&mut self, point: MetricDataPoint) {
        // Remove expired points
        let cutoff_time = Utc::now() - chrono::Duration::from_std(self.retention_period).unwrap();
        while let Some(front) = self.data_points.front() {
            if front.timestamp < cutoff_time {
                self.data_points.pop_front();
            } else {
                break;
            }
        }

        // Add new point
        self.data_points.push_back(point);

        // Maintain max points limit
        if self.data_points.len() > self.max_points {
            self.data_points.pop_front();
        }
    }

    pub fn get_latest(&self) -> Option<&MetricDataPoint> {
        self.data_points.back()
    }

    pub fn get_points(&self) -> Vec<&MetricDataPoint> {
        self.data_points.iter().collect()
    }

    pub fn get_points_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&MetricDataPoint> {
        self.data_points
            .iter()
            .filter(|point| point.timestamp >= start && point.timestamp <= end)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.data_points.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.data_points.iter().map(|p| p.value).sum();
        sum / self.data_points.len() as f64
    }

    pub fn calculate_percentile(&self, percentile: f64) -> f64 {
        if self.data_points.is_empty() {
            return 0.0;
        }

        let mut values: Vec<f64> = self.data_points.iter().map(|p| p.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((values.len() - 1) as f64 * percentile / 100.0).round() as usize;
        values[index.min(values.len() - 1)]
    }

    pub fn detect_anomalies(&self, threshold_multiplier: f64) -> Vec<&MetricDataPoint> {
        if self.data_points.len() < 10 {
            return Vec::new(); // Need enough data for anomaly detection
        }

        let mean = self.calculate_average();
        let variance = self.calculate_variance();
        let std_dev = variance.sqrt();
        let threshold = std_dev * threshold_multiplier;

        self.data_points
            .iter()
            .filter(|point| (point.value - mean).abs() > threshold)
            .collect()
    }

    pub fn calculate_variance(&self) -> f64 {
        if self.data_points.len() < 2 {
            return 0.0;
        }

        let mean = self.calculate_average();
        let sum_squared_diff: f64 = self.data_points
            .iter()
            .map(|p| (p.value - mean).powi(2))
            .sum();
        
        sum_squared_diff / (self.data_points.len() - 1) as f64
    }

    pub fn get_trend(&self) -> UsagePattern {
        if self.data_points.len() < 5 {
            return UsagePattern::Stable;
        }

        // Simple linear regression to detect trend
        let n = self.data_points.len() as f64;
        let points: Vec<(f64, f64)> = self.data_points
            .iter()
            .enumerate()
            .map(|(i, point)| (i as f64, point.value))
            .collect();

        let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        
        // Calculate R² for trend strength
        let mean_y = sum_y / n;
        let ss_tot: f64 = points.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
        let ss_res: f64 = points.iter().map(|(x, y)| {
            let predicted = slope * x + (sum_y - slope * sum_x) / n;
            (y - predicted).powi(2)
        }).sum();
        
        let r_squared = 1.0 - (ss_res / ss_tot);

        // Determine pattern based on slope and correlation
        if r_squared < 0.5 {
            UsagePattern::Volatile
        } else if slope > 0.1 {
            UsagePattern::Growing
        } else if slope < -0.1 {
            UsagePattern::Declining
        } else {
            UsagePattern::Stable
        }
    }
}

/// Aggregated metric for different time windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub sum: f64,
    pub count: u64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl AggregatedMetric {
    pub fn from_points(name: String, points: &[&MetricDataPoint]) -> Self {
        if points.is_empty() {
            return AggregatedMetric {
                name,
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                sum: 0.0,
                count: 0,
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
                start_time: Utc::now(),
                end_time: Utc::now(),
            };
        }

        let mut values: Vec<f64> = points.iter().map(|p| p.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = values.first().copied().unwrap_or(0.0);
        let max = values.last().copied().unwrap_or(0.0);
        let sum: f64 = values.iter().sum();
        let count = values.len() as u64;
        let avg = sum / count as f64;

        let p50_idx = (count as f64 * 0.5).round() as usize;
        let p95_idx = (count as f64 * 0.95).round() as usize;
        let p99_idx = (count as f64 * 0.99).round() as usize;

        let p50 = values.get(p50_idx.min(values.len() - 1)).copied().unwrap_or(0.0);
        let p95 = values.get(p95_idx.min(values.len() - 1)).copied().unwrap_or(0.0);
        let p99 = values.get(p99_idx.min(values.len() - 1)).copied().unwrap_or(0.0);

        let start_time = points.iter().map(|p| p.timestamp).min().unwrap_or_else(Utc::now);
        let end_time = points.iter().map(|p| p.timestamp).max().unwrap_or_else(Utc::now);

        AggregatedMetric {
            name,
            min,
            max,
            avg,
            sum,
            count,
            p50,
            p95,
            p99,
            start_time,
            end_time,
        }
    }
}

/// SLA (Service Level Agreement) definition and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDefinition {
    pub name: String,
    pub metric_name: String,
    pub threshold: f64,
    pub operator: SlaOperator,
    pub target_percentage: f64, // e.g., 99.9% uptime
    pub measurement_window: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlaOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

/// SLA measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaResult {
    pub sla_name: String,
    pub measurement_period_start: DateTime<Utc>,
    pub measurement_period_end: DateTime<Utc>,
    pub compliance_percentage: f64,
    pub violations_count: u64,
    pub total_measurements: u64,
    pub is_compliant: bool,
    pub breach_duration: Duration,
}

/// Alert rule for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAlertRule {
    pub name: String,
    pub metric_name: String,
    pub threshold: f64,
    pub operator: SlaOperator,
    pub severity: AlertType,
    pub evaluation_window: Duration,
    pub cooldown_period: Duration,
    pub enabled: bool,
}

/// Alert triggered by metric rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAlert {
    pub id: String,
    pub rule_name: String,
    pub tenant_id: TenantId,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: AlertType,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged: bool,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDashboard {
    pub name: String,
    pub description: String,
    pub widgets: Vec<DashboardWidget>,
    pub refresh_interval: Duration,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: WidgetType,
    pub title: String,
    pub metric_names: Vec<String>,
    pub time_range: Duration,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub configuration: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    AreaChart,
    BarChart,
    Gauge,
    SingleValue,
    Table,
    Heatmap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

/// Advanced tenant metrics collector and analyzer
pub struct TenantMetricsCollector {
    tenant_id: TenantId,
    metrics: Arc<RwLock<HashMap<String, TimeSeriesMetric>>>,
    sla_definitions: Arc<RwLock<Vec<SlaDefinition>>>,
    alert_rules: Arc<RwLock<Vec<MetricAlertRule>>>,
    active_alerts: Arc<RwLock<Vec<MetricAlert>>>,
    dashboards: Arc<RwLock<Vec<TenantDashboard>>>,
    #[allow(dead_code)] // Collection interval for automated metric collection (configured but not actively used in current implementation)
    collection_interval: Duration,
    #[allow(dead_code)] // Last collection timestamp for scheduling (tracked but not currently utilized)
    last_collection: Arc<Mutex<Instant>>,
    
    // Pre-computed aggregations for performance
    #[allow(dead_code)] // Hourly aggregations for performance optimization (stored but not currently queried)
    hourly_aggregations: HourlyAggregations,
    #[allow(dead_code)] // Daily aggregations for long-term analytics (stored but not currently accessed)
    daily_aggregations: Arc<RwLock<BTreeMap<NaiveDate, HashMap<String, AggregatedMetric>>>>,
}

impl TenantMetricsCollector {
    pub fn new(tenant_id: TenantId) -> Self {
        TenantMetricsCollector {
            tenant_id,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            sla_definitions: Arc::new(RwLock::new(Vec::new())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            dashboards: Arc::new(RwLock::new(Vec::new())),
            collection_interval: Duration::from_secs(60), // 1 minute default
            last_collection: Arc::new(Mutex::new(Instant::now())),
            hourly_aggregations: Arc::new(RwLock::new(BTreeMap::new())),
            daily_aggregations: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Record a metric data point
    pub fn record_metric(&self, name: String, value: f64, labels: Option<HashMap<String, String>>) {
        let point = if let Some(labels) = labels {
            MetricDataPoint::new(value).with_labels(labels)
        } else {
            MetricDataPoint::new(value)
        };

        let mut metrics = self.metrics.write().unwrap();
        let metric = metrics.entry(name.clone()).or_insert_with(|| {
            TimeSeriesMetric::new(name.clone(), 10000, 24) // 10k points, 24h retention
        });

        metric.add_point(point);

        // Check alert rules
        drop(metrics); // Release the lock before checking alerts
        self.check_alert_rules(&name, value);
    }

    /// Record multiple metrics at once
    pub fn record_metrics(&self, metrics: Vec<MetricRecord>) {
        for (name, value, labels) in metrics {
            self.record_metric(name, value, labels);
        }
    }

    /// Get current value of a metric
    pub fn get_current_metric_value(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(name)?.get_latest().map(|point| point.value)
    }

    /// Get metric time series data
    pub fn get_metric_timeseries(
        &self,
        name: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Option<Vec<MetricDataPoint>> {
        let metrics = self.metrics.read().unwrap();
        let metric = metrics.get(name)?;

        let points = if let (Some(start), Some(end)) = (start, end) {
            metric.get_points_in_range(start, end)
        } else {
            metric.get_points()
        };

        Some(points.into_iter().cloned().collect())
    }

    /// Get aggregated metrics for time window
    pub fn get_aggregated_metrics(
        &self,
        names: &[String],
        window: Duration,
    ) -> HashMap<String, AggregatedMetric> {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::from_std(window).unwrap();
        
        let metrics = self.metrics.read().unwrap();
        let mut result = HashMap::new();

        for name in names {
            if let Some(metric) = metrics.get(name) {
                let points = metric.get_points_in_range(start_time, end_time);
                let aggregated = AggregatedMetric::from_points(name.clone(), &points);
                result.insert(name.clone(), aggregated);
            }
        }

        result
    }

    /// Detect anomalies across all metrics
    pub fn detect_anomalies(&self, threshold_multiplier: f64) -> HashMap<String, Vec<MetricDataPoint>> {
        let metrics = self.metrics.read().unwrap();
        let mut anomalies = HashMap::new();

        for (name, metric) in metrics.iter() {
            let anomalous_points = metric.detect_anomalies(threshold_multiplier);
            if !anomalous_points.is_empty() {
                anomalies.insert(
                    name.clone(),
                    anomalous_points.into_iter().cloned().collect()
                );
            }
        }

        anomalies
    }

    /// Get usage patterns for all metrics
    pub fn get_usage_patterns(&self) -> HashMap<String, UsagePattern> {
        let metrics = self.metrics.read().unwrap();
        let mut patterns = HashMap::new();

        for (name, metric) in metrics.iter() {
            patterns.insert(name.clone(), metric.get_trend());
        }

        patterns
    }

    /// Add SLA definition
    pub fn add_sla_definition(&self, sla: SlaDefinition) {
        let mut slas = self.sla_definitions.write().unwrap();
        slas.push(sla);
    }

    /// Check SLA compliance
    pub fn check_sla_compliance(&self) -> Vec<SlaResult> {
        let slas = self.sla_definitions.read().unwrap();
        let metrics = self.metrics.read().unwrap();
        let mut results = Vec::new();

        for sla in slas.iter() {
            if let Some(metric) = metrics.get(&sla.metric_name) {
                let end_time = Utc::now();
                let start_time = end_time - chrono::Duration::from_std(sla.measurement_window).unwrap();
                let points = metric.get_points_in_range(start_time, end_time);

                let mut violations = 0u64;
                let total_measurements = points.len() as u64;
                
                for point in &points {
                    let violates = match sla.operator {
                        SlaOperator::LessThan => point.value >= sla.threshold,
                        SlaOperator::LessThanOrEqual => point.value > sla.threshold,
                        SlaOperator::GreaterThan => point.value <= sla.threshold,
                        SlaOperator::GreaterThanOrEqual => point.value < sla.threshold,
                        SlaOperator::Equal => (point.value - sla.threshold).abs() > f64::EPSILON,
                    };

                    if violates {
                        violations += 1;
                    }
                }

                let compliance_percentage = if total_measurements > 0 {
                    100.0 * (total_measurements - violations) as f64 / total_measurements as f64
                } else {
                    100.0
                };

                results.push(SlaResult {
                    sla_name: sla.name.clone(),
                    measurement_period_start: start_time,
                    measurement_period_end: end_time,
                    compliance_percentage,
                    violations_count: violations,
                    total_measurements,
                    is_compliant: compliance_percentage >= sla.target_percentage,
                    breach_duration: Duration::from_secs(violations * 60), // Simplified
                });
            }
        }

        results
    }

    /// Add alert rule
    pub fn add_alert_rule(&self, rule: MetricAlertRule) {
        let mut rules = self.alert_rules.write().unwrap();
        rules.push(rule);
    }

    /// Check alert rules for a specific metric
    fn check_alert_rules(&self, metric_name: &str, current_value: f64) {
        let rules = self.alert_rules.read().unwrap();
        let mut active_alerts = self.active_alerts.write().unwrap();

        for rule in rules.iter() {
            if !rule.enabled || rule.metric_name != metric_name {
                continue;
            }

            let should_trigger = match rule.operator {
                SlaOperator::LessThan => current_value < rule.threshold,
                SlaOperator::LessThanOrEqual => current_value <= rule.threshold,
                SlaOperator::GreaterThan => current_value > rule.threshold,
                SlaOperator::GreaterThanOrEqual => current_value >= rule.threshold,
                SlaOperator::Equal => (current_value - rule.threshold).abs() < f64::EPSILON,
            };

            if should_trigger {
                // Check if alert already exists and is within cooldown
                let existing_alert = active_alerts.iter().any(|alert| {
                    alert.rule_name == rule.name && 
                    alert.resolved_at.is_none() &&
                    (Utc::now() - alert.triggered_at).to_std().unwrap_or(Duration::ZERO) < rule.cooldown_period
                });

                if !existing_alert {
                    let alert = MetricAlert {
                        id: uuid::Uuid::new_v4().to_string(),
                        rule_name: rule.name.clone(),
                        tenant_id: self.tenant_id.clone(),
                        metric_name: metric_name.to_string(),
                        current_value,
                        threshold: rule.threshold,
                        severity: rule.severity.clone(),
                        message: format!(
                            "Metric {} {} {} (current: {}, threshold: {})",
                            metric_name,
                            match rule.operator {
                                SlaOperator::LessThan => "is less than",
                                SlaOperator::LessThanOrEqual => "is less than or equal to",
                                SlaOperator::GreaterThan => "is greater than",
                                SlaOperator::GreaterThanOrEqual => "is greater than or equal to",
                                SlaOperator::Equal => "equals",
                            },
                            rule.threshold,
                            current_value,
                            rule.threshold
                        ),
                        triggered_at: Utc::now(),
                        resolved_at: None,
                        acknowledged: false,
                    };

                    active_alerts.push(alert);
                }
            }
        }
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<MetricAlert> {
        let active_alerts = self.active_alerts.read().unwrap();
        active_alerts.clone()
    }

    /// Acknowledge alert
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().unwrap();
        if let Some(alert) = active_alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(EventualiError::Tenant(format!("Alert not found: {alert_id}")))
        }
    }

    /// Resolve alert
    pub fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().unwrap();
        if let Some(alert) = active_alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved_at = Some(Utc::now());
            Ok(())
        } else {
            Err(EventualiError::Tenant(format!("Alert not found: {alert_id}")))
        }
    }

    /// Create dashboard
    pub fn create_dashboard(&self, dashboard: TenantDashboard) {
        let mut dashboards = self.dashboards.write().unwrap();
        dashboards.push(dashboard);
    }

    /// Get dashboards
    pub fn get_dashboards(&self) -> Vec<TenantDashboard> {
        let dashboards = self.dashboards.read().unwrap();
        dashboards.clone()
    }

    /// Generate dashboard data
    pub fn generate_dashboard_data(&self, dashboard_name: &str) -> Option<DashboardData> {
        let dashboards = self.dashboards.read().unwrap();
        let dashboard = dashboards.iter().find(|d| d.name == dashboard_name)?;

        let mut widget_data = HashMap::new();
        
        for widget in &dashboard.widgets {
            let time_range = chrono::Duration::from_std(widget.time_range).ok()?;
            let start_time = Utc::now() - time_range;
            let end_time = Utc::now();

            let mut data = Vec::new();
            for metric_name in &widget.metric_names {
                if let Some(timeseries) = self.get_metric_timeseries(metric_name, Some(start_time), Some(end_time)) {
                    data.push((metric_name.clone(), timeseries));
                }
            }

            widget_data.insert(widget.id.clone(), data);
        }

        Some(DashboardData {
            dashboard_name: dashboard_name.to_string(),
            generated_at: Utc::now(),
            widget_data,
        })
    }

    /// Export metrics data for external systems
    pub fn export_metrics(&self, format: ExportFormat, time_range: Option<(DateTime<Utc>, DateTime<Utc>)>) -> Result<String> {
        let metrics = self.metrics.read().unwrap();
        
        match format {
            ExportFormat::Json => {
                let mut export_data = HashMap::new();
                
                for (name, metric) in metrics.iter() {
                    let points = if let Some((start, end)) = time_range {
                        metric.get_points_in_range(start, end)
                    } else {
                        metric.get_points()
                    };
                    
                    export_data.insert(name, points.into_iter().cloned().collect::<Vec<_>>());
                }
                
                Ok(serde_json::to_string_pretty(&export_data)?)
            },
            ExportFormat::Csv => {
                let mut csv_data = String::new();
                csv_data.push_str("metric_name,timestamp,value,labels\n");
                
                for (name, metric) in metrics.iter() {
                    let points = if let Some((start, end)) = time_range {
                        metric.get_points_in_range(start, end)
                    } else {
                        metric.get_points()
                    };
                    
                    for point in points {
                        let labels_str = if point.labels.is_empty() {
                            String::new()
                        } else {
                            serde_json::to_string(&point.labels).unwrap_or_default()
                        };
                        
                        csv_data.push_str(&format!(
                            "{},{},{},{}\n",
                            name,
                            point.timestamp.to_rfc3339(),
                            point.value,
                            labels_str
                        ));
                    }
                }
                
                Ok(csv_data)
            },
            ExportFormat::Prometheus => {
                let mut prom_data = String::new();
                
                for (name, metric) in metrics.iter() {
                    if let Some(latest) = metric.get_latest() {
                        let metric_name = name.replace(['-', ' '], "_");
                        
                        if latest.labels.is_empty() {
                            prom_data.push_str(&format!("{} {}\n", metric_name, latest.value));
                        } else {
                            let labels: Vec<String> = latest.labels.iter()
                                .map(|(k, v)| format!("{k}=\"{v}\""))
                                .collect();
                            prom_data.push_str(&format!(
                                "{}{{{}}} {}\n",
                                metric_name,
                                labels.join(","),
                                latest.value
                            ));
                        }
                    }
                }
                
                Ok(prom_data)
            },
        }
    }

    /// Calculate comprehensive tenant health score
    pub fn calculate_health_score(&self) -> TenantHealthScore {
        let now = Utc::now();
        let _last_hour = now - chrono::Duration::hours(1);
        
        // Get key metrics for health calculation
        let error_rate = self.get_current_metric_value("error_rate").unwrap_or(0.0);
        let response_time = self.get_current_metric_value("response_time_ms").unwrap_or(0.0);
        let cpu_usage = self.get_current_metric_value("cpu_usage_percent").unwrap_or(0.0);
        let memory_usage = self.get_current_metric_value("memory_usage_percent").unwrap_or(0.0);
        let storage_usage = self.get_current_metric_value("storage_usage_percent").unwrap_or(0.0);
        
        // Calculate individual component scores (0-100)
        let error_score = (100.0 - (error_rate * 100.0)).clamp(0.0, 100.0);
        let performance_score = if response_time > 1000.0 {
            (1000.0 / response_time * 100.0).min(100.0)
        } else {
            100.0
        };
        let cpu_score = (100.0 - cpu_usage).clamp(0.0, 100.0);
        let memory_score = (100.0 - memory_usage).clamp(0.0, 100.0);
        let storage_score = (100.0 - storage_usage).clamp(0.0, 100.0);
        
        // Calculate SLA compliance score
        let sla_results = self.check_sla_compliance();
        let sla_score = if sla_results.is_empty() {
            100.0
        } else {
            sla_results.iter()
                .map(|r| r.compliance_percentage)
                .sum::<f64>() / sla_results.len() as f64
        };
        
        // Calculate active alerts impact
        let active_alerts = self.get_active_alerts();
        let alert_penalty = active_alerts.iter()
            .map(|alert| match alert.severity {
                AlertType::Critical => 20.0,
                AlertType::Exceeded => 15.0,
                AlertType::Warning => 5.0,
                AlertType::Violation => 25.0,
            })
            .sum::<f64>();
        
        // Weighted overall score
        let base_score = error_score * 0.25 +
            performance_score * 0.20 +
            cpu_score * 0.15 +
            memory_score * 0.15 +
            storage_score * 0.10 +
            sla_score * 0.15;
        
        let overall_score = (base_score - alert_penalty).clamp(0.0, 100.0);
        
        // Determine health status
        let status = if overall_score >= 90.0 {
            HealthStatus::Excellent
        } else if overall_score >= 75.0 {
            HealthStatus::Good
        } else if overall_score >= 60.0 {
            HealthStatus::Fair
        } else if overall_score >= 40.0 {
            HealthStatus::Poor
        } else {
            HealthStatus::Critical
        };
        
        TenantHealthScore {
            overall_score,
            status,
            component_scores: HashMap::from([
                ("error_rate".to_string(), error_score),
                ("performance".to_string(), performance_score),
                ("cpu_usage".to_string(), cpu_score),
                ("memory_usage".to_string(), memory_score),
                ("storage_usage".to_string(), storage_score),
                ("sla_compliance".to_string(), sla_score),
            ]),
            active_alerts_count: active_alerts.len(),
            critical_alerts_count: active_alerts.iter().filter(|a| matches!(a.severity, AlertType::Critical | AlertType::Violation)).count(),
            calculated_at: now,
            recommendations: self.generate_health_recommendations(overall_score, &active_alerts),
        }
    }

    /// Generate health recommendations based on current state
    fn generate_health_recommendations(&self, score: f64, alerts: &[MetricAlert]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if score < 60.0 {
            recommendations.push("🚨 Critical: Immediate attention required - system health is below acceptable levels".to_string());
        }
        
        if alerts.iter().any(|a| matches!(a.severity, AlertType::Critical)) {
            recommendations.push("🔴 Address critical alerts immediately to prevent service degradation".to_string());
        }
        
        if self.get_current_metric_value("error_rate").unwrap_or(0.0) > 0.05 {
            recommendations.push("📈 High error rate detected - investigate failing operations".to_string());
        }
        
        if self.get_current_metric_value("response_time_ms").unwrap_or(0.0) > 1000.0 {
            recommendations.push("🐌 Slow response times detected - consider performance optimization".to_string());
        }
        
        let cpu_usage = self.get_current_metric_value("cpu_usage_percent").unwrap_or(0.0);
        if cpu_usage > 80.0 {
            recommendations.push("💻 High CPU usage - consider scaling up or optimizing workload".to_string());
        }
        
        let memory_usage = self.get_current_metric_value("memory_usage_percent").unwrap_or(0.0);
        if memory_usage > 85.0 {
            recommendations.push("🧠 High memory usage - check for memory leaks or increase allocation".to_string());
        }
        
        let storage_usage = self.get_current_metric_value("storage_usage_percent").unwrap_or(0.0);
        if storage_usage > 90.0 {
            recommendations.push("💾 Storage nearly full - archive old data or increase storage capacity".to_string());
        }
        
        if score >= 90.0 && alerts.is_empty() {
            recommendations.push("✅ System is operating optimally - maintain current configuration".to_string());
        }
        
        recommendations
    }
}

/// Export formats for metrics data
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
}

/// Dashboard data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub dashboard_name: String,
    pub generated_at: DateTime<Utc>,
    pub widget_data: HashMap<String, Vec<(String, Vec<MetricDataPoint>)>>,
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Comprehensive tenant health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantHealthScore {
    pub overall_score: f64,
    pub status: HealthStatus,
    pub component_scores: HashMap<String, f64>,
    pub active_alerts_count: usize,
    pub critical_alerts_count: usize,
    pub calculated_at: DateTime<Utc>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_metric() {
        let mut metric = TimeSeriesMetric::new("test_metric".to_string(), 100, 1);
        
        metric.add_point(MetricDataPoint::new(10.0));
        metric.add_point(MetricDataPoint::new(20.0));
        metric.add_point(MetricDataPoint::new(15.0));
        
        assert_eq!(metric.calculate_average(), 15.0);
        assert_eq!(metric.get_points().len(), 3);
    }

    #[test]
    fn test_tenant_metrics_collector() {
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        let collector = TenantMetricsCollector::new(tenant_id);
        
        collector.record_metric("cpu_usage".to_string(), 45.0, None);
        collector.record_metric("memory_usage".to_string(), 60.0, None);
        
        assert_eq!(collector.get_current_metric_value("cpu_usage"), Some(45.0));
        assert_eq!(collector.get_current_metric_value("memory_usage"), Some(60.0));
    }

    #[test]
    fn test_aggregated_metric() {
        let points = vec![
            &MetricDataPoint::new(10.0),
            &MetricDataPoint::new(20.0),
            &MetricDataPoint::new(30.0),
            &MetricDataPoint::new(40.0),
        ];
        
        let agg = AggregatedMetric::from_points("test".to_string(), &points);
        assert_eq!(agg.min, 10.0);
        assert_eq!(agg.max, 40.0);
        assert_eq!(agg.avg, 25.0);
        assert_eq!(agg.count, 4);
    }
}