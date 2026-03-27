use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillKind {
    Read,
    Edit,
    Write,
    Bash,
    Search,
    Task,
    Ask,
    Other,
}

impl std::fmt::Display for SkillKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillKind::Read => write!(f, "Read"),
            SkillKind::Edit => write!(f, "Edit"),
            SkillKind::Write => write!(f, "Write"),
            SkillKind::Bash => write!(f, "Bash"),
            SkillKind::Search => write!(f, "Search"),
            SkillKind::Task => write!(f, "Task"),
            SkillKind::Ask => write!(f, "Ask"),
            SkillKind::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Active,
    Waiting,
    Completed,
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentState::Active => write!(f, "active"),
            AgentState::Waiting => write!(f, "waiting"),
            AgentState::Completed => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    Main,
    Team,
    Subagent,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Main => write!(f, "main"),
            AgentRole::Team => write!(f, "team"),
            AgentRole::Subagent => write!(f, "sub"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeEventType {
    ToolStart,
    ToolDone,
    AssistantText,
    PermissionWait,
    TurnWaiting,
    TurnActive,
}

impl std::fmt::Display for RuntimeEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeEventType::ToolStart => write!(f, "tool:start"),
            RuntimeEventType::ToolDone => write!(f, "tool:done"),
            RuntimeEventType::AssistantText => write!(f, "text"),
            RuntimeEventType::PermissionWait => write!(f, "perm:wait"),
            RuntimeEventType::TurnWaiting => write!(f, "turn:wait"),
            RuntimeEventType::TurnActive => write!(f, "turn:active"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionCloseReason {
    ConversationRollover,
    WorkFinished,
    StaleCleanup,
}

impl std::fmt::Display for SessionCloseReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionCloseReason::ConversationRollover => write!(f, "rollover"),
            SessionCloseReason::WorkFinished => write!(f, "finished"),
            SessionCloseReason::StaleCleanup => write!(f, "stale"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IngestSource {
    Http,
    Jsonl,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Agent {
    pub agent_id: String,
    pub display_name: String,
    pub short_id: String,
    pub state: AgentState,
    pub role: AgentRole,
    pub current_skill: Option<SkillKind>,
    pub branch_name: Option<String>,
    pub skill_usage: HashMap<SkillKind, u64>,
    pub skills_invoked: HashMap<String, u64>, // Skill tool invocations: "commit" → 3
    pub total_tokens: u64,
    pub usage_count: u64,
    pub tool_run_count: u64,
    pub last_event_ts: i64,
    pub context_percent: Option<f64>,
    pub cost_usd: Option<f64>,
    pub model_name: Option<String>,
    pub cwd: Option<String>,
    pub ai_tool: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FeedEvent {
    pub id: String,
    pub ts: i64,
    pub agent_id: String,
    pub display_name: String,
    pub short_id: String,
    pub skill: Option<SkillKind>,
    pub event_type: RuntimeEventType,
    pub tool_name: Option<String>,
    pub file_path: Option<String>,
    pub detail: Option<String>,
    pub total_tokens: Option<u64>,
    pub is_error: bool,
    pub ingest_source: IngestSource,
    pub ai_tool: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub display_name: String,
    pub role: AgentRole,
    pub started_at: i64,
    pub ended_at: i64,
    pub duration_ms: u64,
    pub event_count: u64,
    pub tool_run_count: u64,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
    pub close_reason: SessionCloseReason,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RawIngestEvent {
    pub source: IngestSource,
    pub agent_runtime_id: String,
    pub session_runtime_id: Option<String>,
    pub ts: i64,
    pub event_type: RuntimeEventType,
    pub hook_event_name: Option<String>,
    pub tool_name: Option<String>,
    pub file_path: Option<String>,
    pub detail: Option<String>,
    pub total_tokens: Option<u64>,
    pub is_error: bool,
    pub branch_name: Option<String>,
    pub slug: Option<String>,
    pub cwd: Option<String>,
    pub ai_tool: Option<String>,
}
