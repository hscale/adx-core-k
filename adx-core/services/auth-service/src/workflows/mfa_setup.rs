use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use adx_shared::temporal::{
    WorkflowContext, ActivityContext, AdxActivity, TenantAwareActivity,
    ActivityError, WorkflowError, utils as activity_utils,
};
use adx_shared::types::UserId;

/// MFA setup workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupRequest {
    pub user_id: UserId,
    pub mfa_method: MfaMethod,
    pub phone_number: Option<String>,
    pub backup_codes_requested: bool,
}

/// MFA method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    Totp,           // Time-based One-Time Password (Google Authenticator, Authy, etc.)
    Sms,            // SMS-based verification
    Email,          // Email-based verification
    WebAuthn,       // WebAuthn/FIDO2 (hardware keys, biometrics)
    BackupCodes,    // Backup recovery codes
}

/// MFA setup workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupResult {
    pub user_id: UserId,
    pub mfa_enabled: bool,
    pub method_configured: MfaMethod,
    pub secret_key: Option<String>,        // For TOTP
    pub qr_code_url: Option<String>,       // For TOTP
    pub backup_codes: Option<Vec<String>>, // Recovery codes
    pub phone_verified: bool,              // For SMS
    pub setup_completed_at: DateTime<Utc>,
}

/// Generate TOTP secret activity
pub struct GenerateTotpSecretActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTotpSecretInput {
    pub user_id: UserId,
    pub user_email: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTotpSecretOutput {
    pub secret_key: String,
    pub qr_code_url: String,
    pub manual_entry_key: String,
    pub generated_at: DateTime<Utc>,
}

impl AdxActivity<GenerateTotpSecretInput, GenerateTotpSecretOutput> for GenerateTotpSecretActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: GenerateTotpSecretInput,
    ) -> Result<GenerateTotpSecretOutput, ActivityError> {
        let generated_at = Utc::now();

        // Generate a cryptographically secure secret key
        let secret_key = generate_totp_secret();
        
        // Create QR code URL for easy setup
        let qr_code_url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            urlencoding::encode(&input.issuer),
            urlencoding::encode(&input.user_email),
            &secret_key,
            urlencoding::encode(&input.issuer)
        );

        // Format secret for manual entry (groups of 4 characters)
        let manual_entry_key = secret_key
            .chars()
            .collect::<Vec<char>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ");

        tracing::info!(
            user_id = %input.user_id,
            user_email = %input.user_email,
            "Generated TOTP secret"
        );

        Ok(GenerateTotpSecretOutput {
            secret_key,
            qr_code_url,
            manual_entry_key,
            generated_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "generate_totp_secret"
    }
}

impl TenantAwareActivity<GenerateTotpSecretInput, GenerateTotpSecretOutput> for GenerateTotpSecretActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Users can set up MFA for their own accounts
        Ok(())
    }
}

/// Verify TOTP code activity
pub struct VerifyTotpCodeActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyTotpCodeInput {
    pub user_id: UserId,
    pub secret_key: String,
    pub verification_code: String,
    pub allow_time_drift: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyTotpCodeOutput {
    pub code_valid: bool,
    pub time_drift_seconds: Option<i64>,
    pub verified_at: DateTime<Utc>,
}

impl AdxActivity<VerifyTotpCodeInput, VerifyTotpCodeOutput> for VerifyTotpCodeActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: VerifyTotpCodeInput,
    ) -> Result<VerifyTotpCodeOutput, ActivityError> {
        let verified_at = Utc::now();

        // Verify TOTP code
        let (code_valid, time_drift) = verify_totp_code(
            &input.secret_key,
            &input.verification_code,
            input.allow_time_drift,
        )?;

        tracing::info!(
            user_id = %input.user_id,
            code_valid = code_valid,
            time_drift_seconds = ?time_drift,
            "TOTP code verification completed"
        );

        Ok(VerifyTotpCodeOutput {
            code_valid,
            time_drift_seconds: time_drift,
            verified_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "verify_totp_code"
    }
}

/// Send SMS verification activity
pub struct SendSmsVerificationActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSmsVerificationInput {
    pub user_id: UserId,
    pub phone_number: String,
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendSmsVerificationOutput {
    pub sms_sent: bool,
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
}

impl AdxActivity<SendSmsVerificationInput, SendSmsVerificationOutput> for SendSmsVerificationActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: SendSmsVerificationInput,
    ) -> Result<SendSmsVerificationOutput, ActivityError> {
        let message_id = Uuid::new_v4().to_string();
        let sent_at = Utc::now();

        // TODO: Send SMS using SMS provider (Twilio, AWS SNS, etc.)
        tracing::info!(
            user_id = %input.user_id,
            phone_number = %mask_phone_number(&input.phone_number),
            message_id = %message_id,
            "Sending SMS verification code"
        );

        // Simulate SMS sending
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(SendSmsVerificationOutput {
            sms_sent: true,
            message_id,
            sent_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "send_sms_verification"
    }
}

/// Verify phone number activity
pub struct VerifyPhoneNumberActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPhoneNumberInput {
    pub user_id: UserId,
    pub phone_number: String,
    pub verification_code: String,
    pub expected_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPhoneNumberOutput {
    pub phone_verified: bool,
    pub verified_at: DateTime<Utc>,
}

impl AdxActivity<VerifyPhoneNumberInput, VerifyPhoneNumberOutput> for VerifyPhoneNumberActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: VerifyPhoneNumberInput,
    ) -> Result<VerifyPhoneNumberOutput, ActivityError> {
        let verified_at = Utc::now();

        // Verify the SMS code
        let phone_verified = input.verification_code == input.expected_code;

        tracing::info!(
            user_id = %input.user_id,
            phone_number = %mask_phone_number(&input.phone_number),
            phone_verified = phone_verified,
            "Phone number verification completed"
        );

        Ok(VerifyPhoneNumberOutput {
            phone_verified,
            verified_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "verify_phone_number"
    }
}

/// Generate backup codes activity
pub struct GenerateBackupCodesActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBackupCodesInput {
    pub user_id: UserId,
    pub code_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBackupCodesOutput {
    pub backup_codes: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

impl AdxActivity<GenerateBackupCodesInput, GenerateBackupCodesOutput> for GenerateBackupCodesActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: GenerateBackupCodesInput,
    ) -> Result<GenerateBackupCodesOutput, ActivityError> {
        let generated_at = Utc::now();

        // Generate cryptographically secure backup codes
        let backup_codes = (0..input.code_count)
            .map(|_| generate_backup_code())
            .collect();

        tracing::info!(
            user_id = %input.user_id,
            code_count = input.code_count,
            "Generated backup codes"
        );

        Ok(GenerateBackupCodesOutput {
            backup_codes,
            generated_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "generate_backup_codes"
    }
}

/// Store MFA configuration activity
pub struct StoreMfaConfigurationActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMfaConfigurationInput {
    pub user_id: UserId,
    pub mfa_method: MfaMethod,
    pub secret_key: Option<String>,
    pub phone_number: Option<String>,
    pub backup_codes: Option<Vec<String>>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMfaConfigurationOutput {
    pub configuration_stored: bool,
    pub stored_at: DateTime<Utc>,
}

impl AdxActivity<StoreMfaConfigurationInput, StoreMfaConfigurationOutput> for StoreMfaConfigurationActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: StoreMfaConfigurationInput,
    ) -> Result<StoreMfaConfigurationOutput, ActivityError> {
        let stored_at = Utc::now();

        // TODO: Store MFA configuration in database
        // Secret keys and backup codes should be encrypted at rest
        tracing::info!(
            user_id = %input.user_id,
            mfa_method = ?input.mfa_method,
            enabled = input.enabled,
            "Storing MFA configuration"
        );

        Ok(StoreMfaConfigurationOutput {
            configuration_stored: true,
            stored_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "store_mfa_configuration"
    }
}

impl TenantAwareActivity<StoreMfaConfigurationInput, StoreMfaConfigurationOutput> for StoreMfaConfigurationActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Users can configure MFA for their own accounts
        Ok(())
    }
}

/// Send MFA setup notification activity
pub struct SendMfaSetupNotificationActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMfaSetupNotificationInput {
    pub user_id: UserId,
    pub user_email: String,
    pub mfa_method: MfaMethod,
    pub setup_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMfaSetupNotificationOutput {
    pub notification_sent: bool,
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
}

impl AdxActivity<SendMfaSetupNotificationInput, SendMfaSetupNotificationOutput> for SendMfaSetupNotificationActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: SendMfaSetupNotificationInput,
    ) -> Result<SendMfaSetupNotificationOutput, ActivityError> {
        let message_id = Uuid::new_v4().to_string();
        let sent_at = Utc::now();

        // TODO: Send email notification about MFA setup
        tracing::info!(
            user_id = %input.user_id,
            user_email = %input.user_email,
            mfa_method = ?input.mfa_method,
            setup_completed = input.setup_completed,
            message_id = %message_id,
            "Sending MFA setup notification"
        );

        // Simulate email sending
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(SendMfaSetupNotificationOutput {
            notification_sent: true,
            message_id,
            sent_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "send_mfa_setup_notification"
    }
}

/// MFA setup workflow implementation
pub async fn mfa_setup_workflow(
    _context: WorkflowContext,
    request: MfaSetupRequest,
) -> Result<MfaSetupResult, WorkflowError> {
    let setup_completed_at = Utc::now();

    match request.mfa_method {
        MfaMethod::Totp => {
            // TOTP setup workflow
            setup_totp_workflow(request, setup_completed_at).await
        }
        MfaMethod::Sms => {
            // SMS setup workflow
            setup_sms_workflow(request, setup_completed_at).await
        }
        MfaMethod::Email => {
            // Email setup workflow
            setup_email_workflow(request, setup_completed_at).await
        }
        MfaMethod::WebAuthn => {
            // WebAuthn setup workflow
            setup_webauthn_workflow(request, setup_completed_at).await
        }
        MfaMethod::BackupCodes => {
            // Backup codes generation workflow
            setup_backup_codes_workflow(request, setup_completed_at).await
        }
    }
}

async fn setup_totp_workflow(
    request: MfaSetupRequest,
    setup_completed_at: DateTime<Utc>,
) -> Result<MfaSetupResult, WorkflowError> {
    // Step 1: Generate TOTP secret
    let generate_secret_activity = GenerateTotpSecretActivity;
    let generate_secret_input = GenerateTotpSecretInput {
        user_id: request.user_id.clone(),
        user_email: "user@example.com".to_string(), // TODO: Get from user context
        issuer: "ADX Core".to_string(),
    };

    let secret_result = generate_secret_activity.execute(
        create_activity_context("generate_totp_secret", "mfa-setup-workflow"),
        generate_secret_input,
    ).await?;

    // Step 2: Wait for user to verify TOTP code (this would be handled by the frontend)
    // For now, we'll simulate verification
    let verify_code_activity = VerifyTotpCodeActivity;
    let verify_code_input = VerifyTotpCodeInput {
        user_id: request.user_id.clone(),
        secret_key: secret_result.secret_key.clone(),
        verification_code: "123456".to_string(), // This would come from user input
        allow_time_drift: true,
    };

    let verification_result = verify_code_activity.execute(
        create_activity_context("verify_totp_code", "mfa-setup-workflow"),
        verify_code_input,
    ).await?;

    if !verification_result.code_valid {
        return Err(WorkflowError::ValidationFailed {
            errors: vec!["Invalid verification code".to_string()],
        });
    }

    // Step 3: Generate backup codes if requested
    let backup_codes = if request.backup_codes_requested {
        let generate_backup_activity = GenerateBackupCodesActivity;
        let generate_backup_input = GenerateBackupCodesInput {
            user_id: request.user_id.clone(),
            code_count: 10,
        };

        let backup_result = generate_backup_activity.execute(
            create_activity_context("generate_backup_codes", "mfa-setup-workflow"),
            generate_backup_input,
        ).await?;

        Some(backup_result.backup_codes)
    } else {
        None
    };

    // Step 4: Store MFA configuration
    let store_config_activity = StoreMfaConfigurationActivity;
    let store_config_input = StoreMfaConfigurationInput {
        user_id: request.user_id.clone(),
        mfa_method: MfaMethod::Totp,
        secret_key: Some(secret_result.secret_key.clone()),
        phone_number: None,
        backup_codes: backup_codes.clone(),
        enabled: true,
    };

    let _store_result = store_config_activity.execute(
        create_activity_context("store_mfa_configuration", "mfa-setup-workflow"),
        store_config_input,
    ).await?;

    // Step 5: Send setup notification
    let send_notification_activity = SendMfaSetupNotificationActivity;
    let send_notification_input = SendMfaSetupNotificationInput {
        user_id: request.user_id.clone(),
        user_email: "user@example.com".to_string(), // TODO: Get from user context
        mfa_method: MfaMethod::Totp,
        setup_completed: true,
    };

    let _notification_result = send_notification_activity.execute(
        create_activity_context("send_mfa_setup_notification", "mfa-setup-workflow"),
        send_notification_input,
    ).await?;

    Ok(MfaSetupResult {
        user_id: request.user_id,
        mfa_enabled: true,
        method_configured: MfaMethod::Totp,
        secret_key: Some(secret_result.secret_key),
        qr_code_url: Some(secret_result.qr_code_url),
        backup_codes,
        phone_verified: false,
        setup_completed_at,
    })
}

async fn setup_sms_workflow(
    request: MfaSetupRequest,
    setup_completed_at: DateTime<Utc>,
) -> Result<MfaSetupResult, WorkflowError> {
    let phone_number = request.phone_number.ok_or_else(|| WorkflowError::ValidationFailed {
        errors: vec!["phone_number field is required".to_string()],
    })?;

    // Step 1: Generate verification code
    let verification_code = generate_sms_code();

    // Step 2: Send SMS verification
    let send_sms_activity = SendSmsVerificationActivity;
    let send_sms_input = SendSmsVerificationInput {
        user_id: request.user_id.clone(),
        phone_number: phone_number.clone(),
        verification_code: verification_code.clone(),
    };

    let _sms_result = send_sms_activity.execute(
        create_activity_context("send_sms_verification", "mfa-setup-workflow"),
        send_sms_input,
    ).await?;

    // Step 3: Verify phone number (this would be handled by user input)
    let verify_phone_activity = VerifyPhoneNumberActivity;
    let verify_phone_input = VerifyPhoneNumberInput {
        user_id: request.user_id.clone(),
        phone_number: phone_number.clone(),
        verification_code: "123456".to_string(), // This would come from user input
        expected_code: verification_code,
    };

    let phone_verification = verify_phone_activity.execute(
        create_activity_context("verify_phone_number", "mfa-setup-workflow"),
        verify_phone_input,
    ).await?;

    if !phone_verification.phone_verified {
        return Err(WorkflowError::ValidationFailed {
            errors: vec!["Invalid verification code".to_string()],
        });
    }

    // Step 4: Generate backup codes if requested
    let backup_codes = if request.backup_codes_requested {
        let generate_backup_activity = GenerateBackupCodesActivity;
        let generate_backup_input = GenerateBackupCodesInput {
            user_id: request.user_id.clone(),
            code_count: 10,
        };

        let backup_result = generate_backup_activity.execute(
            create_activity_context("generate_backup_codes", "mfa-setup-workflow"),
            generate_backup_input,
        ).await?;

        Some(backup_result.backup_codes)
    } else {
        None
    };

    // Step 5: Store MFA configuration
    let store_config_activity = StoreMfaConfigurationActivity;
    let store_config_input = StoreMfaConfigurationInput {
        user_id: request.user_id.clone(),
        mfa_method: MfaMethod::Sms,
        secret_key: None,
        phone_number: Some(phone_number),
        backup_codes: backup_codes.clone(),
        enabled: true,
    };

    let _store_result = store_config_activity.execute(
        create_activity_context("store_mfa_configuration", "mfa-setup-workflow"),
        store_config_input,
    ).await?;

    // Step 6: Send setup notification
    let send_notification_activity = SendMfaSetupNotificationActivity;
    let send_notification_input = SendMfaSetupNotificationInput {
        user_id: request.user_id.clone(),
        user_email: "user@example.com".to_string(), // TODO: Get from user context
        mfa_method: MfaMethod::Sms,
        setup_completed: true,
    };

    let _notification_result = send_notification_activity.execute(
        create_activity_context("send_mfa_setup_notification", "mfa-setup-workflow"),
        send_notification_input,
    ).await?;

    Ok(MfaSetupResult {
        user_id: request.user_id,
        mfa_enabled: true,
        method_configured: MfaMethod::Sms,
        secret_key: None,
        qr_code_url: None,
        backup_codes,
        phone_verified: true,
        setup_completed_at,
    })
}

async fn setup_email_workflow(
    request: MfaSetupRequest,
    setup_completed_at: DateTime<Utc>,
) -> Result<MfaSetupResult, WorkflowError> {
    // Email MFA setup is simpler - just enable it and generate backup codes
    let backup_codes = if request.backup_codes_requested {
        let generate_backup_activity = GenerateBackupCodesActivity;
        let generate_backup_input = GenerateBackupCodesInput {
            user_id: request.user_id.clone(),
            code_count: 10,
        };

        let backup_result = generate_backup_activity.execute(
            create_activity_context("generate_backup_codes", "mfa-setup-workflow"),
            generate_backup_input,
        ).await?;

        Some(backup_result.backup_codes)
    } else {
        None
    };

    // Store MFA configuration
    let store_config_activity = StoreMfaConfigurationActivity;
    let store_config_input = StoreMfaConfigurationInput {
        user_id: request.user_id.clone(),
        mfa_method: MfaMethod::Email,
        secret_key: None,
        phone_number: None,
        backup_codes: backup_codes.clone(),
        enabled: true,
    };

    let _store_result = store_config_activity.execute(
        create_activity_context("store_mfa_configuration", "mfa-setup-workflow"),
        store_config_input,
    ).await?;

    // Send setup notification
    let send_notification_activity = SendMfaSetupNotificationActivity;
    let send_notification_input = SendMfaSetupNotificationInput {
        user_id: request.user_id.clone(),
        user_email: "user@example.com".to_string(), // TODO: Get from user context
        mfa_method: MfaMethod::Email,
        setup_completed: true,
    };

    let _notification_result = send_notification_activity.execute(
        create_activity_context("send_mfa_setup_notification", "mfa-setup-workflow"),
        send_notification_input,
    ).await?;

    Ok(MfaSetupResult {
        user_id: request.user_id,
        mfa_enabled: true,
        method_configured: MfaMethod::Email,
        secret_key: None,
        qr_code_url: None,
        backup_codes,
        phone_verified: false,
        setup_completed_at,
    })
}

async fn setup_webauthn_workflow(
    request: MfaSetupRequest,
    setup_completed_at: DateTime<Utc>,
) -> Result<MfaSetupResult, WorkflowError> {
    // WebAuthn setup would involve credential creation and verification
    // This is a simplified implementation
    
    let backup_codes = if request.backup_codes_requested {
        let generate_backup_activity = GenerateBackupCodesActivity;
        let generate_backup_input = GenerateBackupCodesInput {
            user_id: request.user_id.clone(),
            code_count: 10,
        };

        let backup_result = generate_backup_activity.execute(
            create_activity_context("generate_backup_codes", "mfa-setup-workflow"),
            generate_backup_input,
        ).await?;

        Some(backup_result.backup_codes)
    } else {
        None
    };

    // Store MFA configuration
    let store_config_activity = StoreMfaConfigurationActivity;
    let store_config_input = StoreMfaConfigurationInput {
        user_id: request.user_id.clone(),
        mfa_method: MfaMethod::WebAuthn,
        secret_key: None,
        phone_number: None,
        backup_codes: backup_codes.clone(),
        enabled: true,
    };

    let _store_result = store_config_activity.execute(
        create_activity_context("store_mfa_configuration", "mfa-setup-workflow"),
        store_config_input,
    ).await?;

    Ok(MfaSetupResult {
        user_id: request.user_id,
        mfa_enabled: true,
        method_configured: MfaMethod::WebAuthn,
        secret_key: None,
        qr_code_url: None,
        backup_codes,
        phone_verified: false,
        setup_completed_at,
    })
}

async fn setup_backup_codes_workflow(
    request: MfaSetupRequest,
    setup_completed_at: DateTime<Utc>,
) -> Result<MfaSetupResult, WorkflowError> {
    // Generate backup codes
    let generate_backup_activity = GenerateBackupCodesActivity;
    let generate_backup_input = GenerateBackupCodesInput {
        user_id: request.user_id.clone(),
        code_count: 10,
    };

    let backup_result = generate_backup_activity.execute(
        create_activity_context("generate_backup_codes", "mfa-setup-workflow"),
        generate_backup_input,
    ).await?;

    // Store MFA configuration
    let store_config_activity = StoreMfaConfigurationActivity;
    let store_config_input = StoreMfaConfigurationInput {
        user_id: request.user_id.clone(),
        mfa_method: MfaMethod::BackupCodes,
        secret_key: None,
        phone_number: None,
        backup_codes: Some(backup_result.backup_codes.clone()),
        enabled: true,
    };

    let _store_result = store_config_activity.execute(
        create_activity_context("store_mfa_configuration", "mfa-setup-workflow"),
        store_config_input,
    ).await?;

    Ok(MfaSetupResult {
        user_id: request.user_id,
        mfa_enabled: true,
        method_configured: MfaMethod::BackupCodes,
        secret_key: None,
        qr_code_url: None,
        backup_codes: Some(backup_result.backup_codes),
        phone_verified: false,
        setup_completed_at,
    })
}

// Helper functions
fn generate_totp_secret() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn verify_totp_code(
    _secret_key: &str,
    _verification_code: &str,
    _allow_time_drift: bool,
) -> Result<(bool, Option<i64>), ActivityError> {
    // TODO: Implement actual TOTP verification using a library like `totp-lite`
    // For now, simulate verification
    Ok((true, Some(0)))
}

fn generate_sms_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..999999))
}

fn generate_backup_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn mask_phone_number(phone: &str) -> String {
    if phone.len() > 4 {
        let visible_part = &phone[phone.len() - 4..];
        format!("****{}", visible_part)
    } else {
        "****".to_string()
    }
}

fn create_activity_context(activity_type: &str, workflow_id: &str) -> ActivityContext {
    ActivityContext {
        activity_id: activity_utils::generate_activity_id(activity_type),
        activity_type: activity_type.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_run_id: Uuid::new_v4().to_string(),
        attempt: 1,
        user_context: adx_shared::temporal::UserContext {
            user_id: "system".to_string(),
            email: "system@adxcore.com".to_string(),
            roles: vec!["system".to_string()],
            permissions: vec!["mfa:setup".to_string()],
            session_id: None,
            device_info: None,
        },
        tenant_context: adx_shared::temporal::TenantContext {
            tenant_id: "default".to_string(),
            tenant_name: "Default".to_string(),
            subscription_tier: adx_shared::temporal::SubscriptionTier::Professional,
            features: vec![],
            quotas: adx_shared::temporal::TenantQuotas {
                max_users: 100,
                max_storage_gb: 1000,
                max_api_calls_per_hour: 10000,
                max_concurrent_workflows: 50,
                max_file_upload_size_mb: 100,
            },
            settings: adx_shared::temporal::TenantSettings {
                default_language: "en".to_string(),
                timezone: "UTC".to_string(),
                date_format: "YYYY-MM-DD".to_string(),
                currency: "USD".to_string(),
                branding: None,
            },
            isolation_level: adx_shared::temporal::TenantIsolationLevel::Schema,
        },
        metadata: adx_shared::temporal::ActivityMetadata {
            start_time: Utc::now(),
            timeout: std::time::Duration::from_secs(300),
            heartbeat_timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: Some(activity_utils::external_service_retry_policy()),
            tags: vec!["mfa_setup".to_string()],
            custom: std::collections::HashMap::new(),
        },
        heartbeat_details: None,
    }
}