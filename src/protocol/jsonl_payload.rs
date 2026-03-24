use chrono::DateTime;
use serde_json::Value;

use super::types::{IngestSource, RawIngestEvent, RuntimeEventType};

/// Parse a single JSONL line from a Claude Code session file into a `RawIngestEvent`.
/// Returns `None` for lines that are empty, unparseable, or carry no actionable event.
pub fn parse_jsonl_line(line: &str) -> Option<RawIngestEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let v: Value = serde_json::from_str(line).ok()?;

    let entry_type = v.get("type")?.as_str()?;

    // session_id doubles as the agent runtime id for JSONL sources
    let session_id = v
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Parse timestamp; fall back to current time
    let ts = v
        .get("timestamp")
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    match entry_type {
        // assistant message — may contain tool_use blocks or plain text
        "assistant" => {
            let content = v
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array());

            if let Some(blocks) = content {
                // Look for the first tool_use block
                for block in blocks {
                    if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                        let tool_name = block
                            .get("name")
                            .and_then(|n| n.as_str())
                            .map(str::to_string);

                        let file_path = block
                            .get("input")
                            .and_then(|i| i.get("file_path"))
                            .and_then(|f| f.as_str())
                            .map(str::to_string);

                        let detail = block
                            .get("input")
                            .and_then(|i| {
                                i.get("command")
                                    .or_else(|| i.get("description"))
                                    .or_else(|| i.get("content"))
                            })
                            .and_then(|d| d.as_str())
                            .map(|s| truncate(s, 200));

                        return Some(RawIngestEvent {
                            source: IngestSource::Jsonl,
                            agent_runtime_id: session_id.clone(),
                            session_runtime_id: Some(session_id),
                            ts,
                            event_type: RuntimeEventType::ToolStart,
                            hook_event_name: Some("PreToolUse".into()),
                            tool_name,
                            file_path,
                            detail,
                            total_tokens: None,
                            is_error: false,
                            branch_name: None,
                        });
                    }
                }

                // No tool_use — look for text blocks
                for block in blocks {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        let detail = block
                            .get("text")
                            .and_then(|t| t.as_str())
                            .map(|s| truncate(s, 200));

                        return Some(RawIngestEvent {
                            source: IngestSource::Jsonl,
                            agent_runtime_id: session_id.clone(),
                            session_runtime_id: Some(session_id),
                            ts,
                            event_type: RuntimeEventType::AssistantText,
                            hook_event_name: Some("Notification".into()),
                            tool_name: None,
                            file_path: None,
                            detail,
                            total_tokens: None,
                            is_error: false,
                            branch_name: None,
                        });
                    }
                }
            }

            None
        }

        // tool result returned to the model
        "tool_result" => {
            let tool_name = v
                .get("name")
                .and_then(|n| n.as_str())
                .map(str::to_string);

            let is_error = v
                .get("is_error")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            let detail = v
                .get("content")
                .and_then(|c| c.as_str())
                .map(|s| truncate(s, 200));

            Some(RawIngestEvent {
                source: IngestSource::Jsonl,
                agent_runtime_id: session_id.clone(),
                session_runtime_id: Some(session_id),
                ts,
                event_type: RuntimeEventType::ToolDone,
                hook_event_name: Some("PostToolUse".into()),
                tool_name,
                file_path: None,
                detail,
                total_tokens: None,
                is_error,
                branch_name: None,
            })
        }

        // human / user turn
        "user" => Some(RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: session_id.clone(),
            session_runtime_id: Some(session_id),
            ts,
            event_type: RuntimeEventType::TurnActive,
            hook_event_name: Some("UserPromptSubmit".into()),
            tool_name: None,
            file_path: None,
            detail: None,
            total_tokens: None,
            is_error: false,
            branch_name: None,
        }),

        // session result / end
        "result" => Some(RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: session_id.clone(),
            session_runtime_id: Some(session_id),
            ts,
            event_type: RuntimeEventType::TurnWaiting,
            hook_event_name: Some("Stop".into()),
            tool_name: None,
            file_path: None,
            detail: v
                .get("result")
                .and_then(|r| r.as_str())
                .map(|s| truncate(s, 200)),
            total_tokens: None,
            is_error: false,
            branch_name: None,
        }),

        _ => None,
    }
}

/// Truncate a string to at most `max_chars` characters.
fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}…", truncated)
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::RuntimeEventType;

    #[test]
    fn parse_tool_use_line() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/foo/bar.rs"}}]},"session_id":"sess1","timestamp":"2025-12-20T10:30:00Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::ToolStart);
        assert_eq!(ev.tool_name.as_deref(), Some("Read"));
        assert_eq!(ev.file_path.as_deref(), Some("/foo/bar.rs"));
    }

    #[test]
    fn parse_tool_result_line() {
        let line = r#"{"type":"tool_result","name":"Read","content":"file contents","session_id":"sess1","timestamp":"2025-12-20T10:30:01Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::ToolDone);
        assert_eq!(ev.tool_name.as_deref(), Some("Read"));
    }

    #[test]
    fn parse_text_assistant_line() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello world"}]},"session_id":"sess1","timestamp":"2025-12-20T10:30:02Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::AssistantText);
        assert_eq!(ev.detail.as_deref(), Some("Hello world"));
    }

    #[test]
    fn parse_user_line() {
        let line = r#"{"type":"user","session_id":"sess1","timestamp":"2025-12-20T10:30:03Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::TurnActive);
    }

    #[test]
    fn empty_line_returns_none() {
        assert!(parse_jsonl_line("").is_none());
        assert!(parse_jsonl_line("   ").is_none());
    }

    #[test]
    fn unknown_type_returns_none() {
        let line = r#"{"type":"unknown_event","session_id":"s1"}"#;
        assert!(parse_jsonl_line(line).is_none());
    }
}
