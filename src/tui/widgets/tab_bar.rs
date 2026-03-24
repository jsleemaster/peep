use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, Tab};
use crate::tui::render::StoreSnapshot;

pub fn render_tab_bar(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let m = &snap.metrics;

    let mut spans = Vec::new();
    spans.push(Span::raw(" "));

    for tab in Tab::all() {
        let is_active = *tab == app.active_tab;

        // Indicator counts next to tab label
        let label = match tab {
            Tab::Stage => format!(" Stage ({}) ", m.total_agents),
            Tab::Feed => format!(" Feed ({}) ", snap.feed.len()),
            Tab::Agents => format!(" Agents ({}) ", m.total_agents),
            Tab::Sessions => format!(" Sessions ({}) ", snap.sessions.len()),
        };

        let style = if is_active {
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Gray)
        };

        spans.push(Span::styled(label, style));
        spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
    }

    // Right-aligned stats
    let right_stats = format!(
        "ctx:{:.0}% │ ${:.2} │ ●{}",
        m.avg_context_percent, m.total_cost, m.active_agents,
    );
    let left_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let padding = (area.width as usize).saturating_sub(left_len + right_stats.chars().count() + 2);
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(
        right_stats,
        Style::default().fg(Color::Cyan),
    ));
    spans.push(Span::raw(" "));

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" packmen ")
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
