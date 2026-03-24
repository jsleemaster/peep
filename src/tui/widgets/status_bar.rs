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

    // Mini sparkline for recent velocity
    let spark_str = if !snap.sparkline.is_empty() {
        let spark_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
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

    let sep = Span::styled(" │ ", Style::default().fg(Color::DarkGray));

    let spans = vec![
        Span::raw(" "),
        Span::styled("agents:", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", m.total_agents), Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(format!("●{}", m.active_agents), Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::styled(format!("◐{}", m.waiting_agents), Style::default().fg(Color::Yellow)),
        sep.clone(),
        Span::styled("events:", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", m.total_events), Style::default().fg(Color::Cyan)),
        sep.clone(),
        Span::styled("tokens:", Style::default().fg(Color::DarkGray)),
        Span::styled(
            AppStore::format_tokens(m.total_tokens),
            Style::default().fg(Color::White),
        ),
        sep.clone(),
        Span::styled("cost:", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("${:.2}", m.total_cost), Style::default().fg(Color::Green)),
        sep.clone(),
        Span::styled(spark_str, Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::styled(
            format!("{}/m", m.velocity_per_min),
            Style::default().fg(Color::Green),
        ),
        sep.clone(),
        Span::styled(
            "q:quit Tab:switch j/k:scroll h/l:focus Enter:detail f:filter",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
