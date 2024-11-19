# Anthropic Provider Integration

## Overview
The Anthropic provider integrates with Claude models through Anthropic's API, with automatic path transformation for OpenAI-compatible endpoints.

## Configuration

### Environment Variables
```bash
ANTHROPIC_API_KEY=sk-ant-...
```

### Headers
```bash
Authorization: Bearer sk-ant-...
x-provider: anthropic
```

## API Compatibility

The provider automatically transforms OpenAI-compatible requests to Anthropic's format:

```typescript:src/providers/anthropic.rs
startLine: 29
endLine: 35
```

## Supported Models
- claude-3-opus-20240229
- claude-3-sonnet-20240229
- claude-3-haiku-20240229
- claude-2.1
- claude-2.0
- claude-instant-1.2

## Examples

### Basic Chat Completion
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: anthropic" \
  -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
  -d '{
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Streaming Response
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: anthropic" \
  -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
  -d '{
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## SDK Integration

### Node.js (using Anthropic SDK)
```typescript
import Anthropic from '@anthropic-ai/sdk';

const anthropic = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY,
  baseURL: "http://localhost:3000/v1",
  defaultHeaders: { "x-provider": "anthropic" }
});
```

### Python
```python
from anthropic import Anthropic

client = Anthropic(
    api_key="your-api-key",
    base_url="http://localhost:3000/v1",
    default_headers={"x-provider": "anthropic"}
)
```

## Error Handling

| Error Code | Description | Solution |
|------------|-------------|----------|
| 401        | Invalid API key | Check your Anthropic API key |
| 429        | Rate limit exceeded | Implement backoff strategy |
| 500        | Server error | Check server logs |

## Best Practices

1. **API Version Header**
   - The provider automatically sets the correct Anthropic API version
   - Current version: `2023-06-01`

2. **Authentication**
   - Use Bearer token format
   - API key is automatically converted to x-api-key format

3. **Content Types**
   - Always use `application/json`
   - UTF-8 encoding required

## Limitations

1. **Model Availability**
   - Not all Claude models may be available
   - Check Anthropic's documentation for current model list

2. **Rate Limits**
   - Follows Anthropic's rate limiting
   - Implement appropriate retry logic

## Additional Resources

- [Anthropic API Documentation](https://docs.anthropic.com/claude/reference)
- [Claude Model Overview](https://docs.anthropic.com/claude/docs/models-overview)
- [Rate Limits](https://docs.anthropic.com/claude/reference/rate-limits) 