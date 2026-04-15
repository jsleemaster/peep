use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::protocol::types::{Agent, FeedEvent, Session};
#[cfg(test)]
pub(crate) use crate::protocol::normalize::normalize_ranked_command;
use crate::tui::app::App;
use crate::tui::widgets::{agent_detail, stage, tab_bar};

/// A snapshot of the store for rendering (avoids holding the lock during draw).
#[allow(dead_code)]
pub struct StoreSnapshot {
    pub agents: Vec<Agent>,
    pub feed: Vec<FeedEvent>,
    pub sessions: Vec<Session>,
    pub sparkline: Vec<u64>,
    pub metrics: crate::store::metrics::DerivedMetrics,
    pub available_skills: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedEntry {
    pub name: String,
    pub count: u64,
    pub last_seen: i64,
}

impl RankedEntry {
    pub fn new(name: impl Into<String>, count: u64, last_seen: i64) -> Self {
        Self {
            name: name.into(),
            count,
            last_seen,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageRankings {
    pub commands: Vec<RankedEntry>,
    pub skills: Vec<RankedEntry>,
}

impl StoreSnapshot {
    pub async fn from_store(store: &crate::store::state::SharedStore) -> Self {
        let s = store.read().await;
        let now = chrono::Utc::now().timestamp();
        let agents = s.sorted_agents().into_iter().cloned().collect();
        let feed: Vec<FeedEvent> = s.feed.iter().cloned().collect();
        let sessions = s.sessions.clone();
        let sparkline = s.velocity_sparkline_data(15, now);
        let metrics = s.derived_metrics(now);
        let available_skills = s.available_skills.clone();
        StoreSnapshot {
            agents,
            feed,
            sessions,
            sparkline,
            metrics,
            available_skills,
        }
    }

    pub fn stage_rankings(
        &self,
        project: Option<&str>,
        focused_agent: Option<&str>,
    ) -> StageRankings {
        let agents: Vec<&Agent> = self
            .agents
            .iter()
            .filter(|agent| match project {
                Some(name) => agent
                    .cwd
                    .as_deref()
                    .map(stage::normalize_project_name)
                    .as_deref()
                    == Some(name),
                None => true,
            })
            .filter(|agent| match focused_agent {
                Some(agent_id) => agent.agent_id == agent_id,
                None => true,
            })
            .collect();

        let mut command_counts = std::collections::HashMap::<String, u64>::new();
        let mut command_last_seen = std::collections::HashMap::<String, i64>::new();
        let mut skill_counts = std::collections::HashMap::<String, u64>::new();
        let mut skill_last_seen = std::collections::HashMap::<String, i64>::new();

        for agent in agents {
            for (name, count) in &agent.command_usage {
                *command_counts.entry(name.clone()).or_insert(0) += count;
                let ts = agent
                    .command_last_seen
                    .get(name)
                    .copied()
                    .unwrap_or(agent.last_event_ts);
                command_last_seen
                    .entry(name.clone())
                    .and_modify(|current| *current = (*current).max(ts))
                    .or_insert(ts);
            }

            for (name, count) in &agent.skills_invoked {
                *skill_counts.entry(name.clone()).or_insert(0) += count;
                let ts = agent
                    .skill_last_seen
                    .get(name)
                    .copied()
                    .unwrap_or(agent.last_event_ts);
                skill_last_seen
                    .entry(name.clone())
                    .and_modify(|current| *current = (*current).max(ts))
                    .or_insert(ts);
            }
        }

        StageRankings {
            commands: sorted_rankings(command_counts, command_last_seen),
            skills: sorted_rankings(skill_counts, skill_last_seen),
        }
    }
}

fn sorted_rankings(
    counts: std::collections::HashMap<String, u64>,
    last_seen: std::collections::HashMap<String, i64>,
) -> Vec<RankedEntry> {
    let mut entries: Vec<_> = counts
        .into_iter()
        .map(|(name, count)| RankedEntry::new(name.clone(), count, last_seen.get(&name).copied().unwrap_or(0)))
        .collect();
    entries.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.last_seen.cmp(&a.last_seen))
            .then_with(|| a.name.cmp(&b.name))
    });
    entries
}

pub fn draw(f: &mut Frame, app: &mut App, snap: &StoreSnapshot) {
    let size = f.area();

    // Main layout: header (3) | body (fill)
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(size);

    // Header (packmen + stats)
    tab_bar::render_tab_bar(f, outer_chunks[0], app, snap);

    // Single view: stage
    stage::render_stage(f, outer_chunks[1], app, snap);

    // Filter overlay
    if app.show_filter {
        render_filter_input(f, size, app);
    }

    // Agent detail overlay
    if app.show_detail_overlay {
        agent_detail::render_agent_detail(f, app, snap);
    }
}

fn render_filter_input(f: &mut Frame, area: Rect, app: &App) {
    let width = 40u16.min(area.width.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height.saturating_sub(5);
    let popup_area = Rect::new(x, y, width, 3);

    f.render_widget(ratatui::widgets::Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Filter ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let input = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Yellow)),
        Span::styled(
            app.filter_text.clone(),
            Style::default().fg(Color::White),
        ),
        Span::styled("_", Style::default().fg(Color::Yellow)),
    ]))
    .block(block);

    f.render_widget(input, popup_area);
}

#[cfg(test)]
mod tests {
    use super::{normalize_ranked_command, RankedEntry, StageRankings, StoreSnapshot};
    use crate::protocol::types::{Agent, AgentRole, AgentState, FeedEvent, IngestSource, RuntimeEventType, SkillKind};
    use std::collections::HashMap;

    fn make_agent(
        agent_id: &str,
        cwd: &str,
        last_event_ts: i64,
        commands: &[(&str, u64, i64)],
        skills: &[(&str, u64, i64)],
    ) -> Agent {
        let mut command_usage = HashMap::new();
        let mut command_last_seen = HashMap::new();
        for (name, count, ts) in commands {
            command_usage.insert((*name).to_string(), *count);
            command_last_seen.insert((*name).to_string(), *ts);
        }

        let mut skills_invoked = HashMap::new();
        let mut skill_last_seen = HashMap::new();
        for (name, count, ts) in skills {
            skills_invoked.insert((*name).to_string(), *count);
            skill_last_seen.insert((*name).to_string(), *ts);
        }

        Agent {
            agent_id: agent_id.to_string(),
            display_name: agent_id.to_string(),
            short_id: agent_id.chars().take(8).collect(),
            state: AgentState::Active,
            role: AgentRole::Subagent,
            current_skill: Some(SkillKind::Bash),
            branch_name: None,
            skill_usage: HashMap::new(),
            skills_invoked,
            skill_last_seen,
            command_usage,
            command_last_seen,
            total_tokens: 0,
            usage_count: 0,
            tool_run_count: 0,
            last_event_ts,
            context_percent: None,
            cost_usd: None,
            model_name: None,
            cwd: Some(cwd.to_string()),
            ai_tool: None,
            parent_session_id: Some("lead".to_string()),
        }
    }

    fn empty_snapshot(agents: Vec<Agent>) -> StoreSnapshot {
        StoreSnapshot {
            agents,
            feed: Vec::<FeedEvent>::new(),
            sessions: Vec::new(),
            sparkline: Vec::new(),
            metrics: crate::store::metrics::DerivedMetrics {
                total_agents: 0,
                active_agents: 0,
                waiting_agents: 0,
                completed_agents: 0,
                total_events: 0,
                total_tokens: 0,
                total_cost: 0.0,
                avg_context_percent: 0.0,
                velocity_per_min: 0,
            },
            available_skills: Vec::new(),
        }
    }

    #[test]
    fn normalize_ranked_command_uses_subcommands_for_common_tools() {
        assert_eq!(normalize_ranked_command("git diff --stat src/main.rs"), Some("git diff".into()));
        assert_eq!(normalize_ranked_command("cargo test stage::tests"), Some("cargo test".into()));
        assert_eq!(normalize_ranked_command("pnpm dev --port 3001"), Some("pnpm dev".into()));
        assert_eq!(normalize_ranked_command("rg focused_agent src/tui"), Some("rg".into()));
        assert_eq!(normalize_ranked_command("  "), None);
    }

    #[test]
    fn stage_rankings_sort_by_count_then_last_seen() {
        let snap = empty_snapshot(vec![
            make_agent(
                "agent-a",
                "/tmp/project-a",
                50,
                &[("git diff", 2, 20), ("cargo test", 2, 30)],
                &[("superpowers:brainstorming", 1, 10)],
            ),
            make_agent(
                "agent-b",
                "/tmp/project-a",
                60,
                &[("cargo test", 1, 40), ("rg", 3, 50)],
                &[("superpowers:brainstorming", 2, 60), ("commit", 2, 55)],
            ),
        ]);

        let StageRankings { commands, skills } = snap.stage_rankings(Some("project-a"), None);

        assert_eq!(
            commands,
            vec![
                RankedEntry::new("rg", 3, 50),
                RankedEntry::new("cargo test", 3, 40),
                RankedEntry::new("git diff", 2, 20),
            ]
        );
        assert_eq!(
            skills,
            vec![
                RankedEntry::new("superpowers:brainstorming", 3, 60),
                RankedEntry::new("commit", 2, 55),
            ]
        );
    }

    #[test]
    fn stage_rankings_can_focus_on_single_agent() {
        let snap = empty_snapshot(vec![
            make_agent(
                "agent-a",
                "/tmp/project-a",
                50,
                &[("git diff", 2, 20)],
                &[("commit", 1, 10)],
            ),
            make_agent(
                "agent-b",
                "/tmp/project-a",
                60,
                &[("cargo test", 3, 40)],
                &[("superpowers:brainstorming", 2, 60)],
            ),
        ]);

        let StageRankings { commands, skills } = snap.stage_rankings(Some("project-a"), Some("agent-b"));

        assert_eq!(commands, vec![RankedEntry::new("cargo test", 3, 40)]);
        assert_eq!(skills, vec![RankedEntry::new("superpowers:brainstorming", 2, 60)]);
    }

    #[test]
    fn placeholder_feed_event_keeps_test_fixture_complete() {
        let event = FeedEvent {
            id: "event-1".into(),
            ts: 1,
            agent_id: "agent-a".into(),
            display_name: "agent-a".into(),
            short_id: "agent-a".into(),
            skill: None,
            event_type: RuntimeEventType::ToolStart,
            tool_name: Some("Bash".into()),
            file_path: None,
            detail: Some("git diff".into()),
            total_tokens: None,
            is_error: false,
            ingest_source: IngestSource::Jsonl,
            ai_tool: Some("codex".into()),
        };

        assert_eq!(event.tool_name.as_deref(), Some("Bash"));
    }
}
