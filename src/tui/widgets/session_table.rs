use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use chrono::{Local, TimeZone};

use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;

pub fn render_session_table(f: &mut Frame, area: ratatui::layout::Rect, app: &App, snap: &StoreSnapshot) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Sessions ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    if snap.sessions.is_empty() {
        let inner = Paragraph::new(Line::from(Span::styled(
            "  No completed sessions yet.",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(inner, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("ID").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("NAME").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("ROLE").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("STARTED").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("DURATION").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("EVENTS").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("TOKENS").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("COST").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
        Cell::from("REASON").style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    let rows: Vec<Row> = snap
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let started = Local
                .timestamp_opt(session.started_at, 0)
                .single()
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".into());

            let duration_secs = session.duration_ms / 1000;
            let duration = if duration_secs >= 3600 {
                format!("{}h{}m", duration_secs / 3600, (duration_secs % 3600) / 60)
            } else {
                format!("{}m{}s", duration_secs / 60, duration_secs % 60)
            };

            let row_style = if i == app.session_scroll_offset {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(session.session_id.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(session.display_name.clone()).style(Style::default().fg(Color::Blue)),
                Cell::from(format!("{}", session.role)).style(Style::default().fg(Color::White)),
                Cell::from(started).style(Style::default().fg(Color::DarkGray)),
                Cell::from(duration).style(Style::default().fg(Color::White)),
                Cell::from(format!("{}", session.event_count)).style(Style::default().fg(Color::Cyan)),
                Cell::from(AppStore::format_tokens(session.total_tokens))
                    .style(Style::default().fg(Color::White)),
                Cell::from(
                    session
                        .cost_usd
                        .map(|c| format!("${:.2}", c))
                        .unwrap_or_else(|| "-".into()),
                )
                .style(Style::default().fg(Color::Green)),
                Cell::from(format!("{}", session.close_reason))
                    .style(Style::default().fg(Color::Yellow)),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(7),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    state.select(Some(app.session_scroll_offset));

    f.render_stateful_widget(table, area, &mut state);
}
