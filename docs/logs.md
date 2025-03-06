Example Request
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
        "status": "success",
        "provider_request_id": "chatcmpl-az-gpt4-001"
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
        "status": "success",
        "provider_request_id": "chatcmpl-tog-llama-001"
      }
    }
  },
  {
    "timestamp": "2025-03-05T16:25:30.789Z",
    "resource": {
      "service.name": "noveum_ai_gateway",
      "service.version": "1.0.0",
      "deployment.environment": "production"
    },
    "name": "ai_gateway_request_log",

    "attributes": {
      "id": "msg_31",
      "threadId": "thread_31",
      "org_id": "org_123",
      "user_id": "user_456",
      "project_id": "proj_chat",

      "provider": "groq",
      "model": "llama-3.1-8b-instant",

      "request": {
        "model": "llama-3.1-8b-instant",
        "messages": [
          {
            "role": "user",
            "content": "Write a poem"
          }
        ],
        "stream": true,
        "max_tokens": 500
      },

      // For streaming responses, the response includes both the final response and all streamed chunks
      "response": {
        "id": "chatcmpl-ec855684-8495-420d-8807-9259228ac717",
        "model": "llama-3.1-8b-instant",
        "choices": [
          {
            "delta": {
              "role": "assistant",
              "content": "\"Moonlit Dreams\"\n\nThe night is dark, the stars are bright,\nA silver glow, a gentle light.\nThe moon, a crescent in the sky,\nLures me to dream, to wonder why.\n\nThe world is hushed, a peaceful sight,\nAs shadows dance, in the moon's pale light.\nThe trees, like sentinels of old,\nStand guard, their branches, stories untold.\n\nIn this quiet hour, I find my peace,\nA sense of calm, my worries release.\nThe moon's soft beams, upon my face,\nIlluminate my soul, a gentle, loving space.\n\nMy heart beats slow, my spirit free,\nAs I let go, and let the moment be.\nThe world, in all its beauty, shines,\nA reflection of the love that's mine.\n\nIn this moonlit night, I find my way,\nA path of dreams, that lead me to a brighter day.\nSo let the moon, its gentle light,\nGuide me through, the darkness of night.\n\nAnd when the dawn, breaks through the sky,\nAnd the sun's warm rays, upon my face, do lie,\nI'll hold on tight, to the memories of this night,\nAnd let the moon's soft beams, shine with all their might."
            }
          }
        ],
        "x_groq": {
          "id": "req_01jnkrrz2ken1bej7emqf9j2af",
          "usage": {
            "queue_time": 0.291194089,
            "prompt_tokens": 38,
            "prompt_time": 0.002701306,
            "completion_tokens": 256,
            "completion_time": 0.341333333,
            "total_tokens": 294,
            "total_time": 0.344034639
          }
        },
        // The streamed_data field contains all individual chunks received during streaming
        "streamed_data": [
          {
            "nonce": "0955",
            "id": "chatcmpl-ec855684-8495-420d-8807-9259228ac717",
            "object": "chat.completion.chunk",
            "created": 1741199015,
            "model": "llama-3.1-8b-instant",
            "system_fingerprint": "fp_fd87957473",
            "choices": [
              {
                "index": 0,
                "delta": {
                  "role": "assistant",
                  "content": ""
                },
                "logprobs": null,
                "finish_reason": null
              }
            ],
            "x_groq": {
              "id": "req_01jnkrrz2ken1bej7emqf9j2af"
            }
          },
          {
            "nonce": "ef5fe2",
            "id": "chatcmpl-ec855684-8495-420d-8807-9259228ac717",
            "object": "chat.completion.chunk",
            "created": 1741199015,
            "model": "llama-3.1-8b-instant",
            "system_fingerprint": "fp_fd87957473",
            "choices": [
              {
                "index": 0,
                "delta": {
                  "content": "\"M"
                },
                "logprobs": null,
                "finish_reason": null
              }
            ]
          },
          // Additional chunks omitted for brevity
          {
            "nonce": "d4fe",
            "id": "chatcmpl-ec855684-8495-420d-8807-9259228ac717",
            "object": "chat.completion.chunk",
            "created": 1741199015,
            "model": "llama-3.1-8b-instant",
            "system_fingerprint": "fp_fd87957473",
            "choices": [
              {
                "index": 0,
                "delta": {},
                "logprobs": null,
                "finish_reason": "stop"
              }
            ],
            "x_groq": {
              "id": "req_01jnkrrz2ken1bej7emqf9j2af",
              "usage": {
                "queue_time": 0.291194089,
                "prompt_tokens": 38,
                "prompt_time": 0.002701306,
                "completion_tokens": 256,
                "completion_time": 0.341333333,
                "total_tokens": 294,
                "total_time": 0.344034639
              }
            }
          }
        ]
      },

      "metadata": {
        "projectId": "proj_chat",
        "projectName": "Chat Application",
        "latency": 1244,
        "tokens": { "input": 38, "output": 256, "total": 294 },
        "cost": 0.0263,
        "status": "success",
        "provider_request_id": "req_01jnkrrz2ken1bej7emqf9j2af"
      }
    }
  }
]