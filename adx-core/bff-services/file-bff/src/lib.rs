pub mod middleware;
pub mod routes;
pub mod services;
pub mod types;

pub use services::{api_client::ApiClient, redis::RedisService};
pub use types::*;

#[derive(Clone)]
pub struct AppState {
    pub api_client: ApiClient,
    pub redis: RedisService,
}