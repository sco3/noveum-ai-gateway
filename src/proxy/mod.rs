use axum::{
    body::{self, Body, Bytes},
    http::{HeaderMap, HeaderValue, Request, Response, StatusCode},
};
use futures_util::StreamExt;
use reqwest::Method;
use std::sync::Arc;
use tracing::{debug, error, info};
use bytes::BytesMut;
use axum::body::to_bytes;
use crate::providers::Provider;

use crate::{config::AppConfig, error::AppError, providers::create_provider};

mod client;
pub use client::CLIENT;
mod signing;

pub async fn proxy_request_to_provider(
    config: Arc<AppConfig>,
    provider_name: &str,
    mut original_request: Request<Body>,
) -> Result<Response<Body>, AppError> {
    let provider = create_provider(provider_name)?;
    
    // Extract body bytes
    let body = std::mem::replace(original_request.body_mut(), Body::empty());
    let body_bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|e| AppError::AxumError(e.into()))?;

    // Call before_request first to set up any provider state
    provider.before_request(original_request.headers(), &body_bytes).await?;

    // Process headers and transform path
    let mut headers = provider.process_headers(original_request.headers())?;
    let path = original_request.uri().path();
    let modified_path = provider.transform_path(path);
    
    // Prepare request body
    let prepared_body = provider.prepare_request_body(body_bytes).await?;
    
    // Construct final URL
    let query = original_request
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let url = format!("{}{}{}", provider.base_url(), modified_path, query);
    debug!("Using URL: {}", url);

    // Handle AWS signing if required
    let final_headers = if provider.requires_signing() {
        if let Some((access_key, secret_key, region)) = provider.get_signing_credentials(&headers) {
            signing::sign_aws_request(
                original_request.method().as_str(),
                &url,
                &headers,
                &prepared_body,
                &access_key,
                &secret_key,
                &region,
                "bedrock",
            ).await?
        } else {
            headers
        }
    } else {
        headers
    };

    debug!("Final headers in proxy_request_to_provider: {:?}", final_headers);

    // Send the request with signed headers
    let response = send_provider_request(
        original_request.method().clone(),
        url,
        final_headers,
        prepared_body,
        &provider,
        config,
    ).await?;

    provider.process_response(response).await
}

pub async fn send_provider_request(
    method: Method,
    url: String,
    headers: HeaderMap,
    body: Bytes,
    provider: &Box<dyn Provider>,
    config: Arc<AppConfig>,
) -> Result<Response<Body>, AppError> {
    let client = &*CLIENT;
    
    let reqwest_headers = headers.iter().filter_map(|(name, value)| {
        name.as_str()
            .parse::<reqwest::header::HeaderName>()
            .ok()
            .and_then(|name_str| {
                reqwest::header::HeaderValue::from_bytes(value.as_bytes())
                    .ok()
                    .map(|v| (name_str, v))
            })
    }).collect::<reqwest::header::HeaderMap>();

    debug!("Final headers in send_provider_request: {:?}", reqwest_headers);

    let response = client
        .request(method, url)
        .headers(reqwest_headers)
        .body(body)
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
    
    let mut response_headers = HeaderMap::with_capacity(response.headers().len());
    
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
        
        // Create a stream that processes chunks immediately
        let stream = response
            .bytes_stream()
            .map(move |result| {
                match result {
                    Ok(bytes) => Ok(bytes),
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
            .header("connection", "keep-alive")
            .header("transfer-encoding", "chunked");

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
