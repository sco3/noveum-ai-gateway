# Fireworks AI Provider Integration

## Overview
The Fireworks provider enables access to various open-source models through Fireworks' high-performance inference infrastructure.

## Base Configuration
```rust:src/providers/fireworks.rs
startLine: 11
endLine: 17
```

## Configuration

### Environment Variables
```bash
FIREWORKS_API_KEY=fw-...
```

### Headers
```bash
Authorization: Bearer fw-...
x-provider: fireworks
```

## Header Processing
The provider implements standard header processing:
```rust:src/providers/fireworks.rs
startLine: 29
endLine: 42
```

## Supported Models

### Open Source Models
- Llama 2 (7B, 13B, 70B)
- Mistral (7B, 8x7B)
- CodeLlama
- MPT Series
- Falcon Series

## API Endpoints

### Chat Completions
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: fireworks" \
  -H "Authorization: Bearer $FIREWORKS_API_KEY" \
  -d '{
    "model": "accounts/fireworks/models/llama-v2-70b-chat",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Streaming
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: fireworks" \
  -H "Authorization: Bearer $FIREWORKS_API_KEY" \
  -d '{
    "model": "accounts/fireworks/models/llama-v2-70b-chat",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## SDK Integration

### Node.js (OpenAI SDK Compatible)
```typescript
import OpenAI from 'openai';

const fireworks = new OpenAI({
  apiKey: process.env.FIREWORKS_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "fireworks" }
});
```

## Error Handling

| Error Code | Description | Solution |
|------------|-------------|----------|
| 401 | Invalid API key | Check your Fireworks API key |
| 429 | Rate limit exceeded | Implement backoff strategy |
| 500 | Server error | Check server logs |

## Best Practices

1. **Model Selection**
   - Choose appropriate model size
   - Consider inference speed requirements
   - Balance cost vs performance

2. **Rate Limiting**
   - Implement exponential backoff
   - Monitor usage quotas
   - Use streaming for long responses

3. **Error Handling**
   - Implement retry logic
   - Handle timeouts gracefully
   - Log errors appropriately

## Additional Resources

- [Fireworks AI Documentation](https://fireworks.ai/docs)
- [Model List](https://fireworks.ai/models)
- [API Reference](https://fireworks.ai/docs/reference)
- [Pricing Information](https://fireworks.ai/pricing) 