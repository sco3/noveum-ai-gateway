# AWS Bedrock Provider Integration

## Overview
The AWS Bedrock provider enables access to AWS's AI models with automatic request signing and OpenAI-compatible endpoints.

## Configuration

### Default Constants
```rust:src/providers/bedrock.rs
startLine: 17
endLine: 22
```

### Environment Variables
```bash
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_REGION=us-east-1  # Optional, defaults to us-east-1
```

### Headers
```bash
x-aws-access-key-id: your_access_key
x-aws-secret-access-key: your_secret_key
x-aws-region: us-east-1  # Optional
x-provider: bedrock
```

## Supported Models

### Amazon Models
- amazon.titan-text-premier-v1:0 (Default)
- amazon.titan-text-express-v1:0
- amazon.titan-embed-text-v1:0

### Third-Party Models
- anthropic.claude-3-sonnet-20240229-v1:0
- anthropic.claude-v2:1
- mistral.mistral-7b-instruct-v0:2
- meta.llama2-70b-chat-v1:0

## Request Transformation
The provider automatically transforms OpenAI-format requests to Bedrock format:

```rust:src/providers/bedrock.rs
startLine: 67
endLine: 91
```

## AWS Signing
AWS Bedrock requires request signing:

```rust:src/providers/bedrock.rs
startLine: 266
endLine: 287
```

## Response Format
Responses are transformed to match OpenAI format:

```rust:src/providers/bedrock.rs
startLine: 199
endLine: 211
```

## Examples

### Basic Chat Completion
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: bedrock" \
  -H "x-aws-access-key-id: YOUR_ACCESS_KEY" \
  -H "x-aws-secret-access-key: YOUR_SECRET_KEY" \
  -d '{
    "model": "anthropic.claude-3-sonnet-20240229-v1:0",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Using Default Model
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-provider: bedrock" \
  -H "x-aws-access-key-id: YOUR_ACCESS_KEY" \
  -H "x-aws-secret-access-key: YOUR_SECRET_KEY" \
  -d '{
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Error Handling

| Error Code | Description | Solution |
|------------|-------------|----------|
| 401 | Invalid AWS credentials | Check AWS access key and secret |
| 403 | Insufficient permissions | Verify IAM permissions |
| 404 | Model not found | Check model availability in region |

## Best Practices

1. **Authentication**
   - Use IAM roles when possible
   - Rotate access keys regularly
   - Set appropriate permissions

2. **Region Selection**
   - Choose closest region
   - Verify model availability
   - Consider costs

3. **Error Handling**
   - Implement retries for AWS-specific errors
   - Handle throttling gracefully
   - Monitor usage quotas

## Additional Resources

- [AWS Bedrock Documentation](https://docs.aws.amazon.com/bedrock)
- [IAM Permissions](https://docs.aws.amazon.com/bedrock/latest/userguide/security-iam.html)
- [Available Models](https://docs.aws.amazon.com/bedrock/latest/userguide/model-ids.html) 