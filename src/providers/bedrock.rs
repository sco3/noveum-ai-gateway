use super::Provider;
use crate::error::AppError;
use async_trait::async_trait;
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Response},
};
use serde_json::{json, Value};
use tracing::{debug, error};

pub struct BedrockProvider {
    base_url: String,
    region: String,
}

impl BedrockProvider {
    pub fn new() -> Self {
        let region = "us-east-1".to_string();
        debug!("Initializing BedrockProvider with region: {}", region);
        Self {
            base_url: format!("https://bedrock-runtime.{}.amazonaws.com", region),
            region,
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
} 