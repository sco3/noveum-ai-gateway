use super::TelemetryPlugin;
use crate::telemetry::RequestMetrics;
use crate::telemetry::metrics::MetricsExporter;
use async_trait::async_trait;
use elasticsearch::{
    auth::Credentials,
    http::transport::{Transport, TransportBuilder, SingleNodeConnectionPool},
    Elasticsearch, IndexParts,
};
use opentelemetry::trace::TraceError;
use std::error::Error;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, warn};
use tokio_retry::{
    strategy::{ExponentialBackoff, jitter},
    Retry,
};

pub struct ElasticsearchPlugin {
    client: Elasticsearch,
    index: String,
}

impl ElasticsearchPlugin {
    pub fn new(url: String, username: Option<String>, password: Option<String>, index: String) -> Result<Self, Box<dyn Error>> {
        let transport = match (username, password) {
            (Some(u), Some(p)) => {
                let credentials = Credentials::Basic(u, p);
                let conn_pool = SingleNodeConnectionPool::new(url.parse()?);
                let transport_builder = TransportBuilder::new(conn_pool)
                    .auth(credentials);
                transport_builder.build()?
            }
            _ => {
                let conn_pool = SingleNodeConnectionPool::new(url.parse()?);
                TransportBuilder::new(conn_pool).build()?
            }
        };

        Ok(Self {
            client: Elasticsearch::new(transport),
            index,
        })
    }

    // Enhanced send_metrics method with retries and better error handling
    async fn send_metrics(&self, document: serde_json::Value) -> Result<(), TraceError> {
        // Configure retry strategy with exponential backoff
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter) // Add jitter to prevent thundering herd
            .take(3);    // Max 3 retries
        
        debug!("Sending metrics to Elasticsearch index: {}", self.index);
        
        let index = self.index.clone();
        let client = self.client.clone();
        
        let result = Retry::spawn(retry_strategy, || async {
            match timeout(Duration::from_secs(10), client.index(IndexParts::Index(&index)).body(document.clone()).send()).await {
                Ok(response_result) => {
                    match response_result {
                        Ok(response) => {
                            if response.status_code().is_success() {
                                debug!("Successfully sent metrics to Elasticsearch");
                                Ok(())
                            } else {
                                let status = response.status_code();
                                let error_text = response.text().await.unwrap_or_else(|_| "Unable to get response text".to_string());
                                warn!("Elasticsearch returned non-success status: {}, response: {}", status, error_text);
                                
                                // Return error but retry for 5xx errors (server errors)
                                if status.as_u16() >= 500 && status.as_u16() < 600 {
                                    Err(TraceError::from(format!("Server error from Elasticsearch: {}", status)))
                                } else {
                                    // Don't retry for 4xx errors (client errors)
                                    Err(TraceError::from(format!("Client error from Elasticsearch: {}", status)))
                                }
                            }
                        },
                        Err(e) => {
                            warn!("Failed to send metrics to Elasticsearch: {}", e);
                            Err(TraceError::from(format!("Failed to send metrics: {}", e)))
                        }
                    }
                },
                Err(_) => {
                    warn!("Elasticsearch request timed out after 10 seconds");
                    Err(TraceError::from("Request to Elasticsearch timed out"))
                }
            }
        }).await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to send metrics after retries: {}", e);
                Err(TraceError::from(format!("Failed to send metrics after retries: {}", e)))
            }
        }
    }
}

#[async_trait]
impl TelemetryPlugin for ElasticsearchPlugin {
    async fn export(&self, metrics: &RequestMetrics) -> Result<(), Box<dyn Error>> {
        // Convert metrics to OpenTelemetry format
        let document = metrics.to_otel_log();
        
        if let Err(e) = self.send_metrics(document).await {
            error!("Failed to export metrics: {}", e);
            return Err(Box::new(e));
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        "elasticsearch"
    }
}

#[async_trait]
impl MetricsExporter for ElasticsearchPlugin {
    async fn export_metrics(&self, metrics: RequestMetrics) -> Result<(), Box<dyn Error>> {
        self.export(&metrics).await
    }

    fn name(&self) -> &str {
        "elasticsearch"
    }
} 