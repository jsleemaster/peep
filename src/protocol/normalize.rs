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
