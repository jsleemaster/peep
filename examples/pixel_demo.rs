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
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame, Terminal,
};

/// A pixel is an optional RGBA color (None = transparent)
type Pixel = Option<Color>;

/// A sprite frame: rows of pixels
struct Sprite {
    width: usize,
    height: usize,
    pixels: Vec<Vec<Pixel>>,
}

/// Convert a sprite to terminal lines using half-block rendering.
/// Each terminal row encodes 2 pixel rows (top via fg, bottom via bg).
fn sprite_to_lines(sprite: &Sprite) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Process two rows at a time
    let mut y = 0;
    while y < sprite.height {
        let mut spans = Vec::new();
        for x in 0..sprite.width {
            let top = sprite.pixels[y][x];
            let btm = if y + 1 < sprite.height {
                sprite.pixels[y + 1][x]
            } else {
                None
            };

            match (top, btm) {
                (Some(tc), Some(bc)) => {
                    // Both pixels filled: ▀ with fg=top, bg=bottom
                    spans.push(Span::styled(
                        "▀",
                        Style::default().fg(tc).bg(bc),
                    ));
                }
                (Some(tc), None) => {
                    // Only top pixel: ▀ with fg=top
                    spans.push(Span::styled(
                        "▀",
                        Style::default().fg(tc),
                    ));
                }
                (None, Some(bc)) => {
                    // Only bottom pixel: ▄ with fg=bottom
                    spans.push(Span::styled(
                        "▄",
                        Style::default().fg(bc),
                    ));
                }
                (None, None) => {
                    // Both transparent
                    spans.push(Span::raw(" "));
                }
            }
        }
        lines.push(Line::from(spans));
        y += 2;
    }

    lines
}

// Color palette
const YELLOW: Color = Color::Rgb(255, 220, 50);
const DARK_YELLOW: Color = Color::Rgb(200, 170, 30);
const ORANGE: Color = Color::Rgb(255, 160, 30);
const EYE_WHITE: Color = Color::Rgb(240, 240, 255);
const EYE_PUPIL: Color = Color::Rgb(20, 20, 60);
const MOUTH_RED: Color = Color::Rgb(200, 50, 50);
const CYAN: Color = Color::Rgb(80, 200, 255);
const DARK_CYAN: Color = Color::Rgb(50, 150, 200);
const PINK: Color = Color::Rgb(255, 140, 200);
const DARK_PINK: Color = Color::Rgb(200, 100, 160);
const SHADOW: Color = Color::Rgb(40, 40, 50);
const LIGHTNING: Color = Color::Rgb(255, 255, 100);
const LIGHTNING2: Color = Color::Rgb(255, 200, 50);
const ZZZ_COLOR: Color = Color::Rgb(150, 150, 200);
const GREEN: Color = Color::Rgb(80, 220, 120);
const DARK_GREEN: Color = Color::Rgb(50, 170, 80);
const CHECK_COLOR: Color = Color::Rgb(100, 255, 150);

fn n() -> Pixel { None } // transparent

/// Main agent - mouth open (active eating frame)
fn packman_main_open() -> Sprite {
    let y = Some(YELLOW);
    let d = Some(DARK_YELLOW);
    let o = Some(ORANGE);
    let w = Some(EYE_WHITE);
    let p = Some(EYE_PUPIL);
    let s = Some(SHADOW);
    let n = n();

    let pixels = vec![
        //0  1  2  3  4  5  6  7  8  9  10 11 12 13
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, d, d, y, y, y, y, y, y, d, d, n, n],
        vec![n, d, y, y, y, y, y, y, y, y, y, y, d, n],
        vec![d, y, y, y, w, w, y, y, w, w, y, y, y, d],
        vec![d, y, y, y, w, p, y, y, w, p, y, y, y, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, n, n],
        vec![d, y, y, y, y, y, y, y, y, y, n, n, n, n],
        vec![d, o, y, y, y, y, y, y, n, n, n, n, n, n],
        vec![d, o, y, y, y, y, y, y, n, n, n, n, n, n],
        vec![d, y, y, y, y, y, y, y, y, y, n, n, n, n],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, n, n],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![n, d, y, y, y, y, y, y, y, y, y, y, d, n],
        vec![n, n, d, d, y, y, y, y, y, y, d, d, n, n],
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, n, n, n, s, s, s, s, n, n, n, n, n],
    ];
    Sprite { width: 14, height: 16, pixels }
}

/// Main agent - mouth closed
fn packman_main_closed() -> Sprite {
    let y = Some(YELLOW);
    let d = Some(DARK_YELLOW);
    let o = Some(ORANGE);
    let w = Some(EYE_WHITE);
    let p = Some(EYE_PUPIL);
    let s = Some(SHADOW);
    let n = n();

    let pixels = vec![
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, d, d, y, y, y, y, y, y, d, d, n, n],
        vec![n, d, y, y, y, y, y, y, y, y, y, y, d, n],
        vec![d, y, y, y, w, w, y, y, w, w, y, y, y, d],
        vec![d, y, y, y, w, p, y, y, w, p, y, y, y, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![d, o, o, o, o, o, o, o, o, o, o, o, o, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![d, y, y, y, y, y, y, y, y, y, y, y, y, d],
        vec![n, d, y, y, y, y, y, y, y, y, y, y, d, n],
        vec![n, n, d, d, y, y, y, y, y, y, d, d, n, n],
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, n, n, n, s, s, s, s, n, n, n, n, n],
    ];
    Sprite { width: 14, height: 16, pixels }
}

/// Team agent (cyan ghost-like)
fn packman_team_open() -> Sprite {
    let c = Some(CYAN);
    let d = Some(DARK_CYAN);
    let w = Some(EYE_WHITE);
    let p = Some(EYE_PUPIL);
    let s = Some(SHADOW);
    let n = n();

    let pixels = vec![
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, d, d, c, c, c, c, c, c, d, d, n, n],
        vec![n, d, c, c, c, c, c, c, c, c, c, c, d, n],
        vec![d, c, c, c, w, w, c, c, w, w, c, c, c, d],
        vec![d, c, c, c, w, p, c, c, w, p, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, c, c, c, c, c, c, c, c, c, c, c, d],
        vec![d, c, d, c, c, d, c, c, d, c, c, d, c, d],
        vec![d, n, d, c, d, n, d, c, d, n, d, c, d, n],
        vec![n, n, n, d, n, n, n, d, n, n, n, d, n, n],
        vec![n, n, n, n, n, s, s, s, s, n, n, n, n, n],
    ];
    Sprite { width: 14, height: 16, pixels }
}

/// Sub agent (pink, smaller feel)
fn packman_sub() -> Sprite {
    let k = Some(PINK);
    let d = Some(DARK_PINK);
    let w = Some(EYE_WHITE);
    let p = Some(EYE_PUPIL);
    let s = Some(SHADOW);
    let n = n();

    let pixels = vec![
        vec![n, n, n, n, n, n, n, n, n, n, n, n, n, n],
        vec![n, n, n, n, n, n, n, n, n, n, n, n, n, n],
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, n, d, k, k, k, k, k, k, d, n, n, n],
        vec![n, n, d, k, w, w, k, k, w, w, k, d, n, n],
        vec![n, n, d, k, w, p, k, k, w, p, k, d, n, n],
        vec![n, n, d, k, k, k, k, k, k, k, k, d, n, n],
        vec![n, n, d, k, k, k, k, k, k, k, k, d, n, n],
        vec![n, n, d, k, k, k, k, k, k, k, k, d, n, n],
        vec![n, n, d, k, k, k, k, k, k, k, k, d, n, n],
        vec![n, n, d, k, k, k, k, k, k, k, k, d, n, n],
        vec![n, n, d, k, d, k, k, d, k, k, d, k, n, n],
        vec![n, n, d, n, d, k, d, n, d, k, d, n, n, n],
        vec![n, n, n, n, n, d, n, n, n, d, n, n, n, n],
        vec![n, n, n, n, n, n, n, n, n, n, n, n, n, n],
        vec![n, n, n, n, n, s, s, s, s, n, n, n, n, n],
    ];
    Sprite { width: 14, height: 16, pixels }
}

/// Completed agent (green, checkmark eyes)
fn packman_done() -> Sprite {
    let g = Some(GREEN);
    let d = Some(DARK_GREEN);
    let c = Some(CHECK_COLOR);
    let s = Some(SHADOW);
    let n = n();

    let pixels = vec![
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, d, d, g, g, g, g, g, g, d, d, n, n],
        vec![n, d, g, g, g, g, g, g, g, g, g, g, d, n],
        vec![d, g, g, g, g, g, g, g, g, g, g, g, g, d],
        vec![d, g, g, g, g, c, g, g, g, c, g, g, g, d],
        vec![d, g, g, g, c, g, g, g, c, g, g, g, g, d],
        vec![d, g, g, c, g, g, g, c, g, g, g, g, g, d],
        vec![d, g, g, g, c, g, g, g, c, g, g, g, g, d],
        vec![d, g, g, g, g, c, g, g, g, c, g, g, g, d],
        vec![d, g, g, g, g, g, g, g, g, g, g, g, g, d],
        vec![d, g, g, g, g, g, g, g, g, g, g, g, g, d],
        vec![d, g, g, g, g, g, g, g, g, g, g, g, g, d],
        vec![n, d, g, g, g, g, g, g, g, g, g, g, d, n],
        vec![n, n, d, d, g, g, g, g, g, g, d, d, n, n],
        vec![n, n, n, n, d, d, d, d, d, d, n, n, n, n],
        vec![n, n, n, n, n, s, s, s, s, n, n, n, n, n],
    ];
    Sprite { width: 14, height: 16, pixels }
}

/// Lightning effect particles
fn lightning_effect(tick: usize) -> Vec<Line<'static>> {
    let l1 = Some(LIGHTNING);
    let l2 = Some(LIGHTNING2);
    let n = n();

    let frames: Vec<Vec<Vec<Pixel>>> = vec![
        vec![
            vec![n, n, l1, n, n],
            vec![n, l1, l2, n, n],
            vec![n, n, l1, l1, n],
            vec![n, n, l2, n, n],
        ],
        vec![
            vec![n, n, n, l1, n],
            vec![n, n, l2, l1, n],
            vec![n, l1, l2, n, n],
            vec![n, l1, n, n, n],
        ],
        vec![
            vec![n, l2, n, n, n],
            vec![n, l1, l2, n, n],
            vec![n, n, l1, l2, n],
            vec![n, n, n, l1, n],
        ],
    ];

    let frame = &frames[tick % frames.len()];
    let sprite = Sprite {
        width: 5,
        height: frame.len(),
        pixels: frame.clone(),
    };
    sprite_to_lines(&sprite)
}

/// Zzz sleep particles
fn sleep_effect(tick: usize) -> Vec<Line<'static>> {
    let z = Some(ZZZ_COLOR);
    let n = n();

    let frames: Vec<Vec<Vec<Pixel>>> = vec![
        vec![
            vec![n, n, n, n, n],
            vec![n, n, n, z, n],
            vec![n, n, n, n, n],
            vec![n, z, n, n, n],
        ],
        vec![
            vec![n, n, n, z, n],
            vec![n, n, n, n, n],
            vec![n, z, n, n, n],
            vec![n, n, n, n, n],
        ],
        vec![
            vec![n, n, z, n, n],
            vec![n, n, n, n, z],
            vec![n, n, n, n, n],
            vec![z, n, n, n, n],
        ],
    ];

    let frame = &frames[tick % frames.len()];
    let sprite = Sprite {
        width: 5,
        height: frame.len(),
        pixels: frame.clone(),
    };
    sprite_to_lines(&sprite)
}

fn render(f: &mut Frame, tick: usize) {
    let area = f.area();

    // Background
    let bg = Paragraph::new("").style(Style::default().bg(Color::Rgb(15, 15, 25)));
    f.render_widget(bg, area);

    // Title
    let title = Line::from(vec![
        Span::styled("  packmen ", Style::default().fg(Color::Rgb(255, 220, 50))),
        Span::styled("pixel art demo", Style::default().fg(Color::Rgb(100, 100, 120))),
        Span::styled("  (q to quit)", Style::default().fg(Color::Rgb(60, 60, 80))),
    ]);
    f.render_widget(Paragraph::new(title), Rect::new(0, 0, area.width, 1));

    // Determine animation frame
    let anim_frame = tick / 6; // slower animation

    // --- Main Agent (Yellow Packman) ---
    let main_sprite = if anim_frame % 2 == 0 {
        packman_main_open()
    } else {
        packman_main_closed()
    };
    let main_lines = sprite_to_lines(&main_sprite);
    let main_label = Line::from(vec![
        Span::styled("  main-worker", Style::default().fg(YELLOW)),
        Span::styled(" active", Style::default().fg(Color::Rgb(100, 255, 100))),
    ]);

    let x_main = 3;
    let y_main = 3;
    render_lines(f, &main_lines, x_main, y_main);
    f.render_widget(Paragraph::new(main_label), Rect::new(x_main, y_main + main_lines.len() as u16, 20, 1));

    // Lightning effect for main
    let lightning = lightning_effect(tick / 4);
    render_lines(f, &lightning, x_main + 15, y_main + 1);

    // --- Team Agent (Cyan Ghost) ---
    let team_lines = sprite_to_lines(&packman_team_open());
    let team_label = Line::from(vec![
        Span::styled("  team-review", Style::default().fg(CYAN)),
        Span::styled(" waiting", Style::default().fg(Color::Rgb(255, 220, 50))),
    ]);

    let x_team = 24;
    let y_team = 3;
    render_lines(f, &team_lines, x_team, y_team);
    f.render_widget(Paragraph::new(team_label), Rect::new(x_team, y_team + team_lines.len() as u16, 20, 1));

    // Zzz effect for team
    let zzz = sleep_effect(tick / 5);
    render_lines(f, &zzz, x_team + 15, y_team);

    // --- Sub Agent (Pink, smaller) ---
    let sub_lines = sprite_to_lines(&packman_sub());
    let sub_label = Line::from(vec![
        Span::styled("  sub-scout", Style::default().fg(PINK)),
        Span::styled(" active", Style::default().fg(Color::Rgb(100, 255, 100))),
    ]);

    let x_sub = 45;
    let y_sub = 3;
    render_lines(f, &sub_lines, x_sub, y_sub);
    f.render_widget(Paragraph::new(sub_label), Rect::new(x_sub, y_sub + sub_lines.len() as u16, 20, 1));

    // --- Done Agent (Green) ---
    let done_lines = sprite_to_lines(&packman_done());
    let done_label = Line::from(vec![
        Span::styled("  agent-done", Style::default().fg(GREEN)),
        Span::styled(" completed", Style::default().fg(Color::Rgb(100, 100, 120))),
    ]);

    let x_done = 64;
    let y_done = 3;
    render_lines(f, &done_lines, x_done, y_done);
    f.render_widget(Paragraph::new(done_label), Rect::new(x_done, y_done + done_lines.len() as u16, 20, 1));

    // Info text
    let info = vec![
        Line::from(Span::styled(
            "  Half-block rendering: each terminal cell = 2 pixels (▀ with fg/bg colors)",
            Style::default().fg(Color::Rgb(120, 120, 150)),
        )),
        Line::from(Span::styled(
            "  14x16 pixel sprite → 14x8 terminal cells, full 24-bit RGB color",
            Style::default().fg(Color::Rgb(120, 120, 150)),
        )),
        Line::from(Span::styled(
            "  Animated: mouth open/close, lightning ⚡, sleep 💤 particles",
            Style::default().fg(Color::Rgb(120, 120, 150)),
        )),
    ];
    let info_y = y_main + 12;
    for (i, line) in info.into_iter().enumerate() {
        f.render_widget(
            Paragraph::new(line),
            Rect::new(0, info_y + i as u16, area.width, 1),
        );
    }
}

fn render_lines(f: &mut Frame, lines: &[Line<'static>], x: u16, y: u16) {
    for (i, line) in lines.iter().enumerate() {
        let row = y + i as u16;
        if row < f.area().height {
            f.render_widget(
                Paragraph::new(line.clone()),
                Rect::new(x, row, 20, 1),
            );
        }
    }
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
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
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
