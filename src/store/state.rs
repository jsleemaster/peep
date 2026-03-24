use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::protocol::normalize::tool_name_to_skill;
use crate::protocol::types::*;

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
            let display_name = raw
                .session_runtime_id
                .clone()
                .unwrap_or_else(|| short_id.clone());
            Agent {
                agent_id: agent_id.clone(),
                display_name,
                short_id: short_id.clone(),
                state: new_state,
                role: AgentRole::Main, // default; could be refined
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
            agent.total_tokens = tokens; // cumulative from Claude Code
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
