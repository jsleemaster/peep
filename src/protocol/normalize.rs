use super::types::{RuntimeEventType, SkillKind};

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
    !lhs.is_empty()
        && !rhs.is_empty()
        && lhs
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
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
