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
use crate::tui::theme::theme;

fn bg() -> Color { theme().bg }
fn card_bg() -> Color { theme().card_bg }
fn dim() -> Color { theme().text_dim }
fn border() -> Color { theme().border }

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
        Paragraph::new("").style(Style::default().bg(bg())),
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
        let mut tab_spans = vec![Span::styled(" ", Style::default().bg(bg()))];
        for (i, proj) in projects.iter().enumerate() {
            let name = short_project_name(proj);
            let is_selected = app.current_project.as_deref() == Some(proj);
            let style = if is_selected {
                Style::default().fg(theme().lead_badge_fg).bg(theme().lead_badge_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(dim()).bg(bg())
            };
            tab_spans.push(Span::styled(format!(" {} ", name), style));
            if i < projects.len() - 1 {
                tab_spans.push(Span::styled(" \u{2502} ", Style::default().fg(theme().hp_empty).bg(bg())));
            }
        }
        f.render_widget(Paragraph::new(Line::from(tab_spans)).style(Style::default().bg(bg())), chunks[0]);
    }

    // Main: left (leader + party) | right (feed)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Fill(1)])
        .split(chunks[1]);

    render_left_panel(f, main[0], app, snap);
    render_right_panel(f, main[1], app, snap);
}

fn render_empty_party(f: &mut Frame, area: Rect, _port: u16) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border()))
        .style(Style::default().bg(card_bg()));

    let paragraph = Paragraph::new("").block(block);
    f.render_widget(paragraph, area);
}

fn render_left_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border()))
        .style(Style::default().bg(card_bg()));
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
                .fg(theme().lead_name)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " LEAD ",
            Style::default()
                .fg(theme().lead_badge_fg)
                .bg(theme().lead_badge_bg)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(
        Paragraph::new(name_line).style(Style::default().bg(card_bg())),
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
    let chicken_lines = sprite_to_lines(&chicken_pixels, card_bg());
    let cw = 28u16;
    let cx = li.x + (li.width.saturating_sub(cw)) / 2;
    for (j, line) in chicken_lines.iter().enumerate() {
        let sy = y + j as u16;
        if sy < li.y + li.height {
            f.render_widget(
                Paragraph::new(line.clone()).style(Style::default().bg(card_bg())),
                Rect::new(cx, sy, cw, 1),
            );
        }
    }
    y += chicken_lines.len() as u16;

    // Leader stats: HP bar (100% = full health, decreases as context fills up)
    if y < li.y + li.height {
        let used_pct = leader.context_percent.unwrap_or_else(|| {
            // Estimate from tokens: assume 1M context window
            if leader.total_tokens > 0 {
                ((leader.total_tokens as f64 / 1_000_000.0) * 100.0).min(100.0)
            } else {
                0.0
            }
        });
        let ctx_pct = (100.0 - used_pct).max(0.0); // HP = remaining
        let filled = ((ctx_pct / 100.0) * 10.0).round() as usize;
        let empty = 10usize.saturating_sub(filled);
        let t = theme();
        let hp_color = if ctx_pct < 20.0 {
            t.hp_danger
        } else if ctx_pct < 50.0 {
            t.hp_warn
        } else {
            t.hp_good
        };

        let tokens_str = AppStore::format_tokens(leader.total_tokens);
        let cost_str = leader
            .cost_usd
            .map(|c| format!("${:.2}", c))
            .unwrap_or_else(|| "-".into());

        let hp_line = Line::from(vec![
            Span::styled(" HP ", Style::default().fg(dim())),
            Span::styled(
                "\u{2588}".repeat(filled),
                Style::default().fg(hp_color),
            ),
            Span::styled(
                "\u{2591}".repeat(empty),
                Style::default().fg(theme().hp_empty),
            ),
            Span::styled(
                format!(" {:.0}%", ctx_pct),
                Style::default().fg(hp_color),
            ),
            Span::styled(
                format!("  {} {}", tokens_str, cost_str),
                Style::default().fg(dim()),
            ),
        ]);
        f.render_widget(
            Paragraph::new(hp_line).style(Style::default().bg(card_bg())),
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
            Style::default().fg(dim()),
        ));
        f.render_widget(
            Paragraph::new(sep).style(Style::default().bg(card_bg())),
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

        let spr_lines = sprite_to_lines(&sprite, card_bg());
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
                    Paragraph::new(line.clone()).style(Style::default().bg(card_bg())),
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
                        Style::default().fg(dim()),
                    )))
                    .style(Style::default().bg(card_bg())),
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
                _ => dim(),
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^width$}", label, width = col_w as usize),
                    Style::default().fg(color),
                )))
                .style(Style::default().bg(card_bg())),
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
                dim()
            } else if is_waiting {
                theme().accent_yellow
            } else {
                theme().accent_green
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^width$}", display_text, width = col_w as usize),
                    Style::default().fg(sc),
                )))
                .style(Style::default().bg(card_bg())),
                Rect::new(mx, state_y, col_w, 1),
            );
        }
    }
}

fn render_right_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let (_proj_agents, proj_feed) = filter_snap_by_project(snap, &app.current_project);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border()))
        .title(" conversation ")
        .title_style(
            Style::default()
                .fg(theme().text_bright)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(card_bg()));

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
    let last_ts = proj_feed.last().map(|e| e.ts).unwrap_or(0);

    for event in proj_feed.iter() {
        // Apply filter
        if !filter.is_empty() {
            let matches = event.display_name.to_lowercase().contains(&f_lower)
                || event.tool_name.as_deref().unwrap_or("").to_lowercase().contains(&f_lower)
                || event.file_path.as_deref().unwrap_or("").to_lowercase().contains(&f_lower)
                || event.detail.as_deref().unwrap_or("").to_lowercase().contains(&f_lower);
            if !matches { continue; }
        }

        let is_latest = event.ts == last_ts;
        let elapsed = format_elapsed(event.ts, snap, is_latest);
        let is_sub = is_sub_agent(event, snap);

        match event.event_type {
            // User prompt = top-level comment with text
            RuntimeEventType::TurnActive => {
                let prompt_text = event.detail.as_deref().unwrap_or("user prompt");
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled(format!("{}", elapsed), Style::default().fg(dim())),
                    Span::styled(" \u{25b6} ", Style::default().fg(theme().user_prompt)),
                    Span::styled(prompt_text.to_string(), Style::default().fg(theme().user_prompt).add_modifier(Modifier::BOLD)),
                ]));
            }

            // Assistant text = reply (indented if sub-agent)
            RuntimeEventType::AssistantText => {
                let text = event.detail.as_deref().unwrap_or("");
                if text.is_empty() { continue; }

                let (tree, icon, color) = if is_sub {
                    (" \u{2502} \u{251c}\u{2500}", "\u{1f423} ", theme().sub_agent_text)
                } else {
                    (" \u{251c}\u{2500}", "\u{1f414} ", theme().assistant_text)
                };

                let mut spans = vec![
                    Span::styled(format!("{}", elapsed), Style::default().fg(dim())),
                    Span::styled(tree, Style::default().fg(border())),
                    Span::styled(icon, Style::default().fg(color)),
                ];
                if let Some(ai) = event.ai_tool.as_deref() {
                    let badge = crate::tui::theme::Theme::ai_tool_badge(ai);
                    let badge_color = theme().ai_tool_color(ai);
                    spans.push(Span::styled(format!("{} ", badge), Style::default().fg(badge_color)));
                }
                spans.push(Span::styled(text.to_string(), Style::default().fg(color)));
                lines.push(Line::from(spans));
            }

            // Tool use = indented action with tree line
            RuntimeEventType::ToolStart => {
                if event.tool_name.is_none() { continue; }
                let tool_text = format_tool(event);
                let tree = if is_sub {
                    " \u{2502} \u{2502} \u{251c}\u{2500}"
                } else {
                    " \u{2502} \u{251c}\u{2500}"
                };

                let t = theme();
                let tool_color = match event.tool_name.as_deref().unwrap_or("") {
                    "Read" | "Grep" | "Glob" => t.tool_read,
                    "Edit" | "Write" => t.tool_edit,
                    "Bash" => t.tool_bash,
                    "Task" | "TaskCreate" | "TaskUpdate" => t.tool_task,
                    _ => t.text,
                };

                let mut spans = vec![
                    Span::styled(format!("{}", elapsed), Style::default().fg(dim())),
                    Span::styled(tree, Style::default().fg(border())),
                    Span::styled("\u{2b24} ", Style::default().fg(theme().accent_yellow)), // orange dot = in progress
                    Span::styled(tool_text, Style::default().fg(tool_color)),
                ];
                if let Some(ai) = event.ai_tool.as_deref() {
                    let badge = crate::tui::theme::Theme::ai_tool_badge(ai);
                    let color = t.ai_tool_color(ai);
                    spans.push(Span::styled(format!(" {}", badge), Style::default().fg(color)));
                }

                lines.push(Line::from(spans));
            }

            // ToolDone with tool_name
            RuntimeEventType::ToolDone if event.tool_name.is_some() => {
                let tool_text = format_tool(event);
                let tree = if is_sub {
                    " \u{2502} \u{2502} \u{2514}\u{2500}"
                } else {
                    " \u{2502} \u{2514}\u{2500}"
                };
                let (dot, dot_color) = if event.is_error {
                    ("\u{2b24} ", theme().accent_red) // red dot = error
                } else {
                    ("\u{2b24} ", theme().accent_green) // green dot = success
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}", elapsed), Style::default().fg(dim())),
                    Span::styled(tree, Style::default().fg(border())),
                    Span::styled(dot, Style::default().fg(dot_color)),
                    Span::styled(tool_text, Style::default().fg(dim())),
                ]));
            }

            // Waiting/permission
            RuntimeEventType::PermissionWait => {
                lines.push(Line::from(vec![
                    Span::styled(format!("{}", elapsed), Style::default().fg(dim())),
                    Span::styled(" \u{23f3} ", Style::default().fg(theme().lead_name)),
                    Span::styled("waiting for permission...", Style::default().fg(theme().lead_name)),
                ]));
            }

            _ => {}
        }
    }

    // Apply scroll offset (j/k keys)
    let max_lines = inner.height as usize;
    let total = lines.len();
    let default_start = total.saturating_sub(max_lines);

    let start = if app.feed_auto_scroll {
        default_start
    } else {
        // scroll_offset 0 = latest (bottom), higher = further back
        let offset = app.feed_scroll_offset.min(total.saturating_sub(1));
        total.saturating_sub(max_lines + offset)
    };
    let end = (start + max_lines).min(total);
    let visible: Vec<Line> = lines[start..end].to_vec();

    f.render_widget(
        Paragraph::new(visible).wrap(Wrap { trim: false }).style(Style::default().bg(card_bg())),
        inner,
    );
}

// --- Helpers ---

fn format_elapsed(ts: i64, _snap: &StoreSnapshot, is_latest: bool) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = (now - ts).max(0);
    let text = if is_latest && diff < 120 {
        if diff < 60 {
            format!("{}초째", diff)
        } else {
            format!("{}분째", diff / 60)
        }
    } else if diff < 60 {
        "방금".to_string()
    } else if diff < 3600 {
        format!("{}분 전", diff / 60)
    } else {
        format!("{}시간 전", diff / 3600)
    };
    format!("{:>7}", text)
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
