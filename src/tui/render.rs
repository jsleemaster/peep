use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::protocol::types::{Agent, FeedEvent, Session};
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
