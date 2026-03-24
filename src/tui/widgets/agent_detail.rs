use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use chrono::{Local, TimeZone};

use crate::protocol::types::{AgentState, SkillKind};
use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;

pub fn render_agent_detail(f: &mut Frame, app: &App, snap: &StoreSnapshot) {
    let agent = match snap.agents.get(app.sidebar_selected) {
        Some(a) => a,
        None => return,
    };

    // Center popup: 60% width, 70% height
    let area = f.area();
    let popup_width = (area.width as f32 * 0.6) as u16;
    let popup_height = (area.height as f32 * 0.7) as u16;
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" {} ", agent.display_name))
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut lines = Vec::new();

    let (state_icon, state_color) = match agent.state {
        AgentState::Active => ("*", Color::Green),
        AgentState::Waiting => ("o", Color::Yellow),
        AgentState::Completed => ("v", Color::DarkGray),
    };

    lines.push(Line::from(vec![
        Span::styled(" State: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} {}", state_icon, agent.state),
            Style::default().fg(state_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled(" Role: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", agent.role), Style::default().fg(Color::White)),
    ]));

    lines.push(Line::raw(""));

    if let Some(ref model) = agent.model_name {
        lines.push(Line::from(vec![
            Span::styled(" Model: ", Style::default().fg(Color::DarkGray)),
            Span::styled(model.clone(), Style::default().fg(Color::Cyan)),
        ]));
    }

    if let Some(ref branch) = agent.branch_name {
        lines.push(Line::from(vec![
            Span::styled(" Branch: ", Style::default().fg(Color::DarkGray)),
            Span::styled(branch.clone(), Style::default().fg(Color::Green)),
        ]));
    }

    lines.push(Line::raw(""));

    lines.push(Line::from(vec![
        Span::styled(" Tokens: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            AppStore::format_tokens(agent.total_tokens),
            Style::default().fg(Color::White),
        ),
        Span::raw("    "),
        Span::styled(" Cost: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            agent
                .cost_usd
                .map(|c| format!("${:.2}", c))
                .unwrap_or_else(|| "-".into()),
            Style::default().fg(Color::Green),
        ),
        Span::raw("    "),
        Span::styled(" Tools: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", agent.tool_run_count),
            Style::default().fg(Color::White),
        ),
    ]));

    lines.push(Line::raw(""));

    if let Some(ctx) = agent.context_percent {
        let filled = ((ctx / 100.0) * 20.0) as usize;
        let empty = 20_usize.saturating_sub(filled);
        let gauge_color = if ctx > 80.0 {
            Color::Red
        } else if ctx > 60.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        lines.push(Line::from(vec![
            Span::styled(" Context: ", Style::default().fg(Color::DarkGray)),
            Span::styled("#".repeat(filled), Style::default().fg(gauge_color)),
            Span::styled("-".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {:.0}%", ctx), Style::default().fg(gauge_color)),
        ]));
    }

    lines.push(Line::raw(""));

    lines.push(Line::from(Span::styled(
        " Skill Usage:",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));

    let max_usage = agent.skill_usage.values().max().copied().unwrap_or(1);
    let mut skills: Vec<_> = agent.skill_usage.iter().collect();
    skills.sort_by(|a, b| b.1.cmp(a.1));

    for (skill, count) in &skills {
        let bar_len = ((**count as f64 / max_usage as f64) * 15.0) as usize;
        let skill_color = match skill {
            SkillKind::Read | SkillKind::Search => Color::Cyan,
            SkillKind::Edit | SkillKind::Write => Color::Yellow,
            SkillKind::Bash => Color::Red,
            SkillKind::Task => Color::Magenta,
            _ => Color::White,
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("   {:<8}", format!("{}", skill)),
                Style::default().fg(skill_color),
            ),
            Span::styled("#".repeat(bar_len), Style::default().fg(skill_color)),
            Span::styled(format!(" {}", count), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::raw(""));

    let last_ts = Local
        .timestamp_opt(agent.last_event_ts, 0)
        .single()
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "??:??:??".into());

    lines.push(Line::from(vec![
        Span::styled(" Last Event: ", Style::default().fg(Color::DarkGray)),
        Span::styled(last_ts, Style::default().fg(Color::White)),
    ]));

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        " Press Enter/Esc to close",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}
