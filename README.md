# AI Gateway Proxy

A high-performance API gateway for routing requests to various AI providers (OpenAI, Anthropic) with streaming support.

## Features

- Unified API interface for multiple AI providers
- Streaming support for real-time responses
- Built-in health checking
- Configurable CORS
- Provider-specific request routing
- Efficient request/response proxying

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/ai-gateway
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
  -H "Authorization: Bearer your-api-key" \
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
  -H "x-api-key: your-api-key" \
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
```

## Architecture

The gateway is built using:

- **Axum** - Web framework
- **Tokio** - Async runtime
- **Tower-HTTP** - HTTP middleware
- **Reqwest** - HTTP client
- **Tracing** - Logging and diagnostics

## Performance Considerations

The gateway is designed to handle high-throughput scenarios with minimal overhead:

- Asynchronous I/O with Tokio
- Connection pooling via Reqwest
- Efficient streaming response handling
- Memory-efficient request/response proxying

## Security Notes

- Always run behind a reverse proxy in production
- Configure CORS appropriately for your use case
- Use environment variables for sensitive configuration
- Consider adding rate limiting for production use

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

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

For support, please open an issue in the GitHub repository.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
