use super::types::{RawIngestEvent, RuntimeEventType, SkillKind};

pub fn map_hook_event_to_runtime_type(name: &str) -> RuntimeEventType {
    match name
        .to_lowercase()
        .replace(&[' ', '_', '-'][..], "")
        .as_str()
    {
        "pretooluse" => RuntimeEventType::ToolStart,
        "posttooluse" => RuntimeEventType::ToolDone,
        "notification" => RuntimeEventType::AssistantText,
        "userpromptsubmit" | "sessionstart" => RuntimeEventType::TurnActive,
        "sessionend" | "stop" | "subagentstop" => RuntimeEventType::TurnWaiting,
        "permissionrequest" => RuntimeEventType::PermissionWait,
        _ => RuntimeEventType::AssistantText,
    }
}

pub fn tool_name_to_skill(name: &str) -> SkillKind {
    match name.to_lowercase().as_str() {
        "read" => SkillKind::Read,
        "edit" => SkillKind::Edit,
        "write" => SkillKind::Write,
        "bash" => SkillKind::Bash,
        "grep" | "glob" | "search" => SkillKind::Search,
        "taskcreate" | "taskupdate" | "todowrite" => SkillKind::Task,
        "askuserquestion" => SkillKind::Ask,
        _ => SkillKind::Other,
    }
}

pub fn extract_ranked_command(tool_name: Option<&str>, detail: Option<&str>) -> Option<String> {
    let detail = detail?.trim();
    if detail.is_empty() {
        return None;
    }

    let normalized = normalize_ranked_command(detail)?;
    let tool_is_command = matches!(tool_name, Some("Bash") | Some("bash"));
    if tool_is_command || is_likely_command_input(detail) {
        Some(normalized)
    } else {
        None
    }
}

pub fn normalize_project_name(cwd: &str) -> String {
    let parts: Vec<&str> = cwd.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if (*part == "services" || *part == "app") && i + 1 < parts.len() {
            return parts[i + 1].to_string();
        }
    }

    let skip = [
        "src",
        "shared",
        "assets",
        "images",
        "ui",
        "components",
        ".claude",
        "mcp",
    ];
    for part in parts.iter().rev() {
        if !part.is_empty() && !skip.contains(part) {
            return part.to_string();
        }
    }

    cwd.rsplit('/').next().unwrap_or(cwd).to_string()
}

pub fn derive_agent_display_name(raw: &RawIngestEvent) -> String {
    let is_subagent = is_subagent_event(raw.hook_event_name.as_deref());
    let short_id = derive_agent_short_id(&raw.agent_runtime_id, is_subagent);

    if is_subagent {
        return extract_subagent_name(raw.detail.as_deref()).unwrap_or(short_id);
    }

    raw.slug
        .clone()
        .or_else(|| raw.session_runtime_id.clone())
        .unwrap_or(short_id)
}

pub fn derive_agent_short_id(agent_id: &str, is_subagent: bool) -> String {
    if is_subagent {
        let tail = agent_id.rsplit('-').next().unwrap_or(agent_id);
        last_n_chars(tail, 8)
    } else {
        agent_id.chars().take(8).collect()
    }
}

pub fn sanitize_agent_display_name(display_name: Option<&str>, agent_id: &str) -> String {
    display_name
        .filter(|name| looks_like_stable_agent_name(name.trim()))
        .map(|name| name.trim().to_string())
        .unwrap_or_else(|| derive_agent_short_id(agent_id, true))
}

pub fn normalize_ranked_command(input: &str) -> Option<String> {
    let mut tokens = input.split_whitespace().peekable();
    while let Some(token) = tokens.peek().copied() {
        if is_env_assignment(token) {
            tokens.next();
        } else {
            break;
        }
    }

    let command = canonical_command_name(tokens.next()?);
    if command.is_empty() || is_shell_control(&command) {
        return None;
    }

    if command == "python" || command == "python3" {
        if let Some("-m") = tokens.peek().copied() {
            tokens.next();
            if let Some(module) = tokens.next() {
                return Some(format!("{} -m {}", command, shorten_token(module)));
            }
        }
        return Some(command);
    }

    if takes_subcommand(&command) {
        if let Some(next) = tokens.peek().copied() {
            if !next.starts_with('-') && !is_shell_control(next) {
                return Some(format!("{} {}", command, shorten_token(next)));
            }
        }
    }

    Some(command)
}

fn canonical_command_name(token: &str) -> String {
    shorten_token(token)
}

fn shorten_token(token: &str) -> String {
    token
        .trim_matches(|c| c == '"' || c == '\'')
        .rsplit('/')
        .next()
        .unwrap_or(token)
        .to_string()
}

fn is_env_assignment(token: &str) -> bool {
    let Some((lhs, rhs)) = token.split_once('=') else {
        return false;
    };
    !lhs.is_empty() && !rhs.is_empty() && lhs.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_shell_control(token: &str) -> bool {
    matches!(token, "|" | "||" | "&&" | ";" | "&")
}

fn takes_subcommand(command: &str) -> bool {
    matches!(
        command,
        "git"
            | "cargo"
            | "pnpm"
            | "npm"
            | "npx"
            | "yarn"
            | "bun"
            | "gh"
            | "go"
            | "docker"
            | "kubectl"
            | "bash"
            | "sh"
            | "zsh"
    )
}

fn is_subagent_event(hook_event_name: Option<&str>) -> bool {
    matches!(hook_event_name, Some("AgentSpawn") | Some("Subagent"))
}

fn extract_subagent_name(detail: Option<&str>) -> Option<String> {
    let candidate = detail?.split(" | ").next()?.trim();
    if looks_like_stable_agent_name(candidate) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn looks_like_stable_agent_name(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.len() <= 48
        && !candidate.chars().any(char::is_whitespace)
        && candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && candidate.chars().any(|c| c.is_ascii_alphabetic())
}

fn last_n_chars(input: &str, count: usize) -> String {
    let chars: Vec<char> = input.chars().collect();
    let start = chars.len().saturating_sub(count);
    chars[start..].iter().collect()
}

fn is_likely_command_input(input: &str) -> bool {
    let Some(command) = normalize_ranked_command(input) else {
        return false;
    };

    let first = command.split_whitespace().next().unwrap_or("");
    matches!(
        first,
        "git"
            | "cargo"
            | "pnpm"
            | "npm"
            | "npx"
            | "yarn"
            | "bun"
            | "rg"
            | "grep"
            | "sed"
            | "awk"
            | "ls"
            | "cat"
            | "find"
            | "gh"
            | "go"
            | "python"
            | "python3"
            | "node"
            | "pytest"
            | "bash"
            | "sh"
            | "zsh"
            | "make"
            | "docker"
            | "kubectl"
    )
}

#[cfg(test)]
mod tests {
    use super::{derive_agent_display_name, derive_agent_short_id, sanitize_agent_display_name};
    use crate::protocol::types::{IngestSource, RawIngestEvent, RuntimeEventType};

    fn subagent_event(agent_id: &str, detail: &str) -> RawIngestEvent {
        RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: agent_id.to_string(),
            session_runtime_id: Some("session-alpha".into()),
            ts: 10,
            event_type: RuntimeEventType::ToolStart,
            hook_event_name: Some("Subagent".into()),
            tool_name: Some("Bash".into()),
            file_path: None,
            detail: Some(detail.to_string()),
            total_tokens: None,
            is_error: false,
            branch_name: None,
            slug: Some("parent-slug".into()),
            cwd: Some("/tmp/project-a".into()),
            ai_tool: Some("codex".into()),
        }
    }

    #[test]
    fn derive_agent_display_name_rejects_conversational_subagent_detail() {
        let raw = subagent_event(
            "session-alpha-12345678",
            "I'll work on this task | prompt preview",
        );

        assert_eq!(derive_agent_display_name(&raw), "12345678");
    }

    #[test]
    fn derive_agent_display_name_accepts_slug_like_subagent_label() {
        let raw = subagent_event("session-alpha-12345678", "code-reviewer | prompt preview");

        assert_eq!(derive_agent_display_name(&raw), "code-reviewer");
    }

    #[test]
    fn derive_agent_short_id_uses_tail_for_subagents() {
        assert_eq!(
            derive_agent_short_id("session-alpha-12345678", true),
            "12345678"
        );
        assert_eq!(
            derive_agent_short_id("main-worker-0001abcd", false),
            "main-wor"
        );
    }

    #[test]
    fn sanitize_agent_display_name_replaces_conversational_labels_with_specific_id() {
        assert_eq!(
            sanitize_agent_display_name(
                Some("I'll work on this task"),
                "session-alpha-12345678"
            ),
            "12345678"
        );
        assert_eq!(
            sanitize_agent_display_name(Some("code-reviewer"), "session-alpha-12345678"),
            "code-reviewer"
        );
    }
}
