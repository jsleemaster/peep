use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;

pub fn render_status_bar(f: &mut Frame, area: Rect, _app: &App, snap: &StoreSnapshot) {
    let m = &snap.metrics;

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let spans = vec![
        Span::raw(" "),
        Span::styled("agents:", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", m.total_agents), Style::default().fg(Color::White)),
        Span::styled(
            format!(" ({}^ {}v)", m.active_agents, m.completed_agents),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled("events:", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", m.total_events), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled("tokens:", Style::default().fg(Color::DarkGray)),
        Span::styled(
            AppStore::format_tokens(m.total_tokens),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("cost:", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("${:.2}", m.total_cost), Style::default().fg(Color::Green)),
        Span::raw("  "),
        Span::styled(
            format!("{}/m", m.velocity_per_min),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            "q:quit Tab:switch j/k:scroll h/l:focus Enter:detail f:filter",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
