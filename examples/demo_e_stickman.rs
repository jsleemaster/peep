//! Demo E: Tiny stickman characters with speech bubbles + RPG status
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
const BUBBLE_BG: Color = Color::Rgb(35, 35, 50);
const BUBBLE_BORDER: Color = Color::Rgb(55, 55, 75);

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
            // Double width for square aspect ratio
            spans.push(Span::styled(format!("{}{}", ch, ch), style));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

const SKIN: Color = Color::Rgb(230, 190, 150);
const HAIR_BROWN: Color = Color::Rgb(90, 60, 40);

/// Tiny stickman: 5 wide x 8 tall pixels
/// Body color = class color, head has skin + hair
fn stickman(body_color: Color, tool_color: Option<Color>, is_walking: bool, frame: usize) -> Vec<Vec<Pixel>> {
    let h = Some(HAIR_BROWN);
    let s = Some(SKIN);
    let b = Some(body_color);
    let n = n();
    let t = tool_color.map(Some).unwrap_or(n);

    if is_walking && frame % 2 == 1 {
        // Walking frame 2
        vec![
            vec![n, h, h, h, n],   // hair
            vec![n, s, s, s, n],   // face
            vec![n, n, b, n, n],   // neck
            vec![t, b, b, b, n],   // body + tool in left hand
            vec![n, n, b, n, n],   // waist
            vec![n, n, b, n, n],   // upper legs
            vec![n, b, n, b, n],   // legs apart
            vec![b, n, n, n, b],   // feet wide
        ]
    } else {
        // Standing / walking frame 1
        vec![
            vec![n, h, h, h, n],   // hair
            vec![n, s, s, s, n],   // face
            vec![n, n, b, n, n],   // neck
            vec![n, b, b, b, t],   // body + tool in right hand
            vec![n, n, b, n, n],   // waist
            vec![n, n, b, n, n],   // upper legs
            vec![n, b, n, b, n],   // legs
            vec![n, b, n, b, n],   // feet
        ]
    }
}

/// Sitting stickman (waiting)
fn stickman_sitting(body_color: Color) -> Vec<Vec<Pixel>> {
    let h = Some(HAIR_BROWN);
    let s = Some(SKIN);
    let b = Some(body_color);
    let n = n();
    let chair = Some(Color::Rgb(80, 60, 50));

    vec![
        vec![n, h, h, h, n],       // hair
        vec![n, s, s, s, n],       // face
        vec![n, n, b, n, n],       // neck
        vec![n, b, b, b, n],       // body
        vec![n, n, b, n, n],       // waist
        vec![n, b, b, b, n],       // legs bent (sitting)
        vec![n, n, chair, n, n],   // chair
        vec![n, chair, n, chair, n], // chair legs
    ]
}

/// Dead/done stickman (lying down trophy)
fn stickman_done(body_color: Color) -> Vec<Vec<Pixel>> {
    let h = Some(HAIR_BROWN);
    let s = Some(SKIN);
    let b = Some(body_color);
    let n = n();
    let star = Some(Color::Rgb(255, 220, 80));

    vec![
        vec![n, n, star, n, n],    // star/trophy above
        vec![n, star, star, star, n],
        vec![n, n, star, n, n],
        vec![n, h, h, h, n],       // hair
        vec![n, s, s, s, n],       // face (happy)
        vec![n, n, b, n, n],       // neck
        vec![n, b, b, b, n],       // body
        vec![n, n, b, b, n],       // relaxed legs
    ]
}

fn tool_pixel_color(class: &str) -> Option<Color> {
    match class {
        "explorer" => Some(Color::Rgb(100, 200, 255)),  // blue magnifier
        "craftsman" => Some(Color::Rgb(255, 200, 80)),   // yellow hammer
        "warrior" => Some(Color::Rgb(255, 100, 100)),    // red sword
        "commander" => Some(Color::Rgb(200, 100, 255)),  // purple flag
        _ => None,
    }
}

struct AgentCard {
    name: &'static str,
    bubble_text: &'static str,
    class: &'static str,
    class_icon: &'static str,
    body_color: Color,
    accent: Color,
    ctx_pct: u16,
    tokens: &'static str,
    cost: &'static str,
    location: &'static str,
    is_waiting: bool,
    is_done: bool,
    commands: Vec<(&'static str, &'static str, Color)>,   // (time, text, color)
    thoughts: Vec<(&'static str, &'static str, Color)>,
}

fn render_card(f: &mut Frame, area: Rect, agent: &AgentCard, tick: usize) {
    let border_color = if agent.is_done {
        Color::Rgb(50, 70, 50)
    } else if agent.is_waiting {
        Color::Rgb(50, 50, 65)
    } else {
        agent.accent
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(CARD_BG));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 16 || inner.height < 10 {
        return;
    }

    let mut y = inner.y;
    let cx = inner.x; // content x
    let cw = inner.width; // content width

    // --- Agent name ---
    let name_line = Line::from(vec![
        Span::styled(
            format!("{:^width$}", agent.name, width = cw as usize),
            Style::default().fg(agent.accent).add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(name_line).style(Style::default().bg(CARD_BG)), Rect::new(cx, y, cw, 1));
    y += 1;

    // --- Speech bubble ---
    let bubble_width = (cw - 4).min(agent.bubble_text.len() as u16 + 4);
    let bubble_x = cx + (cw.saturating_sub(bubble_width)) / 2;

    // Bubble top border
    let top_border = format!("\u{256d}{}\u{256e}", "\u{2500}".repeat((bubble_width - 2) as usize));
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(&top_border, Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)),
        Rect::new(bubble_x, y, bubble_width, 1),
    );
    y += 1;

    // Bubble content
    let text_width = (bubble_width - 4) as usize;
    let display_text: String = if agent.bubble_text.len() > text_width {
        format!("{}...", &agent.bubble_text[..text_width.saturating_sub(3)])
    } else {
        agent.bubble_text.to_string()
    };
    let content = format!("\u{2502} {:<width$} \u{2502}", display_text, width = text_width);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(&content, Style::default().fg(Color::Rgb(180, 180, 200))))).style(Style::default().bg(BUBBLE_BG)),
        Rect::new(bubble_x, y, bubble_width, 1),
    );
    y += 1;

    // Bubble bottom + pointer
    let btm_border = format!("\u{2570}{}\u{256f}", "\u{2500}".repeat((bubble_width - 2) as usize));
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(&btm_border, Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)),
        Rect::new(bubble_x, y, bubble_width, 1),
    );
    y += 1;

    // Pointer triangle
    let pointer_x = cx + cw / 2;
    f.render_widget(
        Paragraph::new(Line::from(Span::styled("\u{25bd}", Style::default().fg(BUBBLE_BORDER)))).style(Style::default().bg(CARD_BG)),
        Rect::new(pointer_x, y, 1, 1),
    );
    y += 1;

    // --- Character sprite (centered) ---
    let tool_color = tool_pixel_color(agent.class);
    let sprite = if agent.is_done {
        stickman_done(agent.body_color)
    } else if agent.is_waiting {
        stickman_sitting(agent.body_color)
    } else {
        stickman(agent.body_color, tool_color, true, tick / 5)
    };

    let sprite_lines = sprite_to_lines(&sprite);
    let sprite_px_width = 10u16; // 5 pixels * 2 chars
    let sprite_x = cx + (cw.saturating_sub(sprite_px_width)) / 2;

    for (j, line) in sprite_lines.iter().enumerate() {
        let sy = y + j as u16;
        if sy < inner.y + inner.height {
            f.render_widget(
                Paragraph::new(line.clone()).style(Style::default().bg(CARD_BG)),
                Rect::new(sprite_x, sy, sprite_px_width, 1),
            );
        }
    }

    // Zzz effect for waiting
    if agent.is_waiting {
        let zzz_frame = (tick / 10) % 4;
        let zzz = ["z", " zz", "  zzz", " zz"][zzz_frame];
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(zzz, Style::default().fg(Color::Rgb(120, 120, 170))))).style(Style::default().bg(CARD_BG)),
            Rect::new(sprite_x + sprite_px_width, y, 5, 1),
        );
    }

    y += sprite_lines.len() as u16;

    // --- Class + Location ---
    if y < inner.y + inner.height {
        let class_line = Line::from(vec![
            Span::styled(
                format!("{:^width$}", format!("{} {} \u{00b7} {}", agent.class_icon, agent.class, agent.location), width = cw as usize),
                Style::default().fg(DIM),
            ),
        ]);
        f.render_widget(Paragraph::new(class_line).style(Style::default().bg(CARD_BG)), Rect::new(cx, y, cw, 1));
        y += 1;
    }

    // --- HP bar (context %) ---
    if y < inner.y + inner.height && !agent.is_done {
        let filled = (agent.ctx_pct / 10) as usize;
        let empty = 10usize.saturating_sub(filled);
        let hp_color = if agent.ctx_pct > 80 { Color::Rgb(255, 80, 80) } else if agent.ctx_pct > 60 { Color::Rgb(255, 200, 80) } else { Color::Rgb(80, 200, 120) };
        let hp_label = if agent.ctx_pct > 80 { "DANGER" } else if agent.ctx_pct > 60 { "WARN" } else { "OK" };

        let hp_line = Line::from(vec![
            Span::styled("  HP ", Style::default().fg(DIM)),
            Span::styled("\u{2588}".repeat(filled), Style::default().fg(hp_color)),
            Span::styled("\u{2591}".repeat(empty), Style::default().fg(Color::Rgb(40, 40, 55))),
            Span::styled(format!(" {}% {}", agent.ctx_pct, hp_label), Style::default().fg(hp_color)),
        ]);
        f.render_widget(Paragraph::new(hp_line).style(Style::default().bg(CARD_BG)), Rect::new(cx, y, cw, 1));
        y += 1;
    }

    // --- Tokens + Cost ---
    if y < inner.y + inner.height {
        let stat_line = Line::from(vec![
            Span::styled("  EXP ", Style::default().fg(DIM)),
            Span::styled(agent.tokens, Style::default().fg(Color::Rgb(180, 180, 220))),
            Span::styled("  GOLD ", Style::default().fg(DIM)),
            Span::styled(agent.cost, Style::default().fg(Color::Rgb(255, 220, 80))),
        ]);
        f.render_widget(Paragraph::new(stat_line).style(Style::default().bg(CARD_BG)), Rect::new(cx, y, cw, 1));
        y += 1;
    }

    // --- Separator ---
    if y < inner.y + inner.height {
        let sep = Line::from(Span::styled(
            format!("{:\u{2500}<width$}", "", width = cw as usize),
            Style::default().fg(Color::Rgb(35, 35, 50)),
        ));
        f.render_widget(Paragraph::new(sep).style(Style::default().bg(CARD_BG)), Rect::new(cx, y, cw, 1));
        y += 1;
    }

    // --- Split feed: commands | thoughts ---
    let feed_h = (inner.y + inner.height).saturating_sub(y);
    if feed_h < 2 {
        return;
    }

    let feed_area = Rect::new(cx, y, cw, feed_h);
    let feed_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(feed_area);

    // Commands
    let mut cmd_lines = vec![
        Line::from(Span::styled(" commands", Style::default().fg(Color::Rgb(100, 180, 255)).add_modifier(Modifier::BOLD))),
    ];
    for (time, text, color) in &agent.commands {
        cmd_lines.push(Line::from(vec![
            Span::styled(format!(" {:>3} ", time), Style::default().fg(DIM)),
            Span::styled(*text, Style::default().fg(*color)),
        ]));
    }
    f.render_widget(Paragraph::new(cmd_lines).style(Style::default().bg(CARD_BG)), feed_cols[0]);

    // Thoughts
    let mut thought_lines = vec![
        Line::from(Span::styled(" thinking", Style::default().fg(Color::Rgb(200, 160, 255)).add_modifier(Modifier::BOLD))),
    ];
    for (time, text, color) in &agent.thoughts {
        thought_lines.push(Line::from(vec![
            Span::styled(format!(" {:>3} ", time), Style::default().fg(DIM)),
            Span::styled(*text, Style::default().fg(*color)),
        ]));
    }
    f.render_widget(Paragraph::new(thought_lines).style(Style::default().bg(CARD_BG)), feed_cols[1]);
}

fn render(f: &mut Frame, tick: usize) {
    f.render_widget(Paragraph::new("").style(Style::default().bg(BG)), f.area());

    let agents = vec![
        AgentCard {
            name: "main-worker",
            bubble_text: "auth module needs refactoring for new JWT flow",
            class: "explorer", class_icon: "\u{1f50d}",
            body_color: Color::Rgb(60, 120, 220),
            accent: Color::Rgb(100, 160, 255),
            ctx_pct: 67, tokens: "45.2k", cost: "$0.32",
            location: "src/auth.ts",
            is_waiting: false, is_done: false,
            commands: vec![
                ("2s", "Edit auth.ts", Color::Yellow),
                ("12s", "Read config.ts", Color::Cyan),
                ("30s", "Bash cargo build", Color::Red),
                ("1m", "Grep \"TODO\"", Color::Cyan),
                ("2m", "Task Fix login", Color::Magenta),
            ],
            thoughts: vec![
                ("3s", "auth module needs..", Color::Rgb(180, 160, 220)),
                ("15s", "checking deps..", Color::Rgb(180, 160, 220)),
                ("45s", "build passed!", Color::Rgb(140, 200, 140)),
                ("1m", "found 3 TODOs", Color::Rgb(180, 160, 220)),
            ],
        },
        AgentCard {
            name: "team-review",
            bubble_text: "waiting for main-worker to finish auth changes...",
            class: "craftsman", class_icon: "\u{1f528}",
            body_color: Color::Rgb(80, 180, 160),
            accent: Color::Rgb(80, 200, 200),
            ctx_pct: 22, tokens: "12.1k", cost: "$0.08",
            location: "test/auth.test.ts",
            is_waiting: true, is_done: false,
            commands: vec![
                ("1m", "Write test.ts", Color::Yellow),
                ("3m", "Read README", Color::Cyan),
            ],
            thoughts: vec![
                ("1m", "coverage looks ok", Color::Rgb(180, 160, 220)),
                ("2m", "waiting for main..", Color::Rgb(200, 200, 80)),
            ],
        },
        AgentCard {
            name: "sub-scout",
            bubble_text: "running full test suite, 2 failures detected",
            class: "warrior", class_icon: "\u{2694}\u{fe0f}",
            body_color: Color::Rgb(220, 100, 160),
            accent: Color::Rgb(255, 140, 200),
            ctx_pct: 45, tokens: "8.4k", cost: "$0.06",
            location: "test/",
            is_waiting: false, is_done: false,
            commands: vec![
                ("5s", "Bash npm test", Color::Red),
                ("20s", "Grep error", Color::Cyan),
                ("1m", "Edit index.ts", Color::Yellow),
            ],
            thoughts: vec![
                ("6s", "running tests..", Color::Rgb(180, 160, 220)),
                ("25s", "2 failures!", Color::Rgb(255, 140, 140)),
            ],
        },
        AgentCard {
            name: "agent-deploy",
            bubble_text: "deployment complete! all checks passed",
            class: "commander", class_icon: "\u{1f3f4}",
            body_color: Color::Rgb(80, 200, 120),
            accent: Color::Rgb(80, 220, 120),
            ctx_pct: 0, tokens: "20.0k", cost: "$0.14",
            location: "deploy/prod",
            is_waiting: false, is_done: true,
            commands: vec![
                ("5m", "Bash deploy.sh", Color::Red),
                ("6m", "Read status", Color::Cyan),
            ],
            thoughts: vec![
                ("5m", "deploying...", Color::Rgb(180, 160, 220)),
                ("6m", "all green!", Color::Rgb(140, 220, 140)),
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

    // Title
    let title = Line::from(vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled("agent monitor", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("  q:quit  j/k:select  Tab:switch", Style::default().fg(Color::Rgb(50, 50, 70))),
    ]);
    f.render_widget(Paragraph::new(title), outer[0]);

    // Cards
    let card_constraints: Vec<Constraint> = agents.iter().map(|_| Constraint::Ratio(1, agents.len() as u32)).collect();
    let card_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(card_constraints)
        .split(outer[1]);

    for (i, agent) in agents.iter().enumerate() {
        render_card(f, card_areas[i], agent, tick);
    }

    // Status bar
    let status = Line::from(vec![
        Span::styled(" agents:", Style::default().fg(DIM)),
        Span::styled("4", Style::default().fg(Color::White)),
        Span::styled("  \u{2502} tokens:", Style::default().fg(DIM)),
        Span::styled("85.7k", Style::default().fg(Color::White)),
        Span::styled("  \u{2502} cost:", Style::default().fg(DIM)),
        Span::styled("$0.60", Style::default().fg(Color::Rgb(255, 220, 80))),
        Span::styled("  \u{2502} ", Style::default().fg(DIM)),
        Span::styled("\u{25cf}main ", Style::default().fg(Color::Rgb(100, 160, 255))),
        Span::styled("\u{25cf}team ", Style::default().fg(Color::Rgb(80, 200, 200))),
        Span::styled("\u{25cf}sub ", Style::default().fg(Color::Rgb(255, 140, 200))),
        Span::styled("\u{25cf}deploy", Style::default().fg(Color::Rgb(80, 220, 120))),
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
