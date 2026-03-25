//! Demo B: Pixel characters + dashboard hybrid
//! Top area: animated pixel characters in a row
//! Bottom area: metrics and feed-like info
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

fn sprite_to_lines(pixels: &[Vec<Pixel>], scale: bool) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut y = 0;
    while y < pixels.len() {
        let mut spans = Vec::new();
        for x in 0..pixels[y].len() {
            let top = pixels[y][x];
            let btm = if y + 1 < pixels.len() { pixels[y + 1][x] } else { None };
            let (ch, style) = match (top, btm) {
                (Some(tc), Some(bc)) => ("\u{2580}", Style::default().fg(tc).bg(bc)),
                (Some(tc), None) => ("\u{2580}", Style::default().fg(tc).bg(Color::Rgb(18, 18, 28))),
                (None, Some(bc)) => ("\u{2584}", Style::default().fg(bc).bg(Color::Rgb(18, 18, 28))),
                (None, None) => (" ", Style::default().bg(Color::Rgb(18, 18, 28))),
            };
            if scale {
                spans.push(Span::styled(format!("{}{}", ch, ch), style));
            } else {
                spans.push(Span::styled(ch.to_string(), style));
            }
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

const YELLOW: Color = Color::Rgb(255, 220, 50);
const DARK_YELLOW: Color = Color::Rgb(200, 170, 30);
const CYAN: Color = Color::Rgb(80, 200, 255);
const DARK_CYAN: Color = Color::Rgb(50, 150, 200);
const PINK: Color = Color::Rgb(255, 140, 200);
const DARK_PINK: Color = Color::Rgb(200, 100, 160);
const ORANGE: Color = Color::Rgb(255, 160, 50);
const DARK_ORANGE: Color = Color::Rgb(200, 120, 30);
const EW: Color = Color::Rgb(240, 240, 255);
const EP: Color = Color::Rgb(20, 20, 60);
const GREEN: Color = Color::Rgb(80, 220, 120);
const DARK_GREEN: Color = Color::Rgb(50, 170, 80);
const ZZZ: Color = Color::Rgb(130, 130, 180);
const BOLT: Color = Color::Rgb(255, 255, 100);

fn packman_open(body: Color, dark: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let d = Some(dark);
    let w = Some(EW); let p = Some(EP); let n = n();
    vec![
        vec![n, b, b, b, n],
        vec![b, w, p, b, n],
        vec![b, b, b, n, n],
        vec![b, b, b, n, n],
        vec![b, b, b, b, n],
        vec![n, b, b, b, n],
    ]
}

fn packman_closed(body: Color, dark: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let d = Some(dark);
    let w = Some(EW); let p = Some(EP); let n = n();
    vec![
        vec![n, b, b, b, n],
        vec![b, w, p, b, b],
        vec![b, b, b, b, b],
        vec![b, d, d, d, b],
        vec![b, b, b, b, b],
        vec![n, b, b, b, n],
    ]
}

fn ghost(body: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body);
    let w = Some(EW); let p = Some(EP); let n = n();
    vec![
        vec![n, b, b, b, n],
        vec![b, w, p, w, p],
        vec![b, b, b, b, b],
        vec![b, b, b, b, b],
        vec![b, b, b, b, b],
        vec![b, n, b, n, b],
    ]
}

fn done_sprite(body: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let c = Some(Color::Rgb(200, 255, 200)); let n = n();
    vec![
        vec![n, b, b, b, n],
        vec![b, b, b, c, b],
        vec![b, b, c, b, b],
        vec![c, b, b, b, b],
        vec![b, c, b, b, b],
        vec![n, b, b, b, n],
    ]
}

struct AgentVis {
    name: &'static str,
    state: &'static str,
    tool: &'static str,
    body: Color,
    dark: Color,
    is_waiting: bool,
    is_done: bool,
}

fn render(f: &mut Frame, tick: usize) {
    let bg = Style::default().bg(Color::Rgb(18, 18, 28));
    f.render_widget(Paragraph::new("").style(bg), f.area());

    let agents = vec![
        AgentVis { name: "main-worker", state: "active", tool: "Edit src/auth.ts", body: YELLOW, dark: DARK_YELLOW, is_waiting: false, is_done: false },
        AgentVis { name: "team-review", state: "waiting", tool: "", body: CYAN, dark: DARK_CYAN, is_waiting: true, is_done: false },
        AgentVis { name: "sub-scout", state: "active", tool: "Bash npm test", body: PINK, dark: DARK_PINK, is_waiting: false, is_done: false },
        AgentVis { name: "agent-build", state: "done", tool: "", body: GREEN, dark: DARK_GREEN, is_waiting: false, is_done: true },
    ];

    let area = f.area();

    // Layout: title(1) + characters area + metrics area + status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // title
            Constraint::Length(12),  // characters
            Constraint::Fill(1),    // metrics/feed
            Constraint::Length(1),   // status
        ])
        .split(area);

    // Title
    let title = Line::from(vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled("agent dashboard", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("  (q to quit)", Style::default().fg(Color::Rgb(50, 50, 70))),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Characters area - render each agent as pixel character with info below
    let char_area = chunks[1];
    let agent_width = char_area.width / agents.len() as u16;

    for (i, agent) in agents.iter().enumerate() {
        let x_offset = char_area.x + (i as u16 * agent_width) + (agent_width / 2).saturating_sub(6);

        // Get sprite
        let sprite = if agent.is_done {
            done_sprite(agent.body)
        } else if agent.is_waiting {
            ghost(agent.body)
        } else if (tick / 6) % 2 == 0 {
            packman_open(agent.body, agent.dark)
        } else {
            packman_closed(agent.body, agent.dark)
        };

        let lines = sprite_to_lines(&sprite, true);

        // Render sprite
        for (j, line) in lines.iter().enumerate() {
            let y = char_area.y + 1 + j as u16;
            if y < char_area.y + char_area.height {
                f.render_widget(Paragraph::new(line.clone()), Rect::new(x_offset, y, 12, 1));
            }
        }

        // Name label below character
        let name_y = char_area.y + 5;
        let name_line = Line::from(vec![
            Span::styled(
                format!("{:^12}", agent.name),
                Style::default().fg(agent.body),
            ),
        ]);
        if name_y < char_area.y + char_area.height {
            f.render_widget(Paragraph::new(name_line), Rect::new(x_offset.saturating_sub(1), name_y, 14, 1));
        }

        // State + tool below name
        let state_y = name_y + 1;
        let state_color = if agent.is_done { Color::Rgb(80, 80, 100) } else if agent.is_waiting { Color::Rgb(200, 200, 80) } else { Color::Rgb(80, 220, 120) };
        let state_text = if !agent.tool.is_empty() {
            format!("{}", agent.tool)
        } else {
            agent.state.to_string()
        };
        let state_line = Line::from(Span::styled(
            format!("{:^14}", state_text),
            Style::default().fg(state_color),
        ));
        if state_y < char_area.y + char_area.height {
            f.render_widget(Paragraph::new(state_line), Rect::new(x_offset.saturating_sub(2), state_y, 16, 1));
        }

        // Effects
        if agent.is_waiting {
            let zzz_frame = (tick / 8) % 3;
            let zzz_text = match zzz_frame {
                0 => "z",
                1 => "zz",
                _ => "zzz",
            };
            let zzz_line = Line::from(Span::styled(zzz_text, Style::default().fg(ZZZ)));
            f.render_widget(Paragraph::new(zzz_line), Rect::new(x_offset + 10, char_area.y + 1, 4, 1));
        } else if !agent.is_done && !agent.tool.is_empty() {
            let bolt_visible = (tick / 3) % 3 != 0;
            if bolt_visible {
                let bolt_line = Line::from(Span::styled("*", Style::default().fg(BOLT)));
                f.render_widget(Paragraph::new(bolt_line), Rect::new(x_offset + 10, char_area.y + 2, 2, 1));
            }
        }

        // Context gauge under character
        let gauge_y = state_y + 1;
        if gauge_y < char_area.y + char_area.height && !agent.is_done {
            let ctx_pct = match i { 0 => 67, 1 => 22, 2 => 45, _ => 0 };
            let filled = ctx_pct / 10;
            let gauge_color = if ctx_pct > 80 { Color::Red } else if ctx_pct > 60 { Color::Yellow } else { Color::Green };
            let gauge = format!("{}{} {}%", "\u{2588}".repeat(filled), "\u{2591}".repeat(10 - filled), ctx_pct);
            let gauge_line = Line::from(Span::styled(
                format!("{:^14}", gauge),
                Style::default().fg(gauge_color),
            ));
            f.render_widget(Paragraph::new(gauge_line), Rect::new(x_offset.saturating_sub(2), gauge_y, 16, 1));
        }
    }

    // Metrics/feed area
    let feed_area = chunks[2];
    let feed_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 60)))
        .title(" Live Feed ")
        .title_style(Style::default().fg(Color::Rgb(100, 100, 130)));

    let feed_events = vec![
        ("2s",  "main",  "Edit",  "src/auth.ts",       Color::Yellow),
        ("5s",  "sub",   "Bash",  "npm test",           Color::Red),
        ("12s", "main",  "Read",  "src/config.ts",      Color::Cyan),
        ("30s", "sub",   "Grep",  "\"TODO\" src/",      Color::Cyan),
        ("1m",  "team",  "Write", "test/auth.test.ts",  Color::Yellow),
        ("2m",  "main",  "Task",  "Fix login bug",      Color::Magenta),
        ("3m",  "main",  "Read",  "package.json",       Color::Cyan),
        ("4m",  "sub",   "Edit",  "src/index.ts",       Color::Yellow),
    ];

    let mut feed_lines = Vec::new();
    for (time, agent, tool, target, tool_color) in &feed_events {
        feed_lines.push(Line::from(vec![
            Span::styled(format!(" {:>4} ", time), Style::default().fg(Color::Rgb(80, 80, 100))),
            Span::styled(format!("{:<6}", agent), Style::default().fg(Color::Rgb(100, 160, 255))),
            Span::styled(format!("{:<6}", tool), Style::default().fg(*tool_color)),
            Span::styled(*target, Style::default().fg(Color::Rgb(150, 150, 170))),
        ]));
    }

    let feed = Paragraph::new(feed_lines).block(feed_block);
    f.render_widget(feed, feed_area);

    // Status bar
    let status = Line::from(vec![
        Span::styled(" agents:", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("4", Style::default().fg(Color::White)),
        Span::styled(" (2 active, 1 wait, 1 done)", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("  tokens:", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("65.7k", Style::default().fg(Color::White)),
        Span::styled("  cost:", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("$0.46", Style::default().fg(Color::Rgb(80, 220, 120))),
        Span::styled("  ", Style::default()),
        Span::styled("q:quit", Style::default().fg(Color::Rgb(60, 60, 80))),
    ]);
    f.render_widget(Paragraph::new(status), chunks[3]);
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
