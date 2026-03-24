use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::protocol::types::AgentState;
use crate::store::state::AppStore;
use crate::tui::app::{App, FocusPane};
use crate::tui::render::StoreSnapshot;

pub fn render_agent_sidebar(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let is_focused = app.focus == FocusPane::Sidebar;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Agents ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    if snap.agents.is_empty() {
        lines.push(Line::from(Span::styled(
            " (none)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, agent) in snap.agents.iter().enumerate() {
            let (icon, icon_color) = match agent.state {
                AgentState::Active => ("*", Color::Green),
                AgentState::Waiting => ("o", Color::Yellow),
                AgentState::Completed => ("v", Color::DarkGray),
            };

            let state_str = format!("{}", agent.state);
            let is_selected = i == app.sidebar_selected;

            let name_style = if is_selected && is_focused {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let state_style = Style::default().fg(match agent.state {
                AgentState::Active => Color::Green,
                AgentState::Waiting => Color::Yellow,
                AgentState::Completed => Color::DarkGray,
            });

            let line = Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(icon_color)),
                Span::styled(format!("{:<8}", agent.short_id), name_style),
                Span::styled(format!(" {}", state_str), state_style),
            ]);
            lines.push(line);
        }
    }

    // Metrics section
    let agent_list_height = snap.agents.len().min(inner.height as usize);
    let metrics_height = (inner.height as usize).saturating_sub(agent_list_height);

    if metrics_height > 1 {
        lines.push(Line::raw(""));

        // Context gauge (text-based)
        let ctx = snap.metrics.avg_context_percent;
        let filled = ((ctx / 100.0) * 10.0) as usize;
        let empty = 10_usize.saturating_sub(filled);
        let gauge_str = format!(
            " ctx {}{} {:.0}%",
            "#".repeat(filled),
            "-".repeat(empty),
            ctx
        );
        lines.push(Line::from(Span::styled(
            gauge_str,
            Style::default().fg(Color::Cyan),
        )));

        // Cost and tokens
        lines.push(Line::from(Span::styled(
            format!(
                " ${:.2} {} tk",
                snap.metrics.total_cost,
                AppStore::format_tokens(snap.metrics.total_tokens)
            ),
            Style::default().fg(Color::White),
        )));

        // Velocity sparkline (text representation)
        if metrics_height > 4 && !snap.sparkline.is_empty() {
            let spark_chars = ['_', '.', '-', '~', '+', '=', '#', '@'];
            let max_val = snap.sparkline.iter().max().copied().unwrap_or(1).max(1);
            let spark: String = snap
                .sparkline
                .iter()
                .map(|&v| {
                    let idx =
                        ((v as f64 / max_val as f64) * (spark_chars.len() - 1) as f64) as usize;
                    spark_chars[idx.min(spark_chars.len() - 1)]
                })
                .collect();
            lines.push(Line::from(Span::styled(
                format!(" {} {}/m", spark, snap.metrics.velocity_per_min),
                Style::default().fg(Color::Green),
            )));
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}
