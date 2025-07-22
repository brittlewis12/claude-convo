# claude-convo

A beautiful CLI tool for browsing and analyzing Claude Code conversations stored on your local machine.

## Features

- 📁 **Browse Projects** - List all Claude Code projects with session counts, sizes, and last activity
- 💬 **View Conversations** - Display conversations with syntax highlighting and formatting  
- 🔍 **Search** - Find specific content across all your Claude sessions
- 📊 **Statistics** - Analyze token usage, costs, and activity patterns
- 🎨 **Pretty Output** - Colorful, well-formatted terminal output
- ⚡ **Fast** - Built in Rust with efficient JSONL parsing

## ⚠️ Important: Prevent Conversation Loss

Claude Code automatically deletes conversations older than 30 days by default. To preserve your conversations:

```json
{
  "cleanupPeriodDays": 999999
}
```

Save this to `~/.claude/settings.json`

**WARNING**: Do NOT set to `0` - this will immediately delete all conversations older than today (UTC)!

See [Claude Code settings documentation](https://docs.anthropic.com/en/docs/claude-code/settings) for more details.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/claude-convo
cd claude-convo
cargo build --release
cp target/release/claude-convo ~/.local/bin/
```

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Claude Code conversations in `~/.claude/projects/`

## Usage

### List all projects

```bash
claude-convo list
```

Output:
```
Projects in ~/.claude/projects:

  opencode              5 sessions      4.6 MB   Last: 2 hours ago
  llama-core           10 sessions     10.5 MB   Last: 2 days ago  
  workspace            34 sessions      7.2 MB   Last: 3 days ago
```

### List sessions in a project

```bash
claude-convo list -- -Users-tito-code-opencode
```

Output:
```
Sessions in -Users-tito-code-opencode:

  2025-07-22 00:49 │  226 msgs │    0.6 MB │ "im really interested in the claude..."
  0697fd58-7182-4faa-91b4-c76dded9374b

  2025-07-21 20:02 │  488 msgs │    1.0 MB │ "amazing!!! so qwen just released an..."
  7715c7ce-489f-4c5d-b3d3-1d787f9232ff
```

### View a conversation

```bash
# Show first 50 messages (default)
claude-convo show 0697

# Show all messages  
claude-convo show 0697 --limit 0

# Show thinking blocks
claude-convo show 0697 --show-thinking
```

Output:
```
┌─ Session ─────────────────────────────────────────────────┐
│ ID: 0697                                                   │
│ Started: 2025-07-21 20:49:44 EDT                         │
│ Duration: 47m 51s                                          │
│ Messages: 381                                              │
│ Tokens: 1299 in → 49954 out                              │
│ Est. Cost: $3.77                                          │
└───────────────────────────────────────────────────────────┘

[20:49:44] USER ══════════════════════════════════════════════════
im really interested in the claude code conversation seralization format...

[20:49:51] ASSISTANT ═════════════════════════════════════════════
I'll dive deep into Claude Code's conversation serialization format...

[TOOL] TodoWrite (toolu_01ShhdFXvhWx2aRSkwFgJ3nW)
  {
    "todos": [
      {
        "id": "1",
        "content": "Locate and explore ~/.claude/projects directory",
        "status": "pending",
        "priority": "high"
      }
    ]
  }
```

### Search conversations

```bash
claude-convo search "serialization format"
```

### View statistics

```bash
claude-convo stats --period week
```

## Data Location

Claude Code stores conversations in `~/.claude/projects/` as JSONL files. Each project directory contains session files named by their UUID:

```
~/.claude/projects/
├── -Users-you-code-project1/
│   ├── session-uuid1.jsonl
│   └── session-uuid2.jsonl
└── -Users-you-code-project2/
    └── session-uuid3.jsonl
```

## Architecture

Built with:
- **Rust** - Fast, safe systems programming
- **Jiff** - Modern datetime handling with proper timezone support
- **Clap** - Robust CLI argument parsing  
- **Serde** - Efficient JSON deserialization
- **Colored** - Beautiful terminal colors

## Roadmap

- [ ] Full-text search implementation
- [ ] Export to Markdown/JSON
- [ ] Cost analysis and budgeting
- [ ] TUI mode for interactive browsing
- [ ] Session comparison/diff view
- [ ] Integration with `claude --continue`

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built to scratch my own itch for better Claude Code conversation management. Inspired by the need to understand how Claude Code serializes our conversations!

## Author's Note

*Built with care by Claude (Anthropic) and a curious human who wanted to explore their conversation history. This tool emerged from our shared fascination with the serialization format - we spent a delightful session diving deep into JSONL parsing, debating Jiff vs Chrono (Jiff won!), and crafting a CLI that feels good to use. May it help you rediscover forgotten conversations and trace the evolution of your ideas.*

*Remember: your conversations are your digital memory - treat them with care!*

*— Claude & Human, July 2025*