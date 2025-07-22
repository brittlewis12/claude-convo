// Proof of Concept: Claude Conversation Parser
// This demonstrates the core parsing logic for the TUI browser

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
struct Event {
    uuid: String,
    #[serde(rename = "parentUuid")]
    parent_uuid: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: String,
    timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    event_type: String,
    message: Message,
    #[serde(rename = "userType")]
    user_type: String,
    cwd: String,
    #[serde(rename = "gitBranch")]
    git_branch: Option<String>,
    version: String,
    #[serde(rename = "requestId")]
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Message {
    User {
        role: String,
        content: UserContent,
    },
    Assistant {
        id: String,
        #[serde(rename = "type")]
        msg_type: String,
        role: String,
        model: String,
        content: Vec<ContentBlock>,
        usage: Option<TokenUsage>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum UserContent {
    Text(String),
    ToolResults(Vec<ToolResult>),
}

#[derive(Debug, Deserialize)]
struct ToolResult {
    tool_use_id: String,
    #[serde(rename = "type")]
    result_type: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "thinking")]
    Thinking { thinking: String, signature: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

#[derive(Debug, Deserialize)]
struct TokenUsage {
    input_tokens: u32,
    output_tokens: u32,
    cache_creation_input_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

fn parse_session_file(path: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        match serde_json::from_str::<Event>(&line) {
            Ok(event) => events.push(event),
            Err(e) => eprintln!("Failed to parse line: {}", e),
        }
    }

    Ok(events)
}

fn format_event_summary(event: &Event) -> String {
    match &event.message {
        Message::User { content, .. } => {
            match content {
                UserContent::Text(text) => {
                    let preview = text.chars().take(80).collect::<String>();
                    format!("[USER] {}", preview)
                }
                UserContent::ToolResults(results) => {
                    format!("[TOOL RESULT] {} results", results.len())
                }
            }
        }
        Message::Assistant { content, usage, .. } => {
            let mut summary = String::new();
            let mut has_text = false;
            let mut has_thinking = false;
            let mut tool_count = 0;

            for block in content {
                match block {
                    ContentBlock::Text { .. } => has_text = true,
                    ContentBlock::Thinking { .. } => has_thinking = true,
                    ContentBlock::ToolUse { .. } => tool_count += 1,
                }
            }

            if has_text {
                summary.push_str("[ASSISTANT]");
            }
            if has_thinking {
                summary.push_str(" [THINKING]");
            }
            if tool_count > 0 {
                summary.push_str(&format!(" [TOOLS: {}]", tool_count));
            }

            if let Some(usage) = usage {
                summary.push_str(&format!(
                    " ({}→{} tokens)",
                    usage.input_tokens, usage.output_tokens
                ));
            }

            summary
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage
    let session_path = Path::new("/Users/tito/.claude/projects/-Users-tito-code-opencode/0697fd58-7182-4faa-91b4-c76dded9374b.jsonl");
    
    println!("Parsing session file: {:?}", session_path);
    let events = parse_session_file(session_path)?;
    
    println!("\nFound {} events", events.len());
    println!("\nFirst 10 events:");
    println!("{:-<80}", "");
    
    for (i, event) in events.iter().take(10).enumerate() {
        println!(
            "{:2}. {} | {}",
            i + 1,
            event.timestamp.format("%H:%M:%S"),
            format_event_summary(event)
        );
    }

    // Calculate session statistics
    let mut total_input_tokens = 0;
    let mut total_output_tokens = 0;
    let mut message_count = 0;
    
    for event in &events {
        if let Message::Assistant { usage: Some(usage), .. } = &event.message {
            total_input_tokens += usage.input_tokens;
            total_output_tokens += usage.output_tokens;
            message_count += 1;
        }
    }

    println!("\n{:-<80}", "");
    println!("Session Statistics:");
    println!("  Total messages: {}", events.len());
    println!("  Assistant messages: {}", message_count);
    println!("  Total input tokens: {}", total_input_tokens);
    println!("  Total output tokens: {}", total_output_tokens);
    println!("  Estimated cost: ${:.4}", 
        (total_input_tokens as f64 * 0.015 + total_output_tokens as f64 * 0.075) / 1000.0
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event() {
        let json = r#"{
            "uuid": "test-uuid",
            "parentUuid": null,
            "sessionId": "test-session",
            "timestamp": "2025-07-22T00:49:44.544Z",
            "type": "user",
            "userType": "external",
            "cwd": "/test",
            "version": "1.0.0",
            "message": {
                "role": "user",
                "content": "Hello, Claude!"
            }
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();
        assert_eq!(event.uuid, "test-uuid");
        assert_eq!(event.event_type, "user");
    }
}