use crate::telemetry::{RequestMetrics, metrics::MetricsExporter};
use async_trait::async_trait;
use std::error::Error;
use serde_json::to_string_pretty;

pub struct ConsolePlugin;

impl ConsolePlugin {
    pub fn new() -> Self {
        ConsolePlugin
    }
}

#[async_trait]
impl MetricsExporter for ConsolePlugin {
    async fn export_metrics(&self, metrics: RequestMetrics) -> Result<(), Box<dyn Error>> {
        // For console output, we have two options:
        // 1. Print the raw RequestMetrics struct
        // 2. Print the OpenTelemetry formatted log
        
        // Option 1: Raw metrics (easier to read in console, but not OTel format)
        println!("Raw Request Metrics: {:#?}", metrics);
        
        // Option 2: OpenTelemetry format (less readable in console but consistent with exporters)
        let otel_log = metrics.to_otel_log();
        println!("OpenTelemetry Log: {}", to_string_pretty(&otel_log).unwrap_or_default());
        
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}
