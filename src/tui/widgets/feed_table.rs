use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use chrono::{Local, TimeZone};

use crate::protocol::types::SkillKind;
use crate::tui::app::{App, FocusPane};
use crate::tui::render::StoreSnapshot;

pub fn render_feed_table(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let is_focused = app.focus == FocusPane::MainPanel;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Live Feed ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    if snap.feed.is_empty() {
        let inner = Paragraph::new(Line::from(Span::styled(
            "  Waiting for events...",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(inner, area);
        return;
    }

    let header = Row::new(vec![
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

    // Feed is already in chronological order (oldest first), display as-is
    let rows: Vec<Row> = snap
        .feed
        .iter()
        .enumerate()
        .map(|(i, event)| {
            let ts_str = Local
                .timestamp_opt(event.ts, 0)
                .single()
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".into());

            let tool_color = match event.skill {
                Some(SkillKind::Read) | Some(SkillKind::Search) => Color::Cyan,
                Some(SkillKind::Edit) | Some(SkillKind::Write) => Color::Yellow,
                Some(SkillKind::Bash) => Color::Red,
                Some(SkillKind::Task) => Color::Magenta,
                _ => Color::White,
            };

            let row_style = if event.is_error {
                Style::default().fg(Color::Red)
            } else if i == app.feed_scroll_offset && is_focused {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(ts_str).style(Style::default().fg(Color::DarkGray)),
                Cell::from(event.short_id.clone()).style(Style::default().fg(Color::Blue)),
                Cell::from(event.tool_name.clone().unwrap_or_default())
                    .style(Style::default().fg(tool_color)),
                Cell::from(event.file_path.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::White)),
                Cell::from(event.detail.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::Gray)),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(24),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    state.select(Some(app.feed_scroll_offset));

    f.render_stateful_widget(table, area, &mut state);
}
