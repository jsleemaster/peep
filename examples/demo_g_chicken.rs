//! Demo G: Chicken & Eggs party concept
//! Leader = mother hen, new sub-agents = eggs, grown subs = chicks
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

// Chicken colors
const WHITE: Color = Color::Rgb(245, 240, 230);
const CREAM: Color = Color::Rgb(230, 220, 200);
const COMB: Color = Color::Rgb(220, 50, 50);      // red comb
const COMB_DARK: Color = Color::Rgb(180, 30, 30);
const BEAK: Color = Color::Rgb(255, 180, 50);
const BEAK_DARK: Color = Color::Rgb(220, 150, 30);
const EYE: Color = Color::Rgb(20, 20, 30);
const WING: Color = Color::Rgb(220, 210, 190);
const WING_DARK: Color = Color::Rgb(200, 190, 170);
const FEET: Color = Color::Rgb(230, 160, 40);
const FEET_DARK: Color = Color::Rgb(200, 130, 20);

// Chick colors
const CHICK_BODY: Color = Color::Rgb(255, 230, 80);
const CHICK_DARK: Color = Color::Rgb(230, 200, 50);
const CHICK_WING: Color = Color::Rgb(240, 210, 60);

// Egg colors
const EGG_LIGHT: Color = Color::Rgb(245, 240, 230);
const EGG_MID: Color = Color::Rgb(230, 225, 210);
const EGG_SHADOW: Color = Color::Rgb(200, 195, 180);
const EGG_CRACK: Color = Color::Rgb(180, 170, 150);

/// Mother hen (leader) - 14x14 pixels, facing right
fn chicken_idle(frame: usize) -> Vec<Vec<Pixel>> {
    let w = Some(WHITE); let c = Some(CREAM);
    let co = Some(COMB); let cd = Some(COMB_DARK);
    let b = Some(BEAK); let bd = Some(BEAK_DARK);
    let e = Some(EYE);
    let wi = Some(WING); let wd = Some(WING_DARK);
    let f = Some(FEET); let fd = Some(FEET_DARK);
    let n = n();

    let blink = frame % 20 < 2;
    let eye = if blink { w } else { e };

    vec![
        vec![n,  n,  n,  n,  co, co, co, n,  n,  n,  n,  n,  n,  n ],
        vec![n,  n,  n,  co, cd, co, cd, co, n,  n,  n,  n,  n,  n ],
        vec![n,  n,  n,  n,  w,  w,  w,  w,  n,  n,  n,  n,  n,  n ],
        vec![n,  n,  n,  w,  w,  w,  w,  w,  w,  n,  n,  n,  n,  n ],
        vec![n,  n,  w,  w,  eye,w,  w,  w,  w,  b,  n,  n,  n,  n ],
        vec![n,  n,  w,  w,  w,  w,  w,  w,  b,  bd, n,  n,  n,  n ],
        vec![n,  n,  w,  w,  w,  w,  w,  w,  w,  n,  n,  n,  n,  n ],
        vec![n,  w,  w,  w,  w,  w,  w,  w,  w,  w,  n,  n,  n,  n ],
        vec![n,  w,  wi, wi, w,  c,  c,  w,  w,  w,  w,  n,  n,  n ],
        vec![n,  w,  wd, wi, wi, c,  c,  c,  w,  w,  w,  w,  n,  n ],
        vec![n,  n,  w,  wd, wi, w,  w,  w,  w,  w,  w,  w,  n,  n ],
        vec![n,  n,  n,  w,  w,  w,  c,  c,  w,  w,  w,  n,  n,  n ],
        vec![n,  n,  n,  n,  n,  f,  fd, n,  f,  fd, n,  n,  n,  n ],
        vec![n,  n,  n,  n,  f,  f,  n,  f,  f,  f,  n,  n,  n,  n ],
    ]
}

/// Mother hen pecking (active, working)
fn chicken_peck(frame: usize) -> Vec<Vec<Pixel>> {
    let w = Some(WHITE); let c = Some(CREAM);
    let co = Some(COMB); let cd = Some(COMB_DARK);
    let b = Some(BEAK); let bd = Some(BEAK_DARK);
    let e = Some(EYE);
    let wi = Some(WING); let wd = Some(WING_DARK);
    let f = Some(FEET); let fd = Some(FEET_DARK);
    let n = n();

    if frame % 4 < 2 {
        // Head down pecking
        vec![
            vec![n,  n,  n,  n,  n,  n,  n,  n,  n,  n,  n,  n,  n,  n ],
            vec![n,  n,  n,  n,  co, co, co, n,  n,  n,  n,  n,  n,  n ],
            vec![n,  n,  n,  n,  cd, co, cd, n,  n,  n,  n,  n,  n,  n ],
            vec![n,  n,  n,  n,  w,  w,  w,  w,  n,  n,  n,  n,  n,  n ],
            vec![n,  n,  n,  w,  w,  e,  w,  w,  b,  b,  n,  n,  n,  n ],
            vec![n,  n,  n,  w,  w,  w,  w,  w,  bd, n,  n,  n,  n,  n ],
            vec![n,  n,  w,  w,  w,  w,  w,  w,  w,  n,  n,  n,  n,  n ],
            vec![n,  w,  w,  w,  w,  w,  w,  w,  w,  w,  n,  n,  n,  n ],
            vec![n,  w,  wi, wi, w,  c,  c,  w,  w,  w,  w,  n,  n,  n ],
            vec![n,  w,  wd, wi, wi, c,  c,  c,  w,  w,  w,  w,  n,  n ],
            vec![n,  n,  w,  wd, wi, w,  w,  w,  w,  w,  w,  w,  n,  n ],
            vec![n,  n,  n,  w,  w,  w,  c,  c,  w,  w,  w,  n,  n,  n ],
            vec![n,  n,  n,  n,  n,  f,  fd, n,  f,  fd, n,  n,  n,  n ],
            vec![n,  n,  n,  n,  f,  f,  n,  f,  f,  f,  n,  n,  n,  n ],
        ]
    } else {
        chicken_idle(frame)
    }
}

/// Chick (grown sub-agent) - 8x8 pixels
fn chick_sprite(frame: usize) -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY); let d = Some(CHICK_DARK);
    let wi = Some(CHICK_WING);
    let e = Some(EYE);
    let b = Some(BEAK); let bd = Some(BEAK_DARK);
    let f = Some(FEET); let fd = Some(FEET_DARK);
    let n = n();

    let blink = frame % 16 < 2;
    let eye = if blink { y } else { e };

    // Little hop animation
    if frame % 6 < 3 {
        vec![
            vec![n, n, y, y, y, n, n, n],
            vec![n, y, y, y, y, y, n, n],
            vec![n, y, eye, y, y, y, b, n],
            vec![n, y, y, y, y, y, bd, n],
            vec![wi, y, y, y, y, y, n, n],
            vec![n, y, d, y, d, y, n, n],
            vec![n, n, f, n, f, n, n, n],
            vec![n, f, fd, f, fd, n, n, n],
        ]
    } else {
        // Hop up slightly
        vec![
            vec![n, n, y, y, y, n, n, n],
            vec![n, y, y, y, y, y, n, n],
            vec![n, y, eye, y, y, y, b, n],
            vec![wi, y, y, y, y, y, bd, n],
            vec![n, y, y, y, y, y, n, n],
            vec![n, n, d, y, d, n, n, n],
            vec![n, n, n, n, n, n, n, n],
            vec![n, n, f, n, f, n, n, n],
        ]
    }
}

/// Chick sleeping (waiting)
fn chick_sleeping(frame: usize) -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY); let d = Some(CHICK_DARK);
    let wi = Some(CHICK_WING);
    let el = Some(Color::Rgb(80, 70, 30)); // closed eyes
    let f = Some(FEET);
    let n = n();

    let _ = frame;
    vec![
        vec![n, n, n, n, n, n, n, n],
        vec![n, n, y, y, y, n, n, n],
        vec![n, y, y, y, y, y, n, n],
        vec![n, y, el, y, el, y, n, n],
        vec![n, y, y, y, y, y, n, n],
        vec![wi, y, y, y, y, y, n, n],
        vec![n, y, d, y, d, y, n, n],
        vec![n, n, f, n, f, n, n, n],
    ]
}

/// Egg (new sub-agent) - 6x8 pixels
fn egg_sprite() -> Vec<Vec<Pixel>> {
    let l = Some(EGG_LIGHT); let m = Some(EGG_MID);
    let s = Some(EGG_SHADOW);
    let n = n();

    vec![
        vec![n, n, n, n, n, n],
        vec![n, n, l, l, n, n],
        vec![n, l, l, l, m, n],
        vec![n, l, l, l, m, n],
        vec![l, l, l, l, m, s],
        vec![l, l, l, m, m, s],
        vec![n, l, m, m, s, n],
        vec![n, n, s, s, n, n],
    ]
}

/// Egg with cracks (about to hatch, medium usage)
fn egg_cracking(frame: usize) -> Vec<Vec<Pixel>> {
    let l = Some(EGG_LIGHT); let m = Some(EGG_MID);
    let s = Some(EGG_SHADOW);
    let cr = Some(EGG_CRACK);
    let n = n();

    let wobble = frame % 8 < 4;

    if wobble {
        vec![
            vec![n, n, n, n, n, n],
            vec![n, n, l, l, n, n],
            vec![n, l, cr, l, m, n],
            vec![n, l, l, cr, m, n],
            vec![l, cr, l, l, m, s],
            vec![l, l, l, m, cr, s],
            vec![n, l, m, cr, s, n],
            vec![n, n, s, s, n, n],
        ]
    } else {
        // Slight tilt
        vec![
            vec![n, n, n, n, n, n],
            vec![n, n, n, l, l, n],
            vec![n, n, l, cr, l, n],
            vec![n, l, l, l, cr, n],
            vec![l, cr, l, l, m, s],
            vec![l, l, l, m, cr, s],
            vec![n, l, m, cr, s, n],
            vec![n, n, s, s, n, n],
        ]
    }
}

/// Egg hatching - chick peeking out from cracked shell (8x8)
fn egg_hatching_chick(frame: usize) -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY); let d = Some(CHICK_DARK);
    let e = Some(EYE);
    let b = Some(BEAK);
    let l = Some(EGG_LIGHT); let m = Some(EGG_MID);
    let s = Some(EGG_SHADOW); let cr = Some(EGG_CRACK);
    let n = n();

    let wobble = frame % 6 < 3;

    if wobble {
        // Chick head poking out, shell cracked open
        vec![
            vec![n, n, y, y, y, n, n, n],
            vec![n, n, y, e, y, b, n, n],
            vec![n, n, d, y, d, n, n, n],
            vec![n, cr, l, l, l, cr, n, n],
            vec![l, l, l, l, l, l, m, n],
            vec![l, l, l, l, m, m, s, n],
            vec![n, l, m, m, m, s, n, n],
            vec![n, n, s, s, s, n, n, n],
        ]
    } else {
        // Wobble right, chick ducking slightly
        vec![
            vec![n, n, n, y, y, n, n, n],
            vec![n, n, y, e, y, b, n, n],
            vec![n, cr, d, y, d, cr, n, n],
            vec![n, l, l, l, l, l, n, n],
            vec![l, l, l, l, l, l, m, n],
            vec![l, l, l, l, m, m, s, n],
            vec![n, l, m, m, m, s, n, n],
            vec![n, n, s, s, s, n, n, n],
        ]
    }
}

/// Chick with trophy (done)
fn chick_done() -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY); let d = Some(CHICK_DARK);
    let e = Some(EYE);
    let b = Some(BEAK);
    let f = Some(FEET);
    let star = Some(Color::Rgb(255, 220, 80));
    let n = n();

    vec![
        vec![n, n, n, n, n, n, star, n],
        vec![n, n, y, y, y, n, star, n],
        vec![n, y, y, y, y, y, star, n],
        vec![n, y, e, y, e, y, n, n],
        vec![n, y, y, y, y, y, b, n],
        vec![n, y, y, y, y, y, n, n],
        vec![n, n, d, y, d, n, n, n],
        vec![n, n, f, n, f, n, n, n],
    ]
}

struct PartyMember {
    name: &'static str,
    state: &'static str,
    tool: &'static str,
    usage: u32,       // determines egg/chick evolution
    ctx_pct: u16,
    is_waiting: bool,
    is_done: bool,
}

impl PartyMember {
    fn stage(&self) -> &'static str {
        if self.is_done { "done" }
        else if self.usage >= 20 { "chick" }      // fully hatched
        else if self.usage >= 10 { "peeking" }    // chick peeking from shell
        else if self.usage >= 5 { "hatching" }    // cracks appearing
        else { "egg" }                              // brand new
    }
}

fn render(f: &mut Frame, tick: usize) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(BG)), f.area());

    let party = vec![
        PartyMember { name: "team-review", state: "waiting", tool: "", usage: 22, ctx_pct: 22, is_waiting: true, is_done: false },
        PartyMember { name: "sub-scout", state: "active", tool: "Bash npm test", usage: 12, ctx_pct: 45, is_waiting: false, is_done: false },
        PartyMember { name: "sub-new", state: "active", tool: "Read file.ts", usage: 2, ctx_pct: 10, is_waiting: false, is_done: false },
        PartyMember { name: "sub-deploy", state: "done", tool: "", usage: 30, ctx_pct: 0, is_waiting: false, is_done: true },
    ];

    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Title
    let title = Line::from(vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled("party monitor", Style::default().fg(DIM)),
        Span::styled("  q:quit", Style::default().fg(Color::Rgb(40, 40, 60))),
    ]);
    f.render_widget(Paragraph::new(title), outer[0]);

    // Main: left (leader + party) | right (feed)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Fill(1)])
        .split(outer[1]);

    // === LEFT PANEL ===
    let left = main[0];
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 55)))
        .style(Style::default().bg(CARD_BG));
    let li = left_block.inner(left);
    f.render_widget(left_block, left);

    let mut y = li.y;

    // Leader name + badge
    let name_line = Line::from(vec![
        Span::styled(" main-worker ", Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD)),
        Span::styled(" LEAD ", Style::default().fg(Color::Rgb(255, 220, 80)).bg(Color::Rgb(80, 60, 20)).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(name_line).style(Style::default().bg(CARD_BG)), Rect::new(li.x, y, li.width, 1));
    y += 1;

    // Bubble
    let bw = li.width.saturating_sub(2);
    let bx = li.x + 1;
    let bubble_text = "auth module needs refactoring, spawning sub-agents...";

    let top_b = format!("\u{256d}{}\u{256e}", "\u{2500}".repeat((bw - 2) as usize));
    f.render_widget(Paragraph::new(Line::from(Span::styled(&top_b, Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)), Rect::new(bx, y, bw, 1));
    y += 1;

    let tw = (bw - 4) as usize;
    let dt: String = if bubble_text.len() > tw { format!("{}...", &bubble_text[..tw.saturating_sub(3)]) } else { bubble_text.to_string() };
    let content = format!("\u{2502} {:<w$} \u{2502}", dt, w = tw);
    f.render_widget(Paragraph::new(Line::from(Span::styled(&content, Style::default().fg(Color::Rgb(180, 180, 210))))).style(Style::default().bg(BUBBLE_BG)), Rect::new(bx, y, bw, 1));
    y += 1;

    let btm_b = format!("\u{2570}{}\u{256f}", "\u{2500}".repeat((bw - 2) as usize));
    f.render_widget(Paragraph::new(Line::from(Span::styled(&btm_b, Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)), Rect::new(bx, y, bw, 1));
    y += 1;

    // Pointer
    f.render_widget(Paragraph::new(Line::from(Span::styled(" \u{25bd}", Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)), Rect::new(li.x + li.width / 2 - 1, y, 3, 1));
    y += 1;

    // Chicken sprite
    let chicken = chicken_peck(tick / 4);
    let chicken_lines = sprite_to_lines(&chicken, CARD_BG);
    let cw = 28u16;
    let cx = li.x + (li.width.saturating_sub(cw)) / 2;
    for (j, line) in chicken_lines.iter().enumerate() {
        let sy = y + j as u16;
        if sy < li.y + li.height {
            f.render_widget(Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)), Rect::new(cx, sy, cw, 1));
        }
    }
    y += chicken_lines.len() as u16;

    // Leader stats
    if y < li.y + li.height {
        let hp_line = Line::from(vec![
            Span::styled(" HP ", Style::default().fg(DIM)),
            Span::styled("\u{2588}".repeat(6), Style::default().fg(Color::Rgb(255, 200, 80))),
            Span::styled("\u{2591}".repeat(4), Style::default().fg(Color::Rgb(40, 40, 55))),
            Span::styled(" 67%", Style::default().fg(Color::Rgb(255, 200, 80))),
            Span::styled("  45.2k $0.32", Style::default().fg(DIM)),
        ]);
        f.render_widget(Paragraph::new(hp_line).style(Style::default().bg(CARD_BG)), Rect::new(li.x, y, li.width, 1));
        y += 1;
    }

    // Party separator
    y += 1;
    if y < li.y + li.height {
        let sep = Line::from(Span::styled(
            format!(" \u{2500}\u{2500} party ({}) \u{2500}{}", party.len(), "\u{2500}".repeat(22)),
            Style::default().fg(Color::Rgb(50, 50, 70)),
        ));
        f.render_widget(Paragraph::new(sep).style(Style::default().bg(CARD_BG)), Rect::new(li.x, y, li.width, 1));
        y += 1;
    }

    // Party members in 2-column grid
    let cols = 2u16;
    let col_w = li.width / cols;
    let row_h = 8u16;

    for (i, member) in party.iter().enumerate() {
        let col = (i as u16) % cols;
        let row = (i as u16) / cols;
        let mx = li.x + col * col_w;
        let my = y + row * row_h;

        if my + row_h > li.y + li.height { break; }

        let stage = member.stage();

        // Sprite
        let sprite = match stage {
            "egg" => egg_sprite(),
            "hatching" => egg_cracking(tick / 3),
            "peeking" => egg_hatching_chick(tick / 3),
            "chick" if member.is_waiting => chick_sleeping(tick),
            "chick" => chick_sprite(tick / 3),
            "done" => chick_done(),
            _ => egg_sprite(),
        };

        let spr_lines = sprite_to_lines(&sprite, CARD_BG);
        let spr_w = if stage == "egg" || stage == "hatching" { 12u16 } else { 16u16 };
        let spr_x = mx + (col_w.saturating_sub(spr_w)) / 2;

        for (j, line) in spr_lines.iter().enumerate() {
            let sy = my + j as u16;
            if sy < li.y + li.height {
                f.render_widget(Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)), Rect::new(spr_x, sy, spr_w, 1));
            }
        }

        // Zzz
        if member.is_waiting {
            let zzz_frame = (tick / 10) % 4;
            let zzz = ["z", " zz", "  zzz", " zz"][zzz_frame];
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(zzz, Style::default().fg(Color::Rgb(120, 120, 170))))).style(Style::default().bg(CARD_BG)),
                Rect::new(spr_x + spr_w, my, 5, 1),
            );
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
            let label = format!("{} {}", stage_icon, member.name);
            let color = match stage {
                "egg" => Color::Rgb(200, 195, 180),
                "hatching" | "peeking" => Color::Rgb(230, 200, 100),
                "chick" | "done" => Color::Rgb(255, 220, 80),
                _ => DIM,
            };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(format!("{:^width$}", label, width = col_w as usize), Style::default().fg(color)))).style(Style::default().bg(CARD_BG)),
                Rect::new(mx, name_y, col_w, 1),
            );
        }

        // State
        let state_y = name_y + 1;
        if state_y < li.y + li.height {
            let st = if !member.tool.is_empty() { member.tool } else { member.state };
            let sc = if member.is_done { DIM } else if member.is_waiting { Color::Rgb(200, 200, 80) } else { Color::Rgb(100, 220, 140) };
            f.render_widget(
                Paragraph::new(Line::from(Span::styled(format!("{:^width$}", st, width = col_w as usize), Style::default().fg(sc)))).style(Style::default().bg(CARD_BG)),
                Rect::new(mx, state_y, col_w, 1),
            );
        }
    }

    // === RIGHT PANEL: Feed ===
    let right = main[1];
    let feed_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(right);

    // Commands
    let cmd_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" commands ")
        .title_style(Style::default().fg(Color::Rgb(100, 180, 255)).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(CARD_BG));

    let feed = vec![
        ("2s", "lead", "Edit src/auth.ts", Color::Yellow),
        ("5s", "\u{2514}sub", "Bash npm test", Color::Red),
        ("8s", "\u{2514}new", "Read file.ts", Color::Cyan),
        ("12s", "lead", "Read src/config.ts", Color::Cyan),
        ("20s", "\u{2514}sub", "Grep error logs/", Color::Cyan),
        ("30s", "\u{2514}team", "Write test.ts", Color::Yellow),
        ("1m", "lead", "Bash cargo build", Color::Red),
        ("2m", "lead", "Grep \"TODO\" src/", Color::Cyan),
        ("3m", "lead", "Task: Fix login", Color::Magenta),
    ];

    let cmd_lines: Vec<Line> = feed.iter().map(|(time, who, text, color)| {
        let who_color = if who.contains("sub") || who.contains("new") {
            Color::Rgb(255, 220, 80)
        } else if who.contains("team") {
            Color::Rgb(80, 200, 200)
        } else {
            Color::Rgb(255, 200, 80)
        };
        Line::from(vec![
            Span::styled(format!(" {:>3} ", time), Style::default().fg(DIM)),
            Span::styled(format!("{:<5} ", who), Style::default().fg(who_color)),
            Span::styled(*text, Style::default().fg(*color)),
        ])
    }).collect();

    f.render_widget(Paragraph::new(cmd_lines).block(cmd_block), feed_cols[0]);

    // Thoughts
    let thought_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(35, 35, 50)))
        .title(" thinking ")
        .title_style(Style::default().fg(Color::Rgb(200, 160, 255)).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(CARD_BG));

    let thoughts: Vec<Line> = vec![
        ("3s", "auth module needs refactoring..", Color::Rgb(180, 160, 220)),
        ("8s", "\u{2514}sub: running test suite..", Color::Rgb(160, 180, 200)),
        ("10s", "\u{2514}new: reading source file..", Color::Rgb(160, 180, 200)),
        ("15s", "checking config deps..", Color::Rgb(180, 160, 220)),
        ("25s", "\u{2514}sub: 2 failures found!", Color::Rgb(255, 140, 140)),
        ("35s", "\u{2514}team: coverage ok", Color::Rgb(160, 200, 160)),
        ("45s", "build passed!", Color::Rgb(140, 220, 140)),
        ("1m", "found 3 TODOs", Color::Rgb(180, 160, 220)),
    ].iter().map(|(time, text, color)| {
        Line::from(vec![
            Span::styled(format!(" {:>3} ", time), Style::default().fg(DIM)),
            Span::styled(*text, Style::default().fg(*color)),
        ])
    }).collect();

    f.render_widget(Paragraph::new(thoughts).block(thought_block), feed_cols[1]);

    // Status bar
    let status = Line::from(vec![
        Span::styled(" party:", Style::default().fg(DIM)),
        Span::styled("5 ", Style::default().fg(Color::White)),
        Span::styled("(\u{1f414}1 \u{1f423}1 \u{1f425}1 \u{1f95a}1 \u{2b50}1)", Style::default().fg(DIM)),
        Span::styled("  \u{2502} tokens:", Style::default().fg(DIM)),
        Span::styled("85.7k", Style::default().fg(Color::White)),
        Span::styled("  \u{2502} cost:", Style::default().fg(DIM)),
        Span::styled("$0.60", Style::default().fg(Color::Rgb(255, 220, 80))),
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
