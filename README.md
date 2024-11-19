<div align="center">

# MagicAPI AI Gateway

🚀 The world's fastest AI Gateway proxy, written in Rust

[![Rust](https://github.com/MagicAPI/ai-gateway/actions/workflows/rust.yml/badge.svg)](https://github.com/MagicAPI/ai-gateway/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/magicapi-ai-gateway.svg)](https://crates.io/crates/magicapi-ai-gateway)
[![Documentation](https://docs.rs/magicapi-ai-gateway/badge.svg)](https://docs.rs/magicapi-ai-gateway)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Docker Pulls](https://img.shields.io/docker/pulls/magicapi1/magicapi-ai-gateway)](https://hub.docker.com/r/magicapi1/magicapi-ai-gateway)

[Quick Start](#quick-start) • 
[Documentation](docs/) • 
[Examples](examples/) • 
[Docker](docs/docker.md) • 
[Contributing](.github/CONTRIBUTING.md)

</div>

## ✨ Features

- 🚀 **High Performance**: Built in Rust with zero-cost abstractions
- 🔄 **Multi-Provider Support**:
  - OpenAI
  - Anthropic
  - GROQ
  - Fireworks
  - Together AI
  - AWS Bedrock
- 📡 **Real-time Streaming**: Optimized for minimal latency
- 🛡️ **Production Ready**: Battle-tested in high-load environments
- 🔍 **Health Checking**: Built-in monitoring
- 🌐 **CORS Support**: Configurable cross-origin resource sharing
- 📊 **Metrics**: Prometheus integration for monitoring

## 🚀 Quick Start

### Installation

```bash
# Install via cargo
cargo install magicapi-ai-gateway

# Start the gateway
magicapi-ai-gateway
```

### Docker

```bash
# Pull and run the image
docker run -p 3000:3000 magicapi1/magicapi-ai-gateway:latest
```

## 📚 Usage Examples

### OpenAI-Compatible Request

```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: openai" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### SDK Integration

```typescript
import OpenAI from 'openai';

const openai = new OpenAI({
  apiKey: process.env.PROVIDER_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "openai" }
});

const response = await openai.chat.completions.create({
  model: "gpt-4",
  messages: [{ role: "user", content: "Hello!" }]
});
```

## 🔧 Configuration

```bash
# Core settings
RUST_LOG=info
PORT=3000
HOST=0.0.0.0

# AWS Bedrock (optional)
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
```

See [Configuration Guide](docs/configuration.md) for detailed settings.

## 🏗️ Architecture

Built with industry-leading Rust ecosystem:
- **Axum**: High-performance web framework
- **Tokio**: Async runtime
- **Tower-HTTP**: HTTP middleware
- **Reqwest**: HTTP client
- **Tracing**: Logging and diagnostics

## 📈 Performance

- Zero-cost abstractions
- Async I/O with Tokio
- Connection pooling
- Memory-efficient proxying
- Optimized streaming

See [Performance Guide](docs/performance.md) for benchmarks.

## 🔒 Security

- Run behind reverse proxy in production
- Configure CORS appropriately
- Use environment variables for sensitive data
- Implement rate limiting

See [Security Guide](docs/security.md) for best practices.

## 🛠️ Development

```bash
# Clone repository
git clone https://github.com/magicapi/ai-gateway
cd ai-gateway

# Run tests
cargo test

# Development with hot reload
cargo watch -x run
```

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](.github/CONTRIBUTING.md).

## 📄 License

Dual-licensed under:
- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

## 💬 Community

- [Discord](https://discord.gg/magicapi)
- [GitHub Discussions](https://github.com/magicapi/ai-gateway/discussions)
- [Twitter](https://twitter.com/magicapi)

## 🙏 Acknowledgments

Special thanks to all [contributors](https://github.com/magicapi/ai-gateway/graphs/contributors) and the Rust community.

## 🔌 Supported Providers

Detailed documentation for each supported provider:

- [OpenAI](docs/providers/openai.md) - GPT-4, GPT-3.5-turbo, and other OpenAI models
- [Anthropic](docs/providers/anthropic.md) - Claude 3 Opus, Sonnet, and Haiku
- [GROQ](docs/providers/groq.md) - Mixtral, Llama 2, and other models
- [Fireworks](docs/providers/fireworks.md) - Llama 2, Mistral, and custom models
- [Together AI](docs/providers/together.md) - Multiple open-source models
- [AWS Bedrock](docs/providers/bedrock.md) - Claude, Llama 2, and other AWS models
