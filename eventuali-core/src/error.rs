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
}