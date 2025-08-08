//! Security module providing encryption, digital signatures, audit trails, RBAC, and GDPR compliance

pub mod encryption;
pub mod rbac;
pub mod audit;
pub mod gdpr;
pub mod signatures;
pub mod retention;
pub mod vulnerability;

pub use encryption::{
    EventEncryption, KeyManager, EncryptionKey, EncryptedEventData, EncryptionAlgorithm
};

pub use rbac::{
    RbacManager, User, Role, Permission, Session, SecurityLevel, 
    AccessDecision, AuditEntry, AccessPolicy, PolicyCondition, PolicyEffect
};

pub use audit::{
    AuditManager, AuditTrailEntry, AuditEventType, AuditOutcome, RiskLevel,
    DataClassification, ComplianceTag, AuditSearchCriteria, ComplianceReport,
    IntegrityStatus, RiskSummary, RetentionPolicy, ComplianceSettings
};

pub use gdpr::{
    GdprManager, DataSubject, ProcessingActivity, ConsentRecord, LawfulBasis,
    BreachNotification, DataProtectionImpactAssessment, SubjectRightsRequest,
    DataExportRecord, DeletionRecord, GdprComplianceStatus, GdprComplianceReport,
    PersonalDataType, DataClassification as GdprDataClassification, LawfulBasisType,
    ConsentStatus, ConsentMethod, ConsentEvidence, DataSubjectRight, RequestStatus,
    BreachType, ExportFormat, DisposalMethod, ComplexityLevel, ResponseMethod
};

pub use signatures::{
    EventSigner, SigningKeyManager, SigningKey, SignatureAlgorithm, 
    EventSignature, SignedEvent
};

pub use retention::{
    RetentionPolicyManager, RetentionPeriod, DeletionMethod,
    DataCategory, RetentionEnforcementResult, LegalHold, LegalHoldStatus,
    EventDataClassification
};

pub use vulnerability::{
    VulnerabilityScanner, VulnerabilityScanResult, VulnerabilityFinding,
    VulnerabilityCategory, VulnerabilitySeverity, VulnerabilityStatus,
    PenetrationTestFramework, PenetrationTest, AttackScenario, AttackType
};