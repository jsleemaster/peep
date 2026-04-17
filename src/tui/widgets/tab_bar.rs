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
use crate::tui::theme::theme;
use crate::tui::widgets::stage;

pub fn render_tab_bar(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let m = &snap.metrics;
    let t = theme();

    let mut title_spans = vec![
        Span::styled(
            " peep",
            Style::default().fg(t.brand).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" v{}", crate::update::UpdateStatus::current()),
            Style::default().fg(t.text_dim),
        ),
    ];
    if let Some(ref new_ver) = app.update_available {
        title_spans.push(Span::styled(
            format!(" → v{} available! (brew upgrade peep)", new_ver),
            Style::default().fg(t.accent_green),
        ));
    }
    let title_line = Line::from(title_spans);

    let active_count = snap
        .agents
        .iter()
        .filter(|a| a.state == AgentState::Active)
        .count();
    let waiting_count = snap
        .agents
        .iter()
        .filter(|a| a.state == AgentState::Waiting)
        .count();
    let party_summary = if !snap.agents.is_empty() {
        stage::party_summary(snap)
    } else {
        String::new()
    };

    let sep = Span::styled(" \u{2502} ", Style::default().fg(t.border));

    // Show branch from the most recently active agent
    let branch = snap
        .agents
        .iter()
        .filter(|a| a.state == AgentState::Active)
        .max_by_key(|a| a.last_event_ts)
        .or_else(|| snap.agents.first())
        .and_then(|a| a.branch_name.clone());

    let mut stats = vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            format!("\u{25cf}{}", active_count),
            Style::default().fg(t.accent_green),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25d0}{}", waiting_count),
            Style::default().fg(t.accent_yellow),
        ),
    ];

    if !party_summary.is_empty() {
        stats.push(sep.clone());
        stats.push(Span::styled(
            party_summary,
            Style::default().fg(t.text_muted),
        ));
    }

    if let Some(br) = branch {
        stats.push(sep.clone());
        stats.push(Span::styled(
            format!("\u{e0a0} {}", br),
            Style::default().fg(t.accent_cyan),
        ));
    }

    stats.push(sep.clone());
    stats.push(Span::styled(
        format!("tokens:{}", AppStore::format_tokens(m.total_tokens)),
        Style::default().fg(Color::Rgb(220, 100, 100)), // soft red for token usage
    ));

    // Only show cost if non-zero
    if m.total_cost > 0.001 {
        stats.push(sep.clone());
        stats.push(Span::styled(
            format!("${:.2}", m.total_cost),
            Style::default().fg(t.accent_green),
        ));
    }

    let hint = "q:quit j/k:scroll Tab:section ,/.:window []:project Enter:filter";
    let left_len: usize = stats.iter().map(|s| s.content.chars().count()).sum();
    let padding = (area.width as usize).saturating_sub(left_len + hint.chars().count() + 2);
    stats.push(Span::raw(" ".repeat(padding)));
    stats.push(Span::styled(hint, Style::default().fg(t.text)));

    let stats_line = Line::from(stats);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.card_bg));

    let paragraph = Paragraph::new(vec![title_line, stats_line]).block(block);
    f.render_widget(paragraph, area);
}
