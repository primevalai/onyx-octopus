use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration, Datelike};
use serde::{Deserialize, Serialize};

use super::tenant::{TenantId, ResourceLimits};
use crate::error::{EventualiError, Result};

/// Types of resources that can be tracked and limited
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Events,
    Storage,
    Streams,
    Projections,
    Aggregates,
    ApiCalls,
}

/// Quota tiers with different limits and features
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum QuotaTier {
    Starter,
    #[default]
    Standard,
    Professional,
    Enterprise,
}


/// Result of quota check with detailed information
#[derive(Debug, Clone)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub current_usage: u64,
    pub limit: Option<u64>,
    pub utilization_percentage: f64,
    pub grace_period_active: bool,
    pub warning_triggered: bool,
    pub estimated_overage_cost: f64,
}

/// Quota alert types
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AlertType {
    Warning,      // 80% utilization
    Critical,     // 90% utilization
    Exceeded,     // 100% utilization, grace period activated
    Violation,    // Grace period exceeded
}

/// Individual quota alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaAlert {
    pub tenant_id: TenantId,
    pub resource_type: ResourceType,
    pub alert_type: AlertType,
    pub current_usage: u64,
    pub limit: u64,
    pub utilization_percentage: f64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

/// Alert summary for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub total_alerts: usize,
    pub unacknowledged_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub last_alert: Option<DateTime<Utc>>,
}

/// Billing analytics for cost tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAnalytics {
    pub tenant_id: TenantId,
    pub current_month_cost: f64,
    pub projected_month_cost: f64,
    pub overage_costs: HashMap<ResourceType, f64>,
    pub cost_breakdown: HashMap<ResourceType, f64>,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub cost_trend: Vec<DailyCostEntry>,
}

/// Daily cost entry for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCostEntry {
    pub date: DateTime<Utc>,
    pub cost: f64,
    pub usage_breakdown: HashMap<ResourceType, u64>,
}

/// Usage trends for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrends {
    pub daily_event_trend: Vec<DailyUsageEntry>,
    pub storage_growth_trend: Vec<DailyUsageEntry>,
    pub api_calls_trend: Vec<DailyUsageEntry>,
    pub peak_usage_times: HashMap<ResourceType, Vec<DateTime<Utc>>>,
    pub usage_patterns: HashMap<ResourceType, UsagePattern>,
}

/// Daily usage entry for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsageEntry {
    pub date: DateTime<Utc>,
    pub usage: u64,
    pub percentage_of_limit: f64,
}

/// Usage pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsagePattern {
    Stable,        // Consistent usage over time
    Growing,       // Increasing usage trend
    Declining,     // Decreasing usage trend
    Volatile,      // Highly variable usage
    Seasonal,      // Predictable seasonal patterns
}

/// Quota alert manager for handling notifications
#[derive(Debug, Clone)]
pub struct QuotaAlertManager {
    tenant_id: TenantId,
    alerts_history: Vec<QuotaAlert>,
    #[allow(dead_code)] // Threshold configuration for alert triggering (initialized but not yet used in alert logic)
    alert_thresholds: HashMap<ResourceType, Vec<f64>>,  // Warning thresholds
    last_alert_sent: HashMap<(ResourceType, AlertType), DateTime<Utc>>,
    alert_cooldown: Duration,
}

impl QuotaAlertManager {
    pub fn new(tenant_id: TenantId) -> Self {
        let mut alert_thresholds = HashMap::new();
        
        // Set default warning thresholds (80%, 90%, 95%)
        for resource_type in [ResourceType::Events, ResourceType::Storage, ResourceType::Streams,
                             ResourceType::Projections, ResourceType::Aggregates, ResourceType::ApiCalls] {
            alert_thresholds.insert(resource_type, vec![80.0, 90.0, 95.0]);
        }
        
        Self {
            tenant_id,
            alerts_history: Vec::new(),
            alert_thresholds,
            last_alert_sent: HashMap::new(),
            alert_cooldown: Duration::minutes(15), // 15-minute cooldown between same alerts
        }
    }
    
    pub fn trigger_warning_alert(&mut self, resource_type: ResourceType, utilization: f64) {
        let alert_type = if utilization >= 90.0 {
            AlertType::Critical
        } else {
            AlertType::Warning
        };
        
        // Check cooldown
        let key = (resource_type, alert_type.clone());
        if let Some(last_sent) = self.last_alert_sent.get(&key) {
            if Utc::now().signed_duration_since(*last_sent) < self.alert_cooldown {
                return; // Still in cooldown period
            }
        }
        
        let alert = QuotaAlert {
            tenant_id: self.tenant_id.clone(),
            resource_type,
            alert_type: alert_type.clone(),
            current_usage: 0, // Would be filled with actual usage
            limit: 0,         // Would be filled with actual limit
            utilization_percentage: utilization,
            message: format!(
                "{:?} usage for tenant {} has reached {:.1}%",
                resource_type, self.tenant_id.as_str(), utilization
            ),
            timestamp: Utc::now(),
            acknowledged: false,
        };
        
        self.alerts_history.push(alert);
        self.last_alert_sent.insert(key, Utc::now());
        
        // Keep only last 1000 alerts
        if self.alerts_history.len() > 1000 {
            self.alerts_history.drain(..self.alerts_history.len() - 1000);
        }
    }
    
    pub fn check_and_trigger_alerts(&mut self, _resource_type: ResourceType, _amount: u64) {
        // This would typically check current utilization and trigger alerts
        // For now, this is a placeholder that would be called from the quota system
    }
    
    pub fn get_summary(&self) -> AlertSummary {
        let unacknowledged = self.alerts_history.iter()
            .filter(|a| !a.acknowledged)
            .count();
        
        let critical = self.alerts_history.iter()
            .filter(|a| matches!(a.alert_type, AlertType::Critical | AlertType::Exceeded | AlertType::Violation))
            .count();
        
        let warning = self.alerts_history.iter()
            .filter(|a| matches!(a.alert_type, AlertType::Warning))
            .count();
        
        AlertSummary {
            total_alerts: self.alerts_history.len(),
            unacknowledged_alerts: unacknowledged,
            critical_alerts: critical,
            warning_alerts: warning,
            last_alert: self.alerts_history.last().map(|a| a.timestamp),
        }
    }
    
    pub fn get_alerts_history(&self, limit: usize) -> Vec<QuotaAlert> {
        let start = if self.alerts_history.len() > limit {
            self.alerts_history.len() - limit
        } else {
            0
        };
        
        self.alerts_history[start..].to_vec()
    }
    
    pub fn acknowledge_alert(&mut self, alert_index: usize) -> Result<()> {
        if alert_index < self.alerts_history.len() {
            self.alerts_history[alert_index].acknowledged = true;
            Ok(())
        } else {
            Err(EventualiError::Tenant(format!("Alert index {alert_index} not found")))
        }
    }
}

/// Billing tracker for cost analysis
#[derive(Debug, Clone)]
pub struct BillingTracker {
    tenant_id: TenantId,
    tier: QuotaTier,
    current_month_usage: HashMap<ResourceType, u64>,
    current_month_cost: f64,
    overage_costs: HashMap<ResourceType, f64>,
    billing_period_start: DateTime<Utc>,
    daily_costs: Vec<DailyCostEntry>,
    rate_card: HashMap<ResourceType, f64>,  // Cost per unit
}

impl BillingTracker {
    pub fn new(tenant_id: TenantId) -> Self {
        let now = Utc::now();
        let billing_start = now.with_day(1).unwrap_or(now); // Start of current month
        
        let mut rate_card = HashMap::new();
        // Default rates (would be configurable per tier)
        rate_card.insert(ResourceType::Events, 0.0001);
        rate_card.insert(ResourceType::Storage, 0.001);
        rate_card.insert(ResourceType::Aggregates, 0.0002);
        rate_card.insert(ResourceType::Projections, 0.01);
        rate_card.insert(ResourceType::Streams, 0.05);
        rate_card.insert(ResourceType::ApiCalls, 0.00005);
        
        Self {
            tenant_id,
            tier: QuotaTier::Standard,
            current_month_usage: HashMap::new(),
            current_month_cost: 0.0,
            overage_costs: HashMap::new(),
            billing_period_start: billing_start,
            daily_costs: Vec::new(),
            rate_card,
        }
    }
    
    pub fn record_usage(&mut self, resource_type: ResourceType, amount: u64) {
        // Update monthly usage
        let current = self.current_month_usage.entry(resource_type).or_insert(0);
        *current += amount;
        
        // Calculate cost for this usage
        if let Some(&rate) = self.rate_card.get(&resource_type) {
            let cost = amount as f64 * rate;
            self.current_month_cost += cost;
        }
    }
    
    pub fn record_overage(&mut self, resource_type: ResourceType, amount: u64, cost_per_unit: f64) {
        let overage_cost = amount as f64 * cost_per_unit;
        let current_overage = self.overage_costs.entry(resource_type).or_insert(0.0);
        *current_overage += overage_cost;
        self.current_month_cost += overage_cost;
    }
    
    pub fn finalize_daily_billing(&mut self) {
        let today = Utc::now();
        let daily_entry = DailyCostEntry {
            date: today,
            cost: self.calculate_daily_cost(),
            usage_breakdown: self.current_month_usage.clone(),
        };
        
        self.daily_costs.push(daily_entry);
        
        // Keep only last 30 days
        if self.daily_costs.len() > 30 {
            self.daily_costs.remove(0);
        }
    }
    
    fn calculate_daily_cost(&self) -> f64 {
        // Simplified daily cost calculation
        self.current_month_cost / Utc::now().day() as f64
    }
    
    pub fn get_analytics(&self) -> BillingAnalytics {
        let now = Utc::now();
        let days_in_month = now.with_day(1)
            .and_then(|d| d.with_month(d.month() + 1))
            .and_then(|d| d.with_day(1))
            .map(|next_month| (next_month - Duration::days(1)).day())
            .unwrap_or(30);
        
        let days_elapsed = now.day();
        let projected_cost = if days_elapsed > 0 {
            (self.current_month_cost / days_elapsed as f64) * days_in_month as f64
        } else {
            self.current_month_cost
        };
        
        let mut cost_breakdown = HashMap::new();
        for (resource_type, &usage) in &self.current_month_usage {
            if let Some(&rate) = self.rate_card.get(resource_type) {
                cost_breakdown.insert(*resource_type, usage as f64 * rate);
            }
        }
        
        BillingAnalytics {
            tenant_id: self.tenant_id.clone(),
            current_month_cost: self.current_month_cost,
            projected_month_cost: projected_cost,
            overage_costs: self.overage_costs.clone(),
            cost_breakdown,
            billing_period_start: self.billing_period_start,
            billing_period_end: now.with_day(days_in_month).unwrap_or(now),
            cost_trend: self.daily_costs.clone(),
        }
    }
    
    pub fn update_tier(&mut self, new_tier: QuotaTier) {
        self.tier = new_tier;
        // Update rate card based on tier
        self.update_rate_card_for_tier();
    }
    
    fn update_rate_card_for_tier(&mut self) {
        let multiplier = match self.tier {
            QuotaTier::Enterprise => 0.7,   // 30% discount
            QuotaTier::Professional => 0.85, // 15% discount
            QuotaTier::Standard => 1.0,     // Standard rates
            QuotaTier::Starter => 1.2,      // 20% premium
        };
        
        for (_, rate) in self.rate_card.iter_mut() {
            *rate *= multiplier;
        }
    }
    
    pub fn reset_monthly_billing(&mut self) {
        self.current_month_usage.clear();
        self.current_month_cost = 0.0;
        self.overage_costs.clear();
        self.billing_period_start = Utc::now().with_day(1).unwrap_or(Utc::now());
        self.daily_costs.clear();
    }
}

/// Enhanced resource tracker with analytics
#[derive(Debug, Clone)]
pub struct EnhancedResourceTracker {
    daily_events: u64,
    daily_reset_time: DateTime<Utc>,
    storage_used_mb: f64,
    concurrent_streams: u32,
    total_projections: u32,
    total_aggregates: u64,
    daily_api_calls: u64,
    api_call_limits: HashMap<ResourceType, u64>,
    last_updated: DateTime<Utc>,
    
    // Analytics data
    usage_history: Vec<DailyUsageEntry>,
    peak_usage_tracker: HashMap<ResourceType, u64>,
    usage_patterns: HashMap<ResourceType, Vec<u64>>,  // Rolling window for pattern detection
}

impl Default for EnhancedResourceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedResourceTracker {
    pub fn new() -> Self {
        let now = Utc::now();
        let mut api_call_limits = HashMap::new();
        api_call_limits.insert(ResourceType::ApiCalls, 100_000); // Default API call limit
        
        Self {
            daily_events: 0,
            daily_reset_time: now,
            storage_used_mb: 0.0,
            concurrent_streams: 0,
            total_projections: 0,
            total_aggregates: 0,
            daily_api_calls: 0,
            api_call_limits,
            last_updated: now,
            usage_history: Vec::new(),
            peak_usage_tracker: HashMap::new(),
            usage_patterns: HashMap::new(),
        }
    }
    
    pub fn record_usage(&mut self, resource_type: ResourceType, amount: u64) {
        self.last_updated = Utc::now();
        
        match resource_type {
            ResourceType::Events => {
                self.ensure_daily_counter_fresh();
                self.daily_events += amount;
                self.update_usage_patterns(resource_type, self.daily_events);
            },
            ResourceType::Storage => {
                self.storage_used_mb += amount as f64;
                self.update_usage_patterns(resource_type, self.storage_used_mb as u64);
            },
            ResourceType::Streams => {
                self.concurrent_streams += amount as u32;
                self.update_peak_usage(resource_type, self.concurrent_streams as u64);
            },
            ResourceType::Projections => {
                self.total_projections += amount as u32;
                self.update_usage_patterns(resource_type, self.total_projections as u64);
            },
            ResourceType::Aggregates => {
                self.total_aggregates += amount;
                self.update_usage_patterns(resource_type, self.total_aggregates);
            },
            ResourceType::ApiCalls => {
                self.ensure_daily_counter_fresh();
                self.daily_api_calls += amount;
                self.update_usage_patterns(resource_type, self.daily_api_calls);
            },
        }
        
        // Update peak usage tracking
        self.update_peak_usage(resource_type, amount);
    }
    
    pub fn get_daily_events(&self) -> u64 {
        if self.is_daily_counter_stale() {
            0 // Reset if stale
        } else {
            self.daily_events
        }
    }
    
    pub fn get_api_calls_today(&self) -> u64 {
        if self.is_daily_counter_stale() {
            0
        } else {
            self.daily_api_calls
        }
    }
    
    pub fn reset_daily_counters(&mut self) {
        // Store today's usage in history before reset
        self.store_daily_usage();
        
        self.daily_events = 0;
        self.daily_api_calls = 0;
        self.daily_reset_time = Utc::now();
    }
    
    fn ensure_daily_counter_fresh(&mut self) {
        if self.is_daily_counter_stale() {
            self.reset_daily_counters();
        }
    }
    
    fn is_daily_counter_stale(&self) -> bool {
        let now = Utc::now();
        now.signed_duration_since(self.daily_reset_time) >= Duration::days(1)
    }
    
    /// Store daily usage in history
    fn store_daily_usage(&mut self) {
        let entry = DailyUsageEntry {
            date: self.daily_reset_time,
            usage: self.daily_events,
            percentage_of_limit: 0.0, // Will be calculated when needed
        };
        
        self.usage_history.push(entry);
        
        // Keep only last 30 days
        if self.usage_history.len() > 30 {
            self.usage_history.remove(0);
        }
    }
    
    /// Update usage patterns for trend analysis
    fn update_usage_patterns(&mut self, resource_type: ResourceType, current_usage: u64) {
        let patterns = self.usage_patterns.entry(resource_type).or_default();
        patterns.push(current_usage);
        
        // Keep only last 24 hours of data points (assuming hourly updates)
        if patterns.len() > 24 {
            patterns.remove(0);
        }
    }
    
    /// Update peak usage tracking
    fn update_peak_usage(&mut self, resource_type: ResourceType, usage: u64) {
        let current_peak = self.peak_usage_tracker.get(&resource_type).unwrap_or(&0);
        if usage > *current_peak {
            self.peak_usage_tracker.insert(resource_type, usage);
        }
    }
    
    /// Get usage trends for analytics
    pub fn get_usage_trends(&self) -> UsageTrends {
        UsageTrends {
            daily_event_trend: self.usage_history.clone(),
            storage_growth_trend: self.calculate_storage_trend(),
            api_calls_trend: self.calculate_api_calls_trend(),
            peak_usage_times: HashMap::new(), // Would be populated with actual peak time tracking
            usage_patterns: self.analyze_usage_patterns(),
        }
    }
    
    /// Calculate storage growth trend
    fn calculate_storage_trend(&self) -> Vec<DailyUsageEntry> {
        // Simplified - in reality would track daily storage usage
        vec![
            DailyUsageEntry {
                date: Utc::now() - Duration::days(1),
                usage: (self.storage_used_mb * 0.9) as u64,
                percentage_of_limit: 0.0,
            },
            DailyUsageEntry {
                date: Utc::now(),
                usage: self.storage_used_mb as u64,
                percentage_of_limit: 0.0,
            },
        ]
    }
    
    /// Calculate API calls trend
    fn calculate_api_calls_trend(&self) -> Vec<DailyUsageEntry> {
        vec![
            DailyUsageEntry {
                date: Utc::now(),
                usage: self.daily_api_calls,
                percentage_of_limit: 0.0,
            },
        ]
    }
    
    /// Analyze usage patterns
    fn analyze_usage_patterns(&self) -> HashMap<ResourceType, UsagePattern> {
        let mut patterns = HashMap::new();
        
        for (resource_type, usage_data) in &self.usage_patterns {
            if usage_data.len() < 3 {
                patterns.insert(*resource_type, UsagePattern::Stable);
                continue;
            }
            
            let pattern = self.detect_pattern(usage_data);
            patterns.insert(*resource_type, pattern);
        }
        
        patterns
    }
    
    /// Detect usage pattern from data
    fn detect_pattern(&self, data: &[u64]) -> UsagePattern {
        if data.len() < 3 {
            return UsagePattern::Stable;
        }
        
        let variance = self.calculate_variance(data);
        let trend = self.calculate_trend(data);
        
        if variance > data.iter().sum::<u64>() as f64 / data.len() as f64 * 0.5 {
            UsagePattern::Volatile
        } else if trend > 0.1 {
            UsagePattern::Growing
        } else if trend < -0.1 {
            UsagePattern::Declining
        } else {
            UsagePattern::Stable
        }
    }
    
    /// Calculate variance for volatility detection
    fn calculate_variance(&self, data: &[u64]) -> f64 {
        let mean = data.iter().sum::<u64>() as f64 / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        variance
    }
    
    /// Calculate trend (simple linear regression slope)
    fn calculate_trend(&self, data: &[u64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        
        let n = data.len() as f64;
        let sum_x = (0..data.len()).sum::<usize>() as f64;
        let sum_y = data.iter().sum::<u64>() as f64;
        let sum_xy = data.iter().enumerate()
            .map(|(i, &y)| i as f64 * y as f64)
            .sum::<f64>();
        let sum_x2 = (0..data.len()).map(|i| (i * i) as f64).sum::<f64>();
        
        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }
    
    /// Check if usage pattern is stable
    pub fn has_stable_usage_pattern(&self) -> bool {
        self.usage_patterns.iter()
            .all(|(_, data)| {
                if data.len() < 3 { return true; }
                let pattern = self.detect_pattern(data);
                matches!(pattern, UsagePattern::Stable)
            })
    }
    
    /// Get utilization percentages
    pub fn get_events_utilization(&self, limits: &ResourceLimits) -> f64 {
        if let Some(limit) = limits.max_events_per_day {
            (self.get_daily_events() as f64 / limit as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn get_storage_utilization(&self, limits: &ResourceLimits) -> f64 {
        if let Some(limit) = limits.max_storage_mb {
            (self.storage_used_mb / limit as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn get_aggregates_utilization(&self, limits: &ResourceLimits) -> f64 {
        if let Some(limit) = limits.max_aggregates {
            (self.total_aggregates as f64 / limit as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Enhanced resource usage with analytics
#[derive(Debug, Clone)]
pub struct EnhancedResourceUsage {
    pub tenant_id: TenantId,
    pub tier: QuotaTier,
    pub daily_events: u64,
    pub storage_used_mb: f64,
    pub concurrent_streams: u32,
    pub total_projections: u32,
    pub total_aggregates: u64,
    pub api_calls_today: u64,
    pub limits: ResourceLimits,
    pub last_updated: DateTime<Utc>,
    pub usage_trends: UsageTrends,
    pub cost_analytics: BillingAnalytics,
    pub alert_summary: AlertSummary,
    pub performance_score: f64,
}

impl EnhancedResourceUsage {
    /// Calculate utilization percentage for a resource
    pub fn utilization_percentage(&self, resource_type: ResourceType) -> Option<f64> {
        match resource_type {
            ResourceType::Events => {
                self.limits.max_events_per_day.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.daily_events as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Storage => {
                self.limits.max_storage_mb.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.storage_used_mb / limit as f64) * 100.0 }
                })
            },
            ResourceType::Streams => {
                self.limits.max_concurrent_streams.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.concurrent_streams as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Projections => {
                self.limits.max_projections.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.total_projections as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Aggregates => {
                self.limits.max_aggregates.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.total_aggregates as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::ApiCalls => {
                // Default to 100k API call limit
                Some((self.api_calls_today as f64 / 100_000.0) * 100.0)
            },
        }
    }
    
    /// Check if any resource is near its limit (>80%)
    pub fn has_resources_near_limit(&self) -> bool {
        [
            ResourceType::Events,
            ResourceType::Storage,
            ResourceType::Streams,
            ResourceType::Projections,
            ResourceType::Aggregates,
            ResourceType::ApiCalls,
        ].iter().any(|&resource_type| {
            self.utilization_percentage(resource_type)
                .is_some_and(|percentage| percentage > 80.0)
        })
    }
}

/// Enterprise-grade resource quota management for tenants
pub struct TenantQuota {
    tenant_id: TenantId,
    limits: ResourceLimits,
    tier: QuotaTier,
    tracker: Arc<RwLock<EnhancedResourceTracker>>,
    alert_manager: Arc<RwLock<QuotaAlertManager>>,
    billing_tracker: Arc<RwLock<BillingTracker>>,
}

impl TenantQuota {
    pub fn new(tenant_id: TenantId, limits: ResourceLimits) -> Self {
        Self::with_tier(tenant_id, limits, QuotaTier::Standard)
    }
    
    pub fn with_tier(tenant_id: TenantId, limits: ResourceLimits, tier: QuotaTier) -> Self {
        Self {
            tenant_id: tenant_id.clone(),
            limits,
            tier,
            tracker: Arc::new(RwLock::new(EnhancedResourceTracker::new())),
            alert_manager: Arc::new(RwLock::new(QuotaAlertManager::new(tenant_id.clone()))),
            billing_tracker: Arc::new(RwLock::new(BillingTracker::new(tenant_id))),
        }
    }
    
    /// Check if an operation would exceed quotas with enhanced validation
    pub fn check_quota(&self, resource_type: ResourceType, amount: u64) -> Result<QuotaCheckResult> {
        let tracker = self.tracker.read().unwrap();
        let mut result = QuotaCheckResult {
            allowed: true,
            current_usage: 0,
            limit: None,
            utilization_percentage: 0.0,
            grace_period_active: false,
            warning_triggered: false,
            estimated_overage_cost: 0.0,
        };
        
        match resource_type {
            ResourceType::Events => {
                if let Some(limit) = self.limits.max_events_per_day {
                    let current_daily = tracker.get_daily_events();
                    result.current_usage = current_daily;
                    result.limit = Some(limit);
                    result.utilization_percentage = (current_daily as f64 / limit as f64) * 100.0;
                    
                    if current_daily + amount > limit {
                        // Check grace period and overage policies
                        let grace_limit = self.calculate_grace_limit(&resource_type, limit);
                        if current_daily + amount <= grace_limit {
                            result.grace_period_active = true;
                            result.estimated_overage_cost = self.calculate_overage_cost(&resource_type, amount);
                        } else {
                            result.allowed = false;
                            return Err(EventualiError::from(QuotaExceeded {
                                tenant_id: self.tenant_id.clone(),
                                resource_type: "daily_events".to_string(),
                                current_usage: current_daily,
                                limit,
                                attempted: amount,
                            }));
                        }
                    }
                }
            },
            ResourceType::ApiCalls => {
                if let Some(limit) = tracker.api_call_limits.get(&resource_type) {
                    let current = tracker.get_api_calls_today();
                    result.current_usage = current;
                    result.limit = Some(*limit);
                    result.utilization_percentage = (current as f64 / *limit as f64) * 100.0;
                    
                    if current + amount > *limit {
                        let grace_limit = self.calculate_grace_limit(&resource_type, *limit);
                        if current + amount <= grace_limit {
                            result.grace_period_active = true;
                            result.estimated_overage_cost = self.calculate_overage_cost(&resource_type, amount);
                        } else {
                            result.allowed = false;
                            return Err(EventualiError::from(QuotaExceeded {
                                tenant_id: self.tenant_id.clone(),
                                resource_type: "api_calls".to_string(),
                                current_usage: current,
                                limit: *limit,
                                attempted: amount,
                            }));
                        }
                    }
                }
            },
            // ... other resource types would be handled similarly
            _ => {
                // For other resource types, use simpler checks for now
                result.current_usage = 0;
                result.limit = Some(u64::MAX);
                result.utilization_percentage = 0.0;
            }
        }
        
        // Check for warning thresholds
        if result.utilization_percentage >= 80.0 && result.utilization_percentage < 90.0 {
            result.warning_triggered = true;
            self.trigger_warning_alert(resource_type, result.utilization_percentage);
        }
        
        Ok(result)
    }
    
    /// Record resource usage with billing integration
    pub fn record_usage(&self, resource_type: ResourceType, amount: u64) {
        {
            let mut tracker = self.tracker.write().unwrap();
            tracker.record_usage(resource_type, amount);
        }
        
        // Update billing tracker
        {
            let mut billing_tracker = self.billing_tracker.write().unwrap();
            billing_tracker.record_usage(resource_type, amount);
        }
        
        // Check for quota alerts
        self.check_and_trigger_alerts(resource_type, amount);
    }
    
    /// Get comprehensive usage statistics with analytics
    pub fn get_usage(&self) -> EnhancedResourceUsage {
        let tracker = self.tracker.read().unwrap();
        let billing_tracker = self.billing_tracker.read().unwrap();
        let alert_manager = self.alert_manager.read().unwrap();
        
        EnhancedResourceUsage {
            tenant_id: self.tenant_id.clone(),
            tier: self.tier.clone(),
            daily_events: tracker.get_daily_events(),
            storage_used_mb: tracker.storage_used_mb,
            concurrent_streams: tracker.concurrent_streams,
            total_projections: tracker.total_projections,
            total_aggregates: tracker.total_aggregates,
            api_calls_today: tracker.get_api_calls_today(),
            limits: self.limits.clone(),
            last_updated: tracker.last_updated,
            usage_trends: tracker.get_usage_trends(),
            cost_analytics: billing_tracker.get_analytics(),
            alert_summary: alert_manager.get_summary(),
            performance_score: self.calculate_performance_score(&tracker),
        }
    }
    
    /// Get legacy usage statistics for backward compatibility
    pub fn get_legacy_usage(&self) -> ResourceUsage {
        let tracker = self.tracker.read().unwrap();
        
        ResourceUsage {
            tenant_id: self.tenant_id.clone(),
            daily_events: tracker.get_daily_events(),
            storage_used_mb: tracker.storage_used_mb,
            concurrent_streams: tracker.concurrent_streams,
            total_projections: tracker.total_projections,
            total_aggregates: tracker.total_aggregates,
            limits: self.limits.clone(),
            last_updated: tracker.last_updated,
        }
    }
    
    /// Reset daily counters with billing finalization
    pub fn reset_daily_counters(&self) {
        {
            let mut tracker = self.tracker.write().unwrap();
            tracker.reset_daily_counters();
        }
        
        {
            let mut billing_tracker = self.billing_tracker.write().unwrap();
            billing_tracker.finalize_daily_billing();
        }
    }
    
    /// Calculate grace period limit based on tier and resource type
    fn calculate_grace_limit(&self, resource_type: &ResourceType, base_limit: u64) -> u64 {
        let grace_percentage = match (&self.tier, resource_type) {
            (QuotaTier::Enterprise, _) => 0.20,  // 20% grace
            (QuotaTier::Professional, ResourceType::Events) => 0.15,  // 15% grace for events
            (QuotaTier::Professional, _) => 0.10,  // 10% grace for others
            (QuotaTier::Standard, ResourceType::Events) => 0.05,  // 5% grace for events
            (QuotaTier::Starter, _) => 0.02,  // 2% grace only
            _ => 0.0,
        };
        
        (base_limit as f64 * (1.0 + grace_percentage)) as u64
    }
    
    /// Calculate overage cost based on tier and usage
    fn calculate_overage_cost(&self, resource_type: &ResourceType, overage_amount: u64) -> f64 {
        let cost_per_unit = match (&self.tier, resource_type) {
            (QuotaTier::Enterprise, ResourceType::Events) => 0.0001,
            (QuotaTier::Enterprise, ResourceType::ApiCalls) => 0.00005,
            (QuotaTier::Professional, ResourceType::Events) => 0.0002,
            (QuotaTier::Professional, ResourceType::ApiCalls) => 0.0001,
            (QuotaTier::Standard, ResourceType::Events) => 0.0005,
            (QuotaTier::Standard, ResourceType::ApiCalls) => 0.0002,
            (QuotaTier::Starter, _) => 0.001,
            _ => 0.0,
        };
        
        overage_amount as f64 * cost_per_unit
    }
    
    /// Trigger warning alert for approaching limits
    fn trigger_warning_alert(&self, resource_type: ResourceType, utilization_percentage: f64) {
        let mut alert_manager = self.alert_manager.write().unwrap();
        alert_manager.trigger_warning_alert(resource_type, utilization_percentage);
    }
    
    /// Check and trigger quota alerts
    fn check_and_trigger_alerts(&self, resource_type: ResourceType, amount: u64) {
        let mut alert_manager = self.alert_manager.write().unwrap();
        alert_manager.check_and_trigger_alerts(resource_type, amount);
    }
    
    /// Calculate performance score based on usage patterns
    fn calculate_performance_score(&self, tracker: &EnhancedResourceTracker) -> f64 {
        let mut score = 100.0_f64;
        
        // Deduct points for high utilization
        let utilizations = [
            tracker.get_events_utilization(&self.limits),
            tracker.get_storage_utilization(&self.limits),
            tracker.get_aggregates_utilization(&self.limits),
        ];
        
        let avg_utilization = utilizations.iter().sum::<f64>() / utilizations.len() as f64;
        
        if avg_utilization > 90.0 {
            score -= 30.0;  // High risk
        } else if avg_utilization > 70.0 {
            score -= 15.0;  // Medium risk
        } else if avg_utilization > 50.0 {
            score -= 5.0;   // Low risk
        }
        
        // Bonus points for stable usage patterns
        if tracker.has_stable_usage_pattern() {
            score += 10.0;
        }
        
        score.clamp(0.0, 100.0)
    }
    
    /// Get billing analytics
    pub fn get_billing_analytics(&self) -> BillingAnalytics {
        let billing_tracker = self.billing_tracker.read().unwrap();
        billing_tracker.get_analytics()
    }
    
    /// Get quota alerts history
    pub fn get_quota_alerts(&self, limit: Option<usize>) -> Vec<QuotaAlert> {
        let alert_manager = self.alert_manager.read().unwrap();
        alert_manager.get_alerts_history(limit.unwrap_or(100))
    }
    
    /// Update quota tier
    pub fn update_tier(&mut self, new_tier: QuotaTier) {
        self.tier = new_tier.clone();
        
        // Update billing tracker with new tier
        let mut billing_tracker = self.billing_tracker.write().unwrap();
        billing_tracker.update_tier(new_tier);
    }
}

/// Error type for quota violations
#[derive(Debug, thiserror::Error)]
#[error("Quota exceeded for tenant {tenant_id}: {resource_type} - current: {current_usage}, limit: {limit}, attempted: {attempted}")]
pub struct QuotaExceeded {
    pub tenant_id: TenantId,
    pub resource_type: String,
    pub current_usage: u64,
    pub limit: u64,
    pub attempted: u64,
}

impl From<QuotaExceeded> for crate::error::EventualiError {
    fn from(err: QuotaExceeded) -> Self {
        crate::error::EventualiError::Tenant(err.to_string())
    }
}

/// Legacy resource usage for backward compatibility
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub tenant_id: TenantId,
    pub daily_events: u64,
    pub storage_used_mb: f64,
    pub concurrent_streams: u32,
    pub total_projections: u32,
    pub total_aggregates: u64,
    pub limits: ResourceLimits,
    pub last_updated: DateTime<Utc>,
}

impl ResourceUsage {
    /// Calculate utilization percentage for a resource
    pub fn utilization_percentage(&self, resource_type: ResourceType) -> Option<f64> {
        match resource_type {
            ResourceType::Events => {
                self.limits.max_events_per_day.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.daily_events as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Storage => {
                self.limits.max_storage_mb.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.storage_used_mb / limit as f64) * 100.0 }
                })
            },
            ResourceType::Streams => {
                self.limits.max_concurrent_streams.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.concurrent_streams as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Projections => {
                self.limits.max_projections.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.total_projections as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::Aggregates => {
                self.limits.max_aggregates.map(|limit| {
                    if limit == 0 { 0.0 } else { (self.total_aggregates as f64 / limit as f64) * 100.0 }
                })
            },
            ResourceType::ApiCalls => {
                // Would typically have API call limits in ResourceLimits
                Some((0.0 / 100_000.0) * 100.0) // Assuming 100k default limit
            },
        }
    }
    
    /// Check if any resource is near its limit (>80%)
    pub fn has_resources_near_limit(&self) -> bool {
        [
            ResourceType::Events,
            ResourceType::Storage,
            ResourceType::Streams,
            ResourceType::Projections,
            ResourceType::Aggregates,
            ResourceType::ApiCalls,
        ].iter().any(|&resource_type| {
            self.utilization_percentage(resource_type)
                .is_some_and(|percentage| percentage > 80.0)
        })
    }
}