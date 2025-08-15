use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::ModuleServiceError;
use crate::config::MarketplaceConfig;
use crate::types::{MarketplaceListing, ModuleSearchRequest, ModuleSearchResponse, PricingModel, ModuleCategory};
use crate::repositories::MarketplaceRepository;
use crate::activities::SyncAction;

#[async_trait]
pub trait MarketplaceServiceTrait {
    async fn search_modules(&self, request: ModuleSearchRequest) -> Result<ModuleSearchResponse, ModuleServiceError>;
    async fn get_module_details(&self, module_id: &str) -> Result<MarketplaceListing, ModuleServiceError>;
    async fn get_featured_modules(&self, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError>;
    async fn get_trending_modules(&self, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError>;
    async fn get_recommendations(&self, user_id: &str, tenant_id: &str, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError>;
    async fn submit_review(&self, module_id: &str, user_id: &str, tenant_id: &str, rating: u8, comment: Option<String>) -> Result<(), ModuleServiceError>;
    async fn process_payment(&self, module_id: &str, user_id: &str, tenant_id: &str, payment_method: &str) -> Result<PaymentResult, ModuleServiceError>;
    async fn fetch_modules(&self, sync_type: &str, module_ids: Option<&Vec<String>>, force_update: bool) -> Result<Vec<MarketplaceListing>, ModuleServiceError>;
    async fn sync_module(&self, module_data: &MarketplaceListing, force_update: bool) -> Result<SyncAction, ModuleServiceError>;
    async fn get_user_purchases(&self, user_id: &str, tenant_id: &str) -> Result<Vec<ModulePurchase>, ModuleServiceError>;
    async fn get_module_analytics(&self, module_id: &str) -> Result<ModuleAnalytics, ModuleServiceError>;
    async fn update_module_rating(&self, module_id: &str) -> Result<(), ModuleServiceError>;
    async fn get_category_recommendations(&self, category: &ModuleCategory, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError>;
    async fn track_module_view(&self, module_id: &str, user_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError>;
    async fn get_similar_modules(&self, module_id: &str, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError>;
}

pub struct MarketplaceService {
    config: MarketplaceConfig,
    client: reqwest::Client,
    repository: Option<MarketplaceRepository>,
    payment_processor: PaymentProcessor,
    recommendation_engine: RecommendationEngine,
    analytics_service: AnalyticsService,
    rating_service: RatingService,
}

impl MarketplaceService {
    pub fn new(config: MarketplaceConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            repository: None,
            payment_processor: PaymentProcessor::new(),
            recommendation_engine: RecommendationEngine::new(),
            analytics_service: AnalyticsService::new(),
            rating_service: RatingService::new(),
        }
    }

    pub fn with_repository(mut self, repository: MarketplaceRepository) -> Self {
        self.repository = Some(repository);
        self
    }
}

#[async_trait]
impl MarketplaceServiceTrait for MarketplaceService {
    async fn search_modules(&self, request: ModuleSearchRequest) -> Result<ModuleSearchResponse, ModuleServiceError> {
        if let Some(repository) = &self.repository {
            let mut response = repository.search_modules(request.clone()).await?;
            
            // Apply AI-powered ranking if query is provided
            if request.query.is_some() {
                response.modules = self.recommendation_engine.rank_search_results(
                    response.modules,
                    &request,
                ).await?;
            }
            
            Ok(response)
        } else {
            // Fallback to external API
            let url = format!("{}/api/v1/modules/search", self.config.api_url);
            let response = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(ModuleServiceError::MarketplaceError(
                    format!("Search failed: {}", response.status())
                ));
            }

            let search_response: ModuleSearchResponse = response.json().await?;
            Ok(search_response)
        }
    }

    async fn get_module_details(&self, module_id: &str) -> Result<MarketplaceListing, ModuleServiceError> {
        if let Some(repository) = &self.repository {
            repository.get_module_by_id(module_id).await?
                .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))
        } else {
            // Fallback to external API
            if !self.config.enabled {
                return Err(ModuleServiceError::MarketplaceError("Marketplace disabled".to_string()));
            }

            let url = format!("{}/api/v1/modules/{}", self.config.api_url, module_id);
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(ModuleServiceError::MarketplaceError(
                    format!("Failed to fetch module {}: {}", module_id, response.status())
                ));
            }

            let module: MarketplaceListing = response.json().await?;
            Ok(module)
        }
    }

    async fn get_featured_modules(&self, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        if let Some(repository) = &self.repository {
            repository.get_featured_modules(limit).await
        } else {
            let url = format!("{}/api/v1/modules/featured?limit={}", self.config.api_url, limit);
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(ModuleServiceError::MarketplaceError(
                    format!("Failed to fetch featured modules: {}", response.status())
                ));
            }

            let modules: Vec<MarketplaceListing> = response.json().await?;
            Ok(modules)
        }
    }

    async fn get_trending_modules(&self, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        let trending = self.analytics_service.get_trending_modules(limit).await?;
        let mut modules = Vec::new();
        
        for trend in trending {
            if let Ok(module) = self.get_module_details(&trend.module_id).await {
                modules.push(module);
            }
        }
        
        Ok(modules)
    }

    async fn get_recommendations(&self, user_id: &str, tenant_id: &str, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        self.recommendation_engine.get_personalized_recommendations(user_id, tenant_id, limit).await
    }

    async fn submit_review(&self, module_id: &str, user_id: &str, tenant_id: &str, rating: u8, comment: Option<String>) -> Result<(), ModuleServiceError> {
        if rating > 5 || rating < 1 {
            return Err(ModuleServiceError::ModuleValidationError("Rating must be between 1 and 5".to_string()));
        }

        // Check if user has purchased the module
        let purchases = self.get_user_purchases(user_id, tenant_id).await?;
        let has_purchased = purchases.iter().any(|p| p.module_id == module_id);

        if let Some(repository) = &self.repository {
            repository.create_review(module_id, user_id, tenant_id, rating, comment, has_purchased).await?;
            
            // Update module rating
            self.update_module_rating(module_id).await?;
        }
        
        Ok(())
    }

    async fn process_payment(&self, module_id: &str, user_id: &str, tenant_id: &str, payment_method: &str) -> Result<PaymentResult, ModuleServiceError> {
        // Get module pricing
        let module = self.get_module_details(module_id).await?;
        
        if let Some(price) = module.price {
            match price.model {
                PricingModel::Free => {
                    // Record free "purchase"
                    if let Some(repository) = &self.repository {
                        repository.record_purchase(module_id, user_id, tenant_id, 0.0, "USD", None).await?;
                    }
                    
                    Ok(PaymentResult {
                        success: true,
                        transaction_id: None,
                        amount: 0.0,
                        currency: "USD".to_string(),
                        purchase_id: Uuid::new_v4().to_string(),
                    })
                }
                PricingModel::OneTime | PricingModel::Subscription => {
                    let payment_result = self.payment_processor.process_payment(
                        price.amount.unwrap_or(0.0),
                        &price.currency,
                        payment_method,
                        user_id,
                        tenant_id,
                        module_id,
                    ).await?;
                    
                    if payment_result.success {
                        // Record purchase
                        if let Some(repository) = &self.repository {
                            repository.record_purchase(
                                module_id,
                                user_id,
                                tenant_id,
                                payment_result.amount,
                                &payment_result.currency,
                                payment_result.transaction_id.clone(),
                            ).await?;
                            
                            // Update download count
                            repository.increment_download_count(module_id).await?;
                        }
                    }
                    
                    Ok(payment_result)
                }
                PricingModel::Usage => {
                    // For usage-based pricing, create a subscription record
                    let subscription_result = self.payment_processor.create_usage_subscription(
                        module_id,
                        user_id,
                        tenant_id,
                        &price,
                    ).await?;
                    
                    Ok(PaymentResult {
                        success: subscription_result.success,
                        transaction_id: subscription_result.subscription_id,
                        amount: 0.0, // No upfront cost for usage-based
                        currency: price.currency,
                        purchase_id: Uuid::new_v4().to_string(),
                    })
                }
                PricingModel::Enterprise => {
                    // Enterprise pricing requires manual approval
                    self.payment_processor.create_enterprise_request(
                        module_id,
                        user_id,
                        tenant_id,
                    ).await?;
                    
                    Ok(PaymentResult {
                        success: false,
                        transaction_id: None,
                        amount: 0.0,
                        currency: "USD".to_string(),
                        purchase_id: "pending_enterprise_approval".to_string(),
                    })
                }
            }
        } else {
            // Free module
            if let Some(repository) = &self.repository {
                repository.record_purchase(module_id, user_id, tenant_id, 0.0, "USD", None).await?;
            }
            
            Ok(PaymentResult {
                success: true,
                transaction_id: None,
                amount: 0.0,
                currency: "USD".to_string(),
                purchase_id: Uuid::new_v4().to_string(),
            })
        }
    }

    async fn fetch_modules(&self, sync_type: &str, module_ids: Option<&Vec<String>>, force_update: bool) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        match sync_type {
            "full" => {
                if !self.config.enabled {
                    return Ok(vec![]);
                }

                let url = format!("{}/api/v1/modules", self.config.api_url);
                let response = self.client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", self.config.api_key))
                    .send()
                    .await?;

                if !response.status().is_success() {
                    return Err(ModuleServiceError::MarketplaceError(
                        format!("Failed to fetch modules: {}", response.status())
                    ));
                }

                let modules: Vec<MarketplaceListing> = response.json().await?;
                Ok(modules)
            }
            "incremental" => {
                if let Some(ids) = module_ids {
                    let mut modules = Vec::new();
                    for id in ids {
                        match self.get_module_details(id).await {
                            Ok(module) => modules.push(module),
                            Err(e) => {
                                tracing::warn!("Failed to fetch module {}: {}", id, e);
                            }
                        }
                    }
                    Ok(modules)
                } else if let Some(repository) = &self.repository {
                    repository.list_all_modules().await
                } else {
                    Ok(vec![])
                }
            }
            _ => Err(ModuleServiceError::MarketplaceError(
                format!("Unknown sync type: {}", sync_type)
            )),
        }
    }

    async fn sync_module(&self, module_data: &MarketplaceListing, force_update: bool) -> Result<SyncAction, ModuleServiceError> {
        if let Some(repository) = &self.repository {
            // Check if module exists
            if let Ok(existing) = repository.get_module_by_id(&module_data.id).await {
                if let Some(existing_module) = existing {
                    if existing_module.last_updated < module_data.last_updated || force_update {
                        repository.update_module(module_data).await?;
                        Ok(SyncAction::Updated)
                    } else {
                        Ok(SyncAction::NoChange)
                    }
                } else {
                    repository.create_module(module_data).await?;
                    Ok(SyncAction::Added)
                }
            } else {
                repository.create_module(module_data).await?;
                Ok(SyncAction::Added)
            }
        } else {
            Ok(SyncAction::NoChange)
        }
    }

    async fn get_user_purchases(&self, user_id: &str, tenant_id: &str) -> Result<Vec<ModulePurchase>, ModuleServiceError> {
        if let Some(repository) = &self.repository {
            repository.get_user_purchases(user_id, tenant_id).await
        } else {
            Ok(vec![])
        }
    }

    async fn get_module_analytics(&self, module_id: &str) -> Result<ModuleAnalytics, ModuleServiceError> {
        self.analytics_service.get_module_analytics(module_id).await
    }

    async fn update_module_rating(&self, module_id: &str) -> Result<(), ModuleServiceError> {
        let new_rating = self.rating_service.calculate_module_rating(module_id).await?;
        if let Some(repository) = &self.repository {
            repository.update_module_rating(module_id, new_rating.average, new_rating.count).await?;
        }
        Ok(())
    }

    async fn get_category_recommendations(&self, category: &ModuleCategory, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        if let Some(repository) = &self.repository {
            repository.get_modules_by_category(category, limit).await
        } else {
            Ok(vec![])
        }
    }

    async fn track_module_view(&self, module_id: &str, user_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError> {
        self.analytics_service.track_module_view(module_id, user_id, tenant_id).await
    }

    async fn get_similar_modules(&self, module_id: &str, limit: usize) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        self.recommendation_engine.get_similar_modules(module_id, limit).await
    }
}

// Supporting types and services

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub purchase_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePurchase {
    pub purchase_id: String,
    pub module_id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub amount: f64,
    pub currency: String,
    pub transaction_id: Option<String>,
    pub purchased_at: DateTime<Utc>,
    pub subscription_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAnalytics {
    pub module_id: String,
    pub total_views: u64,
    pub total_downloads: u64,
    pub total_purchases: u64,
    pub revenue: f64,
    pub rating_average: f32,
    pub rating_count: u32,
    pub views_last_30_days: u64,
    pub downloads_last_30_days: u64,
    pub conversion_rate: f64,
    pub top_countries: Vec<CountryStats>,
    pub usage_trends: Vec<UsageTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryStats {
    pub country_code: String,
    pub views: u64,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrend {
    pub date: DateTime<Utc>,
    pub views: u64,
    pub downloads: u64,
    pub revenue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingModule {
    pub module_id: String,
    pub trend_score: f64,
    pub growth_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRating {
    pub average: f32,
    pub count: u32,
    pub distribution: HashMap<u8, u32>, // rating -> count
}

pub struct PaymentProcessor {
    stripe_client: Option<StripeClient>,
    paypal_client: Option<PayPalClient>,
}

impl PaymentProcessor {
    pub fn new() -> Self {
        Self {
            stripe_client: Some(StripeClient::new()),
            paypal_client: Some(PayPalClient::new()),
        }
    }

    pub async fn process_payment(
        &self,
        amount: f64,
        currency: &str,
        payment_method: &str,
        user_id: &str,
        tenant_id: &str,
        module_id: &str,
    ) -> Result<PaymentResult, ModuleServiceError> {
        if amount <= 0.0 {
            return Err(ModuleServiceError::PaymentError("Invalid amount".to_string()));
        }

        match payment_method {
            "stripe" => {
                if let Some(client) = &self.stripe_client {
                    client.process_payment(amount, currency, user_id, tenant_id, module_id).await
                } else {
                    Err(ModuleServiceError::PaymentError("Stripe not configured".to_string()))
                }
            }
            "paypal" => {
                if let Some(client) = &self.paypal_client {
                    client.process_payment(amount, currency, user_id, tenant_id, module_id).await
                } else {
                    Err(ModuleServiceError::PaymentError("PayPal not configured".to_string()))
                }
            }
            _ => Err(ModuleServiceError::PaymentError("Unsupported payment method".to_string())),
        }
    }

    pub async fn create_usage_subscription(
        &self,
        module_id: &str,
        user_id: &str,
        tenant_id: &str,
        price: &crate::types::ModulePrice,
    ) -> Result<SubscriptionResult, ModuleServiceError> {
        // Create usage-based subscription
        Ok(SubscriptionResult {
            success: true,
            subscription_id: Some(Uuid::new_v4().to_string()),
        })
    }

    pub async fn create_enterprise_request(
        &self,
        module_id: &str,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<(), ModuleServiceError> {
        // Create enterprise pricing request
        // This would typically send an email to sales team
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SubscriptionResult {
    pub success: bool,
    pub subscription_id: Option<String>,
}

pub struct StripeClient;

impl StripeClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_payment(
        &self,
        amount: f64,
        currency: &str,
        user_id: &str,
        tenant_id: &str,
        module_id: &str,
    ) -> Result<PaymentResult, ModuleServiceError> {
        // Mock Stripe payment processing
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(PaymentResult {
            success: true,
            transaction_id: Some(format!("stripe_{}", Uuid::new_v4())),
            amount,
            currency: currency.to_string(),
            purchase_id: Uuid::new_v4().to_string(),
        })
    }
}

pub struct PayPalClient;

impl PayPalClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_payment(
        &self,
        amount: f64,
        currency: &str,
        user_id: &str,
        tenant_id: &str,
        module_id: &str,
    ) -> Result<PaymentResult, ModuleServiceError> {
        // Mock PayPal payment processing
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        Ok(PaymentResult {
            success: true,
            transaction_id: Some(format!("paypal_{}", Uuid::new_v4())),
            amount,
            currency: currency.to_string(),
            purchase_id: Uuid::new_v4().to_string(),
        })
    }
}

pub struct RecommendationEngine {
    user_profiles: HashMap<String, UserProfile>,
    module_similarity: HashMap<String, Vec<String>>,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            user_profiles: HashMap::new(),
            module_similarity: HashMap::new(),
        }
    }

    pub async fn get_personalized_recommendations(
        &self,
        user_id: &str,
        tenant_id: &str,
        limit: usize,
    ) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        // Mock personalized recommendations based on:
        // - User's installed modules
        // - Tenant industry/type
        // - Similar users' preferences
        // - Module popularity trends
        
        // For now, return empty recommendations
        Ok(Vec::new())
    }

    pub async fn rank_search_results(
        &self,
        mut modules: Vec<MarketplaceListing>,
        request: &ModuleSearchRequest,
    ) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        // Apply AI-powered ranking based on:
        // - Query relevance
        // - Module quality scores
        // - User preferences
        // - Popularity metrics
        
        modules.sort_by(|a, b| {
            // Simple ranking by rating and downloads
            let score_a = a.rating * (a.downloads as f32).log10();
            let score_b = b.rating * (b.downloads as f32).log10();
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(modules)
    }

    pub async fn get_similar_modules(
        &self,
        module_id: &str,
        limit: usize,
    ) -> Result<Vec<MarketplaceListing>, ModuleServiceError> {
        // Find similar modules based on:
        // - Category
        // - Tags
        // - Functionality
        // - User co-installation patterns
        
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub tenant_id: String,
    pub installed_modules: Vec<String>,
    pub preferred_categories: Vec<ModuleCategory>,
    pub activity_score: f64,
}

pub struct AnalyticsService {
    metrics_store: HashMap<String, ModuleAnalytics>,
}

impl AnalyticsService {
    pub fn new() -> Self {
        Self {
            metrics_store: HashMap::new(),
        }
    }

    pub async fn get_module_analytics(&self, module_id: &str) -> Result<ModuleAnalytics, ModuleServiceError> {
        // Mock analytics data
        Ok(ModuleAnalytics {
            module_id: module_id.to_string(),
            total_views: 1000,
            total_downloads: 150,
            total_purchases: 120,
            revenue: 2400.0,
            rating_average: 4.2,
            rating_count: 45,
            views_last_30_days: 200,
            downloads_last_30_days: 25,
            conversion_rate: 0.15,
            top_countries: vec![
                CountryStats {
                    country_code: "US".to_string(),
                    views: 400,
                    downloads: 60,
                },
                CountryStats {
                    country_code: "GB".to_string(),
                    views: 200,
                    downloads: 30,
                },
            ],
            usage_trends: vec![],
        })
    }

    pub async fn get_trending_modules(&self, limit: usize) -> Result<Vec<TrendingModule>, ModuleServiceError> {
        // Calculate trending modules based on recent activity
        Ok(vec![
            TrendingModule {
                module_id: "trending-module-1".to_string(),
                trend_score: 0.85,
                growth_rate: 0.25,
            },
            TrendingModule {
                module_id: "trending-module-2".to_string(),
                trend_score: 0.78,
                growth_rate: 0.18,
            },
        ])
    }

    pub async fn track_module_view(
        &self,
        module_id: &str,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<(), ModuleServiceError> {
        // Track module view for analytics
        // This would typically write to a time-series database
        Ok(())
    }
}

pub struct RatingService;

impl RatingService {
    pub fn new() -> Self {
        Self
    }

    pub async fn calculate_module_rating(&self, module_id: &str) -> Result<ModuleRating, ModuleServiceError> {
        // Mock rating calculation
        // In production, this would aggregate all reviews for the module
        Ok(ModuleRating {
            average: 4.2,
            count: 45,
            distribution: {
                let mut dist = HashMap::new();
                dist.insert(5, 20);
                dist.insert(4, 15);
                dist.insert(3, 8);
                dist.insert(2, 2);
                dist.insert(1, 0);
                dist
            },
        })
    }
}