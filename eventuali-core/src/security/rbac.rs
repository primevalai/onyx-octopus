use crate::{EventualiError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// Role-Based Access Control (RBAC) implementation
pub struct RbacManager {
    roles: HashMap<String, Role>,
    users: HashMap<String, User>,
    permissions: HashMap<String, Permission>,
    sessions: HashMap<String, Session>,
    audit_log: Vec<AuditEntry>,
    role_hierarchy: RoleHierarchy,
    #[allow(dead_code)] // Policy engine is part of the RBAC API but not yet implemented
    policy_engine: PolicyEngine,
}

/// User in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub roles: HashSet<String>,
    pub attributes: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub security_level: SecurityLevel,
}

/// Role with permissions and hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub role_id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>,
    pub parent_roles: HashSet<String>,
    pub child_roles: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub is_system_role: bool,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub permission_id: String,
    pub name: String,
    pub description: String,
    pub resource_type: String,
    pub action: String,
    pub conditions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Security level for clearance-based access
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityLevel {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

/// Active session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub permissions_cache: HashSet<String>,
    pub is_active: bool,
}

/// Access control decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessDecision {
    Allow,
    Deny,
    DenyWithReason(String),
}

/// Audit entry for compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub audit_id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub decision: AccessDecision,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub session_id: Option<String>,
    pub reason: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Role hierarchy manager
pub struct RoleHierarchy {
    hierarchy: HashMap<String, HashSet<String>>, // role -> parent roles
}

/// Policy engine for complex access control
pub struct PolicyEngine {
    #[allow(dead_code)] // Policies storage for future dynamic policy evaluation
    policies: HashMap<String, AccessPolicy>,
}

/// Access policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub policy_id: String,
    pub name: String,
    pub resource_pattern: String,
    pub conditions: Vec<PolicyCondition>,
    pub effect: PolicyEffect,
    pub priority: i32,
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub attribute: String,
    pub operator: String,
    pub value: String,
}

/// Policy effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

impl RbacManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        let mut rbac = Self {
            roles: HashMap::new(),
            users: HashMap::new(),
            permissions: HashMap::new(),
            sessions: HashMap::new(),
            audit_log: Vec::new(),
            role_hierarchy: RoleHierarchy::new(),
            policy_engine: PolicyEngine::new(),
        };
        
        rbac.initialize_system_roles();
        rbac
    }
    
    /// Initialize default system roles
    fn initialize_system_roles(&mut self) {
        // System Administrator role
        let admin_role = Role {
            role_id: "system:admin".to_string(),
            name: "System Administrator".to_string(),
            description: "Full system access".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            child_roles: HashSet::new(),
            created_at: Utc::now(),
            is_system_role: true,
        };
        
        // Manager role
        let manager_role = Role {
            role_id: "system:manager".to_string(),
            name: "Manager".to_string(),
            description: "Management access".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            child_roles: HashSet::new(),
            created_at: Utc::now(),
            is_system_role: true,
        };
        
        // Employee role
        let employee_role = Role {
            role_id: "system:employee".to_string(),
            name: "Employee".to_string(),
            description: "Standard employee access".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            child_roles: HashSet::new(),
            created_at: Utc::now(),
            is_system_role: true,
        };
        
        // Guest role
        let guest_role = Role {
            role_id: "system:guest".to_string(),
            name: "Guest".to_string(),
            description: "Limited read-only access".to_string(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            child_roles: HashSet::new(),
            created_at: Utc::now(),
            is_system_role: true,
        };
        
        // Set up hierarchy: Admin > Manager > Employee > Guest
        // Note: In this hierarchy, child roles inherit from parent roles
        // Manager inherits from Employee, Employee inherits from Guest
        self.role_hierarchy.add_parent("system:manager", "system:employee");
        self.role_hierarchy.add_parent("system:employee", "system:guest");
        
        self.roles.insert("system:admin".to_string(), admin_role);
        self.roles.insert("system:manager".to_string(), manager_role);
        self.roles.insert("system:employee".to_string(), employee_role);
        self.roles.insert("system:guest".to_string(), guest_role);
        
        // Initialize system permissions
        self.initialize_system_permissions();
    }
    
    /// Initialize default system permissions
    fn initialize_system_permissions(&mut self) {
        let permissions = vec![
            ("events:read", "Event Store", "read", "Read events from event store"),
            ("events:write", "Event Store", "write", "Write events to event store"),
            ("events:delete", "Event Store", "delete", "Delete events from event store"),
            ("aggregates:read", "Aggregates", "read", "Read aggregates"),
            ("aggregates:write", "Aggregates", "write", "Write aggregates"),
            ("projections:read", "Projections", "read", "Read projections"),
            ("projections:write", "Projections", "write", "Write projections"),
            ("system:admin", "System", "admin", "System administration"),
            ("audit:read", "Audit", "read", "Read audit logs"),
            ("users:manage", "Users", "manage", "Manage users and roles"),
        ];
        
        for (perm_id, resource, action, desc) in permissions {
            let permission = Permission {
                permission_id: perm_id.to_string(),
                name: format!("{resource} {action}"),
                description: desc.to_string(),
                resource_type: resource.to_string(),
                action: action.to_string(),
                conditions: Vec::new(),
                created_at: Utc::now(),
            };
            
            self.permissions.insert(perm_id.to_string(), permission);
        }
        
        // Assign permissions to roles - build hierarchy from most restrictive to most permissive
        
        // Guest role (most restrictive) - read-only access to basic data
        self.assign_permission_to_role("system:guest", "projections:read").unwrap();
        
        // Employee role - inherits Guest + basic event and aggregate access
        self.assign_permission_to_role("system:employee", "events:read").unwrap();
        self.assign_permission_to_role("system:employee", "aggregates:read").unwrap();
        
        // Manager role - inherits Employee + write access and management permissions
        self.assign_permission_to_role("system:manager", "events:write").unwrap();
        self.assign_permission_to_role("system:manager", "aggregates:write").unwrap();
        self.assign_permission_to_role("system:manager", "projections:write").unwrap();
        
        // Admin role - full access to all systems
        self.assign_permission_to_role("system:admin", "system:admin").unwrap();
        self.assign_permission_to_role("system:admin", "users:manage").unwrap();
        self.assign_permission_to_role("system:admin", "events:delete").unwrap();
        self.assign_permission_to_role("system:admin", "audit:read").unwrap();
    }
    
    /// Create a new user
    pub fn create_user(
        &mut self,
        username: String,
        email: String,
        security_level: SecurityLevel,
    ) -> Result<String> {
        let user_id = Uuid::new_v4().to_string();
        
        let user = User {
            user_id: user_id.clone(),
            username: username.clone(),
            email,
            roles: HashSet::new(),
            attributes: HashMap::new(),
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
            security_level,
        };
        
        self.users.insert(user_id.clone(), user);
        
        self.audit_log.push(AuditEntry {
            audit_id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            action: "user:create".to_string(),
            resource: "user".to_string(),
            resource_id: Some(user_id.clone()),
            decision: AccessDecision::Allow,
            timestamp: Utc::now(),
            ip_address: None,
            session_id: None,
            reason: Some(format!("User {username} created")),
            metadata: HashMap::new(),
        });
        
        Ok(user_id)
    }
    
    /// Assign role to user
    pub fn assign_role_to_user(&mut self, user_id: &str, role_id: &str) -> Result<()> {
        if !self.users.contains_key(user_id) {
            return Err(EventualiError::Validation(format!("User {user_id} not found")));
        }
        
        if !self.roles.contains_key(role_id) {
            return Err(EventualiError::Validation(format!("Role {role_id} not found")));
        }
        
        let user = self.users.get_mut(user_id).unwrap();
        user.roles.insert(role_id.to_string());
        
        self.audit_log.push(AuditEntry {
            audit_id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            action: "role:assign".to_string(),
            resource: "user".to_string(),
            resource_id: Some(user_id.to_string()),
            decision: AccessDecision::Allow,
            timestamp: Utc::now(),
            ip_address: None,
            session_id: None,
            reason: Some(format!("Role {role_id} assigned to user")),
            metadata: HashMap::new(),
        });
        
        Ok(())
    }
    
    /// Create role
    pub fn create_role(&mut self, name: String, description: String) -> Result<String> {
        let role_id = format!("custom:{}", Uuid::new_v4());
        
        let role = Role {
            role_id: role_id.clone(),
            name: name.clone(),
            description,
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            child_roles: HashSet::new(),
            created_at: Utc::now(),
            is_system_role: false,
        };
        
        self.roles.insert(role_id.clone(), role);
        
        self.audit_log.push(AuditEntry {
            audit_id: Uuid::new_v4().to_string(),
            user_id: "system".to_string(),
            action: "role:create".to_string(),
            resource: "role".to_string(),
            resource_id: Some(role_id.clone()),
            decision: AccessDecision::Allow,
            timestamp: Utc::now(),
            ip_address: None,
            session_id: None,
            reason: Some(format!("Role {name} created")),
            metadata: HashMap::new(),
        });
        
        Ok(role_id)
    }
    
    /// Assign permission to role
    pub fn assign_permission_to_role(&mut self, role_id: &str, permission_id: &str) -> Result<()> {
        if !self.roles.contains_key(role_id) {
            return Err(EventualiError::Validation(format!("Role {role_id} not found")));
        }
        
        if !self.permissions.contains_key(permission_id) {
            return Err(EventualiError::Validation(format!("Permission {permission_id} not found")));
        }
        
        let role = self.roles.get_mut(role_id).unwrap();
        role.permissions.insert(permission_id.to_string());
        
        Ok(())
    }
    
    /// Authenticate user and create session
    pub fn authenticate(&mut self, username: &str, password: &str, ip_address: Option<String>) -> Result<String> {
        let user_info = {
            let user = self.users.values()
                .find(|u| u.username == username && u.is_active)
                .ok_or_else(|| EventualiError::Authentication("Invalid credentials".to_string()))?;
            (user.user_id.clone(), user.username.clone())
        };
        
        // In production, verify password hash
        if self.verify_password(password) {
            let session_id = Uuid::new_v4().to_string();
            let token = self.generate_session_token(&user_info.0);
            
            let session = Session {
                session_id: session_id.clone(),
                user_id: user_info.0.clone(),
                token: token.clone(),
                created_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(8),
                ip_address: ip_address.clone(),
                user_agent: None,
                permissions_cache: self.get_effective_permissions(&user_info.0)?,
                is_active: true,
            };
            
            self.sessions.insert(session_id.clone(), session);
            
            // Update user last login
            let user = self.users.get_mut(&user_info.0).unwrap();
            user.last_login = Some(Utc::now());
            
            self.audit_log.push(AuditEntry {
                audit_id: Uuid::new_v4().to_string(),
                user_id: user_info.0.clone(),
                action: "authentication:success".to_string(),
                resource: "session".to_string(),
                resource_id: Some(session_id.clone()),
                decision: AccessDecision::Allow,
                timestamp: Utc::now(),
                ip_address,
                session_id: Some(session_id.clone()),
                reason: Some("Authentication successful".to_string()),
                metadata: HashMap::new(),
            });
            
            Ok(token)
        } else {
            self.audit_log.push(AuditEntry {
                audit_id: Uuid::new_v4().to_string(),
                user_id: user_info.0.clone(),
                action: "authentication:failure".to_string(),
                resource: "session".to_string(),
                resource_id: None,
                decision: AccessDecision::Deny,
                timestamp: Utc::now(),
                ip_address,
                session_id: None,
                reason: Some("Invalid password".to_string()),
                metadata: HashMap::new(),
            });
            
            Err(EventualiError::Authentication("Invalid credentials".to_string()))
        }
    }
    
    /// Check access permission
    pub fn check_access(
        &mut self,
        token: &str,
        resource: &str,
        action: &str,
        context: Option<HashMap<String, String>>,
    ) -> AccessDecision {
        // Clone necessary data to avoid borrowing conflicts
        let session_data = match self.get_session_by_token(token) {
            Some(session) if session.is_active && session.expires_at > Utc::now() => {
                (session.user_id.clone(), session.permissions_cache.clone())
            },
            _ => {
                let decision = AccessDecision::DenyWithReason("Invalid or expired token".to_string());
                self.audit_access(None, resource, action, decision.clone(), context);
                return decision;
            }
        };
        
        let user_active = match self.users.get(&session_data.0) {
            Some(user) if user.is_active => true,
            _ => {
                let decision = AccessDecision::DenyWithReason("User inactive".to_string());
                self.audit_access(Some(&session_data.0), resource, action, decision.clone(), context);
                return decision;
            }
        };
        
        if !user_active {
            let decision = AccessDecision::DenyWithReason("User inactive".to_string());
            self.audit_access(Some(&session_data.0), resource, action, decision.clone(), context);
            return decision;
        }
        
        // Check permission
        let permission_id = format!("{resource}:{action}");
        let decision = if session_data.1.contains(&permission_id) {
            AccessDecision::Allow
        } else {
            AccessDecision::DenyWithReason(format!("Permission {permission_id} not granted"))
        };
        
        self.audit_access(Some(&session_data.0), resource, action, decision.clone(), context);
        decision
    }
    
    /// Get effective permissions for user (including hierarchy)
    pub fn get_effective_permissions(&self, user_id: &str) -> Result<HashSet<String>> {
        let user = self.users.get(user_id)
            .ok_or_else(|| EventualiError::Validation("User not found".to_string()))?;
        
        let mut permissions = HashSet::new();
        
        for role_id in &user.roles {
            if let Some(role_permissions) = self.get_role_permissions_with_hierarchy(role_id) {
                permissions.extend(role_permissions);
            }
        }
        
        Ok(permissions)
    }
    
    /// Get role permissions including inherited ones
    fn get_role_permissions_with_hierarchy(&self, role_id: &str) -> Option<HashSet<String>> {
        let mut permissions = HashSet::new();
        let mut visited = HashSet::new();
        
        self.collect_role_permissions(role_id, &mut permissions, &mut visited);
        
        if permissions.is_empty() {
            None
        } else {
            Some(permissions)
        }
    }
    
    /// Recursively collect permissions from role hierarchy
    fn collect_role_permissions(&self, role_id: &str, permissions: &mut HashSet<String>, visited: &mut HashSet<String>) {
        if visited.contains(role_id) {
            return; // Prevent infinite loops
        }
        visited.insert(role_id.to_string());
        
        if let Some(role) = self.roles.get(role_id) {
            permissions.extend(role.permissions.iter().cloned());
            
            // Collect permissions from parent roles
            if let Some(parent_roles) = self.role_hierarchy.hierarchy.get(role_id) {
                for parent_role in parent_roles {
                    self.collect_role_permissions(parent_role, permissions, visited);
                }
            }
        }
    }
    
    /// Generate session token
    fn generate_session_token(&self, user_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        Utc::now().timestamp_nanos_opt().unwrap_or_default().hash(&mut hasher);
        
        format!("eventuali_token_{:x}", hasher.finish())
    }
    
    /// Get session by token
    fn get_session_by_token(&self, token: &str) -> Option<&Session> {
        self.sessions.values().find(|session| session.token == token)
    }
    
    /// Verify password (simplified for example)
    fn verify_password(&self, _password: &str) -> bool {
        // In production, use proper password hashing (bcrypt, scrypt, argon2)
        true
    }
    
    /// Audit access attempt
    fn audit_access(
        &mut self,
        user_id: Option<&str>,
        resource: &str,
        action: &str,
        decision: AccessDecision,
        context: Option<HashMap<String, String>>,
    ) {
        self.audit_log.push(AuditEntry {
            audit_id: Uuid::new_v4().to_string(),
            user_id: user_id.unwrap_or("unknown").to_string(),
            action: format!("{resource}:{action}"),
            resource: resource.to_string(),
            resource_id: None,
            decision,
            timestamp: Utc::now(),
            ip_address: None,
            session_id: None,
            reason: None,
            metadata: context.unwrap_or_default(),
        });
    }
    
    /// Get audit trail
    pub fn get_audit_trail(&self, limit: Option<usize>) -> Vec<&AuditEntry> {
        let limit = limit.unwrap_or(100);
        self.audit_log.iter().rev().take(limit).collect()
    }
    
    /// Revoke session
    pub fn revoke_session(&mut self, token: &str) -> Result<()> {
        if let Some(session) = self.sessions.values_mut().find(|s| s.token == token) {
            session.is_active = false;
            
            self.audit_log.push(AuditEntry {
                audit_id: Uuid::new_v4().to_string(),
                user_id: session.user_id.clone(),
                action: "session:revoke".to_string(),
                resource: "session".to_string(),
                resource_id: Some(session.session_id.clone()),
                decision: AccessDecision::Allow,
                timestamp: Utc::now(),
                ip_address: None,
                session_id: Some(session.session_id.clone()),
                reason: Some("Session revoked".to_string()),
                metadata: HashMap::new(),
            });
            
            Ok(())
        } else {
            Err(EventualiError::Validation("Session not found".to_string()))
        }
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = Utc::now();
        let expired_sessions: Vec<String> = self.sessions
            .iter()
            .filter(|(_, session)| session.expires_at < now)
            .map(|(id, _)| id.clone())
            .collect();
        
        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
        }
    }
    
    /// Get system statistics
    pub fn get_system_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_users".to_string(), serde_json::Value::Number(self.users.len().into()));
        stats.insert("active_users".to_string(), serde_json::Value::Number(
            self.users.values().filter(|u| u.is_active).count().into()
        ));
        stats.insert("total_roles".to_string(), serde_json::Value::Number(self.roles.len().into()));
        stats.insert("total_permissions".to_string(), serde_json::Value::Number(self.permissions.len().into()));
        stats.insert("active_sessions".to_string(), serde_json::Value::Number(
            self.sessions.values().filter(|s| s.is_active && s.expires_at > Utc::now()).count().into()
        ));
        stats.insert("audit_entries".to_string(), serde_json::Value::Number(self.audit_log.len().into()));
        
        stats
    }
}

impl RoleHierarchy {
    fn new() -> Self {
        Self {
            hierarchy: HashMap::new(),
        }
    }
    
    fn add_parent(&mut self, child_role: &str, parent_role: &str) {
        self.hierarchy
            .entry(child_role.to_string())
            .or_default()
            .insert(parent_role.to_string());
    }
}

impl PolicyEngine {
    fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }
}

impl SecurityLevel {
    pub fn level_value(&self) -> u8 {
        match self {
            SecurityLevel::Public => 0,
            SecurityLevel::Internal => 1,
            SecurityLevel::Confidential => 2,
            SecurityLevel::Secret => 3,
            SecurityLevel::TopSecret => 4,
        }
    }
    
    pub fn can_access(&self, required_level: &SecurityLevel) -> bool {
        self.level_value() >= required_level.level_value()
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_initialization() {
        let rbac = RbacManager::new();
        assert_eq!(rbac.roles.len(), 4); // 4 system roles
        assert!(rbac.roles.contains_key("system:admin"));
        assert!(rbac.roles.contains_key("system:manager"));
        assert!(rbac.roles.contains_key("system:employee"));
        assert!(rbac.roles.contains_key("system:guest"));
    }

    #[test]
    fn test_user_creation() {
        let mut rbac = RbacManager::new();
        let user_id = rbac.create_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            SecurityLevel::Internal,
        ).unwrap();
        
        assert!(rbac.users.contains_key(&user_id));
        let user = rbac.users.get(&user_id).unwrap();
        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.security_level, SecurityLevel::Internal);
    }

    #[test]
    fn test_role_assignment() {
        let mut rbac = RbacManager::new();
        let user_id = rbac.create_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            SecurityLevel::Internal,
        ).unwrap();
        
        rbac.assign_role_to_user(&user_id, "system:manager").unwrap();
        
        let user = rbac.users.get(&user_id).unwrap();
        assert!(user.roles.contains("system:manager"));
    }

    #[test]
    fn test_permission_inheritance() {
        let mut rbac = RbacManager::new();
        let user_id = rbac.create_user(
            "admin_user".to_string(),
            "admin@example.com".to_string(),
            SecurityLevel::Secret,
        ).unwrap();
        
        rbac.assign_role_to_user(&user_id, "system:admin").unwrap();
        
        let permissions = rbac.get_effective_permissions(&user_id).unwrap();
        assert!(permissions.contains("system:admin"));
        assert!(permissions.contains("events:read"));
        assert!(permissions.contains("events:write"));
    }

    #[test]
    fn test_authentication_flow() {
        let mut rbac = RbacManager::new();
        let user_id = rbac.create_user(
            "auth_user".to_string(),
            "auth@example.com".to_string(),
            SecurityLevel::Internal,
        ).unwrap();
        
        rbac.assign_role_to_user(&user_id, "system:employee").unwrap();
        
        let token = rbac.authenticate("auth_user", "password", Some("192.168.1.1".to_string())).unwrap();
        assert!(!token.is_empty());
        
        // Test access
        let decision = rbac.check_access(&token, "events", "read", None);
        assert!(matches!(decision, AccessDecision::Allow));
        
        let decision = rbac.check_access(&token, "events", "delete", None);
        assert!(matches!(decision, AccessDecision::DenyWithReason(_)));
    }

    #[test]
    fn test_security_levels() {
        assert!(SecurityLevel::Secret.can_access(&SecurityLevel::Internal));
        assert!(!SecurityLevel::Internal.can_access(&SecurityLevel::Secret));
        assert!(SecurityLevel::TopSecret.can_access(&SecurityLevel::Public));
    }
}