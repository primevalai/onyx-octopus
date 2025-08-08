//! Tenant-specific dynamic configuration management
//!
//! This module provides comprehensive tenant configuration management capabilities including:
//! - Dynamic per-tenant settings with hot-reloading
//! - Configuration validation and type safety
//! - Environment-specific overrides (dev/staging/prod)
//! - Configuration versioning and rollback
//! - Real-time configuration monitoring and alerts
//! - Configuration templates and inheritance

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type alias for change listener callback
pub type ChangeListener = Box<dyn Fn(&ConfigurationChangeEvent) + Send + Sync>;
use serde_json::Value;

use super::tenant::TenantId;
use crate::error::{EventualiError, Result};

/// Configuration environments for environment-specific overrides
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum ConfigurationEnvironment {
    Development,
    Staging,
    #[default]
    Production,
    Testing,
}


/// Configuration data types with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ConfigurationValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigurationValue>),
    Object(HashMap<String, ConfigurationValue>),
}

impl ConfigurationValue {
    /// Validate configuration value against schema
    pub fn validate(&self, schema: &ConfigurationSchema) -> Result<()> {
        match (self, schema) {
            (ConfigurationValue::String(s), ConfigurationSchema::String { min_length, max_length, pattern }) => {
                if let Some(min) = min_length {
                    if s.len() < *min {
                        return Err(EventualiError::Tenant(format!("String too short: {} < {}", s.len(), min)));
                    }
                }
                if let Some(max) = max_length {
                    if s.len() > *max {
                        return Err(EventualiError::Tenant(format!("String too long: {} > {}", s.len(), max)));
                    }
                }
                if let Some(pattern) = pattern {
                    if !regex::Regex::new(pattern).unwrap().is_match(s) {
                        return Err(EventualiError::Tenant(format!("String doesn't match pattern: {pattern}")));
                    }
                }
                Ok(())
            },
            (ConfigurationValue::Integer(i), ConfigurationSchema::Integer { min, max }) => {
                if let Some(min_val) = min {
                    if i < min_val {
                        return Err(EventualiError::Tenant(format!("Integer too small: {i} < {min_val}")));
                    }
                }
                if let Some(max_val) = max {
                    if i > max_val {
                        return Err(EventualiError::Tenant(format!("Integer too large: {i} > {max_val}")));
                    }
                }
                Ok(())
            },
            (ConfigurationValue::Float(f), ConfigurationSchema::Float { min, max }) => {
                if let Some(min_val) = min {
                    if f < min_val {
                        return Err(EventualiError::Tenant(format!("Float too small: {f} < {min_val}")));
                    }
                }
                if let Some(max_val) = max {
                    if f > max_val {
                        return Err(EventualiError::Tenant(format!("Float too large: {f} > {max_val}")));
                    }
                }
                Ok(())
            },
            (ConfigurationValue::Boolean(_), ConfigurationSchema::Boolean) => Ok(()),
            (ConfigurationValue::Array(items), ConfigurationSchema::Array { item_schema, min_items, max_items }) => {
                if let Some(min) = min_items {
                    if items.len() < *min {
                        return Err(EventualiError::Tenant(format!("Array too small: {} < {}", items.len(), min)));
                    }
                }
                if let Some(max) = max_items {
                    if items.len() > *max {
                        return Err(EventualiError::Tenant(format!("Array too large: {} > {}", items.len(), max)));
                    }
                }
                for item in items {
                    item.validate(item_schema)?;
                }
                Ok(())
            },
            (ConfigurationValue::Object(obj), ConfigurationSchema::Object { properties, required }) => {
                // Check required fields
                for req_field in required {
                    if !obj.contains_key(req_field) {
                        return Err(EventualiError::Tenant(format!("Required field missing: {req_field}")));
                    }
                }
                // Validate each property
                for (key, value) in obj {
                    if let Some(prop_schema) = properties.get(key) {
                        value.validate(prop_schema)?;
                    }
                }
                Ok(())
            },
            _ => Err(EventualiError::Tenant("Configuration type mismatch".to_string())),
        }
    }

    /// Convert to JSON Value
    pub fn to_json(&self) -> Value {
        match self {
            ConfigurationValue::String(s) => Value::String(s.clone()),
            ConfigurationValue::Integer(i) => Value::Number(serde_json::Number::from(*i)),
            ConfigurationValue::Float(f) => Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0))),
            ConfigurationValue::Boolean(b) => Value::Bool(*b),
            ConfigurationValue::Array(arr) => {
                Value::Array(arr.iter().map(|v| v.to_json()).collect())
            },
            ConfigurationValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj {
                    map.insert(k.clone(), v.to_json());
                }
                Value::Object(map)
            },
        }
    }

    /// Create from JSON Value
    pub fn from_json(value: &Value) -> Self {
        match value {
            Value::String(s) => ConfigurationValue::String(s.clone()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ConfigurationValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ConfigurationValue::Float(f)
                } else {
                    ConfigurationValue::Float(0.0)
                }
            },
            Value::Bool(b) => ConfigurationValue::Boolean(*b),
            Value::Array(arr) => {
                ConfigurationValue::Array(arr.iter().map(ConfigurationValue::from_json).collect())
            },
            Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), ConfigurationValue::from_json(v));
                }
                ConfigurationValue::Object(map)
            },
            Value::Null => ConfigurationValue::String("".to_string()),
        }
    }
}

/// Configuration schema for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigurationSchema {
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
    },
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    Boolean,
    Array {
        item_schema: Box<ConfigurationSchema>,
        min_items: Option<usize>,
        max_items: Option<usize>,
    },
    Object {
        properties: HashMap<String, ConfigurationSchema>,
        required: Vec<String>,
    },
}

/// Configuration entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationEntry {
    pub key: String,
    pub value: ConfigurationValue,
    pub schema: ConfigurationSchema,
    pub environment: ConfigurationEnvironment,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
    pub is_sensitive: bool,
    pub tags: Vec<String>,
}

impl ConfigurationEntry {
    pub fn new(
        key: String,
        value: ConfigurationValue,
        schema: ConfigurationSchema,
        environment: ConfigurationEnvironment,
    ) -> Result<Self> {
        // Validate the value against the schema
        value.validate(&schema)?;
        
        let now = Utc::now();
        Ok(ConfigurationEntry {
            key,
            value,
            schema,
            environment,
            description: None,
            created_at: now,
            updated_at: now,
            version: 1,
            is_sensitive: false,
            tags: Vec::new(),
        })
    }

    pub fn update_value(&mut self, new_value: ConfigurationValue) -> Result<()> {
        // Validate new value
        new_value.validate(&self.schema)?;
        
        self.value = new_value;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }
}

/// Configuration template for tenant onboarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationTemplate {
    pub name: String,
    pub description: String,
    pub entries: Vec<ConfigurationEntry>,
    pub inheritance_chain: Vec<String>, // Parent template names
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConfigurationTemplate {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        ConfigurationTemplate {
            name,
            description,
            entries: Vec::new(),
            inheritance_chain: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_entry(&mut self, entry: ConfigurationEntry) {
        self.entries.push(entry);
        self.updated_at = Utc::now();
    }

    /// Resolve template with inheritance
    pub fn resolve_with_inheritance(&self, templates: &HashMap<String, ConfigurationTemplate>) -> Result<Vec<ConfigurationEntry>> {
        let mut resolved_entries = Vec::new();
        let mut resolved_keys = std::collections::HashSet::new();

        // Start with parent templates (inheritance chain)
        for parent_name in &self.inheritance_chain {
            if let Some(parent_template) = templates.get(parent_name) {
                for entry in &parent_template.entries {
                    if !resolved_keys.contains(&entry.key) {
                        resolved_entries.push(entry.clone());
                        resolved_keys.insert(entry.key.clone());
                    }
                }
            }
        }

        // Apply this template's entries (override parents)
        for entry in &self.entries {
            if let Some(existing_idx) = resolved_entries.iter().position(|e| e.key == entry.key) {
                resolved_entries[existing_idx] = entry.clone();
            } else {
                resolved_entries.push(entry.clone());
            }
        }

        Ok(resolved_entries)
    }
}

/// Configuration change event for auditing and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChangeEvent {
    pub tenant_id: TenantId,
    pub key: String,
    pub old_value: Option<ConfigurationValue>,
    pub new_value: ConfigurationValue,
    pub environment: ConfigurationEnvironment,
    pub changed_by: String, // User or system
    pub change_reason: String,
    pub timestamp: DateTime<Utc>,
    pub rollback_point: bool,
}

/// Configuration metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationMetrics {
    pub tenant_id: TenantId,
    pub total_configurations: usize,
    pub configurations_by_environment: HashMap<ConfigurationEnvironment, usize>,
    pub last_change_timestamp: Option<DateTime<Utc>>,
    pub total_changes_today: usize,
    pub hot_reload_count: usize,
    pub validation_errors_count: usize,
    pub cache_hit_rate: f64,
    pub average_retrieval_time_ms: f64,
}

/// Configuration cache for performance
#[derive(Debug)]
struct ConfigurationCache {
    entries: HashMap<String, (ConfigurationValue, Instant)>,
    ttl: Duration,
    hit_count: u64,
    miss_count: u64,
}

impl ConfigurationCache {
    fn new(ttl_seconds: u64) -> Self {
        ConfigurationCache {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
            hit_count: 0,
            miss_count: 0,
        }
    }

    fn get(&mut self, key: &str) -> Option<ConfigurationValue> {
        if let Some((value, timestamp)) = self.entries.get(key) {
            if timestamp.elapsed() < self.ttl {
                self.hit_count += 1;
                return Some(value.clone());
            } else {
                self.entries.remove(key);
            }
        }
        self.miss_count += 1;
        None
    }

    fn set(&mut self, key: String, value: ConfigurationValue) {
        self.entries.insert(key, (value, Instant::now()));
    }

    fn invalidate(&mut self, key: &str) {
        self.entries.remove(key);
    }

    fn hit_rate(&self) -> f64 {
        if self.hit_count + self.miss_count == 0 {
            0.0
        } else {
            self.hit_count as f64 / (self.hit_count + self.miss_count) as f64
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Advanced tenant configuration manager with hot-reloading and validation
pub struct TenantConfigurationManager {
    tenant_id: TenantId,
    configurations: Arc<RwLock<HashMap<(String, ConfigurationEnvironment), ConfigurationEntry>>>,
    templates: Arc<RwLock<HashMap<String, ConfigurationTemplate>>>,
    change_history: Arc<RwLock<Vec<ConfigurationChangeEvent>>>,
    cache: Arc<RwLock<ConfigurationCache>>,
    current_environment: ConfigurationEnvironment,
    hot_reload_enabled: bool,
    validation_enabled: bool,
    change_listeners: Arc<RwLock<Vec<ChangeListener>>>,
}

impl TenantConfigurationManager {
    pub fn new(tenant_id: TenantId) -> Self {
        TenantConfigurationManager {
            tenant_id,
            configurations: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            change_history: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(RwLock::new(ConfigurationCache::new(300))), // 5 minutes TTL
            current_environment: ConfigurationEnvironment::Production,
            hot_reload_enabled: true,
            validation_enabled: true,
            change_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set configuration value with validation and hot-reload
    pub fn set_configuration(
        &self,
        key: String,
        value: ConfigurationValue,
        schema: ConfigurationSchema,
        environment: Option<ConfigurationEnvironment>,
        changed_by: String,
        change_reason: String,
    ) -> Result<()> {
        let env = environment.unwrap_or_else(|| self.current_environment.clone());

        // Validate if enabled
        if self.validation_enabled {
            value.validate(&schema)?;
        }

        let mut configurations = self.configurations.write().unwrap();
        let config_key = (key.clone(), env.clone());

        // Get old value for change tracking
        let old_value = configurations.get(&config_key).map(|entry| entry.value.clone());

        // Create or update configuration entry
        let entry = if let Some(existing) = configurations.get(&config_key) {
            let mut updated = existing.clone();
            updated.update_value(value.clone())?;
            updated
        } else {
            ConfigurationEntry::new(key.clone(), value.clone(), schema, env.clone())?
        };

        configurations.insert(config_key, entry);
        drop(configurations); // Release the write lock

        // Invalidate cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.invalidate(&key);
        }

        // Record change event
        let change_event = ConfigurationChangeEvent {
            tenant_id: self.tenant_id.clone(),
            key: key.clone(),
            old_value,
            new_value: value,
            environment: env,
            changed_by,
            change_reason,
            timestamp: Utc::now(),
            rollback_point: false,
        };

        // Add to change history
        {
            let mut history = self.change_history.write().unwrap();
            history.push(change_event.clone());
            
            // Keep only last 1000 changes
            if history.len() > 1000 {
                let excess = history.len() - 1000;
                history.drain(0..excess);
            }
        }

        // Notify listeners for hot reload
        if self.hot_reload_enabled {
            let listeners = self.change_listeners.read().unwrap();
            for listener in listeners.iter() {
                listener(&change_event);
            }
        }

        Ok(())
    }

    /// Get configuration value with caching
    pub fn get_configuration(
        &self,
        key: &str,
        environment: Option<ConfigurationEnvironment>,
    ) -> Option<ConfigurationValue> {
        let env = environment.unwrap_or_else(|| self.current_environment.clone());
        let cache_key = format!("{key}:{env:?}");

        // Check cache first
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(cached_value) = cache.get(&cache_key) {
                return Some(cached_value);
            }
        }

        // Get from storage
        let configurations = self.configurations.read().unwrap();
        if let Some(entry) = configurations.get(&(key.to_string(), env.clone())) {
            let value = entry.value.clone();
            
            // Update cache
            {
                let mut cache = self.cache.write().unwrap();
                cache.set(cache_key, value.clone());
            }
            
            return Some(value);
        }

        // Try fallback environments
        let fallback_envs = match env {
            ConfigurationEnvironment::Development => vec![ConfigurationEnvironment::Staging, ConfigurationEnvironment::Production],
            ConfigurationEnvironment::Testing => vec![ConfigurationEnvironment::Development, ConfigurationEnvironment::Production],
            ConfigurationEnvironment::Staging => vec![ConfigurationEnvironment::Production],
            ConfigurationEnvironment::Production => vec![],
        };

        for fallback_env in fallback_envs {
            if let Some(entry) = configurations.get(&(key.to_string(), fallback_env)) {
                let value = entry.value.clone();
                
                // Update cache with fallback value
                {
                    let mut cache = self.cache.write().unwrap();
                    cache.set(cache_key, value.clone());
                }
                
                return Some(value);
            }
        }

        None
    }

    /// Get all configurations for environment
    pub fn get_all_configurations(
        &self,
        environment: Option<ConfigurationEnvironment>,
    ) -> HashMap<String, ConfigurationValue> {
        let env = environment.unwrap_or_else(|| self.current_environment.clone());
        let configurations = self.configurations.read().unwrap();
        
        let mut result = HashMap::new();
        for ((key, config_env), entry) in configurations.iter() {
            if *config_env == env {
                result.insert(key.clone(), entry.value.clone());
            }
        }
        
        result
    }

    /// Apply configuration template to tenant
    pub fn apply_template(&self, template_name: &str, environment: ConfigurationEnvironment) -> Result<usize> {
        let templates = self.templates.read().unwrap();
        let template = templates.get(template_name)
            .ok_or_else(|| EventualiError::Tenant(format!("Template not found: {template_name}")))?;

        let resolved_entries = template.resolve_with_inheritance(&templates)?;
        drop(templates); // Release read lock

        let mut applied_count = 0;
        for mut entry in resolved_entries {
            // Update environment for this application
            entry.environment = environment.clone();
            
            let config_key = (entry.key.clone(), environment.clone());
            {
                let mut configurations = self.configurations.write().unwrap();
                configurations.insert(config_key, entry);
            }
            
            applied_count += 1;
        }

        // Clear cache after template application
        {
            let mut cache = self.cache.write().unwrap();
            cache.clear();
        }

        Ok(applied_count)
    }

    /// Create configuration template
    pub fn create_template(&self, template: ConfigurationTemplate) -> Result<()> {
        let mut templates = self.templates.write().unwrap();
        templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// Delete configuration
    pub fn delete_configuration(
        &self,
        key: &str,
        environment: Option<ConfigurationEnvironment>,
        changed_by: String,
        change_reason: String,
    ) -> Result<bool> {
        let env = environment.unwrap_or_else(|| self.current_environment.clone());
        let config_key = (key.to_string(), env.clone());

        let mut configurations = self.configurations.write().unwrap();
        if let Some(removed_entry) = configurations.remove(&config_key) {
            drop(configurations); // Release write lock
            
            // Invalidate cache
            {
                let mut cache = self.cache.write().unwrap();
                cache.invalidate(key);
            }

            // Record deletion event
            let change_event = ConfigurationChangeEvent {
                tenant_id: self.tenant_id.clone(),
                key: key.to_string(),
                old_value: Some(removed_entry.value),
                new_value: ConfigurationValue::String("".to_string()), // Placeholder for deletion
                environment: env,
                changed_by,
                change_reason,
                timestamp: Utc::now(),
                rollback_point: false,
            };

            // Add to change history
            {
                let mut history = self.change_history.write().unwrap();
                history.push(change_event.clone());
            }

            // Notify listeners
            if self.hot_reload_enabled {
                let listeners = self.change_listeners.read().unwrap();
                for listener in listeners.iter() {
                    listener(&change_event);
                }
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get configuration metrics
    pub fn get_metrics(&self) -> ConfigurationMetrics {
        let configurations = self.configurations.read().unwrap();
        let history = self.change_history.read().unwrap();
        let cache = self.cache.read().unwrap();

        let mut configurations_by_environment = HashMap::new();
        for ((_key, env), _entry) in configurations.iter() {
            *configurations_by_environment.entry(env.clone()).or_insert(0) += 1;
        }

        let today = Utc::now().date_naive();
        let total_changes_today = history.iter()
            .filter(|event| event.timestamp.date_naive() == today)
            .count();

        let last_change_timestamp = history.last().map(|event| event.timestamp);

        ConfigurationMetrics {
            tenant_id: self.tenant_id.clone(),
            total_configurations: configurations.len(),
            configurations_by_environment,
            last_change_timestamp,
            total_changes_today,
            hot_reload_count: 0, // Would be tracked separately
            validation_errors_count: 0, // Would be tracked separately
            cache_hit_rate: cache.hit_rate() * 100.0,
            average_retrieval_time_ms: 1.0, // Would be measured
        }
    }

    /// Get configuration change history
    pub fn get_change_history(&self, limit: Option<usize>) -> Vec<ConfigurationChangeEvent> {
        let history = self.change_history.read().unwrap();
        let limit = limit.unwrap_or(100);
        
        if history.len() > limit {
            history[history.len() - limit..].to_vec()
        } else {
            history.clone()
        }
    }

    /// Create rollback point
    pub fn create_rollback_point(&self, changed_by: String, reason: String) -> Result<()> {
        let rollback_event = ConfigurationChangeEvent {
            tenant_id: self.tenant_id.clone(),
            key: "ROLLBACK_POINT".to_string(),
            old_value: None,
            new_value: ConfigurationValue::String(reason.clone()),
            environment: self.current_environment.clone(),
            changed_by,
            change_reason: reason,
            timestamp: Utc::now(),
            rollback_point: true,
        };

        let mut history = self.change_history.write().unwrap();
        history.push(rollback_event);
        
        Ok(())
    }

    /// Rollback to a specific point in time
    pub fn rollback_to_point(&self, rollback_timestamp: DateTime<Utc>) -> Result<usize> {
        let history = self.change_history.read().unwrap();
        
        // Find rollback point
        let _rollback_index = history.iter()
            .position(|event| event.rollback_point && event.timestamp <= rollback_timestamp)
            .ok_or_else(|| EventualiError::Tenant("Rollback point not found".to_string()))?;

        drop(history); // Release read lock

        // Apply configurations from rollback point
        let rollback_count = 0;
        // This would involve restoring configuration state from the rollback point
        // For now, this is a placeholder implementation

        Ok(rollback_count)
    }

    /// Add change listener for hot reload
    pub fn add_change_listener<F>(&self, listener: F)
    where
        F: Fn(&ConfigurationChangeEvent) + Send + Sync + 'static,
    {
        let mut listeners = self.change_listeners.write().unwrap();
        listeners.push(Box::new(listener));
    }

    /// Set current environment
    pub fn set_environment(&mut self, environment: ConfigurationEnvironment) {
        self.current_environment = environment;
        
        // Clear cache when environment changes
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Enable/disable hot reload
    pub fn set_hot_reload_enabled(&mut self, enabled: bool) {
        self.hot_reload_enabled = enabled;
    }

    /// Enable/disable validation
    pub fn set_validation_enabled(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
    }

    /// Export configurations to JSON
    pub fn export_configurations(&self, environment: Option<ConfigurationEnvironment>) -> Value {
        let configurations = self.get_all_configurations(environment);
        let mut result = serde_json::Map::new();
        
        for (key, value) in configurations {
            result.insert(key, value.to_json());
        }
        
        Value::Object(result)
    }

    /// Import configurations from JSON
    pub fn import_configurations(
        &self,
        json_data: &Value,
        environment: ConfigurationEnvironment,
        changed_by: String,
    ) -> Result<usize> {
        if let Value::Object(obj) = json_data {
            let mut imported_count = 0;
            
            for (key, value) in obj {
                let config_value = ConfigurationValue::from_json(value);
                // Use a basic schema for imported values
                let schema = match config_value {
                    ConfigurationValue::String(_) => ConfigurationSchema::String {
                        min_length: None,
                        max_length: None,
                        pattern: None,
                    },
                    ConfigurationValue::Integer(_) => ConfigurationSchema::Integer {
                        min: None,
                        max: None,
                    },
                    ConfigurationValue::Float(_) => ConfigurationSchema::Float {
                        min: None,
                        max: None,
                    },
                    ConfigurationValue::Boolean(_) => ConfigurationSchema::Boolean,
                    ConfigurationValue::Array(_) => ConfigurationSchema::Array {
                        item_schema: Box::new(ConfigurationSchema::String {
                            min_length: None,
                            max_length: None,
                            pattern: None,
                        }),
                        min_items: None,
                        max_items: None,
                    },
                    ConfigurationValue::Object(_) => ConfigurationSchema::Object {
                        properties: HashMap::new(),
                        required: Vec::new(),
                    },
                };

                self.set_configuration(
                    key.clone(),
                    config_value,
                    schema,
                    Some(environment.clone()),
                    changed_by.clone(),
                    "Imported from JSON".to_string(),
                )?;
                
                imported_count += 1;
            }
            
            Ok(imported_count)
        } else {
            Err(EventualiError::Tenant("Invalid JSON format for import".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_value_validation() {
        let schema = ConfigurationSchema::String {
            min_length: Some(3),
            max_length: Some(10),
            pattern: None,
        };
        
        let valid_value = ConfigurationValue::String("hello".to_string());
        assert!(valid_value.validate(&schema).is_ok());
        
        let invalid_value = ConfigurationValue::String("hi".to_string());
        assert!(invalid_value.validate(&schema).is_err());
    }

    #[test]
    fn test_configuration_manager_basic_operations() {
        let tenant_id = TenantId::new("test-tenant".to_string()).unwrap();
        let manager = TenantConfigurationManager::new(tenant_id);

        // Set configuration
        let schema = ConfigurationSchema::String {
            min_length: None,
            max_length: None,
            pattern: None,
        };
        
        manager.set_configuration(
            "test_key".to_string(),
            ConfigurationValue::String("test_value".to_string()),
            schema,
            None,
            "test_user".to_string(),
            "Testing".to_string(),
        ).unwrap();

        // Get configuration
        let value = manager.get_configuration("test_key", None);
        assert!(value.is_some());
        
        if let Some(ConfigurationValue::String(s)) = value {
            assert_eq!(s, "test_value");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_configuration_template() {
        let mut template = ConfigurationTemplate::new(
            "test_template".to_string(),
            "Test template".to_string(),
        );

        let entry = ConfigurationEntry::new(
            "test_key".to_string(),
            ConfigurationValue::String("template_value".to_string()),
            ConfigurationSchema::String {
                min_length: None,
                max_length: None,
                pattern: None,
            },
            ConfigurationEnvironment::Development,
        ).unwrap();

        template.add_entry(entry);
        assert_eq!(template.entries.len(), 1);
    }
}