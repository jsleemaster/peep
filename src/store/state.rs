use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::normalize::tool_name_to_skill;
use crate::protocol::types::*;

use chrono::Utc;

const FEED_LIMIT: usize = 1000;
const STALE_AGENT_MS: i64 = 300_000; // 5 minutes

pub struct AppStore {
    pub agents: HashMap<String, Agent>,
    pub feed: VecDeque<FeedEvent>,
    pub sessions: Vec<Session>,
    pub activity_timestamps: VecDeque<i64>,
}

pub type SharedStore = Arc<RwLock<AppStore>>;

impl AppStore {
    pub fn new() -> Self {
        AppStore {
            agents: HashMap::new(),
            feed: VecDeque::new(),
            sessions: Vec::new(),
            activity_timestamps: VecDeque::new(),
        }
    }

    pub fn new_shared() -> SharedStore {
        Arc::new(RwLock::new(Self::new()))
    }

    /// Apply a raw ingest event: upsert agent, append feed event, update metrics.
    pub fn apply_event(&mut self, raw: RawIngestEvent) {
        let agent_id = raw.agent_runtime_id.clone();
        let short_id = if agent_id.len() >= 8 {
            agent_id[..8].to_string()
        } else {
            agent_id.clone()
        };

        let skill = raw.tool_name.as_deref().map(tool_name_to_skill);

        // Derive agent state from event type
        let new_state = match raw.event_type {
            RuntimeEventType::ToolStart
            | RuntimeEventType::TurnActive
            | RuntimeEventType::AssistantText => AgentState::Active,
            RuntimeEventType::PermissionWait | RuntimeEventType::TurnWaiting => AgentState::Waiting,
            RuntimeEventType::ToolDone => AgentState::Active,
        };

        // Upsert agent
        let agent = self.agents.entry(agent_id.clone()).or_insert_with(|| {
            // Sub-agents: use description part (before |) as name
            // Others: slug > session_runtime_id > short_id
            let is_sub = matches!(
                raw.hook_event_name.as_deref(),
                Some("AgentSpawn") | Some("Subagent")
            );
            let display_name = if is_sub {
                raw.detail.as_deref()
                    .and_then(|d| d.split(" | ").next())
                    .or(raw.slug.as_deref())
                    .unwrap_or(&short_id)
                    .to_string()
            } else {
                raw.slug.clone()
                    .or_else(|| raw.session_runtime_id.clone())
                    .unwrap_or_else(|| short_id.clone())
            };
            Agent {
                agent_id: agent_id.clone(),
                display_name,
                short_id: short_id.clone(),
                state: new_state,
                role: if is_sub {
                    AgentRole::Subagent
                } else {
                    AgentRole::Main
                },
                current_skill: skill,
                branch_name: raw.branch_name.clone(),
                skill_usage: HashMap::new(),
                total_tokens: 0,
                usage_count: 0,
                tool_run_count: 0,
                last_event_ts: raw.ts,
                context_percent: None,
                cost_usd: None,
                model_name: None,
                cwd: raw.cwd.clone(),
                ai_tool: raw.ai_tool.clone(),
            }
        });

        agent.state = new_state;
        agent.last_event_ts = raw.ts;
        agent.usage_count += 1;
        agent.current_skill = skill;

        if raw.branch_name.is_some() {
            agent.branch_name = raw.branch_name.clone();
        }

        if let Some(tokens) = raw.total_tokens {
            // Detect session rollover: token count drops >50% → context was reset
            if tokens > 0 && agent.total_tokens > 0 && tokens < agent.total_tokens / 2 {
                agent.total_tokens = tokens; // reset HP to new session baseline
            } else {
                agent.total_tokens = agent.total_tokens.max(tokens);
            }
        }

        if let Some(s) = skill {
            *agent.skill_usage.entry(s).or_insert(0) += 1;
        }

        if raw.tool_name.is_some() {
            agent.tool_run_count += 1;
        }

        let display_name = agent.display_name.clone();
        let agent_short_id = agent.short_id.clone();

        // Create feed event
        let feed_event = FeedEvent {
            id: uuid::Uuid::new_v4().to_string(),
            ts: raw.ts,
            agent_id: agent_id.clone(),
            display_name,
            short_id: agent_short_id,
            skill,
            event_type: raw.event_type,
            tool_name: raw.tool_name,
            file_path: raw.file_path,
            detail: raw.detail,
            total_tokens: raw.total_tokens,
            is_error: raw.is_error,
            ingest_source: raw.source,
            ai_tool: raw.ai_tool,
        };

        self.feed.push_back(feed_event);
        while self.feed.len() > FEED_LIMIT {
            self.feed.pop_front();
        }

        // Record activity timestamp for velocity
        self.activity_timestamps.push_back(raw.ts);
        // Keep only last 5 minutes
        let cutoff = raw.ts - 300;
        while self
            .activity_timestamps
            .front()
            .is_some_and(|&t| t < cutoff)
        {
            self.activity_timestamps.pop_front();
        }
    }

    /// Mark agents with no events for STALE_AGENT_MS as Completed.
    pub fn gc_stale_agents(&mut self, now: i64) {
        let stale_threshold = now - (STALE_AGENT_MS / 1000);
        for agent in self.agents.values_mut() {
            if agent.state != AgentState::Completed && agent.last_event_ts < stale_threshold {
                agent.state = AgentState::Completed;
                agent.current_skill = None;
            }
        }
    }

    /// Count events in the last 60 seconds.
    pub fn velocity_per_min(&self, now: i64) -> usize {
        let cutoff = now - 60;
        self.activity_timestamps
            .iter()
            .filter(|&&t| t >= cutoff)
            .count()
    }

    #[allow(dead_code)]
    pub fn total_tokens(&self) -> u64 {
        self.agents.values().map(|a| a.total_tokens).sum()
    }

    #[allow(dead_code)]
    pub fn total_cost(&self) -> f64 {
        self.agents.values().filter_map(|a| a.cost_usd).sum()
    }

    #[allow(dead_code)]
    pub fn active_count(&self) -> usize {
        self.agents
            .values()
            .filter(|a| a.state == AgentState::Active)
            .count()
    }

    /// Divide last 5 minutes into N buckets, count events per bucket.
    pub fn velocity_sparkline_data(&self, buckets: usize, now: i64) -> Vec<u64> {
        if buckets == 0 {
            return vec![];
        }
        let window = 300i64; // 5 minutes
        let bucket_size = window / buckets as i64;
        let start = now - window;

        let mut result = vec![0u64; buckets];
        for &ts in &self.activity_timestamps {
            if ts >= start {
                let idx = ((ts - start) / bucket_size.max(1)) as usize;
                let idx = idx.min(buckets - 1);
                result[idx] += 1;
            }
        }
        result
    }

    /// Get sorted agent list (Active first, then Waiting, then Completed).
    pub fn sorted_agents(&self) -> Vec<&Agent> {
        let mut agents: Vec<&Agent> = self.agents.values().collect();
        agents.sort_by(|a, b| {
            let order = |s: &AgentState| -> u8 {
                match s {
                    AgentState::Active => 0,
                    AgentState::Waiting => 1,
                    AgentState::Completed => 2,
                }
            };
            order(&a.state)
                .cmp(&order(&b.state))
                .then(b.last_event_ts.cmp(&a.last_event_ts))
        });
        agents
    }

    /// Populate the store with synthetic demo data for `--mock` mode.
    pub fn populate_mock_data(&mut self) {
        let now = Utc::now().timestamp();

        // ----------------------------------------------------------------
        // Agents
        // ----------------------------------------------------------------
        type AgentTuple<'a> = (&'a str, &'a str, AgentState, AgentRole, Option<f64>, &'a str);
        let agents_raw: &[AgentTuple] = &[
            ("main-worker-0001abcd", "main-worker",  AgentState::Active,    AgentRole::Main,     Some(67.0), "/Users/leeo/evar/platform"),
            ("team-review-0002efgh", "team-review",  AgentState::Waiting,   AgentRole::Team,     Some(22.0), "/Users/leeo/evar/platform"),
            ("sub-scout-0003ijkl",   "sub-scout",    AgentState::Completed, AgentRole::Subagent, None,       "/Users/leeo/peep"),
            ("team-builder-0004mnop","team-builder", AgentState::Active,    AgentRole::Team,     Some(45.0), "/Users/leeo/bill-pr"),
        ];

        for (id, name, state, role, ctx, cwd) in agents_raw {
            let mut skill_usage = HashMap::new();
            skill_usage.insert(SkillKind::Read,   12u64);
            skill_usage.insert(SkillKind::Edit,    5);
            skill_usage.insert(SkillKind::Bash,    8);
            skill_usage.insert(SkillKind::Search,  3);

            self.agents.insert(
                id.to_string(),
                Agent {
                    agent_id:      id.to_string(),
                    display_name:  name.to_string(),
                    short_id:      id[..8].to_string(),
                    state:         *state,
                    role:          *role,
                    current_skill: if *state == AgentState::Active { Some(SkillKind::Edit) } else { None },
                    branch_name:   Some(format!("feat/{name}")),
                    skill_usage,
                    total_tokens:  42_000,
                    usage_count:   28,
                    tool_run_count: 28,
                    last_event_ts: now - 30,
                    context_percent: *ctx,
                    cost_usd:      Some(0.12),
                    model_name:    Some("claude-sonnet-4-5".to_string()),
                    cwd:           Some(cwd.to_string()),
                    ai_tool:       Some("claude".to_string()),
                },
            );
        }

        // ----------------------------------------------------------------
        // Feed events (~20)
        // ----------------------------------------------------------------
        let feed_entries: &[(&str, RuntimeEventType, Option<&str>, Option<&str>)] = &[
            ("main-worker-0001abcd",  RuntimeEventType::ToolStart,    Some("Read"),  Some("src/main.rs")),
            ("main-worker-0001abcd",  RuntimeEventType::ToolDone,     Some("Read"),  Some("src/main.rs")),
            ("team-review-0002efgh",  RuntimeEventType::TurnActive,   None,          None),
            ("main-worker-0001abcd",  RuntimeEventType::ToolStart,    Some("Edit"),  Some("src/store/state.rs")),
            ("sub-scout-0003ijkl",    RuntimeEventType::ToolStart,    Some("Bash"),  Some("cargo build")),
            ("team-builder-0004mnop", RuntimeEventType::TurnActive,   None,          None),
            ("main-worker-0001abcd",  RuntimeEventType::ToolDone,     Some("Edit"),  Some("src/store/state.rs")),
            ("team-review-0002efgh",  RuntimeEventType::PermissionWait, None,        Some("waiting for user")),
            ("sub-scout-0003ijkl",    RuntimeEventType::ToolDone,     Some("Bash"),  None),
            ("team-builder-0004mnop", RuntimeEventType::ToolStart,    Some("Search"), Some("Cargo.toml")),
            ("main-worker-0001abcd",  RuntimeEventType::AssistantText, None,         None),
            ("team-builder-0004mnop", RuntimeEventType::ToolDone,     Some("Search"), None),
            ("sub-scout-0003ijkl",    RuntimeEventType::TurnWaiting,  None,          None),
            ("main-worker-0001abcd",  RuntimeEventType::ToolStart,    Some("Bash"),  Some("cargo clippy")),
            ("team-review-0002efgh",  RuntimeEventType::AssistantText, None,         None),
            ("main-worker-0001abcd",  RuntimeEventType::ToolDone,     Some("Bash"),  None),
            ("team-builder-0004mnop", RuntimeEventType::ToolStart,    Some("Read"),  Some("README.md")),
            ("main-worker-0001abcd",  RuntimeEventType::ToolStart,    Some("Write"), Some("src/config.rs")),
            ("team-builder-0004mnop", RuntimeEventType::ToolDone,     Some("Read"),  None),
            ("main-worker-0001abcd",  RuntimeEventType::ToolDone,     Some("Write"), None),
        ];

        for (i, (agent_id, event_type, tool_name, file_path)) in feed_entries.iter().enumerate() {
            let skill = tool_name.map(tool_name_to_skill);
            // Compute display_name from agent_id stored in agents map
            let (display_name, short_id) = self
                .agents
                .get(*agent_id)
                .map(|a| (a.display_name.clone(), a.short_id.clone()))
                .unwrap_or_else(|| (agent_id[..8].to_string(), agent_id[..8].to_string()));

            let ev = FeedEvent {
                id: uuid::Uuid::new_v4().to_string(),
                ts: now - (feed_entries.len() as i64 - i as i64) * 3,
                agent_id: agent_id.to_string(),
                display_name,
                short_id,
                skill,
                event_type: *event_type,
                tool_name: tool_name.map(|s| s.to_string()),
                file_path: file_path.map(|s| s.to_string()),
                detail: None,
                total_tokens: Some(42_000),
                is_error: false,
                ingest_source: IngestSource::Http,
                ai_tool: Some("claude".to_string()),
            };
            self.feed.push_back(ev);
        }

        // ----------------------------------------------------------------
        // Completed sessions (3)
        // ----------------------------------------------------------------
        let sessions_raw: &[(&str, AgentRole, u64, u64)] = &[
            ("session-alpha",   AgentRole::Main,     3_600_000, 84_000),
            ("session-beta",    AgentRole::Team,     1_200_000, 31_000),
            ("session-gamma",   AgentRole::Subagent,   900_000, 18_500),
        ];

        for (session_id, role, duration_ms, tokens) in sessions_raw {
            self.sessions.push(Session {
                session_id:    session_id.to_string(),
                display_name:  session_id.to_string(),
                role:          *role,
                started_at:    now - (*duration_ms as i64 / 1000) - 600,
                ended_at:      now - 600,
                duration_ms:   *duration_ms,
                event_count:   55,
                tool_run_count: 42,
                total_tokens:  *tokens,
                cost_usd:      Some(0.08),
                close_reason:  SessionCloseReason::WorkFinished,
            });
        }

        // Activity timestamps for sparkline
        for i in 0..20i64 {
            self.activity_timestamps.push_back(now - i * 10);
        }
    }

    pub fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}k", tokens as f64 / 1_000.0)
        } else {
            format!("{}", tokens)
        }
    }
}
