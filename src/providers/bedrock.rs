use super::Provider;
use crate::error::AppError;
use async_trait::async_trait;
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Response, StatusCode},
};
use serde_json::{json, Value};
use tracing::{debug, error, warn};
use futures_util::StreamExt;
use std::io::Read;
use aws_event_stream_parser::{parse_message, Message};

#[derive(Clone)]
pub struct BedrockProvider {
    base_url: String,
    region: String,
    buffer: Vec<u8>,
}

impl BedrockProvider {
    pub fn new() -> Self {
        let region = "us-east-1".to_string();
        debug!("Initializing BedrockProvider with region: {}", region);
        Self {
            base_url: format!("https://bedrock-runtime.{}.amazonaws.com", region),
            region,
            buffer: Vec::new(),
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

    async fn prepare_request_body(&self, body: Bytes) -> Result<Bytes, AppError> {
        let request_body: Value = serde_json::from_slice(&body)?;
        let model = request_body["model"]
            .as_str()
            .unwrap_or("amazon.titan-text-premier-v1:0")
            .to_string();
            
        let transformed_body = self.transform_request_body(request_body)?;
        debug!("Using model from body: {}", model);
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

    fn transform_path(&self, path: &str) -> String {
        debug!("Transforming path: {}", path);
        
        let model = if path.contains("chat/completions") {
            "amazon.titan-text-premier-v1:0"
        } else if let Some(model) = path.split('/').last() {
            model
        } else {
            "amazon.titan-text-premier-v1:0"
        };
        
        debug!("Using model for path: {}", model);
        format!("/model/{}/converse-stream", model)
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

            // Build response with transformed stream
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/event-stream")
                .header("cache-control", "no-cache")
                .header("connection", "keep-alive")
                .body(Body::from_stream(stream))
                .unwrap())
        } else {
            Ok(response)
        }
    }
}

impl BedrockProvider {
    fn transform_bedrock_chunk(&self, chunk: Bytes) -> Result<Bytes, AppError> {
        debug!("Received chunk of size: {}", chunk.len());
        let mut response_events = Vec::new();
        let mut remaining = chunk.as_ref();

        while !remaining.is_empty() {
            match parse_message(remaining) {
                Ok((rest, message)) => {
                    debug!("Parsed message: event_type={:?}", 
                        message.headers.headers.iter()
                            .find(|h| h.key == ":event-type")
                            .map(|h| &h.value));
                    
                    // Update remaining bytes for next iteration
                    remaining = rest;

                    // Get event type and content type from headers
                    let event_type = message.headers.headers.iter()
                        .find(|h| h.key == ":event-type")  // Note the colon prefix
                        .and_then(|h| match &h.value {
                            aws_event_stream_parser::HeaderValue::String(s) => Some(s.as_str()),
                            _ => None
                        })
                        .unwrap_or_default();

                    // Process the message based on event type
                    match event_type {
                        "contentBlockDelta" => {
                            if let Ok(body_str) = String::from_utf8(message.body.to_vec()) {
                                if let Ok(json) = serde_json::from_str::<Value>(&body_str) {
                                    if let Some(delta) = json.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
                                        debug!("Found delta text: {}", delta);
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

                                        let formatted = format!("data: {}\n\n", openai_format.to_string());
                                        response_events.push(Bytes::from(formatted));
                                    }
                                }
                            }
                        },
                        // "messageStop" | "contentBlockStop" => {
                        //     if let Ok(body_str) = String::from_utf8(message.body.to_vec()) {
                        //         if let Ok(json) = serde_json::from_str::<Value>(&body_str) {
                        //             if json.get("stopReason").is_some() {
                        //                 debug!("Found stop reason");
                        //                 response_events.push(Bytes::from("data: [DONE]\n\n"));
                        //             }
                        //         }
                        //     }
                        // },
                        "metadata" => {
                            if let Ok(body_str) = String::from_utf8(message.body.to_vec()) {
                                if let Ok(json) = serde_json::from_str::<Value>(&body_str) {
                                    debug!("Processing metadata: {}", json);
                                    // Create final message with usage information
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
                                        let formatted = format!("data: {}\n\n", final_message.to_string());
                                        response_events.push(Bytes::from(formatted));
                                        response_events.push(Bytes::from("data: [DONE]\n\n"));
                                    }
                                }
                            }
                        },
                        _ => {
                            debug!("Skipping event type: {}", event_type);
                        }
                    }

                    // Validate message checksum
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

        if response_events.is_empty() {
            debug!("No events processed, returning empty response");
            Ok(Bytes::new())
        } else {
            debug!("Returning {} processed events", response_events.len());
            Ok(Bytes::from(response_events.concat()))
        }
    }
} 