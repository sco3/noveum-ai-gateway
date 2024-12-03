use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct ProviderMetrics {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub cost: Option<f64>,
    pub model: String,
    pub provider_latency: Duration,
}

pub trait MetricsExtractor: Send + Sync {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics;
    fn extract_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics>;
}

// OpenAI-compatible metrics extractor
pub struct OpenAIMetricsExtractor;

impl MetricsExtractor for OpenAIMetricsExtractor {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics {
        let mut metrics = ProviderMetrics::default();
        
        if let Some(usage) = response_body.get("usage") {
            metrics.input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
        }

        if let Some(model) = response_body.get("model").and_then(|v| v.as_str()) {
            metrics.model = model.to_string();
        }

        metrics
    }

    fn extract_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        if !chunk.contains("[DONE]") {
            return None;
        }

        // Parse the last chunk for completion metrics
        if let Ok(json) = serde_json::from_str::<Value>(chunk) {
            return Some(self.extract_metrics(&json));
        }
        None
    }
}

// Anthropic-specific metrics extractor
pub struct AnthropicMetricsExtractor;

impl MetricsExtractor for AnthropicMetricsExtractor {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics {
        let mut metrics = ProviderMetrics::default();
        
        if let Some(usage) = response_body.get("usage") {
            metrics.input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.total_tokens = Some(
                metrics.input_tokens.unwrap_or(0) + metrics.output_tokens.unwrap_or(0)
            );
        }

        if let Some(model) = response_body.get("model").and_then(|v| v.as_str()) {
            metrics.model = model.to_string();
        }

        metrics
    }

    fn extract_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        None // Anthropic handles metrics differently in streaming mode
    }
}

// Factory function to get the appropriate metrics extractor
pub fn get_metrics_extractor(provider: &str) -> Box<dyn MetricsExtractor> {
    match provider {
        "anthropic" => Box::new(AnthropicMetricsExtractor),
        "bedrock" => Box::new(OpenAIMetricsExtractor), // AWS Bedrock uses OpenAI-compatible format
        "groq" => Box::new(OpenAIMetricsExtractor),    // GROQ uses OpenAI-compatible format
        "fireworks" => Box::new(OpenAIMetricsExtractor),
        "together" => Box::new(OpenAIMetricsExtractor),
        _ => Box::new(OpenAIMetricsExtractor),         // Default to OpenAI format
    }
} 