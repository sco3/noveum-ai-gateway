//! Main entry point for integration tests.
//! These tests need a running instance of the AI Gateway.
//! 
//! # Environment Setup
//! 
//! The tests expect the following environment variables to be set:
//! 
//! ```
//! # Gateway URL (default: http://localhost:3000)
//! GATEWAY_URL=http://localhost:3000
//! 
//! # ElasticSearch Configuration
//! ELASTICSEARCH_URL=http://localhost:9200
//! ELASTICSEARCH_USERNAME=elastic
//! ELASTICSEARCH_PASSWORD=your_password
//! ELASTICSEARCH_INDEX=ai-gateway-metrics
//! 
//! # Provider API Keys
//! OPENAI_API_KEY=your_openai_api_key
//! ANTHROPIC_API_KEY=your_anthropic_api_key
//! # Add other provider API keys as needed
//! ```
//!
//! # Running the tests
//!
//! ```bash
//! # Start the gateway in one terminal
//! cargo run
//!
//! # In another terminal, run the tests
//! cargo test --test run_integration_tests -- --nocapture
//! ```

mod integration; 