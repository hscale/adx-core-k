use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
// Stripe integration using direct HTTP API calls
use uuid::Uuid;

use crate::{
    config::{StripeConfig, PayPalConfig, BillingConfig},
    error::{LicenseError, Result},
    models::*,
};

#[derive(Debug, Clone)]
pub struct BillingService {
    stripe_client: Option<StripeHttpClient>,
    paypal_client: Option<PayPalClient>,
    config: BillingConfig,
}

impl BillingService {
    pub fn new(
        stripe_config: Option<StripeConfig>,
        paypal_config: Option<PayPalConfig>,
        billing_config: BillingConfig,
    ) -> Self {
        let stripe_client = stripe_config.map(|config| {
            StripeHttpClient::new(config)
        });

        let paypal_client = paypal_config.map(|config| {
            PayPalClient::new(config)
        });

        Self {
            stripe_client,
            paypal_client,
            config: billing_config,
        }
    }

    pub async fn create_customer(&self, tenant_id: Uuid, email: &str, name: &str) -> Result<String> {
        if let Some(ref client) = self.stripe_client {
            client.create_customer(tenant_id, email, name).await
        } else {
            Err(LicenseError::ConfigError("Stripe not configured".to_string()))
        }
    }

    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
        billing_cycle: BillingCycle,
    ) -> Result<String> {
        if let Some(ref client) = self.stripe_client {
            client.create_subscription(customer_id, price_id, billing_cycle).await
        } else {
            Err(LicenseError::ConfigError("Stripe not configured".to_string()))
        }
    }

    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<()> {
        if let Some(ref client) = self.stripe_client {
            client.cancel_subscription(subscription_id).await
        } else {
            Err(LicenseError::ConfigError("Stripe not configured".to_string()))
        }
    }

    pub async fn create_invoice(&self, invoice: &BillingInvoice) -> Result<String> {
        if let Some(ref client) = self.stripe_client {
            client.create_invoice(invoice).await
        } else {
            Err(LicenseError::ConfigError("Stripe not configured".to_string()))
        }
    }

    pub async fn process_payment(&self, amount: Decimal, currency: &str, customer_id: &str) -> Result<PaymentResult> {
        if let Some(ref client) = self.stripe_client {
            client.process_payment(amount, currency, customer_id).await
        } else {
            Err(LicenseError::ConfigError("Stripe not configured".to_string()))
        }
    }

    pub async fn generate_invoice_number(&self) -> String {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let random_suffix = uuid::Uuid::new_v4().to_string()[..8].to_uppercase();
        format!("{}-{}-{}", self.config.invoice_prefix, timestamp, random_suffix)
    }

    pub async fn calculate_usage_billing(&self, tenant_id: Uuid, usage_logs: &[UsageLog]) -> Result<BillingInvoice> {
        let mut line_items = Vec::new();
        let mut total_amount = Decimal::ZERO;

        // Group usage by operation type
        let mut usage_by_type: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        
        for log in usage_logs {
            let operation_type = log.operation_type.as_deref().unwrap_or("general");
            *usage_by_type.entry(operation_type.to_string()).or_insert(0) += log.amount;
        }

        // Calculate billing for each usage type
        for (operation_type, total_usage) in usage_by_type {
            let (unit_price, description) = self.get_usage_pricing(&operation_type);
            let line_total = unit_price * Decimal::from(total_usage);
            
            line_items.push(BillingLineItem {
                description,
                quantity: total_usage,
                unit_price,
                total_price: line_total,
                item_type: "usage".to_string(),
            });
            
            total_amount += line_total;
        }

        // Calculate tax
        let tax_amount = total_amount * Decimal::from(self.config.tax_rate);
        if tax_amount > Decimal::ZERO {
            line_items.push(BillingLineItem {
                description: format!("Tax ({}%)", self.config.tax_rate * 100.0),
                quantity: 1,
                unit_price: tax_amount,
                total_price: tax_amount,
                item_type: "tax".to_string(),
            });
        }

        let invoice_number = self.generate_invoice_number().await;
        let billing_period_start = usage_logs.iter()
            .map(|log| log.recorded_at)
            .min()
            .unwrap_or_else(Utc::now);
        let billing_period_end = usage_logs.iter()
            .map(|log| log.recorded_at)
            .max()
            .unwrap_or_else(Utc::now);

        Ok(BillingInvoice {
            invoice_number,
            tenant_id,
            amount: total_amount + tax_amount,
            currency: self.config.default_currency.clone(),
            tax_amount,
            billing_period_start,
            billing_period_end,
            line_items,
            usage_summary: Some(serde_json::to_value(&usage_by_type)?),
        })
    }

    fn get_usage_pricing(&self, operation_type: &str) -> (Decimal, String) {
        match operation_type {
            "api_call" => (Decimal::from_str("0.001").unwrap(), "API Calls".to_string()),
            "workflow_execution" => (Decimal::from_str("0.01").unwrap(), "Workflow Executions".to_string()),
            "file_storage_gb" => (Decimal::from_str("0.10").unwrap(), "File Storage (GB)".to_string()),
            "file_upload" => (Decimal::from_str("0.005").unwrap(), "File Uploads".to_string()),
            _ => (Decimal::from_str("0.001").unwrap(), format!("Usage: {}", operation_type)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PayPalClient {
    config: PayPalConfig,
    client: reqwest::Client,
}

impl PayPalClient {
    pub fn new(config: PayPalConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_subscription(&self, plan_id: &str, customer_email: &str) -> Result<String> {
        let base_url = if self.config.environment == "sandbox" {
            "https://api.sandbox.paypal.com"
        } else {
            "https://api.paypal.com"
        };

        // Get access token
        let access_token = self.get_access_token().await?;

        // Create subscription
        let subscription_request = serde_json::json!({
            "plan_id": plan_id,
            "subscriber": {
                "email_address": customer_email
            },
            "application_context": {
                "brand_name": "ADX Core",
                "user_action": "SUBSCRIBE_NOW",
                "return_url": "https://adxcore.com/billing/success",
                "cancel_url": "https://adxcore.com/billing/cancel"
            }
        });

        let response = self.client
            .post(&format!("{}/v1/billing/subscriptions", base_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&subscription_request)
            .send()
            .await?;

        if response.status().is_success() {
            let subscription: serde_json::Value = response.json().await?;
            Ok(subscription["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("PayPal error: {}", error_text)))
        }
    }

    async fn get_access_token(&self) -> Result<String> {
        let base_url = if self.config.environment == "sandbox" {
            "https://api.sandbox.paypal.com"
        } else {
            "https://api.paypal.com"
        };

        let auth = base64::encode(format!("{}:{}", self.config.client_id, self.config.client_secret));
        
        let response = self.client
            .post(&format!("{}/v1/oauth2/token", base_url))
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            Ok(token_response["access_token"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("PayPal auth error: {}", error_text)))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResult {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub currency: String,
    pub client_secret: Option<String>,
}

use base64;
use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct StripeHttpClient {
    client: reqwest::Client,
    config: StripeConfig,
}

impl StripeHttpClient {
    pub fn new(config: StripeConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn create_customer(&self, tenant_id: Uuid, email: &str, name: &str) -> Result<String> {
        let params = [
            ("email", email),
            ("name", name),
            ("metadata[tenant_id]", &tenant_id.to_string()),
            ("metadata[source]", "adx_core"),
        ];

        let response = self.client
            .post("https://api.stripe.com/v1/customers")
            .header("Authorization", format!("Bearer {}", self.config.secret_key))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let customer: serde_json::Value = response.json().await?;
            Ok(customer["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("Stripe customer creation failed: {}", error_text)))
        }
    }

    pub async fn create_subscription(&self, customer_id: &str, price_id: &str, _billing_cycle: BillingCycle) -> Result<String> {
        let params = [
            ("customer", customer_id),
            ("items[0][price]", price_id),
            ("metadata[source]", "adx_core"),
        ];

        let response = self.client
            .post("https://api.stripe.com/v1/subscriptions")
            .header("Authorization", format!("Bearer {}", self.config.secret_key))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let subscription: serde_json::Value = response.json().await?;
            Ok(subscription["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("Stripe subscription creation failed: {}", error_text)))
        }
    }

    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<()> {
        let response = self.client
            .delete(&format!("https://api.stripe.com/v1/subscriptions/{}", subscription_id))
            .header("Authorization", format!("Bearer {}", self.config.secret_key))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("Stripe subscription cancellation failed: {}", error_text)))
        }
    }

    pub async fn create_invoice(&self, invoice: &BillingInvoice) -> Result<String> {
        let params = [
            ("customer", invoice.tenant_id.to_string().as_str()), // This should be customer_id
            ("currency", invoice.currency.as_str()),
            ("description", &format!("Invoice {} for period {} to {}", 
                invoice.invoice_number,
                invoice.billing_period_start.format("%Y-%m-%d"),
                invoice.billing_period_end.format("%Y-%m-%d")
            )),
            ("metadata[invoice_number]", invoice.invoice_number.as_str()),
            ("metadata[tenant_id]", &invoice.tenant_id.to_string()),
        ];

        let response = self.client
            .post("https://api.stripe.com/v1/invoices")
            .header("Authorization", format!("Bearer {}", self.config.secret_key))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let created_invoice: serde_json::Value = response.json().await?;
            Ok(created_invoice["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("Stripe invoice creation failed: {}", error_text)))
        }
    }

    pub async fn process_payment(&self, amount: Decimal, currency: &str, customer_id: &str) -> Result<PaymentResult> {
        let amount_cents = (amount * Decimal::from(100)).to_i64().unwrap_or(0);
        
        let params = [
            ("amount", amount_cents.to_string().as_str()),
            ("currency", currency),
            ("customer", customer_id),
            ("automatic_payment_methods[enabled]", "true"),
            ("metadata[source]", "adx_core"),
        ];

        let response = self.client
            .post("https://api.stripe.com/v1/payment_intents")
            .header("Authorization", format!("Bearer {}", self.config.secret_key))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let payment_intent: serde_json::Value = response.json().await?;
            
            let status = match payment_intent["status"].as_str().unwrap_or("") {
                "succeeded" => PaymentStatus::Completed,
                "requires_payment_method" => PaymentStatus::Pending,
                "canceled" => PaymentStatus::Cancelled,
                _ => PaymentStatus::Pending,
            };

            Ok(PaymentResult {
                payment_id: payment_intent["id"].as_str().unwrap_or("").to_string(),
                status,
                amount,
                currency: currency.to_string(),
                client_secret: payment_intent["client_secret"].as_str().map(|s| s.to_string()),
            })
        } else {
            let error_text = response.text().await?;
            Err(LicenseError::PaymentError(format!("Stripe payment processing failed: {}", error_text)))
        }
    }
}

impl From<&str> for Decimal {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(Decimal::ZERO)
    }
}

// Helper trait for decimal parsing
trait DecimalFromStr {
    fn from_str(s: &str) -> Result<Decimal, rust_decimal::Error>;
}

impl DecimalFromStr for Decimal {
    fn from_str(s: &str) -> Result<Decimal, rust_decimal::Error> {
        s.parse()
    }
}