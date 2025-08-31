use crate::parser_v2::DisplayEvent;
use colored::*;

pub fn print_session_header(session_id: &str, events: &[DisplayEvent]) {
    if events.is_empty() {
        return;
    }

    let first = &events[0];
    let last = &events[events.len() - 1];
    let duration = last.timestamp.since(first.timestamp).unwrap_or_default();

    let mut total_input = 0u32;
    let mut total_output = 0u32;

    for event in events {
        if event.role == "assistant" {
            if let Some(usage) = &event.usage {
                total_input += usage.input_tokens;
                total_output += usage.output_tokens;
            }
        }
    }

    let cost = (total_input as f64 * 0.015 + total_output as f64 * 0.075) / 1000.0;

    // Convert to local time for display
    let local_time = first.timestamp.to_zoned(jiff::tz::TimeZone::system());
    let local_start = format!("{}", local_time.strftime("%Y-%m-%d %H:%M:%S %Z"));

    let total_minutes = duration.total(jiff::Unit::Minute).unwrap_or(0.0) as i64;
    let seconds = (duration.total(jiff::Unit::Second).unwrap_or(0.0) as i64) % 60;

    println!(
        "{}",
        "┌─ Session ─────────────────────────────────────────────────┐".bright_blue()
    );
    println!("│ {} │", format!("ID: {}", session_id).bright_white());
    println!("│ {} │", format!("Started: {}", local_start).white());
    println!(
        "│ {} │",
        format!("Duration: {}m {}s", total_minutes, seconds).white()
    );
    println!("│ {} │", format!("Messages: {}", events.len()).white());
    println!(
        "│ {} │",
        format!("Tokens: {} in → {} out", total_input, total_output).white()
    );
    println!("│ {} │", format!("Est. Cost: ${:.2}", cost).white());
    println!(
        "{}",
        "└───────────────────────────────────────────────────────────┘".bright_blue()
    );
}
