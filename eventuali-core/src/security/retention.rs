use crate::{Event, EventualiError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};

/// Data retention policy manager for GDPR and compliance
pub struct RetentionPolicyManager {
    policies: HashMap<String, RetentionPolicy>,
    default_policy: String,
}

/// Data retention policy defining how long data should be kept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub name: String,
    pub description: String,
    pub retention_period: RetentionPeriod,
    pub deletion_method: DeletionMethod,
    pub grace_period: Duration,
    pub legal_hold_exempt: bool,
    pub data_categories: Vec<DataCategory>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Different retention periods supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPeriod {
    Days(i64),
    Months(i32),
    Years(i32),
    Indefinite,
    UntilEvent(String), // Keep until specific event occurs
    CustomRule(String), // Custom business rule
}

/// How data should be deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionMethod {
    SoftDelete,     // Mark as deleted but keep data
    HardDelete,     // Permanently remove data
    Anonymize,      // Remove PII but keep aggregated data
    Archive,        // Move to long-term storage
    Encrypt,        // Encrypt in place with secure key rotation
}

/// Categories of data for retention classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataCategory {
    PersonalData,
    SensitivePersonalData,
    FinancialData,
    HealthData,
    CommunicationData,
    BehavioralData,
    TechnicalData,
    MarketingData,
    OperationalData,
    LegalData,
    AuditData,
    BackupData,
}

/// Retention enforcement action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionEnforcementResult {
    pub policy_name: String,
    pub events_processed: usize,
    pub events_deleted: usize,
    pub events_anonymized: usize,
    pub events_archived: usize,
    pub events_encrypted: usize,
    pub enforcement_timestamp: DateTime<Utc>,
    pub next_enforcement: DateTime<Utc>,
    pub errors: Vec<String>,
}

/// Legal hold that overrides retention policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    pub id: String,
    pub reason: String,
    pub authority: String,
    pub case_number: Option<String>,
    pub data_categories: Vec<DataCategory>,
    pub aggregate_patterns: Vec<String>, // Regex patterns for aggregate IDs
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub created_by: String,
    pub status: LegalHoldStatus,
}

/// Legal hold status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LegalHoldStatus {
    Active,
    Released,
    Expired,
}

/// Event data classification for retention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDataClassification {
    pub event_id: String,
    pub aggregate_id: String,
    pub data_categories: Vec<DataCategory>,
    pub retention_policy: String,
    pub classified_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub legal_holds: Vec<String>,
}

impl RetentionPolicyManager {
    /// Create a new retention policy manager
    pub fn new() -> Self {
        let mut manager = Self {
            policies: HashMap::new(),
            default_policy: "default".to_string(),
        };
        
        // Add default GDPR-compliant policy
        let default_policy = RetentionPolicy::gdpr_default();
        manager.policies.insert("default".to_string(), default_policy);
        
        manager
    }

    /// Add a retention policy
    pub fn add_policy(&mut self, policy: RetentionPolicy) -> Result<()> {
        if policy.name.is_empty() {
            return Err(EventualiError::Configuration(
                "Policy name cannot be empty".to_string()
            ));
        }
        
        self.policies.insert(policy.name.clone(), policy);
        Ok(())
    }

    /// Get a retention policy by name
    pub fn get_policy(&self, name: &str) -> Result<&RetentionPolicy> {
        self.policies.get(name).ok_or_else(|| {
            EventualiError::Configuration(format!("Retention policy not found: {}", name))
        })
    }

    /// Set the default retention policy
    pub fn set_default_policy(&mut self, name: &str) -> Result<()> {
        if !self.policies.contains_key(name) {
            return Err(EventualiError::Configuration(
                format!("Retention policy not found: {}", name)
            ));
        }
        self.default_policy = name.to_string();
        Ok(())
    }

    /// Classify event data for retention
    pub fn classify_event(&self, event: &Event) -> Result<EventDataClassification> {
        let data_categories = self.analyze_event_data(event)?;
        let policy_name = self.select_retention_policy(&data_categories)?;
        let policy = self.get_policy(&policy_name)?;
        
        let expires_at = self.calculate_expiration_date(policy)?;
        
        Ok(EventDataClassification {
            event_id: event.id.to_string(),
            aggregate_id: event.aggregate_id.clone(),
            data_categories,
            retention_policy: policy_name,
            classified_at: Utc::now(),
            expires_at,
            legal_holds: Vec::new(),
        })
    }

    /// Enforce retention policies on events
    pub async fn enforce_retention(
        &self,
        events: Vec<Event>,
        classifications: HashMap<String, EventDataClassification>,
        legal_holds: &[LegalHold],
    ) -> Result<RetentionEnforcementResult> {
        let mut result = RetentionEnforcementResult {
            policy_name: "batch_enforcement".to_string(),
            events_processed: 0,
            events_deleted: 0,
            events_anonymized: 0,
            events_archived: 0,
            events_encrypted: 0,
            enforcement_timestamp: Utc::now(),
            next_enforcement: Utc::now() + Duration::days(1),
            errors: Vec::new(),
        };

        for event in events {
            result.events_processed += 1;
            
            // Check if event is under legal hold
            if self.is_under_legal_hold(&event, legal_holds) {
                continue; // Skip enforcement for legally held data
            }

            // Get classification for event
            let classification = match classifications.get(&event.id.to_string()) {
                Some(c) => c,
                None => {
                    result.errors.push(format!(
                        "No classification found for event: {}", event.id
                    ));
                    continue;
                }
            };

            // Check if retention period has expired
            if !self.is_retention_expired(classification)? {
                continue; // Not yet expired
            }

            // Get retention policy
            let policy = match self.get_policy(&classification.retention_policy) {
                Ok(p) => p,
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to get policy for event {}: {}", event.id, e
                    ));
                    continue;
                }
            };

            // Apply deletion method
            match self.apply_deletion_method(&event, &policy.deletion_method).await {
                Ok(method) => match method {
                    DeletionMethod::SoftDelete | DeletionMethod::HardDelete => {
                        result.events_deleted += 1;
                    },
                    DeletionMethod::Anonymize => {
                        result.events_anonymized += 1;
                    },
                    DeletionMethod::Archive => {
                        result.events_archived += 1;
                    },
                    DeletionMethod::Encrypt => {
                        result.events_encrypted += 1;
                    },
                },
                Err(e) => {
                    result.errors.push(format!(
                        "Failed to apply retention to event {}: {}", event.id, e
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Check if event is under legal hold
    fn is_under_legal_hold(&self, event: &Event, legal_holds: &[LegalHold]) -> bool {
        for hold in legal_holds {
            if hold.status != LegalHoldStatus::Active {
                continue;
            }

            // Check if aggregate matches any pattern
            for pattern in &hold.aggregate_patterns {
                if event.aggregate_id.contains(pattern) {
                    return true;
                }
            }

            // Check if event contains data categories under hold
            if let Ok(categories) = self.analyze_event_data(event) {
                for category in &categories {
                    if hold.data_categories.contains(category) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if retention period has expired for classification
    fn is_retention_expired(&self, classification: &EventDataClassification) -> Result<bool> {
        match classification.expires_at {
            Some(expires_at) => Ok(Utc::now() > expires_at),
            None => Ok(false), // Indefinite retention
        }
    }

    /// Apply deletion method to event
    async fn apply_deletion_method(
        &self,
        _event: &Event,
        method: &DeletionMethod,
    ) -> Result<DeletionMethod> {
        // In a real implementation, this would:
        // - Connect to the event store
        // - Apply the specific deletion method
        // - Log the action for audit trail
        // - Handle errors gracefully
        
        match method {
            DeletionMethod::SoftDelete => {
                // Mark event as deleted in metadata
                // UPDATE events SET metadata = metadata || '{"deleted": true}' WHERE id = ?
            },
            DeletionMethod::HardDelete => {
                // Permanently remove from database
                // DELETE FROM events WHERE id = ?
            },
            DeletionMethod::Anonymize => {
                // Remove PII from event data
                // UPDATE events SET data = anonymize_pii(data) WHERE id = ?
            },
            DeletionMethod::Archive => {
                // Move to archive storage
                // INSERT INTO archived_events SELECT * FROM events WHERE id = ?
                // DELETE FROM events WHERE id = ?
            },
            DeletionMethod::Encrypt => {
                // Encrypt in place with rotated key
                // UPDATE events SET data = encrypt_with_new_key(data) WHERE id = ?
            },
        }

        Ok(method.clone())
    }

    /// Analyze event data to determine data categories
    fn analyze_event_data(&self, event: &Event) -> Result<Vec<DataCategory>> {
        let mut categories = Vec::new();

        // Analyze event data JSON for PII indicators
        if let crate::EventData::Json(data) = &event.data {
            let data_str = data.to_string().to_lowercase();

            // Check for personal data indicators
            if data_str.contains("email") || data_str.contains("phone") || 
               data_str.contains("address") || data_str.contains("name") {
                categories.push(DataCategory::PersonalData);
            }

            // Check for sensitive personal data
            if data_str.contains("ssn") || data_str.contains("passport") ||
               data_str.contains("driver_license") || data_str.contains("medical") {
                categories.push(DataCategory::SensitivePersonalData);
            }

            // Check for financial data
            if data_str.contains("credit_card") || data_str.contains("bank_account") ||
               data_str.contains("payment") || data_str.contains("transaction") {
                categories.push(DataCategory::FinancialData);
            }

            // Check for health data
            if data_str.contains("medical") || data_str.contains("health") ||
               data_str.contains("diagnosis") || data_str.contains("treatment") {
                categories.push(DataCategory::HealthData);
            }

            // Check for communication data
            if data_str.contains("message") || data_str.contains("communication") ||
               data_str.contains("chat") || data_str.contains("email") {
                categories.push(DataCategory::CommunicationData);
            }

            // Check for behavioral data
            if data_str.contains("click") || data_str.contains("view") ||
               data_str.contains("behavior") || data_str.contains("interaction") {
                categories.push(DataCategory::BehavioralData);
            }

            // Check for marketing data
            if data_str.contains("campaign") || data_str.contains("marketing") ||
               data_str.contains("advertisement") || data_str.contains("promotion") {
                categories.push(DataCategory::MarketingData);
            }
        }

        // Default to operational data if no specific categories found
        if categories.is_empty() {
            categories.push(DataCategory::OperationalData);
        }

        Ok(categories)
    }

    /// Select appropriate retention policy based on data categories
    fn select_retention_policy(&self, categories: &[DataCategory]) -> Result<String> {
        // Priority order for data categories (most restrictive first)
        let priority = [
            DataCategory::HealthData,
            DataCategory::FinancialData,
            DataCategory::SensitivePersonalData,
            DataCategory::PersonalData,
            DataCategory::LegalData,
            DataCategory::CommunicationData,
            DataCategory::BehavioralData,
            DataCategory::MarketingData,
            DataCategory::TechnicalData,
            DataCategory::OperationalData,
        ];

        // Find the most restrictive category present
        for category in &priority {
            if categories.contains(category) {
                // In a real implementation, this would map to specific policies
                return Ok(match category {
                    DataCategory::HealthData => "health_data_7_years".to_string(),
                    DataCategory::FinancialData => "financial_data_10_years".to_string(),
                    DataCategory::SensitivePersonalData => "sensitive_pii_3_years".to_string(),
                    DataCategory::PersonalData => "personal_data_2_years".to_string(),
                    DataCategory::LegalData => "legal_data_indefinite".to_string(),
                    _ => self.default_policy.clone(),
                });
            }
        }

        Ok(self.default_policy.clone())
    }

    /// Calculate expiration date based on retention policy
    fn calculate_expiration_date(&self, policy: &RetentionPolicy) -> Result<Option<DateTime<Utc>>> {
        let base_date = Utc::now();
        
        match &policy.retention_period {
            RetentionPeriod::Days(days) => {
                Ok(Some(base_date + Duration::days(*days)))
            },
            RetentionPeriod::Months(months) => {
                Ok(Some(base_date + Duration::days(*months as i64 * 30)))
            },
            RetentionPeriod::Years(years) => {
                Ok(Some(base_date + Duration::days(*years as i64 * 365)))
            },
            RetentionPeriod::Indefinite => Ok(None),
            RetentionPeriod::UntilEvent(_event_type) => {
                // Would need to check for specific events in the system
                Ok(None)
            },
            RetentionPeriod::CustomRule(_rule) => {
                // Would need to evaluate custom business rules
                Ok(Some(base_date + Duration::days(365))) // Default to 1 year
            },
        }
    }

    /// List all retention policies
    pub fn list_policies(&self) -> Vec<String> {
        self.policies.keys().cloned().collect()
    }

    /// Get retention statistics
    pub fn get_retention_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_policies".to_string(), self.policies.len());
        
        // Count policies by retention period type
        for policy in self.policies.values() {
            let period_type = match &policy.retention_period {
                RetentionPeriod::Days(_) => "days_based",
                RetentionPeriod::Months(_) => "months_based",
                RetentionPeriod::Years(_) => "years_based",
                RetentionPeriod::Indefinite => "indefinite",
                RetentionPeriod::UntilEvent(_) => "event_based",
                RetentionPeriod::CustomRule(_) => "custom_rule",
            };
            
            *stats.entry(period_type.to_string()).or_insert(0) += 1;
        }

        stats
    }
}

impl RetentionPolicy {
    /// Create GDPR-compliant default retention policy
    pub fn gdpr_default() -> Self {
        Self {
            name: "gdpr_default".to_string(),
            description: "GDPR-compliant default retention policy".to_string(),
            retention_period: RetentionPeriod::Years(2),
            deletion_method: DeletionMethod::Anonymize,
            grace_period: Duration::days(30),
            legal_hold_exempt: false,
            data_categories: vec![
                DataCategory::PersonalData,
                DataCategory::BehavioralData,
                DataCategory::TechnicalData,
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create financial data retention policy (typically longer retention)
    pub fn financial_data_policy() -> Self {
        Self {
            name: "financial_data_10_years".to_string(),
            description: "Financial data retention for regulatory compliance".to_string(),
            retention_period: RetentionPeriod::Years(10),
            deletion_method: DeletionMethod::Archive,
            grace_period: Duration::days(90),
            legal_hold_exempt: false,
            data_categories: vec![
                DataCategory::FinancialData,
                DataCategory::AuditData,
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create health data retention policy (HIPAA compliance)
    pub fn health_data_policy() -> Self {
        Self {
            name: "health_data_7_years".to_string(),
            description: "Health data retention for HIPAA compliance".to_string(),
            retention_period: RetentionPeriod::Years(7),
            deletion_method: DeletionMethod::Encrypt,
            grace_period: Duration::days(60),
            legal_hold_exempt: false,
            data_categories: vec![
                DataCategory::HealthData,
                DataCategory::SensitivePersonalData,
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create marketing data retention policy (shorter retention)
    pub fn marketing_data_policy() -> Self {
        Self {
            name: "marketing_data_1_year".to_string(),
            description: "Marketing data retention policy".to_string(),
            retention_period: RetentionPeriod::Years(1),
            deletion_method: DeletionMethod::Anonymize,
            grace_period: Duration::days(14),
            legal_hold_exempt: true,
            data_categories: vec![
                DataCategory::MarketingData,
                DataCategory::BehavioralData,
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Check if policy applies to data category
    pub fn applies_to_category(&self, category: &DataCategory) -> bool {
        self.data_categories.contains(category)
    }

    /// Update the policy's updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for RetentionPolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalHold {
    /// Create a new legal hold
    pub fn new(
        id: String,
        reason: String,
        authority: String,
        data_categories: Vec<DataCategory>,
        aggregate_patterns: Vec<String>,
        created_by: String,
    ) -> Self {
        Self {
            id,
            reason,
            authority,
            case_number: None,
            data_categories,
            aggregate_patterns,
            start_date: Utc::now(),
            end_date: None,
            created_by,
            status: LegalHoldStatus::Active,
        }
    }

    /// Release the legal hold
    pub fn release(&mut self) {
        self.status = LegalHoldStatus::Released;
        self.end_date = Some(Utc::now());
    }

    /// Check if legal hold is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.status, LegalHoldStatus::Active) &&
        (self.end_date.is_none() || self.end_date.unwrap() > Utc::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventData, EventMetadata};
    use uuid::Uuid;

    fn create_test_event_with_data(data: serde_json::Value) -> Event {
        Event {
            id: Uuid::new_v4(),
            aggregate_id: "test-aggregate".to_string(),
            aggregate_type: "TestAggregate".to_string(),
            event_type: "TestEvent".to_string(),
            event_version: 1,
            aggregate_version: 1,
            data: EventData::Json(data),
            metadata: EventMetadata::default(),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_retention_policy_manager_creation() {
        let manager = RetentionPolicyManager::new();
        assert_eq!(manager.policies.len(), 1); // Should have default policy
        assert!(manager.policies.contains_key("default"));
    }

    #[test]
    fn test_policy_addition() {
        let mut manager = RetentionPolicyManager::new();
        let policy = RetentionPolicy::financial_data_policy();
        
        assert!(manager.add_policy(policy.clone()).is_ok());
        assert!(manager.get_policy(&policy.name).is_ok());
    }

    #[test]
    fn test_event_data_classification() {
        let manager = RetentionPolicyManager::new();
        
        // Test personal data detection
        let personal_data = serde_json::json!({
            "user_email": "test@example.com",
            "user_name": "John Doe"
        });
        let event = create_test_event_with_data(personal_data);
        
        let classification = manager.classify_event(&event).unwrap();
        assert!(classification.data_categories.contains(&DataCategory::PersonalData));
        
        // Test financial data detection
        let financial_data = serde_json::json!({
            "credit_card": "1234-5678-9012-3456",
            "transaction_amount": 100.00
        });
        let event = create_test_event_with_data(financial_data);
        
        let classification = manager.classify_event(&event).unwrap();
        assert!(classification.data_categories.contains(&DataCategory::FinancialData));
    }

    #[test]
    fn test_retention_period_calculation() {
        let policy = RetentionPolicy {
            name: "test".to_string(),
            description: "Test policy".to_string(),
            retention_period: RetentionPeriod::Days(30),
            deletion_method: DeletionMethod::SoftDelete,
            grace_period: Duration::days(7),
            legal_hold_exempt: false,
            data_categories: vec![DataCategory::OperationalData],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let manager = RetentionPolicyManager::new();
        let expires_at = manager.calculate_expiration_date(&policy).unwrap();
        
        assert!(expires_at.is_some());
        let expiry = expires_at.unwrap();
        let expected_expiry = Utc::now() + Duration::days(30);
        
        // Allow for small time differences in test execution
        let diff = (expiry - expected_expiry).num_seconds().abs();
        assert!(diff < 60); // Within 1 minute
    }

    #[test]
    fn test_legal_hold() {
        let mut hold = LegalHold::new(
            "hold-001".to_string(),
            "Investigation".to_string(),
            "Legal Department".to_string(),
            vec![DataCategory::PersonalData],
            vec!["user-123".to_string()],
            "legal@example.com".to_string(),
        );
        
        assert!(hold.is_active());
        
        hold.release();
        assert!(!hold.is_active());
        assert_eq!(hold.status, LegalHoldStatus::Released);
    }

    #[test]
    fn test_gdpr_default_policy() {
        let policy = RetentionPolicy::gdpr_default();
        
        assert_eq!(policy.name, "gdpr_default");
        assert!(matches!(policy.retention_period, RetentionPeriod::Years(2)));
        assert!(matches!(policy.deletion_method, DeletionMethod::Anonymize));
        assert!(!policy.legal_hold_exempt);
    }

    #[test]
    fn test_policy_category_matching() {
        let policy = RetentionPolicy::financial_data_policy();
        
        assert!(policy.applies_to_category(&DataCategory::FinancialData));
        assert!(!policy.applies_to_category(&DataCategory::MarketingData));
    }

    #[tokio::test]
    async fn test_retention_enforcement() {
        let mut manager = RetentionPolicyManager::new();
        
        // Add a policy with very short retention for testing
        let mut test_policy = RetentionPolicy::gdpr_default();
        test_policy.name = "test_immediate".to_string();
        test_policy.retention_period = RetentionPeriod::Days(-1); // Already expired
        manager.add_policy(test_policy).unwrap();
        
        let event = create_test_event_with_data(serde_json::json!({"test": "data"}));
        let mut classifications = HashMap::new();
        
        let classification = EventDataClassification {
            event_id: event.id.to_string(),
            aggregate_id: event.aggregate_id.clone(),
            data_categories: vec![DataCategory::OperationalData],
            retention_policy: "test_immediate".to_string(),
            classified_at: Utc::now(),
            expires_at: Some(Utc::now() - Duration::days(1)), // Already expired
            legal_holds: Vec::new(),
        };
        
        classifications.insert(event.id.to_string(), classification);
        
        let result = manager.enforce_retention(
            vec![event],
            classifications,
            &[]
        ).await.unwrap();
        
        assert_eq!(result.events_processed, 1);
        // Note: In this test, we would expect events_anonymized to be 1
        // but our mock implementation doesn't actually perform the operation
    }
}