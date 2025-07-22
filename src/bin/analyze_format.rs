use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use serde_json::{Value, Map};
use anyhow::Result;

fn main() -> Result<()> {
    let claude_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".claude/projects");
    
    println!("Analyzing JSONL format variations...\n");
    
    let mut all_keys: HashMap<String, HashSet<String>> = HashMap::new();
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut sample_events: HashMap<String, Value> = HashMap::new();
    let mut total_lines = 0;
    let mut parsed_lines = 0;
    
    // Scan all JSONL files
    for entry in std::fs::read_dir(&claude_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            for file_entry in std::fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                
                if file_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                    analyze_file(&file_path, &mut all_keys, &mut type_counts, &mut sample_events, &mut total_lines, &mut parsed_lines)?;
                }
            }
        }
    }
    
    // Report findings
    println!("=== Format Analysis Results ===\n");
    println!("Total lines scanned: {}", total_lines);
    println!("Successfully parsed: {} ({:.1}%)\n", parsed_lines, (parsed_lines as f64 / total_lines as f64) * 100.0);
    
    println!("Event types found:");
    let mut types: Vec<_> = type_counts.iter().collect();
    types.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    for (event_type, count) in types {
        println!("  {:20} {:6} occurrences", event_type, count);
    }
    
    println!("\nKeys by event type:");
    let mut sorted_types: Vec<_> = all_keys.keys().collect();
    sorted_types.sort();
    
    for event_type in sorted_types {
        println!("\n{}:", event_type);
        let mut keys: Vec<_> = all_keys[event_type].iter().collect();
        keys.sort();
        for key in keys {
            println!("  - {}", key);
        }
        
        // Show sample
        if let Some(sample) = sample_events.get(event_type) {
            println!("\n  Sample:");
            let pretty = serde_json::to_string_pretty(sample)?;
            for line in pretty.lines().take(10) {
                println!("  {}", line);
            }
            if pretty.lines().count() > 10 {
                println!("  ...");
            }
        }
    }
    
    Ok(())
}

fn analyze_file(
    path: &Path,
    all_keys: &mut HashMap<String, HashSet<String>>,
    type_counts: &mut HashMap<String, usize>,
    sample_events: &mut HashMap<String, Value>,
    total_lines: &mut usize,
    parsed_lines: &mut usize,
) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        *total_lines += 1;
        
        if line.trim().is_empty() {
            continue;
        }
        
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            *parsed_lines += 1;
            
            if let Some(obj) = value.as_object() {
                // Get event type
                let event_type = obj.get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Count this type
                *type_counts.entry(event_type.clone()).or_insert(0) += 1;
                
                // Store a sample if we don't have one
                if !sample_events.contains_key(&event_type) {
                    sample_events.insert(event_type.clone(), value.clone());
                }
                
                // Collect all keys
                let keys = all_keys.entry(event_type).or_insert_with(HashSet::new);
                collect_keys(obj, String::new(), keys);
            }
        }
    }
    
    Ok(())
}

fn collect_keys(obj: &Map<String, Value>, prefix: String, keys: &mut HashSet<String>) {
    for (key, value) in obj {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };
        
        keys.insert(full_key.clone());
        
        // Recursively collect nested object keys
        if let Some(nested_obj) = value.as_object() {
            collect_keys(nested_obj, full_key, keys);
        } else if let Some(array) = value.as_array() {
            // Check first element of arrays for object structure
            if let Some(first) = array.first() {
                if let Some(nested_obj) = first.as_object() {
                    collect_keys(nested_obj, format!("{}[]", full_key), keys);
                }
            }
        }
    }
}