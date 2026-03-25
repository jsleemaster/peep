use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::sprites::stage_state::StageState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Stage,
    Feed,
    Agents,
    Sessions,
}

impl Tab {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Stage => "Stage",
            Tab::Feed => "Feed",
            Tab::Agents => "Agents",
            Tab::Sessions => "Sessions",
        }
    }

    pub fn all() -> &'static [Tab] {
        &[Tab::Stage, Tab::Feed, Tab::Agents, Tab::Sessions]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Sidebar,
    MainPanel,
}

pub struct App {
    pub active_tab: Tab,
    pub focus: FocusPane,
    pub running: bool,
    pub show_detail_overlay: bool,
    pub show_filter: bool,
    pub filter_text: String,

    // Scroll / selection state
    pub sidebar_selected: usize,
    pub feed_scroll_offset: usize,
    pub session_scroll_offset: usize,
    pub agents_tab_selected: usize,

    // Auto-scroll: true when user hasn't scrolled up
    pub feed_auto_scroll: bool,

    // Cached counts for scroll bounds (updated each frame from store snapshot)
    pub agent_count: usize,
    pub feed_count: usize,
    pub session_count: usize,

    pub port: u16,

    // Stage state
    pub stage: StageState,
    pub tick: usize,
    #[allow(dead_code)]
    pub last_feed_count: usize,

    // Project selection (cwd-based)
    pub current_project: Option<String>,  // selected cwd, None = all
    pub project_index: usize,             // index into project list
}

impl App {
    pub fn new(port: u16) -> Self {
        App {
            active_tab: Tab::Stage,
            focus: FocusPane::MainPanel,
            running: true,
            show_detail_overlay: false,
            show_filter: false,
            filter_text: String::new(),
            sidebar_selected: 0,
            feed_scroll_offset: 0,
            session_scroll_offset: 0,
            agents_tab_selected: 0,
            feed_auto_scroll: true,
            agent_count: 0,
            feed_count: 0,
            session_count: 0,
            port,
            stage: StageState::new(),
            tick: 0,
            last_feed_count: 0,
            current_project: None,
            project_index: 0,
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

        // Detail overlay dismissal
        if self.show_detail_overlay {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.show_detail_overlay = false;
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.running = false,

            // Tab switching
            KeyCode::Tab => self.next_tab(),
            KeyCode::BackTab => self.prev_tab(),
            KeyCode::Char('1') => self.active_tab = Tab::Stage,
            KeyCode::Char('2') => self.active_tab = Tab::Feed,
            KeyCode::Char('3') => self.active_tab = Tab::Agents,
            KeyCode::Char('4') => self.active_tab = Tab::Sessions,

            // Focus switching
            KeyCode::Char('h') | KeyCode::Left => self.focus = FocusPane::Sidebar,
            KeyCode::Char('l') | KeyCode::Right => self.focus = FocusPane::MainPanel,

            // Scrolling
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(),
            KeyCode::Char('g') => self.scroll_to_top(),
            KeyCode::Char('G') => self.scroll_to_bottom(),

            // Detail overlay
            KeyCode::Enter => {
                if self.focus == FocusPane::Sidebar {
                    self.show_detail_overlay = true;
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

            _ => {}
        }
    }

    fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Stage => Tab::Feed,
            Tab::Feed => Tab::Agents,
            Tab::Agents => Tab::Sessions,
            Tab::Sessions => Tab::Stage,
        };
    }

    fn prev_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Stage => Tab::Sessions,
            Tab::Feed => Tab::Stage,
            Tab::Agents => Tab::Feed,
            Tab::Sessions => Tab::Agents,
        };
    }

    fn scroll_down(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                let max = self.agent_count.saturating_sub(1);
                if self.sidebar_selected < max {
                    self.sidebar_selected += 1;
                }
            }
            FocusPane::MainPanel => match self.active_tab {
                Tab::Stage | Tab::Feed => {
                    let max = self.feed_count.saturating_sub(1);
                    if self.feed_scroll_offset < max {
                        self.feed_scroll_offset += 1;
                    }
                    if self.feed_scroll_offset >= self.feed_count.saturating_sub(1) {
                        self.feed_auto_scroll = true;
                    }
                }
                Tab::Sessions => {
                    let max = self.session_count.saturating_sub(1);
                    if self.session_scroll_offset < max {
                        self.session_scroll_offset += 1;
                    }
                }
                Tab::Agents => {
                    let max = self.agent_count.saturating_sub(1);
                    if self.agents_tab_selected < max {
                        self.agents_tab_selected += 1;
                    }
                }
            },
        }
    }

    fn scroll_up(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                self.sidebar_selected = self.sidebar_selected.saturating_sub(1);
            }
            FocusPane::MainPanel => match self.active_tab {
                Tab::Stage | Tab::Feed => {
                    if self.feed_scroll_offset > 0 {
                        self.feed_scroll_offset = self.feed_scroll_offset.saturating_sub(1);
                        self.feed_auto_scroll = false;
                    }
                }
                Tab::Sessions => {
                    self.session_scroll_offset = self.session_scroll_offset.saturating_sub(1);
                }
                Tab::Agents => {
                    self.agents_tab_selected = self.agents_tab_selected.saturating_sub(1);
                }
            },
        }
    }

    fn scroll_to_top(&mut self) {
        match self.focus {
            FocusPane::Sidebar => self.sidebar_selected = 0,
            FocusPane::MainPanel => match self.active_tab {
                Tab::Stage => {}
                Tab::Feed => {
                    self.feed_scroll_offset = 0;
                    self.feed_auto_scroll = false;
                }
                Tab::Sessions => self.session_scroll_offset = 0,
                Tab::Agents => self.agents_tab_selected = 0,
            },
        }
    }

    fn scroll_to_bottom(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                self.sidebar_selected = self.agent_count.saturating_sub(1);
            }
            FocusPane::MainPanel => match self.active_tab {
                Tab::Stage => {}
                Tab::Feed => {
                    self.feed_scroll_offset = self.feed_count.saturating_sub(1);
                    self.feed_auto_scroll = true;
                }
                Tab::Sessions => {
                    self.session_scroll_offset = self.session_count.saturating_sub(1);
                }
                Tab::Agents => {
                    self.agents_tab_selected = self.agent_count.saturating_sub(1);
                }
            },
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
            self.project_index = self.project_index % projects.len();
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
