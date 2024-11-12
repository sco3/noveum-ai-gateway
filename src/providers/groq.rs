use async_trait::async_trait;
use axum::http::HeaderMap;
use tracing::{debug, error};
use crate::error::AppError;
use super::Provider;

pub struct GroqProvider {
    base_url: String,
}

impl GroqProvider {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.groq.com/openai".to_string(),
        }
    }
}

#[async_trait]
impl Provider for GroqProvider {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn name(&self) -> &str {
        "groq"
    }

    fn process_headers(&self, original_headers: &HeaderMap) -> Result<HeaderMap, AppError> {
        debug!("Processing Groq request headers");
        let mut headers = HeaderMap::new();
        
        // Add content type
        headers.insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("application/json"),
        );

        // Process authentication
        if let Some(auth) = original_headers.get("authorization")
            .and_then(|h| h.to_str().ok()) 
        {
            debug!("Using provided authorization header for Groq");
            headers.insert(
                http::header::AUTHORIZATION,
                http::header::HeaderValue::from_str(auth)
                    .map_err(|_| {
                        error!("Failed to process Groq authorization header");
                        AppError::InvalidHeader
                    })?
            );
        } else {
            error!("No authorization header found for Groq request");
            return Err(AppError::MissingApiKey);
        }

        Ok(headers)
    }
} 