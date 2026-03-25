use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::protocol::types::{AgentRole, AgentState, FeedEvent, RuntimeEventType};
use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::render::StoreSnapshot;
use crate::tui::sprites::chicken;
use crate::tui::sprites::renderer::sprite_to_lines;

const CARD_BG: Color = Color::Rgb(22, 22, 34);
const DIM: Color = Color::Rgb(70, 70, 90);
const BUBBLE_BG: Color = Color::Rgb(32, 32, 48);
const BUBBLE_BORDER: Color = Color::Rgb(55, 55, 75);
const BG: Color = Color::Rgb(18, 18, 28);

/// Get unique project names from agents
pub fn get_projects(snap: &StoreSnapshot) -> Vec<String> {
    let mut projects: Vec<String> = snap
        .agents
        .iter()
        .filter_map(|a| a.cwd.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    projects.sort();
    projects
}

/// Shorten cwd to project name (last path component)
fn short_project_name(cwd: &str) -> &str {
    cwd.rsplit('/').next().unwrap_or(cwd)
}

/// Filter snapshot to only include agents/events for the current project
fn filter_snap_by_project<'a>(
    snap: &'a StoreSnapshot,
    project: &Option<String>,
) -> (Vec<&'a crate::protocol::types::Agent>, Vec<&'a FeedEvent>) {
    match project {
        Some(cwd) => {
            let agents: Vec<_> = snap.agents.iter().filter(|a| a.cwd.as_deref() == Some(cwd)).collect();
            let agent_ids: std::collections::HashSet<_> = agents.iter().map(|a| &a.agent_id).collect();
            let feed: Vec<_> = snap.feed.iter().filter(|e| agent_ids.contains(&e.agent_id)).collect();
            (agents, feed)
        }
        None => {
            (snap.agents.iter().collect(), snap.feed.iter().collect())
        }
    }
}

pub fn render_stage(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    // Fill background
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(BG)),
        area,
    );

    if snap.agents.is_empty() && snap.feed.is_empty() {
        render_empty_party(f, area, app.port);
        return;
    }

    // Project tabs + main content
    let projects = get_projects(snap);
    let has_projects = projects.len() > 1;

    let chunks = if has_projects {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(0), Constraint::Fill(1)])
            .split(area)
    };

    // Render project tabs
    if has_projects {
        let mut tab_spans = vec![Span::styled(" ", Style::default().bg(BG))];
        for (i, proj) in projects.iter().enumerate() {
            let name = short_project_name(proj);
            let is_selected = app.current_project.as_deref() == Some(proj);
            let style = if is_selected {
                Style::default().fg(Color::Rgb(255, 220, 80)).bg(Color::Rgb(50, 45, 30)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(DIM).bg(BG)
            };
            tab_spans.push(Span::styled(format!(" {} ", name), style));
            if i < projects.len() - 1 {
                tab_spans.push(Span::styled(" \u{2502} ", Style::default().fg(Color::Rgb(40, 40, 55)).bg(BG)));
            }
        }
        tab_spans.push(Span::styled("  [/] switch", Style::default().fg(Color::Rgb(40, 40, 55)).bg(BG)));
        f.render_widget(Paragraph::new(Line::from(tab_spans)).style(Style::default().bg(BG)), chunks[0]);
    }

    // Main: left (leader + party) | right (feed)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Fill(1)])
        .split(chunks[1]);

    render_left_panel(f, main[0], app, snap);
    render_right_panel(f, main[1], app, snap);
}

fn render_empty_party(f: &mut Frame, area: Rect, port: u16) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 55)))
        .style(Style::default().bg(CARD_BG));

    let lines = vec![
        Line::raw(""),
        Line::raw(""),
        Line::from(Span::styled(
            format!("  Waiting for events on :{} ...", port),
            Style::default().fg(DIM),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "  The party is empty. Send hook events to hatch some chickens!",
            Style::default().fg(DIM),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            format!(
                "  curl -s -X POST http://localhost:{}/hook \\",
                port
            ),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "    -H 'Content-Type: application/json' \\",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "    -d '{\"hook_event_name\":\"PostToolUse\",\"session_id\":\"test\",\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"src/main.rs\"}}'",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_left_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 55)))
        .style(Style::default().bg(CARD_BG));
    let li = left_block.inner(area);
    f.render_widget(left_block, area);

    // Filter by project
    let (proj_agents, _proj_feed) = filter_snap_by_project(snap, &app.current_project);

    // Find the leader (AgentRole::Main or first agent)
    let leader = proj_agents
        .iter()
        .find(|a| a.role == AgentRole::Main)
        .or_else(|| proj_agents.first());

    let leader = match leader {
        Some(l) => l,
        None => return,
    };

    let mut y = li.y;

    // Leader name + badge
    let name_line = Line::from(vec![
        Span::styled(
            format!(" {} ", leader.display_name),
            Style::default()
                .fg(Color::Rgb(255, 200, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " LEAD ",
            Style::default()
                .fg(Color::Rgb(255, 220, 80))
                .bg(Color::Rgb(80, 60, 20))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(
        Paragraph::new(name_line).style(Style::default().bg(CARD_BG)),
        Rect::new(li.x, y, li.width, 1),
    );
    y += 1;

    // (leader bubble removed — conversation panel shows it)

    // Chicken sprite (leader is always a chicken)
    let tick = app.tick;
    let is_active = leader.state == AgentState::Active;
    let chicken_pixels = if is_active {
        chicken::chicken_peck(tick / 4)
    } else {
        chicken::chicken_idle(tick / 4)
    };
    let chicken_lines = sprite_to_lines(&chicken_pixels, CARD_BG);
    let cw = 28u16;
    let cx = li.x + (li.width.saturating_sub(cw)) / 2;
    for (j, line) in chicken_lines.iter().enumerate() {
        let sy = y + j as u16;
        if sy < li.y + li.height {
            f.render_widget(
                Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)),
                Rect::new(cx, sy, cw, 1),
            );
        }
    }
    y += chicken_lines.len() as u16;

    // Leader stats: HP bar (use context% if available, else estimate from tokens)
    if y < li.y + li.height {
        let ctx_pct = leader.context_percent.unwrap_or_else(|| {
            // Rough estimate: assume 200k token context, show usage as %
            if leader.total_tokens > 0 {
                ((leader.total_tokens as f64 / 200_000.0) * 100.0).min(100.0)
            } else {
                0.0
            }
        });
        let filled = ((ctx_pct / 100.0) * 10.0).round() as usize;
        let empty = 10usize.saturating_sub(filled);
        let hp_color = if ctx_pct > 80.0 {
            Color::Rgb(255, 80, 80)
        } else if ctx_pct > 50.0 {
            Color::Rgb(255, 200, 80)
        } else {
            Color::Rgb(100, 220, 140)
        };

        let tokens_str = AppStore::format_tokens(leader.total_tokens);
        let cost_str = leader
            .cost_usd
            .map(|c| format!("${:.2}", c))
            .unwrap_or_else(|| "-".into());

        let hp_line = Line::from(vec![
            Span::styled(" HP ", Style::default().fg(DIM)),
            Span::styled(
                "\u{2588}".repeat(filled),
                Style::default().fg(hp_color),
            ),
            Span::styled(
                "\u{2591}".repeat(empty),
                Style::default().fg(Color::Rgb(40, 40, 55)),
            ),
            Span::styled(
                format!(" {:.0}%", ctx_pct),
                Style::default().fg(hp_color),
            ),
            Span::styled(
                format!("  {} {}", tokens_str, cost_str),
                Style::default().fg(DIM),
            ),
        ]);
        f.render_widget(
            Paragraph::new(hp_line).style(Style::default().bg(CARD_BG)),
            Rect::new(li.x, y, li.width, 1),
        );
        y += 1;
    }

    // Party separator
    let party_members: Vec<_> = proj_agents
        .iter()
        .filter(|a| a.agent_id != leader.agent_id)
        .copied()
        .collect();

    y += 1;
    if y < li.y + li.height {
        let sep = Line::from(Span::styled(
            format!(
                " \u{2500}\u{2500} party ({}) \u{2500}{}",
                party_members.len(),
                "\u{2500}".repeat(22)
            ),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(
            Paragraph::new(sep).style(Style::default().bg(CARD_BG)),
            Rect::new(li.x, y, li.width, 1),
        );
        y += 1;
    }

    // Party members in 2-column grid
    let cols = 2u16;
    let col_w = li.width / cols;
    let row_h = 8u16;

    for (i, member) in party_members.iter().enumerate() {
        let col = (i as u16) % cols;
        let row = (i as u16) / cols;
        let mx = li.x + col * col_w;
        let my = y + row * row_h;

        if my + row_h > li.y + li.height {
            break;
        }

        let is_done = member.state == AgentState::Completed;
        let is_waiting = member.state == AgentState::Waiting;
        let stage = chicken::agent_growth_stage(member.usage_count, is_done);

        // Sprite
        let sprite = match stage {
            "egg" => chicken::egg_sprite(),
            "hatching" => chicken::egg_cracking(tick / 3),
            "peeking" => chicken::egg_hatching_chick(tick / 3),
            "chick" if is_waiting => chicken::chick_sleeping(tick),
            "chick" => chicken::chick_sprite(tick / 3),
            "done" => chicken::chick_done(),
            _ => chicken::egg_sprite(),
        };

        let spr_lines = sprite_to_lines(&sprite, CARD_BG);
        let spr_w = if stage == "egg" || stage == "hatching" {
            12u16
        } else {
            16u16
        };
        let spr_x = mx + (col_w.saturating_sub(spr_w)) / 2;

        for (j, line) in spr_lines.iter().enumerate() {
            let sy = my + j as u16;
            if sy < li.y + li.height {
                f.render_widget(
                    Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)),
                    Rect::new(spr_x, sy, spr_w, 1),
                );
            }
        }

        // Zzz for waiting
        if is_waiting && stage == "chick" {
            let zzz_frame = (tick / 10) % 4;
            let zzz = ["z", " zz", "  zzz", " zz"][zzz_frame];
            if spr_x + spr_w + 5 <= li.x + li.width {
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        zzz,
                        Style::default().fg(Color::Rgb(120, 120, 170)),
                    )))
                    .style(Style::default().bg(CARD_BG)),
                    Rect::new(spr_x + spr_w, my, 5, 1),
                );
            }
        }

        // Name
        let name_y = my + spr_lines.len() as u16;
        if name_y < li.y + li.height {
            let stage_icon = match stage {
                "egg" => "\u{1f95a}",
                "hatching" => "\u{1fab6}",
                "peeking" => "\u{1f425}",
                "chick" => "\u{1f423}",
                "done" => "\u{2b50}",
                _ => "",
            };
            let label = format!("{} {}", stage_icon, member.display_name);
            let color = match stage {
                "egg" => Color::Rgb(200, 195, 180),
                "hatching" | "peeking" => Color::Rgb(230, 200, 100),
                "chick" | "done" => Color::Rgb(255, 220, 80),
                _ => DIM,
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^width$}", label, width = col_w as usize),
                    Style::default().fg(color),
                )))
                .style(Style::default().bg(CARD_BG)),
                Rect::new(mx, name_y, col_w, 1),
            );
        }

        // State / tool + mini speech bubble
        let state_y = name_y + 1;
        if state_y < li.y + li.height {
            // Find latest activity for this member
            let latest_text = snap
                .feed
                .iter()
                .rev()
                .find(|e| e.agent_id == member.agent_id && e.detail.is_some())
                .and_then(|e| e.detail.as_deref());

            let display_text = if let Some(tool) = &member.current_skill {
                format!("{}", tool)
            } else if let Some(text) = latest_text {
                let chars: Vec<char> = text.chars().collect();
                if chars.len() > (col_w as usize).saturating_sub(2) {
                    chars[..(col_w as usize).saturating_sub(5)].iter().collect::<String>() + "..."
                } else {
                    text.to_string()
                }
            } else {
                format!("{}", member.state)
            };

            let sc = if is_done {
                DIM
            } else if is_waiting {
                Color::Rgb(200, 200, 80)
            } else {
                Color::Rgb(100, 220, 140)
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^width$}", display_text, width = col_w as usize),
                    Style::default().fg(sc),
                )))
                .style(Style::default().bg(CARD_BG)),
                Rect::new(mx, state_y, col_w, 1),
            );
        }
    }
}

fn render_right_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let (_proj_agents, proj_feed) = filter_snap_by_project(snap, &app.current_project);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" conversation ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(180, 180, 220))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(CARD_BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 10 || inner.height < 3 {
        return;
    }

    // Build conversation timeline: group events by user turns
    // user prompt → assistant text → tool calls → assistant text → ...
    let filter = &app.filter_text;
    let f_lower = filter.to_lowercase();

    let mut lines: Vec<Line> = Vec::new();

    for event in proj_feed.iter() {
        // Apply filter
        if !filter.is_empty() {
            let matches = event.display_name.to_lowercase().contains(&f_lower)
                || event.tool_name.as_deref().unwrap_or("").to_lowercase().contains(&f_lower)
                || event.file_path.as_deref().unwrap_or("").to_lowercase().contains(&f_lower)
                || event.detail.as_deref().unwrap_or("").to_lowercase().contains(&f_lower);
            if !matches { continue; }
        }

        let elapsed = format_elapsed(event.ts, snap);
        let is_sub = is_sub_agent(event, snap);

        match event.event_type {
            // User prompt = top-level comment
            RuntimeEventType::TurnActive => {
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                    Span::styled("\u{25b6} ", Style::default().fg(Color::Rgb(100, 200, 100))),
                    Span::styled("user prompt", Style::default().fg(Color::Rgb(100, 200, 100)).add_modifier(Modifier::BOLD)),
                ]));
            }

            // Assistant text = reply (indented if sub-agent)
            RuntimeEventType::AssistantText => {
                let text = event.detail.as_deref().unwrap_or("");
                if text.is_empty() { continue; }

                let (indent, icon, color) = if is_sub {
                    ("  ", "\u{2514}\u{1f423} ", Color::Rgb(160, 180, 200))
                } else {
                    (" ", "\u{1f414} ", Color::Rgb(180, 170, 220))
                };

                lines.push(Line::from(vec![
                    Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                    Span::styled(indent, Style::default()),
                    Span::styled(icon, Style::default().fg(color)),
                    Span::styled(text.to_string(), Style::default().fg(color)),
                ]));
            }

            // Tool use = indented action
            RuntimeEventType::ToolStart => {
                if event.tool_name.is_none() { continue; }
                let tool_text = format_tool(event);
                let (indent, prefix) = if is_sub {
                    ("    ", "\u{2514} ")
                } else {
                    ("  ", "")
                };

                let tool_color = match event.tool_name.as_deref().unwrap_or("") {
                    "Read" | "Grep" | "Glob" => Color::Cyan,
                    "Edit" | "Write" => Color::Yellow,
                    "Bash" => Color::Red,
                    "Task" | "TaskCreate" | "TaskUpdate" => Color::Magenta,
                    _ => Color::White,
                };

                lines.push(Line::from(vec![
                    Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                    Span::styled(indent, Style::default()),
                    Span::styled(prefix, Style::default().fg(DIM)),
                    Span::styled("\u{2699} ", Style::default().fg(tool_color)),
                    Span::styled(tool_text, Style::default().fg(tool_color)),
                ]));
            }

            // ToolDone with tool_name
            RuntimeEventType::ToolDone if event.tool_name.is_some() => {
                let tool_text = format_tool(event);
                let indent = if is_sub { "    " } else { "  " };
                lines.push(Line::from(vec![
                    Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                    Span::styled(indent, Style::default()),
                    Span::styled("\u{2713} ", Style::default().fg(Color::Rgb(80, 180, 80))),
                    Span::styled(tool_text, Style::default().fg(DIM)),
                ]));
            }

            // Waiting/permission
            RuntimeEventType::PermissionWait => {
                lines.push(Line::from(vec![
                    Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                    Span::styled(" \u{23f3} ", Style::default().fg(Color::Rgb(255, 200, 80))),
                    Span::styled("waiting for permission...", Style::default().fg(Color::Rgb(255, 200, 80))),
                ]));
            }

            _ => {}
        }
    }

    // Show last N lines that fit
    let max_lines = inner.height as usize;
    let start = lines.len().saturating_sub(max_lines);
    let visible: Vec<Line> = lines[start..].to_vec();

    f.render_widget(
        Paragraph::new(visible).wrap(Wrap { trim: false }).style(Style::default().bg(CARD_BG)),
        inner,
    );
}

// --- Helpers ---

fn format_elapsed(ts: i64, _snap: &StoreSnapshot) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = (now - ts).max(0);
    if diff < 60 {
        format!("{}s", diff)
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else {
        format!("{}h", diff / 3600)
    }
}

fn format_who(e: &FeedEvent, snap: &StoreSnapshot) -> String {
    // Check if this is the leader (Main role)
    let is_leader = snap
        .agents
        .iter()
        .find(|a| a.agent_id == e.agent_id)
        .map(|a| a.role == AgentRole::Main)
        .unwrap_or(false);

    if is_leader {
        "lead".to_string()
    } else {
        format!("\u{2514}{}", &e.short_id)
    }
}

fn who_color(e: &FeedEvent, snap: &StoreSnapshot) -> Color {
    let role = snap
        .agents
        .iter()
        .find(|a| a.agent_id == e.agent_id)
        .map(|a| a.role);

    match role {
        Some(AgentRole::Main) => Color::Rgb(255, 200, 80),
        Some(AgentRole::Team) => Color::Rgb(80, 200, 200),
        _ => Color::Rgb(255, 220, 80),
    }
}

fn is_sub_agent(e: &FeedEvent, snap: &StoreSnapshot) -> bool {
    snap.agents
        .iter()
        .find(|a| a.agent_id == e.agent_id)
        .map(|a| a.role != AgentRole::Main)
        .unwrap_or(true)
}

fn format_tool(e: &FeedEvent) -> String {
    let tool = e.tool_name.as_deref().unwrap_or("?");
    let path = e.file_path.as_deref().unwrap_or("");
    if path.is_empty() {
        tool.to_string()
    } else {
        // Show last 2-3 path components for context
        let parts: Vec<&str> = path.rsplit('/').take(3).collect();
        let short_path: String = parts.into_iter().rev().collect::<Vec<_>>().join("/");
        format!("{} {}", tool, short_path)
    }
}

/// Compute party composition summary for the status bar.
pub fn party_summary(snap: &StoreSnapshot) -> String {
    let leader = snap
        .agents
        .iter()
        .find(|a| a.role == AgentRole::Main)
        .or_else(|| snap.agents.first());

    let mut chickens = 0u16;
    let mut chicks = 0u16;
    let mut eggs = 0u16;
    let mut stars = 0u16;

    // Leader counts as a chicken
    if leader.is_some() {
        chickens += 1;
    }

    for agent in &snap.agents {
        if leader.map(|l| l.agent_id == agent.agent_id).unwrap_or(false) {
            continue; // skip leader, already counted
        }
        let is_done = agent.state == AgentState::Completed;
        let stage = chicken::agent_growth_stage(agent.usage_count, is_done);
        match stage {
            "chick" => chicks += 1,
            "done" => stars += 1,
            "egg" | "hatching" | "peeking" => eggs += 1,
            _ => eggs += 1,
        }
    }

    let mut parts = Vec::new();
    if chickens > 0 {
        parts.push(format!("\u{1f414}{}", chickens));
    }
    if chicks > 0 {
        parts.push(format!("\u{1f423}{}", chicks));
    }
    if eggs > 0 {
        parts.push(format!("\u{1f95a}{}", eggs));
    }
    if stars > 0 {
        parts.push(format!("\u{2b50}{}", stars));
    }

    parts.join(" ")
}
