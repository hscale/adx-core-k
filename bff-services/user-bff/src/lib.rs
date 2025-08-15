pub mod middleware;
pub mod routes;
pub mod services;
pub mod types;

pub use services::{api_client::ApiClient, redis::RedisService, temporal_client::TemporalClient};
pub use types::*;

#[derive(Clone)]
pub struct AppState {
    pub api_client: ApiClient,
    pub redis: RedisService,
    pub temporal_client: TemporalClient,
}