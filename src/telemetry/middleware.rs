use super::metrics::MetricsRegistry;
use super::provider_metrics::{get_metrics_extractor, ProviderMetrics, MetricsExtractor};
use super::RequestMetrics;
use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{Request, Response},
    middleware::Next,
};
use futures_util::StreamExt;
use std::{sync::Arc, time::Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error};
use axum::body::to_bytes;
use hyper::body::HttpBody;
use hyper::Error;
use serde_json::Value;

pub async fn metrics_middleware(
    State(registry): State<Arc<MetricsRegistry>>,
    mut req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let start = Instant::now();

    // Extract provider and other request info
    let provider = req
        .headers()
        .get("x-provider")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("openai")
        .to_string();

    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    // Extract project_id, org_id, and user_id from headers
    let project_id = req
        .headers()
        .get("x-project-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
        
    let org_id = req
        .headers()
        .get("x-organisation-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
        
    let user_id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
        
    let experiment_id = req
        .headers()
        .get("x-experiment-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    debug!("Received request: provider={}, path={}, method={}", provider, path, method);

    // Get metrics extractor for this provider
    let metrics_extractor = get_metrics_extractor(&provider);

    // Store original values before consuming request
    let original_method = req.method().clone();
    let original_uri = req.uri().clone();
    let original_headers = req.headers().clone();
    
    // Extract and store the original request body
    let (req_size, req_body, body) = {
        let bytes = to_bytes(req.into_body(), usize::MAX).await.unwrap_or_default();
        let size = bytes.len();
        let req_body = serde_json::from_slice(&bytes).ok();
        debug!("Request body size: {} bytes", size);
        (size, req_body, Body::from(bytes))
    };

    // Reconstruct request with original values
    let mut new_req = Request::builder()
        .method(original_method)
        .uri(original_uri)
        .body(body)
        .unwrap();
    *new_req.headers_mut() = original_headers;

    // Process the response
    let response = next.run(new_req).await;

    let is_streaming = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/event-stream"))
        .unwrap_or(false);

    debug!("Response is streaming: {}", is_streaming);

    if is_streaming {
        handle_streaming_response(
            response,
            registry,
            provider,
            path,
            method,
            req_size,
            req_body,
            start,
            metrics_extractor,
            project_id,
            org_id,
            user_id,
            experiment_id,
        )
        .await
    } else {
        handle_regular_response(
            response,
            registry,
            provider,
            path,
            method,
            req_size,
            req_body,
            start,
            metrics_extractor,
            project_id,
            org_id,
            user_id,
            experiment_id,
        )
        .await
    }
}

async fn handle_regular_response(
    response: Response<Body>,
    registry: Arc<MetricsRegistry>,
    provider: String,
    path: String,
    method: String,
    req_size: usize,
    req_body: Option<Value>,
    start: Instant,
    metrics_extractor: Box<dyn MetricsExtractor>,
    project_id: Option<String>,
    org_id: Option<String>,
    user_id: Option<String>,
    experiment_id: Option<String>,
) -> Response<Body> {
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
    let resp_size = bytes.len();

    debug!("Regular response body size: {} bytes", resp_size);

    // Extract metrics from response body
    let (provider_metrics, resp_body) = if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
        (metrics_extractor.extract_metrics(&json), Some(json))
    } else {
        (ProviderMetrics::default(), None)
    };

    debug!("Extracted provider metrics: {:?}", provider_metrics);

    let metrics = RequestMetrics {
        provider,
        path,
        method,
        model: provider_metrics.model,
        total_latency: start.elapsed(),
        provider_latency: provider_metrics.provider_latency,
        request_size: req_size,
        response_size: resp_size,
        input_tokens: provider_metrics.input_tokens,
        output_tokens: provider_metrics.output_tokens,
        total_tokens: provider_metrics.total_tokens,
        status_code: parts.status.as_u16(),
        cost: provider_metrics.cost,
        project_id,
        org_id,
        user_id,
        request_body: req_body,
        response_body: resp_body,
        ..Default::default()
    };

    registry.record_metrics(metrics).await;

    Response::from_parts(parts, Body::from(bytes))
}

async fn handle_streaming_response(
    response: Response<Body>,
    registry: Arc<MetricsRegistry>,
    provider: String,
    path: String,
    method: String,
    req_size: usize,
    req_body: Option<Value>,
    start: Instant,
    metrics_extractor: Box<dyn MetricsExtractor>,
    project_id: Option<String>,
    org_id: Option<String>,
    user_id: Option<String>,
    experiment_id: Option<String>,
) -> Response<Body> {
    let (parts, body) = response.into_parts();
    let (tx, rx) = mpsc::channel::<Result<Bytes, Error>>(100);

    let metrics_registry = registry.clone();
    let mut accumulated_text = String::new();

    // Process the stream
    tokio::spawn(async move {
        let mut response_size = 0;
        let mut accumulated_metrics = ProviderMetrics::default();
        let mut final_metrics_found = false;
        let mut resp_body = None;

        let mut stream = body.into_data_stream();
        while let Some(chunk) = stream.next().await {
            if let Ok(bytes) = chunk {
                response_size += bytes.len();
                debug!("Streaming response chunk size: {} bytes", bytes.len());

                if let Ok(chunk_str) = String::from_utf8(bytes.to_vec()) {
                    accumulated_text.push_str(&chunk_str);
                    
                    // Try to extract metrics from this chunk
                    if let Some(metrics) = metrics_extractor.extract_streaming_metrics(&chunk_str) {
                        accumulated_metrics = metrics;
                        final_metrics_found = true;
                        debug!("Extracted metrics from streaming chunk: {:?}", accumulated_metrics);
                    }
                }

                let _ = tx.send(Ok(bytes)).await;
            }
        }

        // Try to parse the accumulated response
        if !accumulated_text.is_empty() {
            resp_body = serde_json::from_str(&accumulated_text).ok();
        }

        // Record final metrics if we found them
        if final_metrics_found {
            let metrics = RequestMetrics {
                provider,
                path,
                method,
                model: accumulated_metrics.model,
                total_latency: start.elapsed(),
                provider_latency: accumulated_metrics.provider_latency,
                request_size: req_size,
                response_size,
                input_tokens: accumulated_metrics.input_tokens,
                output_tokens: accumulated_metrics.output_tokens,
                total_tokens: accumulated_metrics.total_tokens,
                status_code: parts.status.as_u16(),
                cost: accumulated_metrics.cost,
                project_id,
                org_id,
                user_id,
                request_body: req_body,
                response_body: resp_body,
                ..Default::default()
            };

            debug!("Recording streaming metrics: {:?}", metrics);
            metrics_registry.record_metrics(metrics).await;
        } else {
            debug!("No final metrics found in streaming response. Total text accumulated: {} bytes", accumulated_text.len());
        }
    });

    Response::from_parts(parts, Body::from_stream(ReceiverStream::new(rx)))
}

async fn measure_body_size(body: Body) -> (usize, Body) {
    let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
    let size = bytes.len();
    (size, Body::from(bytes))
}
