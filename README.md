# claude-convo

A beautiful CLI tool for browsing and analyzing Claude Code conversations stored on your local machine.

## Features

- 📁 **Browse Projects** - List all Claude Code projects with session counts, sizes, and last activity
- 💬 **View Conversations** - Display conversations with syntax highlighting, formatting, and tool usage
- 🔍 **Search** - Find specific content across all your Claude sessions with BM25 ranking
- 📊 **Statistics** - Analyze token usage, costs, and activity patterns with beautiful visualizations
- 📤 **Export** - Save conversations as Markdown files (with optional thinking blocks)
- 🎨 **Pretty Output** - Colorful, well-formatted terminal output with proper conversation flow
- ⚡ **Fast** - Built in Rust with efficient JSONL parsing
- 🏷️ **Session Names** - Memorable word-pair names for easy session identification
- 📄 **Automatic Paging** - Large outputs automatically use your system's pager

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

```bash
git clone https://github.com/brittlewis12/claude-convo
cd claude-convo
mise install  # installs correct Rust version
cargo install --path .
```

### Prerequisites

- **Rust** - Install via [rustup.rs](https://rustup.rs) or [mise](https://mise.jdx.dev/)
- **Claude Code** conversations in `~/.claude/projects/`

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

  2025-07-22 00:49 │ nebula-quasar │  226 msgs │    0.6 MB │ "im really interested in the claude..."
  0697fd58-7182-4faa-91b4-c76dded9374b

  2025-07-21 20:02 │ galaxy-meteor │  488 msgs │    1.0 MB │ "amazing!!! so qwen just released an..."
  7715c7ce-489f-4c5d-b3d3-1d787f9232ff
```

### View a conversation

```bash
# Show all messages (default: includes thinking & tools, with automatic paging)
claude-convo show 0697

# Show first 50 messages  
claude-convo show 0697 --limit 50

# Hide thinking blocks
claude-convo show 0697 --no-thinking

# Hide tool usage
claude-convo show 0697 --no-tools

# Show by memorable name
claude-convo show nebula-quasar
```

Output:
```
┌─ Session ─────────────────────────────────────────────────┐
│ ID: 0697fd58 (nebula-quasar)                             │
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
# Search with BM25 relevance ranking (auto-pages in terminal)
claude-convo search "serialization format"

# Limit results (outputs to scrollback, no pager)
claude-convo search "bm25 algorithm" --limit 10

# Search within a specific project
claude-convo search "error" --project myproject
```

### View statistics

```bash
# View overall stats
claude-convo stats

# View weekly stats
claude-convo stats --period week

# View stats for a specific project
claude-convo stats --project -Users-you-code-project
```

### Export conversations

```bash
# Export to Markdown (default: includes thinking & tools)
claude-convo export 0697

# Export with custom filename
claude-convo export 0697 --output my-conversation.md

# Hide thinking blocks in export
claude-convo export 0697 --no-thinking

# Hide tool usage in export
claude-convo export 0697 --no-tools
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
- **Pager** - Automatic paging for large outputs
- **Glob** - File pattern matching

## Roadmap

- [x] Full-text search with BM25 ranking
- [x] Export to Markdown  
- [x] Cost analysis in stats
- [x] Session naming system
- [x] Automatic pager support
- [ ] TUI mode for interactive browsing
- [ ] Integration with `claude --continue`

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built to scratch my own itch for better Claude Code conversation management. Inspired by the need to understand how Claude Code serializes our conversations!

### Session Names*

*The deterministic session naming system generates memorable names like "ancient-golden-dragon" from session IDs. With 600 adjectives × 600 adjectives × 1400 nouns = 504M possible combinations, you can expect ~46,000 unique sessions before hitting your first collision (1.6x better than the birthday paradox prediction!). If you somehow manage to hit this limit... reach out, I'd love to hear about your prolific Claude usage! 😄

## Author's Note

*Built with care by Claude (Anthropic) and a curious human who wanted to explore their conversation history. This tool emerged from our shared fascination with the serialization format - we spent a delightful session diving deep into JSONL parsing, debating Jiff vs Chrono (Jiff won!), and crafting a CLI that feels good to use. May it help you rediscover forgotten conversations and trace the evolution of your ideas.*

*Remember: your conversations are your digital memory - treat them with care!*

*— Claude & Human, July 2025*