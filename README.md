# MagicAPI AI Gateway

The world's fastest AI Gateway proxy, written in Rust and optimized for maximum performance. This high-performance API gateway routes requests to various AI providers (OpenAI, Anthropic) with streaming support, making it perfect for developers who need reliable and blazing-fast AI API access.

[![Rust](https://github.com/MagicAPI/ai-gateway/actions/workflows/rust.yml/badge.svg)](https://github.com/MagicAPI/ai-gateway/actions/workflows/rust.yml)

## Features

- 🚀 Blazing fast performance - built in Rust with zero-cost abstractions
- ⚡ Optimized for low latency and high throughput
- 🔄 Unified API interface for multiple AI providers (OpenAI, Anthropic)
- 📡 Real-time streaming support with minimal overhead
- 🔍 Built-in health checking
- 🛡️ Configurable CORS
- 🔀 Smart provider-specific request routing
- 📊 Efficient request/response proxying
- 💪 Production-ready and battle-tested

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

#### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/magicapi/ai-gateway
cd ai-gateway
```

2. Build the project:
```bash
cargo build --release
```

3. Run the server:
```bash
cargo run --release
```

The server will start on `http://127.0.0.1:3000`.

## Usage

### Making Requests

To make requests through the gateway, use the `/v1/*` endpoint and specify the provider using the `x-provider` header.

#### Example: OpenAI Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: openai" \
  -H "Authorization: Bearer your-openai-api-key" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

#### Example: Anthropic Request

```bash
curl -X POST http://localhost:3000/v1/complete \
  -H "Content-Type: application/json" \
  -H "x-provider: anthropic" \
  -H "x-api-key: your-anthropic-api-key" \
  -d '{
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Streaming Responses

To enable streaming, add the `Accept: text/event-stream` header to your request.

## Configuration

The gateway can be configured using environment variables:

```bash
RUST_LOG=debug # Logging level (debug, info, warn, error)
OPENAI_API_KEY=your_key # OpenAI API key
```

## Performance

MagicAPI Developer AI Gateway is designed for maximum performance:

- **Zero-cost abstractions** using Rust's ownership model
- **Asynchronous I/O** with Tokio for optimal resource utilization
- **Connection pooling** via Reqwest for efficient HTTP connections
- **Memory-efficient** request/response proxying
- **Minimal overhead** in the request path
- **Optimized streaming** response handling

## Architecture

The gateway leverages the best-in-class Rust ecosystem:

- **Axum** - High-performance web framework
- **Tokio** - Industry-standard async runtime
- **Tower-HTTP** - Robust HTTP middleware
- **Reqwest** - Fast and reliable HTTP client
- **Tracing** - Zero-overhead logging and diagnostics

## Security Notes

- Always run behind a reverse proxy in production
- Configure CORS appropriately for your use case
- Use environment variables for sensitive configuration
- Consider adding rate limiting for production use

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch

# Run tests
cargo test

# Run with hot reload
cargo watch -x run
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Check if port 3000 is available
   - Verify the HOST and PORT settings

2. **Streaming Not Working**
   - Ensure `Accept: text/event-stream` header is set
   - Check client supports streaming
   - Verify provider supports streaming for the requested endpoint

3. **Provider Errors**
   - Verify provider API keys are correct
   - Check provider-specific headers are properly set
   - Ensure the provider endpoint exists and is correctly formatted

## Support

For support, please open an issue in the GitHub repository. Our community is active and happy to help!

## License

This project is dual-licensed under both the MIT License and the Apache License (Version 2.0). You may choose either license at your option. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) files for details.

## Acknowledgments

Special thanks to all our contributors and the Rust community for making this project possible.
