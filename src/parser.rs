use serde::{Deserialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use jiff::Timestamp;
use anyhow::Result;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SessionEntry {
    #[serde(rename = "summary")]
    Summary {
        #[allow(dead_code)]
        summary: String,
        #[allow(dead_code)]
        timestamp: Option<Timestamp>,
    },
    #[serde(rename = "user")]
    User(Event),
    #[serde(rename = "assistant")]
    Assistant(Event),
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[allow(dead_code)]
    pub uuid: String,
    #[serde(rename = "parentUuid")]
    #[allow(dead_code)]
    pub parent_uuid: Option<String>,
    #[serde(rename = "sessionId")]
    #[allow(dead_code)]
    pub session_id: String,
    pub timestamp: Timestamp,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub event_type: String,
    pub message: Message,
    #[serde(rename = "userType")]
    #[allow(dead_code)]
    pub user_type: String,
    #[allow(dead_code)]
    pub cwd: String,
    #[serde(rename = "gitBranch")]
    #[allow(dead_code)]
    pub git_branch: Option<String>,
    #[allow(dead_code)]
    pub version: String,
    #[serde(rename = "requestId")]
    #[allow(dead_code)]
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
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
pub enum UserContent {
    Text(String),
    ToolResults(Vec<ToolResult>),
}

#[derive(Debug, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
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
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}

pub fn parse_session_file(path: &Path) -> Result<Vec<Event>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        match serde_json::from_str::<SessionEntry>(&line) {
            Ok(SessionEntry::User(event)) => events.push(event),
            Ok(SessionEntry::Assistant(event)) => events.push(event),
            Ok(SessionEntry::Summary { .. }) => {
                // Skip summary entries for now
            },
            Err(_) => {
                // Skip unparseable lines silently
            }
        }
    }

    Ok(events)
}