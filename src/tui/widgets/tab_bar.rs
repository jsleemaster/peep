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
    let mut spans = Vec::new();
    spans.push(Span::raw(" "));

    for tab in Tab::all() {
        let is_active = *tab == app.active_tab;
        let style = if is_active {
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        spans.push(Span::styled(format!(" {} ", tab.label()), style));
        spans.push(Span::raw(" "));
    }

    // Right-aligned stats
    let m = &snap.metrics;
    let right_stats = format!(
        "ctx:{:.0}% ${:.2} ^{}",
        m.avg_context_percent, m.total_cost, m.active_agents,
    );
    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let padding = (area.width as usize).saturating_sub(left_len + right_stats.len() + 2);
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
