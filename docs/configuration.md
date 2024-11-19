# Configuration Guide

## Environment Variables

### Core Settings
```bash
# Server Configuration
PORT=3000
HOST=0.0.0.0
RUST_LOG=info

# Performance Tuning
MAX_CONNECTIONS=100
TCP_KEEPALIVE_INTERVAL=60
TCP_NODELAY=true

# Security
ENABLE_CORS=true
ALLOWED_ORIGINS="https://example.com,https://api.example.com"
```

### Provider API Keys
```bash
# OpenAI
OPENAI_API_KEY=sk-...

# Anthropic
ANTHROPIC_API_KEY=sk-ant-...

# GROQ
GROQ_API_KEY=gsk_...

# Fireworks
FIREWORKS_API_KEY=fw-...

# Together AI
TOGETHER_API_KEY=tok_...

# AWS Bedrock
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=...
AWS_REGION=us-east-1
```

## Providers

This section details the configuration and usage of various API providers integrated into the system. Each provider has specific requirements and configurations that must be adhered to for successful integration.

### OpenAI
- **Base URL**: `https://api.openai.com`
- **Authentication**: Requires `x-magicapi-api-key` or `Authorization` header with a Bearer token.
- **Headers**: Automatically sets `Content-Type` to `application/json`.

### Anthropic
- **Base URL**: `https://api.anthropic.com`
- **Authentication**: Requires `Authorization` header with a Bearer token, converted to `x-api-key`.
- **Headers**: Sets `Content-Type` to `application/json` and `anthropic-version` to `2023-06-01`.
- **Path Transformation**: Transforms `/chat/completions` to `/v1/messages`.

### Groq
- **Base URL**: `https://api.groq.com/openai`
- **Authentication**: Requires `Authorization` header with a Bearer token.
- **Headers**: Sets `Content-Type` to `application/json`.

### Fireworks
- **Base URL**: `https://api.fireworks.ai/inference/v1`
- **Authentication**: Requires `Authorization` header with a Bearer token.
- **Headers**: Sets `Content-Type` and `Accept` to `application/json`.
- **Path Transformation**: Strips `/v1` prefix from paths.

### Together
- **Base URL**: `https://api.together.xyz`
- **Authentication**: Requires `Authorization` header with a Bearer token.
- **Headers**: Sets `Content-Type` to `application/json`.

### AWS Bedrock
- **Base URL**: `https://bedrock-runtime.{region}.amazonaws.com`
- **Authentication**: Requires AWS signing with `x-aws-access-key-id`, `x-aws-secret-access-key`, and `x-aws-region`.
- **Headers**: Sets `Content-Type` to `application/json` and preserves AWS-specific headers.
- **Path Transformation**: Transforms paths to `/model/{model}/converse-stream`.
- **Request Body Transformation**: Converts request body to include `inferenceConfig`.
- **Response Processing**: Handles AWS event stream responses and adds CORS headers.

Ensure that the environment variables for each provider are correctly set in your configuration files, as shown in the `configuration.md` file. This includes API keys and any other necessary credentials.
