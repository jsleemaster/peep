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

    // Session ID: try both camelCase and snake_case
    let session_id = v
        .get("sessionId")
        .or_else(|| v.get("session_id"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Use slug as display name if available, otherwise session_id prefix
    let _slug = v.get("slug").and_then(|s| s.as_str()).map(str::to_string);

    // Git branch
    let branch_name = v
        .get("gitBranch")
        .or_else(|| v.get("git_branch"))
        .and_then(|s| s.as_str())
        .map(str::to_string);

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
                            .and_then(|i| {
                                i.get("file_path")
                                    .or_else(|| i.get("filePath"))
                                    .or_else(|| i.get("path"))
                            })
                            .and_then(|f| f.as_str())
                            .map(str::to_string);

                        let detail = block
                            .get("input")
                            .and_then(|i| {
                                i.get("command")
                                    .or_else(|| i.get("description"))
                                    .or_else(|| i.get("content"))
                                    .or_else(|| i.get("query"))
                                    .or_else(|| i.get("prompt"))
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
                            branch_name,
                        });
                    }
                }

                // No tool_use — look for text blocks
                for block in blocks {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        let text = block.get("text").and_then(|t| t.as_str()).unwrap_or("");
                        // Skip very short or empty text
                        if text.len() < 3 {
                            return None;
                        }
                        let detail = Some(truncate(text, 200));

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
                            branch_name,
                        });
                    }
                }
            }

            None
        }

        // User message — may contain tool_result blocks
        "user" => {
            let content = v
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array());

            if let Some(blocks) = content {
                for block in blocks {
                    if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                        let is_error = block
                            .get("is_error")
                            .and_then(|e| e.as_bool())
                            .unwrap_or(false);

                        // tool_result doesn't carry the tool name directly;
                        // we could match by tool_use_id but that's complex.
                        // Just emit a ToolDone with the content as detail.
                        let detail = block
                            .get("content")
                            .and_then(|c| c.as_str())
                            .map(|s| truncate(s, 200));

                        return Some(RawIngestEvent {
                            source: IngestSource::Jsonl,
                            agent_runtime_id: session_id.clone(),
                            session_runtime_id: Some(session_id),
                            ts,
                            event_type: RuntimeEventType::ToolDone,
                            hook_event_name: Some("PostToolUse".into()),
                            tool_name: None, // tool_result doesn't repeat the name
                            file_path: None,
                            detail,
                            total_tokens: None,
                            is_error,
                            branch_name,
                        });
                    }
                }

                // Plain user message (no tool_result) = new turn
                Some(RawIngestEvent {
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
                    branch_name,
                })
            } else {
                None
            }
        }

        // Progress events (hooks running, etc.) — skip most, but keep hook info
        "progress" => {
            // These are hook execution progress, not interesting for monitoring
            None
        }

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
            branch_name,
        }),

        // Legacy format: tool_result at top level
        "tool_result" => {
            let tool_name = v
                .get("name")
                .and_then(|n| n.as_str())
                .map(str::to_string);

            let is_error = v
                .get("is_error")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            Some(RawIngestEvent {
                source: IngestSource::Jsonl,
                agent_runtime_id: session_id.clone(),
                session_runtime_id: Some(session_id),
                ts,
                event_type: RuntimeEventType::ToolDone,
                hook_event_name: Some("PostToolUse".into()),
                tool_name,
                file_path: None,
                detail: None,
                total_tokens: None,
                is_error,
                branch_name,
            })
        }

        _ => None,
    }
}

/// Truncate a string to at most `max_chars` characters (UTF-8 safe).
fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().take(max_chars + 1).collect();
    if chars.len() > max_chars {
        let truncated: String = chars[..max_chars].iter().collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::RuntimeEventType;

    #[test]
    fn parse_tool_use_line() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/foo/bar.rs"}}]},"sessionId":"sess1","timestamp":"2025-12-20T10:30:00Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::ToolStart);
        assert_eq!(ev.tool_name.as_deref(), Some("Read"));
        assert_eq!(ev.file_path.as_deref(), Some("/foo/bar.rs"));
    }

    #[test]
    fn parse_tool_use_snake_case_session() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"t1","name":"Edit","input":{"file_path":"src/main.rs"}}]},"session_id":"sess2","timestamp":"2025-12-20T10:30:00Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.agent_runtime_id, "sess2");
    }

    #[test]
    fn parse_user_tool_result() {
        let line = r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"t1","type":"tool_result","content":"file contents","is_error":false}]},"sessionId":"sess1","timestamp":"2025-12-20T10:30:01Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::ToolDone);
    }

    #[test]
    fn parse_text_assistant_line() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello world this is a test"}]},"sessionId":"sess1","timestamp":"2025-12-20T10:30:02Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::AssistantText);
    }

    #[test]
    fn parse_user_plain_message() {
        let line = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"hello"}]},"sessionId":"sess1","timestamp":"2025-12-20T10:30:03Z"}"#;
        let ev = parse_jsonl_line(line).unwrap();
        assert_eq!(ev.event_type, RuntimeEventType::TurnActive);
    }

    #[test]
    fn progress_returns_none() {
        let line = r#"{"type":"progress","data":{"type":"hook_progress"},"sessionId":"s1","timestamp":"2025-12-20T10:30:00Z"}"#;
        assert!(parse_jsonl_line(line).is_none());
    }

    #[test]
    fn empty_line_returns_none() {
        assert!(parse_jsonl_line("").is_none());
        assert!(parse_jsonl_line("   ").is_none());
    }

    #[test]
    fn unknown_type_returns_none() {
        let line = r#"{"type":"unknown_event","sessionId":"s1"}"#;
        assert!(parse_jsonl_line(line).is_none());
    }
}
