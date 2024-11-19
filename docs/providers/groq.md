# GROQ Provider Integration

## Overview
The GROQ provider offers OpenAI-compatible API access to GROQ's high-performance inference infrastructure.

## Base Configuration
```rust:src/providers/groq.rs
startLine: 13
endLine: 15
```

## Supported Models
- Mixtral 8x7B
- LLaMA 2 70B
- Custom models (check GROQ's documentation)

## Configuration

### Environment Variables
```bash
GROQ_API_KEY=gsk_...
```

### Headers
```bash
Authorization: Bearer gsk_...
x-provider: groq
Content-Type: application/json
```

## API Endpoints

### Chat Completions
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: groq" \
  -H "Authorization: Bearer $GROQ_API_KEY" \
  -d '{
    "model": "mixtral-8x7b-32768",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Streaming
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: groq" \
  -H "Authorization: Bearer $GROQ_API_KEY" \
  -d '{
    "model": "mixtral-8x7b-32768",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## SDK Integration

### Node.js (OpenAI SDK)
```typescript
import OpenAI from 'openai';

const groq = new OpenAI({
  apiKey: process.env.GROQ_API_KEY,
  baseURL: "http://localhost:3000/v1/",
  defaultHeaders: { "x-provider": "groq" }
});

const response = await groq.chat.completions.create({
  model: "mixtral-8x7b-32768",
  messages: [{ role: "user", content: "Hello!" }]
});
```

## Error Handling

| Error Code | Description | Solution |
|------------|-------------|----------|
| 401 | Invalid API key | Verify your GROQ API key |
| 429 | Rate limit exceeded | Implement backoff strategy |
| 500 | Server error | Check server logs |

## Authentication
The provider implements strict authentication validation:
```rust:src/providers/groq.rs
startLine: 40
endLine: 55
``` 