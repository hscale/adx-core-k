use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use semver::Version;
use reqwest::Client;
use tokio::sync::RwLock;

use crate::{
    ModuleResult, ModuleError, ModuleMetadata, ModulePackage, ModuleSearchQuery,
    ModuleSearchResult, ModuleMarketplace as ModuleMarketplaceTrait, ModuleReview,
    ModulePurchase, PurchaseResult, PurchaseStatus, PaymentMethod,
};

/// Comprehensive module marketplace with payment processing and recommendations
pub struct ModuleMarketplace {
    /// HTTP client for marketplace API
    client: Client,
    
    /// Marketplace configuration
    config: MarketplaceConfig,
    
    /// Payment processor
    payment_processor: Arc<PaymentProcessor>,
    
    /// Recommendation engine
    recommendation_engine: Arc<RecommendationEngine>,
    
    /// Analytics service
    analytics: Arc<MarketplaceAnalytics>,
    
    /// Review system
    review_system: Arc<ReviewSystem>,
    
    /// Local cache
    cache: Arc<RwLock<MarketplaceCache>>,
}

#[derive(Debug, Clone)]
pub struct MarketplaceConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub enable_analytics: bool,
    pub enable_recommendations: bool,
    pub payment_providers: Vec<PaymentProvider>,
}

#[derive(Debug, Clone)]
pub enum PaymentProvider {
    Stripe { secret_key: String },
    PayPal { client_id: String, client_secret: String },
    Square { access_token: String },
    Enterprise { webhook_url: String },
}

impl ModuleMarketplace {
    pub fn new(config: MarketplaceConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            payment_processor: Arc::new(PaymentProcessor::new(config.payment_providers.clone())),
            recommendation_engine: Arc::new(RecommendationEngine::new()),
            analytics: Arc::new(MarketplaceAnalytics::new()),
            review_system: Arc::new(ReviewSystem::new()),
            cache: Arc::new(RwLock::new(MarketplaceCache::new())),
            config,
        }
    }

    /// Get module recommendations based on user context
    pub async fn get_recommendations(
        &self,
        user_context: &UserContext,
        limit: usize,
    ) -> ModuleResult<Vec<ModuleRecommendation>> {
        if !self.config.enable_recommendations {
            return Ok(vec![]);
        }

        self.recommendation_engine
            .generate_recommendations(user_context, limit)
            .await
    }

    /// Get trending modules
    pub async fn get_trending_modules(
        &self,
        timeframe: TrendingTimeframe,
        limit: usize,
    ) -> ModuleResult<Vec<ModuleMetadata>> {
        let cache_key = format!("trending:{}:{}", timeframe.as_str(), limit);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get_trending(&cache_key) {
                return Ok(cached);
            }
        }

        // Fetch from API
        let url = format!("{}/api/v1/modules/trending", self.config.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .query(&[
                ("timeframe", timeframe.as_str()),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to fetch trending modules: {}", response.status())
            ));
        }

        let trending_response: TrendingModulesResponse = response.json().await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.set_trending(cache_key, trending_response.modules.clone());
        }

        Ok(trending_response.modules)
    }

    /// Get module analytics
    pub async fn get_module_analytics(
        &self,
        module_id: &str,
        timeframe: AnalyticsTimeframe,
    ) -> ModuleResult<ModuleAnalytics> {
        if !self.config.enable_analytics {
            return Err(ModuleError::ConfigurationError("Analytics disabled".to_string()));
        }

        self.analytics.get_module_analytics(module_id, timeframe).await
    }

    /// Submit module for review and approval
    pub async fn submit_module(
        &self,
        submission: ModuleSubmission,
    ) -> ModuleResult<SubmissionResult> {
        let url = format!("{}/api/v1/modules/submit", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&submission)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to submit module: {}", response.status())
            ));
        }

        let result: SubmissionResult = response.json().await?;
        Ok(result)
    }

    /// Get module pricing information
    pub async fn get_module_pricing(
        &self,
        module_id: &str,
        tenant_id: &str,
    ) -> ModuleResult<ModulePricing> {
        let url = format!("{}/api/v1/modules/{}/pricing", self.config.base_url, module_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .query(&[("tenant_id", tenant_id)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to fetch pricing: {}", response.status())
            ));
        }

        let pricing: ModulePricing = response.json().await?;
        Ok(pricing)
    }

    /// Process module license renewal
    pub async fn renew_license(
        &self,
        renewal_request: LicenseRenewalRequest,
    ) -> ModuleResult<LicenseRenewalResult> {
        // Process payment for renewal
        let payment_result = self.payment_processor
            .process_payment(PaymentRequest {
                amount: renewal_request.amount,
                currency: renewal_request.currency.clone(),
                payment_method: renewal_request.payment_method.clone(),
                description: format!("License renewal for module {}", renewal_request.module_id),
                metadata: serde_json::json!({
                    "module_id": renewal_request.module_id,
                    "tenant_id": renewal_request.tenant_id,
                    "renewal_period": renewal_request.renewal_period
                }),
            })
            .await?;

        if payment_result.status != PaymentStatus::Completed {
            return Err(ModuleError::PaymentError(
                format!("Payment failed: {:?}", payment_result.status)
            ));
        }

        // Update license
        let url = format!("{}/api/v1/licenses/renew", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&LicenseUpdateRequest {
                module_id: renewal_request.module_id.clone(),
                tenant_id: renewal_request.tenant_id.clone(),
                payment_id: payment_result.transaction_id,
                renewal_period: renewal_request.renewal_period,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to renew license: {}", response.status())
            ));
        }

        let result: LicenseRenewalResult = response.json().await?;
        Ok(result)
    }
}

#[async_trait]
impl ModuleMarketplaceTrait for ModuleMarketplace {
    async fn search(&self, query: &ModuleSearchQuery) -> ModuleResult<ModuleSearchResult> {
        let cache_key = format!("search:{}", serde_json::to_string(query)?);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get_search(&cache_key) {
                return Ok(cached);
            }
        }

        let url = format!("{}/api/v1/modules/search", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(query)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Search failed: {}", response.status())
            ));
        }

        let result: ModuleSearchResult = response.json().await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.set_search(cache_key, result.clone());
        }

        // Track search analytics
        if self.config.enable_analytics {
            self.analytics.track_search(query, &result).await?;
        }

        Ok(result)
    }

    async fn get_module(&self, module_id: &str) -> ModuleResult<Option<ModuleMetadata>> {
        let cache_key = format!("module:{}", module_id);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get_module(&cache_key) {
                return Ok(Some(cached));
            }
        }

        let url = format!("{}/api/v1/modules/{}", self.config.base_url, module_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to fetch module: {}", response.status())
            ));
        }

        let metadata: ModuleMetadata = response.json().await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.set_module(cache_key, metadata.clone());
        }

        Ok(Some(metadata))
    }

    async fn download(&self, module_id: &str, version: &str) -> ModuleResult<ModulePackage> {
        let url = format!("{}/api/v1/modules/{}/versions/{}/download", 
                         self.config.base_url, module_id, version);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Download failed: {}", response.status())
            ));
        }

        let package_data = response.bytes().await?;
        
        // Parse package
        let package = self.parse_module_package(package_data.to_vec()).await?;
        
        // Track download analytics
        if self.config.enable_analytics {
            self.analytics.track_download(module_id, version).await?;
        }

        Ok(package)
    }

    async fn get_reviews(&self, module_id: &str) -> ModuleResult<Vec<ModuleReview>> {
        self.review_system.get_reviews(module_id).await
    }

    async fn submit_review(&self, review: &ModuleReview) -> ModuleResult<()> {
        self.review_system.submit_review(review).await
    }

    async fn get_featured(&self) -> ModuleResult<Vec<ModuleMetadata>> {
        let cache_key = "featured_modules".to_string();
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get_featured(&cache_key) {
                return Ok(cached);
            }
        }

        let url = format!("{}/api/v1/modules/featured", self.config.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to fetch featured modules: {}", response.status())
            ));
        }

        let featured_response: FeaturedModulesResponse = response.json().await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.set_featured(cache_key, featured_response.modules.clone());
        }

        Ok(featured_response.modules)
    }

    async fn get_trending(&self) -> ModuleResult<Vec<ModuleMetadata>> {
        self.get_trending_modules(TrendingTimeframe::Week, 20).await
    }

    async fn purchase_module(&self, purchase: &ModulePurchase) -> ModuleResult<PurchaseResult> {
        // Get module pricing
        let pricing = self.get_module_pricing(&purchase.module_id, &purchase.tenant_id).await?;
        
        // Process payment
        let payment_result = self.payment_processor
            .process_payment(PaymentRequest {
                amount: pricing.price,
                currency: pricing.currency.clone(),
                payment_method: purchase.payment_method.clone(),
                description: format!("Purchase of module {}", purchase.module_id),
                metadata: serde_json::json!({
                    "module_id": purchase.module_id,
                    "version": purchase.version,
                    "tenant_id": purchase.tenant_id,
                    "user_id": purchase.user_id
                }),
            })
            .await?;

        // Create license
        let license_result = if payment_result.status == PaymentStatus::Completed {
            self.create_module_license(&purchase, &payment_result.transaction_id).await?
        } else {
            None
        };

        Ok(PurchaseResult {
            transaction_id: payment_result.transaction_id,
            status: payment_result.status,
            license_key: license_result,
            expires_at: pricing.license_duration.map(|duration| {
                Utc::now() + chrono::Duration::days(duration as i64)
            }),
        })
    }

    // Helper methods

    async fn parse_module_package(&self, data: Vec<u8>) -> ModuleResult<ModulePackage> {
        // Parse the downloaded package data
        // This would involve extracting metadata, manifest, and content
        todo!("Implement package parsing")
    }

    async fn create_module_license(
        &self,
        purchase: &ModulePurchase,
        transaction_id: &str,
    ) -> ModuleResult<Option<String>> {
        let url = format!("{}/api/v1/licenses/create", self.config.base_url);
        
        let license_request = CreateLicenseRequest {
            module_id: purchase.module_id.clone(),
            version: purchase.version.clone(),
            tenant_id: purchase.tenant_id.clone(),
            user_id: purchase.user_id.clone(),
            transaction_id: transaction_id.to_string(),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&license_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModuleError::MarketplaceError(
                format!("Failed to create license: {}", response.status())
            ));
        }

        let license_response: CreateLicenseResponse = response.json().await?;
        Ok(Some(license_response.license_key))
    }
}

// Supporting structures and services

pub struct PaymentProcessor {
    providers: Vec<PaymentProvider>,
}

impl PaymentProcessor {
    pub fn new(providers: Vec<PaymentProvider>) -> Self {
        Self { providers }
    }

    pub async fn process_payment(&self, request: PaymentRequest) -> ModuleResult<PaymentResult> {
        match &request.payment_method {
            PaymentMethod::CreditCard { token } => {
                self.process_credit_card_payment(token, &request).await
            }
            PaymentMethod::PayPal { account_id } => {
                self.process_paypal_payment(account_id, &request).await
            }
            PaymentMethod::BankTransfer { account_number } => {
                self.process_bank_transfer(account_number, &request).await
            }
            PaymentMethod::Enterprise { contract_id } => {
                self.process_enterprise_payment(contract_id, &request).await
            }
        }
    }

    async fn process_credit_card_payment(
        &self,
        token: &str,
        request: &PaymentRequest,
    ) -> ModuleResult<PaymentResult> {
        // Process credit card payment through Stripe or similar
        Ok(PaymentResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: PaymentStatus::Completed,
            amount: request.amount,
            currency: request.currency.clone(),
            processed_at: Utc::now(),
        })
    }

    async fn process_paypal_payment(
        &self,
        account_id: &str,
        request: &PaymentRequest,
    ) -> ModuleResult<PaymentResult> {
        // Process PayPal payment
        Ok(PaymentResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: PaymentStatus::Completed,
            amount: request.amount,
            currency: request.currency.clone(),
            processed_at: Utc::now(),
        })
    }

    async fn process_bank_transfer(
        &self,
        account_number: &str,
        request: &PaymentRequest,
    ) -> ModuleResult<PaymentResult> {
        // Process bank transfer
        Ok(PaymentResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: PaymentStatus::Pending, // Bank transfers are typically pending
            amount: request.amount,
            currency: request.currency.clone(),
            processed_at: Utc::now(),
        })
    }

    async fn process_enterprise_payment(
        &self,
        contract_id: &str,
        request: &PaymentRequest,
    ) -> ModuleResult<PaymentResult> {
        // Process enterprise payment (usually invoicing)
        Ok(PaymentResult {
            transaction_id: Uuid::new_v4().to_string(),
            status: PaymentStatus::Completed,
            amount: request.amount,
            currency: request.currency.clone(),
            processed_at: Utc::now(),
        })
    }
}

pub struct RecommendationEngine {
    // AI-powered recommendation system
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_recommendations(
        &self,
        user_context: &UserContext,
        limit: usize,
    ) -> ModuleResult<Vec<ModuleRecommendation>> {
        // Generate personalized module recommendations
        // This would use ML algorithms to recommend modules
        Ok(vec![])
    }
}

pub struct MarketplaceAnalytics {
    // Analytics and metrics collection
}

impl MarketplaceAnalytics {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn track_search(
        &self,
        query: &ModuleSearchQuery,
        result: &ModuleSearchResult,
    ) -> ModuleResult<()> {
        // Track search analytics
        Ok(())
    }

    pub async fn track_download(&self, module_id: &str, version: &str) -> ModuleResult<()> {
        // Track download analytics
        Ok(())
    }

    pub async fn get_module_analytics(
        &self,
        module_id: &str,
        timeframe: AnalyticsTimeframe,
    ) -> ModuleResult<ModuleAnalytics> {
        // Get module analytics data
        Ok(ModuleAnalytics {
            module_id: module_id.to_string(),
            downloads: 0,
            active_installations: 0,
            revenue: 0.0,
            rating_average: 0.0,
            review_count: 0,
            timeframe,
        })
    }
}

pub struct ReviewSystem {
    // Module review and rating system
}

impl ReviewSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_reviews(&self, module_id: &str) -> ModuleResult<Vec<ModuleReview>> {
        // Get module reviews
        Ok(vec![])
    }

    pub async fn submit_review(&self, review: &ModuleReview) -> ModuleResult<()> {
        // Submit module review
        Ok(())
    }
}

pub struct MarketplaceCache {
    // In-memory cache for marketplace data
    search_cache: HashMap<String, (ModuleSearchResult, DateTime<Utc>)>,
    module_cache: HashMap<String, (ModuleMetadata, DateTime<Utc>)>,
    featured_cache: HashMap<String, (Vec<ModuleMetadata>, DateTime<Utc>)>,
    trending_cache: HashMap<String, (Vec<ModuleMetadata>, DateTime<Utc>)>,
    ttl_seconds: u64,
}

impl MarketplaceCache {
    pub fn new() -> Self {
        Self {
            search_cache: HashMap::new(),
            module_cache: HashMap::new(),
            featured_cache: HashMap::new(),
            trending_cache: HashMap::new(),
            ttl_seconds: 300, // 5 minutes default TTL
        }
    }

    pub fn get_search(&self, key: &str) -> Option<ModuleSearchResult> {
        self.search_cache.get(key).and_then(|(result, timestamp)| {
            if self.is_expired(*timestamp) {
                None
            } else {
                Some(result.clone())
            }
        })
    }

    pub fn set_search(&mut self, key: String, result: ModuleSearchResult) {
        self.search_cache.insert(key, (result, Utc::now()));
    }

    pub fn get_module(&self, key: &str) -> Option<ModuleMetadata> {
        self.module_cache.get(key).and_then(|(metadata, timestamp)| {
            if self.is_expired(*timestamp) {
                None
            } else {
                Some(metadata.clone())
            }
        })
    }

    pub fn set_module(&mut self, key: String, metadata: ModuleMetadata) {
        self.module_cache.insert(key, (metadata, Utc::now()));
    }

    pub fn get_featured(&self, key: &str) -> Option<Vec<ModuleMetadata>> {
        self.featured_cache.get(key).and_then(|(modules, timestamp)| {
            if self.is_expired(*timestamp) {
                None
            } else {
                Some(modules.clone())
            }
        })
    }

    pub fn set_featured(&mut self, key: String, modules: Vec<ModuleMetadata>) {
        self.featured_cache.insert(key, (modules, Utc::now()));
    }

    pub fn get_trending(&self, key: &str) -> Option<Vec<ModuleMetadata>> {
        self.trending_cache.get(key).and_then(|(modules, timestamp)| {
            if self.is_expired(*timestamp) {
                None
            } else {
                Some(modules.clone())
            }
        })
    }

    pub fn set_trending(&mut self, key: String, modules: Vec<ModuleMetadata>) {
        self.trending_cache.insert(key, (modules, Utc::now()));
    }

    fn is_expired(&self, timestamp: DateTime<Utc>) -> bool {
        let elapsed = Utc::now().signed_duration_since(timestamp);
        elapsed.num_seconds() > self.ttl_seconds as i64
    }
}

// Additional types and structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub tenant_id: String,
    pub preferences: Vec<String>,
    pub installed_modules: Vec<String>,
    pub usage_patterns: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRecommendation {
    pub module_id: String,
    pub score: f64,
    pub reason: String,
    pub metadata: ModuleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendingTimeframe {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl TrendingTimeframe {
    pub fn as_str(&self) -> &str {
        match self {
            TrendingTimeframe::Day => "day",
            TrendingTimeframe::Week => "week",
            TrendingTimeframe::Month => "month",
            TrendingTimeframe::Quarter => "quarter",
            TrendingTimeframe::Year => "year",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsTimeframe {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAnalytics {
    pub module_id: String,
    pub downloads: u64,
    pub active_installations: u64,
    pub revenue: f64,
    pub rating_average: f64,
    pub review_count: u32,
    pub timeframe: AnalyticsTimeframe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSubmission {
    pub metadata: ModuleMetadata,
    pub package_data: Vec<u8>,
    pub documentation: String,
    pub screenshots: Vec<String>,
    pub demo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionResult {
    pub submission_id: String,
    pub status: SubmissionStatus,
    pub review_timeline: String,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmissionStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    RequiresChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePricing {
    pub module_id: String,
    pub price: f64,
    pub currency: String,
    pub pricing_model: PricingModel,
    pub license_duration: Option<u32>, // days
    pub trial_period: Option<u32>, // days
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Free,
    OneTime,
    Subscription,
    Usage,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub amount: f64,
    pub currency: String,
    pub payment_method: PaymentMethod,
    pub description: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub transaction_id: String,
    pub status: PaymentStatus,
    pub amount: f64,
    pub currency: String,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseRenewalRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub renewal_period: u32, // days
    pub amount: f64,
    pub currency: String,
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseRenewalResult {
    pub license_key: String,
    pub expires_at: DateTime<Utc>,
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLicenseRequest {
    pub module_id: String,
    pub version: String,
    pub tenant_id: String,
    pub user_id: String,
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLicenseResponse {
    pub license_key: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseUpdateRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub payment_id: String,
    pub renewal_period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingModulesResponse {
    pub modules: Vec<ModuleMetadata>,
    pub timeframe: TrendingTimeframe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturedModulesResponse {
    pub modules: Vec<ModuleMetadata>,
}