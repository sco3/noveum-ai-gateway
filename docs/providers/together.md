# Together AI Provider Integration

## Overview
The Together AI provider enables access to multiple open-source models through Together's inference infrastructure.

## Base Configuration
```rust:src/providers/together.rs
startLine: 11
endLine: 17
```

## Configuration

### Environment Variables
```bash
TOGETHER_API_KEY=tok_...
```

### Headers
```bash
Authorization: Bearer tok_...
x-provider: together
Content-Type: application/json
```

## Supported Models

### Open Source Models
- Llama 2 (7B, 13B, 70B)
- Mixtral-8x7B
- CodeLlama
- Stable LM
- Yi Series
- Qwen Series

## API Endpoints

### Chat Completions
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: together" \
  -H "Authorization: Bearer $TOGETHER_API_KEY" \
  -d '{
    "model": "togethercomputer/llama-2-70b-chat",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Streaming
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: together" \
  -H "Authorization: Bearer $TOGETHER_API_KEY" \
  -d '{
    "model": "togethercomputer/llama-2-70b-chat",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## SDK Integration

### Node.js (OpenAI SDK Compatible)
```typescript
import OpenAI from 'openai';

const together = new OpenAI({
  apiKey: process.env.TOGETHER_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "together" }
});

const response = await together.chat.completions.create({
  model: "togethercomputer/llama-2-70b-chat",
  messages: [{ role: "user", content: "Hello!" }]
});
```

## Error Handling

| Error Code | Description | Solution |
|------------|-------------|----------|
| 401 | Invalid API key | Check your Together API key |
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

## Headers Processing
The provider implements header processing as shown:
```rust:src/providers/together.rs
startLine: 29
endLine: 45
```

## Additional Resources

- [Together AI Documentation](https://docs.together.ai)
- [Available Models](https://docs.together.ai/reference/models)
- [Pricing Information](https://www.together.ai/pricing) 