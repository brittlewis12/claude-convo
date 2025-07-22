# Claude Code Conversation Serialization Format - Complete Documentation

## Overview
Claude Code stores conversation transcripts locally in JSONL (JSON Lines) format. Based on analysis of 6,660+ real events across multiple sessions, this document provides comprehensive format documentation.

## Directory Structure
```
~/.claude/
├── projects/                    # Main conversation storage
│   ├── -Users-tito-code-opencode/   # Project-specific folder (path is URL-encoded)
│   │   ├── 0697fd58-7182-4faa-91b4-c76dded9374b.jsonl  # Session file (UUID)
│   │   └── ...
│   └── ...
├── todos/                      # Todo list storage for sessions
├── shell-snapshots/           # Shell state snapshots
└── statsig/                   # Analytics/telemetry data
```

## Event Types

Four primary event types exist in the format:

| Type | Occurrences | Purpose |
|------|-------------|---------|
| `assistant` | 3,774 (56.7%) | Claude's responses |
| `user` | 2,590 (38.9%) | User messages and tool results |
| `summary` | 293 (4.4%) | Session metadata/summaries |
| `system` | 3 (0.05%) | System warnings and metadata |

## Complete Event Structures

### 1. User Events

User events contain messages from the user and tool execution results:

```json
{
  "type": "user",
  "uuid": "unique-identifier",
  "parentUuid": "parent-uuid-or-null",
  "sessionId": "session-id",
  "timestamp": "2025-07-22T00:49:44.544Z",
  "userType": "external",
  "cwd": "/Users/tito/code/project",
  "gitBranch": "main",
  "version": "1.0.57",
  "isSidechain": false,
  "isMeta": false,
  "isCompactSummary": false,
  "message": {
    "role": "user",
    "content": "string or array"  // See content variants below
  },
  "toolUseResult": { ... }  // Optional, see tool results section
}
```

#### User Content Variants

1. **Simple text**:
```json
"content": "Hello Claude, help me debug this code"
```

2. **Tool results**:
```json
"content": [{
  "type": "tool_result",
  "tool_use_id": "toolu_01ShhdFXvhWx2aRSkwFgJ3nW",
  "content": "Command executed successfully",
  "is_error": false
}]
```

3. **Images**:
```json
"content": [{
  "type": "image",
  "source": {
    "type": "base64",
    "media_type": "image/png",
    "data": "iVBORw0KGgoAAAANS..."
  }
}]
```

### 2. Assistant Events

Assistant events contain Claude's responses, including text, thinking, and tool use:

```json
{
  "type": "assistant",
  "uuid": "unique-identifier",
  "parentUuid": "parent-uuid",
  "sessionId": "session-id",
  "timestamp": "2025-07-22T00:49:51.123Z",
  "userType": "assistant",
  "cwd": "/Users/tito/code/project",
  "gitBranch": "main",
  "version": "1.0.57",
  "isSidechain": false,
  "isApiErrorMessage": false,
  "requestId": "req_01Sh...",
  "message": {
    "id": "msg_01Sh...",
    "type": "message",
    "role": "assistant",
    "model": "claude-opus-4-20250514",
    "content": [
      {
        "type": "text",
        "text": "I'll help you debug that code."
      },
      {
        "type": "thinking",
        "thinking": "The user wants debugging help. Let me analyze...",
        "signature": "sha256:abc123..."
      },
      {
        "type": "tool_use",
        "id": "toolu_01Sh...",
        "name": "Read",
        "input": {
          "file_path": "/path/to/file.py"
        }
      }
    ],
    "stop_reason": "end_turn",
    "stop_sequence": null,
    "usage": {
      "input_tokens": 1234,
      "output_tokens": 567,
      "cache_creation_input_tokens": 0,
      "cache_read_input_tokens": 123,
      "service_tier": "default",
      "server_tool_use": {
        "web_search_requests": 1
      }
    }
  }
}
```

### 3. Summary Events

Summary events appear at the start of some sessions:

```json
{
  "type": "summary",
  "summary": "Session continuation from previous conversation...",
  "leafUuid": "last-event-uuid-from-previous-session"
}
```

Common summary types:
- Session continuations with full context
- Error messages (e.g., "Invalid API key · Please run /login")
- Compact summaries of previous conversations

### 4. System Events

System events capture warnings, errors, and metadata:

```json
{
  "type": "system",
  "content": "PostToolUse:WebSearch hook execution cancelled",
  "level": "warning",
  "uuid": "unique-identifier",
  "parentUuid": "parent-uuid",
  "sessionId": "session-id",
  "timestamp": "2025-07-11T04:15:42.330Z",
  "cwd": "/Users/tito/code/opencode",
  "gitBranch": "dev",
  "version": "1.0.57",
  "userType": "system",
  "toolUseID": "toolu_01Sh...",
  "isMeta": false,
  "isSidechain": false
}
```

## Tool Use Results

Tool execution results are embedded in user events with extensive metadata:

```json
{
  "toolUseResult": {
    "type": "bash",
    "stdout": "file1.txt\nfile2.py\n",
    "stderr": "",
    "code": 0,
    "returnCodeInterpretation": "success",
    "durationMs": 45,
    "durationSeconds": 0.045,
    "interrupted": false,
    "truncated": false,
    "wasInterrupted": false,
    
    // File operations
    "file": {
      "filePath": "/path/to/file",
      "content": "file contents...",
      "numLines": 100,
      "totalLines": 100,
      "originalSize": 2048,
      "type": "text",
      "base64": "..."  // For binary files
    },
    
    // Edit operations
    "edits": [{
      "old_string": "before",
      "new_string": "after",
      "replace_all": false
    }],
    
    // Search results
    "query": "search term",
    "results": [{
      "content": [{
        "title": "Result title",
        "url": "https://..."
      }],
      "tool_use_id": "toolu_..."
    }],
    
    // Todo operations
    "oldTodos": [{
      "id": "1",
      "content": "Old task",
      "status": "completed",
      "priority": "high"
    }],
    "newTodos": [{
      "id": "2",
      "content": "New task",
      "status": "pending",
      "priority": "medium"
    }],
    
    // Structured patches (for diffs)
    "structuredPatch": [{
      "oldStart": 10,
      "oldLines": 5,
      "newStart": 10,
      "newLines": 6,
      "lines": ["...", "-old", "+new", "..."]
    }],
    
    // Usage tracking
    "totalTokens": 1500,
    "totalDurationMs": 234,
    "totalToolUseCount": 3,
    "usage": {
      "input_tokens": 1000,
      "output_tokens": 500,
      "cache_creation_input_tokens": 0,
      "cache_read_input_tokens": 100,
      "service_tier": "default"
    }
  }
}
```

## Key Fields Reference

### Common Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | string | Unique event identifier |
| `parentUuid` | string/null | Links to previous event (forms conversation chain) |
| `sessionId` | string | Session identifier |
| `timestamp` | ISO 8601 | Event timestamp with timezone |
| `type` | string | Event type: user, assistant, summary, system |
| `userType` | string | "external", "assistant", or "system" |
| `cwd` | string | Current working directory |
| `gitBranch` | string/null | Current git branch |
| `version` | string | Claude Code version |
| `isSidechain` | boolean | Indicates parallel conversation branch |
| `isMeta` | boolean | Metadata event flag |
| `requestId` | string | API request identifier (assistant events) |

### Token Usage Fields

| Field | Type | Description |
|-------|------|-------------|
| `input_tokens` | number | Tokens in the input |
| `output_tokens` | number | Tokens generated |
| `cache_creation_input_tokens` | number | Tokens used to create cache |
| `cache_read_input_tokens` | number | Tokens read from cache |
| `service_tier` | string | Service level used |
| `server_tool_use` | object | Server-side tool usage (e.g., web search) |

## Format Evolution

The format has evolved significantly over time:

1. **Early versions**: Simple user/assistant message pairs
2. **Mid versions**: Added tool use, thinking blocks, token usage
3. **Recent versions**: 
   - Summary events for session continuations
   - System events for warnings/errors
   - Rich tool result metadata
   - Image support in user messages
   - Sidechain support for parallel conversations
   - Server-side tool usage tracking

## Practical Considerations

1. **Parsing**: Use a flexible parser that can handle missing fields gracefully
2. **Event Chaining**: Follow `parentUuid` links to reconstruct conversation flow
3. **Tool Results**: Tool execution results can be very large (stdout, file contents)
4. **Summary Events**: May contain the entire previous conversation context
5. **Binary Data**: Images and binary files are base64-encoded
6. **Token Costs**: Calculate costs using token usage fields and current pricing

## Security Notes

- Conversations are stored in plain text (no encryption)
- Tool results may contain sensitive data (environment variables, file contents)
- Git branch information may reveal project structure
- Consider rotating/purging old sessions with sensitive data