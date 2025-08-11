use async_trait::async_trait;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use adx_shared::{
    temporal::{
        ActivityContext, AdxActivity, TenantAwareActivity, DatabaseActivity, 
        ActivityError, utils::database_retry_policy
    },
    auth::{UserContext, TenantContext},
    database::DatabasePool,
    Error, Result,
};

use crate::repositories::{UserRepository, user::{User, UserStatus}};

/// Request for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub send_verification_email: bool,
}

/// Response from creating a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub user_id: String,
    pub email: String,
    pub status: UserStatus,
    pub verification_required: bool,
    pub created_at: DateTime<Utc>,
}

/// Activity for creating new users with password hashing and validation
pub struct CreateUserActivity {
    database_pool: DatabasePool,
}

impl CreateUserActivity {
    pub fn new(database_pool: DatabasePool) -> Self {
        Self { database_pool }
    }

    /// Validate email format
    fn validate_email(&self, email: &str) -> Result<(), ActivityError> {
        if email.is_empty() {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Email cannot be empty".to_string(),
            });
        }

        // Basic email validation
        if !email.contains('@') || !email.contains('.') {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }

        // Check email length
        if email.len() > 255 {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Email too long (max 255 characters)".to_string(),
            });
        }

        Ok(())
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> Result<(), ActivityError> {
        if password.is_empty() {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password cannot be empty".to_string(),
            });
        }

        if password.len() < 8 {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password must be at least 8 characters long".to_string(),
            });
        }

        if password.len() > 128 {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password too long (max 128 characters)".to_string(),
            });
        }

        // Check for at least one uppercase letter
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password must contain at least one uppercase letter".to_string(),
            });
        }

        // Check for at least one lowercase letter
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password must contain at least one lowercase letter".to_string(),
            });
        }

        // Check for at least one digit
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password must contain at least one digit".to_string(),
            });
        }

        // Check for at least one special character
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        if !password.chars().any(|c| special_chars.contains(c)) {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password must contain at least one special character".to_string(),
            });
        }

        Ok(())
    }

    /// Hash password using bcrypt
    fn hash_password(&self, password: &str) -> Result<String, ActivityError> {
        hash(password, DEFAULT_COST).map_err(|e| ActivityError::InternalError {
            message: format!("Failed to hash password: {}", e),
        })
    }

    /// Check if user already exists
    async fn check_user_exists(&self, tenant_id: &str, email: &str) -> Result<bool, ActivityError> {
        let user_repo = UserRepository::new(self.database_pool.clone(), tenant_id.to_string());
        
        match user_repo.find_by_email(email).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(ActivityError::DatabaseError {
                message: format!("Failed to check if user exists: {}", e),
            }),
        }
    }
}

#[async_trait]
impl AdxActivity<CreateUserRequest, CreateUserResponse> for CreateUserActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: CreateUserRequest,
    ) -> Result<CreateUserResponse, ActivityError> {
        // Validate input
        self.validate_input(&input)?;

        // Validate email format
        self.validate_email(&input.email)?;

        // Validate password strength
        self.validate_password(&input.password)?;

        // Check if user already exists
        if self.check_user_exists(&context.tenant_context.tenant_id, &input.email).await? {
            return Err(ActivityError::ConflictError {
                message: format!("User with email {} already exists", input.email),
            });
        }

        // Hash password
        let password_hash = self.hash_password(&input.password)?;

        // Create user
        let user_repo = UserRepository::new(
            self.database_pool.clone(), 
            context.tenant_context.tenant_id.clone()
        );

        let user = User {
            id: Uuid::new_v4().to_string(),
            tenant_id: context.tenant_context.tenant_id.clone(),
            email: input.email.clone(),
            password_hash,
            first_name: input.first_name,
            last_name: input.last_name,
            status: if input.send_verification_email {
                UserStatus::PendingVerification
            } else {
                UserStatus::Active
            },
            roles: input.roles.unwrap_or_else(|| vec!["user".to_string()]),
            permissions: input.permissions.unwrap_or_default(),
            preferences: serde_json::json!({}),
            last_login_at: None,
            email_verified_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_user = user_repo.create(user).await.map_err(|e| ActivityError::DatabaseError {
            message: format!("Failed to create user: {}", e),
        })?;

        Ok(CreateUserResponse {
            user_id: created_user.id,
            email: created_user.email,
            status: created_user.status,
            verification_required: input.send_verification_email,
            created_at: created_user.created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "create_user_activity"
    }

    fn validate_input(&self, input: &CreateUserRequest) -> Result<(), ActivityError> {
        if input.email.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
            });
        }

        if input.password.is_empty() {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password is required".to_string(),
            });
        }

        // Validate name lengths if provided
        if let Some(ref first_name) = input.first_name {
            if first_name.len() > 100 {
                return Err(ActivityError::ValidationError {
                    field: "first_name".to_string(),
                    message: "First name too long (max 100 characters)".to_string(),
                });
            }
        }

        if let Some(ref last_name) = input.last_name {
            if last_name.len() > 100 {
                return Err(ActivityError::ValidationError {
                    field: "last_name".to_string(),
                    message: "Last name too long (max 100 characters)".to_string(),
                });
            }
        }

        // Validate roles if provided
        if let Some(ref roles) = input.roles {
            if roles.is_empty() {
                return Err(ActivityError::ValidationError {
                    field: "roles".to_string(),
                    message: "Roles cannot be empty if provided".to_string(),
                });
            }

            for role in roles {
                if role.trim().is_empty() {
                    return Err(ActivityError::ValidationError {
                        field: "roles".to_string(),
                        message: "Role names cannot be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn default_options(&self) -> adx_shared::temporal::ActivityExecutionOptions {
        let mut options = adx_shared::temporal::ActivityExecutionOptions::default();
        options.retry_policy = Some(database_retry_policy());
        options.tags.push("user_creation".to_string());
        options.tags.push("authentication".to_string());
        options
    }
}

#[async_trait]
impl TenantAwareActivity<CreateUserRequest, CreateUserResponse> for CreateUserActivity {
    async fn validate_tenant_access(
        &self,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<(), ActivityError> {
        // Check if user has permission to create users in this tenant
        if !user_context.permissions.contains(&"user:create".to_string()) &&
           !user_context.roles.contains(&"admin".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Insufficient permissions to create users".to_string(),
            });
        }

        // Check if tenant is active
        if !tenant_context.is_active {
            return Err(ActivityError::AuthorizationError {
                message: "Cannot create users in inactive tenant".to_string(),
            });
        }

        Ok(())
    }

    async fn check_tenant_quotas(
        &self,
        tenant_context: &TenantContext,
        resource_type: &str,
        requested_amount: u64,
    ) -> Result<(), ActivityError> {
        if resource_type == "users" {
            // Check user quota
            if let Some(max_users) = tenant_context.quotas.max_users {
                let user_repo = UserRepository::new(
                    self.database_pool.clone(), 
                    tenant_context.tenant_id.clone()
                );
                
                let current_count = user_repo.count(None).await.map_err(|e| ActivityError::DatabaseError {
                    message: format!("Failed to count users: {}", e),
                })? as u32;

                if current_count + requested_amount as u32 > max_users {
                    return Err(ActivityError::QuotaExceededError {
                        resource_type: "users".to_string(),
                        current_usage: current_count as u64,
                        limit: max_users as u64,
                        requested: requested_amount,
                    });
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl DatabaseActivity<CreateUserRequest, CreateUserResponse> for CreateUserActivity {
    async fn get_tenant_connection(
        &self,
        _tenant_context: &TenantContext,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, ActivityError> {
        Ok(Box::new(self.database_pool.clone()))
    }

    async fn execute_transaction<F, R>(
        &self,
        _tenant_context: &TenantContext,
        transaction: F,
    ) -> Result<R, ActivityError>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, ActivityError>> + Send>> + Send,
        R: Send + Sync,
    {
        // For now, execute without explicit transaction
        // TODO: Implement proper transaction support when needed
        transaction().await
    }
}

// Tests commented out for now due to compilation issues
// #[cfg(test)]
// mod tests {
//     // Test implementations would go here
// }