use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[cfg(test)]
pub(crate) use crate::protocol::normalize::normalize_ranked_command;
use crate::protocol::types::{Agent, FeedEvent, Session};
use crate::store::analytics::{AnalyticsQuery, AnalyticsWindow, SharedAnalytics};
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
    pub rankings: StageRankings,
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
    pub window: AnalyticsWindow,
    pub agents_used: usize,
    pub completed: usize,
    pub commands: Vec<RankedEntry>,
    pub skills: Vec<RankedEntry>,
    pub agents: Vec<RankedEntry>,
    pub warming: bool,
}

impl StoreSnapshot {
    pub async fn from_stores(
        store: &crate::store::state::SharedStore,
        analytics: &SharedAnalytics,
        project: Option<&str>,
        focused_agent: Option<&str>,
        window: AnalyticsWindow,
    ) -> Self {
        let s = store.read().await;
        let now = chrono::Utc::now().timestamp();
        let agents = s.sorted_agents().into_iter().cloned().collect();
        let feed: Vec<FeedEvent> = s.feed.iter().cloned().collect();
        let sessions = s.sessions.clone();
        let sparkline = s.velocity_sparkline_data(15, now);
        let metrics = s.derived_metrics(now);
        let available_skills = s.available_skills.clone();
        drop(s);

        let rankings_view =
            analytics
                .read()
                .await
                .query(AnalyticsQuery::new(window, project, focused_agent, now));
        StoreSnapshot {
            agents,
            feed,
            sessions,
            sparkline,
            metrics,
            available_skills,
            rankings: StageRankings {
                window: rankings_view.summary.window,
                agents_used: rankings_view.summary.agents_used,
                completed: rankings_view.summary.completed,
                commands: analytics_entries(rankings_view.commands),
                skills: analytics_entries(rankings_view.skills),
                agents: analytics_entries(rankings_view.agents),
                warming: rankings_view.warming,
            },
        }
    }
}

fn analytics_entries(entries: Vec<crate::store::analytics::AnalyticsEntry>) -> Vec<RankedEntry> {
    entries
        .into_iter()
        .map(|entry| RankedEntry::new(entry.name, entry.count, entry.last_seen))
        .collect()
}

pub fn draw(f: &mut Frame, app: &mut App, snap: &StoreSnapshot) {
    let size = f.area();

    // Main layout: header (3) | body (fill)
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
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
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let input = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Yellow)),
        Span::styled(app.filter_text.clone(), Style::default().fg(Color::White)),
        Span::styled("_", Style::default().fg(Color::Yellow)),
    ]))
    .block(block);

    f.render_widget(input, popup_area);
}

#[cfg(test)]
mod tests {
    use super::{draw, normalize_ranked_command, StageRankings, StoreSnapshot};
    use crate::protocol::types::{FeedEvent, IngestSource, RawIngestEvent, RuntimeEventType};
    use crate::store::analytics::{AnalyticsQuery, AnalyticsStore, AnalyticsWindow};
    use crate::store::metrics::DerivedMetrics;
    use crate::tui::app::App;
    use crate::tui::theme::{init_theme, Theme};
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::Once;

    fn ensure_theme() {
        static INIT: Once = Once::new();
        INIT.call_once(|| init_theme(Theme::dark()));
    }

    fn empty_snapshot() -> StoreSnapshot {
        ensure_theme();
        StoreSnapshot {
            agents: Vec::new(),
            feed: Vec::new(),
            sessions: Vec::new(),
            sparkline: Vec::new(),
            metrics: DerivedMetrics {
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
            rankings: StageRankings::default(),
        }
    }

    fn raw_event(
        agent_id: &str,
        ts: i64,
        tool_name: Option<&str>,
        detail: Option<&str>,
        cwd: &str,
    ) -> RawIngestEvent {
        RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: agent_id.to_string(),
            session_runtime_id: Some(format!("session-{agent_id}")),
            ts,
            event_type: RuntimeEventType::ToolStart,
            hook_event_name: Some("PreToolUse".into()),
            tool_name: tool_name.map(str::to_string),
            file_path: None,
            detail: detail.map(str::to_string),
            total_tokens: None,
            is_error: false,
            branch_name: None,
            slug: Some(agent_id.to_string()),
            cwd: Some(cwd.to_string()),
            ai_tool: Some("codex".into()),
        }
    }

    #[test]
    fn normalize_ranked_command_uses_subcommands_for_common_tools() {
        assert_eq!(
            normalize_ranked_command("git diff --stat src/main.rs"),
            Some("git diff".into())
        );
        assert_eq!(
            normalize_ranked_command("cargo test stage::tests"),
            Some("cargo test".into())
        );
        assert_eq!(
            normalize_ranked_command("pnpm dev --port 3001"),
            Some("pnpm dev".into())
        );
        assert_eq!(
            normalize_ranked_command("rg focused_agent src/tui"),
            Some("rg".into())
        );
        assert_eq!(normalize_ranked_command("  "), None);
    }

    #[test]
    fn stage_rankings_sort_by_count_then_last_seen() {
        let mut analytics = AnalyticsStore::default();
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-a",
                20,
                Some("Bash"),
                Some("git diff src/main.rs"),
                "/tmp/project-a",
            ),
            "agent-a",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-a",
                30,
                Some("Bash"),
                Some("cargo test render::tests"),
                "/tmp/project-a",
            ),
            "agent-a",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-a",
                10,
                Some("Skill"),
                Some("superpowers:brainstorming scope"),
                "/tmp/project-a",
            ),
            "agent-a",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                40,
                Some("Bash"),
                Some("cargo test stage::tests"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                50,
                Some("Bash"),
                Some("rg focused_agent src/tui"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                60,
                Some("Skill"),
                Some("superpowers:brainstorming revise"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                55,
                Some("Skill"),
                Some("commit stage files"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );

        let view = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Hours24,
            Some("project-a"),
            None,
            100,
        ));

        assert_eq!(
            view.commands,
            vec![
                crate::store::analytics::AnalyticsEntry::new("cargo test", 2, 40),
                crate::store::analytics::AnalyticsEntry::new("rg", 1, 50),
                crate::store::analytics::AnalyticsEntry::new("git diff", 1, 20),
            ]
        );
        assert_eq!(
            view.skills,
            vec![
                crate::store::analytics::AnalyticsEntry::new("superpowers:brainstorming", 2, 60),
                crate::store::analytics::AnalyticsEntry::new("commit", 1, 55),
            ]
        );
    }

    #[test]
    fn stage_rankings_can_focus_on_single_agent() {
        let mut analytics = AnalyticsStore::default();
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-a",
                20,
                Some("Bash"),
                Some("git diff src/main.rs"),
                "/tmp/project-a",
            ),
            "agent-a",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                40,
                Some("Bash"),
                Some("cargo test stage::tests"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                60,
                Some("Skill"),
                Some("superpowers:brainstorming revise"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );

        let view = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Hours24,
            Some("project-a"),
            Some("agent-b"),
            100,
        ));

        assert_eq!(
            view.commands,
            vec![crate::store::analytics::AnalyticsEntry::new(
                "cargo test",
                1,
                40
            )]
        );
        assert_eq!(
            view.skills,
            vec![crate::store::analytics::AnalyticsEntry::new(
                "superpowers:brainstorming",
                1,
                60,
            )]
        );
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

    #[test]
    fn draw_does_not_panic_on_narrow_empty_terminal() {
        let backend = TestBackend::new(16, 82);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(8080);
        let snap = empty_snapshot();

        terminal.draw(|frame| draw(frame, &mut app, &snap)).unwrap();
    }

    #[test]
    fn draw_does_not_panic_on_medium_empty_terminal() {
        let backend = TestBackend::new(38, 83);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(8080);
        let snap = empty_snapshot();

        terminal.draw(|frame| draw(frame, &mut app, &snap)).unwrap();
    }

    #[test]
    fn draw_renders_non_empty_output_with_agent_present() {
        use crate::protocol::types::{Agent, AgentRole, AgentState, SkillKind};
        use crate::store::metrics::DerivedMetrics;
        use ratatui::{backend::TestBackend, Terminal};
        use std::collections::HashMap;

        ensure_theme();
        let backend = TestBackend::new(60, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(8080);
        let snap = StoreSnapshot {
            agents: vec![Agent {
                agent_id: "lead".into(),
                display_name: "lead".into(),
                short_id: "lead".into(),
                first_seen_ts: 0,
                state: AgentState::Active,
                role: AgentRole::Main,
                current_skill: Some(SkillKind::Bash),
                branch_name: None,
                skill_usage: HashMap::new(),
                skills_invoked: HashMap::new(),
                skill_last_seen: HashMap::new(),
                command_usage: HashMap::new(),
                command_last_seen: HashMap::new(),
                total_tokens: 100,
                usage_count: 5,
                tool_run_count: 1,
                last_event_ts: 1,
                completed_at: None,
                completed_visible_until: None,
                completion_recorded: false,
                context_percent: Some(40.0),
                cost_usd: None,
                model_name: None,
                cwd: Some("/tmp/project-a".into()),
                ai_tool: Some("codex".into()),
                parent_session_id: None,
            }],
            feed: Vec::new(),
            sessions: Vec::new(),
            sparkline: Vec::new(),
            metrics: DerivedMetrics {
                total_agents: 1,
                active_agents: 1,
                waiting_agents: 0,
                completed_agents: 0,
                total_events: 0,
                total_tokens: 100,
                total_cost: 0.0,
                avg_context_percent: 40.0,
                velocity_per_min: 0,
            },
            available_skills: Vec::new(),
            rankings: StageRankings::default(),
        };

        terminal.draw(|frame| draw(frame, &mut app, &snap)).unwrap();
    }

    #[test]
    fn draw_does_not_panic_on_medium_terminal_with_agent_present() {
        use crate::protocol::types::{Agent, AgentRole, AgentState, SkillKind};
        use crate::store::metrics::DerivedMetrics;
        use ratatui::{backend::TestBackend, Terminal};
        use std::collections::HashMap;

        ensure_theme();
        let backend = TestBackend::new(38, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(8080);
        let snap = StoreSnapshot {
            agents: vec![Agent {
                agent_id: "lead".into(),
                display_name: "lead".into(),
                short_id: "lead".into(),
                first_seen_ts: 0,
                state: AgentState::Active,
                role: AgentRole::Main,
                current_skill: Some(SkillKind::Bash),
                branch_name: None,
                skill_usage: HashMap::new(),
                skills_invoked: HashMap::new(),
                skill_last_seen: HashMap::new(),
                command_usage: HashMap::new(),
                command_last_seen: HashMap::new(),
                total_tokens: 100,
                usage_count: 5,
                tool_run_count: 1,
                last_event_ts: 1,
                completed_at: None,
                completed_visible_until: None,
                completion_recorded: false,
                context_percent: Some(40.0),
                cost_usd: None,
                model_name: None,
                cwd: Some("/tmp/project-a".into()),
                ai_tool: Some("codex".into()),
                parent_session_id: None,
            }],
            feed: Vec::new(),
            sessions: Vec::new(),
            sparkline: Vec::new(),
            metrics: DerivedMetrics {
                total_agents: 1,
                active_agents: 1,
                waiting_agents: 0,
                completed_agents: 0,
                total_events: 0,
                total_tokens: 100,
                total_cost: 0.0,
                avg_context_percent: 40.0,
                velocity_per_min: 0,
            },
            available_skills: Vec::new(),
            rankings: StageRankings::default(),
        };

        terminal.draw(|frame| draw(frame, &mut app, &snap)).unwrap();
    }
}
