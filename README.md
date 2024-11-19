# MagicAPI AI Gateway

The world's fastest AI Gateway proxy, written in Rust and optimized for maximum performance. This high-performance API gateway routes requests to various AI providers (OpenAI, Anthropic, GROQ, Fireworks, Together) with streaming support, making it perfect for developers who need reliable and blazing-fast AI API access.

[![Rust](https://github.com/MagicAPI/ai-gateway/actions/workflows/rust.yml/badge.svg)](https://github.com/MagicAPI/ai-gateway/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/magicapi-ai-gateway.svg)](https://crates.io/crates/magicapi-ai-gateway)

## Features

- 🚀 Blazing fast performance - built in Rust with zero-cost abstractions
- ⚡ Optimized for low latency and high throughput
- 🔄 Unified API interface for multiple AI providers (OpenAI, Anthropic, GROQ, Fireworks, Together)
- 📡 Real-time streaming support with minimal overhead
- 🔍 Built-in health checking
- 🛡️ Configurable CORS
- 🔀 Smart provider-specific request routing
- 📊 Efficient request/response proxying
- 💪 Production-ready and battle-tested

## Quick Start

### Installation

You can install MagicAPI Gateway using one of these methods:

#### Using Cargo Install

```bash
cargo install magicapi-ai-gateway
```

After installation, you can start the gateway by running:
```bash
magicapi-ai-gateway
```

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

The server will start on `http://127.0.0.1:3000` by default.

### Running the Gateway

You can configure the gateway using environment variables:

```bash
# Basic configuration
export RUST_LOG=info

# Start the gateway
magicapi-ai-gateway

# Or with custom port
PORT=8080 magicapi-ai-gateway
```

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

#### Example: GROQ Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: groq" \
  -H "Authorization: Bearer your-groq-api-key" \
  -d '{
    "model": "llama2-70b-4096",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true,
    "max_tokens": 300
  }'
```

#### Example: Anthropic Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: anthropic" \
  -H "Authorization: Bearer your-anthropic-api-key" \
  -d '{
    "model": "claude-3-5-sonnet-20241022",
    "messages": [{"role": "user", "content": "Write a poem"}],
    "stream": true,
    "max_tokens": 1024
  }'
```

#### Example: Fireworks Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: fireworks" \
  -H "Authorization: Bearer your-fireworks-api-key" \
  -d '{
    "model": "accounts/fireworks/models/llama-v3p1-8b-instruct",
    "messages": [{"role": "user", "content": "Write a poem"}],
    "stream": true,
    "max_tokens": 300,
    "temperature": 0.6,
    "top_p": 1,
    "top_k": 40
  }'
```

#### Example: Together AI Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: together" \
  -H "Authorization: Bearer your-together-api-key" \
  -d '{
    "model": "meta-llama/Llama-2-7b-chat-hf",
    "messages": [{"role": "user", "content": "Write a poem"}],
    "stream": true,
    "max_tokens": 512,
    "temperature": 0.7,
    "top_p": 0.7,
    "top_k": 50,
    "repetition_penalty": 1
  }'
```

#### Example: AWS Bedrock Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: bedrock" \
  -H "x-aws-access-key-id: YOUR_AWS_ACCESS_KEY" \
  -H "x-aws-secret-access-key: YOUR_AWS_SECRET_KEY" \
  -H "x-aws-region: us-east-1" \
  -H "x-aws-session-token: YOUR_SESSION_TOKEN" \
  -d '{
    "model": "anthropic.claude-v2",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 1000,
    "temperature": 0.7,
    "top_p": 1,
    "top_k": 250
  }'
```

Note: When using Bedrock as the provider, the gateway automatically:
- Signs requests using AWS SigV4 authentication
- Transforms OpenAI-style requests to Bedrock format
- Handles AWS credentials and region configuration
- Supports Claude models through Bedrock

## SDK Compatibility

The MagicAPI AI Gateway is designed to work seamlessly with popular AI SDKs. You can use the official OpenAI SDK to interact with any supported provider by simply configuring the baseURL and adding the appropriate provider header.

### Using with OpenAI's Official Node.js SDK

```typescript
import OpenAI from 'openai';

// Configure the SDK to use MagicAPI Gateway
const openai = new OpenAI({
  apiKey: process.env.PROVIDER_API_KEY, // Use any provider's API key
  baseURL: "http://localhost:3000/v1/", // Point to the gateway
  defaultHeaders: {
    "x-provider": "groq", // Specify the provider you want to use
  },
});

// Make requests as usual
const chatCompletion = await openai.chat.completions.create({
  messages: [
    { role: "system", content: "Write a poem" },
    { role: "user", content: "" }
  ],
  model: "llama-3.1-8b-instant",
  temperature: 1,
  max_tokens: 100,
  top_p: 1,
  stream: false,
});

// For AWS Bedrock
const bedrockClient = new OpenAI({
  apiKey: 'not-needed', // AWS uses separate authentication
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { 
    "x-provider": "bedrock",
    "x-aws-access-key-id": process.env.AWS_ACCESS_KEY_ID,
    "x-aws-secret-access-key": process.env.AWS_SECRET_ACCESS_KEY,
    "x-aws-region": process.env.AWS_REGION || "us-east-1",
    // Optional: Include if using temporary credentials
    "x-aws-session-token": process.env.AWS_SESSION_TOKEN
  },
});

// Make requests as usual
const completion = await bedrockClient.chat.completions.create({
  model: "anthropic.claude-v2",
  messages: [{ role: "user", content: "Hello!" }],
  temperature: 0.7,
  max_tokens: 1000
});
```

You can easily switch between providers by changing the `x-provider` header and API key:

```typescript
// For OpenAI
const openaiClient = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "openai" },
});

// For Anthropic
const anthropicClient = new OpenAI({
  apiKey: process.env.ANTHROPIC_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "anthropic" },
});

// For GROQ
const groqClient = new OpenAI({
  apiKey: process.env.GROQ_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "groq" },
});

// For Fireworks
const fireworksClient = new OpenAI({
  apiKey: process.env.FIREWORKS_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "fireworks" },
});

// For Together AI
const togetherClient = new OpenAI({
  apiKey: process.env.TOGETHER_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "together" },
});
```

The gateway automatically handles the necessary transformations to ensure compatibility with each provider's API format while maintaining the familiar OpenAI SDK interface.


## Configuration

The gateway can be configured using environment variables:

```bash
RUST_LOG=debug # Logging level (debug, info, warn, error)
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

4. **AWS Bedrock Authentication Errors**
   - Verify AWS credentials are correct
   - Check if the region is properly configured
   - Ensure IAM permissions include Bedrock access
   - Verify session token if using temporary credentials

## Support

For support, please open an issue in the GitHub repository. Our community is active and happy to help!

## License

This project is dual-licensed under both the MIT License and the Apache License (Version 2.0). You may choose either license at your option. See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) files for details.

## Acknowledgments

Special thanks to all our contributors and the Rust community for making this project possible.

## Docker Support

### Building and Running with Docker

1. Build the Docker image:
```bash
docker buildx build --platform linux/amd64 -t magicapi1/magicapi-ai-gateway:latest . --load
```

2. Push the image to Docker Hub:
```bash
docker push magicapi1/magicapi-ai-gateway:latest
```

3. Run the container:
```bash
docker run -p 3000:3000 \
  -e RUST_LOG=info \
  magicapi1/magicapi-ai-gateway:latest
```

### Using Pre-built Docker Image

```bash
docker pull magicapi1/magicapi-ai-gateway:latest
docker run -p 3000:3000 \
  -e RUST_LOG=info \
  magicapi1/magicapi-ai-gateway:latest
```

### Docker Compose

#### Option 1: Build from Source

Create a `docker-compose.yml` file:

```yaml
version: '3.8'
services:
  gateway:
    build: .
    platform: linux/amd64
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    restart: unless-stopped
```

#### Option 2: Use Prebuilt Image

Create a `docker-compose.yml` file:

```yaml
version: '3.8'
services:
  gateway:
    image: magicapi1/magicapi-ai-gateway:latest
    platform: linux/amd64
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
    restart: unless-stopped
```

Then run either option with:
```bash
docker-compose up -d
```

## Release Process for magicapi-ai-gateway

### 1. Pre-release Checklist
- [ ] Update version number in `Cargo.toml`
- [ ] Update CHANGELOG.md (if you have one)
- [ ] Ensure all tests pass: `cargo test`
- [ ] Verify the crate builds locally: `cargo build --release`
- [ ] Run `cargo clippy` to check for any linting issues
- [ ] Run `cargo fmt` to ensure consistent formatting

### 2. Git Commands
```bash
# Create and switch to a release branch
git checkout -b release/v0.1.6

# Stage and commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v0.1.6"

# Create a git tag
git tag -a v0.1.7 -m "Release v0.1.7"

# Push changes and tag
git push origin release/v0.1.7
git push origin v0.1.7
```

### 3. Publishing to crates.io
```bash
# Verify the package contents
cargo package

# Publish to crates.io (requires authentication)
cargo publish
```

### 4. Post-release
1. Create a GitHub release (if using GitHub)
   - Go to Releases → Draft a new release
   - Choose the tag v0.1.7
   - Add release notes
   - Publish release

2. Merge the release branch back to main
```bash
git checkout main
git merge release/v0.1.7
git push origin main
```

### 5. Version Verification
After publishing, verify:
- The new version appears on [crates.io](https://crates.io/crates/magicapi-ai-gateway)
- Documentation is updated on [docs.rs](https://docs.rs/magicapi-ai-gateway)
- The GitHub release is visible (if using GitHub)


This process follows Rust community best practices for releasing crates. Remember to:
- Follow semantic versioning (MAJOR.MINOR.PATCH)
- Test thoroughly before releasing
- Document all significant changes
- Keep your repository and crates.io package in sync

Would you like me to explain any part of this process in more detail?

## Testing Deployment

MagicAPI provides a testing deployment of the AI Gateway, hosted in our London data centre. This deployment is intended for testing and evaluation purposes only, and should not be used for production workloads.

### Testing Gateway URL
```
https://gateway.magicapi.dev
```

### Send Example Request to Testing Gateway
```bash
curl --location 'https://gateway.magicapi.dev/v1/chat/completions' \
  --header 'Authorization: Bearer YOUR_API_KEY' \
  --header 'Content-Type: application/json' \
  --header 'x-provider: groq' \
  --data '{
    "model": "llama-3.1-8b-instant",
    "messages": [
        {
            "role": "user",
            "content": "Write a poem"
        }
    ],
    "stream": true,
    "max_tokens": 300
}'
```

> **Note**: This deployment is provided for testing and evaluation purposes only. For production workloads, please deploy your own instance of the gateway or contact us for information about production-ready managed solutions.

## Supported Models

### AWS Bedrock Models

The gateway currently supports the following AWS Bedrock models:

- `anthropic.claude-v2`
- `anthropic.claude-v2:1`
- `anthropic.claude-3-sonnet-20240229-v1:0`
- `anthropic.claude-3-haiku-20240307-v1:0`

More models will be added in future releases.

## Environment Variables

```bash
RUST_LOG=debug # Logging level (debug, info, warn, error)

# AWS Bedrock Configuration (Optional - can also be set via headers)
AWS_REGION=us-east-1                  # Default AWS region for Bedrock
AWS_ACCESS_KEY_ID=your_access_key     # AWS access key
AWS_SECRET_ACCESS_KEY=your_secret_key # AWS secret key
AWS_SESSION_TOKEN=your_session_token  # Optional: For temporary credentials
```
