use axum::{
    extract::State,
    response::IntoResponse,
    http::{HeaderMap, StatusCode, Request},
    body::Body,
};
use std::sync::Arc;
use crate::{config::AppConfig, proxy::proxy_request_to_provider};

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn proxy_request(
    State(config): State<Arc<AppConfig>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> impl IntoResponse {
    let provider = headers
        .get("x-provider")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("openai");

    match proxy_request_to_provider(config, provider, request).await {
        Ok(response) => response.into_response(),
        Err(e) => {
            tracing::error!("Proxy error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
} 