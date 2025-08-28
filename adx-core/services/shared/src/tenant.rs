// Tenant management utilities

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{Result, ServiceError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Professional,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub admin_email: String,
    pub subscription_tier: SubscriptionTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_name: String,
    pub subscription_tier: SubscriptionTier,
    pub features: Vec<String>,
    pub quotas: TenantQuotas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    pub max_users: u32,
    pub max_storage_gb: u32,
    pub max_api_calls_per_hour: u32,
    pub max_workflows_per_hour: u32,
}

impl Default for TenantQuotas {
    fn default() -> Self {
        Self {
            max_users: 10,
            max_storage_gb: 5,
            max_api_calls_per_hour: 1000,
            max_workflows_per_hour: 100,
        }
    }
}

impl TenantQuotas {
    pub fn for_tier(tier: &SubscriptionTier) -> Self {
        match tier {
            SubscriptionTier::Free => Self {
                max_users: 5,
                max_storage_gb: 1,
                max_api_calls_per_hour: 100,
                max_workflows_per_hour: 10,
            },
            SubscriptionTier::Professional => Self {
                max_users: 50,
                max_storage_gb: 100,
                max_api_calls_per_hour: 10000,
                max_workflows_per_hour: 1000,
            },
            SubscriptionTier::Enterprise => Self {
                max_users: u32::MAX,
                max_storage_gb: u32::MAX,
                max_api_calls_per_hour: u32::MAX,
                max_workflows_per_hour: u32::MAX,
            },
        }
    }
}

pub struct TenantManager {
    // In a real implementation, this would have database connections, etc.
}

impl TenantManager {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn validate_tenant_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(ServiceError::Validation("Tenant name cannot be empty".to_string()));
        }
        
        if name.len() < 3 {
            return Err(ServiceError::Validation("Tenant name must be at least 3 characters".to_string()));
        }
        
        if name.len() > 50 {
            return Err(ServiceError::Validation("Tenant name cannot exceed 50 characters".to_string()));
        }
        
        // Check for valid characters
        if !name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
            return Err(ServiceError::Validation("Tenant name contains invalid characters".to_string()));
        }
        
        Ok(())
    }
    
    pub fn get_features_for_tier(&self, tier: &SubscriptionTier) -> Vec<String> {
        match tier {
            SubscriptionTier::Free => vec![
                "basic_auth".to_string(),
                "file_storage".to_string(),
            ],
            SubscriptionTier::Professional => vec![
                "basic_auth".to_string(),
                "file_storage".to_string(),
                "advanced_workflows".to_string(),
                "api_access".to_string(),
                "email_support".to_string(),
            ],
            SubscriptionTier::Enterprise => vec![
                "basic_auth".to_string(),
                "file_storage".to_string(),
                "advanced_workflows".to_string(),
                "api_access".to_string(),
                "email_support".to_string(),
                "sso_integration".to_string(),
                "custom_modules".to_string(),
                "priority_support".to_string(),
                "white_label".to_string(),
            ],
        }
    }
    
    pub fn create_tenant_context(&self, tenant: &Tenant) -> TenantContext {
        TenantContext {
            tenant_id: tenant.id.clone(),
            tenant_name: tenant.name.clone(),
            subscription_tier: tenant.subscription_tier.clone(),
            features: self.get_features_for_tier(&tenant.subscription_tier),
            quotas: TenantQuotas::for_tier(&tenant.subscription_tier),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tenant() -> Tenant {
        Tenant {
            id: "test-tenant-123".to_string(),
            name: "Test Tenant".to_string(),
            admin_email: "admin@test.com".to_string(),
            subscription_tier: SubscriptionTier::Professional,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        }
    }

    #[test]
    fn test_tenant_quotas_for_tier() {
        let free_quotas = TenantQuotas::for_tier(&SubscriptionTier::Free);
        assert_eq!(free_quotas.max_users, 5);
        assert_eq!(free_quotas.max_storage_gb, 1);
        
        let pro_quotas = TenantQuotas::for_tier(&SubscriptionTier::Professional);
        assert_eq!(pro_quotas.max_users, 50);
        assert_eq!(pro_quotas.max_storage_gb, 100);
        
        let enterprise_quotas = TenantQuotas::for_tier(&SubscriptionTier::Enterprise);
        assert_eq!(enterprise_quotas.max_users, u32::MAX);
    }

    #[test]
    fn test_tenant_name_validation() {
        let manager = TenantManager::new();
        
        // Valid names
        assert!(manager.validate_tenant_name("Valid Tenant").is_ok());
        assert!(manager.validate_tenant_name("Test-Tenant_123").is_ok());
        
        // Invalid names
        assert!(manager.validate_tenant_name("").is_err());
        assert!(manager.validate_tenant_name("AB").is_err());
        assert!(manager.validate_tenant_name(&"A".repeat(51)).is_err());
        assert!(manager.validate_tenant_name("Invalid@Tenant").is_err());
    }

    #[test]
    fn test_features_for_tier() {
        let manager = TenantManager::new();
        
        let free_features = manager.get_features_for_tier(&SubscriptionTier::Free);
        assert_eq!(free_features.len(), 2);
        assert!(free_features.contains(&"basic_auth".to_string()));
        
        let pro_features = manager.get_features_for_tier(&SubscriptionTier::Professional);
        assert_eq!(pro_features.len(), 5);
        assert!(pro_features.contains(&"advanced_workflows".to_string()));
        
        let enterprise_features = manager.get_features_for_tier(&SubscriptionTier::Enterprise);
        assert_eq!(enterprise_features.len(), 9);
        assert!(enterprise_features.contains(&"white_label".to_string()));
    }

    #[test]
    fn test_create_tenant_context() {
        let manager = TenantManager::new();
        let tenant = create_test_tenant();
        
        let context = manager.create_tenant_context(&tenant);
        
        assert_eq!(context.tenant_id, tenant.id);
        assert_eq!(context.tenant_name, tenant.name);
        assert_eq!(context.subscription_tier, tenant.subscription_tier);
        assert!(!context.features.is_empty());
        assert_eq!(context.quotas.max_users, 50); // Professional tier
    }

    #[test]
    fn test_subscription_tier_serialization() {
        let tier = SubscriptionTier::Professional;
        let serialized = serde_json::to_string(&tier).unwrap();
        let deserialized: SubscriptionTier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tier, deserialized);
    }
}