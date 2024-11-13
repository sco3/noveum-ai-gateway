use axum::{
    body::{self, Body},
    http::{HeaderMap, HeaderValue, Request, Response, StatusCode},
};
use futures_util::StreamExt;
use reqwest::Method;
use std::sync::Arc;
use tracing::{debug, error, info};
use bytes::BytesMut;

use crate::{config::AppConfig, error::AppError, providers::create_provider};

mod client;
pub use client::CLIENT;

pub async fn proxy_request_to_provider(
    config: Arc<AppConfig>,
    provider_name: &str,
    original_request: Request<Body>,
) -> Result<Response<Body>, AppError> {
    info!(
        provider = provider_name,
        method = %original_request.method(),
        path = %original_request.uri().path(),
        "Incoming request"
    );

    debug!("Creating provider instance for: {}", provider_name);
    let provider = create_provider(provider_name)?;

    debug!("Executing before_request hook");
    provider.before_request(&original_request).await?;

    let path = original_request.uri().path();
    let modified_path = provider.transform_path(path);

    let query = original_request
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    let url = format!("{}{}{}", provider.base_url(), modified_path, query);

    debug!(
        provider = provider_name,
        url = %url,
        "Preparing proxy request"
    );

    // Process headers
    let headers = provider.process_headers(original_request.headers())?;

    // Create and send request with optimized buffer handling
    let response = send_provider_request(
        original_request.method().clone(),
        url,
        headers,
        original_request.into_body(),
        config.clone(),
    )
    .await?;

    // Process response
    let processed_response = provider.process_response(response).await?;

    // Call after request hook
    provider.after_request(&processed_response).await?;

    Ok(processed_response)
}

async fn send_provider_request(
    method: http::Method,
    url: String,
    headers: HeaderMap,
    body: Body,
    config: Arc<AppConfig>,
) -> Result<Response<Body>, AppError> {
    debug!("Preparing to send request: {} {}", method, url);
    
    let body_bytes = body::to_bytes(body, usize::MAX).await?;
    debug!("Request body size: {} bytes", body_bytes.len());

    let client = &*CLIENT;
    let method = Method::from_bytes(method.as_str().as_bytes())
        .map_err(|_| AppError::InvalidMethod)?;

    // Pre-allocate headers map with known capacity
    let mut reqwest_headers = reqwest::header::HeaderMap::with_capacity(headers.len());
    
    // Batch process headers
    for (name, value) in headers.iter() {
        if let (Ok(name_str), Ok(v)) = (
            name.as_str().parse::<reqwest::header::HeaderName>(),
            reqwest::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            reqwest_headers.insert(name_str, v);
        }
    }

    let response = client
        .request(method, url)
        .headers(reqwest_headers)
        .body(body_bytes.to_vec())
        .send()
        .await?;

    process_response(response, config).await
}

async fn process_response(
    response: reqwest::Response,
    config: Arc<AppConfig>,
) -> Result<Response<Body>, AppError> {
    let status = StatusCode::from_u16(response.status().as_u16())?;
    debug!("Processing response with status: {}", status);
    
    // Pre-allocate headers map
    let mut response_headers = HeaderMap::with_capacity(response.headers().len());
    
    // Batch process headers
    for (name, value) in response.headers() {
        if let (Ok(header_name), Ok(v)) = (
            http::HeaderName::from_bytes(name.as_ref()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            response_headers.insert(header_name, v);
        }
    }

    // Check for streaming response
    if response.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map_or(false, |ct| ct.contains("text/event-stream"))
    {
        info!("Processing streaming response");
        debug!("Setting up stream with buffer size: {}", config.buffer_size);
        
        let stream = response.bytes_stream().map(move |result| {
            match result {
                Ok(bytes) => {
                    let mut buffer = BytesMut::with_capacity(config.buffer_size);
                    buffer.extend_from_slice(&bytes);
                    Ok(buffer.freeze())
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                }
            }
        });

        let mut response_builder = Response::builder()
            .status(status)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive");

        // Add all headers from response_headers
        for (key, value) in response_headers {
            if let Some(key) = key {
                response_builder = response_builder.header(key, value);
            }
        }

        Ok(response_builder
            .body(Body::from_stream(stream))
            .unwrap())
    } else {
        debug!("Processing regular response");
        
        let body = response.bytes().await?;
        
        let mut response_builder = Response::builder().status(status);
        
        // Add all headers from response_headers
        for (key, value) in response_headers {
            if let Some(key) = key {
                response_builder = response_builder.header(key, value);
            }
        }

        Ok(response_builder
            .body(Body::from(body))
            .unwrap())
    }
}
