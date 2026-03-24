use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::protocol::types::*;
use crate::store::state::AppStore;
use crate::tui::app::{App, Tab};
use crate::tui::widgets::{agent_detail, agent_sidebar, feed_table, session_table, stage, status_bar, tab_bar};

/// A snapshot of the store for rendering (avoids holding the lock during draw).
pub struct StoreSnapshot {
    pub agents: Vec<Agent>,
    pub feed: Vec<FeedEvent>,
    pub sessions: Vec<Session>,
    pub sparkline: Vec<u64>,
    pub metrics: crate::store::metrics::DerivedMetrics,
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
        StoreSnapshot {
            agents,
            feed,
            sessions,
            sparkline,
            metrics,
        }
    }
}

pub fn draw(f: &mut Frame, app: &App, snap: &StoreSnapshot) {
    let size = f.area();

    // Main layout: tab bar (3) | body (fill) | status bar (2)
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .split(size);

    // Tab bar
    tab_bar::render_tab_bar(f, outer_chunks[0], app, snap);

    // Body depends on active tab
    match app.active_tab {
        Tab::Stage => stage::render_stage(f, outer_chunks[1], app, snap),
        Tab::Feed => render_feed_layout(f, outer_chunks[1], app, snap),
        Tab::Agents => render_agents_tab(f, outer_chunks[1], app, snap),
        Tab::Sessions => session_table::render_session_table(f, outer_chunks[1], app, snap),
    }

    // Status bar
    status_bar::render_status_bar(f, outer_chunks[2], app, snap);

    // Filter overlay
    if app.show_filter {
        render_filter_input(f, size, app);
    }

    // Agent detail overlay
    if app.show_detail_overlay {
        agent_detail::render_agent_detail(f, app, snap);
    }
}

fn render_feed_layout(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    if snap.agents.is_empty() && snap.feed.is_empty() {
        // Empty state
        render_empty_state(f, area, app.port);
        return;
    }

    // 25% sidebar | 75% feed
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    agent_sidebar::render_agent_sidebar(f, chunks[0], app, snap);
    feed_table::render_feed_table(f, chunks[1], app, snap);
}

fn render_empty_state(f: &mut Frame, area: Rect, port: u16) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" packmen ")
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

    let lines = vec![
        Line::raw(""),
        Line::raw(""),
        Line::from(Span::styled(
            format!("  Waiting for events on :{} ...", port),
            Style::default().fg(Color::DarkGray),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  Send a test hook:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            format!(
                "  curl -s -X POST http://localhost:{}/hook \\",
                port
            ),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "    -H 'Content-Type: application/json' \\",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "    -d '{\"hook_event_name\":\"PostToolUse\",\"session_id\":\"test\",\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"src/main.rs\"}}'",
            Style::default().fg(Color::Cyan),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  q: quit  Tab: switch tabs  j/k: scroll  h/l: focus",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_agents_tab(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    use ratatui::widgets::{Cell, Row, Table, TableState};

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" All Agents ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    if snap.agents.is_empty() {
        let inner = Paragraph::new(Line::from(Span::styled(
            "  No agents yet. Waiting for hook events...",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(inner, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("STATUS").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("NAME").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("ROLE").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("SKILL").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("CTX").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("TOKENS").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("TOOLS").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("COST").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("MODEL").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("BRANCH").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    let rows: Vec<Row> = snap
        .agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let (icon, state_color) = match agent.state {
                AgentState::Active => ("● active", Color::Green),
                AgentState::Waiting => ("◐ waiting", Color::Yellow),
                AgentState::Completed => ("✓ done", Color::DarkGray),
            };

            let ctx_str = agent
                .context_percent
                .map(|c| format!("{:.0}%", c))
                .unwrap_or_else(|| "-".into());

            let row_style = if i == app.agents_tab_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(icon.to_string()).style(Style::default().fg(state_color)),
                Cell::from(agent.display_name.clone()).style(Style::default().fg(Color::Blue)),
                Cell::from(format!("{}", agent.role)).style(Style::default().fg(Color::White)),
                Cell::from(
                    agent
                        .current_skill
                        .map(|s| format!("{}", s))
                        .unwrap_or_else(|| "-".into()),
                )
                .style(Style::default().fg(Color::Cyan)),
                Cell::from(ctx_str).style(Style::default().fg(Color::Yellow)),
                Cell::from(AppStore::format_tokens(agent.total_tokens))
                    .style(Style::default().fg(Color::White)),
                Cell::from(format!("{}", agent.tool_run_count))
                    .style(Style::default().fg(Color::White)),
                Cell::from(
                    agent
                        .cost_usd
                        .map(|c| format!("${:.2}", c))
                        .unwrap_or_else(|| "-".into()),
                )
                .style(Style::default().fg(Color::Green)),
                Cell::from(agent.model_name.clone().unwrap_or_else(|| "-".into()))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(agent.branch_name.clone().unwrap_or_else(|| "-".into()))
                    .style(Style::default().fg(Color::Green)),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(11),
        Constraint::Length(14),
        Constraint::Length(6),
        Constraint::Length(7),
        Constraint::Length(5),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(7),
        Constraint::Length(26),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    state.select(Some(app.agents_tab_selected));

    f.render_stateful_widget(table, area, &mut state);
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
