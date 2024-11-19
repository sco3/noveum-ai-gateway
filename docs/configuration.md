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

## Configuration File
You can also use a `config.toml` file:

```toml
[server]
port = 3000
host = "0.0.0.0"
max_connections = 100

[security]
enable_cors = true
allowed_origins = ["https://example.com"]

[providers]
default = "openai"

[providers.openai]
api_key = "sk-..."
timeout = 30

[providers.anthropic]
api_key = "sk-ant-..."
timeout = 30
```

## Rate Limiting

### Global Rate Limits
```toml
[rate_limiting]
requests_per_minute = 1000
burst_size = 50
```

### Per-Provider Rate Limits
```toml
[rate_limiting.openai]
requests_per_minute = 500
burst_size = 25

[rate_limiting.anthropic]
requests_per_minute = 300
burst_size = 15
```

## Logging Configuration

### Log Levels
```bash
RUST_LOG=error    # Only errors
RUST_LOG=warn     # Warnings and errors
RUST_LOG=info     # Normal logging (recommended)
RUST_LOG=debug    # Verbose logging
RUST_LOG=trace    # Very verbose logging
```

### Custom Logging Format
```bash
RUST_LOG_FORMAT="timestamp,level,target"
```

## Monitoring & Metrics

### Prometheus Metrics
```toml
[metrics]
enable_prometheus = true
prometheus_path = "/metrics"
```

### Health Checks
```toml
[health]
enable_health_check = true
health_check_path = "/health"
```

## Cache Configuration

### Redis Cache
```toml
[cache]
enable_redis = true
redis_url = "redis://localhost:6379"
cache_ttl = 3600  # seconds
```

## SSL/TLS Configuration

### Certificate Settings
```toml
[tls]
enable_tls = true
cert_path = "/path/to/cert.pem"
key_path = "/path/to/key.pem"
```

## Advanced Configuration

### Connection Pooling
```toml
[connection_pool]
max_idle_per_host = 100
max_lifetime = 3600
idle_timeout = 300
```

### Retry Settings
```toml
[retry]
max_retries = 3
initial_backoff = 1000  # milliseconds
max_backoff = 30000    # milliseconds
```

## Environment-Specific Configurations

### Development
```bash
RUST_LOG=debug
ENABLE_CORS=true
ALLOWED_ORIGINS="*"
```

### Production
```bash
RUST_LOG=info
ENABLE_CORS=true
ALLOWED_ORIGINS="https://api.production.com"
```

### Testing
```bash
RUST_LOG=debug
TEST_MODE=true
MOCK_RESPONSES=true
``` 