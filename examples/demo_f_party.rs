//! Demo F: Leader-centered party UI with maple-style pixel characters
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
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

type Pixel = Option<Color>;
fn n() -> Pixel { None }

const BG: Color = Color::Rgb(18, 18, 28);
const CARD_BG: Color = Color::Rgb(22, 22, 34);
const DIM: Color = Color::Rgb(70, 70, 90);
const BUBBLE_BG: Color = Color::Rgb(32, 32, 48);
const BUBBLE_BORDER: Color = Color::Rgb(55, 55, 75);

fn sprite_to_lines(pixels: &[Vec<Pixel>], bg: Color) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        for x in 0..pixels[y].len() {
            let top = pixels[y][x];
            let btm = if y + 1 < pixels.len() { pixels[y + 1][x] } else { None };
            let (ch, style) = match (top, btm) {
                (Some(tc), Some(bc)) => ("\u{2580}", Style::default().fg(tc).bg(bc)),
                (Some(tc), None) => ("\u{2580}", Style::default().fg(tc).bg(bg)),
                (None, Some(bc)) => ("\u{2584}", Style::default().fg(bc).bg(bg)),
                (None, None) => (" ", Style::default().bg(bg)),
            };
            spans.push(Span::styled(format!("{}{}", ch, ch), style));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

// Colors
const SKIN: Color = Color::Rgb(240, 200, 160);
const SKIN_SHADOW: Color = Color::Rgb(210, 170, 130);
const EYE: Color = Color::Rgb(30, 30, 50);
const MOUTH: Color = Color::Rgb(200, 100, 100);
const SHOE: Color = Color::Rgb(60, 40, 30);
const SHOE_LIGHT: Color = Color::Rgb(80, 55, 40);

/// Big leader character: 12x16 pixel maple-style
/// hair_color, shirt_color, pants_color customizable
fn leader_sprite(
    hair: Color, shirt: Color, shirt_dark: Color, pants: Color,
    tool_color: Option<Color>, frame: usize,
) -> Vec<Vec<Pixel>> {
    let h = Some(hair);
    let hd = Some(Color::Rgb(
        hair.to_string().len() as u8, // just darken
        30, 20,
    ));
    let s = Some(SKIN);
    let sd = Some(SKIN_SHADOW);
    let e = Some(EYE);
    let m = Some(MOUTH);
    let sh = Some(shirt);
    let shd = Some(shirt_dark);
    let p = Some(pants);
    let shoe = Some(SHOE);
    let sl = Some(SHOE_LIGHT);
    let t = tool_color.map(Some).unwrap_or(n());
    let n = n();

    let walk = frame % 2 == 1;

    if walk {
        vec![
            // Hair
            vec![n, n, n, h, h, h, h, h, h, n, n, n],
            vec![n, n, h, h, h, h, h, h, h, h, n, n],
            vec![n, h, h, h, h, h, h, h, h, h, h, n],
            // Face
            vec![n, h, s, s, s, s, s, s, s, s, h, n],
            vec![n, n, s, e, s, s, s, e, s, s, n, n],
            vec![n, n, s, s, s, s, s, s, s, s, n, n],
            vec![n, n, s, s, s, m, s, s, s, n, n, n],
            vec![n, n, n, sd, s, s, s, sd, n, n, n, n],
            // Body
            vec![n, n, n, n, sh, sh, sh, sh, n, n, n, n],
            vec![n, t, n, sh, sh, sh, sh, sh, sh, n, n, n],
            vec![n, t, n, sh, shd, sh, sh, shd, sh, n, n, n],
            vec![n, n, n, n, sh, sh, sh, sh, n, n, n, n],
            // Pants + legs
            vec![n, n, n, n, p, p, p, p, n, n, n, n],
            vec![n, n, n, p, p, n, n, p, p, n, n, n],
            vec![n, n, n, shoe, sl, n, n, n, shoe, n, n, n],
            vec![n, n, shoe, shoe, n, n, n, n, shoe, shoe, n, n],
        ]
    } else {
        vec![
            // Hair
            vec![n, n, n, h, h, h, h, h, h, n, n, n],
            vec![n, n, h, h, h, h, h, h, h, h, n, n],
            vec![n, h, h, h, h, h, h, h, h, h, h, n],
            // Face
            vec![n, h, s, s, s, s, s, s, s, s, h, n],
            vec![n, n, s, e, s, s, s, e, s, s, n, n],
            vec![n, n, s, s, s, s, s, s, s, s, n, n],
            vec![n, n, s, s, s, m, s, s, s, n, n, n],
            vec![n, n, n, sd, s, s, s, sd, n, n, n, n],
            // Body
            vec![n, n, n, n, sh, sh, sh, sh, n, n, n, n],
            vec![n, n, n, sh, sh, sh, sh, sh, sh, n, t, n],
            vec![n, n, n, sh, shd, sh, sh, shd, sh, n, t, n],
            vec![n, n, n, n, sh, sh, sh, sh, n, n, n, n],
            // Pants + legs
            vec![n, n, n, n, p, p, p, p, n, n, n, n],
            vec![n, n, n, n, p, n, n, p, n, n, n, n],
            vec![n, n, n, shoe, sl, n, shoe, sl, n, n, n, n],
            vec![n, n, n, shoe, shoe, n, shoe, shoe, n, n, n, n],
        ]
    }
}

/// Sleeping leader (waiting state)
fn leader_sleeping(hair: Color, shirt: Color, shirt_dark: Color, pants: Color) -> Vec<Vec<Pixel>> {
    let h = Some(hair);
    let s = Some(SKIN);
    let sd = Some(SKIN_SHADOW);
    let e = Some(Color::Rgb(60, 60, 80)); // closed eyes
    let sh = Some(shirt);
    let shd = Some(shirt_dark);
    let p = Some(pants);
    let shoe = Some(SHOE);
    let sl = Some(SHOE_LIGHT);
    let n = n();

    vec![
        vec![n, n, n, h, h, h, h, h, h, n, n, n],
        vec![n, n, h, h, h, h, h, h, h, h, n, n],
        vec![n, h, h, h, h, h, h, h, h, h, h, n],
        vec![n, h, s, s, s, s, s, s, s, s, h, n],
        vec![n, n, s, e, e, s, s, e, e, s, n, n],  // closed eyes (lines)
        vec![n, n, s, s, s, s, s, s, s, s, n, n],
        vec![n, n, s, s, s, s, s, s, s, n, n, n],  // no mouth (sleeping)
        vec![n, n, n, sd, s, s, s, sd, n, n, n, n],
        vec![n, n, n, n, sh, sh, sh, sh, n, n, n, n],
        vec![n, n, n, sh, sh, sh, sh, sh, sh, n, n, n],
        vec![n, n, n, sh, shd, sh, sh, shd, sh, n, n, n],
        vec![n, n, n, n, sh, sh, sh, sh, n, n, n, n],
        vec![n, n, n, n, p, p, p, p, n, n, n, n],
        vec![n, n, n, n, p, n, n, p, n, n, n, n],
        vec![n, n, n, shoe, sl, n, shoe, sl, n, n, n, n],
        vec![n, n, n, shoe, shoe, n, shoe, shoe, n, n, n, n],
    ]
}

/// Party member (10x14) - active, arms visible, walking
fn member_active(hair: Color, shirt: Color, shirt_dark: Color, pants: Color, frame: usize) -> Vec<Vec<Pixel>> {
    let h = Some(hair);
    let s = Some(SKIN);
    let sd = Some(SKIN_SHADOW);
    let e = Some(EYE);
    let m = Some(MOUTH);
    let sh = Some(shirt);
    let shd = Some(shirt_dark);
    let p = Some(pants);
    let shoe = Some(SHOE);
    let sl = Some(SHOE_LIGHT);
    let n = n();

    if frame % 2 == 1 {
        vec![
            vec![n, n, n, h, h, h, h, n, n, n],
            vec![n, n, h, h, h, h, h, h, n, n],
            vec![n, n, h, s, s, s, s, h, n, n],
            vec![n, n, s, e, s, s, e, s, n, n],
            vec![n, n, s, s, s, m, s, s, n, n],
            vec![n, n, n, sd, sd, sd, sd, n, n, n],
            vec![n, s, sh, sh, sh, sh, sh, sh, s, n],  // arms out
            vec![n, s, sh, shd, sh, sh, shd, sh, s, n],
            vec![n, n, n, sh, sh, sh, sh, n, n, n],
            vec![n, n, n, n, p, p, n, n, n, n],
            vec![n, n, n, p, p, n, p, n, n, n],
            vec![n, n, p, n, n, n, n, p, n, n],
            vec![n, shoe, sl, n, n, n, n, shoe, sl, n],
            vec![n, shoe, shoe, n, n, n, n, shoe, shoe, n],
        ]
    } else {
        vec![
            vec![n, n, n, h, h, h, h, n, n, n],
            vec![n, n, h, h, h, h, h, h, n, n],
            vec![n, n, h, s, s, s, s, h, n, n],
            vec![n, n, s, e, s, s, e, s, n, n],
            vec![n, n, s, s, s, m, s, s, n, n],
            vec![n, n, n, sd, sd, sd, sd, n, n, n],
            vec![n, s, sh, sh, sh, sh, sh, sh, s, n],
            vec![n, s, sh, shd, sh, sh, shd, sh, s, n],
            vec![n, n, n, sh, sh, sh, sh, n, n, n],
            vec![n, n, n, n, p, p, n, n, n, n],
            vec![n, n, n, n, p, p, n, n, n, n],
            vec![n, n, n, shoe, sl, shoe, sl, n, n, n],
            vec![n, n, n, shoe, shoe, shoe, shoe, n, n, n],
            vec![n, n, n, n, n, n, n, n, n, n],
        ]
    }
}

/// Party member waiting - eyes closed, arms down relaxed
fn member_waiting(hair: Color, shirt: Color, shirt_dark: Color, pants: Color, _frame: usize) -> Vec<Vec<Pixel>> {
    let h = Some(hair);
    let s = Some(SKIN);
    let sd = Some(SKIN_SHADOW);
    let el = Some(Color::Rgb(80, 70, 100)); // closed eye
    let sh = Some(shirt);
    let shd = Some(shirt_dark);
    let p = Some(pants);
    let shoe = Some(SHOE);
    let sl = Some(SHOE_LIGHT);
    let n = n();

    vec![
        vec![n, n, n, h, h, h, h, n, n, n],
        vec![n, n, h, h, h, h, h, h, n, n],
        vec![n, n, h, s, s, s, s, h, n, n],
        vec![n, n, s, el, el, el, el, s, n, n],  // eyes closed
        vec![n, n, s, s, s, s, s, s, n, n],
        vec![n, n, n, sd, sd, sd, sd, n, n, n],
        vec![n, s, sh, sh, sh, sh, sh, sh, s, n],
        vec![n, n, sh, shd, sh, sh, shd, sh, n, n],
        vec![n, n, n, sh, sh, sh, sh, n, n, n],
        vec![n, n, n, n, p, p, n, n, n, n],
        vec![n, n, n, n, p, p, n, n, n, n],
        vec![n, n, n, shoe, sl, shoe, sl, n, n, n],
        vec![n, n, n, shoe, shoe, shoe, shoe, n, n, n],
        vec![n, n, n, n, n, n, n, n, n, n],
    ]
}

/// Party member done - trophy held up
fn member_done(hair: Color, shirt: Color, shirt_dark: Color, pants: Color) -> Vec<Vec<Pixel>> {
    let h = Some(hair);
    let s = Some(SKIN);
    let sd = Some(SKIN_SHADOW);
    let e = Some(EYE);
    let sh = Some(shirt);
    let shd = Some(shirt_dark);
    let p = Some(pants);
    let shoe = Some(SHOE);
    let sl = Some(SHOE_LIGHT);
    let star = Some(Color::Rgb(255, 220, 80));
    let n = n();

    vec![
        vec![n, n, n, n, n, n, n, n, star, n],
        vec![n, n, n, h, h, h, h, n, star, n],
        vec![n, n, h, h, h, h, h, h, star, n],
        vec![n, n, h, s, s, s, s, h, s, n],
        vec![n, n, s, e, s, s, e, s, n, n],
        vec![n, n, s, s, s, s, s, s, n, n],
        vec![n, n, n, sd, sd, sd, sd, n, n, n],
        vec![n, s, sh, sh, sh, sh, sh, sh, s, n],
        vec![n, n, sh, shd, sh, sh, shd, sh, n, n],
        vec![n, n, n, sh, sh, sh, sh, n, n, n],
        vec![n, n, n, n, p, p, n, n, n, n],
        vec![n, n, n, shoe, sl, shoe, sl, n, n, n],
        vec![n, n, n, shoe, shoe, shoe, shoe, n, n, n],
        vec![n, n, n, n, n, n, n, n, n, n],
    ]
}

fn class_info(class: &str) -> (&str, Color) {
    match class {
        "explorer" => ("\u{1f50d} explorer", Color::Rgb(100, 200, 255)),
        "craftsman" => ("\u{1f528} craftsman", Color::Rgb(255, 200, 80)),
        "warrior" => ("\u{2694}\u{fe0f} warrior", Color::Rgb(255, 120, 120)),
        "commander" => ("\u{1f3f4} commander", Color::Rgb(200, 130, 255)),
        _ => ("agent", DIM),
    }
}

fn tool_color(class: &str) -> Option<Color> {
    match class {
        "explorer" => Some(Color::Rgb(100, 200, 255)),
        "craftsman" => Some(Color::Rgb(255, 200, 80)),
        "warrior" => Some(Color::Rgb(255, 100, 100)),
        "commander" => Some(Color::Rgb(200, 100, 255)),
        _ => None,
    }
}

struct PartyMember {
    name: &'static str,
    state: &'static str,
    tool: &'static str,
    color: Color,       // accent color
    hair: Color,
    shirt: Color,
    shirt_dark: Color,
    pants: Color,
    ctx_pct: u16,
    tokens: &'static str,
    is_waiting: bool,
    is_done: bool,
}

struct LeaderData {
    name: &'static str,
    bubble: &'static str,
    class: &'static str,
    hair: Color,
    shirt: Color,
    shirt_dark: Color,
    pants: Color,
    accent: Color,
    ctx_pct: u16,
    tokens: &'static str,
    cost: &'static str,
    location: &'static str,
    is_waiting: bool,
    party: Vec<PartyMember>,
    feed: Vec<(&'static str, &'static str, &'static str, Color)>, // (time, who, text, color)
    thoughts: Vec<(&'static str, &'static str, Color)>, // (time, text, color)
}

fn render(f: &mut Frame, tick: usize) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(BG)), f.area());

    let leader = LeaderData {
        name: "main-worker",
        bubble: "auth module needs refactoring, spawning sub-agents for testing and review...",
        class: "explorer",
        hair: Color::Rgb(50, 35, 25),
        shirt: Color::Rgb(60, 100, 200),
        shirt_dark: Color::Rgb(40, 70, 160),
        pants: Color::Rgb(70, 70, 90),
        accent: Color::Rgb(100, 160, 255),
        ctx_pct: 67,
        tokens: "45.2k",
        cost: "$0.32",
        location: "src/auth.ts",
        is_waiting: false,
        party: vec![
            PartyMember {
                name: "team-review", state: "waiting", tool: "",
                color: Color::Rgb(80, 200, 200),
                hair: Color::Rgb(180, 140, 60), shirt: Color::Rgb(80, 160, 150), shirt_dark: Color::Rgb(50, 120, 110), pants: Color::Rgb(60, 60, 80),
                ctx_pct: 22, tokens: "12.1k", is_waiting: true, is_done: false,
            },
            PartyMember {
                name: "sub-scout", state: "active", tool: "Bash npm test",
                color: Color::Rgb(255, 140, 200),
                hair: Color::Rgb(30, 30, 40), shirt: Color::Rgb(200, 80, 140), shirt_dark: Color::Rgb(160, 50, 110), pants: Color::Rgb(70, 60, 80),
                ctx_pct: 45, tokens: "8.4k", is_waiting: false, is_done: false,
            },
            PartyMember {
                name: "sub-deploy", state: "done", tool: "",
                color: Color::Rgb(80, 220, 120),
                hair: Color::Rgb(100, 60, 30), shirt: Color::Rgb(60, 170, 100), shirt_dark: Color::Rgb(40, 130, 70), pants: Color::Rgb(70, 70, 80),
                ctx_pct: 0, tokens: "20.0k", is_waiting: false, is_done: true,
            },
        ],
        feed: vec![
            ("2s", "lead", "Edit src/auth.ts", Color::Yellow),
            ("5s", "\u{2514}sub", "Bash npm test", Color::Red),
            ("12s", "lead", "Read src/config.ts", Color::Cyan),
            ("20s", "\u{2514}sub", "Grep error logs/", Color::Cyan),
            ("30s", "\u{2514}team", "Write test/auth.test.ts", Color::Yellow),
            ("1m", "lead", "Bash cargo build", Color::Red),
            ("1m", "\u{2514}sub", "Edit src/index.ts", Color::Yellow),
            ("2m", "lead", "Grep \"TODO\" src/", Color::Cyan),
            ("2m", "\u{2514}team", "Read README.md", Color::Cyan),
            ("3m", "lead", "Task: Fix login bug", Color::Magenta),
        ],
        thoughts: vec![
            ("3s", "auth module needs refactoring for JWT..", Color::Rgb(180, 160, 220)),
            ("8s", "\u{2514}sub: running full test suite..", Color::Rgb(160, 180, 200)),
            ("15s", "checking config dependencies..", Color::Rgb(180, 160, 220)),
            ("25s", "\u{2514}sub: 2 test failures detected!", Color::Rgb(255, 140, 140)),
            ("35s", "\u{2514}team: test coverage looks good", Color::Rgb(160, 200, 160)),
            ("45s", "build passed! proceeding..", Color::Rgb(140, 220, 140)),
            ("1m", "found 3 TODOs to address", Color::Rgb(180, 160, 220)),
            ("2m", "\u{2514}team: waiting for lead changes..", Color::Rgb(200, 200, 120)),
        ],
    };

    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // title
            Constraint::Fill(1),     // main content
            Constraint::Length(1),   // status
        ])
        .split(area);

    // Title
    let title = Line::from(vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled("party monitor", Style::default().fg(DIM)),
        Span::styled("  q:quit", Style::default().fg(Color::Rgb(40, 40, 60))),
    ]);
    f.render_widget(Paragraph::new(title), outer[0]);

    // Main content: left (character + party) | right (feed)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Fill(1)])
        .split(outer[1]);

    // === LEFT: Character panel ===
    let left = main_chunks[0];
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 55)))
        .style(Style::default().bg(CARD_BG));
    let left_inner = left_block.inner(left);
    f.render_widget(left_block, left);

    let mut y = left_inner.y;

    // Leader name
    let name_line = Line::from(vec![
        Span::styled(
            format!(" {} ", leader.name),
            Style::default().fg(leader.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" LEAD ", Style::default().fg(Color::Rgb(255, 220, 80)).bg(Color::Rgb(80, 60, 20)).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(name_line).style(Style::default().bg(CARD_BG)), Rect::new(left_inner.x, y, left_inner.width, 1));
    y += 1;

    // Speech bubble
    let bw = left_inner.width.saturating_sub(2);
    let bubble_x = left_inner.x + 1;

    let top_b = format!("\u{256d}{}\u{256e}", "\u{2500}".repeat((bw - 2) as usize));
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(&top_b, Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)),
        Rect::new(bubble_x, y, bw, 1),
    );
    y += 1;

    // Bubble text (wrap to 2 lines)
    let text_w = (bw - 4) as usize;
    let lines_text: Vec<&str> = leader.bubble.as_bytes()
        .chunks(text_w)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .take(2)
        .collect();

    for line_text in &lines_text {
        let content = format!("\u{2502} {:<width$} \u{2502}", line_text, width = text_w);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(&content, Style::default().fg(Color::Rgb(180, 180, 210))))).style(Style::default().bg(BUBBLE_BG)),
            Rect::new(bubble_x, y, bw, 1),
        );
        y += 1;
    }

    let btm_b = format!("\u{2570}{}\u{256f}", "\u{2500}".repeat((bw - 2) as usize));
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(&btm_b, Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)),
        Rect::new(bubble_x, y, bw, 1),
    );
    y += 1;

    // Pointer
    f.render_widget(
        Paragraph::new(Line::from(Span::styled("  \u{25bd}", Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)),
        Rect::new(left_inner.x + left_inner.width / 2 - 1, y, 3, 1),
    );
    y += 1;

    // Leader sprite (12x16 → 24 cols x 8 rows)
    let sprite = if leader.is_waiting {
        leader_sleeping(leader.hair, leader.shirt, leader.shirt_dark, leader.pants)
    } else {
        leader_sprite(leader.hair, leader.shirt, leader.shirt_dark, leader.pants, tool_color(leader.class), tick / 5)
    };

    let sprite_lines = sprite_to_lines(&sprite, CARD_BG);
    let sprite_width = 24u16; // 12px * 2
    let sprite_x = left_inner.x + (left_inner.width.saturating_sub(sprite_width)) / 2;

    for (j, line) in sprite_lines.iter().enumerate() {
        let sy = y + j as u16;
        if sy < left_inner.y + left_inner.height {
            f.render_widget(
                Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)),
                Rect::new(sprite_x, sy, sprite_width, 1),
            );
        }
    }
    y += sprite_lines.len() as u16;

    // Class + location
    if y < left_inner.y + left_inner.height {
        let (class_label, class_color) = class_info(leader.class);
        let loc_line = Line::from(vec![
            Span::styled(format!(" {} ", class_label), Style::default().fg(class_color)),
            Span::styled("\u{00b7} ", Style::default().fg(DIM)),
            Span::styled(leader.location, Style::default().fg(Color::Rgb(140, 140, 170))),
        ]);
        f.render_widget(Paragraph::new(loc_line).style(Style::default().bg(CARD_BG)), Rect::new(left_inner.x, y, left_inner.width, 1));
        y += 1;
    }

    // HP bar
    if y < left_inner.y + left_inner.height {
        let filled = (leader.ctx_pct / 10) as usize;
        let empty = 10usize.saturating_sub(filled);
        let hp_color = if leader.ctx_pct > 80 { Color::Rgb(255, 80, 80) } else if leader.ctx_pct > 60 { Color::Rgb(255, 200, 80) } else { Color::Rgb(80, 200, 120) };
        let hp_line = Line::from(vec![
            Span::styled(" HP ", Style::default().fg(DIM)),
            Span::styled("\u{2588}".repeat(filled), Style::default().fg(hp_color)),
            Span::styled("\u{2591}".repeat(empty), Style::default().fg(Color::Rgb(40, 40, 55))),
            Span::styled(format!(" {}%", leader.ctx_pct), Style::default().fg(hp_color)),
            Span::styled(format!("  {} {}", leader.tokens, leader.cost), Style::default().fg(DIM)),
        ]);
        f.render_widget(Paragraph::new(hp_line).style(Style::default().bg(CARD_BG)), Rect::new(left_inner.x, y, left_inner.width, 1));
        y += 1;
    }

    // Separator
    if y < left_inner.y + left_inner.height {
        y += 1;
        let party_label = Line::from(Span::styled(
            format!(" \u{2500}\u{2500} party ({}) \u{2500}{}", leader.party.len(), "\u{2500}".repeat(20)),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(Paragraph::new(party_label).style(Style::default().bg(CARD_BG)), Rect::new(left_inner.x, y, left_inner.width, 1));
        y += 1;
    }

    // Party members in 2-column grid
    if y + 6 < left_inner.y + left_inner.height {
        let cols = 2u16;
        let member_width = left_inner.width / cols;

        for (i, member) in leader.party.iter().enumerate() {
            let col = (i as u16) % cols;
            let row = (i as u16) / cols;
            let row_height = 10u16; // sprite (7 rows) + name + state + gap
            let mx = left_inner.x + col * member_width;
            let base_y = y + row * row_height;

            // Human sprite
            let mini = if member.is_done {
                member_done(member.hair, member.shirt, member.shirt_dark, member.pants)
            } else if member.is_waiting {
                member_waiting(member.hair, member.shirt, member.shirt_dark, member.pants, tick / 8)
            } else {
                member_active(member.hair, member.shirt, member.shirt_dark, member.pants, tick / 5)
            };

            let mini_lines = sprite_to_lines(&mini, CARD_BG);
            let mini_w = 20u16; // 10px * 2 chars
            let mini_x = mx + (member_width.saturating_sub(mini_w)) / 2;

            for (j, line) in mini_lines.iter().enumerate() {
                let my = base_y + j as u16;
                if my < left_inner.y + left_inner.height {
                    f.render_widget(
                        Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)),
                        Rect::new(mini_x, my, mini_w, 1),
                    );
                }
            }

            // Zzz for waiting
            if member.is_waiting {
                let zzz_frame = (tick / 10) % 4;
                let zzz = ["z", " zz", "  zzz", " zz"][zzz_frame];
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(zzz, Style::default().fg(Color::Rgb(120, 120, 170))))).style(Style::default().bg(CARD_BG)),
                    Rect::new(mini_x + mini_w, base_y, 5, 1),
                );
            }

            // Member name below sprite
            let name_y = base_y + mini_lines.len() as u16;
            if name_y < left_inner.y + left_inner.height {
                let label = format!("{:^width$}", member.name, width = member_width as usize);
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(&label[..label.len().min(member_width as usize)], Style::default().fg(member.color)))).style(Style::default().bg(CARD_BG)),
                    Rect::new(mx, name_y, member_width, 1),
                );
            }

            // Member state
            let state_y = name_y + 1;
            if state_y < left_inner.y + left_inner.height {
                let state_text = if !member.tool.is_empty() { member.tool } else { member.state };
                let st_color = if member.is_done { DIM } else if member.is_waiting { Color::Rgb(200, 200, 80) } else { Color::Rgb(100, 220, 140) };
                let st_label = format!("{:^width$}", state_text, width = member_width as usize);
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(&st_label[..st_label.len().min(member_width as usize)], Style::default().fg(st_color)))).style(Style::default().bg(CARD_BG)),
                    Rect::new(mx, state_y, member_width, 1),
                );
            }

        }
    }

    // === RIGHT: Feed panel ===
    let right = main_chunks[1];

    let right_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(right);

    // Commands feed
    let cmd_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" commands ")
        .title_style(Style::default().fg(Color::Rgb(100, 180, 255)).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(CARD_BG));

    let mut cmd_lines = Vec::new();
    for (time, who, text, color) in &leader.feed {
        let who_color = if who.contains("sub") {
            Color::Rgb(220, 100, 160)
        } else if who.contains("team") {
            Color::Rgb(80, 180, 160)
        } else {
            leader.accent
        };

        cmd_lines.push(Line::from(vec![
            Span::styled(format!(" {:>3} ", time), Style::default().fg(DIM)),
            Span::styled(format!("{:<5} ", who), Style::default().fg(who_color)),
            Span::styled(*text, Style::default().fg(*color)),
        ]));
    }
    let cmd_para = Paragraph::new(cmd_lines).block(cmd_block);
    f.render_widget(cmd_para, right_chunks[0]);

    // Thoughts feed
    let thought_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" thinking ")
        .title_style(Style::default().fg(Color::Rgb(200, 160, 255)).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(CARD_BG));

    let mut thought_lines = Vec::new();
    for (time, text, color) in &leader.thoughts {
        thought_lines.push(Line::from(vec![
            Span::styled(format!(" {:>3} ", time), Style::default().fg(DIM)),
            Span::styled(*text, Style::default().fg(*color)),
        ]));
    }
    let thought_para = Paragraph::new(thought_lines).block(thought_block);
    f.render_widget(thought_para, right_chunks[1]);

    // Status bar
    let status = Line::from(vec![
        Span::styled(" party:", Style::default().fg(DIM)),
        Span::styled("4", Style::default().fg(Color::White)),
        Span::styled("  \u{2502} tokens:", Style::default().fg(DIM)),
        Span::styled("85.7k", Style::default().fg(Color::White)),
        Span::styled("  \u{2502} cost:", Style::default().fg(DIM)),
        Span::styled("$0.60", Style::default().fg(Color::Rgb(255, 220, 80))),
        Span::styled("  \u{2502} ", Style::default().fg(DIM)),
        Span::styled(format!("\u{25cf}{} ", leader.name), Style::default().fg(leader.accent)),
        Span::styled(format!("\u{25cf}{} ", leader.party[0].name), Style::default().fg(leader.party[0].color)),
        Span::styled(format!("\u{25cf}{} ", leader.party[1].name), Style::default().fg(leader.party[1].color)),
        Span::styled(format!("\u{25cf}{}", leader.party[2].name), Style::default().fg(leader.party[2].color)),
    ]);
    f.render_widget(Paragraph::new(status), outer[2]);
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(100);
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
