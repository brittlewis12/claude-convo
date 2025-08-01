use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;
use std::path::{Path, PathBuf};
use std::fs;
use jiff::Timestamp;

mod parser;
mod parser_v2;
mod bm25;
mod display;
mod session_names;

#[derive(Parser)]
#[command(name = "claude-convo")]
#[command(about = "Browse and analyze Claude Code conversations", long_about = None)]
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
        
        /// Show thinking blocks
        #[arg(long)]
        show_thinking: bool,
        
        /// Limit number of messages
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    
    /// Search conversations  
    Search {
        /// Search query
        query: String,
        
        /// Filter by project
        #[arg(long)]
        project: Option<String>,
    },
    
    /// Show usage statistics
    Stats {
        /// Time period (day, week, month, all)
        #[arg(long, default_value = "week")]
        period: String,
    },
    
    /// Export conversation to Markdown
    Export {
        /// Session ID (can be partial)
        session: String,
        
        /// Output file path (optional, defaults to session-id.md)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Include thinking blocks
        #[arg(long)]
        include_thinking: bool,

        /// Include tool usage
        #[arg(long)]
        include_tools: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::List { project } => {
            list_command(project)?;
        }
        Commands::Show { session, show_thinking, limit } => {
            show_command(&session, show_thinking, limit)?;
        }
        Commands::Search { query, project } => {
            search_command(&query, project)?;
        }
        Commands::Stats { period } => {
            stats_command(&period)?;
        }
        Commands::Export { session, output, include_thinking, include_tools } => {
            export_command(&session, output, include_thinking, include_tools)?;
        }
    }
    
    Ok(())
}

fn list_command(project: Option<String>) -> Result<()> {
    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".claude/projects");
    
    if !claude_dir.exists() {
        println!("{}", "No Claude projects directory found at ~/.claude/projects".red());
        return Ok(());
    }
    
    if let Some(proj) = project {
        list_sessions(&claude_dir, &proj)?;
    } else {
        list_projects(&claude_dir)?;
    }
    Ok(())
}

fn list_projects(claude_dir: &Path) -> Result<()> {
    println!("{}", "Projects in ~/.claude/projects:".bright_blue().bold());
    println!();
    
    let mut projects = Vec::new();
    
    for entry in fs::read_dir(claude_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            // Count sessions and calculate size
            let (session_count, total_size, last_modified) = get_project_stats(&path)?;
            
            projects.push((name.to_string(), session_count, total_size, last_modified));
        }
    }
    
    // Sort by last modified
    projects.sort_by(|a, b| b.3.cmp(&a.3));
    
    for (name, count, size, last_mod) in projects {
        let size_mb = size as f64 / 1_000_000.0;
        let time_ago = format_time_ago(last_mod);
        
        println!("  {:<20} {} sessions   {:>6.1} MB   Last: {}",
            name.bright_white(),
            format!("{:>3}", count).cyan(),
            size_mb,
            time_ago.dimmed()
        );
    }
    
    Ok(())
}

fn list_sessions(claude_dir: &Path, project: &str) -> Result<()> {
    let project_dir = claude_dir.join(project);
    
    if !project_dir.exists() {
        println!("{}", format!("Project '{}' not found", project).red());
        return Ok(());
    }
    
    println!("{}", format!("Sessions in {}:", project).bright_blue().bold());
    println!();
    
    let mut sessions = Vec::new();
    let generator = session_names::SessionNameGenerator::new();
    
    for entry in fs::read_dir(&project_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            let metadata = fs::metadata(&path)?;
            let _modified = metadata.modified()?;
            let size = metadata.len();
            
            // Parse first line to get session info
            if let Ok(events) = parser_v2::parse_session_file(&path) {
                if let Some(first_event) = events.first() {
                    let msg_count = events.len();
                    let preview = get_first_user_message(&events);
                    
                    // Extract project type from the session file path
                    let project_type = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("unknown");
                    
                    // Generate a memorable name for this session
                    let session_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let name = generator.generate(session_id, project_type);
                    
                    sessions.push((
                        path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                        first_event.timestamp,
                        msg_count,
                        size,
                        preview,
                        name
                    ));
                }
            }
        }
    }
    
    // Sort by timestamp (newest first)
    sessions.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (id, timestamp, msg_count, size, preview, name) in sessions {
        let size_mb = size as f64 / 1_000_000.0;
        let local_time = timestamp.to_zoned(jiff::tz::TimeZone::system());
        let time_str = format!("{}", local_time.strftime("%Y-%m-%d %H:%M"));
        
        println!("  {} │ {:>4} msgs │ {:>6.1} MB │ {} │ {}",
            time_str.bright_white(),
            msg_count,
            size_mb,
            preview.dimmed(),
            name.bright_cyan()
        );
        println!("  {}", id.dimmed());
        println!();
    }
    
    Ok(())
}

fn get_project_stats(path: &Path) -> Result<(usize, u64, Timestamp)> {
    let mut count = 0;
    let mut total_size = 0u64;
    let mut last_modified = Timestamp::UNIX_EPOCH;
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            count += 1;
            let metadata = fs::metadata(&path)?;
            total_size += metadata.len();
            
            if let Ok(modified) = metadata.modified() {
                let duration = modified.duration_since(std::time::UNIX_EPOCH)?;
                let modified_time = Timestamp::new(duration.as_secs() as i64, duration.subsec_nanos() as i32)?;
                if modified_time > last_modified {
                    last_modified = modified_time;
                }
            }
        }
    }
    
    Ok((count, total_size, last_modified))
}

fn format_time_ago(time: Timestamp) -> String {
    let now = Timestamp::now();
    let span = now.since(time).unwrap_or_default();
    
    // Jiff 0.2 has better span handling
    let total_seconds = span.total(jiff::Unit::Second).unwrap_or(0.0) as i64;
    
    if total_seconds < 60 {
        "just now".to_string()
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        format!("{} minute{} ago", minutes, if minutes == 1 { "" } else { "s" })
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = total_seconds / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}

fn get_first_user_message(events: &[parser_v2::DisplayEvent]) -> String {
    for event in events {
        if event.role == "user" && !event.content.is_empty() {
            let preview = event.content.chars()
                .take(60)
                .collect::<String>()
                .replace('\n', " ");
            return format!("\"{}...\"", preview);
        }
    }
    "(no preview available)".to_string()
}

fn show_command(session: &str, show_thinking: bool, limit: usize) -> Result<()> {
    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".claude/projects");
    
    // Find the session file
    let session_path = find_session_file(&claude_dir, session)?;
    
    if let Some(path) = session_path {
        let events = parser_v2::parse_session_file(&path)?;
        
        if events.is_empty() {
            println!("{}", "No events found in session".red());
            return Ok(());
        }
        
        // Print header - use the actual session ID from the file
        let file_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
        display::print_session_header(
            file_id,
            &events
        );
        
        println!();
        
        // Display messages
        let display_limit = if limit == 0 { events.len() } else { limit.min(events.len()) };
        
        for (i, event) in events.iter().take(display_limit).enumerate() {
            display_event(event, show_thinking)?;
            
            if i < display_limit - 1 {
                println!();
            }
        }
        
        if events.len() > display_limit {
            println!();
            println!("{}", 
                format!("... {} more messages (use --limit 0 to show all)", 
                    events.len() - display_limit
                ).dimmed()
            );
        }
        
    } else {
        println!("{}", format!("Session '{}' not found", session).red());
    }
    
    Ok(())
}

fn find_session_file(claude_dir: &Path, session_id: &str) -> Result<Option<PathBuf>> {
    // Search all project directories
    for entry in fs::read_dir(claude_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Try to find by session ID first (as before)
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                
                if let Some(name) = file_path.file_stem().and_then(|s| s.to_str()) {
                    if name.starts_with(session_id) {
                        return Ok(Some(file_path));
                    }
                }
            }
            
            // If not found by ID, try to find by memorable name
            // This is a best-effort search that looks for a session with the given name
            // This could match multiple sessions, so we'll return the first match
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                
                if let Some(name) = file_path.file_stem().and_then(|s| s.to_str()) {
                    // Extract the project type from the path
                    let project_type = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                    
                    // Generate the expected name for this session
                    let generator = session_names::SessionNameGenerator::new();
                    let expected_name = generator.generate(name, project_type);
                    
                    // Check if the expected name matches the requested name
                    if expected_name == session_id {
                        return Ok(Some(file_path));
                    }
                }
            }
        }
    }
    
    Ok(None)
}

fn display_event(event: &parser_v2::DisplayEvent, show_thinking: bool) -> Result<()> {
    // Convert to local timezone
    let local_time = event.timestamp.to_zoned(jiff::tz::TimeZone::system());
    let time_str = format!("{}", local_time.strftime("%H:%M:%S"));
    
    // Display based on role
    match event.role.as_str() {
        "user" => {
            println!("{} {} {}", 
                format!("[{}]", time_str).dimmed(),
                "USER".bright_cyan().bold(),
                "═".repeat(50).bright_cyan()
            );
            println!("{}", event.content);
        }
        "assistant" => {
            println!("{} {} {}", 
                format!("[{}]", time_str).dimmed(),
                "ASSISTANT".bright_green().bold(),
                "═".repeat(45).bright_green()
            );
            
            if !event.content.is_empty() {
                println!("{}", event.content);
            }
            
            if let Some(thinking) = &event.thinking {
                if show_thinking {
                    println!();
                    println!("{}", "[💭 Thinking]".bright_magenta());
                    println!("{}", thinking.dimmed());
                    println!();
                } else {
                    println!("{}", 
                        format!("[💭 Thinking - {} chars] (use --show-thinking to expand)", 
                            thinking.len()
                        ).dimmed()
                    );
                }
            }
            
            if let Some(tool_info) = &event.tool_info {
                println!();
                println!("{} {} {}", 
                    "[TOOL]".bright_blue().bold(),
                    tool_info.name.bright_white(),
                    format!("({})", tool_info.id).dimmed()
                );
                
                // Pretty print JSON input
                if let Ok(pretty) = serde_json::to_string_pretty(&tool_info.input) {
                    for line in pretty.lines() {
                        println!("  {}", line.dimmed());
                    }
                }
            }
            
            if let Some(usage) = &event.usage {
                println!();
                println!("{}", 
                    format!("Tokens: {} → {} | Model: {}", 
                        usage.input_tokens, 
                        usage.output_tokens,
                        event.model.as_deref().unwrap_or("unknown")
                    ).dimmed()
                );
            }
        }
        role if role.starts_with("system:") => {
            println!("{} {} {}", 
                format!("[{}]", time_str).dimmed(),
                "SYSTEM".bright_yellow().bold(),
                "═".repeat(47).bright_yellow()
            );
            println!("{}", event.content.dimmed());
        }
        _ => {
            println!("{} {} {}", 
                format!("[{}]", time_str).dimmed(),
                event.role.to_uppercase().bright_white(),
                "═".repeat(50).white()
            );
            println!("{}", event.content);
        }
    }
    
    Ok(())
}

fn search_command(query: &str, project: Option<String>) -> Result<()> {
    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".claude/projects");
    
    if !claude_dir.exists() {
        println!("{}", "No Claude projects directory found".red());
        return Ok(());
    }
    
    println!("{}", format!("Searching for: \"{}\"", query).bright_yellow().bold());
    println!();
    
    let mut total_matches = 0;
    let mut results = Vec::new();
    
    // Determine which projects to search
    let projects_to_search = if let Some(proj) = project {
        vec![claude_dir.join(&proj)]
    } else {
        // Search all projects
        let mut projects = Vec::new();
        for entry in fs::read_dir(&claude_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                projects.push(path);
            }
        }
        projects
    };
    
    // Search each project
    for project_path in projects_to_search {
        let project_name = project_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        // Search all JSONL files in the project
        for entry in fs::read_dir(&project_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                if let Ok(matches) = search_in_session(&path, query) {
                    if !matches.is_empty() {
                        total_matches += matches.len();
                        let session_id = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown");
                        results.push((project_name.to_string(), session_id.to_string(), matches));
                    }
                }
            }
        }
    }
    
    // Display results
    if results.is_empty() {
        println!("{}", "No matches found".dimmed());
    } else {
        println!("{}", 
            format!("Found {} match{} across {} session{}:", 
                total_matches, 
                if total_matches == 1 { "" } else { "es" },
                results.len(),
                if results.len() == 1 { "" } else { "s" }
            ).green()
        );
        println!();
        
        for (project, session, matches) in results {
            println!("{}/{} {}", 
                project.bright_white(), 
                &session[..8.min(session.len())].dimmed(),
                format!("[{}]", matches[0].timestamp.to_zoned(jiff::tz::TimeZone::system()).strftime("%Y-%m-%d %H:%M:%S")).dimmed()
            );
            
            for match_info in matches.iter().take(3) {
                println!("  {} {}", 
                    match match_info.role.as_str() {
                        "user" => "USER:".bright_cyan(),
                        "assistant" => "ASSISTANT:".bright_green(),
                        _ => "OTHER:".dimmed()
                    },
                    highlight_match(&match_info.content, query)
                );
            }
            
            if matches.len() > 3 {
                println!("  {} more matches in this session", matches.len() - 3);
            }
            println!();
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct SearchMatch {
    timestamp: Timestamp,
    role: String,
    content: String,
    score: f64,
}

fn search_in_session(path: &Path, query: &str) -> Result<Vec<SearchMatch>> {
    let events = parser_v2::parse_session_file(path)?;
    
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    
    // Build corpus for BM25
    let mut documents = Vec::new();
    let mut event_indices = Vec::new();
    
    for (idx, event) in events.iter().enumerate() {
        let mut search_content = event.content.clone();
        
        // Include thinking in search
        if let Some(thinking) = &event.thinking {
            search_content.push_str("\n");
            search_content.push_str(thinking);
        }
        
        // Include tool info in search
        if let Some(tool_info) = &event.tool_info {
            search_content.push_str(&format!("\n[Tool: {}]", tool_info.name));
        }
        
        documents.push(search_content);
        event_indices.push(idx);
    }
    
    // Create BM25 scorer with standard parameters
    let bm25 = crate::bm25::BM25::new(&documents, 1.2, 0.75);
    
    // Score all documents
    let mut scored_matches = Vec::new();
    
    for (doc_idx, (doc, event_idx)) in documents.iter().zip(event_indices.iter()).enumerate() {
        let score = bm25.score(query, doc);
        
        // Only include documents with positive scores
        if score > 0.0 {
            let event = &events[*event_idx];
            // For snippet, try to find the first matching query term
            let query_words: Vec<&str> = query.split_whitespace().collect();
            let snippet = extract_snippet_with_words(doc, &query_words, 100);
            
            scored_matches.push(SearchMatch {
                timestamp: event.timestamp,
                role: event.role.clone(),
                content: snippet,
                score,
            });
        }
    }
    
    // Sort by score (highest first)
    scored_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    
    Ok(scored_matches)
}

fn extract_snippet_with_words(text: &str, query_words: &[&str], context_chars: usize) -> String {
    let lower_text = text.to_lowercase();
    
    // Try to find the first occurrence of any query word
    let mut best_pos = None;
    for word in query_words {
        if let Some(pos) = lower_text.find(&word.to_lowercase()) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
            }
        }
    }
    
    if let Some(pos) = best_pos {
        // Find safe char boundaries
        let mut start = pos.saturating_sub(context_chars);
        let mut end = (pos + context_chars).min(text.len());
        
        // Adjust to char boundaries
        while start > 0 && !text.is_char_boundary(start) {
            start -= 1;
        }
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        
        let snippet = &text[start..end];
        format!("...{}...", snippet.trim())
    } else {
        // No match found, return beginning of text
        let mut end = context_chars.min(text.len());
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        format!("{}...", &text[..end].trim())
    }
}

fn extract_snippet(text: &str, query: &str, context_chars: usize) -> String {
    let lower_text = text.to_lowercase();
    
    if let Some(byte_pos) = lower_text.find(query) {
        // Convert byte position to char position for safe slicing
        let chars: Vec<char> = text.chars().collect();
        let char_pos = text[..byte_pos].chars().count();
        
        let start_char = char_pos.saturating_sub(context_chars);
        let end_char = (char_pos + query.len() + context_chars).min(chars.len());
        
        let mut snippet = String::new();
        if start_char > 0 {
            snippet.push_str("...");
        }
        
        let slice: String = chars[start_char..end_char].iter().collect();
        snippet.push_str(&slice.replace('\n', " "));
        
        if end_char < chars.len() {
            snippet.push_str("...");
        }
        snippet
    } else {
        text.chars().take(context_chars * 2).collect::<String>().replace('\n', " ")
    }
}

fn highlight_match(text: &str, query: &str) -> String {
    let mut result = text.to_string();
    let lower_text = text.to_lowercase();
    
    // Split query into words and filter out very short words
    let query_words: Vec<String> = query.split_whitespace()
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 3)  // Skip very short words like "a", "is", "to"
        .collect();
    
    // Track positions we've already highlighted to avoid overlaps
    let mut highlighted_ranges: Vec<(usize, usize)> = Vec::new();
    
    for word in &query_words {
        let mut search_pos = 0;
        while let Some(rel_pos) = lower_text[search_pos..].find(word) {
            let byte_pos = search_pos + rel_pos;
            
            // Check for word boundaries (don't highlight partial matches)
            let at_word_start = byte_pos == 0 || 
                lower_text.chars().nth(byte_pos.saturating_sub(1))
                    .map(|c| !c.is_alphanumeric()).unwrap_or(true);
            let at_word_end = byte_pos + word.len() >= lower_text.len() ||
                lower_text.chars().nth(byte_pos + word.len())
                    .map(|c| !c.is_alphanumeric()).unwrap_or(true);
            
            if at_word_start && at_word_end {
                // Check if this position overlaps with already highlighted text
                let overlaps = highlighted_ranges.iter()
                    .any(|(start, end)| byte_pos < *end && byte_pos + word.len() > *start);
                
                if !overlaps {
                    // Find char boundaries for safe slicing
                    let mut start = byte_pos;
                    while start > 0 && !text.is_char_boundary(start) {
                        start -= 1;
                    }
                    
                    let mut end = byte_pos + word.len();
                    while end < text.len() && !text.is_char_boundary(end) {
                        end += 1;
                    }
                    
                    highlighted_ranges.push((start, end));
                }
            }
            
            search_pos = byte_pos + 1;  // Move forward by 1 to find all occurrences
        }
    }
    
    // Sort ranges by start position (descending) to apply highlights from end to start
    highlighted_ranges.sort_by(|a, b| b.0.cmp(&a.0));
    
    // Apply highlights
    for (start, end) in highlighted_ranges {
        let before = &result[..start];
        let matched = &result[start..end];
        let after = &result[end..];
        
        result = format!("{}{}{}", before, matched.on_yellow().black(), after);
    }
    
    result
}

fn stats_command(period: &str) -> Result<()> {
    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".claude/projects");
    
    if !claude_dir.exists() {
        println!("{}", "No Claude projects directory found".red());
        return Ok(());
    }
    
    // Calculate period boundaries
    let now = Timestamp::now();
    let period_start = match period {
        "day" => {
            let now_zoned = now.to_zoned(jiff::tz::TimeZone::system());
            let day_ago_zoned = now_zoned.checked_sub(jiff::Span::new().days(1)).unwrap_or(now_zoned);
            day_ago_zoned.timestamp()
        },
        "week" => {
            let now_zoned = now.to_zoned(jiff::tz::TimeZone::system());
            let week_ago_zoned = now_zoned.checked_sub(jiff::Span::new().days(7)).unwrap_or(now_zoned);
            week_ago_zoned.timestamp()
        },
        "month" => {
            let now_zoned = now.to_zoned(jiff::tz::TimeZone::system());
            let month_ago_zoned = now_zoned.checked_sub(jiff::Span::new().days(30)).unwrap_or(now_zoned);
            month_ago_zoned.timestamp()
        },
        "all" => Timestamp::UNIX_EPOCH,
        _ => {
            println!("{}", "Invalid period. Use: day, week, month, or all".red());
            return Ok(());
        }
    };
    
    // Collect statistics
    let mut total_sessions = 0;
    let mut total_messages = 0;
    let mut total_input_tokens = 0u64;
    let mut total_output_tokens = 0u64;
    let mut tool_usage: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut model_usage: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut daily_activity: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_duration = jiff::Span::new();
    
    // Scan all projects
    for entry in fs::read_dir(&claude_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                
                if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    if let Ok(events) = parser_v2::parse_session_file(&file_path) {
                        if events.is_empty() {
                            continue;
                        }
                        
                        // Check if session is within period
                        let session_start = events.first().unwrap().timestamp;
                        if session_start < period_start {
                            continue;
                        }
                        
                        total_sessions += 1;
                        
                        // Only count messages within the period
                        let messages_in_period = events.iter()
                            .filter(|e| e.timestamp >= period_start)
                            .count();
                        total_messages += messages_in_period;
                        
                        // Calculate session duration
                        if events.len() > 1 {
                            let session_end = events.last().unwrap().timestamp;
                            if let Ok(duration) = session_end.since(session_start) {
                                total_duration = total_duration.checked_add(duration).unwrap_or(total_duration);
                            }
                        }
                        
                        // Count daily activity
                        let day_key = session_start.to_zoned(jiff::tz::TimeZone::system())
                            .strftime("%a").to_string();
                        *daily_activity.entry(day_key).or_insert(0) += 1;
                        
                        // Process each event
                        for event in &events {
                            // Skip events outside the period
                            if event.timestamp < period_start {
                                continue;
                            }
                            
                            if event.role == "assistant" {
                                // Count tokens
                                if let Some(usage) = &event.usage {
                                    total_input_tokens += usage.input_tokens as u64;
                                    total_output_tokens += usage.output_tokens as u64;
                                }
                                
                                // Count model usage
                                if let Some(model) = &event.model {
                                    *model_usage.entry(model.clone()).or_insert(0) += 1;
                                }
                                
                                // Count tool usage
                                if let Some(tool_info) = &event.tool_info {
                                    *tool_usage.entry(tool_info.name.clone()).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Display statistics
    println!();
    println!("{}", format!("Claude Code Usage Statistics ({})", 
        match period {
            "day" => "Last 24 hours",
            "week" => "Last 7 days",
            "month" => "Last 30 days",
            "all" => "All time",
            _ => period
        }
    ).bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_cyan());
    println!();
    
    // Session stats
    println!("{}:", "Sessions".bright_white());
    println!("  Total:          {}", total_sessions);
    if total_sessions > 0 {
        let avg_messages = total_messages / total_sessions;
        println!("  Avg messages:   {} per session", avg_messages);
        
        let total_minutes = total_duration.total(jiff::Unit::Minute).unwrap_or(0.0) as i64;
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        println!("  Total time:     {}h {}m", hours, minutes);
        
        if total_sessions > 1 {
            let avg_duration = total_minutes / total_sessions as i64;
            println!("  Avg duration:   {} minutes", avg_duration);
        }
    }
    println!();
    
    // Token usage
    println!("{}:", "Token Usage".bright_white());
    println!("  Input:          {:>10} tokens", format_number(total_input_tokens));
    println!("  Output:         {:>10} tokens", format_number(total_output_tokens));
    println!("  Total:          {:>10} tokens", format_number(total_input_tokens + total_output_tokens));
    println!();
    
    // Cost estimation
    println!("{}:", "Estimated Costs".bright_white());
    let input_cost = (total_input_tokens as f64 * 0.015) / 1000.0;
    let output_cost = (total_output_tokens as f64 * 0.075) / 1000.0;
    let total_cost = input_cost + output_cost;
    println!("  Input:          ${:>8.2}", input_cost);
    println!("  Output:         ${:>8.2}", output_cost);
    println!("  Total:          ${:>8.2}", total_cost);
    if total_sessions > 0 {
        println!("  Per session:    ${:>8.2}", total_cost / total_sessions as f64);
    }
    println!();
    
    // Tool usage
    if !tool_usage.is_empty() {
        println!("{}:", "Most Used Tools".bright_white());
        let mut tools: Vec<_> = tool_usage.iter().collect();
        tools.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (i, (tool, count)) in tools.iter().take(10).enumerate() {
            println!("  {:2}. {:<20} {} calls", i + 1, tool, count);
        }
        println!();
    }
    
    // Model usage
    if !model_usage.is_empty() {
        println!("{}:", "Model Usage".bright_white());
        let mut models: Vec<_> = model_usage.iter().collect();
        models.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (model, count) in models {
            println!("  {:<40} {} messages", model, count);
        }
        println!();
    }
    
    // Daily activity
    if !daily_activity.is_empty() {
        println!("{}:", "Activity by Day".bright_white());
        let days = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let max_activity = *daily_activity.values().max().unwrap_or(&1) as f64;
        
        for day in days {
            let count = daily_activity.get(day).copied().unwrap_or(0);
            let bar_width = ((count as f64 / max_activity) * 20.0) as usize;
            let bar = "█".repeat(bar_width);
            let padding = "░".repeat(20 - bar_width);
            let percentage = (count as f64 / total_sessions as f64 * 100.0) as usize;
            
            println!("  {} {}{} {:>3}%", 
                day, 
                bar.bright_green(), 
                padding.dimmed(),
                percentage
            );
        }
    }
    
    Ok(())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn export_command(session: &str, output: Option<String>, include_thinking: bool, include_tools: bool) -> Result<()> {
    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".claude/projects");
    
    // Find the session file
    let session_path = find_session_file(&claude_dir, session)?;
    
    if let Some(path) = session_path {
        let events = parser_v2::parse_session_file(&path)?;
        
        if events.is_empty() {
            println!("{}", "No events found in session".red());
            return Ok(());
        }
        
        // Determine output filename
        let output_path = if let Some(out) = output {
            PathBuf::from(out)
        } else {
            PathBuf::from(format!("{}.md", session))
        };
        
        // Generate Markdown content
        let mut content = String::new();
        
        // Add header
        content.push_str("# Claude Code Conversation\n\n");
        
        // Add session metadata
        let first = &events[0];
        let last = &events[events.len() - 1];
        let duration = last.timestamp.since(first.timestamp).unwrap_or_default();
        
        content.push_str(&format!("**Session ID**: {}\n", session));
        content.push_str(&format!("**Date**: {}\n", first.timestamp.to_zoned(jiff::tz::TimeZone::system()).strftime("%Y-%m-%d %H:%M:%S %Z")));
        content.push_str(&format!("**Duration**: {}m {}s\n", 
            duration.total(jiff::Unit::Minute).unwrap_or(0.0) as i64,
            duration.total(jiff::Unit::Second).unwrap_or(0.0) as i64 % 60
        ));
        content.push_str(&format!("**Messages**: {}\n", events.len()));
        
        // Calculate token usage
        let mut total_input = 0u32;
        let mut total_output = 0u32;
        for event in &events {
            if event.role == "assistant" {
                if let Some(usage) = &event.usage {
                    total_input += usage.input_tokens;
                    total_output += usage.output_tokens;
                }
            }
        }
        
        if total_input > 0 || total_output > 0 {
            content.push_str(&format!("**Tokens**: {} → {} (${:.2})\n", 
                format_number(total_input as u64),
                format_number(total_output as u64),
                (total_input as f64 * 0.015 + total_output as f64 * 0.075) / 1000.0
            ));
        }
        
        content.push_str("\n---\n\n");
        
        // Add conversation
        for event in &events {
            let time = event.timestamp.to_zoned(jiff::tz::TimeZone::system());
            
            match event.role.as_str() {
                "user" => {
                    content.push_str(&format!("## User [{}]\n\n", time.strftime("%H:%M:%S")));
                    content.push_str(&event.content);
                    content.push_str("\n\n");
                }
                "assistant" => {
                    content.push_str(&format!("## Assistant [{}]", time.strftime("%H:%M:%S")));
                    
                    if let Some(model) = &event.model {
                        content.push_str(&format!(" ({})", model));
                    }
                    content.push_str("\n\n");
                    
                    // Add content
                    if !event.content.is_empty() {
                        content.push_str(&event.content);
                        content.push_str("\n\n");
                    }
                    
                    // Add thinking if requested
                    if include_thinking {
                        if let Some(thinking) = &event.thinking {
                            content.push_str("<details>\n<summary>💭 Thinking</summary>\n\n");
                            content.push_str(thinking);
                            content.push_str("\n\n</details>\n\n");
                        }
                    } else if event.thinking.is_some() {
                        content.push_str("*[Thinking block omitted - use --include-thinking to include]*\n\n");
                    }
                    
                    // Add tool use
                    if include_tools {
                        if let Some(tool_info) = &event.tool_info {
                            content.push_str(&format!("### Tool: {}\n\n", tool_info.name));
                            content.push_str("```json\n");
                            if let Ok(pretty) = serde_json::to_string_pretty(&tool_info.input) {
                                content.push_str(&pretty);
                            }
                            content.push_str("\n```\n\n");
                        }
                    }
                    
                    // Add token usage if available
                    if let Some(usage) = &event.usage {
                        content.push_str(&format!("*Tokens: {} → {}*\n\n", 
                            usage.input_tokens, 
                            usage.output_tokens
                        ));
                    }
                }
                role if role.starts_with("system:") => {
                    content.push_str(&format!("## System [{}]\n\n", time.strftime("%H:%M:%S")));
                    content.push_str(&format!("> {}\n\n", event.content));
                }
                _ => {
                    content.push_str(&format!("## {} [{}]\n\n", event.role, time.strftime("%H:%M:%S")));
                    content.push_str(&event.content);
                    content.push_str("\n\n");
                }
            }
        }
        
        // Write to file
        std::fs::write(&output_path, &content)?;
        
        println!("{}", format!("✅ Exported to: {}", output_path.display()).green());
        println!("   {} messages", events.len());
        println!("   {} bytes", content.len());
        
    } else {
        println!("{}", format!("Session '{}' not found", session).red());
    }
    
    Ok(())
}
