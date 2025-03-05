use serde_json::Value;
use std::time::Duration;
use tracing::debug;

/// Represents metrics collected from an LLM provider response
///
/// This struct contains all the relevant metrics data extracted from provider responses,
/// including token counts, cost calculations, model information, and latency.
/// All token fields are optional to handle cases where providers don't include this data.
#[derive(Debug, Clone, Default)]
pub struct ProviderMetrics {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub cost: Option<f64>,
    pub model: String,
    pub provider_latency: Duration,
}

impl ProviderMetrics {
    /// Estimates token count from raw text content
    ///
    /// This uses a simple heuristic of approximately 4 characters per token for English text.
    /// More precise estimation would require provider-specific tokenizers.
    ///
    /// # Arguments
    /// * `text` - The text to estimate token count for
    ///
    /// # Returns
    /// Estimated token count as u32
    pub fn estimate_tokens_from_text(text: &str) -> u32 {
        if text.is_empty() {
            return 0;
        }
        
        // Rough token estimation based on text length
        // Approximation: ~4 characters per token for English text
        // More accurate estimation would require tokenizer-specific logic
        ((text.len() as f64) / 4.0).ceil() as u32
    }
    
    /// Creates partial metrics with available information
    ///
    /// This is useful for streaming responses where complete token metrics
    /// are not available, but we can estimate output tokens from accumulated text.
    ///
    /// # Arguments
    /// * `model` - The model name string
    /// * `accumulated_text` - Text received so far to estimate tokens
    ///
    /// # Returns
    /// A ProviderMetrics instance with partial information
    pub fn create_partial_metrics(model: String, accumulated_text: &str) -> Self {
        let output_tokens = if !accumulated_text.is_empty() {
            Some(Self::estimate_tokens_from_text(accumulated_text))
        } else {
            None
        };
        
        ProviderMetrics {
            model,
            provider_latency: Duration::from_millis(0),
            output_tokens,
            // We don't have input tokens or total tokens
            ..Default::default()
        }
    }
}

/// The MetricsExtractor trait defines a layered approach to extracting metrics from provider responses
///
/// This architecture provides several key benefits:
/// 1. **Layered Extraction**: Tries provider-specific extraction first, falls back to common patterns
/// 2. **Extensibility**: New providers can be added by implementing just the parts they need to customize
/// 3. **Safety Net**: Common patterns catch metrics when provider-specific logic fails
/// 4. **Maintainability**: Clear separation between provider-specific and common logic
///
/// ## How to add a new provider:
/// 1. Create a new struct for your provider: `struct MyProviderMetricsExtractor;`
/// 2. Implement the required `extract_metrics` method
/// 3. Optionally override `try_extract_provider_specific_streaming_metrics` if your provider 
///    needs special handling for streaming responses
/// 4. Update the `get_metrics_extractor` factory function to return your implementation
///
/// The default implementation will handle common patterns automatically.
pub trait MetricsExtractor: Send + Sync {
    /// Extract complete metrics from a full response body
    ///
    /// This is the primary method each provider must implement to extract 
    /// metrics from a complete response.
    ///
    /// # Arguments
    /// * `response_body` - The JSON response body from the provider
    ///
    /// # Returns
    /// A ProviderMetrics instance with extracted data
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics;
    
    /// Extract metrics from streaming chunks
    ///
    /// This method implements a layered approach:
    /// 1. First tries provider-specific extraction
    /// 2. Falls back to common extraction patterns if that returns None
    ///
    /// This layered approach allows new providers to work out-of-the-box
    /// while letting existing providers maintain their specialized logic.
    ///
    /// # Arguments
    /// * `chunk` - A string containing a streaming chunk from the provider
    ///
    /// # Returns
    /// Option<ProviderMetrics> with metrics if they could be extracted
    fn extract_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        // First try provider-specific detection based on known patterns
        if let Some(metrics) = self.try_extract_provider_specific_streaming_metrics(chunk) {
            return Some(metrics);
        }
        
        // Then fall back to common extraction patterns as a safety net
        self.try_extract_common_streaming_metrics(chunk)
    }
    
    /// Provider-specific implementation for streaming metrics
    ///
    /// Override this method to implement custom extraction logic for a specific provider.
    /// The default implementation returns None, which causes the common extraction to be used.
    ///
    /// # Arguments
    /// * `chunk` - A string containing a streaming chunk from the provider
    ///
    /// # Returns
    /// Option<ProviderMetrics> with metrics if they could be extracted
    fn try_extract_provider_specific_streaming_metrics(&self, _chunk: &str) -> Option<ProviderMetrics> {
        None // Default is to skip provider-specific extraction
    }
    
    /// Generic implementation that works for most providers
    ///
    /// This provides a safety net for new providers or when specific extraction fails.
    /// It uses common patterns found across most LLM providers to identify and extract
    /// whatever metrics information is available.
    ///
    /// # Arguments
    /// * `chunk` - A string containing a streaming chunk from the provider
    ///
    /// # Returns
    /// Option<ProviderMetrics> with metrics if they could be extracted
    fn try_extract_common_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        debug!("Attempting common streaming metrics extraction for chunk");
        
        // Try to parse the chunk as JSON
        if let Ok(json) = serde_json::from_str::<Value>(chunk) {
            // If we have usage data, extract full metrics
            if json.get("usage").is_some() {
                debug!("Found usage in streaming chunk, extracting metrics");
                return Some(self.extract_metrics(&json));
            }
            
            // For any provider's streaming, extract what we can even if usage is missing
            let model = json.get("model").and_then(|m| m.as_str()).unwrap_or("unknown").to_string();
            
            // Check for general indicators that this is a model output
            let is_llm_response = 
                json.get("choices").is_some() || 
                json.get("completion").is_some() ||
                json.get("delta").is_some() ||
                json.get("finish_reason").is_some();
                
            if is_llm_response {
                debug!("LLM streaming response detected in common handler, creating partial metrics");
                return Some(ProviderMetrics {
                    model,
                    provider_latency: Duration::from_millis(0),
                    // Leave token counts and cost as None
                    ..Default::default()
                });
            }
        }
        
        debug!("No metrics data found in common streaming handler");
        None
    }
}

// OpenAI-specific metrics extractor
pub struct OpenAIMetricsExtractor;

impl MetricsExtractor for OpenAIMetricsExtractor {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics {
        debug!("Extracting OpenAI metrics from response: {}", response_body);
        let mut metrics = ProviderMetrics::default();
        
        if let Some(usage) = response_body.get("usage") {
            debug!("Found usage data: {:?}", usage);
            metrics.input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            debug!("Extracted tokens - input: {:?}, output: {:?}, total: {:?}", 
                metrics.input_tokens, metrics.output_tokens, metrics.total_tokens);
        }

        if let Some(model) = response_body.get("model").and_then(|v| v.as_str()) {
            debug!("Found model: {}", model);
            metrics.model = model.to_string();
        }

        if let (Some(total_tokens), Some(model)) = (metrics.total_tokens, response_body.get("model")) {
            metrics.cost = Some(calculate_cost(model.as_str().unwrap_or(""), total_tokens));
            debug!("Calculated cost: {:?} for model {} and {} tokens", 
                metrics.cost, metrics.model, total_tokens);
        }

        debug!("Final extracted metrics: {:?}", metrics);
        metrics
    }
    
    // Override with OpenAI-specific streaming metrics extraction
    fn try_extract_provider_specific_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        debug!("Attempting to extract metrics from OpenAI streaming chunk: {}", chunk);
        if let Ok(json) = serde_json::from_str::<Value>(chunk) {
            // If we have usage data, extract full metrics
            if json.get("usage").is_some() {
                debug!("Found usage in OpenAI streaming chunk, extracting metrics");
                return Some(self.extract_metrics(&json));
            }
            
            // For OpenAI streaming, extract what we can even if usage is missing
            // This will handle the common case where OpenAI omits token counts in streaming
            let model = json.get("model").and_then(|m| m.as_str()).unwrap_or("unknown").to_string();
            
            if model.contains("gpt") || json.get("object").and_then(|o| o.as_str()).unwrap_or("") == "chat.completion.chunk" {
                debug!("OpenAI streaming response detected without usage data, creating partial metrics");
                return Some(ProviderMetrics {
                    model,
                    provider_latency: Duration::from_millis(0), // We can't determine this from chunks
                    // Leave token counts and cost as None
                    ..Default::default()
                });
            }
        }
        debug!("No usage data found in OpenAI streaming chunk");
        None
    }
}

// Helper function to calculate cost based on model and tokens
fn calculate_cost(model: &str, total_tokens: u32) -> f64 {
    match model {
        m if m.contains("gpt-4") => (total_tokens as f64) * 0.00003,
        m if m.contains("gpt-3.5") => (total_tokens as f64) * 0.000002,
        _ => 0.0,
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
    
    // Anthropic doesn't need special streaming handling, use the default
    fn try_extract_provider_specific_streaming_metrics(&self, _chunk: &str) -> Option<ProviderMetrics> {
        None // Anthropic handles metrics differently in streaming mode
    }
}

// Add a new metrics extractor for Bedrock
pub struct BedrockMetricsExtractor;

impl MetricsExtractor for BedrockMetricsExtractor {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics {
        debug!("Extracting Bedrock metrics from response: {}", response_body);
        let mut metrics = ProviderMetrics::default();
        
        if let Some(usage) = response_body.get("usage") {
            debug!("Found Bedrock usage data: {:?}", usage);
            metrics.input_tokens = usage.get("inputTokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.output_tokens = usage.get("outputTokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            metrics.total_tokens = usage.get("totalTokens").and_then(|v| v.as_u64()).map(|v| v as u32);
            debug!("Extracted Bedrock tokens - input: {:?}, output: {:?}, total: {:?}", 
                metrics.input_tokens, metrics.output_tokens, metrics.total_tokens);
        }

        if let Some(model) = response_body.get("model").and_then(|v| v.as_str()) {
            debug!("Found Bedrock model: {}", model);
            metrics.model = model.to_string();
        }

        if let (Some(total_tokens), Some(model)) = (metrics.total_tokens, response_body.get("model")) {
            metrics.cost = Some(calculate_bedrock_cost(model.as_str().unwrap_or(""), total_tokens));
            debug!("Calculated Bedrock cost: {:?} for model {} and {} tokens", 
                metrics.cost, metrics.model, total_tokens);
        }

        debug!("Final extracted Bedrock metrics: {:?}", metrics);
        metrics
    }
    
    // Override with Bedrock-specific streaming metrics extraction
    fn try_extract_provider_specific_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        debug!("Attempting to extract metrics from Bedrock streaming chunk: {}", chunk);
        if let Ok(json) = serde_json::from_str::<Value>(chunk) {
            if json.get("usage").is_some() {
                debug!("Found usage in Bedrock streaming chunk");
                return Some(self.extract_metrics(&json));
            }
        }
        None
    }
}

// Helper function for Bedrock-specific cost calculation
fn calculate_bedrock_cost(model: &str, total_tokens: u32) -> f64 {
    match model {
        m if m.contains("claude") => (total_tokens as f64) * 0.00001102,
        m if m.contains("titan") => (total_tokens as f64) * 0.00001,
        m if m.contains("llama2") => (total_tokens as f64) * 0.00001,
        _ => 0.0,
    }
}

// Add Groq-specific metrics extractor
pub struct GroqMetricsExtractor;

impl MetricsExtractor for GroqMetricsExtractor {
    fn extract_metrics(&self, response_body: &Value) -> ProviderMetrics {
        debug!("Extracting Groq metrics from response: {}", response_body);
        let mut metrics = ProviderMetrics::default();
        
        // Try to get metrics from x_groq field first
        if let Some(x_groq) = response_body.get("x_groq") {
            if let Some(usage) = x_groq.get("usage") {
                debug!("Found Groq usage data in x_groq: {:?}", usage);
                metrics.input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                metrics.output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                metrics.total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                
                // Also capture provider latency from Groq's timing info
                if let Some(total_time) = usage.get("total_time").and_then(|v| v.as_f64()) {
                    metrics.provider_latency = Duration::from_secs_f64(total_time);
                }
            }
        }
        
        // If no metrics found in x_groq, try root level usage
        if metrics.total_tokens.is_none() {
            if let Some(usage) = response_body.get("usage") {
                debug!("Found Groq usage data at root level: {:?}", usage);
                metrics.input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                metrics.output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                metrics.total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                
                // Also capture provider latency
                if let Some(total_time) = usage.get("total_time").and_then(|v| v.as_f64()) {
                    metrics.provider_latency = Duration::from_secs_f64(total_time);
                }
            }
        }

        if let Some(model) = response_body.get("model").and_then(|v| v.as_str()) {
            debug!("Found Groq model: {}", model);
            metrics.model = model.to_string();
        }

        if let (Some(total_tokens), Some(model)) = (metrics.total_tokens, response_body.get("model")) {
            metrics.cost = Some(calculate_groq_cost(model.as_str().unwrap_or(""), total_tokens));
            debug!("Calculated Groq cost: {:?} for model {} and {} tokens", 
                metrics.cost, metrics.model, total_tokens);
        }

        debug!("Final extracted Groq metrics: {:?}", metrics);
        metrics
    }
    
    // Override with Groq-specific streaming metrics extraction
    fn try_extract_provider_specific_streaming_metrics(&self, chunk: &str) -> Option<ProviderMetrics> {
        debug!("Attempting to extract metrics from Groq streaming chunk");
        
        // First try parsing the chunk directly as JSON
        if let Ok(json) = serde_json::from_str::<Value>(chunk) {
            // Check if this is the final chunk with usage data
            if let Some(x_groq) = json.get("x_groq") {
                debug!("Found x_groq field in direct JSON: {}", json);
                if let Some(usage) = x_groq.get("usage") {
                    // Extract token counts from the usage data
                    let mut metrics = ProviderMetrics::default();
                    
                    metrics.input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                    metrics.output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                    metrics.total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                    
                    // Capture provider latency from Groq's timing info if available
                    if let Some(total_time) = usage.get("total_time").and_then(|v| v.as_f64()) {
                        metrics.provider_latency = Duration::from_secs_f64(total_time);
                    }
                    
                    // Get the model name
                    if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
                        metrics.model = model.to_string();
                    }
                    
                    // Calculate cost if we have total tokens and model
                    if let Some(total_tokens) = metrics.total_tokens {
                        metrics.cost = Some(calculate_groq_cost(&metrics.model, total_tokens));
                        debug!("Calculated Groq cost: {:?} for model {} and {} tokens", 
                            metrics.cost, metrics.model, total_tokens);
                    }
                    
                    debug!("Extracted complete Groq metrics from streaming chunk: {:?}", metrics);
                    return Some(metrics);
                }
            }
            
            // Check for final chunk with finish_reason: "stop"
            let is_final_chunk = json.get("choices")
                .and_then(|c| c.as_array())
                .and_then(|choices| choices.first())
                .and_then(|choice| choice.get("finish_reason"))
                .and_then(|f| f.as_str())
                .map(|reason| reason == "stop")
                .unwrap_or(false);
                
            // Extract model information for any chunk
            let model = json.get("model").and_then(|v| v.as_str()).unwrap_or("llama").to_string();
            
            // Check if this is a Groq response
            let is_groq_response = 
                model.contains("llama") || 
                model.contains("gemma") || 
                json.get("object").and_then(|o| o.as_str()).map(|obj| obj == "chat.completion.chunk").unwrap_or(false);
                
            if is_groq_response {
                debug!("Groq streaming chunk detected for model: {}, is_final: {}", model, is_final_chunk);
                
                // Create metrics with available information
                return Some(ProviderMetrics {
                    model,
                    provider_latency: Duration::from_millis(0),
                    // Token counts and cost will be None until the final chunk with usage info
                    ..Default::default()
                });
            }
        }
        
        // Handle SSE format: split the chunk into lines and process each line
        for line in chunk.lines() {
            if !line.starts_with("data: ") {
                continue;
            }

            let json_str = line.trim_start_matches("data: ");
            if json_str == "[DONE]" {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                // Check if this is the final chunk with x_groq usage data
                if let Some(x_groq) = json.get("x_groq") {
                    debug!("Found x_groq field in SSE data: {}", json_str);
                    if let Some(usage) = x_groq.get("usage") {
                        // This is the final chunk with complete metrics
                        let mut metrics = ProviderMetrics::default();
                        
                        metrics.input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                        metrics.output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                        metrics.total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
                        
                        // Capture provider latency if available
                        if let Some(total_time) = usage.get("total_time").and_then(|v| v.as_f64()) {
                            metrics.provider_latency = Duration::from_secs_f64(total_time);
                        }
                        
                        // Get the model name
                        if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
                            metrics.model = model.to_string();
                        }
                        
                        // Calculate cost if we have total tokens and model
                        if let Some(total_tokens) = metrics.total_tokens {
                            metrics.cost = Some(calculate_groq_cost(&metrics.model, total_tokens));
                        }
                        
                        debug!("Extracted complete Groq metrics from SSE streaming chunk: {:?}", metrics);
                        return Some(metrics);
                    }
                }
                
                // Extract model information for partial metrics
                let model = json.get("model").and_then(|v| v.as_str()).unwrap_or("llama").to_string();
                
                // Check if this is a Groq response
                let is_groq_response = 
                    model.contains("llama") || 
                    model.contains("gemma") || 
                    json.get("object").and_then(|o| o.as_str()).map(|obj| obj == "chat.completion.chunk").unwrap_or(false);
                    
                if is_groq_response {
                    // Create partial metrics with model information
                    debug!("Groq SSE streaming chunk detected for model: {}", model);
                    return Some(ProviderMetrics {
                        model,
                        provider_latency: Duration::from_millis(0),
                        // Token counts and cost will be filled in the final chunk
                        ..Default::default()
                    });
                }
            }
        }
        
        debug!("No usage data found in Groq streaming chunk");
        None
    }
}

// Helper function for Groq-specific cost calculation
fn calculate_groq_cost(model: &str, total_tokens: u32) -> f64 {
    let tokens = total_tokens as f64;
    
    match model {
        // Llama 3 models
        m if m.contains("llama-3") && m.contains("70b") => tokens * 0.0009,
        m if m.contains("llama-3") && m.contains("8b") => tokens * 0.0001,
        m if m.contains("llama-3.1") && m.contains("70b") => tokens * 0.0009,
        m if m.contains("llama-3.1") && m.contains("8b") => tokens * 0.0001,
        
        // Legacy Llama 2 models
        m if m.contains("llama-2") && m.contains("70b") => tokens * 0.0007,
        m if m.contains("llama-2") && m.contains("13b") => tokens * 0.0002,
        m if m.contains("llama-2") && m.contains("7b") => tokens * 0.0001,
        
        // Mixtral models
        m if m.contains("mixtral-8x7b") => tokens * 0.0002,
        m if m.contains("mixtral-8x22b") => tokens * 0.0006,
        
        // Gemma models
        m if m.contains("gemma") && m.contains("7b") => tokens * 0.0001,
        m if m.contains("gemma") && m.contains("27b") => tokens * 0.0004,
        
        // Generic fallbacks by model family
        m if m.contains("mixtral") => tokens * 0.0002,
        m if m.contains("llama") => tokens * 0.0001,
        m if m.contains("gemma") => tokens * 0.0001,
        
        // Default case - apply minimal cost to avoid zero cost which might mislead
        _ => {
            debug!("Unknown Groq model for cost calculation: {}", model);
            tokens * 0.0001
        },
    }
}

/// Factory function to get the appropriate metrics extractor for a provider
///
/// This function maps provider names to their specific MetricsExtractor implementations.
/// For new providers without a specific implementation, the OpenAIMetricsExtractor is used
/// as a fallback, which provides reasonable extraction using common patterns.
///
/// # Arguments
/// * `provider` - The provider name as a string
///
/// # Returns
/// A boxed dyn MetricsExtractor trait object for the specified provider
pub fn get_metrics_extractor(provider: &str) -> Box<dyn MetricsExtractor> {
    match provider {
        "anthropic" => Box::new(AnthropicMetricsExtractor),
        "bedrock" => Box::new(BedrockMetricsExtractor),
        "groq" => Box::new(GroqMetricsExtractor),    // Use Groq-specific extractor
        "fireworks" => Box::new(OpenAIMetricsExtractor),
        "together" => Box::new(OpenAIMetricsExtractor),
        _ => Box::new(OpenAIMetricsExtractor),
    }
} 