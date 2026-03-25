mod collector;
mod config;
mod protocol;
mod store;
mod tui;

use std::io;
use std::panic;
use std::path::PathBuf;

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
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::store::state::AppStore;
use crate::tui::app::App;
use crate::tui::event::{AppEvent, EventHandler};
use crate::tui::render::{self, StoreSnapshot};

#[derive(Parser, Debug)]
#[command(
    name = "packmen",
    about = "Terminal dashboard for monitoring Claude Code agent sessions",
    long_about = None
)]
struct Cli {
    /// HTTP hook server port (overrides config file)
    #[arg(long)]
    port: Option<u16>,

    /// HTTP hook server bind address (overrides config file)
    #[arg(long)]
    bind: Option<String>,

    /// Tick rate in milliseconds (overrides config file)
    #[arg(long)]
    tick_rate: Option<u64>,

    /// JSONL watch directory (defaults to ~/.claude/projects/)
    #[arg(long)]
    watch_dir: Option<PathBuf>,

    /// Disable JSONL watcher
    #[arg(long)]
    no_jsonl: bool,

    /// Populate store with synthetic demo data (HTTP server still runs)
    #[arg(long)]
    mock: bool,

    /// Write tracing logs to this file path
    #[arg(long)]
    log_file: Option<PathBuf>,
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

    // ----------------------------------------------------------------
    // Load config file (missing file = defaults; parse errors are fatal)
    // ----------------------------------------------------------------
    let cfg = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: failed to load config file: {e}");
        Config::default()
    });

    // CLI args override config file values
    let port = cli.port.unwrap_or(cfg.server.port);
    let bind = cli.bind.unwrap_or(cfg.server.bind);
    let tick_rate = cli.tick_rate.unwrap_or(cfg.tui.tick_rate);
    let watcher_enabled = cfg.watcher.enabled && !cli.no_jsonl;
    let watch_dir = cli.watch_dir.or(cfg.watcher.watch_dir);

    // ----------------------------------------------------------------
    // Tracing / logging
    // ----------------------------------------------------------------
    if let Some(log_path) = &cli.log_file {
        let parent = log_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let file_name = log_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("packmen.log"));

        let file_appender = tracing_appender::rolling::never(parent, file_name);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        // _guard must stay alive for the duration of main — store in a Box to avoid
        // it being dropped immediately (the compiler warns if we name it _).
        Box::leak(Box::new(_guard));

        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
    }

    // ----------------------------------------------------------------
    // Shutdown token — shared across all background tasks
    // ----------------------------------------------------------------
    let shutdown = CancellationToken::new();

    // ----------------------------------------------------------------
    // Panic hook: restore terminal then run original handler
    // ----------------------------------------------------------------
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // ----------------------------------------------------------------
    // Shared store
    // ----------------------------------------------------------------
    let shared_store = AppStore::new_shared();

    // Pre-populate with mock data when --mock is passed
    if cli.mock {
        let mut store = shared_store.write().await;
        store.populate_mock_data();
    }

    // ----------------------------------------------------------------
    // Event channel
    // ----------------------------------------------------------------
    let (tx, mut rx) = mpsc::channel(1024);

    // ----------------------------------------------------------------
    // Spawn HTTP server
    // ----------------------------------------------------------------
    {
        let bind_addr = bind.clone();
        let tx_http = tx.clone();
        let sd = shutdown.clone();
        tokio::spawn(async move {
            tokio::select! {
                result = collector::http_server::run_http_server(bind_addr, port, tx_http) => {
                    if let Err(e) = result {
                        // Port already in use or other bind error — log and continue;
                        // TUI still works via JSONL watcher.
                        tracing::warn!("HTTP server failed to start: {e}");
                        eprintln!("Warning: HTTP server could not bind on port {port}: {e}");
                        eprintln!("  -> TUI will still work via JSONL watcher.");
                    }
                }
                _ = sd.cancelled() => {}
            }
        });
    }

    // ----------------------------------------------------------------
    // Spawn JSONL watcher
    // ----------------------------------------------------------------
    if watcher_enabled {
        let dir = watch_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".claude")
                .join("projects")
        });

        if !dir.exists() {
            eprintln!(
                "Warning: JSONL watch directory does not exist: {}",
                dir.display()
            );
            eprintln!("  -> Continuing without JSONL watcher.");
        } else {
            let tx_jsonl = tx.clone();
            let sd = shutdown.clone();
            tokio::spawn(async move {
                tokio::select! {
                    result = collector::jsonl_watcher::run_jsonl_watcher(dir, tx_jsonl) => {
                        if let Err(e) = result {
                            tracing::warn!("JSONL watcher error: {e}");
                            eprintln!("JSONL watcher error: {e}");
                        }
                    }
                    _ = sd.cancelled() => {}
                }
            });
        }
    }

    // ----------------------------------------------------------------
    // Store updater task
    // ----------------------------------------------------------------
    {
        let store_for_updater = shared_store.clone();
        let sd = shutdown.clone();
        tokio::spawn(async move {
            let mut gc_interval =
                tokio::time::interval(std::time::Duration::from_secs(10));
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
                    _ = sd.cancelled() => break,
                }
            }
        });
    }

    // ----------------------------------------------------------------
    // Terminal setup
    // ----------------------------------------------------------------
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Drop guard ensures cleanup even if we return early via ?
    let _guard = TerminalGuard;

    // ----------------------------------------------------------------
    // TUI main loop
    // ----------------------------------------------------------------
    let mut app = App::new(port);
    let event_handler = EventHandler::new(tick_rate);

    while app.running {
        let snap = StoreSnapshot::from_store(&shared_store).await;
        app.update_counts(snap.agents.len(), snap.feed.len(), snap.sessions.len());
        terminal.draw(|f| render::draw(f, &app, &snap))?;

        match event_handler.next()? {
            AppEvent::Key(key) => app.handle_key(key),
            AppEvent::Tick => {
                app.tick += 1;

                if !app.stage.initialized {
                    app.stage.initialized = true;
                }
                app.stage.tick += 1;

                // Update project list
                let projects = crate::tui::widgets::stage::get_projects(&snap);
                app.update_projects(&projects);
            }
            AppEvent::Resize(_, _) => {}
        }
    }

    // ----------------------------------------------------------------
    // Graceful shutdown: signal tasks and give them 500 ms to finish
    // ----------------------------------------------------------------
    shutdown.cancel();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    Ok(())
}
