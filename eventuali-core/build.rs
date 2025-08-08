use std::io::Result;
use std::env;

fn main() -> Result<()> {
    // Check if we should skip protobuf generation
    if env::var("SKIP_PROTO").is_ok() {
        println!("cargo:warning=Skipping protobuf generation (SKIP_PROTO is set)");
        return Ok(());
    }
    
    let fallback_code = r#"
// Fallback proto definitions
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub aggregate_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub aggregate_type: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub event_type: ::prost::alloc::string::String,
    #[prost(int32, tag = "5")]
    pub event_version: i32,
    #[prost(int64, tag = "6")]
    pub aggregate_version: i64,
    #[prost(bytes, tag = "7")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "8")]
    pub metadata: ::core::option::Option<EventMetadata>,
    #[prost(int64, tag = "9")]
    pub timestamp: i64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventMetadata {
    #[prost(string, tag = "1")]
    pub causation_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub correlation_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "4")]
    pub headers: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AggregateSnapshot {
    #[prost(string, tag = "1")]
    pub aggregate_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub aggregate_type: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub version: i64,
    #[prost(bytes, tag = "4")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "5")]
    pub timestamp: i64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRegistered {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub email: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserEmailChanged {
    #[prost(string, tag = "1")]
    pub old_email: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub new_email: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserDeactivated {
    #[prost(string, tag = "1")]
    pub reason: ::prost::alloc::string::String,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderPlaced {
    #[prost(string, tag = "1")]
    pub customer_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub items: ::prost::alloc::vec::Vec<OrderItem>,
    #[prost(double, tag = "3")]
    pub total_amount: f64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderItem {
    #[prost(string, tag = "1")]
    pub product_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(int32, tag = "3")]
    pub quantity: i32,
    #[prost(double, tag = "4")]
    pub unit_price: f64,
}
"#;
    
    let mut config = prost_build::Config::new();
    // Remove serde attributes to avoid compatibility issues
    // config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    // config.field_attribute(".", "#[serde(default)]");
    
    // Try to compile protos, but provide fallback if protoc is missing
    match config.compile_protos(&["proto/event.proto"], &["proto/"]) {
        Ok(()) => {
            // Successfully generated protobuf code (no output to avoid unnecessary warnings)
        },
        Err(e) => {
            println!("cargo:warning=Failed to compile protos: {e}. Using fallback definitions.");
            
            let out_dir = env::var("OUT_DIR").unwrap();
            let dest_path = std::path::Path::new(&out_dir).join("eventuali.rs");
            std::fs::write(dest_path, fallback_code)?;
        }
    }
    
    Ok(())
}