use super::TelemetryPlugin;
use crate::telemetry::RequestMetrics;
use crate::telemetry::metrics::MetricsExporter;
use async_trait::async_trait;
use elasticsearch::{
    auth::Credentials,
    http::transport::{Transport, TransportBuilder, SingleNodeConnectionPool},
    Elasticsearch, IndexParts,
};
use std::error::Error;
use tracing::{debug, error};

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
}

#[async_trait]
impl TelemetryPlugin for ElasticsearchPlugin {
    async fn export(&self, metrics: &RequestMetrics) -> Result<(), Box<dyn Error>> {
        // Convert metrics to OpenTelemetry format
        let document = metrics.to_otel_log();

        debug!("Sending metrics to Elasticsearch index: {}", self.index);
        
        let response = self.client
            .index(IndexParts::Index(&self.index))
            .body(document)
            .send()
            .await?;

        if !response.status_code().is_success() {
            error!(
                "Failed to index metrics: status={}, response={:?}",
                response.status_code(),
                response.text().await?
            );
            return Err("Failed to index metrics".into());
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