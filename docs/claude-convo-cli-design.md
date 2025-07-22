# Claude Convo CLI - Revised Design

## CLI-First Approach with Beautiful Output

Starting with a powerful CLI tool that produces gorgeous, readable output. This can later be wrapped in a TUI or used as-is.

## Core Commands

### `claude-convo list`
```bash
# List all projects
$ claude-convo list
Projects in ~/.claude/projects:
  opencode       5 sessions   2.3 MB   Last: 2 hours ago
  llama-core    10 sessions   5.1 MB   Last: 2 days ago
  workspace     34 sessions  12.4 MB   Last: 3 days ago

# List sessions in a project
$ claude-convo list opencode
Sessions in opencode:
  2025-07-22 00:49  │ 45 msgs  │   2.3 MB │ "im really interested in the claude..."
  2025-07-21 20:46  │ 312 msgs │  15.2 MB │ "Help me implement a parser for..."
  2025-07-11 01:46  │ 1.2k msgs│  48.7 MB │ "Can you analyze this codebase..."
```

### `claude-convo show`
```bash
# Show a conversation with pretty formatting
$ claude-convo show 0697fd58

┌─ Session: 0697fd58-7182-4faa-91b4-c76dded9374b ─────────────┐
│ Project: opencode                                            │
│ Started: 2025-07-22 00:49:44 UTC                           │
│ Duration: 5 minutes 23 seconds                              │
│ Messages: 45 (23 user, 22 assistant)                       │
│ Tokens: 14,695 in → 6,523 out                              │
│ Cost: ~$0.42                                                │
└──────────────────────────────────────────────────────────────┘

[00:49:44] USER ═══════════════════════════════════════════════
im really interested in the claude code conversation 
serialization format. you could say intensely so! ultrathink 
deep research to understand it better...

[00:49:51] ASSISTANT ═════════════════════════════════════════
I'll dive deep into Claude Code's conversation serialization 
format. Let me start by creating a research plan and exploring 
the actual files.

[💭 Thinking - 650 tokens] (use --show-thinking to expand)

[00:49:57] TOOL: TodoWrite ═══════════════════════════════════
{
  "todos": [
    {
      "id": "1",
      "content": "Locate and explore ~/.claude/projects",
      "status": "pending",
      "priority": "high"
    }
  ]
}

[00:49:57] RESULT ════════════════════════════════════════════
✓ Todos have been modified successfully
```

### `claude-convo search`
```bash
# Search across all conversations
$ claude-convo search "serialization format"

Found 3 matches across 2 sessions:

opencode/0697fd58 [2025-07-22 00:49:44]
  USER: "...the claude code conversation serialization format..."
  ASSISTANT: "...Claude Code's conversation serialization format..."

workspace/38ef2388 [2025-07-11 01:46:02]  
  ASSISTANT: "...The JSON serialization format uses..."
```

### `claude-convo stats`
```bash
# Show usage statistics
$ claude-convo stats --period week

Claude Code Usage Statistics (Last 7 days)
═══════════════════════════════════════════════════════════════

Sessions:         23
Total Duration:   4h 32m
Messages:         1,847 (934 user, 913 assistant)

Token Usage:
  Input:          487,293 tokens
  Output:         234,821 tokens  
  Cache Hits:     45% (saved ~$12.30)

Costs:
  Estimated:      $42.15
  Per Session:    $1.83 avg

Most Used Tools:
  1. Bash         (234 calls)
  2. Edit         (189 calls)
  3. Read         (156 calls)
  4. TodoWrite    (89 calls)

Activity by Day:
  Mon ████████████████████░░░░ 35%
  Tue ███████████░░░░░░░░░░░░░ 22%
  Wed ████████░░░░░░░░░░░░░░░░ 15%
  Thu ██████░░░░░░░░░░░░░░░░░░ 12%
  Fri █████████░░░░░░░░░░░░░░░ 16%
```

### `claude-convo export`
```bash
# Export to various formats
$ claude-convo export 0697fd58 --format markdown
Writing to: claude-convo-export-20250722-004944.md
✓ Exported 45 messages (2.3 MB)

# Export with filters
$ claude-convo export 0697fd58 --format json --only-tools
✓ Exported 12 tool calls to: tools-export.json

# Export for analysis
$ claude-convo export --all --format csv --output usage-data.csv
✓ Exported usage data for 52 sessions
```

## Implementation Plan (CLI-First)

### Phase 1: Core CLI (Days 1-3)
```rust
use clap::{Parser, Subcommand};
use colored::*;
use prettytable::{Table, row};

#[derive(Parser)]
#[command(name = "claude-convo")]
#[command(about = "Browse and analyze Claude Code conversations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List projects or sessions
    List {
        /// Project name (optional)
        project: Option<String>,
    },
    
    /// Show a conversation
    Show {
        /// Session ID (can be partial)
        session: String,
        
        #[arg(long)]
        show_thinking: bool,
        
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    
    /// Search conversations
    Search {
        /// Search query
        query: String,
        
        #[arg(long)]
        project: Option<String>,
    },
    
    /// Show statistics
    Stats {
        #[arg(long, default_value = "week")]
        period: String,
    },
    
    /// Export conversations
    Export {
        /// Session ID or "all"
        session: String,
        
        #[arg(long, default_value = "markdown")]
        format: ExportFormat,
    },
}
```

### Phase 2: Pretty Output (Days 4-5)
```rust
// Beautiful terminal output
fn print_session_header(session: &Session) {
    println!("{}", "┌─ Session ─────────────────┐".bright_blue());
    println!("│ {} │", format!("ID: {}", session.id).bright_white());
    println!("│ {} │", format!("Started: {}", session.start_time).white());
    println!("{}", "└───────────────────────────┘".bright_blue());
}

fn print_message(msg: &Message) {
    match msg {
        Message::User(content) => {
            println!("{}", format!("[{}] USER {}", 
                msg.timestamp.format("%H:%M:%S"),
                "═".repeat(50)
            ).bright_cyan());
            println!("{}", content.white());
        }
        Message::Assistant(content) => {
            println!("{}", format!("[{}] ASSISTANT {}", 
                msg.timestamp.format("%H:%M:%S"),
                "═".repeat(45)
            ).bright_green());
            // Format with syntax highlighting
        }
    }
}
```

### Phase 3: Rich Features (Days 6-7)
- JSON/CSV/Markdown export
- Streaming for large files
- Progress bars for long operations
- Config file support (~/.claude-convo.toml)

## Key Benefits of CLI-First

1. **I can actually test it!** Run commands and see output
2. **Faster iteration** - No UI state management complexity
3. **Scriptable** - Users can pipe, grep, integrate with other tools
4. **Progressive enhancement** - Add TUI later as a wrapper
5. **Better for CI/CD** - Can be used in automation

## Example Development Session

```bash
# Initial implementation
$ cargo run -- list
Error: No sessions found in /Users/tito/.claude/projects

# Fix path detection
$ cargo run -- list
Projects in ~/.claude/projects:
  opencode       5 sessions   2.3 MB   Last: 2 hours ago

# Test show command
$ cargo run -- show 0697
Parsing session 0697fd58-7182-4faa-91b4-c76dded9374b...
[renders conversation]

# Add pretty colors
$ cargo run -- show 0697 --color always
[beautiful colored output]

# Test export
$ cargo run -- export 0697 --format json | jq '.messages[0]'
{
  "timestamp": "2025-07-22T00:49:44.544Z",
  "role": "user",
  "content": "im really interested in..."
}
```

## Migration to TUI

Once the CLI is solid, the TUI becomes a thin wrapper:

```rust
// The TUI just calls our existing CLI functions
match key {
    KeyCode::Enter => {
        let session = get_selected_session();
        let output = cli::show_session(session, ShowOpts::default());
        self.current_view = View::Conversation(output);
    }
}
```

This approach is SO much better for development! 🚀