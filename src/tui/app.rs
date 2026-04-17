use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::store::analytics::AnalyticsWindow;

// Single view — no tabs needed

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Sidebar,
    MainPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingsSection {
    Commands,
    Skills,
    Agents,
}

impl RankingsSection {
    fn next(self) -> Self {
        match self {
            RankingsSection::Commands => RankingsSection::Skills,
            RankingsSection::Skills => RankingsSection::Agents,
            RankingsSection::Agents => RankingsSection::Commands,
        }
    }

    fn prev(self) -> Self {
        match self {
            RankingsSection::Commands => RankingsSection::Agents,
            RankingsSection::Skills => RankingsSection::Commands,
            RankingsSection::Agents => RankingsSection::Skills,
        }
    }
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
    pub sidebar_count: usize,
    pub commands_scroll_offset: usize,
    pub skills_scroll_offset: usize,
    pub agents_scroll_offset: usize,
    pub rankings_section: RankingsSection,
    pub rankings_window: AnalyticsWindow,

    // Cached counts for scroll bounds (updated each frame from store snapshot)
    pub agent_count: usize,
    pub commands_count: usize,
    pub skills_count: usize,
    pub rankings_agents_count: usize,
    pub session_count: usize,

    pub port: u16,

    pub tick: usize,

    // Project selection (cwd-based)
    pub current_project: Option<String>, // selected cwd, None = all
    pub project_index: usize,            // index into project list

    // Agent filter mode: when set, rankings show only this agent's data
    pub focused_agent: Option<String>,
    pub pending_focus_select: bool,
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
            sidebar_count: 0,
            commands_scroll_offset: 0,
            skills_scroll_offset: 0,
            agents_scroll_offset: 0,
            rankings_section: RankingsSection::Commands,
            rankings_window: AnalyticsWindow::Hours24,
            agent_count: 0,
            commands_count: 0,
            skills_count: 0,
            rankings_agents_count: 0,
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
    pub fn update_counts(
        &mut self,
        sidebar_count: usize,
        commands_count: usize,
        skills_count: usize,
        agents_count: usize,
        session_count: usize,
    ) {
        self.sidebar_count = sidebar_count;
        self.agent_count = sidebar_count;
        self.commands_count = commands_count;
        self.skills_count = skills_count;
        self.rankings_agents_count = agents_count;
        self.session_count = session_count;
        self.commands_scroll_offset = self
            .commands_scroll_offset
            .min(commands_count.saturating_sub(1));
        self.skills_scroll_offset = self
            .skills_scroll_offset
            .min(skills_count.saturating_sub(1));
        self.agents_scroll_offset = self
            .agents_scroll_offset
            .min(agents_count.saturating_sub(1));
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

        // Esc exits agent filter first, then normal behavior
        if code == KeyCode::Esc && self.focused_agent.is_some() {
            self.focused_agent = None;
            self.reset_rankings_scroll();
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
            KeyCode::Tab => self.advance_main_section(),
            KeyCode::BackTab => self.rewind_main_section(),
            KeyCode::Char(',') => self.prev_window(),
            KeyCode::Char('.') => self.next_window(),

            // Enter: in sidebar = filter rankings by selected agent
            KeyCode::Enter => {
                if self.focus == FocusPane::Sidebar {
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
                self.reset_rankings_scroll();
            }

            _ => {}
        }
    }

    /// Handle mouse scroll — macOS natural scrolling: ScrollDown = content up
    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            // macOS natural scrolling: ScrollDown moves content up (= scroll_down)
            MouseEventKind::ScrollDown => self.scroll_down(),
            MouseEventKind::ScrollUp => self.scroll_up(),
            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                let max = self.sidebar_count.saturating_sub(1);
                if self.sidebar_selected < max {
                    self.sidebar_selected += 1;
                }
            }
            FocusPane::MainPanel => {
                let max = self.active_section_count().saturating_sub(1);
                let offset = self.active_section_offset_mut();
                if *offset < max {
                    *offset += 1;
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
                let offset = self.active_section_offset_mut();
                if *offset > 0 {
                    *offset = (*offset).saturating_sub(1);
                }
            }
        }
    }

    fn scroll_to_top(&mut self) {
        match self.focus {
            FocusPane::Sidebar => self.sidebar_selected = 0,
            FocusPane::MainPanel => {
                *self.active_section_offset_mut() = 0;
            }
        }
    }

    fn scroll_to_bottom(&mut self) {
        match self.focus {
            FocusPane::Sidebar => {
                self.sidebar_selected = self.sidebar_count.saturating_sub(1);
            }
            FocusPane::MainPanel => {
                *self.active_section_offset_mut() = self.active_section_count().saturating_sub(1);
            }
        }
    }

    fn advance_main_section(&mut self) {
        if self.focus == FocusPane::MainPanel {
            self.rankings_section = self.rankings_section.next();
        }
    }

    fn rewind_main_section(&mut self) {
        if self.focus == FocusPane::MainPanel {
            self.rankings_section = self.rankings_section.prev();
        }
    }

    fn next_window(&mut self) {
        self.rankings_window = self.rankings_window.next();
        self.reset_rankings_scroll();
    }

    fn prev_window(&mut self) {
        self.rankings_window = self.rankings_window.prev();
        self.reset_rankings_scroll();
    }

    fn reset_rankings_scroll(&mut self) {
        self.commands_scroll_offset = 0;
        self.skills_scroll_offset = 0;
        self.agents_scroll_offset = 0;
    }

    fn active_section_count(&self) -> usize {
        match self.rankings_section {
            RankingsSection::Commands => self.commands_count,
            RankingsSection::Skills => self.skills_count,
            RankingsSection::Agents => self.rankings_agents_count,
        }
    }

    fn active_section_offset_mut(&mut self) -> &mut usize {
        match self.rankings_section {
            RankingsSection::Commands => &mut self.commands_scroll_offset,
            RankingsSection::Skills => &mut self.skills_scroll_offset,
            RankingsSection::Agents => &mut self.agents_scroll_offset,
        }
    }

    /// Update current_project based on available projects from agents.
    /// Preserves selection by name when sort order changes.
    pub fn update_projects(&mut self, projects: &[String]) {
        if projects.is_empty() {
            self.current_project = None;
            self.project_index = 0;
            return;
        }

        // If we have a current selection, find it in the new list
        if let Some(ref current) = self.current_project {
            if let Some(pos) = projects.iter().position(|p| p == current) {
                self.project_index = pos;
                return; // selection still valid
            }
        }

        // No previous selection or it vanished — use index
        if self.project_index >= projects.len() {
            self.project_index = 0;
        }
        self.current_project = Some(projects[self.project_index].clone());
    }

    fn next_project(&mut self) {
        self.project_index = self.project_index.wrapping_add(1);
        self.current_project = None; // force re-resolve from new index
    }

    fn prev_project(&mut self) {
        self.current_project = None; // force re-resolve from new index
        if self.project_index == 0 {
            self.project_index = usize::MAX; // will wrap in update_projects
        } else {
            self.project_index -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{App, FocusPane, RankingsSection};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn tab_cycles_rankings_sections() {
        let mut app = App::new(8080);
        app.focus = FocusPane::MainPanel;

        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.rankings_section, RankingsSection::Skills);

        app.handle_key(key(KeyCode::Tab));
        assert_eq!(app.rankings_section, RankingsSection::Agents);
    }

    #[test]
    fn comma_and_period_cycle_windows() {
        let mut app = App::new(8080);

        app.handle_key(key(KeyCode::Char('.')));
        assert_eq!(app.rankings_window.label(), "7d");

        app.handle_key(key(KeyCode::Char(',')));
        assert_eq!(app.rankings_window.label(), "24h");
    }
}
