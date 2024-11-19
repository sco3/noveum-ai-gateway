use super::Provider;
use crate::error::AppError;
use async_trait::async_trait;
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, HeaderValue, Response, StatusCode},
};
use serde_json::{json, Value};
use tracing::{debug, error, warn};
use futures_util::StreamExt;
use std::io::Read;
use aws_event_stream_parser::{parse_message, Message};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct BedrockProvider {
    base_url: String,
    region: String,
    buffer: Vec<u8>,
    current_model: Arc<RwLock<String>>,
}

impl BedrockProvider {
    pub fn new() -> Self {
        let region = "us-east-1".to_string();
        debug!("Initializing BedrockProvider with region: {}", region);
        Self {
            base_url: format!("https://bedrock-runtime.{}.amazonaws.com", region),
            region,
            buffer: Vec::new(),
            current_model: Arc::new(RwLock::new("amazon.titan-text-premier-v1:0".to_string())),
        }
    }

    fn get_model_name(&self, path: &str) -> String {
        if let Some(model) = path.split('/').last() {
            model.to_string()
        } else {
            "amazon.titan-embed-text-v1".to_string()
        }
    }

    fn transform_request_body(&self, body: Value) -> Result<Value, AppError> {
        debug!("Transforming request body: {:#?}", body);
        
        // If the body is already in the correct format, return it as is
        if body.get("inferenceConfig").is_some() {
            return Ok(body);
        }

        let messages = body["messages"]
            .as_array()
            .ok_or_else(|| {
                error!("Invalid request format: messages array not found");
                AppError::InvalidRequestFormat
            })?;

        let transformed_messages = messages.iter().map(|msg| {
            let content = msg["content"].as_str().unwrap_or_default();
            json!({
                "role": msg["role"].as_str().unwrap_or("user"),
                "content": [
                    {
                        "text": content
                    }
                ]
            })
        }).collect::<Vec<_>>();

        let transformed = json!({
            "messages": transformed_messages,
            "inferenceConfig": {
                "maxTokens": body["max_tokens"].as_u64().unwrap_or(1000),
                "temperature": body["temperature"].as_f64().unwrap_or(0.7),
                "topP": body["top_p"].as_f64().unwrap_or(1.0)
            }
        });

        debug!("Transformed body: {:#?}", transformed);
        Ok(transformed)
    }
}

#[async_trait]
impl Provider for BedrockProvider {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn name(&self) -> &str {
        "bedrock"
    }

    async fn before_request(&self, headers: &HeaderMap, body: &Bytes) -> Result<(), AppError> {
        // Extract and set the model from the request body before any other processing
        if let Ok(request_body) = serde_json::from_slice::<Value>(body) {
            if let Some(model) = request_body["model"].as_str() {
                debug!("Setting model from before_request: {}", model);
                *self.current_model.write() = model.to_string();
            }
        }
        Ok(())
    }

    fn transform_path(&self, path: &str) -> String {
        let model = self.current_model.read();
        debug!("Transforming path with model: {}", *model);
        format!("/model/{}/converse-stream", *model)
    }

    async fn prepare_request_body(&self, body: Bytes) -> Result<Bytes, AppError> {
        let request_body: Value = serde_json::from_slice(&body)?;
        let transformed_body = self.transform_request_body(request_body)?;
        Ok(Bytes::from(serde_json::to_vec(&transformed_body)?))
    }

    fn process_headers(&self, headers: &HeaderMap) -> Result<HeaderMap, AppError> {
        let mut final_headers = HeaderMap::new();
        
        // Add standard headers
        final_headers.insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("application/json"),
        );

        // Preserve AWS specific headers
        for (key, value) in headers {
            if key.as_str().starts_with("x-aws-") {
                final_headers.insert(key.clone(), value.clone());
            }
        }

        Ok(final_headers)
    }

    fn requires_signing(&self) -> bool {
        true
    }

    fn get_signing_credentials(&self, headers: &HeaderMap) -> Option<(String, String, String)> {
        let access_key = headers.get("x-aws-access-key-id")?.to_str().ok()?;
        let secret_key = headers.get("x-aws-secret-access-key")?.to_str().ok()?;
        let region = headers
            .get("x-aws-region")
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&self.region);
        
        Some((
            access_key.to_string(),
            secret_key.to_string(),
            region.to_string()
        ))
    }

    fn get_signing_host(&self) -> String {
        format!("bedrock-runtime.{}.amazonaws.com", self.region)
    }

    async fn process_response(&self, response: Response<Body>) -> Result<Response<Body>, AppError> {
        if response.headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map_or(false, |ct| ct.contains("application/vnd.amazon.eventstream"))
        {
            debug!("Processing Bedrock event stream response");
            
            // Create transformed stream
            let provider = self.clone();
            let stream = response
                .into_body()
                .into_data_stream()
                .map(move |chunk| {
                    match chunk {
                        Ok(bytes) => {
                            match provider.transform_bedrock_chunk(bytes) {
                                Ok(transformed) => Ok(transformed),
                                Err(e) => {
                                    error!("Error transforming chunk: {}", e);
                                    Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                                }
                            }
                        }
                        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                    }
                });

            // Build response with transformed stream and all necessary headers
            Ok(Response::builder()
                .status(StatusCode::OK)
                // SSE specific headers
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .header("connection", "keep-alive")
                .header("transfer-encoding", "chunked")
                // CORS headers
                .header("access-control-allow-origin", "*")
                .header("access-control-allow-methods", "POST, OPTIONS")
                .header("access-control-allow-headers", "content-type, x-provider, x-aws-access-key-id, x-aws-secret-access-key, x-aws-region")
                .header("access-control-expose-headers", "*")
                // SSE specific headers for better client compatibility
                .header("x-accel-buffering", "no")
                .header("keep-alive", "timeout=600")
                .body(Body::from_stream(stream))
                .unwrap())
        } else {
            // For non-streaming responses, still add CORS headers
            let mut response = response;
            let headers = response.headers_mut();
            headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
            headers.insert("access-control-allow-methods", HeaderValue::from_static("POST, OPTIONS"));
            headers.insert("access-control-allow-headers", 
                HeaderValue::from_static("content-type, x-provider, x-aws-access-key-id, x-aws-secret-access-key, x-aws-region"));
            headers.insert("access-control-expose-headers", HeaderValue::from_static("*"));
            Ok(response)
        }
    }
}

impl BedrockProvider {
    fn transform_bedrock_chunk(&self, chunk: Bytes) -> Result<Bytes, AppError> {
        debug!("Received chunk of size: {}", chunk.len());
        let mut remaining = chunk.as_ref();
        let mut response_events = Vec::new();

        while !remaining.is_empty() {
            match parse_message(remaining) {
                Ok((rest, message)) => {
                    debug!("Parsed message: event_type={:?}", 
                        message.headers.headers.iter()
                            .find(|h| h.key == ":event-type")
                            .map(|h| &h.value));
                    
                    remaining = rest;

                    let event_type = message.headers.headers.iter()
                        .find(|h| h.key == ":event-type")
                        .and_then(|h| match &h.value {
                            aws_event_stream_parser::HeaderValue::String(s) => Some(s.as_str()),
                            _ => None
                        })
                        .unwrap_or_default();

                    match event_type {
                        "contentBlockDelta" => {
                            if let Ok(body_str) = String::from_utf8(message.body.to_vec()) {
                                if let Ok(json) = serde_json::from_str::<Value>(&body_str) {
                                    if let Some(delta) = json.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                                        let openai_format = json!({
                                            "id": "chatcmpl-bedrock",
                                            "object": "chat.completion.chunk",
                                            "created": chrono::Utc::now().timestamp(),
                                            "model": "bedrock",
                                            "choices": [{
                                                "index": 0,
                                                "delta": {
                                                    "content": delta
                                                },
                                                "finish_reason": null
                                            }]
                                        });

                                        response_events.push(format!("data: {}\n\n", openai_format.to_string()));
                                    }
                                }
                            }
                        },
                        "metadata" => {
                            if let Ok(body_str) = String::from_utf8(message.body.to_vec()) {
                                if let Ok(json) = serde_json::from_str::<Value>(&body_str) {
                                    if let Some(usage) = json.get("usage") {
                                        let final_message = json!({
                                            "id": "chatcmpl-bedrock",
                                            "object": "chat.completion.chunk",
                                            "created": chrono::Utc::now().timestamp(),
                                            "model": "bedrock",
                                            "choices": [{
                                                "index": 0,
                                                "delta": {},
                                                "finish_reason": "stop"
                                            }],
                                            "usage": usage
                                        });
                                        response_events.push(format!("data: {}\ndata: [DONE]\n\n", 
                                            final_message.to_string()));
                                    }
                                }
                            }
                        },
                        _ => {
                            debug!("Skipping event type: {}", event_type);
                        }
                    }

                    if !message.valid() {
                        warn!("Invalid message checksum");
                    }
                }
                Err(e) => {
                    debug!("Failed to parse message: {:?}", e);
                    break;
                }
            }
        }

        // Join all events and return as a single chunk
        Ok(Bytes::from(response_events.join("")))
    }
} 