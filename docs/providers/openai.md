# OpenAI Provider Integration

## Overview
The OpenAI provider supports all OpenAI models and maintains compatibility with the official OpenAI API specification.

## Supported Models
- GPT-4 Turbo
- GPT-4
- GPT-3.5-turbo
- GPT-3.5-turbo-16k
- Text embedding models
- DALL-E 3
- Whisper

## Configuration

### Environment Variables
```bash
OPENAI_API_KEY=sk-...
OPENAI_ORG_ID=org-... # Optional
```

### Headers
```bash
Authorization: Bearer sk-...
x-provider: openai
```

## API Endpoints

### Chat Completions
```bash
POST /v1/chat/completions
```

Example request:
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: openai" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}],
    "temperature": 0.7
  }'
```

### Streaming
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: openai" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## SDK Integration

### Node.js
```typescript
import OpenAI from 'openai';

const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "openai" }
});
```

### Python
```python
from openai import OpenAI

client = OpenAI(
    api_key="your-api-key",
    base_url="http://localhost:3000/v1/",
    default_headers={"x-provider": "openai"}
)
```

## Error Handling

| Error Code | Description | Solution |
|------------|-------------|----------|
| 401 | Invalid API key | Check your OpenAI API key |
| 429 | Rate limit exceeded | Implement backoff strategy |
| 500 | Server error | Check server logs |

## Best Practices

1. **Rate Limiting**
   - Implement exponential backoff
   - Monitor usage quotas
   - Use streaming for long responses

2. **Error Handling**
   - Implement retry logic
   - Handle timeouts gracefully
   - Log errors appropriately

3. **Performance**
   - Use appropriate max_tokens
   - Batch requests when possible
   - Implement caching for repeated requests

## Examples

### Function Calling
```json
{
  "model": "gpt-4",
  "messages": [{"role": "user", "content": "What's the weather?"}],
  "functions": [{
    "name": "get_weather",
    "parameters": {
      "type": "object",
      "properties": {
        "location": {"type": "string"},
        "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}
      }
    }
  }]
}
```

### Vision API
```json
{
  "model": "gpt-4-vision-preview",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "What's in this image?"
        },
        {
          "type": "image_url",
          "image_url": "https://example.com/image.jpg"
        }
      ]
    }
  ]
}
```

## Monitoring

### Available Metrics
- Request latency
- Token usage
- Error rates
- Request volume

### Prometheus Metrics
```bash
# HELP openai_requests_total Total number of requests to OpenAI
# TYPE openai_requests_total counter
openai_requests_total{model="gpt-4"} 100
``` 