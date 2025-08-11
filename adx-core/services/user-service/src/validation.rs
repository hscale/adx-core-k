use regex::Regex;
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
    #[error("Password too weak: {0}")]
    WeakPassword(String),
    #[error("Invalid phone number format: {0}")]
    InvalidPhoneNumber(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Field too long: {field} exceeds {max_length} characters")]
    FieldTooLong { field: String, max_length: usize },
    #[error("Field required: {0}")]
    FieldRequired(String),
    #[error("Invalid value: {field} has invalid value '{value}'")]
    InvalidValue { field: String, value: String },
    #[error("Invalid date: {0}")]
    InvalidDate(String),
    #[error("Invalid timezone: {0}")]
    InvalidTimezone(String),
    #[error("Invalid language code: {0}")]
    InvalidLanguage(String),
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    #[error("Invalid permission: {0}")]
    InvalidPermission(String),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub struct UserValidator {
    email_regex: Regex,
    phone_regex: Regex,
    url_regex: Regex,
    valid_timezones: HashSet<String>,
    valid_languages: HashSet<String>,
    valid_roles: HashSet<String>,
    valid_permissions: HashSet<String>,
}

impl UserValidator {
    pub fn new() -> Self {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        
        // Common timezones (in production, this would be loaded from a comprehensive list)
        let valid_timezones = [
            "UTC", "America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles",
            "Europe/London", "Europe/Paris", "Europe/Berlin", "Asia/Tokyo", "Asia/Shanghai",
            "Australia/Sydney", "Pacific/Auckland"
        ].iter().map(|s| s.to_string()).collect();
        
        // ISO 639-1 language codes (subset)
        let valid_languages = [
            "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh", "ar", "hi"
        ].iter().map(|s| s.to_string()).collect();
        
        // Valid user roles
        let valid_roles = [
            "user", "admin", "manager", "developer", "analyst", "designer", "support"
        ].iter().map(|s| s.to_string()).collect();
        
        // Valid permissions (subset)
        let valid_permissions = [
            "user:read", "user:write", "user:admin", "file:read", "file:write", "file:admin",
            "tenant:read", "tenant:write", "tenant:admin", "workflow:execute", "workflow:admin"
        ].iter().map(|s| s.to_string()).collect();
        
        Self {
            email_regex,
            phone_regex,
            url_regex,
            valid_timezones,
            valid_languages,
            valid_roles,
            valid_permissions,
        }
    }
    
    pub fn validate_email(&self, email: &str) -> ValidationResult<()> {
        if email.is_empty() {
            return Err(ValidationError::FieldRequired("email".to_string()));
        }
        
        if email.len() > 255 {
            return Err(ValidationError::FieldTooLong {
                field: "email".to_string(),
                max_length: 255,
            });
        }
        
        if !self.email_regex.is_match(email) {
            return Err(ValidationError::InvalidEmail(email.to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_password(&self, password: &str) -> ValidationResult<()> {
        if password.is_empty() {
            return Err(ValidationError::FieldRequired("password".to_string()));
        }
        
        if password.len() < 8 {
            return Err(ValidationError::WeakPassword("Password must be at least 8 characters long".to_string()));
        }
        
        if password.len() > 128 {
            return Err(ValidationError::FieldTooLong {
                field: "password".to_string(),
                max_length: 128,
            });
        }
        
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        if !has_uppercase {
            return Err(ValidationError::WeakPassword("Password must contain at least one uppercase letter".to_string()));
        }
        
        if !has_lowercase {
            return Err(ValidationError::WeakPassword("Password must contain at least one lowercase letter".to_string()));
        }
        
        if !has_digit {
            return Err(ValidationError::WeakPassword("Password must contain at least one digit".to_string()));
        }
        
        if !has_special {
            return Err(ValidationError::WeakPassword("Password must contain at least one special character".to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_phone_number(&self, phone: &str) -> ValidationResult<()> {
        if phone.is_empty() {
            return Ok(()); // Phone number is optional
        }
        
        if phone.len() > 20 {
            return Err(ValidationError::FieldTooLong {
                field: "phone_number".to_string(),
                max_length: 20,
            });
        }
        
        if !self.phone_regex.is_match(phone) {
            return Err(ValidationError::InvalidPhoneNumber(phone.to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_url(&self, url: &str) -> ValidationResult<()> {
        if url.is_empty() {
            return Ok(()); // URL is optional
        }
        
        if url.len() > 2048 {
            return Err(ValidationError::FieldTooLong {
                field: "url".to_string(),
                max_length: 2048,
            });
        }
        
        if !self.url_regex.is_match(url) {
            return Err(ValidationError::InvalidUrl(url.to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_string_length(&self, field_name: &str, value: &str, max_length: usize) -> ValidationResult<()> {
        if value.len() > max_length {
            return Err(ValidationError::FieldTooLong {
                field: field_name.to_string(),
                max_length,
            });
        }
        Ok(())
    }
    
    pub fn validate_timezone(&self, timezone: &str) -> ValidationResult<()> {
        if timezone.is_empty() {
            return Ok(()); // Will use default
        }
        
        if !self.valid_timezones.contains(timezone) {
            return Err(ValidationError::InvalidTimezone(timezone.to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_language(&self, language: &str) -> ValidationResult<()> {
        if language.is_empty() {
            return Ok(()); // Will use default
        }
        
        if !self.valid_languages.contains(language) {
            return Err(ValidationError::InvalidLanguage(language.to_string()));
        }
        
        Ok(())
    }
    
    pub fn validate_roles(&self, roles: &[String]) -> ValidationResult<()> {
        for role in roles {
            if !self.valid_roles.contains(role) {
                return Err(ValidationError::InvalidRole(role.clone()));
            }
        }
        Ok(())
    }
    
    pub fn validate_permissions(&self, permissions: &[String]) -> ValidationResult<()> {
        for permission in permissions {
            if !self.valid_permissions.contains(permission) {
                return Err(ValidationError::InvalidPermission(permission.clone()));
            }
        }
        Ok(())
    }
    
    pub fn sanitize_text(&self, text: &str) -> String {
        // Remove potentially dangerous characters and normalize whitespace
        text.chars()
            .filter(|c| !c.is_control() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
    }
    
    pub fn sanitize_html(&self, html: &str) -> String {
        // Basic HTML sanitization - in production, use a proper HTML sanitizer
        html.replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('&', "&amp;")
    }
    
    pub fn validate_uuid(&self, uuid_str: &str) -> ValidationResult<Uuid> {
        Uuid::parse_str(uuid_str)
            .map_err(|_| ValidationError::InvalidValue {
                field: "uuid".to_string(),
                value: uuid_str.to_string(),
            })
    }
}

impl Default for UserValidator {
    fn default() -> Self {
        Self::new()
    }
}

// Validation helper functions
pub fn validate_create_user_request(
    validator: &UserValidator,
    request: &crate::models::CreateUserRequest,
) -> ValidationResult<()> {
    validator.validate_email(&request.email)?;
    validator.validate_password(&request.password)?;
    
    if let Some(first_name) = &request.first_name {
        validator.validate_string_length("first_name", first_name, 100)?;
    }
    
    if let Some(last_name) = &request.last_name {
        validator.validate_string_length("last_name", last_name, 100)?;
    }
    
    if let Some(roles) = &request.roles {
        validator.validate_roles(roles)?;
    }
    
    if let Some(profile) = &request.profile {
        validate_create_user_profile_request(validator, profile)?;
    }
    
    Ok(())
}

pub fn validate_create_user_profile_request(
    validator: &UserValidator,
    request: &crate::models::CreateUserProfileRequest,
) -> ValidationResult<()> {
    if let Some(display_name) = &request.display_name {
        validator.validate_string_length("display_name", display_name, 255)?;
    }
    
    if let Some(bio) = &request.bio {
        validator.validate_string_length("bio", bio, 1000)?;
    }
    
    if let Some(location) = &request.location {
        validator.validate_string_length("location", location, 255)?;
    }
    
    if let Some(website_url) = &request.website_url {
        validator.validate_url(website_url)?;
    }
    
    if let Some(timezone) = &request.timezone {
        validator.validate_timezone(timezone)?;
    }
    
    if let Some(language) = &request.language {
        validator.validate_language(language)?;
    }
    
    if let Some(phone_number) = &request.phone_number {
        validator.validate_phone_number(phone_number)?;
    }
    
    if let Some(job_title) = &request.job_title {
        validator.validate_string_length("job_title", job_title, 255)?;
    }
    
    if let Some(department) = &request.department {
        validator.validate_string_length("department", department, 255)?;
    }
    
    Ok(())
}

pub fn validate_update_user_request(
    validator: &UserValidator,
    request: &crate::models::UpdateUserRequest,
) -> ValidationResult<()> {
    if let Some(first_name) = &request.first_name {
        validator.validate_string_length("first_name", first_name, 100)?;
    }
    
    if let Some(last_name) = &request.last_name {
        validator.validate_string_length("last_name", last_name, 100)?;
    }
    
    if let Some(roles) = &request.roles {
        validator.validate_roles(roles)?;
    }
    
    if let Some(permissions) = &request.permissions {
        validator.validate_permissions(permissions)?;
    }
    
    Ok(())
}

pub fn validate_update_user_profile_request(
    validator: &UserValidator,
    request: &crate::models::UpdateUserProfileRequest,
) -> ValidationResult<()> {
    if let Some(display_name) = &request.display_name {
        validator.validate_string_length("display_name", display_name, 255)?;
    }
    
    if let Some(bio) = &request.bio {
        validator.validate_string_length("bio", bio, 1000)?;
    }
    
    if let Some(avatar_url) = &request.avatar_url {
        validator.validate_url(avatar_url)?;
    }
    
    if let Some(cover_image_url) = &request.cover_image_url {
        validator.validate_url(cover_image_url)?;
    }
    
    if let Some(location) = &request.location {
        validator.validate_string_length("location", location, 255)?;
    }
    
    if let Some(website_url) = &request.website_url {
        validator.validate_url(website_url)?;
    }
    
    if let Some(timezone) = &request.timezone {
        validator.validate_timezone(timezone)?;
    }
    
    if let Some(language) = &request.language {
        validator.validate_language(language)?;
    }
    
    if let Some(phone_number) = &request.phone_number {
        validator.validate_phone_number(phone_number)?;
    }
    
    if let Some(job_title) = &request.job_title {
        validator.validate_string_length("job_title", job_title, 255)?;
    }
    
    if let Some(department) = &request.department {
        validator.validate_string_length("department", department, 255)?;
    }
    
    Ok(())
}