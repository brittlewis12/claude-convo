# Claude Convo - Implementation Roadmap

## Quick Start Development Plan

### Week 1: Foundation Sprint

#### Day 1-2: Project Setup & Core Parsing
```bash
cargo new claude-convo
cd claude-convo
```

**Cargo.toml:**
```toml
[package]
name = "claude-convo"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.26"
crossterm = "0.27"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.36", features = ["full"] }
anyhow = "1.0"
dirs = "5.0"
glob = "0.3"
```

**Key Tasks:**
- ✓ JSONL parser implementation (see POC)
- Session discovery logic
- Event chain builder
- Basic error handling

#### Day 3-4: State Management & Architecture
```rust
// src/state.rs
pub struct AppState {
    projects: Vec<Project>,
    current_project: Option<usize>,
    current_session: Option<Session>,
    view_mode: ViewMode,
    search_query: Option<String>,
}

pub enum ViewMode {
    ProjectList,
    SessionList,
    ConversationView,
    SearchResults,
}

// Event-driven updates
pub enum AppEvent {
    KeyPress(KeyEvent),
    SessionLoaded(Session),
    SearchComplete(Vec<SearchResult>),
    Error(String),
}
```

#### Day 5-7: Basic TUI Layout
```rust
// src/ui/mod.rs
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(f.size());

    draw_sidebar(f, app, chunks[0]);
    draw_main_content(f, app, chunks[1]);
}
```

### Week 2: Core Features

#### Priority Features Order:
1. **Session List Navigation** - Browse projects and sessions
2. **Message Rendering** - Basic display with proper formatting
3. **Keyboard Shortcuts** - Vim-like navigation
4. **Syntax Highlighting** - Code blocks with syntect

### Performance Optimizations

#### Large File Handling
```rust
// Streaming parser for large JSONL files
pub struct StreamingSessionReader {
    reader: BufReader<File>,
    event_index: HashMap<String, u64>, // UUID -> byte offset
}

impl StreamingSessionReader {
    pub fn get_event(&mut self, uuid: &str) -> Result<Event> {
        if let Some(offset) = self.event_index.get(uuid) {
            self.reader.seek(SeekFrom::Start(*offset))?;
            // Read and parse single line
        }
    }
}
```

#### Search Index
```rust
// Build search index on first run
pub struct SearchIndex {
    sessions: HashMap<String, SessionMeta>,
    content_index: tantivy::Index, // Full-text search
}
```

### Testing Strategy

1. **Unit Tests** - Parser, state management
2. **Integration Tests** - File system operations
3. **TUI Tests** - Using ratatui's test backend
4. **Performance Tests** - Large file handling

### Release Plan

#### v0.1.0 - MVP (Week 2)
- Basic browsing functionality
- Project/session navigation
- Simple message display

#### v0.2.0 - Rich Viewing (Week 4)
- Syntax highlighting
- Tool call formatting
- Thinking block toggles

#### v0.3.0 - Search & Export (Week 6)
- Full-text search
- Export to Markdown/JSON
- Configuration file support

### Distribution

```bash
# GitHub Release with pre-built binaries
- claude-convo-x86_64-apple-darwin.tar.gz
- claude-convo-x86_64-unknown-linux-gnu.tar.gz  
- claude-convo-x86_64-pc-windows-msvc.zip

# Homebrew (macOS/Linux)
brew tap user/claude-convo
brew install claude-convo

# Cargo install
cargo install claude-convo
```

## Development Workflow

```bash
# Run in development
cargo run -- --project opencode

# Run tests
cargo test

# Build release
cargo build --release

# Generate test data
./scripts/generate-test-sessions.sh
```

## Risk Mitigation

1. **Large Memory Usage**
   - Solution: Streaming parser, lazy loading
   
2. **Complex Event Chains**
   - Solution: Build index on first load, cache

3. **Cross-platform Terminal Issues**
   - Solution: Stick to crossterm basics, test on all platforms

4. **Performance with Many Sessions**
   - Solution: Background indexing, SQLite cache

## Metrics for Success

- [ ] Load 10,000 event session in <500ms
- [ ] Search across 100 sessions in <100ms  
- [ ] Memory usage <100MB for typical use
- [ ] 60fps UI responsiveness
- [ ] Zero-config startup