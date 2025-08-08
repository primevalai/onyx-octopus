use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap, HashSet};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Comprehensive audit trail system for enterprise compliance
pub struct AuditManager {
    audit_entries: Vec<AuditTrailEntry>,
    search_index: AuditSearchIndex,
    integrity_chain: IntegrityChain,
    retention_policy: RetentionPolicy,
    compliance_settings: ComplianceSettings,
    alert_rules: Vec<AuditAlertRule>,
}

/// Enhanced audit entry with compliance features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    pub entry_id: String,
    pub event_type: AuditEventType,
    pub user_id: String,
    pub session_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub outcome: AuditOutcome,
    pub risk_level: RiskLevel,
    pub metadata: HashMap<String, String>,
    pub compliance_tags: HashSet<ComplianceTag>,
    pub data_classification: DataClassification,
    pub integrity_hash: String,
    pub previous_hash: Option<String>,
    pub correlation_id: Option<String>,
    pub geographic_location: Option<String>,
    pub duration_ms: Option<u64>,
    pub error_details: Option<String>,
}

/// Types of audit events for comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditEventType {
    Authentication,
    Authorization, 
    DataAccess,
    DataModification,
    SystemAccess,
    ConfigurationChange,
    SecurityViolation,
    PrivilegedOperation,
    DataExport,
    AccountManagement,
    SessionManagement,
    PolicyViolation,
    Backup,
    Recovery,
    SystemMaintenance,
}

/// Audit outcome for compliance reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Partial,
    Warning,
    Blocked,
    Escalated,
}

/// Risk level assessment for security monitoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Data classification for regulatory compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    HealthcareData,
    FinancialData,
    PersonalData,
}

/// Compliance framework tags
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplianceTag {
    SOX,        // Sarbanes-Oxley
    GDPR,       // General Data Protection Regulation
    HIPAA,      // Health Insurance Portability and Accountability Act
    PCI_DSS,    // Payment Card Industry Data Security Standard
    ISO27001,   // Information Security Management
    NIST,       // National Institute of Standards and Technology
    COBIT,      // Control Objectives for Information and Related Technologies
    ITIL,       // Information Technology Infrastructure Library
}

/// Search index for efficient audit queries
pub struct AuditSearchIndex {
    by_user: BTreeMap<String, Vec<usize>>,
    by_resource: BTreeMap<String, Vec<usize>>,
    by_event_type: BTreeMap<AuditEventType, Vec<usize>>,
    by_timestamp: BTreeMap<DateTime<Utc>, Vec<usize>>,
    by_risk_level: BTreeMap<RiskLevel, Vec<usize>>,
    by_compliance_tag: BTreeMap<ComplianceTag, Vec<usize>>,
    by_ip_address: BTreeMap<String, Vec<usize>>,
}

/// Cryptographic integrity chain for tamper detection
pub struct IntegrityChain {
    chain_hash: String,
    entry_count: usize,
    last_verification: DateTime<Utc>,
}

/// Retention policy for audit data lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub default_retention_days: u32,
    pub by_event_type: HashMap<AuditEventType, u32>,
    pub by_compliance_tag: HashMap<ComplianceTag, u32>,
    pub archive_after_days: u32,
    pub auto_delete_after_days: Option<u32>,
}

/// Compliance settings for different regulatory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSettings {
    pub enabled_frameworks: HashSet<ComplianceTag>,
    pub minimum_retention_days: u32,
    pub require_integrity_verification: bool,
    pub real_time_monitoring: bool,
    pub automatic_reporting: bool,
    pub data_anonymization_after_days: Option<u32>,
}

/// Alert rule for suspicious activity detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAlertRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub event_types: HashSet<AuditEventType>,
    pub conditions: Vec<AlertCondition>,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_minutes: u32,
}

/// Condition for audit alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub field: String,
    pub operator: AlertOperator,
    pub value: String,
    pub time_window_minutes: Option<u32>,
}

/// Alert operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertOperator {
    Equals,
    NotEquals,
    Contains,
    FrequencyExceeds,
    PatternMatches,
    RiskLevelAbove,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Search criteria for audit queries
#[derive(Debug, Clone)]
pub struct AuditSearchCriteria {
    pub user_id: Option<String>,
    pub event_types: Option<HashSet<AuditEventType>>,
    pub resources: Option<HashSet<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub risk_levels: Option<HashSet<RiskLevel>>,
    pub compliance_tags: Option<HashSet<ComplianceTag>>,
    pub ip_addresses: Option<HashSet<String>>,
    pub outcomes: Option<HashSet<AuditOutcome>>,
    pub text_search: Option<String>,
}

/// Compliance report for regulatory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: String,
    pub framework: ComplianceTag,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub by_event_type: HashMap<AuditEventType, usize>,
    pub security_violations: usize,
    pub policy_violations: usize,
    pub failed_authentications: usize,
    pub privileged_operations: usize,
    pub data_access_events: usize,
    pub integrity_status: IntegrityStatus,
    pub risk_summary: RiskSummary,
    pub recommendations: Vec<String>,
}

/// Integrity verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityStatus {
    pub chain_verified: bool,
    pub tamper_detected: bool,
    pub last_verification: DateTime<Utc>,
    pub total_entries: usize,
    pub verification_errors: Vec<String>,
}

/// Risk assessment summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub by_level: HashMap<RiskLevel, usize>,
    pub trending_up: bool,
    pub high_risk_users: Vec<String>,
    pub suspicious_patterns: Vec<String>,
}

impl AuditManager {
    /// Create a new audit manager with default settings
    pub fn new() -> Self {
        Self {
            audit_entries: Vec::new(),
            search_index: AuditSearchIndex::new(),
            integrity_chain: IntegrityChain::new(),
            retention_policy: RetentionPolicy::default(),
            compliance_settings: ComplianceSettings::default(),
            alert_rules: Vec::new(),
        }
    }

    /// Create audit manager with specific compliance requirements
    pub fn with_compliance(frameworks: HashSet<ComplianceTag>) -> Self {
        let mut audit_manager = Self::new();
        audit_manager.compliance_settings.enabled_frameworks = frameworks;
        audit_manager.initialize_compliance_rules();
        audit_manager
    }

    /// Log an audit event with comprehensive tracking
    pub fn log_audit_event(
        &mut self,
        event_type: AuditEventType,
        user_id: String,
        action: String,
        resource: String,
        outcome: AuditOutcome,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let entry_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Determine risk level and compliance tags
        let risk_level = self.assess_risk_level(&event_type, &outcome, &metadata);
        let compliance_tags = self.determine_compliance_tags(&event_type, &resource);
        let data_classification = self.classify_data(&resource, &metadata);

        // Calculate integrity hash
        let previous_hash = self.integrity_chain.get_current_hash();
        let integrity_hash = self.calculate_integrity_hash(&entry_id, &timestamp, &previous_hash);

        let entry = AuditTrailEntry {
            entry_id: entry_id.clone(),
            event_type: event_type.clone(),
            user_id: user_id.clone(),
            session_id: None,
            action,
            resource: resource.clone(),
            resource_id: None,
            timestamp,
            ip_address: None,
            user_agent: None,
            outcome,
            risk_level,
            metadata: metadata.unwrap_or_default(),
            compliance_tags,
            data_classification,
            integrity_hash: integrity_hash.clone(),
            previous_hash,
            correlation_id: None,
            geographic_location: None,
            duration_ms: None,
            error_details: None,
        };

        // Add to audit log
        let index = self.audit_entries.len();
        self.audit_entries.push(entry.clone());

        // Update search index
        self.search_index.add_entry(index, &entry);

        // Update integrity chain
        self.integrity_chain.update(integrity_hash, index + 1);

        // Check alert rules
        self.check_alert_rules(&entry);

        // Apply retention policy if needed
        if self.audit_entries.len() % 1000 == 0 {
            self.apply_retention_policy();
        }

        Ok(entry_id)
    }

    /// Log authentication event with enhanced details
    pub fn log_authentication_event(
        &mut self,
        user_id: String,
        session_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        success: bool,
        failure_reason: Option<String>,
    ) -> Result<String> {
        let mut metadata = HashMap::new();
        if let Some(reason) = &failure_reason {
            metadata.insert("failure_reason".to_string(), reason.clone());
        }
        if let Some(agent) = &user_agent {
            metadata.insert("user_agent".to_string(), agent.clone());
        }

        let outcome = if success {
            AuditOutcome::Success
        } else {
            AuditOutcome::Failure
        };

        let entry_id = self.log_audit_event(
            AuditEventType::Authentication,
            user_id.clone(),
            if success { "login_success".to_string() } else { "login_failure".to_string() },
            "authentication_system".to_string(),
            outcome,
            Some(metadata),
        )?;

        // Update the entry with authentication-specific details
        if let Some(entry) = self.audit_entries.last_mut() {
            entry.session_id = session_id;
            entry.ip_address = ip_address;
            entry.user_agent = user_agent;
            entry.error_details = failure_reason;
        }

        Ok(entry_id)
    }

    /// Log data access event with privacy compliance
    pub fn log_data_access_event(
        &mut self,
        user_id: String,
        resource: String,
        resource_id: Option<String>,
        operation: String,
        data_classification: DataClassification,
        success: bool,
    ) -> Result<String> {
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), operation.clone());
        metadata.insert("data_classification".to_string(), format!("{:?}", data_classification));

        let outcome = if success {
            AuditOutcome::Success
        } else {
            AuditOutcome::Failure
        };

        let entry_id = self.log_audit_event(
            AuditEventType::DataAccess,
            user_id,
            operation,
            resource,
            outcome,
            Some(metadata),
        )?;

        // Update with data-specific details
        if let Some(entry) = self.audit_entries.last_mut() {
            entry.resource_id = resource_id;
            entry.data_classification = data_classification;
        }

        Ok(entry_id)
    }

    /// Search audit entries with flexible criteria
    pub fn search_audit_entries(
        &self,
        criteria: &AuditSearchCriteria,
        limit: Option<usize>,
    ) -> Vec<&AuditTrailEntry> {
        let mut results = Vec::new();
        let limit = limit.unwrap_or(1000);

        for entry in &self.audit_entries {
            if self.matches_criteria(entry, criteria) {
                results.push(entry);
                if results.len() >= limit {
                    break;
                }
            }
        }

        // Sort by timestamp descending (most recent first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results
    }

    /// Generate compliance report for specific framework
    pub fn generate_compliance_report(
        &self,
        framework: ComplianceTag,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<ComplianceReport> {
        let report_id = Uuid::new_v4().to_string();
        let generated_at = Utc::now();

        // Filter entries for the time period and framework
        let relevant_entries: Vec<_> = self.audit_entries
            .iter()
            .filter(|entry| {
                entry.timestamp >= start_time 
                && entry.timestamp <= end_time
                && entry.compliance_tags.contains(&framework)
            })
            .collect();

        let total_events = relevant_entries.len();

        // Count by event type
        let mut by_event_type = HashMap::new();
        for entry in &relevant_entries {
            *by_event_type.entry(entry.event_type.clone()).or_insert(0) += 1;
        }

        // Count specific compliance metrics
        let security_violations = relevant_entries.iter()
            .filter(|e| e.event_type == AuditEventType::SecurityViolation)
            .count();

        let policy_violations = relevant_entries.iter()
            .filter(|e| e.event_type == AuditEventType::PolicyViolation)
            .count();

        let failed_authentications = relevant_entries.iter()
            .filter(|e| e.event_type == AuditEventType::Authentication && 
                     matches!(e.outcome, AuditOutcome::Failure))
            .count();

        let privileged_operations = relevant_entries.iter()
            .filter(|e| e.event_type == AuditEventType::PrivilegedOperation)
            .count();

        let data_access_events = relevant_entries.iter()
            .filter(|e| matches!(e.event_type, AuditEventType::DataAccess | AuditEventType::DataModification))
            .count();

        // Verify integrity
        let integrity_status = self.verify_integrity();

        // Generate risk summary
        let risk_summary = self.generate_risk_summary(&relevant_entries);

        // Generate recommendations based on findings
        let recommendations = self.generate_compliance_recommendations(&framework, &relevant_entries);

        Ok(ComplianceReport {
            report_id,
            framework,
            generated_at,
            period_start: start_time,
            period_end: end_time,
            total_events,
            by_event_type,
            security_violations,
            policy_violations,
            failed_authentications,
            privileged_operations,
            data_access_events,
            integrity_status,
            risk_summary,
            recommendations,
        })
    }

    /// Verify integrity of audit trail using cryptographic hashes
    pub fn verify_integrity(&self) -> IntegrityStatus {
        let mut verification_errors = Vec::new();
        let mut tamper_detected = false;
        let total_entries = self.audit_entries.len();

        // Verify each entry's hash
        let mut previous_hash: Option<String> = None;
        for (index, entry) in self.audit_entries.iter().enumerate() {
            let expected_hash = self.calculate_integrity_hash(&entry.entry_id, &entry.timestamp, &previous_hash);
            
            if entry.integrity_hash != expected_hash {
                tamper_detected = true;
                verification_errors.push(format!("Hash mismatch at entry {}: {}", index, entry.entry_id));
            }

            if entry.previous_hash != previous_hash {
                tamper_detected = true;
                verification_errors.push(format!("Chain break at entry {}: {}", index, entry.entry_id));
            }

            previous_hash = Some(entry.integrity_hash.clone());
        }

        IntegrityStatus {
            chain_verified: !tamper_detected,
            tamper_detected,
            last_verification: Utc::now(),
            total_entries,
            verification_errors,
        }
    }

    /// Get audit statistics for monitoring dashboard
    pub fn get_audit_statistics(&self, last_hours: u32) -> HashMap<String, serde_json::Value> {
        let since = Utc::now() - Duration::hours(last_hours as i64);
        let recent_entries: Vec<_> = self.audit_entries
            .iter()
            .filter(|e| e.timestamp >= since)
            .collect();

        let mut stats = HashMap::new();

        stats.insert("total_entries".to_string(), serde_json::Value::Number(self.audit_entries.len().into()));
        stats.insert("recent_entries".to_string(), serde_json::Value::Number(recent_entries.len().into()));

        // Count by event type
        let mut by_event_type = HashMap::new();
        for entry in &recent_entries {
            *by_event_type.entry(format!("{:?}", entry.event_type)).or_insert(0) += 1;
        }
        stats.insert("by_event_type".to_string(), serde_json::to_value(by_event_type).unwrap_or_default());

        // Count by risk level
        let mut by_risk_level = HashMap::new();
        for entry in &recent_entries {
            *by_risk_level.entry(format!("{:?}", entry.risk_level)).or_insert(0) += 1;
        }
        stats.insert("by_risk_level".to_string(), serde_json::to_value(by_risk_level).unwrap_or_default());

        // Count by outcome
        let mut by_outcome = HashMap::new();
        for entry in &recent_entries {
            *by_outcome.entry(format!("{:?}", entry.outcome)).or_insert(0) += 1;
        }
        stats.insert("by_outcome".to_string(), serde_json::to_value(by_outcome).unwrap_or_default());

        // Integrity status
        let integrity = self.verify_integrity();
        stats.insert("integrity_verified".to_string(), serde_json::Value::Bool(integrity.chain_verified));
        stats.insert("tamper_detected".to_string(), serde_json::Value::Bool(integrity.tamper_detected));

        stats
    }

    // Private helper methods

    fn assess_risk_level(
        &self,
        event_type: &AuditEventType,
        outcome: &AuditOutcome,
        _metadata: &Option<HashMap<String, String>>,
    ) -> RiskLevel {
        match event_type {
            AuditEventType::SecurityViolation | AuditEventType::PolicyViolation => RiskLevel::Critical,
            AuditEventType::PrivilegedOperation => RiskLevel::High,
            AuditEventType::Authentication if matches!(outcome, AuditOutcome::Failure) => RiskLevel::Medium,
            AuditEventType::DataExport => RiskLevel::Medium,
            AuditEventType::ConfigurationChange => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    fn determine_compliance_tags(&self, event_type: &AuditEventType, resource: &str) -> HashSet<ComplianceTag> {
        let mut tags = HashSet::new();

        // Add tags based on event type
        match event_type {
            AuditEventType::Authentication | AuditEventType::Authorization => {
                tags.insert(ComplianceTag::SOX);
                tags.insert(ComplianceTag::ISO27001);
            }
            AuditEventType::DataAccess | AuditEventType::DataModification => {
                tags.insert(ComplianceTag::GDPR);
                if resource.contains("payment") || resource.contains("card") {
                    tags.insert(ComplianceTag::PCI_DSS);
                }
                if resource.contains("health") || resource.contains("medical") {
                    tags.insert(ComplianceTag::HIPAA);
                }
            }
            AuditEventType::ConfigurationChange | AuditEventType::PrivilegedOperation => {
                tags.insert(ComplianceTag::SOX);
                tags.insert(ComplianceTag::NIST);
            }
            _ => {
                tags.insert(ComplianceTag::ISO27001);
            }
        }

        // Filter by enabled frameworks
        tags.retain(|tag| self.compliance_settings.enabled_frameworks.contains(tag));
        tags
    }

    fn classify_data(&self, resource: &str, _metadata: &Option<HashMap<String, String>>) -> DataClassification {
        if resource.contains("health") || resource.contains("medical") || resource.contains("patient") {
            DataClassification::HealthcareData
        } else if resource.contains("payment") || resource.contains("financial") || resource.contains("card") {
            DataClassification::FinancialData
        } else if resource.contains("personal") || resource.contains("pii") {
            DataClassification::PersonalData
        } else if resource.contains("confidential") || resource.contains("secret") {
            DataClassification::Confidential
        } else if resource.contains("restricted") || resource.contains("classified") {
            DataClassification::Restricted
        } else if resource.contains("internal") {
            DataClassification::Internal
        } else {
            DataClassification::Public
        }
    }

    fn calculate_integrity_hash(&self, entry_id: &str, timestamp: &DateTime<Utc>, previous_hash: &Option<String>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(entry_id.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    fn check_alert_rules(&self, entry: &AuditTrailEntry) {
        // In a real implementation, this would trigger alerts based on rules
        // For now, we'll just log high-risk events
        if entry.risk_level == RiskLevel::Critical || entry.risk_level == RiskLevel::High {
            eprintln!("AUDIT ALERT: High-risk event detected: {:?} by user {} at {}", 
                     entry.event_type, entry.user_id, entry.timestamp);
        }
    }

    fn apply_retention_policy(&mut self) {
        let now = Utc::now();
        let retention_duration = Duration::days(self.retention_policy.default_retention_days as i64);
        let cutoff_time = now - retention_duration;

        // In a real implementation, this would archive or delete old entries
        // For now, we'll just identify entries that would be affected
        let old_entries = self.audit_entries
            .iter()
            .filter(|e| e.timestamp < cutoff_time)
            .count();

        if old_entries > 0 {
            eprintln!("RETENTION: {} entries eligible for archival", old_entries);
        }
    }

    fn matches_criteria(&self, entry: &AuditTrailEntry, criteria: &AuditSearchCriteria) -> bool {
        if let Some(user_id) = &criteria.user_id {
            if entry.user_id != *user_id {
                return false;
            }
        }

        if let Some(event_types) = &criteria.event_types {
            if !event_types.contains(&entry.event_type) {
                return false;
            }
        }

        if let Some(resources) = &criteria.resources {
            if !resources.contains(&entry.resource) {
                return false;
            }
        }

        if let Some(start_time) = criteria.start_time {
            if entry.timestamp < start_time {
                return false;
            }
        }

        if let Some(end_time) = criteria.end_time {
            if entry.timestamp > end_time {
                return false;
            }
        }

        if let Some(risk_levels) = &criteria.risk_levels {
            if !risk_levels.contains(&entry.risk_level) {
                return false;
            }
        }

        if let Some(compliance_tags) = &criteria.compliance_tags {
            if !compliance_tags.iter().any(|tag| entry.compliance_tags.contains(tag)) {
                return false;
            }
        }

        if let Some(ip_addresses) = &criteria.ip_addresses {
            if let Some(ip) = &entry.ip_address {
                if !ip_addresses.contains(ip) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn generate_risk_summary(&self, entries: &[&AuditTrailEntry]) -> RiskSummary {
        let mut by_level = HashMap::new();
        let mut user_risk_counts = HashMap::new();

        for entry in entries {
            *by_level.entry(entry.risk_level.clone()).or_insert(0) += 1;
            
            if entry.risk_level == RiskLevel::High || entry.risk_level == RiskLevel::Critical {
                *user_risk_counts.entry(entry.user_id.clone()).or_insert(0) += 1;
            }
        }

        let high_risk_users: Vec<String> = user_risk_counts
            .into_iter()
            .filter(|(_, count)| *count > 5)
            .map(|(user, _)| user)
            .collect();

        let suspicious_patterns = vec![
            "Multiple failed authentications detected".to_string(),
            "Unusual after-hours access patterns".to_string(),
            "High-volume data access events".to_string(),
        ];

        RiskSummary {
            by_level,
            trending_up: false, // Would calculate based on historical data
            high_risk_users,
            suspicious_patterns,
        }
    }

    fn generate_compliance_recommendations(&self, framework: &ComplianceTag, _entries: &[&AuditTrailEntry]) -> Vec<String> {
        let mut recommendations = Vec::new();

        match framework {
            ComplianceTag::SOX => {
                recommendations.push("Implement stronger segregation of duties for financial operations".to_string());
                recommendations.push("Enhance monitoring of privileged account activities".to_string());
            }
            ComplianceTag::GDPR => {
                recommendations.push("Implement data minimization principles for personal data access".to_string());
                recommendations.push("Enhance consent tracking for data processing activities".to_string());
            }
            ComplianceTag::HIPAA => {
                recommendations.push("Strengthen access controls for healthcare data".to_string());
                recommendations.push("Implement minimum necessary access principles".to_string());
            }
            ComplianceTag::PCI_DSS => {
                recommendations.push("Enhance monitoring of payment card data access".to_string());
                recommendations.push("Implement stronger encryption for card data transmission".to_string());
            }
            _ => {
                recommendations.push("Review security policies and access controls".to_string());
            }
        }

        recommendations
    }

    fn initialize_compliance_rules(&mut self) {
        // Initialize default alert rules for compliance frameworks
        let enabled_frameworks: Vec<_> = self.compliance_settings.enabled_frameworks.iter().cloned().collect();
        for framework in enabled_frameworks {
            match framework {
                ComplianceTag::SOX => self.add_sox_rules(),
                ComplianceTag::GDPR => self.add_gdpr_rules(),
                ComplianceTag::HIPAA => self.add_hipaa_rules(),
                ComplianceTag::PCI_DSS => self.add_pci_rules(),
                _ => {}
            }
        }
    }

    fn add_sox_rules(&mut self) {
        let rule = AuditAlertRule {
            rule_id: "SOX_001".to_string(),
            name: "Financial System Access Alert".to_string(),
            description: "Alert on financial system access outside business hours".to_string(),
            event_types: [AuditEventType::DataAccess, AuditEventType::PrivilegedOperation].into_iter().collect(),
            conditions: vec![
                AlertCondition {
                    field: "resource".to_string(),
                    operator: AlertOperator::Contains,
                    value: "financial".to_string(),
                    time_window_minutes: None,
                }
            ],
            severity: AlertSeverity::High,
            enabled: true,
            cooldown_minutes: 60,
        };
        self.alert_rules.push(rule);
    }

    fn add_gdpr_rules(&mut self) {
        let rule = AuditAlertRule {
            rule_id: "GDPR_001".to_string(),
            name: "Personal Data Access Alert".to_string(),
            description: "Alert on personal data access and modifications".to_string(),
            event_types: [AuditEventType::DataAccess, AuditEventType::DataModification, AuditEventType::DataExport].into_iter().collect(),
            conditions: vec![
                AlertCondition {
                    field: "data_classification".to_string(),
                    operator: AlertOperator::Equals,
                    value: "PersonalData".to_string(),
                    time_window_minutes: None,
                }
            ],
            severity: AlertSeverity::Medium,
            enabled: true,
            cooldown_minutes: 30,
        };
        self.alert_rules.push(rule);
    }

    fn add_hipaa_rules(&mut self) {
        let rule = AuditAlertRule {
            rule_id: "HIPAA_001".to_string(),
            name: "Healthcare Data Alert".to_string(),
            description: "Alert on healthcare data access violations".to_string(),
            event_types: [AuditEventType::DataAccess, AuditEventType::SecurityViolation].into_iter().collect(),
            conditions: vec![
                AlertCondition {
                    field: "data_classification".to_string(),
                    operator: AlertOperator::Equals,
                    value: "HealthcareData".to_string(),
                    time_window_minutes: None,
                }
            ],
            severity: AlertSeverity::Critical,
            enabled: true,
            cooldown_minutes: 15,
        };
        self.alert_rules.push(rule);
    }

    fn add_pci_rules(&mut self) {
        let rule = AuditAlertRule {
            rule_id: "PCI_001".to_string(),
            name: "Payment Card Data Alert".to_string(),
            description: "Alert on payment card data access and processing".to_string(),
            event_types: [AuditEventType::DataAccess, AuditEventType::DataModification, AuditEventType::DataExport].into_iter().collect(),
            conditions: vec![
                AlertCondition {
                    field: "data_classification".to_string(),
                    operator: AlertOperator::Equals,
                    value: "FinancialData".to_string(),
                    time_window_minutes: None,
                }
            ],
            severity: AlertSeverity::Critical,
            enabled: true,
            cooldown_minutes: 10,
        };
        self.alert_rules.push(rule);
    }
}

impl AuditSearchIndex {
    fn new() -> Self {
        Self {
            by_user: BTreeMap::new(),
            by_resource: BTreeMap::new(),
            by_event_type: BTreeMap::new(),
            by_timestamp: BTreeMap::new(),
            by_risk_level: BTreeMap::new(),
            by_compliance_tag: BTreeMap::new(),
            by_ip_address: BTreeMap::new(),
        }
    }

    fn add_entry(&mut self, index: usize, entry: &AuditTrailEntry) {
        self.by_user.entry(entry.user_id.clone())
            .or_insert_with(Vec::new)
            .push(index);

        self.by_resource.entry(entry.resource.clone())
            .or_insert_with(Vec::new)
            .push(index);

        self.by_event_type.entry(entry.event_type.clone())
            .or_insert_with(Vec::new)
            .push(index);

        self.by_timestamp.entry(entry.timestamp)
            .or_insert_with(Vec::new)
            .push(index);

        self.by_risk_level.entry(entry.risk_level.clone())
            .or_insert_with(Vec::new)
            .push(index);

        for tag in &entry.compliance_tags {
            self.by_compliance_tag.entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }

        if let Some(ip) = &entry.ip_address {
            self.by_ip_address.entry(ip.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }
    }
}

impl IntegrityChain {
    fn new() -> Self {
        Self {
            chain_hash: "genesis".to_string(),
            entry_count: 0,
            last_verification: Utc::now(),
        }
    }

    fn update(&mut self, new_hash: String, count: usize) {
        self.chain_hash = new_hash;
        self.entry_count = count;
        self.last_verification = Utc::now();
    }

    fn get_current_hash(&self) -> Option<String> {
        if self.entry_count > 0 {
            Some(self.chain_hash.clone())
        } else {
            None
        }
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        let mut by_event_type = HashMap::new();
        by_event_type.insert(AuditEventType::SecurityViolation, 2555); // 7 years
        by_event_type.insert(AuditEventType::Authentication, 365);      // 1 year
        by_event_type.insert(AuditEventType::DataAccess, 1095);         // 3 years

        let mut by_compliance_tag = HashMap::new();
        by_compliance_tag.insert(ComplianceTag::SOX, 2555);    // 7 years for SOX
        by_compliance_tag.insert(ComplianceTag::HIPAA, 2190);  // 6 years for HIPAA
        by_compliance_tag.insert(ComplianceTag::GDPR, 1095);   // 3 years for GDPR

        Self {
            default_retention_days: 365,
            by_event_type,
            by_compliance_tag,
            archive_after_days: 180,
            auto_delete_after_days: None,
        }
    }
}

impl Default for ComplianceSettings {
    fn default() -> Self {
        Self {
            enabled_frameworks: [ComplianceTag::SOX, ComplianceTag::GDPR, ComplianceTag::ISO27001].into_iter().collect(),
            minimum_retention_days: 365,
            require_integrity_verification: true,
            real_time_monitoring: true,
            automatic_reporting: false,
            data_anonymization_after_days: Some(1095),
        }
    }
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_manager_creation() {
        let audit_manager = AuditManager::new();
        assert_eq!(audit_manager.audit_entries.len(), 0);
        assert!(audit_manager.compliance_settings.enabled_frameworks.len() > 0);
    }

    #[test]
    fn test_audit_event_logging() {
        let mut audit_manager = AuditManager::new();
        let entry_id = audit_manager.log_audit_event(
            AuditEventType::Authentication,
            "user123".to_string(),
            "login".to_string(),
            "system".to_string(),
            AuditOutcome::Success,
            None,
        ).unwrap();

        assert!(!entry_id.is_empty());
        assert_eq!(audit_manager.audit_entries.len(), 1);
        assert_eq!(audit_manager.audit_entries[0].user_id, "user123");
        assert_eq!(audit_manager.audit_entries[0].event_type, AuditEventType::Authentication);
    }

    #[test]
    fn test_integrity_verification() {
        let mut audit_manager = AuditManager::new();
        
        // Add some entries
        for i in 0..5 {
            audit_manager.log_audit_event(
                AuditEventType::DataAccess,
                format!("user{}", i),
                "read".to_string(),
                "database".to_string(),
                AuditOutcome::Success,
                None,
            ).unwrap();
        }

        let integrity_status = audit_manager.verify_integrity();
        assert!(integrity_status.chain_verified);
        assert!(!integrity_status.tamper_detected);
        assert_eq!(integrity_status.total_entries, 5);
        assert!(integrity_status.verification_errors.is_empty());
    }

    #[test]
    fn test_compliance_report_generation() {
        let mut audit_manager = AuditManager::new();
        let start_time = Utc::now() - Duration::hours(1);
        let end_time = Utc::now();

        // Add some test events
        audit_manager.log_audit_event(
            AuditEventType::DataAccess,
            "user123".to_string(),
            "read_personal_data".to_string(),
            "customer_database".to_string(),
            AuditOutcome::Success,
            None,
        ).unwrap();

        let report = audit_manager.generate_compliance_report(
            ComplianceTag::GDPR,
            start_time,
            end_time,
        ).unwrap();

        assert!(!report.report_id.is_empty());
        assert_eq!(report.framework, ComplianceTag::GDPR);
        assert!(report.total_events > 0);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_audit_search() {
        let mut audit_manager = AuditManager::new();

        // Add test entries
        audit_manager.log_audit_event(
            AuditEventType::Authentication,
            "user1".to_string(),
            "login".to_string(),
            "system".to_string(),
            AuditOutcome::Success,
            None,
        ).unwrap();

        audit_manager.log_audit_event(
            AuditEventType::DataAccess,
            "user2".to_string(),
            "read".to_string(),
            "database".to_string(),
            AuditOutcome::Success,
            None,
        ).unwrap();

        let criteria = AuditSearchCriteria {
            user_id: Some("user1".to_string()),
            event_types: None,
            resources: None,
            start_time: None,
            end_time: None,
            risk_levels: None,
            compliance_tags: None,
            ip_addresses: None,
            outcomes: None,
            text_search: None,
        };

        let results = audit_manager.search_audit_entries(&criteria, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, "user1");
    }
}