use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

    // Main: left (leader + party) | right (feed)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Fill(1)])
        .split(area);

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

    // Find the leader (AgentRole::Main or first agent)
    let leader = snap
        .agents
        .iter()
        .find(|a| a.role == AgentRole::Main)
        .or_else(|| snap.agents.first());

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

    // Bubble: latest AssistantText for the leader
    let bubble_text = snap
        .feed
        .iter()
        .rev()
        .find(|e| {
            e.agent_id == leader.agent_id
                && e.event_type == RuntimeEventType::AssistantText
                && e.detail.is_some()
        })
        .and_then(|e| e.detail.clone())
        .unwrap_or_else(|| "...".to_string());

    let bw = li.width.saturating_sub(2);
    let bx = li.x + 1;

    if bw > 6 {
        let top_b = format!(
            "\u{256d}{}\u{256e}",
            "\u{2500}".repeat((bw - 2) as usize)
        );
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                top_b,
                Style::default().fg(BUBBLE_BORDER),
            )))
            .style(Style::default().bg(CARD_BG)),
            Rect::new(bx, y, bw, 1),
        );
        y += 1;

        let tw = (bw - 4) as usize;
        let dt: String = {
            let chars: Vec<char> = bubble_text.chars().collect();
            if chars.len() > tw {
                let truncated: String = chars[..tw.saturating_sub(3)].iter().collect();
                format!("{}...", truncated)
            } else {
                bubble_text
            }
        };
        let display_w = dt.chars().count();
        let pad = tw.saturating_sub(display_w);
        let content = format!("\u{2502} {}{} \u{2502}", dt, " ".repeat(pad));
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                content,
                Style::default().fg(Color::Rgb(180, 180, 210)),
            )))
            .style(Style::default().bg(BUBBLE_BG)),
            Rect::new(bx, y, bw, 1),
        );
        y += 1;

        let btm_b = format!(
            "\u{2570}{}\u{256f}",
            "\u{2500}".repeat((bw - 2) as usize)
        );
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                btm_b,
                Style::default().fg(BUBBLE_BORDER),
            )))
            .style(Style::default().bg(CARD_BG)),
            Rect::new(bx, y, bw, 1),
        );
        y += 1;

        // Pointer
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " \u{25bd}",
                Style::default().fg(BUBBLE_BORDER),
            )))
            .style(Style::default().bg(CARD_BG)),
            Rect::new(li.x + li.width / 2 - 1, y, 3, 1),
        );
        y += 1;
    }

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
    let party_members: Vec<_> = snap
        .agents
        .iter()
        .filter(|a| a.agent_id != leader.agent_id)
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

        // State / tool info
        let state_y = name_y + 1;
        if state_y < li.y + li.height {
            let st = member
                .current_skill
                .map(|s| format!("{}", s))
                .unwrap_or_else(|| format!("{}", member.state));
            let sc = if is_done {
                DIM
            } else if is_waiting {
                Color::Rgb(200, 200, 80)
            } else {
                Color::Rgb(100, 220, 140)
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    format!("{:^width$}", st, width = col_w as usize),
                    Style::default().fg(sc),
                )))
                .style(Style::default().bg(CARD_BG)),
                Rect::new(mx, state_y, col_w, 1),
            );
        }
    }
}

fn render_right_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let feed_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_commands_panel(f, feed_cols[0], app, snap);
    render_thinking_panel(f, feed_cols[1], app, snap);
}

fn render_commands_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let cmd_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" commands ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(100, 180, 255))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(CARD_BG));

    // Filter for tool events with actual tool names (skip empty ToolDone)
    let filter = &app.filter_text;
    let cmd_events: Vec<&FeedEvent> = snap
        .feed
        .iter()
        .filter(|e| {
            e.tool_name.is_some()
                && (e.event_type == RuntimeEventType::ToolStart
                    || e.event_type == RuntimeEventType::ToolDone)
        })
        .filter(|e| {
            if filter.is_empty() {
                true
            } else {
                let f_lower = filter.to_lowercase();
                e.display_name.to_lowercase().contains(&f_lower)
                    || e.tool_name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&f_lower)
                    || e.file_path
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&f_lower)
            }
        })
        .collect();

    // Show last N that fit
    let inner_h = area.height.saturating_sub(2) as usize;
    let start = cmd_events.len().saturating_sub(inner_h);

    let cmd_lines: Vec<Line> = cmd_events[start..]
        .iter()
        .map(|e| {
            let elapsed = format_elapsed(e.ts, snap);
            let who = format_who(e, snap);
            let who_color = who_color(e, snap);
            let tool_text = format_tool(e);
            let tool_color = if e.is_error {
                Color::Red
            } else {
                match e.event_type {
                    RuntimeEventType::ToolStart => Color::Cyan,
                    RuntimeEventType::ToolDone => Color::Yellow,
                    _ => Color::White,
                }
            };
            Line::from(vec![
                Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                Span::styled(format!("{:<5} ", who), Style::default().fg(who_color)),
                Span::styled(tool_text, Style::default().fg(tool_color)),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(cmd_lines).block(cmd_block), area);
}

fn render_thinking_panel(f: &mut Frame, area: Rect, app: &App, snap: &StoreSnapshot) {
    let thought_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" thinking ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(200, 160, 255))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(CARD_BG));

    let filter = &app.filter_text;
    let text_events: Vec<&FeedEvent> = snap
        .feed
        .iter()
        .filter(|e| e.event_type == RuntimeEventType::AssistantText)
        .filter(|e| {
            if filter.is_empty() {
                true
            } else {
                let f_lower = filter.to_lowercase();
                e.display_name.to_lowercase().contains(&f_lower)
                    || e.detail
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&f_lower)
            }
        })
        .collect();

    let inner_h = area.height.saturating_sub(2) as usize;
    let start = text_events.len().saturating_sub(inner_h);

    let thought_lines: Vec<Line> = text_events[start..]
        .iter()
        .map(|e| {
            let elapsed = format_elapsed(e.ts, snap);
            let who = format_who(e, snap);
            let text = e
                .detail
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>();

            let text_color = if e.is_error {
                Color::Rgb(255, 140, 140)
            } else {
                let is_sub = is_sub_agent(e, snap);
                if is_sub {
                    Color::Rgb(160, 180, 200)
                } else {
                    Color::Rgb(180, 160, 220)
                }
            };

            let prefix = if is_sub_agent(e, snap) {
                format!("\u{2514}{}: ", who)
            } else {
                String::new()
            };

            Line::from(vec![
                Span::styled(format!(" {:>3} ", elapsed), Style::default().fg(DIM)),
                Span::styled(prefix, Style::default().fg(text_color)),
                Span::styled(text, Style::default().fg(text_color)),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(thought_lines).block(thought_block), area);
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
        // Shorten path to last component
        let short_path = path.rsplit('/').next().unwrap_or(path);
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
