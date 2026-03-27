use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

/// Color palette that adapts to dark/light mode.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub mode: ThemeMode,

    // Backgrounds
    pub bg: Color,
    pub card_bg: Color,
    pub bubble_bg: Color,

    // Borders
    pub border: Color,
    pub border_light: Color,
    pub bubble_border: Color,

    // Text
    pub text: Color,
    pub text_dim: Color,
    pub text_muted: Color,
    pub text_bright: Color,

    // Accent / brand
    pub brand: Color,         // packmen yellow
    pub accent_green: Color,
    pub accent_yellow: Color,
    pub accent_red: Color,
    pub accent_cyan: Color,
    pub accent_magenta: Color,

    // Conversation
    pub tool_read: Color,
    pub tool_edit: Color,
    pub tool_bash: Color,
    pub tool_task: Color,
    pub tool_done: Color,
    pub assistant_text: Color,
    pub sub_agent_text: Color,
    pub user_prompt: Color,
    pub tree_line: Color,

    // HP bar
    pub hp_good: Color,
    pub hp_warn: Color,
    pub hp_danger: Color,
    pub hp_empty: Color,

    // Leader
    pub lead_badge_fg: Color,
    pub lead_badge_bg: Color,
    pub lead_name: Color,

    // AI tool brand colors
    pub ai_claude: Color,
    pub ai_codex: Color,
    pub ai_gemini: Color,
    pub ai_opencode: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Theme {
            mode: ThemeMode::Dark,
            bg: Color::Rgb(18, 18, 28),
            card_bg: Color::Rgb(22, 22, 34),
            bubble_bg: Color::Rgb(32, 32, 48),
            border: Color::Rgb(50, 50, 70),
            border_light: Color::Rgb(60, 60, 80),
            bubble_border: Color::Rgb(55, 55, 75),
            text: Color::White,
            text_dim: Color::Rgb(110, 110, 140),
            text_muted: Color::Rgb(140, 140, 160),
            text_bright: Color::Rgb(220, 220, 240),
            brand: Color::Rgb(255, 220, 50),
            accent_green: Color::Rgb(100, 220, 140),
            accent_yellow: Color::Rgb(255, 200, 80),
            accent_red: Color::Red,
            accent_cyan: Color::Cyan,
            accent_magenta: Color::Magenta,
            tool_read: Color::Cyan,
            tool_edit: Color::Yellow,
            tool_bash: Color::Red,
            tool_task: Color::Magenta,
            tool_done: Color::Rgb(80, 180, 80),
            assistant_text: Color::Rgb(180, 170, 220),
            sub_agent_text: Color::Rgb(160, 180, 200),
            user_prompt: Color::Rgb(100, 200, 100),
            tree_line: Color::Rgb(50, 50, 70),
            hp_good: Color::Rgb(100, 220, 140),
            hp_warn: Color::Rgb(255, 200, 80),
            hp_danger: Color::Rgb(255, 80, 80),
            hp_empty: Color::Rgb(40, 40, 55),
            lead_badge_fg: Color::Rgb(255, 220, 80),
            lead_badge_bg: Color::Rgb(80, 60, 20),
            lead_name: Color::Rgb(255, 200, 80),
            ai_claude: Color::Rgb(255, 160, 50),
            ai_codex: Color::Rgb(120, 200, 255),
            ai_gemini: Color::Rgb(60, 120, 255),
            ai_opencode: Color::Rgb(180, 100, 255),
        }
    }

    pub fn light() -> Self {
        Theme {
            mode: ThemeMode::Light,
            bg: Color::Rgb(245, 245, 240),
            card_bg: Color::Rgb(255, 255, 252),
            bubble_bg: Color::Rgb(235, 235, 230),
            border: Color::Rgb(200, 200, 195),
            border_light: Color::Rgb(180, 180, 175),
            bubble_border: Color::Rgb(190, 190, 185),
            text: Color::Rgb(30, 30, 40),
            text_dim: Color::Rgb(100, 100, 110),
            text_muted: Color::Rgb(130, 130, 140),
            text_bright: Color::Rgb(20, 20, 30),
            brand: Color::Rgb(180, 140, 0),
            accent_green: Color::Rgb(30, 140, 60),
            accent_yellow: Color::Rgb(180, 140, 0),
            accent_red: Color::Rgb(200, 40, 40),
            accent_cyan: Color::Rgb(0, 120, 150),
            accent_magenta: Color::Rgb(140, 50, 140),
            tool_read: Color::Rgb(0, 120, 150),
            tool_edit: Color::Rgb(160, 120, 0),
            tool_bash: Color::Rgb(180, 40, 40),
            tool_task: Color::Rgb(130, 50, 130),
            tool_done: Color::Rgb(40, 140, 60),
            assistant_text: Color::Rgb(80, 60, 140),
            sub_agent_text: Color::Rgb(50, 90, 130),
            user_prompt: Color::Rgb(20, 120, 40),
            tree_line: Color::Rgb(180, 180, 175),
            hp_good: Color::Rgb(30, 160, 70),
            hp_warn: Color::Rgb(200, 150, 0),
            hp_danger: Color::Rgb(200, 40, 40),
            hp_empty: Color::Rgb(220, 220, 215),
            lead_badge_fg: Color::Rgb(120, 90, 0),
            lead_badge_bg: Color::Rgb(255, 240, 180),
            lead_name: Color::Rgb(150, 110, 0),
            ai_claude: Color::Rgb(200, 110, 20),
            ai_codex: Color::Rgb(40, 120, 200),
            ai_gemini: Color::Rgb(30, 80, 210),
            ai_opencode: Color::Rgb(120, 50, 200),
        }
    }

    /// Auto-detect from terminal environment or default to dark.
    pub fn auto_detect() -> Self {
        // Check $COLORFGBG (format: "fg;bg", bg > 8 suggests light)
        if let Ok(val) = std::env::var("COLORFGBG") {
            if let Some(bg_str) = val.split(';').next_back() {
                if let Ok(bg) = bg_str.trim().parse::<u32>() {
                    if bg > 8 {
                        return Self::light();
                    }
                }
            }
        }
        Self::dark()
    }
}

impl Theme {
    /// Return the brand color for a given AI tool identifier string.
    #[allow(dead_code)]
    pub fn ai_tool_color(&self, tool: &str) -> Color {
        match tool {
            "claude" => self.ai_claude,
            "codex" => self.ai_codex,
            "gemini" => self.ai_gemini,
            "opencode" => self.ai_opencode,
            _ => self.text_dim,
        }
    }

    /// Short badge label for an AI tool (e.g. "[C]").
    pub fn ai_tool_badge(tool: &str) -> &'static str {
        match tool {
            "claude" => "[C]",
            "codex" => "[X]",
            "gemini" => "[G]",
            "opencode" => "[O]",
            _ => "[?]",
        }
    }

    /// Unique color for sub-agent by index (0-based). Cycles through a palette.
    pub fn sub_agent_color(&self, index: usize) -> Color {
        let palette = match self.mode {
            ThemeMode::Dark => &[
                Color::Rgb(255, 180, 100), // warm orange
                Color::Rgb(100, 200, 255), // sky blue
                Color::Rgb(200, 140, 255), // lavender
                Color::Rgb(255, 140, 180), // pink
                Color::Rgb(140, 230, 180), // mint
                Color::Rgb(255, 220, 100), // gold
                Color::Rgb(180, 200, 255), // periwinkle
                Color::Rgb(255, 160, 160), // coral
            ],
            ThemeMode::Light => &[
                Color::Rgb(180, 100, 40),  // warm brown
                Color::Rgb(30, 120, 180),  // deep blue
                Color::Rgb(120, 60, 180),  // purple
                Color::Rgb(180, 60, 100),  // rose
                Color::Rgb(40, 140, 90),   // forest
                Color::Rgb(160, 130, 0),   // dark gold
                Color::Rgb(80, 100, 180),  // slate blue
                Color::Rgb(180, 70, 70),   // brick
            ],
        };
        palette[index % palette.len()]
    }
}

/// Global theme access. Set once at startup.
static THEME: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

pub fn init_theme(theme: Theme) {
    THEME.set(theme).expect("theme already initialized");
}

pub fn theme() -> &'static Theme {
    THEME.get().expect("theme not initialized")
}
