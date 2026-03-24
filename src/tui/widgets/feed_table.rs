use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use chrono::{Local, TimeZone};

use crate::protocol::types::{IngestSource, RuntimeEventType};
use crate::tui::app::{App, FocusPane};
use crate::tui::render::StoreSnapshot;

/// Format a unix timestamp as a relative time string (e.g. "2s", "5m", "2h").
/// Falls back to absolute HH:MM:SS for events older than 1 hour.
fn format_relative_ts(event_ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now.saturating_sub(event_ts);
    if diff < 60 {
        format!("{}s", diff)
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else {
        // Older than 1 hour — show absolute time
        Local
            .timestamp_opt(event_ts, 0)
            .single()
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "??:??:??".into())
    }
}

pub fn render_feed_table(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let is_focused = app.focus == FocusPane::MainPanel;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Apply filter if active
    let filter = app.filter_text.to_lowercase();
    let filtered_feed: Vec<_> = if filter.is_empty() {
        snap.feed.iter().collect()
    } else {
        snap.feed
            .iter()
            .filter(|e| {
                e.agent_id.to_lowercase().contains(&filter)
                    || e.short_id.to_lowercase().contains(&filter)
                    || e.tool_name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&filter)
                    || e.file_path
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&filter)
                    || e.detail
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&filter)
            })
            .collect()
    };

    let title = if !filter.is_empty() {
        format!(
            " Live Feed (filtered: {}/{}) ",
            filtered_feed.len(),
            snap.feed.len()
        )
    } else {
        " Live Feed ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title)
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    if filtered_feed.is_empty() {
        let msg = if !filter.is_empty() {
            "  No events match filter."
        } else {
            "  Waiting for events..."
        };
        let inner = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(inner, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("SRC").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("TIME").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("AGENT").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("TOOL").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("TARGET").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("DETAIL").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .height(1);

    // Determine the display scroll offset within the filtered view
    let display_offset = app.feed_scroll_offset.min(filtered_feed.len().saturating_sub(1));

    // Feed is already in chronological order (oldest first), display as-is
    let rows: Vec<Row> = filtered_feed
        .iter()
        .enumerate()
        .map(|(i, event)| {
            let ts_str = format_relative_ts(event.ts);

            // Event type dot badge
            let (dot_color, dot) = match event.event_type {
                RuntimeEventType::ToolStart => (Color::Cyan, "●"),
                RuntimeEventType::ToolDone => (Color::Green, "●"),
                RuntimeEventType::AssistantText => (Color::Blue, "●"),
                RuntimeEventType::PermissionWait => (Color::Red, "●"),
                _ => (Color::DarkGray, "●"),
            };

            // tool_color is not used separately; dot_color already reflects event type

            // Source indicator
            let src_str = match event.ingest_source {
                IngestSource::Http => "H",
                IngestSource::Jsonl => "J",
            };
            let src_color = match event.ingest_source {
                IngestSource::Http => Color::Cyan,
                IngestSource::Jsonl => Color::Green,
            };

            // Tool name with dot badge
            let tool_display = format!(
                "{} {}",
                dot,
                event.tool_name.as_deref().unwrap_or("")
            );

            let row_style = if event.is_error {
                Style::default()
                    .fg(Color::Red)
                    .bg(Color::Rgb(60, 0, 0))
            } else if i == display_offset && is_focused {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(src_str).style(Style::default().fg(src_color)),
                Cell::from(ts_str).style(Style::default().fg(Color::DarkGray)),
                Cell::from(event.short_id.clone()).style(Style::default().fg(Color::Blue)),
                Cell::from(tool_display).style(Style::default().fg(dot_color).bg(
                    if event.is_error { Color::Rgb(60, 0, 0) } else { Color::Reset },
                )),
                Cell::from(event.file_path.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::White)),
                Cell::from(event.detail.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::Gray)),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(24),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    state.select(Some(display_offset));

    f.render_stateful_widget(table, area, &mut state);
}
