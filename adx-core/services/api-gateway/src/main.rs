use adx_shared::{init_tracing, RequestContext, TenantId, UserId};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{any, get},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub service_registry: Arc<ServiceRegistry>,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let service_registry = Arc::new(ServiceRegistry::new());
    let app_state = AppState { service_registry };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/*path", any(proxy_request))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    auth_middleware,
                )),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!(
        "API Gateway listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn auth_middleware(
    State(_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Skip auth for health check and auth endpoints
    if path == "/health" || path.starts_with("/api/v1/auth/") {
        return Ok(next.run(request).await);
    }

    // Extract authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if let Some(token) = auth_header {
        // Validate token with auth service
        match validate_token_with_auth_service(token).await {
            Ok(claims) => {
                let user_context = RequestContext::new(claims.tenant_id, Some(claims.user_id));
                request.extensions_mut().insert(user_context);
            }
            Err(_) => {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        // For development, allow requests without auth to some endpoints
        let user_context = RequestContext::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(), // Demo tenant
            Some(Uuid::new_v4()),
        );
        request.extensions_mut().insert(user_context);
    }

    Ok(next.run(request).await)
}

#[derive(serde::Deserialize)]
struct TokenClaims {
    user_id: UserId,
    tenant_id: TenantId,
}

async fn validate_token_with_auth_service(token: &str) -> Result<TokenClaims, StatusCode> {
    // Mock validation for now - replace with actual auth service call
    if token == "valid_token" {
        Ok(TokenClaims {
            user_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
        })
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn proxy_request(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    let service_name = determine_service_from_path(path);

    let service_url = state
        .service_registry
        .get_service_url(&service_name)
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Extract request details
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build target URL
    let target_url = format!(
        "{}{}",
        service_url,
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("")
    ); // Create HTTP client and forward request
    let client = reqwest::Client::new();
    let method_str = method.as_str();
    let req_method =
        reqwest::Method::from_bytes(method_str.as_bytes()).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut req_builder = client.request(req_method, &target_url);

    // Forward headers (except host)
    for (name, value) in headers.iter() {
        if name != "host" {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_bytes(name.as_str().as_bytes()),
                reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
            ) {
                req_builder = req_builder.header(header_name, header_value);
            }
        }
    }

    // Forward body if present
    if !body.is_empty() {
        req_builder = req_builder.body(body);
    }

    // Send request
    let response = req_builder.send().await.map_err(|e| {
        tracing::error!("Failed to proxy request: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    // Build response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let axum_status =
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response_builder = Response::builder().status(axum_status);

    // Forward response headers
    for (name, value) in headers.iter() {
        if let (Ok(header_name), Ok(header_value)) = (
            axum::http::HeaderName::from_bytes(name.as_str().as_bytes()),
            axum::http::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            response_builder = response_builder.header(header_name, header_value);
        }
    }

    response_builder
        .body(body.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub struct ServiceRegistry {
    services: std::collections::HashMap<String, String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        let mut services = std::collections::HashMap::new();
        services.insert("users".to_string(), "http://localhost:8082".to_string());
        services.insert("files".to_string(), "http://localhost:8083".to_string());
        services.insert("workflows".to_string(), "http://localhost:8084".to_string());
        services.insert("tenants".to_string(), "http://localhost:8085".to_string());
        services.insert("auth".to_string(), "http://localhost:8081".to_string());

        Self { services }
    }

    pub fn get_service_url(&self, service_name: &str) -> Option<&String> {
        self.services.get(service_name)
    }
}

fn determine_service_from_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 4 {
        parts[3].to_string()
    } else {
        "unknown".to_string()
    }
}
