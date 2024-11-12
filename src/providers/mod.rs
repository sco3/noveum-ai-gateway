use async_trait::async_trait;
use axum::{http::{Request, Response, HeaderMap}, body::Body};
use crate::error::AppError;
use tracing::error;

#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the base URL for the provider's API
    fn base_url(&self) -> &str;
    
    /// Get the provider's name for logging and identification
    fn name(&self) -> &str;
    
    /// Transform the request path if needed (e.g., for API version compatibility)
    fn transform_path(&self, path: &str) -> String {
        path.to_string()
    }
    
    /// Process and validate headers before sending request
    fn process_headers(&self, headers: &HeaderMap) -> Result<HeaderMap, AppError>;
    
    /// Process response before returning to client
    async fn process_response(&self, response: Response<Body>) -> Result<Response<Body>, AppError> {
        Ok(response)
    }
    
    /// Hook called before processing request
    async fn before_request(&self, _request: &Request<Body>) -> Result<(), AppError> {
        Ok(())
    }
    
    /// Hook called after processing request
    async fn after_request(&self, _response: &Response<Body>) -> Result<(), AppError> {
        Ok(())
    }
}

mod openai;
mod anthropic;
mod groq;

pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;
pub use groq::GroqProvider;

/// Factory function to create provider instances
pub fn create_provider(provider_name: &str) -> Result<Box<dyn Provider>, AppError> {
    match provider_name.to_lowercase().as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new())),
        "anthropic" => Ok(Box::new(AnthropicProvider::new())),
        "groq" => Ok(Box::new(GroqProvider::new())),
        unknown => {
            error!("Attempted to use unsupported provider: {}", unknown);
            Err(AppError::UnsupportedProvider)
        }
    }
} 