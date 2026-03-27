use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Single view — no tabs needed

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Sidebar,
    MainPanel,
}

pub struct App {
    pub focus: FocusPane,
    pub update_available: Option<String>, // Some("0.3.0") if new version
    pub running: bool,
    pub show_detail_overlay: bool,
    pub show_filter: bool,
    pub filter_text: String,

    // Scroll / selection state
    pub sidebar_selected: usize,
    pub feed_scroll_offset: usize,

    // Auto-scroll: true when user hasn't scrolled up
    pub feed_auto_scroll: bool,

    // Cached counts for scroll bounds (updated each frame from store snapshot)
    pub agent_count: usize,
    pub feed_count: usize,
    pub session_count: usize,

    pub port: u16,

    pub tick: usize,

    // Project selection (cwd-based)
    pub current_project: Option<String>,  // selected cwd, None = all
    pub project_index: usize,             // index into project list

    // Sub-agent focus mode: when set, conversation shows only this agent's events
    pub focused_agent: Option<String>,    // agent_id of focused sub-agent
    pub pending_focus_select: bool,       // set by Enter key, resolved by renderer
}

impl App {
    pub fn new(port: u16) -> Self {
        App {
            focus: FocusPane::MainPanel,
            update_available: None,
            running: true,
            show_detail_overlay: false,
            show_filter: false,
            filter_text: String::new(),
            sidebar_selected: 0,
            feed_scroll_offset: 0,
            feed_auto_scroll: true,
            agent_count: 0,
            feed_count: 0,
            session_count: 0,
            port,
            tick: 0,
            current_project: None,
            project_index: 0,
            focused_agent: None,
            pending_focus_select: false,
        }
    }

    /// Called each tick to update cached counts from the store snapshot.
    pub fn update_counts(&mut self, agent_count: usize, feed_count: usize, session_count: usize) {
        self.agent_count = agent_count;
        self.feed_count = feed_count;
        self.session_count = session_count;

        // Auto-scroll feed to bottom
        if self.feed_auto_scroll && feed_count > 0 {
            self.feed_scroll_offset = feed_count.saturating_sub(1);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }

        // Filter mode input
        if self.show_filter {
            match key.code {
                KeyCode::Esc => {
                    self.show_filter = false;
                    self.filter_text.clear();
                }
                KeyCode::Enter => {
                    self.show_filter = false;
                }
                KeyCode::Backspace => {
                    self.filter_text.pop();
                }
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                }
                _ => {}
            }
            return;
        }

        // Detail overlay dismissal (legacy, kept for compatibility)
        if self.show_detail_overlay {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('ㅂ') => {
                    self.show_detail_overlay = false;
                }
                _ => {}
            }
            return;
        }

        // Normalize Korean IME characters to their QWERTY equivalents
        let code = match key.code {
            KeyCode::Char('ㅓ') => KeyCode::Char('j'),
            KeyCode::Char('ㅏ') => KeyCode::Char('k'),
            KeyCode::Char('ㅗ') => KeyCode::Char('h'),
            KeyCode::Char('ㅣ') => KeyCode::Char('l'),
            KeyCode::Char('ㅂ') => KeyCode::Char('q'),
            KeyCode::Char('ㄹ') => KeyCode::Char('f'),
            KeyCode::Char('ㅎ') => KeyCode::Char('g'),
            other => other,
        };

        // Esc exits focus mode first, then normal behavior
        if code == KeyCode::Esc && self.focused_agent.is_some() {
            self.focused_agent = None;
            return;
        }

        match code {
            KeyCode::Char('q') => self.running = false,

            // Focus switching
            KeyCode::Char('h') | KeyCode::Left => self.focus = FocusPane::Sidebar,
            KeyCode::Char('l') | KeyCode::Right => self.focus = FocusPane::MainPanel,

            // Scrolling
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
            KeyCode::Char('g') => self.scroll_to_top(),
            KeyCode::Char('G') => self.scroll_to_bottom(),

            // Enter: in sidebar = focus on selected agent's conversation
            KeyCode::Enter => {
                if self.focus == FocusPane::Sidebar {
                    // agent_id will be resolved by the renderer from sidebar_selected
                    self.pending_focus_select = true;
                }
            }

            // Filter
            KeyCode::Char('f') => {
                self.show_filter = true;
                self.filter_text.clear();
            }

            // Project cycling
            KeyCode::Char('[') => self.prev_project(),
            KeyCode::Char(']') => self.next_project(),

            // Esc: exit focus mode or do nothing
            KeyCode::Esc => {
                self.focused_agent = None;
            }

            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                let max = self.agent_count.saturating_sub(1);
                if self.sidebar_selected < max {
                    self.sidebar_selected += 1;
                }
            }
            FocusPane::MainPanel => {
                let max = self.feed_count.saturating_sub(1);
                if self.feed_scroll_offset < max {
                    self.feed_scroll_offset += 1;
                }
                if self.feed_scroll_offset >= self.feed_count.saturating_sub(1) {
                    self.feed_auto_scroll = true;
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                self.sidebar_selected = self.sidebar_selected.saturating_sub(1);
            }
            FocusPane::MainPanel => {
                if self.feed_scroll_offset > 0 {
                    self.feed_scroll_offset = self.feed_scroll_offset.saturating_sub(1);
                    self.feed_auto_scroll = false;
                }
            }
        }
    }

    fn scroll_to_top(&mut self) {
        match self.focus {
            FocusPane::Sidebar => self.sidebar_selected = 0,
            FocusPane::MainPanel => {
                self.feed_scroll_offset = 0;
                self.feed_auto_scroll = false;
            }
        }
    }

    fn scroll_to_bottom(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                self.sidebar_selected = self.agent_count.saturating_sub(1);
            }
            FocusPane::MainPanel => {
                self.feed_scroll_offset = self.feed_count.saturating_sub(1);
                self.feed_auto_scroll = true;
            }
        }
    }

    /// Update current_project based on available projects from agents
    pub fn update_projects(&mut self, projects: &[String]) {
        if projects.is_empty() {
            self.current_project = None;
            self.project_index = 0;
            return;
        }
        // Wrap index if out of bounds
        if self.project_index >= projects.len() {
            self.project_index %= projects.len();
        }
        // Resolve project from index
        self.current_project = Some(projects[self.project_index].clone());
    }

    fn next_project(&mut self) {
        self.project_index = self.project_index.wrapping_add(1);
    }

    fn prev_project(&mut self) {
        if self.project_index == 0 {
            self.project_index = usize::MAX; // will wrap in update_projects
        } else {
            self.project_index -= 1;
        }
    }
}
