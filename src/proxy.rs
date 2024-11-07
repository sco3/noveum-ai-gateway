use axum::{
    body::{self, Body, Bytes},
    http::{Request, Response, HeaderMap, StatusCode}
};
use std::sync::Arc;
use http::HeaderValue;
use std::str::FromStr;
use crate::config::AppConfig;
use crate::error::AppError;
use tracing::{info, error};
use std::time::Duration;
use once_cell::sync::Lazy;
use futures_util::StreamExt;

/// Static HTTP client with optimized connection pooling and timeout settings
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(32)
        .tcp_keepalive(Duration::from_secs(60))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
});

/// Proxies incoming requests to the specified provider while maintaining optimal performance
/// through connection pooling and efficient streaming.
pub async fn proxy_request_to_provider(
    _config: Arc<AppConfig>,
    provider: &str,
    original_request: Request<Body>,
) -> Result<Response<Body>, AppError> {
    info!(
        provider = provider,
        method = %original_request.method(),
        path = %original_request.uri().path(),
        "Incoming request"
    );

    let base_url = match provider {
        "openai" => "https://api.openai.com",
        "anthropic" => "https://api.anthropic.com",
        _ => {
            error!(provider = provider, "Unsupported provider");
            return Err(AppError::UnsupportedProvider);
        }
    };

    let path = original_request.uri().path();
    let query = original_request
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    let url = format!("{}{}{}", base_url, path, query);
    info!(
        provider = provider,
        url = %url,
        method = %original_request.method(),
        "Preparing proxy request"
    );

    let method = reqwest::Method::from_str(original_request.method().as_str())
        .map_err(|_| AppError::InvalidMethod)?;
    
    // Optimize headers handling with pre-allocated capacity
    let mut reqwest_headers = reqwest::header::HeaderMap::with_capacity(8);
    reqwest_headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    // Efficient header handling for OpenAI
    if provider == "openai" {
        if let Some(api_key) = original_request.headers().get("x-portkey-api-key")
            .and_then(|h| h.to_str().ok()) {
            reqwest_headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|_| AppError::InvalidHeader)?
            );
        } else if let Some(auth) = original_request.headers().get("authorization")
            .and_then(|h| h.to_str().ok()) {
            reqwest_headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(auth)
                    .map_err(|_| AppError::InvalidHeader)?
            );
        }
    }

    // Efficiently handle request body
    let body_bytes = body::to_bytes(original_request.into_body(), usize::MAX).await?;
    
    let proxy_request = CLIENT
        .request(method, url)
        .headers(reqwest_headers)
        .body(body_bytes.to_vec());

    let response = proxy_request.send().await?;
    let status = StatusCode::from_u16(response.status().as_u16())?;

    // Optimize streaming response handling
    if response.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map_or(false, |ct| ct.contains("text/event-stream")) 
    {
        // Efficient headers copying with proper type conversion
        let mut response_headers = HeaderMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = HeaderValue::from_bytes(value.as_bytes()) {
                if let Ok(header_name) = http::HeaderName::from_bytes(name.as_ref()) {
                    response_headers.insert(header_name, v);
                }
            }
        }

        // Efficient stream handling with proper error mapping
        let stream = response.bytes_stream()
            .map(|result| match result {
                Ok(bytes) => Ok(Bytes::from(bytes)),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e))
            });

        Ok(Response::builder()
            .status(status)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive")
            .extension(response_headers)
            .body(Body::from_stream(stream))
            .unwrap())
    } else {
        let body = response.bytes().await?;
        Ok(Response::builder()
            .status(status)
            .body(Body::from(body))
            .unwrap())
    }
} 