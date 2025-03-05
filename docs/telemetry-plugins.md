# Telemetry Plugins Guide

This guide explains how to create and integrate new telemetry exporters into the MagicAPI Gateway.

## Overview

The MagicAPI Gateway uses a plugin-based architecture for telemetry exporters, allowing easy integration of new exporters like Elasticsearch, DataDog, New Relic, etc.

## Creating a New Exporter Plugin

### 1. Create a New Exporter Module

Create a new file in `src/telemetry/exporters/elasticsearch.rs`:

```rust
use async_trait::async_trait;
use opentelemetry::trace::TraceError;
use serde_json::json;
use std::sync::Arc;
use reqwest::Client;
use tracing::info;

use crate::telemetry::{TelemetryConfig, TelemetryExporter};

pub struct ElasticsearchExporter {
    client: Client,
    url: String,
    index: String,
}

impl ElasticsearchExporter {
    pub fn new(url: String, index: String) -> Self {
        Self {
            client: Client::new(),
            url,
            index,
        }
    }

    async fn send_metrics(&self, metrics: serde_json::Value) -> Result<(), TraceError> {
        let url = format!("{}/{}/metrics", self.url, self.index);
        
        self.client
            .post(&url)
            .json(&metrics)
            .send()
            .await
            .map_err(|e| TraceError::from(e.to_string()))?;
            
        Ok(())
    }
}

#[async_trait]
impl TelemetryExporter for ElasticsearchExporter {
    fn name(&self) -> &str {
        "elasticsearch"
    }

    async fn init(&self, config: &TelemetryConfig) -> Result<(), TraceError> {
        info!("Initializing Elasticsearch exporter");
        
        // Test connection
        let health_url = format!("{}/_cluster/health", self.url);
        self.client
            .get(&health_url)
            .send()
            .await
            .map_err(|e| TraceError::from(e.to_string()))?;
            
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), TraceError> {
        info!("Shutting down Elasticsearch exporter");
        Ok(())
    }
}
```

### 2. Create a Plugin Implementation

Create a new file in `src/telemetry/plugins/elasticsearch.rs`:

```rust
use inventory::submit;
use crate::telemetry::{plugins::TelemetryPlugin, TelemetryExporter};
use crate::telemetry::exporters::elasticsearch::ElasticsearchExporter;

pub struct ElasticsearchPlugin;

#[async_trait::async_trait]
impl TelemetryPlugin for ElasticsearchPlugin {
    fn name(&self) -> &str {
        "elasticsearch"
    }

    fn create_exporter(&self) -> Box<dyn TelemetryExporter> {
        let url = std::env::var("ELASTICSEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:9200".to_string());
        let index = std::env::var("ELASTICSEARCH_INDEX")
            .unwrap_or_else(|_| "magicapi-metrics".to_string());
            
        Box::new(ElasticsearchExporter::new(url, index))
    }
}

// Register the plugin
inventory::submit! {
    Box::new(ElasticsearchPlugin) as Box<dyn TelemetryPlugin>
}
```

### 3. Update Module Declarations

Add to `src/telemetry/exporters/mod.rs`:
```rust
pub mod elasticsearch;
pub use elasticsearch::ElasticsearchExporter;
```

## Configuration

### Environment Variables

```bash
# Enable Elasticsearch exporter
ENABLED_EXPORTERS=prometheus,elasticsearch

# Elasticsearch configuration
ELASTICSEARCH_URL=http://localhost:9200
ELASTICSEARCH_INDEX=magicapi-metrics
```

### Docker Compose Example

```yaml
version: '3.8'
services:
  gateway:
    image: magicapi1/magicapi-ai-gateway:latest
    environment:
      - ENABLED_EXPORTERS=prometheus,elasticsearch
      - ELASTICSEARCH_URL=http://elasticsearch:9200
      - ELASTICSEARCH_INDEX=magicapi-metrics
    ports:
      - "3000:3000"
    depends_on:
      - elasticsearch

  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.12.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    ports:
      - "9200:9200"
```

## Custom Metrics

You can extend the metrics collection by adding new metrics to `GatewayMetrics`:

```rust
// In src/telemetry/metrics.rs
pub struct GatewayMetrics {
    // Existing metrics...
    
    // Add custom metrics
    pub custom_metric: Counter<u64>,
    pub custom_histogram: Histogram<f64>,
}

impl GatewayMetrics {
    pub fn new(meter: Meter) -> Self {
        Self {
            // Existing metrics...
            
            custom_metric: meter
                .u64_counter("custom_metric")
                .with_description("Description of custom metric")
                .with_unit(Unit::new("units"))
                .init(),
                
            custom_histogram: meter
                .f64_histogram("custom_histogram")
                .with_description("Description of custom histogram")
                .with_unit(Unit::new("units"))
                .init(),
        }
    }
}
```

## Best Practices

1. **Error Handling**
   - Implement proper error handling and logging
   - Use appropriate backoff strategies for failed exports
   - Consider circuit breakers for external services

2. **Performance**
   - Batch metrics when possible
   - Use appropriate buffer sizes
   - Implement rate limiting if needed

3. **Security**
   - Support TLS/SSL connections
   - Implement authentication
   - Sanitize sensitive data

4. **Configuration**
   - Make all parameters configurable
   - Provide sensible defaults
   - Document all configuration options

## Example: Adding DataDog Exporter

Here's a quick example of adding a DataDog exporter:

```rust
// src/telemetry/exporters/datadog.rs
pub struct DatadogExporter {
    api_key: String,
    site: String,
    client: Client,
}

#[async_trait]
impl TelemetryExporter for DatadogExporter {
    fn name(&self) -> &str {
        "datadog"
    }

    async fn init(&self, config: &TelemetryConfig) -> Result<(), TraceError> {
        // Initialize DataDog connection
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), TraceError> {
        // Cleanup
        Ok(())
    }
}

// src/telemetry/plugins/datadog.rs
pub struct DatadogPlugin;

#[async_trait]
impl TelemetryPlugin for DatadogPlugin {
    fn name(&self) -> &str {
        "datadog"
    }

    fn create_exporter(&self) -> Box<dyn TelemetryExporter> {
        let api_key = std::env::var("DD_API_KEY").expect("DD_API_KEY must be set");
        let site = std::env::var("DD_SITE").unwrap_or_else(|_| "datadoghq.com".to_string());
        
        Box::new(DatadogExporter::new(api_key, site))
    }
}

inventory::submit! {
    Box::new(DatadogPlugin) as Box<dyn TelemetryPlugin>
}
```

## Testing Plugins

Create tests for your exporter:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_elasticsearch_exporter() {
        let exporter = ElasticsearchExporter::new(
            "http://localhost:9200".to_string(),
            "test-metrics".to_string(),
        );
        
        let config = TelemetryConfig::default();
        assert!(exporter.init(&config).await.is_ok());
        // Add more tests...
    }
}
```

## Troubleshooting

Common issues and solutions:

1. **Connection Issues**
   ```bash
   # Check connectivity
   curl -X GET "localhost:9200/_cluster/health"
   ```

2. **Authentication Errors**
   - Verify credentials
   - Check environment variables
   - Ensure proper permissions

3. **Performance Issues**
   - Monitor memory usage
   - Check batch sizes
   - Verify network latency

## Resources

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Elasticsearch API Reference](https://www.elastic.co/guide/en/elasticsearch/reference/current/index.html)
- [DataDog API Documentation](https://docs.datadoghq.com/api/)

## OpenTelemetry Compatible Log Format

MagicAPI Gateway now supports OpenTelemetry compatible logs for all telemetry plugins. This structured format provides consistent logging across different exporters and makes it easier to integrate with observability platforms.

### Log Structure

```json
{
  "timestamp": "2025-03-05T16:03:20.123Z",
  "resource": {
    "service.name": "noveum_ai_gateway",
    "service.version": "1.0.0",
    "deployment.environment": "production"
  },
  "name": "ai_gateway_request_log",
  "attributes": {
    "id": "msg_29",
    "thread_id": "thread_29",
    "org_id": "org_123",
    "user_id": "user_456",
    "project_id": "proj_design",
    "provider": "azure",
    "model": "gpt-4-turbo",
    "request": { /* Complete request object */ },
    "response": { /* Complete response object */ },
    "metadata": {
      "project_id": "proj_design",
      "project_name": "UX Design",
      "latency": 6250,
      "tokens": { "input": 48, "output": 865, "total": 913 },
      "cost": 0.0456,
      "status": "success",
      "path": "/v1/chat/completions",
      "method": "POST",
      "request_size": 193,
      "response_size": 52280,
      "provider_latency": 255,
      "status_code": 200,
      "provider_status_code": 0,
      "error_count": 0,
      "error_type": null,
      "provider_error_count": 0,
      "provider_error_type": null,
      "provider_request_id": "req_01jnkrrz2ken1bej7emqf9j2af"
    }
  }
}
```

### Streaming Response Format

For streaming responses, the gateway captures both the final accumulated response and all individual streaming chunks. This provides complete visibility into the streaming process without impacting performance:

```json
{
  "timestamp": "2025-03-05T16:25:30.789Z",
  "resource": {
    "service.name": "noveum_ai_gateway",
    "service.version": "1.0.0",
    "deployment.environment": "production"
  },
  "name": "ai_gateway_request_log",
  "attributes": {
    "id": "msg_31",
    "thread_id": "thread_31",
    "org_id": "org_123",
    "user_id": "user_456",
    "project_id": "proj_chat",
    "provider": "groq",
    "model": "llama-3.1-8b-instant",
    "request": {
      "model": "llama-3.1-8b-instant",
      "messages": [
        {
          "role": "user",
          "content": "Write a poem"
        }
      ],
      "stream": true,
      "max_tokens": 500
    },
    "response": {
      "id": "chatcmpl-ec855684-8495-420d-8807-9259228ac717",
      "model": "llama-3.1-8b-instant",
      "choices": [
        {
          "delta": {
            "role": "assistant",
            "content": "\"Moonlit Dreams\"\n\nThe night is dark..."
          }
        }
      ],
      "streamed_data": [
        {
          "nonce": "0955",
          "id": "chatcmpl-ec855684-8495-420d-8807-9259228ac717",
          "object": "chat.completion.chunk",
          "choices": [
            {
              "index": 0,
              "delta": {
                "role": "assistant",
                "content": ""
              },
              "finish_reason": null
            }
          ]
        },
        // Additional chunks...
        {
          "nonce": "d4fe",
          "choices": [
            {
              "index": 0,
              "delta": {},
              "finish_reason": "stop"
            }
          ],
          "x_groq": {
            "usage": {
              "prompt_tokens": 38,
              "completion_tokens": 256,
              "total_tokens": 294
            }
          }
        }
      ]
    },
    "metadata": {
      "latency": 1244,
      "tokens": { "input": 38, "output": 256, "total": 294 },
      "cost": 0.0263,
      "status": "success"
    }
  }
}
```

### Working with the Log Format

When creating new telemetry plugins, you should use the `to_otel_log()` method on the `RequestMetrics` struct to convert metrics to this format:

```rust
async fn export(&self, metrics: &RequestMetrics) -> Result<(), Box<dyn Error>> {
    // Convert metrics to OpenTelemetry format
    let document = metrics.to_otel_log();
    
    // Export the document to your telemetry system
    // ...
}
```

For streaming responses, the gateway automatically:
1. Captures each streaming chunk as it's received
2. Stores all chunks in the `streamed_data` array
3. Includes both the final response and all chunks in the log

This approach ensures complete visibility into streaming responses without impacting performance, as chunks are collected asynchronously during normal processing.

### Custom Values from Request Headers

The gateway automatically extracts and includes these values from request headers:

| Header Name | Field in Log |
|-------------|--------------|
| x-project-id | attributes.project_id |
| x-organisation-id | attributes.org_id |
| x-user-id | attributes.user_id |
| x-experiment-id | experiment_id (for internal use) |

## Example: Creating a PostgreSQL Exporter

Here's an example of creating a PostgreSQL exporter using the OpenTelemetry format:

```rust
use super::TelemetryPlugin;
use crate::telemetry::RequestMetrics;
use crate::telemetry::metrics::MetricsExporter;
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::error::Error;
use tracing::{debug, error};

pub struct PostgresPlugin {
    pool: Pool<Postgres>,
    table_name: String,
}

impl PostgresPlugin {
    pub async fn new(
        connection_string: String, 
        table_name: String
    ) -> Result<Self, Box<dyn Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?;
            
        // Ensure table exists
        sqlx::query(&format!("
            CREATE TABLE IF NOT EXISTS {} (
                id SERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                log_data JSONB NOT NULL
            )
        ", table_name))
            .execute(&pool)
            .await?;
            
        Ok(Self {
            pool,
            table_name,
        })
    }
}

#[async_trait]
impl TelemetryPlugin for PostgresPlugin {
    async fn export(&self, metrics: &RequestMetrics) -> Result<(), Box<dyn Error>> {
        // Convert metrics to OpenTelemetry format
        let document = metrics.to_otel_log();
        let json_data = serde_json::to_value(&document)?;
        
        debug!("Sending metrics to PostgreSQL table: {}", self.table_name);
        
        // Insert into database
        let result = sqlx::query(&format!(
            "INSERT INTO {} (timestamp, log_data) VALUES ($1, $2)",
            self.table_name
        ))
        .bind(chrono::Utc::now())
        .bind(json_data)
        .execute(&self.pool)
        .await;
        
        if let Err(e) = result {
            error!("Failed to insert metrics into PostgreSQL: {}", e);
            return Err(e.into());
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        "postgres"
    }
}

#[async_trait]
impl MetricsExporter for PostgresPlugin {
    async fn export_metrics(&self, metrics: RequestMetrics) -> Result<(), Box<dyn Error>> {
        self.export(&metrics).await
    }

    fn name(&self) -> &str {
        "postgres"
    }
} 