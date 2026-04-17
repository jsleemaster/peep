#[path = "../src/protocol/mod.rs"]
mod protocol;
#[path = "../src/store/mod.rs"]
mod store;
#[path = "../src/tui/mod.rs"]
mod tui;
#[path = "../src/update.rs"]
mod update;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};
use ratatui::Terminal;
use unicode_width::UnicodeWidthStr;

use crate::store::state::AppStore;
use crate::tui::app::{App, FocusPane};
use crate::tui::render::{self, RankedEntry, StageRankings, StoreSnapshot};

const TERMINAL_COLS: u16 = 140;
const TERMINAL_ROWS: u16 = 40;
const CELL_WIDTH: f32 = 9.8;
const CELL_HEIGHT: f32 = 18.8;
const SHELL_X: f32 = 28.0;
const SHELL_Y: f32 = 28.0;
const WINDOW_BAR_HEIGHT: f32 = 38.0;
const TERMINAL_PADDING: f32 = 22.0;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let theme = match args.theme.as_str() {
        "dark" => tui::theme::Theme::dark(),
        "light" => tui::theme::Theme::light(),
        other => bail!("unsupported theme `{other}` (expected `dark` or `light`)"),
    };
    tui::theme::init_theme(theme);

    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("failed to create {}", args.output_dir.display()))?;

    for scenario in scenarios_for_theme(&args.theme) {
        let svg = render_scenario_svg(*scenario)?;
        let output = args
            .output_dir
            .join(format!("{}-{}.svg", scenario.slug(), args.theme));
        fs::write(&output, svg).with_context(|| format!("failed to write {}", output.display()))?;
        println!("{}", output.display());
    }

    Ok(())
}

struct Args {
    theme: String,
    output_dir: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut theme = None;
        let mut output_dir = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--theme" => theme = args.next(),
                "--output-dir" => output_dir = args.next().map(PathBuf::from),
                "--help" | "-h" => {
                    println!(
                        "Usage: cargo run --example export_marketing_assets -- --theme <dark|light> --output-dir <dir>"
                    );
                    std::process::exit(0);
                }
                other => bail!("unknown argument `{other}`"),
            }
        }

        Ok(Self {
            theme: theme.unwrap_or_else(|| "dark".to_string()),
            output_dir: output_dir.unwrap_or_else(|| PathBuf::from("assets/product-hunt/raw")),
        })
    }
}

#[derive(Clone, Copy)]
enum Scenario {
    Empty,
    MockOverview,
    MockFocus,
    MockLightOverview,
}

impl Scenario {
    fn slug(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::MockOverview => "mock-overview",
            Self::MockFocus => "mock-focus",
            Self::MockLightOverview => "mock-overview",
        }
    }

    fn window_title(self) -> &'static str {
        match self {
            Self::Empty => "peep waiting for sessions",
            Self::MockOverview | Self::MockLightOverview => "peep --mock",
            Self::MockFocus => "peep --mock (focus mode)",
        }
    }
}

fn scenarios_for_theme(theme: &str) -> &'static [Scenario] {
    match theme {
        "light" => &[Scenario::MockLightOverview],
        _ => &[Scenario::MockOverview, Scenario::MockFocus, Scenario::Empty],
    }
}

fn render_scenario_svg(scenario: Scenario) -> Result<String> {
    let mut store = AppStore::new();
    if !matches!(scenario, Scenario::Empty) {
        store.populate_mock_data();
    }

    let snap = snapshot_from_store(&store);
    let mut app = App::new(3100);
    app.tick = 2400;
    app.update_counts(
        snap.agents.len(),
        snap.rankings.commands.len(),
        snap.rankings.skills.len(),
        snap.rankings.agents.len(),
        snap.sessions.len(),
    );

    let projects = tui::widgets::stage::get_projects(&snap);
    match scenario {
        Scenario::Empty => {
            app.update_projects(&projects);
        }
        Scenario::MockOverview | Scenario::MockLightOverview => {
            app.project_index = 1;
            app.update_projects(&projects);
        }
        Scenario::MockFocus => {
            app.project_index = 1;
            app.update_projects(&projects);
            app.focus = FocusPane::Sidebar;
            app.sidebar_selected = 1;
            app.focused_agent = Some("team-review-0002efgh".to_string());
        }
    }

    let backend = TestBackend::new(TERMINAL_COLS, TERMINAL_ROWS);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render::draw(frame, &mut app, &snap))?;

    let buffer = terminal.backend().buffer();
    Ok(buffer_to_svg(
        buffer,
        scenario.window_title(),
        current_colors(),
    ))
}

fn snapshot_from_store(store: &AppStore) -> StoreSnapshot {
    let now = Utc::now().timestamp();
    let mut analytics = crate::store::analytics::AnalyticsStore::default();
    analytics.populate_mock_data(now);
    let view = analytics.query(crate::store::analytics::AnalyticsQuery::new(
        crate::store::analytics::AnalyticsWindow::Hours24,
        Some("platform"),
        None,
        now,
    ));
    StoreSnapshot {
        agents: store.sorted_agents().into_iter().cloned().collect(),
        feed: store.feed.iter().cloned().collect(),
        sessions: store.sessions.clone(),
        sparkline: store.velocity_sparkline_data(15, now),
        metrics: store.derived_metrics(now),
        available_skills: store.available_skills.clone(),
        rankings: StageRankings {
            window: view.summary.window,
            agents_used: view.summary.agents_used,
            completed: view.summary.completed,
            commands: view
                .commands
                .into_iter()
                .map(|entry| RankedEntry::new(entry.name, entry.count, entry.last_seen))
                .collect(),
            skills: view
                .skills
                .into_iter()
                .map(|entry| RankedEntry::new(entry.name, entry.count, entry.last_seen))
                .collect(),
            agents: view
                .agents
                .into_iter()
                .map(|entry| RankedEntry::new(entry.name, entry.count, entry.last_seen))
                .collect(),
            warming: false,
        },
    }
}

#[derive(Clone, Copy)]
struct ThemeColors {
    bg: Color,
    card_bg: Color,
    fg: Color,
    muted: Color,
    shell_bg: Color,
}

fn current_colors() -> ThemeColors {
    let theme = tui::theme::theme();
    ThemeColors {
        bg: theme.bg,
        card_bg: theme.card_bg,
        fg: theme.text_bright,
        muted: theme.text_dim,
        shell_bg: theme.card_bg,
    }
}

fn buffer_to_svg(buffer: &Buffer, title: &str, colors: ThemeColors) -> String {
    let width = SHELL_X * 2.0 + TERMINAL_PADDING * 2.0 + buffer.area.width as f32 * CELL_WIDTH;
    let height = SHELL_Y * 2.0
        + WINDOW_BAR_HEIGHT
        + TERMINAL_PADDING * 2.0
        + buffer.area.height as f32 * CELL_HEIGHT;
    let screen_y = SHELL_Y + WINDOW_BAR_HEIGHT + TERMINAL_PADDING;
    let screen_x = SHELL_X + TERMINAL_PADDING;

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}" height="{height:.0}" viewBox="0 0 {width:.0} {height:.0}" fill="none">"#
    ));
    svg.push_str(
        r##"<defs>
  <filter id="shadow" x="-20%" y="-20%" width="140%" height="160%">
    <feDropShadow dx="0" dy="24" stdDeviation="28" flood-color="#020617" flood-opacity="0.34"/>
  </filter>
  <linearGradient id="backdrop" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#0b1120"/>
    <stop offset="100%" stop-color="#151b2e"/>
  </linearGradient>
  <linearGradient id="glass" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#ffffff" stop-opacity="0.12"/>
    <stop offset="100%" stop-color="#ffffff" stop-opacity="0.02"/>
  </linearGradient>
</defs>"##,
    );

    svg.push_str(&format!(
        r#"<rect width="{width:.0}" height="{height:.0}" fill="{}"/>"#,
        css_color(colors.bg, colors.bg)
    ));
    svg.push_str(
        r##"<circle cx="120" cy="92" r="210" fill="#f59e0b" fill-opacity="0.12"/>
<circle cx="1420" cy="120" r="160" fill="#38bdf8" fill-opacity="0.10"/>
<circle cx="1480" cy="860" r="220" fill="#fb7185" fill-opacity="0.08"/>"##,
    );

    let shell_w = width - SHELL_X * 2.0;
    let shell_h = height - SHELL_Y * 2.0;
    svg.push_str(&format!(
        r#"<g filter="url(#shadow)">
  <rect x="{SHELL_X}" y="{SHELL_Y}" width="{shell_w:.0}" height="{shell_h:.0}" rx="28" fill="{}" stroke="{}" stroke-width="1.5"/>
  <rect x="{SHELL_X}" y="{SHELL_Y}" width="{shell_w:.0}" height="54" rx="28" fill="url(#glass)"/>
  <rect x="{SHELL_X}" y="{SHELL_Y}" width="{shell_w:.0}" height="{shell_h:.0}" rx="28" fill="none" stroke="rgba(255,255,255,0.06)"/>
</g>"#,
        css_color(colors.shell_bg, colors.shell_bg),
        css_color(colors.muted, colors.muted)
    ));

    let dot_y = SHELL_Y + 20.0;
    svg.push_str(&format!(
        r##"<circle cx="{:.1}" cy="{dot_y:.1}" r="6" fill="#fb7185"/>"##,
        SHELL_X + 20.0
    ));
    svg.push_str(&format!(
        r##"<circle cx="{:.1}" cy="{dot_y:.1}" r="6" fill="#fbbf24"/>"##,
        SHELL_X + 42.0
    ));
    svg.push_str(&format!(
        r##"<circle cx="{:.1}" cy="{dot_y:.1}" r="6" fill="#34d399"/>"##,
        SHELL_X + 64.0
    ));
    svg.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" fill="{}" font-size="17" font-family="'Avenir Next', 'Trebuchet MS', sans-serif" letter-spacing="0.06em">{}</text>"#,
        SHELL_X + 92.0,
        SHELL_Y + 26.0,
        css_color(colors.muted, colors.muted),
        escape_xml(title)
    ));

    svg.push_str(&format!(
        r#"<rect x="{screen_x:.1}" y="{screen_y:.1}" width="{:.1}" height="{:.1}" rx="18" fill="{}"/>"#,
        buffer.area.width as f32 * CELL_WIDTH,
        buffer.area.height as f32 * CELL_HEIGHT,
        css_color(colors.card_bg, colors.card_bg)
    ));

    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            let bg = css_color(cell.bg, colors.card_bg);
            let cell_x = screen_x + x as f32 * CELL_WIDTH;
            let cell_y = screen_y + y as f32 * CELL_HEIGHT;
            if bg != css_color(colors.card_bg, colors.card_bg) {
                svg.push_str(&format!(
                    r#"<rect x="{cell_x:.1}" y="{cell_y:.1}" width="{CELL_WIDTH:.1}" height="{CELL_HEIGHT:.1}" fill="{bg}"/>"#
                ));
            }
        }
    }

    for y in 0..buffer.area.height {
        let mut skip = 0usize;
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let symbol = cell.symbol();
            let width = UnicodeWidthStr::width(symbol).max(1);
            skip = width.saturating_sub(1);
            if symbol.trim().is_empty() {
                continue;
            }
            let fg = css_color(cell.fg, colors.fg);
            let opacity = if cell.modifier.contains(Modifier::DIM) {
                0.78
            } else {
                1.0
            };
            let font_weight = if cell.modifier.contains(Modifier::BOLD) {
                "700"
            } else {
                "500"
            };
            let text_x = screen_x + x as f32 * CELL_WIDTH + 0.8;
            let text_y = screen_y + y as f32 * CELL_HEIGHT + (CELL_HEIGHT * 0.79);
            svg.push_str(&format!(
                r#"<text x="{text_x:.1}" y="{text_y:.1}" fill="{fg}" fill-opacity="{opacity:.2}" font-family="'SFMono-Regular', Menlo, Monaco, Consolas, 'Liberation Mono', monospace" font-size="17" font-weight="{font_weight}" xml:space="preserve">{}</text>"#,
                escape_xml(symbol)
            ));
        }
    }

    svg.push_str("</svg>");
    svg
}

fn css_color(color: Color, fallback: Color) -> String {
    match color {
        Color::Reset => css_color(fallback, fallback),
        Color::Black => "#000000".to_string(),
        Color::Red => "#ef4444".to_string(),
        Color::Green => "#22c55e".to_string(),
        Color::Yellow => "#facc15".to_string(),
        Color::Blue => "#3b82f6".to_string(),
        Color::Magenta => "#d946ef".to_string(),
        Color::Cyan => "#22d3ee".to_string(),
        Color::Gray => "#94a3b8".to_string(),
        Color::DarkGray => "#475569".to_string(),
        Color::LightRed => "#f87171".to_string(),
        Color::LightGreen => "#4ade80".to_string(),
        Color::LightYellow => "#fde047".to_string(),
        Color::LightBlue => "#60a5fa".to_string(),
        Color::LightMagenta => "#e879f9".to_string(),
        Color::LightCyan => "#67e8f9".to_string(),
        Color::White => "#f8fafc".to_string(),
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(idx) => indexed_color(idx),
    }
}

fn indexed_color(idx: u8) -> String {
    const COLORS: [&str; 16] = [
        "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
        "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
    ];
    COLORS
        .get(idx as usize)
        .copied()
        .unwrap_or("#94a3b8")
        .to_string()
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[allow(dead_code)]
fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
