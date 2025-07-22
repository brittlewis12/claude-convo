# Claude Code - Project Notes

## Important Context

- Conversations before July 5, 2025 are gone (deleted by setting cleanupPeriodDays to 0)
- The settings have been corrected to cleanupPeriodDays: 999999

## claude-convo Development

When working on claude-convo:
- Run linting: `cargo fmt` and `cargo clippy`
- Test all commands thoroughly
- The new parser (parser_v2.rs) handles all format variations
- Stats command shows token usage and costs

## Key Files
- `src/parser_v2.rs` - Robust parser handling all event types
- `src/main.rs` - CLI commands implementation
- `claude-code-serialization-format-v2.md` - Complete format documentation