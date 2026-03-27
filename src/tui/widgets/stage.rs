use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

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

pub fn render_stage(f: &mut Frame, area: Rect, app: &mut App, snap: &StoreSnapshot) {
    // Fill background
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(bg())),
        area,
    );

    if snap.agents.is_empty() && snap.feed.is_empty() {
        render_empty_party(f, area, app.port);
        return;
    }

    // Resolve pending focus select (Enter key on sidebar)
    if app.pending_focus_select {
        app.pending_focus_select = false;
        let (proj_agents, _) = filter_snap_by_project(snap, &app.current_project);
        if let Some(agent) = proj_agents.get(app.sidebar_selected) {
            if agent.role == AgentRole::Main && app.focused_agent.is_some() {
                // Pressing Enter on leader while focused → unfocus
                app.focused_agent = None;
            } else if agent.role != AgentRole::Main {
                app.focused_agent = Some(agent.agent_id.clone());
                app.feed_auto_scroll = true;
            }
        }
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
    // Fill background
    f.render_widget(
        Paragraph::new("").style(Style::default().bg(card_bg())),
        area,
    );

    let tick = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as usize;

    // Animated chicken sprite (alternates idle/peck)
    let chicken_pixels = if (tick / 600).is_multiple_of(2) {
        chicken::chicken_idle(tick / 150)
    } else {
        chicken::chicken_peck(tick / 150)
    };
    let chicken_lines = sprite_to_lines(&chicken_pixels, card_bg());

    // Center everything
    let content_height = chicken_lines.len() as u16 + 10; // sprite + text
    let start_y = area.y + area.height.saturating_sub(content_height) / 2;
    let center_x = area.x + area.width / 2;

    // Draw chicken centered
    let sprite_w = 28u16; // 14px * 2
    let sprite_x = center_x.saturating_sub(sprite_w / 2);
    for (j, line) in chicken_lines.iter().enumerate() {
        let y = start_y + j as u16;
        if y < area.y + area.height {
            f.render_widget(
                Paragraph::new(line.clone()).style(Style::default().bg(card_bg())),
                Rect::new(sprite_x, y, sprite_w, 1),
            );
        }
    }

    let text_y = start_y + chicken_lines.len() as u16 + 1;
    let t = theme();

    // Title
    if text_y < area.y + area.height {
        let title = Line::from(vec![
            Span::styled("peep", Style::default().fg(t.brand).add_modifier(Modifier::BOLD)),
            Span::styled(" — AI agent monitor", Style::default().fg(t.text_dim)),
        ]);
        let title_w = 30u16;
        f.render_widget(
            Paragraph::new(title).style(Style::default().bg(card_bg())),
            Rect::new(center_x.saturating_sub(title_w / 2), text_y, title_w, 1),
        );
    }

    // Subtitle
    let sub_y = text_y + 2;
    if sub_y < area.y + area.height {
        let sub = Line::from(Span::styled(
            "Start any AI coding tool to begin",
            Style::default().fg(t.text_dim),
        ));
        let sub_w = 40u16;
        f.render_widget(
            Paragraph::new(sub).style(Style::default().bg(card_bg())),
            Rect::new(center_x.saturating_sub(sub_w / 2), sub_y, sub_w, 1),
        );
    }

    // Supported tools
    let tools_y = sub_y + 2;
    if tools_y < area.y + area.height {
        let tools = Line::from(vec![
            Span::styled("Claude", Style::default().fg(t.ai_claude)),
            Span::styled(" · ", Style::default().fg(t.text_dim)),
            Span::styled("Codex", Style::default().fg(t.ai_codex)),
            Span::styled(" · ", Style::default().fg(t.text_dim)),
            Span::styled("Gemini", Style::default().fg(t.ai_gemini)),
            Span::styled(" · ", Style::default().fg(t.text_dim)),
            Span::styled("OpenCode", Style::default().fg(t.ai_opencode)),
        ]);
        let tools_w = 40u16;
        f.render_widget(
            Paragraph::new(tools).style(Style::default().bg(card_bg())),
            Rect::new(center_x.saturating_sub(tools_w / 2), tools_y, tools_w, 1),
        );
    }

    // Dots animation
    let dots_y = tools_y + 2;
    if dots_y < area.y + area.height {
        let dot_count = (tick / 500) % 4;
        let dots = ".".repeat(dot_count);
        let waiting = format!("waiting{:<3}", dots);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                waiting,
                Style::default().fg(t.text_dim),
            ))).style(Style::default().bg(card_bg())),
            Rect::new(center_x.saturating_sub(6), dots_y, 12, 1),
        );
    }
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
    y += 2; // padding after name

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
    y += chicken_lines.len() as u16 + 1; // padding after chicken

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

    // Skills invoked by leader
    if !leader.skills_invoked.is_empty() && y < li.y + li.height {
        y += 1;
        let mut skills: Vec<_> = leader.skills_invoked.iter().collect();
        skills.sort_by(|a, b| b.1.cmp(a.1)); // most used first
        let skills_text: String = skills.iter()
            .take(3) // top 3
            .map(|(name, count)| {
                // shorten "superpowers:brainstorming" → "brainstorming"
                let short = name.rsplit(':').next().unwrap_or(name);
                format!("{} ×{}", short, count)
            })
            .collect::<Vec<_>>()
            .join("  ");
        let skills_line = Line::from(vec![
            Span::styled(" ⚡ ", Style::default().fg(theme().accent_yellow)),
            Span::styled(skills_text, Style::default().fg(dim())),
        ]);
        f.render_widget(
            Paragraph::new(skills_line).style(Style::default().bg(card_bg())),
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

        // Name with sub-agent number tag and color
        let name_y = my + spr_lines.len() as u16;
        if name_y < li.y + li.height {
            let sub_index = i; // 0-based index among party members
            let stage_icon = match stage {
                "egg" => "\u{1f95a}",
                "hatching" => "\u{1fab6}",
                "peeking" => "\u{1f425}",
                "chick" => "\u{1f423}",
                "done" => "\u{2b50}",
                _ => "",
            };
            let sub_color = theme().sub_agent_color(sub_index);
            let is_focused = app.focused_agent.as_deref() == Some(&member.agent_id);
            // sidebar_selected: 0 = leader, 1+ = party members (1-indexed)
            let is_selected = app.focus == crate::tui::app::FocusPane::Sidebar
                && app.sidebar_selected == i + 1; // +1 because 0 is leader
            let tag = format!("[{}]", stage_icon);
            let label = format!("{} {}", tag, member.display_name);
            let color = if is_focused || is_selected {
                sub_color
            } else {
                match stage {
                    "egg" => Color::Rgb(200, 195, 180),
                    "hatching" | "peeking" => Color::Rgb(230, 200, 100),
                    "chick" | "done" => Color::Rgb(255, 220, 80),
                    _ => dim(),
                }
            };
            let style = if is_focused || is_selected {
                Style::default().fg(color).bg(Color::Rgb(40, 40, 60)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^width$}", label, width = col_w as usize),
                    style,
                )))
                .style(Style::default().bg(card_bg())),
                Rect::new(mx, name_y, col_w, 1),
            );
        }

        // State text — truncate with display width, no "Other"
        let state_y = name_y + 1;
        if state_y < li.y + li.height {
            let latest_text = snap
                .feed
                .iter()
                .rev()
                .find(|e| e.agent_id == member.agent_id && e.detail.is_some())
                .and_then(|e| e.detail.as_deref());

            let max_text_w = (col_w as usize).saturating_sub(4); // padding
            let display_text = if let Some(tool) = &member.current_skill {
                format!("{}", tool)
            } else if let Some(text) = latest_text {
                truncate_to_width(text.split('\n').next().unwrap_or(text), max_text_w)
            } else {
                match member.state {
                    AgentState::Active => "working...".to_string(),
                    AgentState::Waiting => "waiting".to_string(),
                    AgentState::Completed => "done".to_string(),
                }
            };

            // Add "..." if truncated
            let display = if display_width(&display_text) < display_width(latest_text.unwrap_or(""))
                && !display_text.ends_with("...")
            {
                format!("{}...", display_text)
            } else {
                display_text
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
                    format!("{:^width$}", display, width = col_w as usize),
                    Style::default().fg(sc),
                )))
                .style(Style::default().bg(card_bg())),
                Rect::new(mx, state_y, col_w, 1),
            );
        }
    }
}

fn render_right_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let (proj_agents, proj_feed) = filter_snap_by_project(snap, &app.current_project);

    // Build sub-agent index map: agent_id → (index, color)
    let sub_agent_map = build_sub_agent_map(&proj_agents);

    // Focus mode: filter to only focused agent's events
    let focused = &app.focused_agent;
    let is_focused = focused.is_some();
    let feed_iter: Vec<&FeedEvent> = if let Some(ref focused_id) = focused {
        proj_feed.iter().filter(|e| e.agent_id == *focused_id).copied().collect()
    } else {
        proj_feed.to_vec()
    };

    // Title changes in focus mode
    let title = if let Some(ref focused_id) = focused {
        let name = proj_agents.iter()
            .find(|a| &a.agent_id == focused_id)
            .map(|a| a.display_name.as_str())
            .unwrap_or("agent");
        let idx = sub_agent_map.get(focused_id.as_str()).map(|(i, _)| i + 1).unwrap_or(0);
        format!(" \u{1f423}{} {} (Esc to return) ", idx, name)
    } else {
        " conversation ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused { theme().accent_yellow } else { border() }))
        .title(title)
        .title_style(
            Style::default()
                .fg(if is_focused { theme().accent_yellow } else { theme().text_bright })
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(card_bg()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 10 || inner.height < 3 {
        return;
    }

    // ── Vertical timeline layout ──
    // Spine: emoji icons (🐔🐣◇) on event lines, │ between events
    // Layout: [elapsed 7dw] [space] [emoji 2dw] [space] [content...]
    //          0─────────6   7       8─────9     10       11+
    // │ at position 9 (right edge of 2-cell emoji at 8-9), content at 11
    let filter = &app.filter_text;
    let f_lower = filter.to_lowercase();

    let max_w = inner.width as usize;
    let spine_pos = 9usize;
    let content_start = 11usize;
    let spine_sep = format!("{}│", " ".repeat(spine_pos));
    let cont_prefix = format!("{}│{}", " ".repeat(spine_pos), " ".repeat(content_start - spine_pos - 1));

    let mut lines: Vec<Line> = Vec::new();
    let last_ts = feed_iter.last().map(|e| e.ts).unwrap_or(0);

    for event in feed_iter.iter() {
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

        // Determine spine emoji icon (2 cells each)
        let (marker, marker_fg) = if event.event_type == RuntimeEventType::TurnActive {
            ("\u{25c7} ", theme().user_prompt) // ◇ + space = 2 cells for user prompt
        } else if event.event_type == RuntimeEventType::PermissionWait {
            ("\u{23f3}", theme().lead_name) // ⏳ waiting
        } else if is_sub && !is_focused {
            // Use per-agent color from palette
            let color = sub_agent_map.get(event.agent_id.as_str())
                .map(|(_, c)| *c)
                .unwrap_or(theme().sub_agent_text);
            ("\u{1f423}", color) // 🐣 sub-agent (unique color)
        } else {
            ("\u{1f414}", theme().assistant_text) // 🐔 main agent
        };

        // Status badge dot for tool events
        // ToolStart: always yellow (in progress), ToolDone: green/red
        let tool_badge: Option<(&str, Color)> = match event.event_type {
            RuntimeEventType::ToolStart => {
                Some(("\u{25cf}", theme().accent_yellow)) // ● yellow = in progress
            }
            RuntimeEventType::ToolDone => {
                if event.is_error {
                    Some(("\u{25cf}", theme().accent_red)) // ● red = error
                } else {
                    Some(("\u{25cf}", theme().accent_green)) // ● green = success
                }
            }
            _ => None,
        };

        // Spine separator line (skip before first event)
        if !lines.is_empty() {
            lines.push(Line::from(Span::styled(spine_sep.clone(), Style::default().fg(border()))));
        }

        // Elapsed color: "N초째/N분째" = yellow (active), others = dim
        let elapsed_color = if is_latest && {
            let now = chrono::Utc::now().timestamp();
            (now - event.ts).abs() < 120
        } {
            theme().accent_yellow
        } else {
            dim()
        };

        // Build prefix: elapsed + space + marker + space
        let mut prefix = vec![
            Span::styled(elapsed.clone(), Style::default().fg(elapsed_color)),
            Span::styled(" ", Style::default().bg(card_bg())),
            Span::styled(marker, Style::default().fg(marker_fg)),
            Span::styled(" ", Style::default().bg(card_bg())),
        ];

        // Add badge dot right after marker (before content) for tool events
        if let Some((dot, dot_color)) = tool_badge {
            prefix.push(Span::styled(format!("{} ", dot), Style::default().fg(dot_color)));
        }

        // Sub-agent tag no longer needed in content — spine emoji distinguishes agents

        match event.event_type {
            // User prompt
            RuntimeEventType::TurnActive => {
                let text = event.detail.as_deref().unwrap_or("user prompt");
                let wrapped = wrap_with_tree(
                    prefix,
                    text,
                    Style::default().fg(theme().user_prompt).add_modifier(Modifier::BOLD),
                    &cont_prefix,
                    max_w,
                );
                lines.extend(wrapped);
            }

            // Assistant text
            RuntimeEventType::AssistantText => {
                let text = event.detail.as_deref().unwrap_or("");
                if text.is_empty() { continue; }
                let mut content = String::new();
                if let Some(ai) = event.ai_tool.as_deref() {
                    content.push_str(crate::tui::theme::Theme::ai_tool_badge(ai));
                    content.push(' ');
                }
                content.push_str(text);
                let color = if is_sub && !is_focused {
                    sub_agent_map.get(event.agent_id.as_str())
                        .map(|(_, c)| *c)
                        .unwrap_or(theme().sub_agent_text)
                } else {
                    theme().assistant_text
                };
                let wrapped = wrap_with_tree(prefix, &content, Style::default().fg(color), &cont_prefix, max_w);
                lines.extend(wrapped);
            }

            // Tool start
            RuntimeEventType::ToolStart => {
                if event.tool_name.is_none() { continue; }
                let t = theme();
                let tool_color = match event.tool_name.as_deref().unwrap_or("") {
                    "Read" | "Grep" | "Glob" => t.tool_read,
                    "Edit" | "Write" => t.tool_edit,
                    "Bash" => t.tool_bash,
                    "Task" | "TaskCreate" | "TaskUpdate" => t.tool_task,
                    _ => t.text,
                };
                let tool_text = format_tool(event);
                let mut content = tool_text;
                if let Some(ai) = event.ai_tool.as_deref() {
                    content.push_str(&format!(" {}", crate::tui::theme::Theme::ai_tool_badge(ai)));
                }
                let wrapped = wrap_with_tree(prefix, &content, Style::default().fg(tool_color), &cont_prefix, max_w);
                lines.extend(wrapped);
            }

            // Tool done — text color matches dot (green/red)
            RuntimeEventType::ToolDone if event.tool_name.is_some() => {
                let tool_text = format_tool(event);
                let content = tool_text;
                let done_text_color = if event.is_error { theme().accent_red } else { theme().accent_green };
                let wrapped = wrap_with_tree(prefix, &content, Style::default().fg(done_text_color), &cont_prefix, max_w);
                lines.extend(wrapped);
            }

            // Permission wait
            RuntimeEventType::PermissionWait => {
                let wrapped = wrap_with_tree(
                    prefix,
                    "waiting for permission...",
                    Style::default().fg(theme().lead_name),
                    &cont_prefix,
                    max_w,
                );
                lines.extend(wrapped);
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
        Paragraph::new(visible).style(Style::default().bg(card_bg())),
        inner,
    );
}

// --- Helpers ---

fn is_korean_locale() -> bool {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .map(|v| v.starts_with("ko"))
        .unwrap_or(false)
}

fn format_elapsed(ts: i64, _snap: &StoreSnapshot, is_latest: bool) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = (now - ts).max(0);
    let ko = is_korean_locale();
    let text = match (is_latest && diff < 120, diff, ko) {
        (true, 0..=59, true) => format!("{}초", diff),
        (true, 0..=59, false) => format!("{}s", diff),
        (true, _, true) => format!("{}분", diff / 60),
        (true, _, false) => format!("{}m", diff / 60),
        (false, 0..=59, true) => "방금".to_string(),
        (false, 0..=59, false) => "now".to_string(),
        (false, 60..=3599, true) => format!("{}분 전", diff / 60),
        (false, 60..=3599, false) => format!("{}m ago", diff / 60),
        (false, _, true) => format!("{}시간 전", diff / 3600),
        (false, _, false) => format!("{}h ago", diff / 3600),
    };
    // Use display width for correct CJK alignment
    let w = display_width(&text);
    let pad = 7usize.saturating_sub(w);
    format!("{}{}", " ".repeat(pad), text)
}

/// Compute display width of a string (handles CJK, emoji).
fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Wrap a line that has prefix spans + a final text span.
/// If the line fits in `max_width`, returns it as-is.
/// Otherwise splits the text content and creates continuation lines
/// with tree continuation prefix.
///
/// `prefix_spans`: everything before the text content (time, tree, icon, tags)
/// `text`: the text content to wrap
/// `text_style`: style for the text
/// `tree_cont`: the tree continuation string for wrapped lines (e.g. " │      ")
/// `max_width`: available width
fn wrap_with_tree<'a>(
    prefix_spans: Vec<Span<'a>>,
    text: &str,
    text_style: Style,
    tree_cont: &str,
    max_width: usize,
) -> Vec<Line<'a>> {
    let prefix_width: usize = prefix_spans.iter().map(|s| display_width(s.content.as_ref())).sum();
    let cont_width = display_width(tree_cont);
    let avail_first = max_width.saturating_sub(prefix_width);
    let avail_cont = max_width.saturating_sub(cont_width);

    if avail_first == 0 || avail_cont == 0 {
        let mut spans = prefix_spans;
        let truncated = truncate_to_width(text, max_width.saturating_sub(prefix_width).saturating_sub(3));
        spans.push(Span::styled(format!("{}...", truncated), text_style));
        return vec![Line::from(spans)];
    }

    // Split by newlines first, then by width
    let mut result: Vec<Line> = Vec::new();
    let mut is_first = true;

    for line_text in text.split('\n') {
        let trimmed = line_text.trim();
        if trimmed.is_empty() && !is_first {
            continue; // skip empty lines in continuation
        }

        let avail = if is_first { avail_first } else { avail_cont };
        let line_w = display_width(trimmed);

        if line_w <= avail {
            // Fits on one line
            if is_first {
                let mut spans = prefix_spans.clone();
                spans.push(Span::styled(trimmed.to_string(), text_style));
                result.push(Line::from(spans));
                is_first = false;
            } else {
                result.push(Line::from(vec![
                    Span::styled(tree_cont.to_string(), Style::default().fg(border())),
                    Span::styled(trimmed.to_string(), text_style),
                ]));
            }
        } else {
            // Need width-based wrapping
            let chunks = if is_first {
                split_by_width(trimmed, avail_first, avail_cont)
            } else {
                split_by_width(trimmed, avail_cont, avail_cont)
            };
            for chunk in &chunks {
                if is_first {
                    let mut spans = prefix_spans.clone();
                    spans.push(Span::styled(chunk.to_string(), text_style));
                    result.push(Line::from(spans));
                    is_first = false;
                } else {
                    result.push(Line::from(vec![
                        Span::styled(tree_cont.to_string(), Style::default().fg(border())),
                        Span::styled(chunk.to_string(), text_style),
                    ]));
                }
            }
        }
    }

    if result.is_empty() {
        let mut spans = prefix_spans;
        spans.push(Span::styled("", text_style));
        result.push(Line::from(spans));
    }

    result
}

/// Split text into chunks: first chunk fits `first_width`, rest fit `cont_width`.
fn split_by_width(text: &str, first_width: usize, cont_width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();
    let mut current_w = 0usize;
    let target_w = first_width;

    // First chunk
    while let Some(&c) = chars.peek() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if current_w + cw > target_w && !current.is_empty() {
            break;
        }
        current.push(c);
        current_w += cw;
        chars.next();
    }
    chunks.push(current);

    // Remaining chunks
    while chars.peek().is_some() {
        let mut chunk = String::new();
        let mut chunk_w = 0usize;
        while let Some(&c) = chars.peek() {
            let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
            if chunk_w + cw > cont_width && !chunk.is_empty() {
                break;
            }
            chunk.push(c);
            chunk_w += cw;
            chars.next();
        }
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
    }

    chunks
}

/// Truncate string to fit within a given display width.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut w = 0usize;
    for c in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if w + cw > max_width {
            break;
        }
        result.push(c);
        w += cw;
    }
    result
}

fn is_sub_agent(e: &FeedEvent, snap: &StoreSnapshot) -> bool {
    snap.agents
        .iter()
        .find(|a| a.agent_id == e.agent_id)
        .map(|a| a.role != AgentRole::Main)
        .unwrap_or(true)
}

/// Build a map of sub-agent agent_id → (index, color) for consistent numbering
fn build_sub_agent_map<'a>(
    agents: &[&'a crate::protocol::types::Agent],
) -> std::collections::HashMap<&'a str, (usize, Color)> {
    let t = theme();
    let mut map = std::collections::HashMap::new();
    let mut idx = 0usize;
    for agent in agents {
        if agent.role != AgentRole::Main {
            let color = t.sub_agent_color(idx);
            map.insert(agent.agent_id.as_str(), (idx, color));
            idx += 1;
        }
    }
    map
}

/// Get the growth stage icon for a sub-agent event
#[allow(dead_code)]
fn sub_agent_stage_icon(e: &FeedEvent, snap: &StoreSnapshot) -> &'static str {
    snap.agents
        .iter()
        .find(|a| a.agent_id == e.agent_id)
        .map(|a| {
            let is_done = a.state == AgentState::Completed;
            match chicken::agent_growth_stage(a.usage_count, is_done) {
                "egg" => "\u{1f95a}",
                "hatching" => "\u{1fab6}",
                "peeking" => "\u{1f425}",
                "chick" => "\u{1f423}",
                "done" => "\u{2b50}",
                _ => "\u{1f95a}",
            }
        })
        .unwrap_or("\u{1f95a}")
}

fn format_tool(e: &FeedEvent) -> String {
    let tool = e.tool_name.as_deref().unwrap_or("?");
    let path = e.file_path.as_deref().unwrap_or("");
    if !path.is_empty() {
        let parts: Vec<&str> = path.rsplit('/').take(3).collect();
        let short_path: String = parts.into_iter().rev().collect::<Vec<_>>().join("/");
        format!("{} {}", tool, short_path)
    } else if let Some(ref detail) = e.detail {
        format!("{} {}", tool, detail)
    } else {
        tool.to_string()
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
