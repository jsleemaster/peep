use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::protocol::types::AgentState;
use crate::store::state::AppStore;
use crate::tui::app::{App, Tab};
use crate::tui::render::StoreSnapshot;
use crate::tui::widgets::stage;

pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let m = &snap.metrics;

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    // On Stage tab, show party composition
    if app.active_tab == Tab::Stage && !snap.agents.is_empty() {
        let summary = stage::party_summary(snap);

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

        let sep = Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray));

        let spans = vec![
            Span::raw(" "),
            Span::styled("party:", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ", snap.agents.len()),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("({})", summary),
                Style::default().fg(Color::DarkGray),
            ),
            sep.clone(),
            Span::styled(
                format!("\u{25cf}{}", active_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" "),
            Span::styled(
                format!("\u{25d0}{}", waiting_count),
                Style::default().fg(Color::Yellow),
            ),
            sep.clone(),
            Span::styled("tokens:", Style::default().fg(Color::DarkGray)),
            Span::styled(
                AppStore::format_tokens(m.total_tokens),
                Style::default().fg(Color::White),
            ),
            sep.clone(),
            Span::styled("cost:", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", m.total_cost),
                Style::default().fg(Color::Rgb(255, 220, 80)),
            ),
        ];

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).block(block);
        f.render_widget(paragraph, area);
        return;
    }

    // Mini sparkline for recent velocity
    let spark_str = if !snap.sparkline.is_empty() {
        let spark_chars = ['\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
        let max_val = snap.sparkline.iter().max().copied().unwrap_or(1).max(1);
        snap.sparkline
            .iter()
            .map(|&v| {
                let idx =
                    ((v as f64 / max_val as f64) * (spark_chars.len() - 1) as f64) as usize;
                spark_chars[idx.min(spark_chars.len() - 1)]
            })
            .collect::<String>()
    } else {
        String::new()
    };

    let sep = Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray));

    let spans = vec![
        Span::raw(" "),
        Span::styled("agents:", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", m.total_agents),
            Style::default().fg(Color::White),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25cf}{}", m.active_agents),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" "),
        Span::styled(
            format!("\u{25d0}{}", m.waiting_agents),
            Style::default().fg(Color::Yellow),
        ),
        sep.clone(),
        Span::styled("events:", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", m.total_events),
            Style::default().fg(Color::Cyan),
        ),
        sep.clone(),
        Span::styled("tokens:", Style::default().fg(Color::DarkGray)),
        Span::styled(
            AppStore::format_tokens(m.total_tokens),
            Style::default().fg(Color::White),
        ),
        sep.clone(),
        Span::styled("cost:", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("${:.2}", m.total_cost),
            Style::default().fg(Color::Green),
        ),
        sep.clone(),
        Span::styled(spark_str, Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::styled(
            format!("{}/m", m.velocity_per_min),
            Style::default().fg(Color::Green),
        ),
        sep,
        Span::styled(
            "q:quit Tab:switch j/k:scroll h/l:focus Enter:detail f:filter",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
