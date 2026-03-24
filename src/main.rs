mod collector;
mod protocol;
mod store;
mod tui;

use std::io;
use std::panic;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::event::{AppEvent, EventHandler};
use crate::tui::render::{self, StoreSnapshot};

#[derive(Parser, Debug)]
#[command(name = "packmen", about = "Terminal dashboard for Claude Code agents")]
struct Cli {
    /// HTTP hook server port
    #[arg(long, default_value_t = 3100)]
    port: u16,

    /// HTTP hook server bind address
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,

    /// Tick rate in milliseconds
    #[arg(long, default_value_t = 100)]
    tick_rate: u64,
}

/// Guard that restores terminal on drop (including panics)
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Create shared store
    let shared_store = AppStore::new_shared();

    // Create mpsc channel
    let (tx, mut rx) = mpsc::channel(1024);

    // Spawn HTTP server
    let bind = cli.bind.clone();
    let port = cli.port;
    tokio::spawn(async move {
        if let Err(e) = collector::http_server::run_http_server(bind, port, tx).await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    // Spawn store updater task
    let store_for_updater = shared_store.clone();
    tokio::spawn(async move {
        let mut gc_interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            tokio::select! {
                Some(event) = rx.recv() => {
                    let mut store = store_for_updater.write().await;
                    store.apply_event(event);
                }
                _ = gc_interval.tick() => {
                    let now = Utc::now().timestamp();
                    let mut store = store_for_updater.write().await;
                    store.gc_stale_agents(now);
                }
            }
        }
    });

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Drop guard ensures cleanup even if we return early via ?
    let _guard = TerminalGuard;

    // Create app state
    let mut app = App::new(cli.port);

    // Event handler
    let event_handler = EventHandler::new(cli.tick_rate);

    // Main loop
    while app.running {
        // Take a snapshot of the store for rendering
        let snap = StoreSnapshot::from_store(&shared_store).await;

        // Update cached counts for scroll bounds
        app.update_counts(snap.agents.len(), snap.feed.len(), snap.sessions.len());

        terminal.draw(|f| render::draw(f, &app, &snap))?;

        match event_handler.next()? {
            AppEvent::Key(key) => app.handle_key(key),
            AppEvent::Tick => {
                // Snapshot already taken above
            }
            AppEvent::Resize(_, _) => {
                // Terminal handles resize automatically on next draw
            }
        }
    }

    Ok(())
}
