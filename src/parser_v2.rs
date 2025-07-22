use serde::{Deserialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use jiff::Timestamp;
use anyhow::Result;

// Top-level enum for all JSONL entry types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SessionEntry {
    Summary {
        summary: String,
        #[serde(rename = "leafUuid")]
        leaf_uuid: Option<String>,
        timestamp: Option<Timestamp>,
    },
    User {
        #[serde(flatten)]
        event: UserEvent,
    },
    Assistant {
        #[serde(flatten)]
        event: AssistantEvent,
    },
    System {
        content: String,
        level: Option<String>,
        #[serde(flatten)]
        metadata: EventMetadata,
    },
}

// Common metadata fields
#[derive(Debug, Deserialize)]
pub struct EventMetadata {
    pub uuid: String,
    #[serde(rename = "parentUuid")]
    pub parent_uuid: Option<String>,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub timestamp: Timestamp,
    pub cwd: String,
    #[serde(rename = "gitBranch")]
    pub git_branch: Option<String>,
    #[serde(rename = "userType")]
    pub user_type: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
    #[serde(rename = "isSidechain")]
    pub is_sidechain: Option<bool>,
    #[serde(rename = "isMeta")]
    pub is_meta: Option<bool>,
}

// User event structure
#[derive(Debug, Deserialize)]
pub struct UserEvent {
    #[serde(flatten)]
    pub metadata: EventMetadata,
    pub message: UserMessage,
    #[serde(rename = "toolUseResult")]
    pub tool_use_result: Option<ToolUseResult>,
    #[serde(rename = "isCompactSummary")]
    pub is_compact_summary: Option<bool>,
}

// Assistant event structure
#[derive(Debug, Deserialize)]
pub struct AssistantEvent {
    #[serde(flatten)]
    pub metadata: EventMetadata,
    pub message: AssistantMessage,
    #[serde(rename = "isApiErrorMessage")]
    pub is_api_error_message: Option<bool>,
}

// User message variants
#[derive(Debug, Deserialize)]
pub struct UserMessage {
    pub role: String,
    pub content: UserContent,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    Text(String),
    Blocks(Vec<UserContentBlock>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    Text { 
        text: String 
    },
    Image { 
        source: ImageSource 
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: Option<bool>,
    },
}

#[derive(Debug, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

// Assistant message structure
#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<AssistantContentBlock>,
    pub usage: Option<TokenUsage>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssistantContentBlock {
    Text { 
        text: String 
    },
    Thinking { 
        thinking: String,
        signature: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

// Tool use result (extensive metadata from tool executions)
#[derive(Debug, Deserialize)]
pub struct ToolUseResult {
    #[serde(rename = "type")]
    pub result_type: Option<String>,
    pub content: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub code: Option<i32>,
    #[serde(rename = "durationMs")]
    pub duration_ms: Option<u64>,
    pub interrupted: Option<bool>,
    pub truncated: Option<bool>,
    // File operations
    pub file: Option<FileResult>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub edits: Option<Vec<EditResult>>,
    // Search/query results
    pub query: Option<String>,
    pub results: Option<Vec<Value>>,
    // Todo operations
    #[serde(rename = "newTodos")]
    pub new_todos: Option<Vec<Todo>>,
    #[serde(rename = "oldTodos")]
    pub old_todos: Option<Vec<Todo>>,
    // ... many more fields possible
}

#[derive(Debug, Deserialize)]
pub struct FileResult {
    pub content: Option<String>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    #[serde(rename = "numLines")]
    pub num_lines: Option<usize>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EditResult {
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct Todo {
    pub id: String,
    pub content: String,
    pub status: String,
    pub priority: String,
}

// Token usage statistics
#[derive(Debug, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
    pub service_tier: Option<String>,
}

// Simplified event structure for display
pub struct DisplayEvent {
    pub timestamp: Timestamp,
    pub role: String,
    pub content: String,
    pub tool_info: Option<ToolInfo>,
    pub thinking: Option<String>,
    pub usage: Option<TokenUsage>,
    pub model: Option<String>,
}

pub struct ToolInfo {
    pub name: String,
    pub id: String,
    pub input: Value,
}

// Parse a session file into display events
pub fn parse_session_file(path: &Path) -> Result<Vec<DisplayEvent>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        match serde_json::from_str::<SessionEntry>(&line) {
            Ok(entry) => {
                if let Some(event) = convert_to_display_event(entry) {
                    events.push(event);
                }
            }
            Err(_) => {
                // Skip unparseable lines silently
            }
        }
    }

    Ok(events)
}

fn convert_to_display_event(entry: SessionEntry) -> Option<DisplayEvent> {
    match entry {
        SessionEntry::User { event } => {
            let content = match &event.message.content {
                UserContent::Text(text) => text.clone(),
                UserContent::Blocks(blocks) => {
                    blocks.iter()
                        .filter_map(|block| match block {
                            UserContentBlock::Text { text } => Some(text.clone()),
                            UserContentBlock::ToolResult { content, .. } => Some(content.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };
            
            Some(DisplayEvent {
                timestamp: event.metadata.timestamp,
                role: "user".to_string(),
                content,
                tool_info: None,
                thinking: None,
                usage: None,
                model: None,
            })
        }
        SessionEntry::Assistant { event } => {
            let mut content = String::new();
            let mut tool_info = None;
            let mut thinking = None;
            
            for block in &event.message.content {
                match block {
                    AssistantContentBlock::Text { text } => {
                        if !content.is_empty() {
                            content.push('\n');
                        }
                        content.push_str(text);
                    }
                    AssistantContentBlock::Thinking { thinking: t, .. } => {
                        thinking = Some(t.clone());
                    }
                    AssistantContentBlock::ToolUse { name, id, input } => {
                        tool_info = Some(ToolInfo {
                            name: name.clone(),
                            id: id.clone(),
                            input: input.clone(),
                        });
                    }
                }
            }
            
            Some(DisplayEvent {
                timestamp: event.metadata.timestamp,
                role: "assistant".to_string(),
                content,
                tool_info,
                thinking,
                usage: event.message.usage,
                model: Some(event.message.model),
            })
        }
        SessionEntry::System { content, level, metadata } => {
            Some(DisplayEvent {
                timestamp: metadata.timestamp,
                role: format!("system:{}", level.as_deref().unwrap_or("info")),
                content,
                tool_info: None,
                thinking: None,
                usage: None,
                model: None,
            })
        }
        SessionEntry::Summary { .. } => {
            // Skip summary entries for display
            None
        }
    }
}