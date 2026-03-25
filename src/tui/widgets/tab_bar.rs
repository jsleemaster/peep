use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::protocol::types::AgentState;
use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;
use crate::tui::widgets::stage;

pub fn render_tab_bar(f: &mut Frame, area: Rect, _app: &App, snap: &StoreSnapshot) {
    let m = &snap.metrics;

    // Line 1: packmen title
    // Line 2: stats (agents, party, tokens, cost, keybindings)
    let title_line = Line::from(vec![
        Span::styled(" packmen", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
    ]);

    let active_count = snap.agents.iter().filter(|a| a.state == AgentState::Active).count();
    let waiting_count = snap.agents.iter().filter(|a| a.state == AgentState::Waiting).count();
    let party_summary = if !snap.agents.is_empty() {
        stage::party_summary(snap)
    } else {
        String::new()
    };

    let sep = Span::styled(" \u{2502} ", Style::default().fg(Color::Rgb(50, 50, 70)));

    let mut stats = vec![
        Span::styled(" ", Style::default()),
        Span::styled(format!("\u{25cf}{}", active_count), Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::styled(format!("\u{25d0}{}", waiting_count), Style::default().fg(Color::Yellow)),
    ];

    if !party_summary.is_empty() {
        stats.push(sep.clone());
        stats.push(Span::styled(party_summary, Style::default().fg(Color::Rgb(180, 180, 200))));
    }

    stats.push(sep.clone());
    stats.push(Span::styled(
        format!("tokens:{}", AppStore::format_tokens(m.total_tokens)),
        Style::default().fg(Color::Rgb(140, 140, 160)),
    ));

    stats.push(sep.clone());
    stats.push(Span::styled(
        format!("${:.2}", m.total_cost),
        Style::default().fg(Color::Rgb(100, 220, 140)),
    ));

    // Right-aligned keybindings
    let hint = "q:quit j/k:scroll [,]:project";
    let left_len: usize = stats.iter().map(|s| s.content.chars().count()).sum();
    let padding = (area.width as usize).saturating_sub(left_len + hint.chars().count() + 2);
    stats.push(Span::raw(" ".repeat(padding)));
    stats.push(Span::styled(hint, Style::default().fg(Color::White)));

    let stats_line = Line::from(stats);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Rgb(50, 50, 70)))
        .style(Style::default().bg(Color::Rgb(22, 22, 34)));

    let paragraph = Paragraph::new(vec![title_line, stats_line]).block(block);
    f.render_widget(paragraph, area);
}
