//! Demo C: Minimal Stage — characters stand still, effects show state
//! No movement, no maze. Clean status display with pixel art personalities.
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
            // Double width for square pixels
            spans.push(Span::styled(format!("{}{}", ch, ch), style));
        }
        lines.push(Line::from(spans));
        y += 2;
    }
    lines
}

const EW: Color = Color::Rgb(240, 240, 255);
const EP: Color = Color::Rgb(20, 20, 60);

fn packman_open(body: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let w = Some(EW); let p = Some(EP); let n = n();
    vec![
        vec![n, n, b, b, b, b, n, n],
        vec![n, b, b, b, b, b, b, n],
        vec![b, b, w, p, b, b, n, n],
        vec![b, b, b, b, b, n, n, n],
        vec![b, b, b, b, n, n, n, n],
        vec![b, b, b, b, b, n, n, n],
        vec![n, b, b, b, b, b, b, n],
        vec![n, n, b, b, b, b, n, n],
    ]
}

fn packman_closed(body: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let d = Some(Color::Rgb(180, 150, 30));
    let w = Some(EW); let p = Some(EP); let n = n();
    vec![
        vec![n, n, b, b, b, b, n, n],
        vec![n, b, b, b, b, b, b, n],
        vec![b, b, w, p, b, b, b, b],
        vec![b, b, b, b, b, b, b, b],
        vec![b, d, d, d, d, d, d, b],
        vec![b, b, b, b, b, b, b, b],
        vec![n, b, b, b, b, b, b, n],
        vec![n, n, b, b, b, b, n, n],
    ]
}

fn ghost(body: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let w = Some(EW); let p = Some(EP); let n = n();
    vec![
        vec![n, n, b, b, b, b, n, n],
        vec![n, b, b, b, b, b, b, n],
        vec![b, w, p, b, b, w, p, b],
        vec![b, b, b, b, b, b, b, b],
        vec![b, b, b, b, b, b, b, b],
        vec![b, b, b, b, b, b, b, b],
        vec![b, b, n, b, b, n, b, b],
        vec![b, n, n, b, n, n, b, n],
    ]
}

fn done_sprite(body: Color) -> Vec<Vec<Pixel>> {
    let b = Some(body); let c = Some(Color::Rgb(200, 255, 200)); let n = n();
    vec![
        vec![n, n, b, b, b, b, n, n],
        vec![n, b, b, b, b, b, b, n],
        vec![b, b, b, b, b, c, b, b],
        vec![b, b, b, b, c, b, b, b],
        vec![b, b, c, b, b, b, b, b],  // checkmark
        vec![b, b, b, c, b, b, b, b],
        vec![n, b, b, b, b, b, b, n],
        vec![n, n, b, b, b, b, n, n],
    ]
}

struct AgentCard {
    name: &'static str,
    state: &'static str,
    tool: &'static str,
    body: Color,
    ctx_pct: u16,
    tokens: &'static str,
    cost: &'static str,
    is_waiting: bool,
    is_done: bool,
}

fn render_agent_card(f: &mut Frame, area: Rect, agent: &AgentCard, tick: usize, bg: Color) {
    // Card background
    let border_color = if agent.is_done {
        Color::Rgb(50, 50, 60)
    } else if agent.is_waiting {
        Color::Rgb(60, 60, 80)
    } else {
        Color::Rgb(agent.body.to_string().len() as u8 * 3, 60, 80) // subtle
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 55)))
        .style(Style::default().bg(bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 10 || inner.height < 6 {
        return;
    }

    // Sprite - centered at top of card
    let sprite = if agent.is_done {
        done_sprite(agent.body)
    } else if agent.is_waiting {
        ghost(agent.body)
    } else if (tick / 6) % 2 == 0 {
        packman_open(agent.body)
    } else {
        packman_closed(agent.body)
    };

    let lines = sprite_to_lines(&sprite, bg);
    let sprite_width = 16u16; // 8 pixels * 2 chars
    let sprite_x = inner.x + (inner.width.saturating_sub(sprite_width)) / 2;

    for (j, line) in lines.iter().enumerate() {
        let y = inner.y + j as u16;
        if y < inner.y + inner.height {
            f.render_widget(Paragraph::new(line.clone()).style(Style::default().bg(bg)), Rect::new(sprite_x, y, sprite_width, 1));
        }
    }

    // Effects next to sprite
    let effect_x = sprite_x + sprite_width + 1;
    if agent.is_waiting {
        let zzz_frame = (tick / 10) % 4;
        let zzz_texts = ["z", " zz", "  zzz", "   zz"];
        let zzz = zzz_texts[zzz_frame];
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(zzz, Style::default().fg(Color::Rgb(130, 130, 180))))),
            Rect::new(effect_x, inner.y + 1, 6, 1),
        );
    } else if !agent.is_done {
        let bolt_on = (tick / 3) % 4 != 0;
        if bolt_on {
            f.render_widget(
                Paragraph::new(Line::from(Span::styled("*", Style::default().fg(Color::Rgb(255, 255, 100))))),
                Rect::new(effect_x, inner.y + 1, 2, 1),
            );
        }
    }

    // Info below sprite
    let info_y = inner.y + 5;

    // Name
    if info_y < inner.y + inner.height {
        let name_line = Line::from(Span::styled(
            format!("{:^width$}", agent.name, width = inner.width as usize),
            Style::default().fg(agent.body).add_modifier(Modifier::BOLD),
        ));
        f.render_widget(Paragraph::new(name_line), Rect::new(inner.x, info_y, inner.width, 1));
    }

    // State / tool
    let state_y = info_y + 1;
    if state_y < inner.y + inner.height {
        let state_color = if agent.is_done { Color::Rgb(80, 80, 100) } else if agent.is_waiting { Color::Rgb(200, 200, 80) } else { Color::Rgb(100, 220, 140) };
        let text = if !agent.tool.is_empty() { agent.tool } else { agent.state };
        let state_line = Line::from(Span::styled(
            format!("{:^width$}", text, width = inner.width as usize),
            Style::default().fg(state_color),
        ));
        f.render_widget(Paragraph::new(state_line), Rect::new(inner.x, state_y, inner.width, 1));
    }

    // Context gauge
    let gauge_y = state_y + 2;
    if gauge_y < inner.y + inner.height && !agent.is_done {
        let filled = (agent.ctx_pct / 10) as usize;
        let empty = 10usize.saturating_sub(filled);
        let gauge_color = if agent.ctx_pct > 80 { Color::Red } else if agent.ctx_pct > 60 { Color::Yellow } else { Color::Rgb(80, 200, 120) };
        let gauge_str = format!("ctx {}{} {}%", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty), agent.ctx_pct);
        let gauge_line = Line::from(Span::styled(
            format!("{:^width$}", gauge_str, width = inner.width as usize),
            Style::default().fg(gauge_color),
        ));
        f.render_widget(Paragraph::new(gauge_line), Rect::new(inner.x, gauge_y, inner.width, 1));
    }

    // Tokens + cost
    let metric_y = gauge_y + 1;
    if metric_y < inner.y + inner.height {
        let metric_line = Line::from(vec![
            Span::styled(
                format!("{:^width$}", format!("{} {}", agent.tokens, agent.cost), width = inner.width as usize),
                Style::default().fg(Color::Rgb(100, 100, 130)),
            ),
        ]);
        f.render_widget(Paragraph::new(metric_line), Rect::new(inner.x, metric_y, inner.width, 1));
    }
}

fn render(f: &mut Frame, tick: usize) {
    let bg = Color::Rgb(18, 18, 28);
    f.render_widget(Paragraph::new("").style(Style::default().bg(bg)), f.area());

    let agents = vec![
        AgentCard { name: "main-worker", state: "active", tool: "Edit auth.ts", body: Color::Rgb(255, 220, 50), ctx_pct: 67, tokens: "45.2k", cost: "$0.32", is_waiting: false, is_done: false },
        AgentCard { name: "team-review", state: "waiting", tool: "", body: Color::Rgb(80, 200, 255), ctx_pct: 22, tokens: "12.1k", cost: "$0.08", is_waiting: true, is_done: false },
        AgentCard { name: "sub-scout", state: "active", tool: "Bash npm test", body: Color::Rgb(255, 140, 200), ctx_pct: 45, tokens: "8.4k", cost: "$0.06", is_waiting: false, is_done: false },
        AgentCard { name: "agent-build", state: "done", tool: "", body: Color::Rgb(80, 220, 120), ctx_pct: 0, tokens: "20.0k", cost: "$0.14", is_waiting: false, is_done: true },
    ];

    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // title
            Constraint::Fill(1),     // cards
            Constraint::Length(1),   // status
        ])
        .split(area);

    // Title
    let title = Line::from(vec![
        Span::styled(" packmen ", Style::default().fg(Color::Rgb(255, 220, 50)).add_modifier(Modifier::BOLD)),
        Span::styled("agent monitor", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("  (q to quit)", Style::default().fg(Color::Rgb(50, 50, 70))),
    ]);
    f.render_widget(Paragraph::new(title), outer[0]);

    // Agent cards - evenly spaced
    let card_constraints: Vec<Constraint> = agents.iter().map(|_| Constraint::Ratio(1, agents.len() as u32)).collect();
    let card_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(card_constraints)
        .split(outer[1]);

    for (i, agent) in agents.iter().enumerate() {
        render_agent_card(f, card_areas[i], agent, tick, bg);
    }

    // Status bar
    let status = Line::from(vec![
        Span::styled(" agents:", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("4", Style::default().fg(Color::White)),
        Span::styled("  tokens:", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("85.7k", Style::default().fg(Color::White)),
        Span::styled("  cost:", Style::default().fg(Color::Rgb(80, 80, 100))),
        Span::styled("$0.60", Style::default().fg(Color::Rgb(80, 220, 120))),
        Span::styled("  ", Style::default()),
        Span::styled("q:quit  Tab:switch tabs  j/k:scroll", Style::default().fg(Color::Rgb(60, 60, 80))),
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
