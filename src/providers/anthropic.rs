use super::Provider;
use crate::error::AppError;
use crate::telemetry::provider_metrics::{MetricsExtractor, ProviderMetrics};
use async_trait::async_trait;
use axum::http::HeaderMap;
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error};

pub struct AnthropicProvider {
    base_url: String,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.anthropic.com".to_string(),
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn base_url(&self) -> String {
        self.base_url.clone()
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn transform_path(&self, path: &str) -> String {
        if path.contains("/chat/completions") {
            "/v1/messages".to_string()
        } else {
            path.to_string()
        }
    }

    fn process_headers(&self, original_headers: &HeaderMap) -> Result<HeaderMap, AppError> {
        debug!("Processing Anthropic request headers");
        let mut headers = HeaderMap::new();

        // Add content type
        headers.insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("application/json"),
        );

        // Add Anthropic version header
        headers.insert(
            http::header::HeaderName::from_static("anthropic-version"),
            http::header::HeaderValue::from_static("2023-06-01"),
        );

        // Process authentication
        if let Some(auth) = original_headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
        {
            debug!("Converting Bearer token to x-api-key format");
            let api_key = auth.trim_start_matches("Bearer ");
            headers.insert(
                http::header::HeaderName::from_static("x-api-key"),
                http::header::HeaderValue::from_str(api_key).map_err(|_| {
                    error!("Failed to process Anthropic authorization header");
                    AppError::InvalidHeader
                })?,
            );
        } else {
            error!("No authorization header found for Anthropic request");
            return Err(AppError::MissingApiKey);
        }

        Ok(headers)
    }
}

// Anthropic-specific metrics extractor
pub struct AnthropicMetricsExtractor;

impl MetricsExtractor for AnthropicMetricsExtractor {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics {
        debug!("Extracting Anthropic metrics from response: {}", response_body);
        let mut metrics = ProviderMetrics::default();
        
        // Extract usage data
        if let Some(usage) = response_body.get("usage") {
            metrics.input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            
            // Calculate total tokens if both input and output tokens are available
            if let (Some(input), Some(output)) = (metrics.input_tokens, metrics.output_tokens) {
                metrics.total_tokens = Some(input + output);
            }
            // If only one is available, use that as the total
            else if metrics.input_tokens.is_some() {
                metrics.total_tokens = metrics.input_tokens;
            } 
            else if metrics.output_tokens.is_some() {
                metrics.total_tokens = metrics.output_tokens;
            }
        }

        // Extract model information
        if let Some(model) = response_body.get("model").and_then(|v| v.as_str()) {
            metrics.model = model.to_string();
            
            // Calculate cost if we have token information
            if let Some(total_tokens) = metrics.total_tokens {
                metrics.cost = Some(calculate_anthropic_cost(&metrics.model, total_tokens));
            }
        }

        // Extract message ID to use as request ID
        if let Some(id) = response_body.get("id").and_then(|v| v.as_str()) {
            debug!("Found Anthropic message ID: {}", id);
            metrics.request_id = Some(id.to_string());
        } else if let Some(message) = response_body.get("message") {
            if let Some(id) = message.get("id").and_then(|v| v.as_str()) {
                debug!("Found Anthropic message ID in message object: {}", id);
                metrics.request_id = Some(id.to_string());
            }
        }

        debug!("Final extracted Anthropic metrics: {:?}", metrics);
        metrics
    }
    
    fn try_extract_provider_specific_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        debug!("Attempting to extract metrics from Anthropic streaming chunk: {}", chunk);
        
        // Try to parse the chunk as JSON
        if let Ok(json) = serde_json::from_str::<Value>(chunk) {
            // Check for message_start event which contains token information
            if let Some(event_type) = json.get("type").and_then(|t| t.as_str()) {
                match event_type {
                    "message_start" => {
                        if let Some(message) = json.get("message") {
                            let mut metrics = ProviderMetrics::default();
                            
                            // Extract token information from usage field
                            if let Some(usage) = message.get("usage") {
                                metrics.input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                                metrics.output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                                
                                // Calculate total tokens
                                if let (Some(input), Some(output)) = (metrics.input_tokens, metrics.output_tokens) {
                                    metrics.total_tokens = Some(input + output);
                                } else if metrics.input_tokens.is_some() {
                                    metrics.total_tokens = metrics.input_tokens;
                                } else if metrics.output_tokens.is_some() {
                                    metrics.total_tokens = metrics.output_tokens;
                                }
                            }
                            
                            // Extract model information
                            if let Some(model) = message.get("model").and_then(|v| v.as_str()) {
                                metrics.model = model.to_string();
                                
                                // Calculate cost if we have token information
                                if let Some(total_tokens) = metrics.total_tokens {
                                    metrics.cost = Some(calculate_anthropic_cost(&metrics.model, total_tokens));
                                }
                            }
                            
                            // Extract message ID to use as request ID
                            if let Some(id) = message.get("id").and_then(|v| v.as_str()) {
                                debug!("Found Anthropic message ID in streaming chunk: {}", id);
                                metrics.request_id = Some(id.to_string());
                            }
                            
                            debug!("Extracted Anthropic metrics from message_start event: {:?}", metrics);
                            return Some(metrics);
                        }
                    },
                    "message_delta" => {
                        // For message_delta events, capture usage updates if available
                        if let Some(usage) = json.get("usage") {
                            let mut metrics = ProviderMetrics::default();
                            
                            metrics.input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                            metrics.output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                            
                            // Calculate total tokens
                            if let (Some(input), Some(output)) = (metrics.input_tokens, metrics.output_tokens) {
                                metrics.total_tokens = Some(input + output);
                            } else if metrics.input_tokens.is_some() {
                                metrics.total_tokens = metrics.input_tokens;
                            } else if metrics.output_tokens.is_some() {
                                metrics.total_tokens = metrics.output_tokens;
                            }
                            
                            // We might not have model information in deltas, use a placeholder
                            metrics.model = "claude".to_string();
                            
                            debug!("Extracted Anthropic metrics from message_delta event: {:?}", metrics);
                            return Some(metrics);
                        }
                    },
                    "message_stop" => {
                        // For message_stop events, look for final usage stats
                        if let Some(usage) = json.get("usage") {
                            let mut metrics = ProviderMetrics::default();
                            
                            metrics.input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                            metrics.output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                            
                            // Calculate total tokens
                            if let (Some(input), Some(output)) = (metrics.input_tokens, metrics.output_tokens) {
                                metrics.total_tokens = Some(input + output);
                            } else if metrics.input_tokens.is_some() {
                                metrics.total_tokens = metrics.input_tokens;
                            } else if metrics.output_tokens.is_some() {
                                metrics.total_tokens = metrics.output_tokens;
                            }
                            
                            // We might not have model information in stop events, use a placeholder
                            metrics.model = "claude".to_string();
                            
                            debug!("Extracted Anthropic metrics from message_stop event: {:?}", metrics);
                            return Some(metrics);
                        }
                    },
                    _ => {
                        // For other event types, just try to extract any useful information
                        let mut metrics = ProviderMetrics::default();
                        
                        // Extract model information if available
                        if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
                            metrics.model = model.to_string();
                        } else {
                            metrics.model = "claude".to_string();
                        }
                        
                        debug!("Created partial Anthropic metrics for event type {}: {:?}", event_type, metrics);
                        return Some(metrics);
                    }
                }
            }
        }
        
        // If we couldn't parse as JSON or find relevant data, fallback to generic event detection
        if chunk.contains("content_block") || chunk.contains("message_start") || 
           chunk.contains("message_delta") || chunk.contains("message_stop") {
            // This appears to be an Anthropic event, create minimal metrics
            debug!("Detected Anthropic streaming event without parseable metrics");
            return Some(ProviderMetrics {
                model: "claude".to_string(),
                provider_latency: Duration::from_millis(0),
                ..Default::default()
            });
        }
        
        debug!("No metrics found in Anthropic streaming chunk");
        None
    }
}

// Helper function for Anthropic-specific cost calculation
fn calculate_anthropic_cost(model: &str, total_tokens: u32) -> f64 {
    let tokens = total_tokens as f64;
    
    match model {
        // Claude 3.5 models
        m if m.contains("claude-3.5-sonnet") => tokens * 0.000003, // $3.00 per million tokens
        
        // Claude 3 models
        m if m.contains("claude-3-opus") => tokens * 0.000015, // $15.00 per million tokens
        m if m.contains("claude-3-sonnet") => tokens * 0.000003, // $3.00 per million tokens
        m if m.contains("claude-3-haiku") => tokens * 0.000000125, // $0.25 per million input, $1.25 per million output, using average
        
        // Claude 2 models
        m if m.contains("claude-2") => tokens * 0.000008, // $8.00 per million tokens
        
        // Claude Instant models
        m if m.contains("claude-instant") => tokens * 0.000001, // $1.00 per million tokens
        
        // Generic fallbacks by model family
        m if m.contains("claude-3") => tokens * 0.000003, // Use Sonnet pricing as default for Claude 3
        m if m.contains("claude") => tokens * 0.000002, // Use a conservative estimate for unknown Claude models
        
        // Default case
        _ => {
            debug!("Unknown Anthropic model for cost calculation: {}", model);
            tokens * 0.000002 // Conservative default
        },
    }
}
