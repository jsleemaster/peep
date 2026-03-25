use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;
use crate::store::state::AppStore;

pub fn render_tab_bar(f: &mut Frame, area: Rect, _app: &App, snap: &StoreSnapshot) {
    let m = &snap.metrics;

    let mut spans = vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("agents:{} ", m.total_agents), Style::default().fg(Color::Rgb(140, 140, 160))),
        Span::styled(format!("\u{25cf}{} ", m.active_agents), Style::default().fg(Color::Green)),
        Span::styled(format!("\u{25d0}{}", m.waiting_agents), Style::default().fg(Color::Yellow)),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("tokens:{} ", AppStore::format_tokens(m.total_tokens)), Style::default().fg(Color::Rgb(140, 140, 160))),
        Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("${:.2}", m.total_cost), Style::default().fg(Color::Rgb(100, 220, 140))),
    ];

    // Right-aligned: keybindings hint
    let hint = "q:quit j/k:scroll [,]:project h/l:focus";
    let left_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let padding = (area.width as usize).saturating_sub(left_len + hint.chars().count() + 2);
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(hint, Style::default().fg(Color::Rgb(60, 60, 80))));
    spans.push(Span::raw(" "));

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
