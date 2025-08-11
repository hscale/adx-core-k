pub mod handlers;
pub mod middleware;
pub mod repositories;
pub mod routes;
pub mod server;
pub mod workflows;
pub mod worker;

use adx_shared::{
    config::AppConfig,
    auth::JwtManager,
    database::DatabasePool,
};
use crate::{
    middleware::rate_limit::RateLimiter,
    repositories::{UserRepository, SessionRepository, AuthTokenRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub jwt_manager: JwtManager,
    pub rate_limiter: RateLimiter,
    pub database_pool: DatabasePool,
    // TODO: Add Redis client when caching layer is implemented
    // pub redis_client: RedisClient,
}

impl AppState {
    /// Create a user repository for the given tenant
    pub fn user_repository(&self, tenant_id: &str) -> UserRepository {
        UserRepository::new(self.database_pool.clone(), tenant_id.to_string())
    }

    /// Create a session repository for the given tenant
    pub fn session_repository(&self, tenant_id: &str) -> SessionRepository {
        SessionRepository::new(self.database_pool.clone(), tenant_id.to_string())
    }

    /// Create an auth token repository for the given tenant
    pub fn auth_token_repository(&self, tenant_id: &str) -> AuthTokenRepository {
        AuthTokenRepository::new(self.database_pool.clone(), tenant_id.to_string())
    }
}

pub use server::AuthServer;
pub use worker::AuthWorker;