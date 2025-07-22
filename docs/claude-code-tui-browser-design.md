# Claude Code Conversation Browser TUI - Design Document

## Project: `claude-convo` - Terminal UI for browsing Claude Code conversations

## 🎯 Core Features & User Stories

### MVP Features
1. **Session Browser**
   - List all projects in `~/.claude/projects`
   - List all sessions within a project
   - Show session metadata (date, size, duration)
   - Search/filter sessions by date or content

2. **Conversation Viewer**
   - Display messages in chronological order
   - Syntax highlighting for code blocks
   - Collapse/expand long messages
   - Show thinking blocks (toggle visibility)
   - Display tool calls and results
   - Token usage statistics

3. **Navigation**
   - Keyboard-driven interface
   - Jump to specific messages
   - Search within conversation
   - Export conversation segments

### User Stories
- As a developer, I want to quickly browse my past Claude conversations
- As a power user, I want to search across all my sessions for specific code/commands
- As an analyst, I want to see token usage and cost metrics
- As a debugger, I want to trace through tool executions step-by-step

## 🛠 Tech Stack Evaluation

### Language: **Rust** (Selected)
**Why Rust?**
- Excellent TUI ecosystem (ratatui, crossterm)
- Fast JSONL parsing with serde
- Memory safety for large file handling
- Great CLI tooling (clap)
- Single binary distribution

**Alternatives Considered:**
- **Go**: Good TUI libs (bubbletea), but less mature JSON handling
- **Python**: Rich/Textual are excellent, but slower and distribution is harder
- **Node.js**: Blessed/Ink are good, but heavy runtime dependency

### Key Dependencies
```toml
[dependencies]
ratatui = "0.26"        # TUI framework
crossterm = "0.27"      # Terminal control
serde_json = "1.0"      # JSON parsing
clap = "4.0"           # CLI arguments
tokio = "1.0"          # Async runtime
syntect = "5.0"        # Syntax highlighting
fuzzy-matcher = "0.3"   # Fuzzy search
```

## 🏗 Architecture

### Data Model
```rust
struct Session {
    id: String,
    project_path: PathBuf,
    events: Vec<Event>,
    metadata: SessionMetadata,
}

struct Event {
    uuid: String,
    parent_uuid: Option<String>,
    timestamp: DateTime<Utc>,
    event_type: EventType,
    message: Message,
}

enum EventType {
    User,
    Assistant,
}

enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
}

struct AssistantMessage {
    id: String,
    model: String,
    content: Vec<ContentBlock>,
    usage: TokenUsage,
}

enum ContentBlock {
    Text(String),
    Thinking { content: String, signature: String },
    ToolUse { id: String, name: String, input: Value },
}
```

### Component Architecture
```
┌─────────────────────────────────────────────┐
│              CLI Interface (clap)           │
├─────────────────────────────────────────────┤
│                TUI Layer                    │
│  ┌─────────┐ ┌─────────┐ ┌──────────────┐ │
│  │Sessions │ │ Viewer  │ │   Search     │ │
│  │  List   │ │  Pane   │ │   Dialog     │ │
│  └─────────┘ └─────────┘ └──────────────┘ │
├─────────────────────────────────────────────┤
│            State Management                 │
│         (Event-driven with tokio)           │
├─────────────────────────────────────────────┤
│             Data Layer                      │
│  ┌─────────┐ ┌─────────┐ ┌──────────────┐ │
│  │ Parser  │ │ Index   │ │    Cache     │ │
│  └─────────┘ └─────────┘ └──────────────┘ │
└─────────────────────────────────────────────┘
```

## 📊 Implementation Milestones

### Milestone 1: Core Data Layer (Week 1)
- [x] JSONL parser for conversation files
- [x] Event chain reconstruction
- [x] Basic data structures
- [x] File system traversal

### Milestone 2: Basic TUI (Week 2)
- [ ] Session list view
- [ ] Basic message display
- [ ] Keyboard navigation
- [ ] Help screen

### Milestone 3: Rich Viewing (Week 3)
- [ ] Syntax highlighting
- [ ] Thinking block toggles
- [ ] Tool call formatting
- [ ] Message collapsing

### Milestone 4: Search & Filter (Week 4)
- [ ] Fuzzy search implementation
- [ ] Date range filtering
- [ ] Content search
- [ ] Search results highlighting

### Milestone 5: Advanced Features (Week 5)
- [ ] Export functionality
- [ ] Token usage analytics
- [ ] Session comparison
- [ ] Bookmarks/favorites

### Milestone 6: Polish & Performance (Week 6)
- [ ] Lazy loading for large files
- [ ] Caching layer
- [ ] Configuration file
- [ ] Package & distribute

## ⚖️ Key Tradeoffs & Decisions

### 1. **Rust vs Higher-Level Language**
- **Tradeoff**: Longer development time vs better performance
- **Decision**: Rust for performance and distribution
- **Rationale**: Single binary, no runtime deps, handles large files well

### 2. **Full Indexing vs On-Demand Parsing**
- **Tradeoff**: Startup time & disk usage vs search speed
- **Decision**: Hybrid - index metadata, parse content on-demand
- **Rationale**: Most sessions won't be opened, but search needs to be fast

### 3. **TUI Complexity**
- **Tradeoff**: Feature richness vs simplicity
- **Decision**: Start simple, progressive enhancement
- **Rationale**: MVP should be immediately useful

### 4. **Thinking Block Privacy**
- **Tradeoff**: Full transparency vs privacy
- **Decision**: Hidden by default, toggle to show
- **Rationale**: Thinking blocks may contain sensitive reasoning

### 5. **Search Scope**
- **Tradeoff**: Local project vs global search
- **Decision**: Both, with clear UI distinction
- **Rationale**: Different use cases need different scopes

## 🎨 UI/UX Mockup

```
┌─ Claude Conversation Browser ────────────────────────────────┐
│ [Projects] [Sessions] [Search] [Help]                  [q]uit │
├──────────────────┬───────────────────────────────────────────┤
│ opencode (5)     │ Session: 0697fd58-7182-4faa-91b4         │
│ > llama-core (10)│ 2025-07-22 00:49:44 | 2.3MB | 45 msgs   │
│ workspace (34)   ├───────────────────────────────────────────┤
│                  │ USER: im really interested in the claude  │
│ Sessions:        │ code conversation seralization format...  │
│ ─────────        │                                           │
│ > 2025-07-22     │ ──────────────────────────────────────    │
│   00:49 [45m]    │                                           │
│ 2025-07-21       │ ASSISTANT: I'll dive deep into Claude    │
│   20:46 [312m]   │ Code's conversation serialization format. │
│ 2025-07-11       │                                           │
│   01:46 [1.2k]   │ [↓ Thinking (650 tokens)]                 │
│                  │                                           │
│ [/] Search       │ [TodoWrite] ────────────────────          │
│ [f] Filter       │ {                                         │
│ [e] Export       │   "todos": [                              │
│                  │     {                                     │
│                  │       "id": "1",                          │
├──────────────────┴───────────────────────────────────────────┤
│ Tokens: 14,695 in | 6,523 out | Cost: $0.42 | Model: opus-4 │
└───────────────────────────────────────────────────────────────┘
```

## 🚀 Usage Examples

```bash
# Browse all conversations
claude-convo

# Open specific project
claude-convo --project opencode

# Search across all sessions
claude-convo --search "serialization format"

# Export session to markdown
claude-convo --export 0697fd58-7182-4faa-91b4 -o conversation.md

# Show only sessions from last week
claude-convo --since "1 week ago"

# Compare token usage across sessions
claude-convo --stats
```

## 🔮 Future Enhancements

1. **Web UI** - Optional web server mode
2. **Conversation Replay** - Step through tool executions
3. **Diff View** - Compare code changes across messages
4. **Analytics Dashboard** - Usage patterns, cost tracking
5. **Plugin System** - Custom renderers for specific tools
6. **Multi-Account** - Support multiple CLAUDE_HOME directories
7. **Real-time Monitoring** - Watch active sessions live

## 🎯 Success Metrics

- Load 1GB+ session files without lag
- Search across 1000+ sessions in <1s
- Keyboard-only navigation
- Zero configuration required
- Cross-platform (macOS, Linux, Windows)