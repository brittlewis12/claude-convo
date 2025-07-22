# Claude Code Conversation Serialization Format Deep Dive

## Overview
Claude Code stores conversation transcripts locally on your machine in a structured JSONL (JSON Lines) format. Each conversation is saved as a separate file, with each line representing a single event in the conversation.

## Directory Structure
```
~/.claude/
├── projects/                    # Main conversation storage
│   ├── -Users-tito-code-opencode/   # Project-specific folder (path is hashed)
│   │   ├── 0697fd58-7182-4faa-91b4-c76dded9374b.jsonl  # Session file
│   │   └── ...
│   └── ...
├── todos/                      # Todo list storage for sessions
├── shell-snapshots/           # Shell state snapshots
└── statsig/                   # Analytics/telemetry data
```

## JSONL File Format

Each line in a JSONL file is a complete JSON object representing a conversation event. The events form a linked list through parent-child UUID relationships.

### Core Fields Present in Every Event

```json
{
  "uuid": "unique-identifier",           // Unique ID for this event
  "parentUuid": "parent-uuid-or-null",   // Links to previous event
  "sessionId": "session-id",              // Session identifier
  "timestamp": "ISO-8601-timestamp",      // When event occurred
  "type": "user|assistant",               // Message originator
  "userType": "external",                 // User type
  "cwd": "/current/working/directory",    // Working directory
  "gitBranch": "branch-name",            // Current git branch
  "version": "1.0.57",                   // Claude Code version
  "isSidechain": false,                  // Whether part of main chain
  "message": { ... }                     // Message content (varies by type)
}
```

### Message Types and Structures

#### 1. User Messages
```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": "string or array"
  }
}
```

For tool results:
```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [{
      "tool_use_id": "toolu_xxx",
      "type": "tool_result",
      "content": "result string"
    }]
  },
  "toolUseResult": {
    "oldTodos": [...],
    "newTodos": [...],
    // Other tool-specific result data
  }
}
```

#### 2. Assistant Messages
```json
{
  "type": "assistant",
  "message": {
    "id": "msg_xxx",
    "type": "message",
    "role": "assistant",
    "model": "claude-opus-4-20250514",
    "content": [
      {
        "type": "text",
        "text": "Response text"
      },
      {
        "type": "thinking",
        "thinking": "Internal reasoning",
        "signature": "cryptographic-signature"
      },
      {
        "type": "tool_use",
        "id": "toolu_xxx",
        "name": "ToolName",
        "input": { ... }
      }
    ],
    "stop_reason": null,
    "stop_sequence": null,
    "usage": {
      "input_tokens": 10,
      "cache_creation_input_tokens": 14695,
      "cache_read_input_tokens": 0,
      "output_tokens": 6,
      "service_tier": "standard"
    }
  },
  "requestId": "req_xxx"
}
```

### Tool Call Serialization

Tools are represented in the assistant's content array:
```json
{
  "type": "tool_use",
  "id": "toolu_uniqueid",
  "name": "Bash",
  "input": {
    "command": "ls -la",
    "description": "List directory contents"
  }
}
```

Tool results come back as user messages with:
```json
{
  "tool_use_id": "toolu_uniqueid",
  "type": "tool_result",
  "content": "Command output...",
  "is_error": false
}
```

### Special Features

1. **Thinking Blocks**: Assistant messages can contain thinking content with cryptographic signatures
2. **Token Usage**: Each assistant message includes detailed token usage metrics
3. **Request IDs**: Assistant messages have unique request IDs for API tracking
4. **Tool Results**: User messages can include structured tool result data

## Data Flow

1. User sends a message → Creates event with `parentUuid: null` (if first) or linking to previous
2. Assistant processes → Multiple events may be created (thinking, text, tool use)
3. Tool execution → User event with tool result
4. Chain continues with each event linking to its parent

## Missing Components

Based on the documentation, I expected but didn't find:
- `conversation_index.json` file
- SQLite database for indexing
- These may be in a different location or deprecated in newer versions

## Security Considerations

- Files are stored in plain text (no encryption)
- Contains full conversation history including code and commands
- Tool execution results may contain sensitive data
- Users should secure/purge transcripts containing credentials

## Practical Uses

1. **Session Recovery**: The `--continue` and `--resume` flags use these files
2. **Audit Trail**: Complete record of all actions taken
3. **Analysis**: Can analyze token usage, tool patterns, conversation flow
4. **Backup**: Easy to backup/migrate conversations
5. **Debugging**: Full context for troubleshooting issues