use crate::config::WhiteLabelConfig;
use crate::error::{WhiteLabelError, WhiteLabelResult};
use crate::types::{EmailTemplate, EmailTemplateRequest};
use crate::workflows::BrandingContext;
use handlebars::Handlebars;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EmailService {
    config: Arc<WhiteLabelConfig>,
    handlebars: Handlebars<'static>,
    smtp_transport: SmtpTransport,
}

impl EmailService {
    pub fn new(config: Arc<WhiteLabelConfig>) -> WhiteLabelResult<Self> {
        let mut handlebars = Handlebars::new();
        
        // Register built-in helpers
        handlebars.register_helper("upper", Box::new(uppercase_helper));
        handlebars.register_helper("lower", Box::new(lowercase_helper));
        handlebars.register_helper("format_date", Box::new(format_date_helper));

        // Set up SMTP transport
        let smtp_transport = SmtpTransport::relay(&config.email_config.smtp_host)
            .map_err(|e| WhiteLabelError::Configuration(format!("SMTP configuration error: {}", e)))?
            .port(config.email_config.smtp_port)
            .credentials(Credentials::new(
                config.email_config.smtp_username.clone(),
                config.email_config.smtp_password.clone(),
            ))
            .build();

        Ok(Self {
            config,
            handlebars,
            smtp_transport,
        })
    }

    pub async fn process_template(
        &self,
        template_request: &EmailTemplateRequest,
        branding_context: &BrandingContext,
    ) -> WhiteLabelResult<EmailTemplate> {
        // Create template context
        let context = json!({
            "brand_name": branding_context.brand_name,
            "primary_color": branding_context.colors.primary_color,
            "secondary_color": branding_context.colors.secondary_color,
            "accent_color": branding_context.colors.accent_color,
            "background_color": branding_context.colors.background_color,
            "text_color": branding_context.colors.text_color,
            "logo_url": branding_context.asset_urls.get("logo"),
            "favicon_url": branding_context.asset_urls.get("favicon"),
        });

        // Process subject template
        let processed_subject = self
            .handlebars
            .render_template(&template_request.subject, &context)
            .map_err(|e| {
                WhiteLabelError::TemplateProcessing(format!("Subject template error: {}", e))
            })?;

        // Process HTML body template
        let processed_html_body = self
            .handlebars
            .render_template(&template_request.html_body, &context)
            .map_err(|e| {
                WhiteLabelError::TemplateProcessing(format!("HTML body template error: {}", e))
            })?;

        // Process text body template
        let processed_text_body = self
            .handlebars
            .render_template(&template_request.text_body, &context)
            .map_err(|e| {
                WhiteLabelError::TemplateProcessing(format!("Text body template error: {}", e))
            })?;

        // Extract variables from templates
        let variables = self.extract_template_variables(&template_request.html_body);

        Ok(EmailTemplate {
            subject: processed_subject,
            html_body: processed_html_body,
            text_body: processed_text_body,
            variables,
        })
    }

    pub async fn send_email(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        template: &EmailTemplate,
        variables: &HashMap<String, String>,
    ) -> WhiteLabelResult<()> {
        // Render final template with variables
        let final_subject = self.render_with_variables(&template.subject, variables)?;
        let final_html_body = self.render_with_variables(&template.html_body, variables)?;
        let final_text_body = self.render_with_variables(&template.text_body, variables)?;

        // Create email message
        let to_mailbox = if let Some(name) = to_name {
            format!("{} <{}>", name, to_email)
        } else {
            to_email.to_string()
        };

        let to_mailbox: Mailbox = to_mailbox.parse()
            .map_err(|e| WhiteLabelError::TemplateProcessing(format!("Invalid email address: {}", e)))?;

        let from_mailbox = format!("{} <{}>", 
            self.config.email_config.from_name, 
            self.config.email_config.from_email
        );
        let from_mailbox: Mailbox = from_mailbox.parse()
            .map_err(|e| WhiteLabelError::Configuration(format!("Invalid from email address: {}", e)))?;

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(final_subject)
            .header(ContentType::TEXT_HTML)
            .body(final_html_body)
            .map_err(|e| WhiteLabelError::TemplateProcessing(format!("Email building error: {}", e)))?;

        // Send email
        self.smtp_transport
            .send(&email)
            .map_err(|e| WhiteLabelError::ExternalService(format!("Email sending error: {}", e)))?;

        Ok(())
    }

    pub async fn send_reseller_welcome_email(
        &self,
        reseller_name: &str,
        email: &str,
    ) -> WhiteLabelResult<()> {
        let variables = HashMap::from([
            ("reseller_name".to_string(), reseller_name.to_string()),
            ("login_url".to_string(), "https://app.adxcore.com/login".to_string()),
            ("support_email".to_string(), "support@adxcore.com".to_string()),
        ]);

        let template = EmailTemplate {
            subject: "Welcome to ADX Core Reseller Program".to_string(),
            html_body: self.get_reseller_welcome_html_template(),
            text_body: self.get_reseller_welcome_text_template(),
            variables: vec!["reseller_name".to_string(), "login_url".to_string()],
        };

        self.send_email(email, Some(reseller_name), &template, &variables).await
    }

    pub async fn send_domain_verification_email(
        &self,
        tenant_name: &str,
        email: &str,
        domain: &str,
        verification_token: &str,
    ) -> WhiteLabelResult<()> {
        let variables = HashMap::from([
            ("tenant_name".to_string(), tenant_name.to_string()),
            ("domain".to_string(), domain.to_string()),
            ("verification_token".to_string(), verification_token.to_string()),
            ("verification_url".to_string(), format!("https://app.adxcore.com/verify-domain?token={}", verification_token)),
        ]);

        let template = EmailTemplate {
            subject: "Domain Verification Required".to_string(),
            html_body: self.get_domain_verification_html_template(),
            text_body: self.get_domain_verification_text_template(),
            variables: vec!["tenant_name".to_string(), "domain".to_string(), "verification_token".to_string()],
        };

        self.send_email(email, Some(tenant_name), &template, &variables).await
    }

    fn render_with_variables(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> WhiteLabelResult<String> {
        self.handlebars
            .render_template(template, variables)
            .map_err(|e| WhiteLabelError::TemplateProcessing(format!("Template rendering error: {}", e)))
    }

    fn extract_template_variables(&self, template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                
                let mut var_name = String::new();
                let mut brace_count = 0;
                
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        if brace_count == 0 && chars.peek() == Some(&'}') {
                            chars.next(); // consume second '}'
                            break;
                        }
                        brace_count -= 1;
                    } else if ch == '{' {
                        brace_count += 1;
                    }
                    
                    if brace_count >= 0 {
                        var_name.push(ch);
                    }
                }
                
                let var_name = var_name.trim();
                if !var_name.is_empty() && !variables.contains(&var_name.to_string()) {
                    variables.push(var_name.to_string());
                }
            }
        }
        
        variables
    }

    fn get_reseller_welcome_html_template(&self) -> String {
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <title>Welcome to ADX Core Reseller Program</title>
        </head>
        <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
            <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                <h1 style="color: #2c3e50;">Welcome to ADX Core Reseller Program</h1>
                
                <p>Dear {{reseller_name}},</p>
                
                <p>Congratulations! You have been successfully enrolled in the ADX Core Reseller Program.</p>
                
                <p>As a reseller, you now have access to:</p>
                <ul>
                    <li>White-label branding capabilities</li>
                    <li>Custom domain setup</li>
                    <li>Revenue sharing program</li>
                    <li>Dedicated support channel</li>
                    <li>Marketing resources and materials</li>
                </ul>
                
                <p>To get started, please log in to your reseller dashboard:</p>
                <p><a href="{{login_url}}" style="background-color: #3498db; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Access Reseller Dashboard</a></p>
                
                <p>If you have any questions, please don't hesitate to contact our support team at {{support_email}}.</p>
                
                <p>Best regards,<br>The ADX Core Team</p>
            </div>
        </body>
        </html>
        "#.to_string()
    }

    fn get_reseller_welcome_text_template(&self) -> String {
        r#"
        Welcome to ADX Core Reseller Program
        
        Dear {{reseller_name}},
        
        Congratulations! You have been successfully enrolled in the ADX Core Reseller Program.
        
        As a reseller, you now have access to:
        - White-label branding capabilities
        - Custom domain setup
        - Revenue sharing program
        - Dedicated support channel
        - Marketing resources and materials
        
        To get started, please log in to your reseller dashboard: {{login_url}}
        
        If you have any questions, please don't hesitate to contact our support team at {{support_email}}.
        
        Best regards,
        The ADX Core Team
        "#.to_string()
    }

    fn get_domain_verification_html_template(&self) -> String {
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="utf-8">
            <title>Domain Verification Required</title>
        </head>
        <body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
            <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
                <h1 style="color: #2c3e50;">Domain Verification Required</h1>
                
                <p>Dear {{tenant_name}},</p>
                
                <p>You have requested to add the custom domain <strong>{{domain}}</strong> to your ADX Core account.</p>
                
                <p>To complete the setup, please verify domain ownership by adding the following DNS record:</p>
                
                <div style="background-color: #f8f9fa; padding: 15px; border-radius: 5px; margin: 20px 0;">
                    <strong>Record Type:</strong> TXT<br>
                    <strong>Name:</strong> _adx-verification.{{domain}}<br>
                    <strong>Value:</strong> {{verification_token}}
                </div>
                
                <p>Once you've added the DNS record, click the button below to verify:</p>
                <p><a href="{{verification_url}}" style="background-color: #28a745; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Verify Domain</a></p>
                
                <p>If you have any questions, please contact our support team.</p>
                
                <p>Best regards,<br>The ADX Core Team</p>
            </div>
        </body>
        </html>
        "#.to_string()
    }

    fn get_domain_verification_text_template(&self) -> String {
        r#"
        Domain Verification Required
        
        Dear {{tenant_name}},
        
        You have requested to add the custom domain {{domain}} to your ADX Core account.
        
        To complete the setup, please verify domain ownership by adding the following DNS record:
        
        Record Type: TXT
        Name: _adx-verification.{{domain}}
        Value: {{verification_token}}
        
        Once you've added the DNS record, visit: {{verification_url}}
        
        If you have any questions, please contact our support team.
        
        Best regards,
        The ADX Core Team
        "#.to_string()
    }
}

// Handlebars helpers
fn uppercase_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(&param.to_uppercase())?;
    Ok(())
}

fn lowercase_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    out.write(&param.to_lowercase())?;
    Ok(())
}

fn format_date_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let param = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
    // Simple date formatting - in a real implementation, this would be more sophisticated
    out.write(param)?;
    Ok(())
}