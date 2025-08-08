use thiserror::Error;

pub type Result<T> = std::result::Result<T, EventualiError>;

#[derive(Error, Debug)]
pub enum EventualiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Protobuf error: {0}")]
    Protobuf(#[from] prost::DecodeError),
    
    #[error("Aggregate not found: {id}")]
    AggregateNotFound { id: String },
    
    #[error("Optimistic concurrency error: expected version {expected}, got {actual}")]
    OptimisticConcurrency { expected: i64, actual: i64 },
    
    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Tenant error: {0}")]
    Tenant(String),
    
    #[error("Observability error: {0}")]
    ObservabilityError(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Backpressure applied: {0}")]
    BackpressureApplied(String),
    
    #[error("Batch processing error: {0}")]
    BatchProcessingError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}