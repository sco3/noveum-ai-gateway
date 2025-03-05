pub mod exporters;
pub mod metrics;
pub mod plugins;
pub mod middleware;
pub mod provider_metrics;

pub use self::{
    metrics::MetricsRegistry,
    plugins::ConsolePlugin,
    exporters::prometheus::PrometheusExporter,
    middleware::metrics_middleware,
};

use serde::{Deserialize, Serialize};
use std::time::Duration;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    #[serde(rename = "service.name")]
    pub service_name: String,
    #[serde(rename = "service.version")]
    pub service_version: String,
    #[serde(rename = "deployment.environment")]
    pub deployment_environment: String,
}

impl Default for ResourceInfo {
    fn default() -> Self {
        Self {
            service_name: "noveum_ai_gateway".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            deployment_environment: std::env::var("DEPLOYMENT_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAttributes {
    // Basic identifying fields
    pub id: String,
    pub thread_id: String,
    pub org_id: Option<String>,    
    pub user_id: Option<String>,
    pub project_id: Option<String>,

    // Provider/model details
    pub provider: String,
    pub model: String,

    // Request/Response objects (can be stored as JSON Value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Value>,

    // Metadata
    pub metadata: LogMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetadata {
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub latency: u128,
    pub tokens: TokenInfo,
    pub cost: Option<f64>,
    pub status: String,
    pub path: String,
    pub method: String,
    pub request_size: usize,
    pub response_size: usize,
    pub provider_latency: u128,
    pub status_code: u16,
    pub provider_status_code: u16,
    pub error_count: u32,
    pub error_type: Option<String>,
    pub provider_error_count: u32,
    pub provider_error_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub input: Option<u32>,
    pub output: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    // Request metadata
    pub provider: String,
    pub model: String,
    pub path: String,
    pub method: String,
    
    // Timing metrics
    pub total_latency: Duration,
    pub provider_latency: Duration,
    
    // Size metrics
    pub request_size: usize,
    pub response_size: usize,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    
    // Status metrics
    pub status_code: u16,
    pub provider_status_code: u16,
    
    // Error metrics
    pub error_count: u32,
    pub error_type: Option<String>,
    pub provider_error_count: u32,
    pub provider_error_type: Option<String>,
    
    // Cost metrics
    pub cost: Option<f64>,
    
    // OpenTelemetry additional fields
    pub id: Option<String>,
    pub thread_id: Option<String>,
    pub org_id: Option<String>,
    pub user_id: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    
    // Original request and response
    pub request_body: Option<Value>,
    pub response_body: Option<Value>,
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            provider: String::new(),
            model: String::new(),
            path: String::new(),
            method: String::new(),
            total_latency: Duration::default(),
            provider_latency: Duration::default(),
            request_size: 0,
            response_size: 0,
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            status_code: 0,
            provider_status_code: 0,
            error_count: 0,
            error_type: None,
            provider_error_count: 0,
            provider_error_type: None,
            cost: None,
            id: None,
            thread_id: None,
            org_id: None,
            user_id: None,
            project_id: None,
            project_name: None,
            request_body: None,
            response_body: None,
        }
    }
}

impl RequestMetrics {
    /// Convert to OpenTelemetry compatible log format
    pub fn to_otel_log(&self) -> serde_json::Value {
        let status = if self.error_count > 0 || self.provider_error_count > 0 {
            "error"
        } else {
            "success"
        };
        
        let token_info = TokenInfo {
            input: self.input_tokens,
            output: self.output_tokens,
            total: self.total_tokens,
        };
        
        let metadata = LogMetadata {
            project_id: self.project_id.clone(),
            project_name: self.project_name.clone(),
            latency: self.total_latency.as_millis(),
            tokens: token_info,
            cost: self.cost,
            status: status.to_string(),
            path: self.path.clone(),
            method: self.method.clone(),
            request_size: self.request_size,
            response_size: self.response_size,
            provider_latency: self.provider_latency.as_millis(),
            status_code: self.status_code,
            provider_status_code: self.provider_status_code,
            error_count: self.error_count,
            error_type: self.error_type.clone(),
            provider_error_count: self.provider_error_count,
            provider_error_type: self.provider_error_type.clone(),
        };
        
        let attributes = LogAttributes {
            id: self.id.clone().unwrap_or_else(|| format!("msg_{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"))),
            thread_id: self.thread_id.clone().unwrap_or_else(|| format!("thread_{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"))),
            org_id: self.org_id.clone(),
            user_id: self.user_id.clone(),
            project_id: self.project_id.clone(),
            provider: self.provider.clone(),
            model: self.model.clone(),
            request: self.request_body.clone(),
            response: self.response_body.clone(),
            metadata,
        };
        
        let resource = ResourceInfo::default();
        
        json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "resource": resource,
            "name": "ai_gateway_request_log",
            "attributes": attributes
        })
    }
} 