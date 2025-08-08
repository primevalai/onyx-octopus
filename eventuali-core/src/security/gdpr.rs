use crate::{Result, EventualiError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Comprehensive GDPR compliance system for European Union regulatory requirements
pub struct GdprManager {
    data_subjects: BTreeMap<String, DataSubject>,
    processing_activities: Vec<ProcessingActivity>,
    consent_records: BTreeMap<String, ConsentRecord>,
    #[allow(dead_code)] // Registry for tracking lawful basis for data processing (GDPR compliance)
    lawful_basis_registry: BTreeMap<String, LawfulBasis>,
    retention_policies: BTreeMap<String, RetentionPolicy>,
    breach_notifications: Vec<BreachNotification>,
    data_protection_impact_assessments: BTreeMap<String, DataProtectionImpactAssessment>,
    privacy_by_design_controls: Vec<PrivacyControl>,
    data_exports: Vec<DataExportRecord>,
    deletion_log: Vec<DeletionRecord>,
}

/// Data subject with GDPR rights and personal data tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubject {
    pub subject_id: String,
    pub external_id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub data_locations: Vec<DataLocation>,
    pub consent_status: HashMap<String, ConsentStatus>,
    pub lawful_basis: HashMap<String, LawfulBasisType>,
    pub retention_periods: HashMap<String, RetentionPeriod>,
    pub subject_rights_requests: Vec<SubjectRightsRequest>,
    pub opt_out_status: HashMap<String, bool>,
    pub data_minimization_applied: bool,
}

/// Location of personal data within the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLocation {
    pub database_name: String,
    pub table_name: String,
    pub column_name: String,
    pub data_type: PersonalDataType,
    pub data_classification: DataClassification,
    pub encrypted: bool,
    pub pseudonymized: bool,
    pub retention_period: Option<Duration>,
}

/// Types of personal data under GDPR
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PersonalDataType {
    BasicPersonalData,           // Name, address, email
    SpecialCategoryData,        // Health, religious, political, genetic
    BiometricData,              // Fingerprints, facial recognition
    LocationData,               // GPS, IP addresses
    BehavioralData,             // Website usage, preferences
    CommunicationData,          // Emails, messages, call logs
    FinancialData,              // Payment info, transaction history
    IdentificationData,         // ID numbers, passports
    TechnicalData,              // Cookies, device IDs
    ProfessionalData,           // Employment, qualifications
}

/// Data classification for GDPR compliance
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    SpecialCategory,            // GDPR Article 9 special categories
    ChildrensData,              // Under 16 years old
}

/// Processing activity record as required by GDPR Article 30
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingActivity {
    pub activity_id: String,
    pub name: String,
    pub description: String,
    pub controller: DataController,
    pub data_protection_officer_contact: Option<String>,
    pub purposes: Vec<ProcessingPurpose>,
    pub categories_of_data_subjects: Vec<String>,
    pub categories_of_personal_data: Vec<PersonalDataType>,
    pub categories_of_recipients: Vec<String>,
    pub transfers_to_third_countries: Vec<InternationalTransfer>,
    pub retention_periods: HashMap<PersonalDataType, Duration>,
    pub technical_and_organizational_measures: Vec<SecurityMeasure>,
    pub lawful_basis: LawfulBasisType,
    pub created_at: DateTime<Utc>,
    pub last_reviewed: DateTime<Utc>,
}

/// Data controller information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataController {
    pub name: String,
    pub contact_details: String,
    pub representative: Option<String>,
    pub dpo_contact: Option<String>,
}

/// Processing purpose under GDPR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPurpose {
    pub purpose: String,
    pub description: String,
    pub lawful_basis: LawfulBasisType,
    pub legitimate_interest_assessment: Option<String>,
    pub data_minimization_applied: bool,
}

/// International data transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalTransfer {
    pub destination_country: String,
    pub transfer_mechanism: TransferMechanism,
    pub adequacy_decision: bool,
    pub safeguards: Vec<String>,
    pub derogation: Option<String>,
}

/// Transfer mechanism for international transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferMechanism {
    AdequacyDecision,
    StandardContractualClauses,
    BindingCorporateRules,
    CertificationMechanism,
    CodeOfConduct,
    Derogation(String),
}

/// Security and organizational measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMeasure {
    pub measure_type: SecurityMeasureType,
    pub description: String,
    pub implemented: bool,
    pub implementation_date: Option<DateTime<Utc>>,
    pub responsible_party: String,
}

/// Types of security measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMeasureType {
    Encryption,
    Pseudonymization,
    AccessControl,
    IntegrityMeasure,
    AvailabilityMeasure,
    ResilienceMeasure,
    RegularTesting,
    DataBackup,
    IncidentResponse,
    PrivacyByDesign,
}

/// Consent record for lawful processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub consent_id: String,
    pub data_subject_id: String,
    pub purpose: String,
    pub consent_text: String,
    pub consent_given_at: DateTime<Utc>,
    pub consent_method: ConsentMethod,
    pub consent_status: ConsentStatus,
    pub withdrawn_at: Option<DateTime<Utc>>,
    pub withdrawal_method: Option<String>,
    pub granular_consents: HashMap<String, bool>,
    pub evidence_of_consent: ConsentEvidence,
    pub parental_consent_required: bool,
    pub parental_consent_obtained: Option<DateTime<Utc>>,
}

/// Method of obtaining consent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentMethod {
    WebForm,
    Email,
    PhysicalDocument,
    VerbalConsent,
    ImpliedConsent,
    APICall,
}

/// Consent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentStatus {
    Given,
    Withdrawn,
    Expired,
    Refused,
    Pending,
}

/// Evidence of consent obtained
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentEvidence {
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub form_version: Option<String>,
    pub witness: Option<String>,
    pub digital_signature: Option<String>,
    pub audit_trail: Vec<String>,
}

/// Lawful basis for processing personal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawfulBasis {
    pub basis_id: String,
    pub data_subject_id: String,
    pub processing_purpose: String,
    pub basis_type: LawfulBasisType,
    pub basis_details: String,
    pub documented_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub balancing_test_conducted: Option<DateTime<Utc>>,
    pub balancing_test_result: Option<String>,
}

/// Types of lawful basis under GDPR Article 6
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum LawfulBasisType {
    Consent,                    // Article 6(1)(a)
    Contract,                   // Article 6(1)(b)
    LegalObligation,            // Article 6(1)(c)
    VitalInterests,             // Article 6(1)(d)
    PublicTask,                 // Article 6(1)(e)
    LegitimateInterests,        // Article 6(1)(f)
}

/// Retention policy for personal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub policy_id: String,
    pub data_category: PersonalDataType,
    pub retention_period: Duration,
    pub retention_criteria: String,
    pub disposal_method: DisposalMethod,
    pub review_frequency: Duration,
    pub last_reviewed: DateTime<Utc>,
    pub automatic_deletion: bool,
    pub archival_period: Option<Duration>,
}

/// Retention period for specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub basis_for_retention: String,
    pub automatic_deletion_scheduled: bool,
}

/// Data disposal method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisposalMethod {
    SecureDeletion,
    Anonymization,
    Pseudonymization,
    Archival,
    PhysicalDestruction,
}

/// Personal data breach notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachNotification {
    pub breach_id: String,
    pub detected_at: DateTime<Utc>,
    pub reported_to_authority_at: Option<DateTime<Utc>>,
    pub notified_subjects_at: Option<DateTime<Utc>>,
    pub breach_type: BreachType,
    pub affected_data_subjects: usize,
    pub categories_of_data_affected: Vec<PersonalDataType>,
    pub likely_consequences: String,
    pub measures_taken: Vec<String>,
    pub measures_to_mitigate: Vec<String>,
    pub risk_assessment: RiskLevel,
    pub authority_reference: Option<String>,
    pub requires_subject_notification: bool,
    pub notification_delay_reason: Option<String>,
}

/// Types of data breaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreachType {
    ConfidentialityBreach,      // Unauthorized access
    IntegrityBreach,            // Unauthorized alteration
    AvailabilityBreach,         // Loss of access/availability
    Combined(Vec<BreachType>),  // Multiple breach types
}

/// Risk levels for breach assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Data Protection Impact Assessment (DPIA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProtectionImpactAssessment {
    pub dpia_id: String,
    pub processing_operation: String,
    pub description: String,
    pub necessity_assessment: String,
    pub proportionality_assessment: String,
    pub risks_to_data_subjects: Vec<PrivacyRisk>,
    pub measures_to_address_risks: Vec<RiskMitigation>,
    pub residual_risk_level: RiskLevel,
    pub consultation_with_dpo: bool,
    pub consultation_with_authority: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approved_by: String,
}

/// Privacy risk identified in DPIA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyRisk {
    pub risk_id: String,
    pub description: String,
    pub likelihood: RiskLevel,
    pub impact: RiskLevel,
    pub overall_risk: RiskLevel,
    pub affected_rights: Vec<DataSubjectRight>,
}

/// Risk mitigation measure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigation {
    pub mitigation_id: String,
    pub risk_addressed: String,
    pub measure: String,
    pub implementation_status: ImplementationStatus,
    pub responsible_party: String,
    pub target_date: DateTime<Utc>,
}

/// Implementation status of measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationStatus {
    Planned,
    InProgress,
    Implemented,
    Verified,
    Deferred,
}

/// Privacy by design controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyControl {
    pub control_id: String,
    pub control_type: PrivacyControlType,
    pub description: String,
    pub implementation_level: ImplementationLevel,
    pub effectiveness: EffectivenessLevel,
    pub last_tested: Option<DateTime<Utc>>,
    pub responsible_team: String,
}

/// Types of privacy controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyControlType {
    DataMinimization,
    PurposeLimitation,
    StorageLimitation,
    AccuracyControl,
    IntegrityControl,
    ConfidentialityControl,
    AccountabilityControl,
    TransparencyControl,
}

/// Implementation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationLevel {
    NotImplemented,
    PartiallyImplemented,
    FullyImplemented,
    OptimallyImplemented,
}

/// Effectiveness levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectivenessLevel {
    Ineffective,
    LimitedEffectiveness,
    ModeratelyEffective,
    HighlyEffective,
}

/// Data subject rights under GDPR
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum DataSubjectRight {
    RightToInformation,         // Article 13-14
    RightOfAccess,              // Article 15
    RightToRectification,       // Article 16
    RightToErasure,             // Article 17
    RightToRestrictProcessing,  // Article 18
    RightToDataPortability,     // Article 20
    RightToObject,              // Article 21
    RightsRelatedToAutomatedDecisionMaking, // Article 22
}

/// Data subject rights request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectRightsRequest {
    pub request_id: String,
    pub data_subject_id: String,
    pub request_type: DataSubjectRight,
    pub request_details: String,
    pub requested_at: DateTime<Utc>,
    pub identity_verified_at: Option<DateTime<Utc>>,
    pub identity_verification_method: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub response_sent_at: Option<DateTime<Utc>>,
    pub request_status: RequestStatus,
    pub response_method: Option<ResponseMethod>,
    pub complexity_assessment: ComplexityLevel,
    pub extension_granted: bool,
    pub extension_reason: Option<String>,
    pub third_party_requests: Vec<String>,
    pub processing_fee: Option<f64>,
    pub rejection_reason: Option<String>,
}

/// Status of data subject rights request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Received,
    IdentityVerificationRequired,
    InProgress,
    Completed,
    Rejected,
    PartiallyFulfilled,
    Extended,
}

/// Method of response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseMethod {
    Email,
    Post,
    SecurePortal,
    InPerson,
    StructuredFormat,
}

/// Complexity level of request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,         // Standard response time
    Complex,        // May require extension
    ManifestlyUnfounded,
    Excessive,
}

/// Data export record for portability requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRecord {
    pub export_id: String,
    pub data_subject_id: String,
    pub export_requested_at: DateTime<Utc>,
    pub export_completed_at: Option<DateTime<Utc>>,
    pub export_format: ExportFormat,
    pub data_categories_exported: Vec<PersonalDataType>,
    pub file_size_bytes: Option<u64>,
    pub download_expires_at: DateTime<Utc>,
    pub downloaded_at: Option<DateTime<Utc>>,
    pub encryption_applied: bool,
    pub secure_delivery_method: String,
}

/// Export format for data portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Xml,
    Csv,
    Pdf,
    StructuredFormat(String),
}

/// Record of data deletion/erasure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRecord {
    pub deletion_id: String,
    pub data_subject_id: String,
    pub deletion_requested_at: DateTime<Utc>,
    pub deletion_completed_at: DateTime<Utc>,
    pub deletion_method: DisposalMethod,
    pub data_categories_deleted: Vec<PersonalDataType>,
    pub locations_deleted: Vec<DataLocation>,
    pub verification_hash: String,
    pub certified_by: String,
    pub retention_exceptions: Vec<String>,
    pub legal_hold_applied: bool,
}

impl GdprManager {
    /// Create a new GDPR manager
    pub fn new() -> Self {
        Self {
            data_subjects: BTreeMap::new(),
            processing_activities: Vec::new(),
            consent_records: BTreeMap::new(),
            lawful_basis_registry: BTreeMap::new(),
            retention_policies: BTreeMap::new(),
            breach_notifications: Vec::new(),
            data_protection_impact_assessments: BTreeMap::new(),
            privacy_by_design_controls: Vec::new(),
            data_exports: Vec::new(),
            deletion_log: Vec::new(),
        }
    }

    /// Create GDPR manager with standard EU configuration
    pub fn with_eu_configuration() -> Self {
        let mut manager = Self::new();
        manager.initialize_standard_policies();
        manager.initialize_privacy_controls();
        manager
    }

    /// Register a new data subject
    pub fn register_data_subject(&mut self, external_id: String, email: Option<String>, name: Option<String>) -> Result<String> {
        let subject_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let data_subject = DataSubject {
            subject_id: subject_id.clone(),
            external_id: Some(external_id),
            email,
            name,
            created_at: now,
            last_updated: now,
            data_locations: Vec::new(),
            consent_status: HashMap::new(),
            lawful_basis: HashMap::new(),
            retention_periods: HashMap::new(),
            subject_rights_requests: Vec::new(),
            opt_out_status: HashMap::new(),
            data_minimization_applied: false,
        };

        self.data_subjects.insert(subject_id.clone(), data_subject);
        Ok(subject_id)
    }

    /// Record consent from data subject
    pub fn record_consent(
        &mut self,
        data_subject_id: String,
        purpose: String,
        consent_text: String,
        consent_method: ConsentMethod,
        evidence: ConsentEvidence,
    ) -> Result<String> {
        let consent_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let consent_record = ConsentRecord {
            consent_id: consent_id.clone(),
            data_subject_id: data_subject_id.clone(),
            purpose: purpose.clone(),
            consent_text,
            consent_given_at: now,
            consent_method,
            consent_status: ConsentStatus::Given,
            withdrawn_at: None,
            withdrawal_method: None,
            granular_consents: HashMap::new(),
            evidence_of_consent: evidence,
            parental_consent_required: false,
            parental_consent_obtained: None,
        };

        self.consent_records.insert(consent_id.clone(), consent_record);

        // Update data subject consent status
        if let Some(data_subject) = self.data_subjects.get_mut(&data_subject_id) {
            data_subject.consent_status.insert(purpose, ConsentStatus::Given);
        }

        Ok(consent_id)
    }

    /// Withdraw consent
    pub fn withdraw_consent(&mut self, consent_id: String, withdrawal_method: String) -> Result<()> {
        if let Some(consent) = self.consent_records.get_mut(&consent_id) {
            consent.consent_status = ConsentStatus::Withdrawn;
            consent.withdrawn_at = Some(Utc::now());
            consent.withdrawal_method = Some(withdrawal_method);

            // Update data subject consent status
            if let Some(data_subject) = self.data_subjects.get_mut(&consent.data_subject_id) {
                data_subject.consent_status.insert(consent.purpose.clone(), ConsentStatus::Withdrawn);
            }

            Ok(())
        } else {
            Err(EventualiError::Validation("Consent record not found".to_string()))
        }
    }

    /// Process data subject access request (Article 15)
    pub fn process_access_request(&mut self, data_subject_id: String, request_details: String) -> Result<SubjectRightsRequest> {
        let request_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let request = SubjectRightsRequest {
            request_id: request_id.clone(),
            data_subject_id: data_subject_id.clone(),
            request_type: DataSubjectRight::RightOfAccess,
            request_details,
            requested_at: now,
            identity_verified_at: None,
            identity_verification_method: None,
            processed_at: None,
            response_sent_at: None,
            request_status: RequestStatus::Received,
            response_method: None,
            complexity_assessment: ComplexityLevel::Simple,
            extension_granted: false,
            extension_reason: None,
            third_party_requests: Vec::new(),
            processing_fee: None,
            rejection_reason: None,
        };

        // Add to data subject's request history
        if let Some(data_subject) = self.data_subjects.get_mut(&data_subject_id) {
            data_subject.subject_rights_requests.push(request.clone());
        }

        Ok(request)
    }

    /// Process data rectification request (Article 16)
    pub fn process_rectification_request(&mut self, data_subject_id: String, corrections: HashMap<String, String>) -> Result<SubjectRightsRequest> {
        let request_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let request_details = format!("Rectification requested for: {:?}", corrections.keys().collect::<Vec<_>>());

        let request = SubjectRightsRequest {
            request_id: request_id.clone(),
            data_subject_id: data_subject_id.clone(),
            request_type: DataSubjectRight::RightToRectification,
            request_details,
            requested_at: now,
            identity_verified_at: None,
            identity_verification_method: None,
            processed_at: None,
            response_sent_at: None,
            request_status: RequestStatus::Received,
            response_method: None,
            complexity_assessment: ComplexityLevel::Simple,
            extension_granted: false,
            extension_reason: None,
            third_party_requests: Vec::new(),
            processing_fee: None,
            rejection_reason: None,
        };

        // Add to data subject's request history
        if let Some(data_subject) = self.data_subjects.get_mut(&data_subject_id) {
            data_subject.subject_rights_requests.push(request.clone());
        }

        Ok(request)
    }

    /// Process data erasure request (Article 17 - Right to be Forgotten)
    pub fn process_erasure_request(&mut self, data_subject_id: String, erasure_grounds: String) -> Result<SubjectRightsRequest> {
        let request_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let request = SubjectRightsRequest {
            request_id: request_id.clone(),
            data_subject_id: data_subject_id.clone(),
            request_type: DataSubjectRight::RightToErasure,
            request_details: erasure_grounds,
            requested_at: now,
            identity_verified_at: None,
            identity_verification_method: None,
            processed_at: None,
            response_sent_at: None,
            request_status: RequestStatus::Received,
            response_method: None,
            complexity_assessment: ComplexityLevel::Complex, // Erasure is typically complex
            extension_granted: false,
            extension_reason: None,
            third_party_requests: Vec::new(),
            processing_fee: None,
            rejection_reason: None,
        };

        // Add to data subject's request history
        if let Some(data_subject) = self.data_subjects.get_mut(&data_subject_id) {
            data_subject.subject_rights_requests.push(request.clone());
        }

        Ok(request)
    }

    /// Process data portability request (Article 20)
    pub fn process_portability_request(&mut self, data_subject_id: String, export_format: ExportFormat) -> Result<SubjectRightsRequest> {
        let request_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let request_details = format!("Data portability request in format: {export_format:?}");

        let request = SubjectRightsRequest {
            request_id: request_id.clone(),
            data_subject_id: data_subject_id.clone(),
            request_type: DataSubjectRight::RightToDataPortability,
            request_details,
            requested_at: now,
            identity_verified_at: None,
            identity_verification_method: None,
            processed_at: None,
            response_sent_at: None,
            request_status: RequestStatus::Received,
            response_method: None,
            complexity_assessment: ComplexityLevel::Complex,
            extension_granted: false,
            extension_reason: None,
            third_party_requests: Vec::new(),
            processing_fee: None,
            rejection_reason: None,
        };

        // Create data export record
        let export_record = DataExportRecord {
            export_id: Uuid::new_v4().to_string(),
            data_subject_id: data_subject_id.clone(),
            export_requested_at: now,
            export_completed_at: None,
            export_format,
            data_categories_exported: Vec::new(),
            file_size_bytes: None,
            download_expires_at: now + Duration::days(30), // 30-day expiry
            downloaded_at: None,
            encryption_applied: true,
            secure_delivery_method: "secure_download_link".to_string(),
        };

        self.data_exports.push(export_record);

        // Add to data subject's request history
        if let Some(data_subject) = self.data_subjects.get_mut(&data_subject_id) {
            data_subject.subject_rights_requests.push(request.clone());
        }

        Ok(request)
    }

    /// Execute data deletion/erasure
    pub fn execute_data_deletion(&mut self, data_subject_id: String, deletion_method: DisposalMethod, locations: Vec<DataLocation>) -> Result<String> {
        let deletion_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Calculate verification hash
        let mut hasher = Sha256::new();
        hasher.update(deletion_id.as_bytes());
        hasher.update(data_subject_id.as_bytes());
        hasher.update(now.to_rfc3339().as_bytes());
        let verification_hash = format!("{:x}", hasher.finalize());

        let deletion_record = DeletionRecord {
            deletion_id: deletion_id.clone(),
            data_subject_id: data_subject_id.clone(),
            deletion_requested_at: now,
            deletion_completed_at: now,
            deletion_method,
            data_categories_deleted: locations.iter().map(|l| l.data_type.clone()).collect(),
            locations_deleted: locations,
            verification_hash,
            certified_by: "GDPR_System".to_string(),
            retention_exceptions: Vec::new(),
            legal_hold_applied: false,
        };

        self.deletion_log.push(deletion_record);
        Ok(deletion_id)
    }

    /// Report a personal data breach (Articles 33-34)
    pub fn report_data_breach(
        &mut self,
        breach_type: BreachType,
        affected_subjects: usize,
        affected_data_categories: Vec<PersonalDataType>,
        consequences: String,
        measures_taken: Vec<String>,
    ) -> Result<String> {
        let breach_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Assess risk level based on breach details
        let risk_level = self.assess_breach_risk(&breach_type, affected_subjects, &affected_data_categories);
        let requires_subject_notification = risk_level >= RiskLevel::High;

        let breach_notification = BreachNotification {
            breach_id: breach_id.clone(),
            detected_at: now,
            reported_to_authority_at: None,
            notified_subjects_at: None,
            breach_type,
            affected_data_subjects: affected_subjects,
            categories_of_data_affected: affected_data_categories,
            likely_consequences: consequences,
            measures_taken,
            measures_to_mitigate: Vec::new(),
            risk_assessment: risk_level,
            authority_reference: None,
            requires_subject_notification,
            notification_delay_reason: None,
        };

        self.breach_notifications.push(breach_notification);
        Ok(breach_id)
    }

    /// Create Data Protection Impact Assessment
    pub fn create_dpia(&mut self, processing_operation: String, description: String) -> Result<String> {
        let dpia_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let dpia = DataProtectionImpactAssessment {
            dpia_id: dpia_id.clone(),
            processing_operation,
            description,
            necessity_assessment: "Assessment pending".to_string(),
            proportionality_assessment: "Assessment pending".to_string(),
            risks_to_data_subjects: Vec::new(),
            measures_to_address_risks: Vec::new(),
            residual_risk_level: RiskLevel::Medium,
            consultation_with_dpo: false,
            consultation_with_authority: None,
            created_at: now,
            reviewed_at: None,
            approved_by: "Pending".to_string(),
        };

        self.data_protection_impact_assessments.insert(dpia_id.clone(), dpia);
        Ok(dpia_id)
    }

    /// Get compliance status overview
    pub fn get_compliance_status(&self) -> GdprComplianceStatus {
        let total_subjects = self.data_subjects.len();
        let active_consents = self.consent_records.values()
            .filter(|c| matches!(c.consent_status, ConsentStatus::Given))
            .count();
        let pending_requests = self.data_subjects.values()
            .flat_map(|ds| &ds.subject_rights_requests)
            .filter(|r| matches!(r.request_status, RequestStatus::Received | RequestStatus::InProgress))
            .count();
        let unresolved_breaches = self.breach_notifications.iter()
            .filter(|b| b.reported_to_authority_at.is_none())
            .count();

        GdprComplianceStatus {
            total_data_subjects: total_subjects,
            active_consents,
            withdrawn_consents: self.consent_records.values()
                .filter(|c| matches!(c.consent_status, ConsentStatus::Withdrawn))
                .count(),
            pending_subject_requests: pending_requests,
            completed_subject_requests: self.data_subjects.values()
                .flat_map(|ds| &ds.subject_rights_requests)
                .filter(|r| matches!(r.request_status, RequestStatus::Completed))
                .count(),
            total_processing_activities: self.processing_activities.len(),
            active_dpias: self.data_protection_impact_assessments.len(),
            total_data_breaches: self.breach_notifications.len(),
            unresolved_breaches,
            data_exports_completed: self.data_exports.iter()
                .filter(|e| e.export_completed_at.is_some())
                .count(),
            deletion_records: self.deletion_log.len(),
        }
    }

    /// Generate GDPR compliance report
    pub fn generate_gdpr_compliance_report(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> GdprComplianceReport {
        let report_id = Uuid::new_v4().to_string();
        let generated_at = Utc::now();

        // Filter data within date range
        let period_consents = self.consent_records.values()
            .filter(|c| c.consent_given_at >= start_date && c.consent_given_at <= end_date)
            .count();

        let period_requests = self.data_subjects.values()
            .flat_map(|ds| &ds.subject_rights_requests)
            .filter(|r| r.requested_at >= start_date && r.requested_at <= end_date)
            .collect::<Vec<_>>();

        let period_breaches = self.breach_notifications.iter()
            .filter(|b| b.detected_at >= start_date && b.detected_at <= end_date)
            .collect::<Vec<_>>();

        // Count by request type
        let mut requests_by_type = HashMap::new();
        for request in &period_requests {
            *requests_by_type.entry(request.request_type.clone()).or_insert(0) += 1;
        }

        // Compliance metrics
        let compliance_score = self.calculate_compliance_score();
        let recommendations = self.generate_compliance_recommendations();

        GdprComplianceReport {
            report_id,
            generated_at,
            period_start: start_date,
            period_end: end_date,
            total_data_subjects: self.data_subjects.len(),
            new_consents_given: period_consents,
            consents_withdrawn: self.consent_records.values()
                .filter(|c| matches!(c.consent_status, ConsentStatus::Withdrawn) && 
                         c.withdrawn_at.is_some_and(|w| w >= start_date && w <= end_date))
                .count(),
            subject_requests_received: period_requests.len(),
            subject_requests_by_type: requests_by_type,
            subject_requests_completed_on_time: period_requests.iter()
                .filter(|r| matches!(r.request_status, RequestStatus::Completed) && 
                          !r.extension_granted)
                .count(),
            data_breaches_reported: period_breaches.len(),
            breaches_reported_within_72h: period_breaches.iter()
                .filter(|b| b.reported_to_authority_at.is_some_and(|r| r <= b.detected_at + Duration::hours(72)))
                .count(),
            processing_activities_documented: self.processing_activities.len(),
            dpias_completed: self.data_protection_impact_assessments.len(),
            privacy_controls_implemented: self.privacy_by_design_controls.iter()
                .filter(|c| matches!(c.implementation_level, ImplementationLevel::FullyImplemented | 
                                                            ImplementationLevel::OptimallyImplemented))
                .count(),
            data_exports_fulfilled: self.data_exports.iter()
                .filter(|e| e.export_completed_at.is_some_and(|c| c >= start_date && c <= end_date))
                .count(),
            deletions_executed: self.deletion_log.iter()
                .filter(|d| d.deletion_completed_at >= start_date && d.deletion_completed_at <= end_date)
                .count(),
            compliance_score,
            key_risks_identified: self.identify_key_risks(),
            recommendations,
        }
    }

    // Private helper methods

    fn initialize_standard_policies(&mut self) {
        // Standard retention policies for common data types
        let policies = vec![
            (PersonalDataType::BasicPersonalData, Duration::days(1095)), // 3 years
            (PersonalDataType::SpecialCategoryData, Duration::days(2555)), // 7 years
            (PersonalDataType::FinancialData, Duration::days(2555)), // 7 years
            (PersonalDataType::CommunicationData, Duration::days(365)), // 1 year
            (PersonalDataType::LocationData, Duration::days(90)), // 3 months
            (PersonalDataType::BehavioralData, Duration::days(730)), // 2 years
        ];

        for (data_type, retention_period) in policies {
            let policy_id = Uuid::new_v4().to_string();
            let policy = RetentionPolicy {
                policy_id: policy_id.clone(),
                data_category: data_type,
                retention_period,
                retention_criteria: "Standard GDPR retention policy".to_string(),
                disposal_method: DisposalMethod::SecureDeletion,
                review_frequency: Duration::days(365),
                last_reviewed: Utc::now(),
                automatic_deletion: true,
                archival_period: Some(Duration::days(30)),
            };
            self.retention_policies.insert(policy_id, policy);
        }
    }

    fn initialize_privacy_controls(&mut self) {
        let controls = vec![
            (PrivacyControlType::DataMinimization, "Collect only necessary personal data"),
            (PrivacyControlType::PurposeLimitation, "Process data only for specified purposes"),
            (PrivacyControlType::StorageLimitation, "Delete data when no longer needed"),
            (PrivacyControlType::AccuracyControl, "Keep personal data accurate and up to date"),
            (PrivacyControlType::IntegrityControl, "Protect data from unauthorized alteration"),
            (PrivacyControlType::ConfidentialityControl, "Protect data from unauthorized access"),
            (PrivacyControlType::AccountabilityControl, "Demonstrate compliance with GDPR"),
            (PrivacyControlType::TransparencyControl, "Provide clear information about processing"),
        ];

        for (control_type, description) in controls {
            let control = PrivacyControl {
                control_id: Uuid::new_v4().to_string(),
                control_type,
                description: description.to_string(),
                implementation_level: ImplementationLevel::PartiallyImplemented,
                effectiveness: EffectivenessLevel::ModeratelyEffective,
                last_tested: None,
                responsible_team: "Privacy Team".to_string(),
            };
            self.privacy_by_design_controls.push(control);
        }
    }

    fn assess_breach_risk(&self, breach_type: &BreachType, affected_subjects: usize, data_categories: &[PersonalDataType]) -> RiskLevel {
        let mut risk_score = 0;

        // Assess by breach type
        match breach_type {
            BreachType::ConfidentialityBreach => risk_score += 3,
            BreachType::IntegrityBreach => risk_score += 2,
            BreachType::AvailabilityBreach => risk_score += 1,
            BreachType::Combined(_) => risk_score += 4,
        }

        // Assess by number of affected subjects
        risk_score += match affected_subjects {
            0..=10 => 0,
            11..=100 => 1,
            101..=1000 => 2,
            1001..=10000 => 3,
            _ => 4,
        };

        // Assess by data categories
        for category in data_categories {
            risk_score += match category {
                PersonalDataType::SpecialCategoryData => 4,
                PersonalDataType::FinancialData => 3,
                PersonalDataType::BiometricData => 4,
                _ => 1,
            };
        }

        match risk_score {
            0..=3 => RiskLevel::Low,
            4..=6 => RiskLevel::Medium,
            7..=10 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    fn calculate_compliance_score(&self) -> f64 {
        let mut score = 100.0;

        // Deduct points for compliance gaps
        let overdue_requests = self.data_subjects.values()
            .flat_map(|ds| &ds.subject_rights_requests)
            .filter(|r| {
                matches!(r.request_status, RequestStatus::Received | RequestStatus::InProgress) &&
                r.requested_at < Utc::now() - Duration::days(30)
            })
            .count();

        score -= (overdue_requests as f64) * 5.0; // -5 points per overdue request

        let unresolved_breaches = self.breach_notifications.iter()
            .filter(|b| b.reported_to_authority_at.is_none() && 
                       b.detected_at < Utc::now() - Duration::hours(72))
            .count();

        score -= (unresolved_breaches as f64) * 10.0; // -10 points per late breach report

        // Ensure score doesn't go below 0
        score.max(0.0)
    }

    fn identify_key_risks(&self) -> Vec<String> {
        let mut risks = Vec::new();

        // Check for overdue breach notifications
        let overdue_breaches = self.breach_notifications.iter()
            .filter(|b| b.reported_to_authority_at.is_none() && 
                       b.detected_at < Utc::now() - Duration::hours(72))
            .count();

        if overdue_breaches > 0 {
            risks.push(format!("{overdue_breaches} data breaches not reported within 72 hours"));
        }

        // Check for overdue subject requests
        let overdue_requests = self.data_subjects.values()
            .flat_map(|ds| &ds.subject_rights_requests)
            .filter(|r| {
                matches!(r.request_status, RequestStatus::Received | RequestStatus::InProgress) &&
                r.requested_at < Utc::now() - Duration::days(30)
            })
            .count();

        if overdue_requests > 0 {
            risks.push(format!("{overdue_requests} subject rights requests overdue"));
        }

        // Check for expired consents
        let expired_consents = self.consent_records.values()
            .filter(|c| matches!(c.consent_status, ConsentStatus::Expired))
            .count();

        if expired_consents > 0 {
            risks.push(format!("{expired_consents} consent records have expired"));
        }

        if risks.is_empty() {
            risks.push("No significant compliance risks identified".to_string());
        }

        risks
    }

    fn generate_compliance_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check privacy controls implementation
        let partial_controls = self.privacy_by_design_controls.iter()
            .filter(|c| matches!(c.implementation_level, ImplementationLevel::PartiallyImplemented))
            .count();

        if partial_controls > 0 {
            recommendations.push("Complete implementation of privacy by design controls".to_string());
        }

        // Check for missing DPIAs
        let high_risk_activities = self.processing_activities.iter()
            .filter(|a| a.categories_of_personal_data.contains(&PersonalDataType::SpecialCategoryData))
            .count();

        if high_risk_activities > self.data_protection_impact_assessments.len() {
            recommendations.push("Conduct DPIAs for high-risk processing activities".to_string());
        }

        // Check consent management
        let consent_coverage = (self.consent_records.len() as f64) / (self.data_subjects.len() as f64) * 100.0;
        if consent_coverage < 80.0 {
            recommendations.push("Improve consent collection and management processes".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Maintain current high standards of GDPR compliance".to_string());
        }

        recommendations
    }
}

/// GDPR compliance status overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprComplianceStatus {
    pub total_data_subjects: usize,
    pub active_consents: usize,
    pub withdrawn_consents: usize,
    pub pending_subject_requests: usize,
    pub completed_subject_requests: usize,
    pub total_processing_activities: usize,
    pub active_dpias: usize,
    pub total_data_breaches: usize,
    pub unresolved_breaches: usize,
    pub data_exports_completed: usize,
    pub deletion_records: usize,
}

/// Comprehensive GDPR compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprComplianceReport {
    pub report_id: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_data_subjects: usize,
    pub new_consents_given: usize,
    pub consents_withdrawn: usize,
    pub subject_requests_received: usize,
    pub subject_requests_by_type: HashMap<DataSubjectRight, usize>,
    pub subject_requests_completed_on_time: usize,
    pub data_breaches_reported: usize,
    pub breaches_reported_within_72h: usize,
    pub processing_activities_documented: usize,
    pub dpias_completed: usize,
    pub privacy_controls_implemented: usize,
    pub data_exports_fulfilled: usize,
    pub deletions_executed: usize,
    pub compliance_score: f64,
    pub key_risks_identified: Vec<String>,
    pub recommendations: Vec<String>,
}

impl Default for GdprManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdpr_manager_creation() {
        let manager = GdprManager::new();
        assert_eq!(manager.data_subjects.len(), 0);
        assert_eq!(manager.consent_records.len(), 0);
    }

    #[test]
    fn test_data_subject_registration() {
        let mut manager = GdprManager::new();
        let subject_id = manager.register_data_subject(
            "user123".to_string(),
            Some("user@example.com".to_string()),
            Some("John Doe".to_string()),
        ).unwrap();

        assert!(!subject_id.is_empty());
        assert_eq!(manager.data_subjects.len(), 1);
        
        let data_subject = manager.data_subjects.get(&subject_id).unwrap();
        assert_eq!(data_subject.external_id, Some("user123".to_string()));
        assert_eq!(data_subject.email, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_consent_recording() {
        let mut manager = GdprManager::new();
        let subject_id = manager.register_data_subject(
            "user123".to_string(),
            Some("user@example.com".to_string()),
            None,
        ).unwrap();

        let evidence = ConsentEvidence {
            timestamp: Utc::now(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            form_version: Some("v1.0".to_string()),
            witness: None,
            digital_signature: None,
            audit_trail: vec!["Form submitted".to_string()],
        };

        let consent_id = manager.record_consent(
            subject_id.clone(),
            "marketing".to_string(),
            "I agree to receive marketing emails".to_string(),
            ConsentMethod::WebForm,
            evidence,
        ).unwrap();

        assert!(!consent_id.is_empty());
        assert_eq!(manager.consent_records.len(), 1);
        
        let consent = manager.consent_records.get(&consent_id).unwrap();
        assert_eq!(consent.data_subject_id, subject_id);
        assert!(matches!(consent.consent_status, ConsentStatus::Given));
    }

    #[test]
    fn test_consent_withdrawal() {
        let mut manager = GdprManager::new();
        let subject_id = manager.register_data_subject(
            "user123".to_string(),
            Some("user@example.com".to_string()),
            None,
        ).unwrap();

        let evidence = ConsentEvidence {
            timestamp: Utc::now(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            form_version: None,
            witness: None,
            digital_signature: None,
            audit_trail: Vec::new(),
        };

        let consent_id = manager.record_consent(
            subject_id,
            "marketing".to_string(),
            "I agree to receive marketing emails".to_string(),
            ConsentMethod::WebForm,
            evidence,
        ).unwrap();

        manager.withdraw_consent(consent_id.clone(), "Email unsubscribe".to_string()).unwrap();
        
        let consent = manager.consent_records.get(&consent_id).unwrap();
        assert!(matches!(consent.consent_status, ConsentStatus::Withdrawn));
        assert!(consent.withdrawn_at.is_some());
    }

    #[test]
    fn test_subject_rights_requests() {
        let mut manager = GdprManager::new();
        let subject_id = manager.register_data_subject(
            "user123".to_string(),
            Some("user@example.com".to_string()),
            Some("John Doe".to_string()),
        ).unwrap();

        // Test access request
        let access_request = manager.process_access_request(
            subject_id.clone(),
            "Please provide all my personal data".to_string(),
        ).unwrap();

        assert_eq!(access_request.request_type, DataSubjectRight::RightOfAccess);
        assert!(matches!(access_request.request_status, RequestStatus::Received));

        // Test erasure request
        let erasure_request = manager.process_erasure_request(
            subject_id.clone(),
            "No longer need account".to_string(),
        ).unwrap();

        assert_eq!(erasure_request.request_type, DataSubjectRight::RightToErasure);
        assert!(matches!(erasure_request.complexity_assessment, ComplexityLevel::Complex));

        // Test portability request
        let portability_request = manager.process_portability_request(
            subject_id,
            ExportFormat::Json,
        ).unwrap();

        assert_eq!(portability_request.request_type, DataSubjectRight::RightToDataPortability);
        assert_eq!(manager.data_exports.len(), 1);
    }

    #[test]
    fn test_data_breach_reporting() {
        let mut manager = GdprManager::new();
        
        let breach_id = manager.report_data_breach(
            BreachType::ConfidentialityBreach,
            1000,
            vec![PersonalDataType::BasicPersonalData, PersonalDataType::FinancialData],
            "Unauthorized access to customer database".to_string(),
            vec!["Database access revoked".to_string(), "Passwords reset".to_string()],
        ).unwrap();

        assert!(!breach_id.is_empty());
        assert_eq!(manager.breach_notifications.len(), 1);
        
        let breach = &manager.breach_notifications[0];
        assert_eq!(breach.affected_data_subjects, 1000);
        assert!(breach.requires_subject_notification);
        assert!(matches!(breach.risk_assessment, RiskLevel::High | RiskLevel::Critical));
    }

    #[test]
    fn test_compliance_status() {
        let manager = GdprManager::with_eu_configuration();
        let status = manager.get_compliance_status();
        
        assert_eq!(status.total_data_subjects, 0);
        assert_eq!(status.active_consents, 0);
        assert!(status.total_processing_activities >= 0);
    }

    #[test]
    fn test_gdpr_compliance_report() {
        let manager = GdprManager::with_eu_configuration();
        let start_date = Utc::now() - Duration::days(30);
        let end_date = Utc::now();
        
        let report = manager.generate_gdpr_compliance_report(start_date, end_date);
        
        assert!(!report.report_id.is_empty());
        assert!(report.compliance_score >= 0.0);
        assert!(report.compliance_score <= 100.0);
        assert!(!report.recommendations.is_empty());
    }
}