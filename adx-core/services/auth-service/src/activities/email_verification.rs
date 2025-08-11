use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use adx_shared::{
    temporal::{
        ActivityContext, AdxActivity, TenantAwareActivity, ExternalServiceActivity,
        ActivityError, utils::external_service_retry_policy
    },
    auth::{UserContext, TenantContext},
    database::DatabasePool,
    Error, Result,
};

use crate::repositories::{AuthTokenRepository, auth_token::{AuthToken, TokenType}};

/// Request for sending verification email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVerificationEmailRequest {
    pub user_id: String,
    pub email: String,
    pub user_name: Option<String>,
    pub verification_url_base: String,
    pub template_name: Option<String>,
    pub language: Option<String>,
}

/// Response from sending verification email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVerificationEmailResponse {
    pub email_sent: bool,
    pub token_id: String,
    pub expires_at: DateTime<Utc>,
    pub email_provider: String,
    pub message_id: Option<String>,
}

/// Email template data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

/// Activity for sending verification emails with templates
pub struct SendVerificationEmailActivity {
    database_pool: DatabasePool,
    email_service_url: String,
    email_service_api_key: String,
    default_from_email: String,
    default_from_name: String,
}

impl SendVerificationEmailActivity {
    pub fn new(
        database_pool: DatabasePool,
        email_service_url: String,
        email_service_api_key: String,
        default_from_email: String,
        default_from_name: String,
    ) -> Self {
        Self {
            database_pool,
            email_service_url,
            email_service_api_key,
            default_from_email,
            default_from_name,
        }
    }

    /// Generate verification token
    async fn generate_verification_token(
        &self,
        tenant_id: &str,
        user_id: &str,
        email: &str,
    ) -> Result<AuthToken, ActivityError> {
        let token_repo = AuthTokenRepository::new(
            self.database_pool.clone(),
            tenant_id.to_string(),
        );

        // Invalidate any existing verification tokens for this user
        if let Err(e) = token_repo.invalidate_user_tokens(user_id, TokenType::EmailVerification).await {
            tracing::warn!("Failed to invalidate existing tokens: {}", e);
        }

        // Generate new token
        let token = AuthToken {
            id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            token_type: TokenType::EmailVerification,
            token_hash: Uuid::new_v4().to_string(), // This would be a secure random token in production
            metadata: serde_json::json!({
                "email": email,
                "purpose": "email_verification"
            }),
            expires_at: Utc::now() + Duration::hours(24), // 24 hour expiry
            used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        token_repo.create(token).await.map_err(|e| ActivityError::DatabaseError {
            message: format!("Failed to create verification token: {}", e),
        })
    }

    /// Get email template based on language and template name
    fn get_email_template(
        &self,
        template_name: &str,
        language: &str,
        user_name: Option<&str>,
        verification_url: &str,
    ) -> EmailTemplate {
        let display_name = user_name.unwrap_or("User");
        
        match (template_name, language) {
            ("welcome_verification", "es") => EmailTemplate {
                subject: "Verifica tu cuenta - ADX Core".to_string(),
                html_body: format!(
                    r#"
                    <html>
                    <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                        <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                            <h1 style="color: #2c3e50;">¡Bienvenido a ADX Core!</h1>
                            <p>Hola {},</p>
                            <p>Gracias por registrarte en ADX Core. Para completar tu registro, por favor verifica tu dirección de correo electrónico haciendo clic en el botón de abajo:</p>
                            <div style="text-align: center; margin: 30px 0;">
                                <a href="{}" style="background-color: #3498db; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">Verificar Email</a>
                            </div>
                            <p>Si no puedes hacer clic en el botón, copia y pega este enlace en tu navegador:</p>
                            <p style="word-break: break-all; color: #7f8c8d;">{}</p>
                            <p>Este enlace expirará en 24 horas por razones de seguridad.</p>
                            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
                            <p style="font-size: 12px; color: #7f8c8d;">Si no creaste esta cuenta, puedes ignorar este email de forma segura.</p>
                        </div>
                    </body>
                    </html>
                    "#,
                    display_name, verification_url, verification_url
                ),
                text_body: format!(
                    "¡Bienvenido a ADX Core!\n\nHola {},\n\nGracias por registrarte en ADX Core. Para completar tu registro, por favor verifica tu dirección de correo electrónico visitando este enlace:\n\n{}\n\nEste enlace expirará en 24 horas por razones de seguridad.\n\nSi no creaste esta cuenta, puedes ignorar este email de forma segura.",
                    display_name, verification_url
                ),
            },
            ("welcome_verification", "fr") => EmailTemplate {
                subject: "Vérifiez votre compte - ADX Core".to_string(),
                html_body: format!(
                    r#"
                    <html>
                    <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                        <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                            <h1 style="color: #2c3e50;">Bienvenue sur ADX Core !</h1>
                            <p>Bonjour {},</p>
                            <p>Merci de vous être inscrit sur ADX Core. Pour terminer votre inscription, veuillez vérifier votre adresse e-mail en cliquant sur le bouton ci-dessous :</p>
                            <div style="text-align: center; margin: 30px 0;">
                                <a href="{}" style="background-color: #3498db; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">Vérifier l'Email</a>
                            </div>
                            <p>Si vous ne pouvez pas cliquer sur le bouton, copiez et collez ce lien dans votre navigateur :</p>
                            <p style="word-break: break-all; color: #7f8c8d;">{}</p>
                            <p>Ce lien expirera dans 24 heures pour des raisons de sécurité.</p>
                            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
                            <p style="font-size: 12px; color: #7f8c8d;">Si vous n'avez pas créé ce compte, vous pouvez ignorer cet e-mail en toute sécurité.</p>
                        </div>
                    </body>
                    </html>
                    "#,
                    display_name, verification_url, verification_url
                ),
                text_body: format!(
                    "Bienvenue sur ADX Core !\n\nBonjour {},\n\nMerci de vous être inscrit sur ADX Core. Pour terminer votre inscription, veuillez vérifier votre adresse e-mail en visitant ce lien :\n\n{}\n\nCe lien expirera dans 24 heures pour des raisons de sécurité.\n\nSi vous n'avez pas créé ce compte, vous pouvez ignorer cet e-mail en toute sécurité.",
                    display_name, verification_url
                ),
            },
            _ => EmailTemplate {
                subject: "Verify your account - ADX Core".to_string(),
                html_body: format!(
                    r#"
                    <html>
                    <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
                        <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                            <h1 style="color: #2c3e50;">Welcome to ADX Core!</h1>
                            <p>Hello {},</p>
                            <p>Thank you for signing up for ADX Core. To complete your registration, please verify your email address by clicking the button below:</p>
                            <div style="text-align: center; margin: 30px 0;">
                                <a href="{}" style="background-color: #3498db; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">Verify Email</a>
                            </div>
                            <p>If you can't click the button, copy and paste this link into your browser:</p>
                            <p style="word-break: break-all; color: #7f8c8d;">{}</p>
                            <p>This link will expire in 24 hours for security reasons.</p>
                            <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">
                            <p style="font-size: 12px; color: #7f8c8d;">If you didn't create this account, you can safely ignore this email.</p>
                        </div>
                    </body>
                    </html>
                    "#,
                    display_name, verification_url, verification_url
                ),
                text_body: format!(
                    "Welcome to ADX Core!\n\nHello {},\n\nThank you for signing up for ADX Core. To complete your registration, please verify your email address by visiting this link:\n\n{}\n\nThis link will expire in 24 hours for security reasons.\n\nIf you didn't create this account, you can safely ignore this email.",
                    display_name, verification_url
                ),
            },
        }
    }

    /// Send email via external service
    async fn send_email(
        &self,
        to_email: &str,
        template: &EmailTemplate,
    ) -> Result<String, ActivityError> {
        let client = reqwest::Client::new();
        
        let email_payload = serde_json::json!({
            "from": {
                "email": self.default_from_email,
                "name": self.default_from_name
            },
            "to": [{"email": to_email}],
            "subject": template.subject,
            "html": template.html_body,
            "text": template.text_body,
            "tags": ["verification", "authentication"]
        });

        let response = client
            .post(&format!("{}/send", self.email_service_url))
            .header("Authorization", format!("Bearer {}", self.email_service_api_key))
            .header("Content-Type", "application/json")
            .json(&email_payload)
            .send()
            .await
            .map_err(|e| ActivityError::ExternalServiceError {
                service: "email_service".to_string(),
                message: format!("Failed to send email: {}", e),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ActivityError::ExternalServiceError {
                service: "email_service".to_string(),
                message: format!("Email service returned error: {}", error_text),
            });
        }

        let response_data: serde_json::Value = response.json().await.map_err(|e| {
            ActivityError::ExternalServiceError {
                service: "email_service".to_string(),
                message: format!("Failed to parse email service response: {}", e),
            }
        })?;

        // Extract message ID if available
        let message_id = response_data
            .get("message_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(message_id)
    }
}

#[async_trait]
impl AdxActivity<SendVerificationEmailRequest, SendVerificationEmailResponse> for SendVerificationEmailActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: SendVerificationEmailRequest,
    ) -> Result<SendVerificationEmailResponse, ActivityError> {
        // Validate input
        self.validate_input(&input)?;

        // Generate verification token
        let token = self.generate_verification_token(
            &context.tenant_context.tenant_id,
            &input.user_id,
            &input.email,
        ).await?;

        // Build verification URL
        let verification_url = format!(
            "{}?token={}&user_id={}",
            input.verification_url_base.trim_end_matches('/'),
            token.token_hash,
            input.user_id
        );

        // Get email template
        let template_name = input.template_name.as_deref().unwrap_or("welcome_verification");
        let language = input.language.as_deref().unwrap_or("en");
        let template = self.get_email_template(
            template_name,
            language,
            input.user_name.as_deref(),
            &verification_url,
        );

        // Send email
        let message_id = self.send_email(&input.email, &template).await?;

        Ok(SendVerificationEmailResponse {
            email_sent: true,
            token_id: token.id,
            expires_at: token.expires_at,
            email_provider: "external_service".to_string(),
            message_id: Some(message_id),
        })
    }

    fn activity_type(&self) -> &'static str {
        "send_verification_email_activity"
    }

    fn validate_input(&self, input: &SendVerificationEmailRequest) -> Result<(), ActivityError> {
        if input.user_id.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "user_id".to_string(),
                message: "User ID is required".to_string(),
            });
        }

        if input.email.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
            });
        }

        // Basic email validation
        if !input.email.contains('@') || !input.email.contains('.') {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }

        if input.verification_url_base.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "verification_url_base".to_string(),
                message: "Verification URL base is required".to_string(),
            });
        }

        // Validate URL format
        if !input.verification_url_base.starts_with("http://") && 
           !input.verification_url_base.starts_with("https://") {
            return Err(ActivityError::ValidationError {
                field: "verification_url_base".to_string(),
                message: "Verification URL must start with http:// or https://".to_string(),
            });
        }

        Ok(())
    }

    fn default_options(&self) -> adx_shared::temporal::ActivityExecutionOptions {
        let mut options = adx_shared::temporal::ActivityExecutionOptions::default();
        options.retry_policy = Some(external_service_retry_policy());
        options.tags.push("email_verification".to_string());
        options.tags.push("external_service".to_string());
        options
    }
}

#[async_trait]
impl TenantAwareActivity<SendVerificationEmailRequest, SendVerificationEmailResponse> for SendVerificationEmailActivity {
    async fn validate_tenant_access(
        &self,
        tenant_context: &TenantContext,
        _user_context: &UserContext,
    ) -> Result<(), ActivityError> {
        // Check if tenant is active
        if !tenant_context.is_active {
            return Err(ActivityError::AuthorizationError {
                message: "Cannot send emails for inactive tenant".to_string(),
            });
        }

        // Check if tenant has email features enabled
        if !tenant_context.features.contains(&"email_verification".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Email verification feature not enabled for tenant".to_string(),
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
        if resource_type == "emails" {
            // Check email quota (this would be implemented based on tenant's email limits)
            // For now, we'll just check if the tenant has email features
            if !tenant_context.features.contains(&"email_verification".to_string()) {
                return Err(ActivityError::QuotaExceededError {
                    resource_type: "emails".to_string(),
                    current_usage: 0,
                    limit: 0,
                    requested: requested_amount,
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ExternalServiceActivity<SendVerificationEmailRequest, SendVerificationEmailResponse> for SendVerificationEmailActivity {
    fn get_service_endpoint(&self) -> &str {
        &self.email_service_url
    }

    async fn get_auth_headers(&self) -> Result<HashMap<String, String>, ActivityError> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.email_service_api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(headers)
    }

    async fn handle_rate_limit(&self, retry_after: std::time::Duration) -> Result<(), ActivityError> {
        tracing::warn!("Email service rate limited, waiting {:?}", retry_after);
        tokio::time::sleep(retry_after).await;
        Ok(())
    }

    fn validate_response(&self, response: &serde_json::Value) -> Result<(), ActivityError> {
        // Check if response indicates success
        if let Some(status) = response.get("status") {
            if status.as_str() == Some("error") {
                let error_message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(ActivityError::ExternalServiceError {
                    service: "email_service".to_string(),
                    message: format!("Email service error: {}", error_message),
                });
            }
        }

        Ok(())
    }
}

// Tests commented out for now due to compilation issues
// #[cfg(test)]
// mod tests {
//     // Test implementations would go here
// }