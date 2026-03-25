use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

#[allow(dead_code)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        EventHandler {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    pub fn next(&self) -> Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => Ok(AppEvent::Key(key)),
                Event::Key(_) => Ok(AppEvent::Tick), // ignore Release/Repeat
                Event::Resize(w, h) => Ok(AppEvent::Resize(w, h)),
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
