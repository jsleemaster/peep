//! Demo D: Card per agent with split feed (commands vs thoughts)
use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

type Pixel = Option<Color>;
fn n() -> Pixel { None }

const BG: Color = Color::Rgb(18, 18, 28);
const CARD_BG: Color = Color::Rgb(22, 22, 34);
const BORDER: Color = Color::Rgb(40, 40, 55);
const DIM: Color = Color::Rgb(70, 70, 90);
const EW: Color = Color::Rgb(240, 240, 255);
const EP: Color = Color::Rgb(20, 20, 60);

fn sprite_to_lines(pixels: &[Vec<Pixel>]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        for x in 0..pixels[y].len() {
            let top = pixels[y][x];
            let btm = if y + 1 < pixels.len() { pixels[y + 1][x] } else { None };
            let (ch, style) = match (top, btm) {
                (Some(tc), Some(bc)) => ("\u{2580}", Style::default().fg(tc).bg(bc)),
                (Some(tc), None) => ("\u{2580}", Style::default().fg(tc).bg(CARD_BG)),
                (None, Some(bc)) => ("\u{2584}", Style::default().fg(bc).bg(CARD_BG)),
                (None, None) => (" ", Style::default().bg(CARD_BG)),
            };
            spans.push(Span::styled(format!("{}{}", ch, ch), style));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

// Monitor face colors
const FRAME_COLOR: Color = Color::Rgb(60, 60, 75);
const FRAME_LIGHT: Color = Color::Rgb(80, 80, 95);
const SCREEN_BG: Color = Color::Rgb(25, 30, 40);
const STAND_COLOR: Color = Color::Rgb(50, 50, 65);
const MOUTH_HAPPY: Color = Color::Rgb(100, 220, 140);
const MOUTH_LINE: Color = Color::Rgb(150, 150, 170);

/// Active monitor: happy face on screen, eyes looking around
fn monitor_active(body: Color, tick: usize) -> Vec<Vec<Pixel>> {
    let f = Some(FRAME_COLOR); let fl = Some(FRAME_LIGHT);
    let s = Some(SCREEN_BG); let n = n();
    let w = Some(EW); let p = Some(EP);
    let m = Some(MOUTH_HAPPY);
    let st = Some(STAND_COLOR);
    let b = Some(body); // accent color on frame

    // Eye position shifts with tick
    let eye_shift = (tick / 8) % 3; // 0=left, 1=center, 2=right

    let (eye_l, eye_r) = match eye_shift {
        0 => (
            vec![p, w, s, s],  // looking left
            vec![p, w, s, s],
        ),
        2 => (
            vec![s, s, w, p],  // looking right
            vec![s, s, w, p],
        ),
        _ => (
            vec![s, w, p, s],  // center
            vec![s, w, p, s],
        ),
    };

    vec![
        vec![n,  b,  fl, fl, fl, fl, fl, b,  n ],
        vec![b,  f,  s,  s,  s,  s,  s,  f,  b ],
        vec![f,  s,  eye_l[0], eye_l[1], s, eye_r[0], eye_r[1], s,  f ],
        vec![f,  s,  eye_l[2], eye_l[3], s, eye_r[2], eye_r[3], s,  f ],
        vec![f,  s,  s,  m,  m,  m,  s,  s,  f ],
        vec![f,  s,  s,  s,  s,  s,  s,  s,  f ],
        vec![n,  b,  f,  f,  f,  f,  f,  b,  n ],
        vec![n,  n,  n,  st, st, st, n,  n,  n ],
    ]
}

/// Waiting monitor: sleepy face, half-closed eyes
fn monitor_waiting(body: Color, tick: usize) -> Vec<Vec<Pixel>> {
    let f = Some(FRAME_COLOR); let fl = Some(FRAME_LIGHT);
    let s = Some(SCREEN_BG); let n = n();
    let ml = Some(MOUTH_LINE);
    let st = Some(STAND_COLOR);
    let b = Some(body);
    let half = Some(Color::Rgb(120, 120, 150)); // half-closed eyelid

    let blink = (tick / 12) % 4 == 0;
    let eye = if blink { s } else { half };

    vec![
        vec![n,  b,  fl, fl, fl, fl, fl, b,  n ],
        vec![b,  f,  s,  s,  s,  s,  s,  f,  b ],
        vec![f,  s,  s,  s,  s,  s,  s,  s,  f ],
        vec![f,  s,  eye, eye, s,  eye, eye, s,  f ],
        vec![f,  s,  s,  ml, ml, ml, s,  s,  f ],
        vec![f,  s,  s,  s,  s,  s,  s,  s,  f ],
        vec![n,  b,  f,  f,  f,  f,  f,  b,  n ],
        vec![n,  n,  n,  st, st, st, n,  n,  n ],
    ]
}

/// Done monitor: checkmark on screen, green tint
fn monitor_done(body: Color) -> Vec<Vec<Pixel>> {
    let f = Some(FRAME_COLOR); let fl = Some(FRAME_LIGHT);
    let s = Some(SCREEN_BG); let n = n();
    let st = Some(STAND_COLOR);
    let b = Some(body);
    let c = Some(Color::Rgb(80, 220, 120)); // checkmark green

    vec![
        vec![n,  b,  fl, fl, fl, fl, fl, b,  n ],
        vec![b,  f,  s,  s,  s,  s,  s,  f,  b ],
        vec![f,  s,  s,  s,  s,  s,  c,  s,  f ],
        vec![f,  s,  s,  s,  s,  c,  s,  s,  f ],
        vec![f,  s,  c,  s,  c,  s,  s,  s,  f ],
        vec![f,  s,  s,  c,  s,  s,  s,  s,  f ],
        vec![n,  b,  f,  f,  f,  f,  f,  b,  n ],
        vec![n,  n,  n,  st, st, st, n,  n,  n ],
    ]
}

/// Error monitor: red X on screen
fn monitor_error(body: Color) -> Vec<Vec<Pixel>> {
    let f = Some(FRAME_COLOR); let fl = Some(FRAME_LIGHT);
    let s = Some(SCREEN_BG); let n = n();
    let st = Some(STAND_COLOR);
    let b = Some(body);
    let r = Some(Color::Rgb(255, 80, 80));

    vec![
        vec![n,  b,  fl, fl, fl, fl, fl, b,  n ],
        vec![b,  f,  s,  s,  s,  s,  s,  f,  b ],
        vec![f,  s,  r,  s,  s,  s,  r,  s,  f ],
        vec![f,  s,  s,  r,  s,  r,  s,  s,  f ],
        vec![f,  s,  s,  s,  r,  s,  s,  s,  f ],
        vec![f,  s,  s,  r,  s,  r,  s,  s,  f ],
        vec![n,  b,  f,  f,  f,  f,  f,  b,  n ],
        vec![n,  n,  n,  st, st, st, n,  n,  n ],
    ]
}

struct FeedEntry {
    time: &'static str,
    text: &'static str,
    color: Color,
}

struct AgentData {
    name: &'static str,
    state: &'static str,
    tool: &'static str,
    body: Color,
    ctx_pct: u16,
    tokens: &'static str,
    cost: &'static str,
    is_waiting: bool,
    is_done: bool,
    commands: Vec<FeedEntry>,
    thoughts: Vec<FeedEntry>,
}

fn render_agent_card(f: &mut Frame, area: Rect, agent: &AgentData, tick: usize) {
    let active_border = if agent.is_done {
        Color::Rgb(50, 80, 60)
    } else if agent.is_waiting {
        Color::Rgb(60, 60, 80)
    } else {
        Color::Rgb(60, 50, 80)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(active_border))
        .title(format!(" {} ", agent.name))
        .title_style(Style::default().fg(agent.body).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(CARD_BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 12 || inner.height < 4 {
        return;
    }

    // Top section: sprite + info side by side
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Fill(1)])
        .split(Rect::new(inner.x, inner.y, inner.width, 5.min(inner.height)));

    // Sprite (9x8 monitor face)
    let sprite = if agent.is_done {
        monitor_done(agent.body)
    } else if agent.is_waiting {
        monitor_waiting(agent.body, tick)
    } else {
        monitor_active(agent.body, tick)
    };

    let sprite_lines = sprite_to_lines(&sprite);
    let sprite_x = top_chunks[0].x + 1;
    for (j, line) in sprite_lines.iter().enumerate() {
        let y = top_chunks[0].y + j as u16;
        if y < top_chunks[0].y + top_chunks[0].height {
            f.render_widget(
                Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)),
                Rect::new(sprite_x, y, 18, 1),
            );
        }
    }

    // Effects
    if agent.is_waiting {
        let zzz_frame = (tick / 10) % 4;
        let zzz = ["z", " zz", "  zzz", " zz"][zzz_frame];
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(zzz, Style::default().fg(Color::Rgb(130, 130, 180)).bg(CARD_BG)))),
            Rect::new(sprite_x + 16, top_chunks[0].y, 5, 1),
        );
    } else if !agent.is_done {
        if (tick / 3) % 4 != 0 {
            f.render_widget(
                Paragraph::new(Line::from(Span::styled("*", Style::default().fg(Color::Rgb(255, 255, 100)).bg(CARD_BG)))),
                Rect::new(sprite_x + 16, top_chunks[0].y + 1, 2, 1),
            );
        }
    }

    // Info next to sprite
    let info_area = top_chunks[1];
    if info_area.width >= 8 {
        let mut info_lines = Vec::new();

        // State/tool
        let state_color = if agent.is_done { DIM } else if agent.is_waiting { Color::Rgb(200, 200, 80) } else { Color::Rgb(100, 220, 140) };
        let state_text = if !agent.tool.is_empty() { agent.tool } else { agent.state };
        info_lines.push(Line::from(Span::styled(state_text, Style::default().fg(state_color))));

        // Blank
        info_lines.push(Line::raw(""));

        // Gauge
        if !agent.is_done {
            let filled = (agent.ctx_pct / 10) as usize;
            let empty = 10usize.saturating_sub(filled);
            let gc = if agent.ctx_pct > 80 { Color::Red } else if agent.ctx_pct > 60 { Color::Yellow } else { Color::Rgb(80, 200, 120) };
            info_lines.push(Line::from(vec![
                Span::styled(format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty)), Style::default().fg(gc)),
                Span::styled(format!(" {}%", agent.ctx_pct), Style::default().fg(gc)),
            ]));
        }

        // Tokens + cost
        info_lines.push(Line::from(vec![
            Span::styled(format!("{} ", agent.tokens), Style::default().fg(Color::Rgb(140, 140, 160))),
            Span::styled(agent.cost, Style::default().fg(Color::Rgb(80, 200, 120))),
        ]));

        let info_para = Paragraph::new(info_lines).style(Style::default().bg(CARD_BG));
        f.render_widget(info_para, info_area);
    }

    // Bottom section: split feed (commands | thoughts)
    let feed_y = inner.y + 5;
    let feed_h = inner.height.saturating_sub(5);
    if feed_h < 2 {
        return;
    }
    let feed_area = Rect::new(inner.x, feed_y, inner.width, feed_h);

    // Separator line
    let sep = Line::from(Span::styled(
        "\u{2500}".repeat(inner.width as usize),
        Style::default().fg(Color::Rgb(35, 35, 50)),
    ));
    f.render_widget(Paragraph::new(sep).style(Style::default().bg(CARD_BG)), Rect::new(inner.x, feed_y, inner.width, 1));

    // Two columns: commands | thoughts
    let feed_content = Rect::new(feed_area.x, feed_area.y + 1, feed_area.width, feed_area.height.saturating_sub(1));

    let feed_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(feed_content);

    // Commands column
    let cmd_header = Line::from(Span::styled(" commands", Style::default().fg(Color::Rgb(100, 180, 255)).add_modifier(Modifier::BOLD)));
    let mut cmd_lines = vec![cmd_header];
    for entry in &agent.commands {
        cmd_lines.push(Line::from(vec![
            Span::styled(format!(" {} ", entry.time), Style::default().fg(DIM)),
            Span::styled(entry.text, Style::default().fg(entry.color)),
        ]));
    }
    f.render_widget(Paragraph::new(cmd_lines).style(Style::default().bg(CARD_BG)), feed_cols[0]);

    // Thoughts column
    let thought_header = Line::from(Span::styled(" thinking", Style::default().fg(Color::Rgb(200, 160, 255)).add_modifier(Modifier::BOLD)));
    let mut thought_lines = vec![thought_header];
    for entry in &agent.thoughts {
        thought_lines.push(Line::from(vec![
            Span::styled(format!(" {} ", entry.time), Style::default().fg(DIM)),
            Span::styled(entry.text, Style::default().fg(entry.color)),
        ]));
    }
    f.render_widget(Paragraph::new(thought_lines).style(Style::default().bg(CARD_BG)), feed_cols[1]);
}

fn render(f: &mut Frame, tick: usize) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(BG)), f.area());

    let agents = vec![
        AgentData {
            name: "main-worker", state: "active", tool: "Edit auth.ts",
            body: Color::Rgb(255, 220, 50), ctx_pct: 67, tokens: "45.2k", cost: "$0.32",
            is_waiting: false, is_done: false,
            commands: vec![
                FeedEntry { time: "2s", text: "Edit auth.ts", color: Color::Yellow },
                FeedEntry { time: "12s", text: "Read config.ts", color: Color::Cyan },
                FeedEntry { time: "30s", text: "Bash cargo build", color: Color::Red },
                FeedEntry { time: "1m", text: "Grep \"TODO\"", color: Color::Cyan },
                FeedEntry { time: "2m", text: "Task Fix bug", color: Color::Magenta },
            ],
            thoughts: vec![
                FeedEntry { time: "3s", text: "auth module needs..", color: Color::Rgb(180, 160, 220) },
                FeedEntry { time: "15s", text: "checking config..", color: Color::Rgb(180, 160, 220) },
                FeedEntry { time: "45s", text: "build succeeded", color: Color::Rgb(140, 200, 140) },
                FeedEntry { time: "1m", text: "found 3 TODOs", color: Color::Rgb(180, 160, 220) },
            ],
        },
        AgentData {
            name: "team-review", state: "waiting", tool: "",
            body: Color::Rgb(80, 200, 255), ctx_pct: 22, tokens: "12.1k", cost: "$0.08",
            is_waiting: true, is_done: false,
            commands: vec![
                FeedEntry { time: "1m", text: "Write test.ts", color: Color::Yellow },
                FeedEntry { time: "3m", text: "Read README", color: Color::Cyan },
            ],
            thoughts: vec![
                FeedEntry { time: "1m", text: "test coverage ok", color: Color::Rgb(180, 160, 220) },
                FeedEntry { time: "2m", text: "waiting for main..", color: Color::Rgb(200, 200, 80) },
            ],
        },
        AgentData {
            name: "sub-scout", state: "active", tool: "Bash npm test",
            body: Color::Rgb(255, 140, 200), ctx_pct: 45, tokens: "8.4k", cost: "$0.06",
            is_waiting: false, is_done: false,
            commands: vec![
                FeedEntry { time: "5s", text: "Bash npm test", color: Color::Red },
                FeedEntry { time: "20s", text: "Grep error logs/", color: Color::Cyan },
                FeedEntry { time: "1m", text: "Edit index.ts", color: Color::Yellow },
            ],
            thoughts: vec![
                FeedEntry { time: "6s", text: "running tests..", color: Color::Rgb(180, 160, 220) },
                FeedEntry { time: "25s", text: "2 failures found", color: Color::Rgb(255, 140, 140) },
            ],
        },
    ];

    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Title bar
    let title = Line::from(vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled("agent monitor", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("  q:quit  Tab:switch", Style::default().fg(Color::Rgb(50, 50, 70))),
    ]);
    f.render_widget(Paragraph::new(title), outer[0]);

    // Agent cards
    let card_constraints: Vec<Constraint> = agents.iter().map(|_| Constraint::Ratio(1, agents.len() as u32)).collect();
    let card_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(card_constraints)
        .split(outer[1]);

    for (i, agent) in agents.iter().enumerate() {
        render_agent_card(f, card_areas[i], agent, tick);
    }

    // Status bar
    let status = Line::from(vec![
        Span::styled(" agents:", Style::default().fg(DIM)),
        Span::styled("3", Style::default().fg(Color::White)),
        Span::styled(" (2\u{25cf} 1\u{25cb})", Style::default().fg(DIM)),
        Span::styled("  \u{2502} tokens:", Style::default().fg(DIM)),
        Span::styled("65.7k", Style::default().fg(Color::White)),
        Span::styled("  \u{2502} cost:", Style::default().fg(DIM)),
        Span::styled("$0.46", Style::default().fg(Color::Rgb(80, 220, 120))),
        Span::styled("  \u{2502} ", Style::default().fg(DIM)),
        Span::styled("\u{25cf}main ", Style::default().fg(Color::Rgb(255, 220, 50))),
        Span::styled("\u{25cf}team ", Style::default().fg(Color::Rgb(80, 200, 255))),
        Span::styled("\u{25cf}sub", Style::default().fg(Color::Rgb(255, 140, 200))),
    ]);
    f.render_widget(Paragraph::new(status), outer[2]);
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(80);
    let mut tick: usize = 0;
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| render(f, tick))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc { break; }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            tick += 1;
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
