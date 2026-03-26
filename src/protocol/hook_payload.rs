use chrono::Utc;
use serde_json::Value;

use super::normalize::{map_hook_event_to_runtime_type, tool_name_to_skill};
use super::types::{IngestSource, RawIngestEvent, RuntimeEventType};

/// Try multiple JSON paths to extract a string value.
fn try_str<'a>(v: &'a Value, paths: &[&str]) -> Option<&'a str> {
    for path in paths {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = v;
        let mut found = true;
        for part in &parts {
            match current.get(part) {
                Some(next) => current = next,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            if let Some(s) = current.as_str() {
                return Some(s);
            }
        }
    }
    None
}

fn try_u64(v: &Value, paths: &[&str]) -> Option<u64> {
    for path in paths {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = v;
        let mut found = true;
        for part in &parts {
            match current.get(part) {
                Some(next) => current = next,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            if let Some(n) = current.as_u64() {
                return Some(n);
            }
        }
    }
    None
}

fn try_bool(v: &Value, paths: &[&str]) -> Option<bool> {
    for path in paths {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = v;
        let mut found = true;
        for part in &parts {
            match current.get(part) {
                Some(next) => current = next,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            if let Some(b) = current.as_bool() {
                return Some(b);
            }
        }
    }
    None
}

/// Parse a Claude Code hook JSON payload into a RawIngestEvent.
pub fn parse_hook_payload(body: &Value) -> Option<RawIngestEvent> {
    let hook_event_name = try_str(
        body,
        &["hook_event_name", "hookEventName", "event_name", "event"],
    )
    .map(|s| s.to_string());

    let event_type = hook_event_name
        .as_deref()
        .map(map_hook_event_to_runtime_type)
        .unwrap_or(RuntimeEventType::AssistantText);

    let session_id = try_str(
        body,
        &["session_id", "sessionId", "session.id"],
    )
    .map(|s| s.to_string());

    // Use session_id as agent_runtime_id (Claude Code uses one session per agent)
    let agent_runtime_id = session_id
        .clone()
        .unwrap_or_else(|| format!("unknown-{}", uuid::Uuid::new_v4()));

    let tool_name = try_str(
        body,
        &["tool_name", "toolName", "tool.name", "tool_use_name"],
    )
    .map(|s| s.to_string());

    let file_path = try_str(
        body,
        &["tool_input.file_path", "tool_input.filePath", "file_path"],
    )
    .map(|s| s.to_string());

    let detail = try_str(
        body,
        &[
            "tool_input.command",
            "tool_input.content",
            "message",
            "detail",
        ],
    )
    .map(|s| {
        let chars: Vec<char> = s.chars().take(201).collect();
        if chars.len() > 200 {
            format!("{}...", chars[..197].iter().collect::<String>())
        } else {
            s.to_string()
        }
    });

    let total_tokens = try_u64(
        body,
        &[
            "total_tokens",
            "totalTokens",
            "usage.total_tokens",
            "session.total_tokens",
        ],
    );

    let is_error = try_bool(body, &["is_error", "isError", "error"])
        .unwrap_or(false);

    let branch_name = try_str(body, &["branch", "branch_name", "git_branch"])
        .map(|s| s.to_string());

    // If we have a tool_name, determine skill; used in FeedEvent creation later
    let _skill = tool_name.as_deref().map(tool_name_to_skill);

    Some(RawIngestEvent {
        source: IngestSource::Http,
        agent_runtime_id,
        session_runtime_id: session_id,
        ts: Utc::now().timestamp(),
        event_type,
        hook_event_name,
        tool_name,
        file_path,
        detail,
        total_tokens,
        is_error,
        branch_name,
        slug: None,
        cwd: None,
    })
}
