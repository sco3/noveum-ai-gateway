


Example Request
```
[
  {
    // The timestamp when this log was created (ISO 8601 or Unix time).
    "timestamp": "2025-03-05T16:03:20.123Z",

    // Identifies your service; helps group logs in observability tools.
    "resource": {
      "service.name": "noveum_ai_gateway",
      "service.version": "1.0.0",
      "deployment.environment": "production"
    },

    // A short name or type for the log record.
    "name": "ai_gateway_request_log",

    // Additional attributes capturing the details of this request/response cycle.
    "attributes": {
      // Basic identifying fields for the request.
      "id": "msg_29",
      "threadId": "thread_29",
      "org_id": "org_123",                    // Example org identifier
      "user_id": "user_456",                  // Example user identifier
      "project_id": "proj_design",            // Aligns with metadata.projectId

      // Provider/model details
      "provider": "azure",
      "model": "gpt-4-turbo",

      // Full request object (as-is from your logs).
      "request": {
        "model": "gpt-4-turbo",
        "messages": [
          {
            "role": "user",
            "content": "I'm designing a mobile app for personal finance management. Can you help me create user personas..."
          }
        ],
        "temperature": 0.7
      },

      // Full response object (as-is from your logs).
      "response": {
        "id": "chatcmpl-az-gpt4-001",
        "choices": [
          {
            "message": {
              "role": "assistant",
              "content": "# User Personas for Personal Finance Management App\n\n## Persona 1: Career-Focused Millennial..."
            },
            "finish_reason": "stop"
          }
        ],
        "usage": {
          "prompt_tokens": 48,
          "completion_tokens": 865,
          "total_tokens": 913
        }
      },

      // Metadata stored from your logs (latency, cost, tokens, statuses, etc.)
      "metadata": {
        "projectId": "proj_design",
        "projectName": "UX Design",
        "latency": 6250,
        "tokens": { "input": 48, "output": 865, "total": 913 },
        "cost": 0.0456,
        "status": "success"
      }
    }
  },
  {
    "timestamp": "2025-03-05T16:18:20.456Z",
    "resource": {
      "service.name": "noveum_ai_gateway",
      "service.version": "1.0.0",
      "deployment.environment": "production"
    },
    "name": "ai_gateway_request_log",

    "attributes": {
      "id": "msg_30",
      "threadId": "thread_30",
      "org_id": "org_789",                    // Example org identifier
      "user_id": "user_999",                  // Example user identifier
      "project_id": "proj_education",

      "provider": "together",
      "model": "llama-3-70b-instruct",

      "request": {
        "model": "llama-3-70b-instruct",
        "messages": [
          {
            "role": "user",
            "content": "I need to explain quantum computing to high school students. Can you provide a simple explanation..."
          }
        ],
        "temperature": 0.5
      },

      "response": {
        "id": "chatcmpl-tog-llama-001",
        "choices": [
          {
            "message": {
              "role": "assistant",
              "content": "# Quantum Computing for High School Students\n\n## Simple Explanation\n\nQuantum computing..."
            },
            "finish_reason": "stop"
          }
        ],
        "usage": {
          "prompt_tokens": 36,
          "completion_tokens": 595,
          "total_tokens": 631
        }
      },

      "metadata": {
        "projectId": "proj_education",
        "projectName": "Educational Content",
        "latency": 5320,
        "tokens": { "input": 36, "output": 595, "total": 631 },
        "cost": 0.0315,
        "status": "success"
      }
    }
  }
]
```