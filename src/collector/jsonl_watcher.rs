use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::protocol::jsonl_payload::parse_jsonl_line;
use crate::protocol::types::RawIngestEvent;

/// Detect which AI tool produced a JSONL file based on its path.
fn detect_ai_tool(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    if path_str.contains("/.claude/") {
        Some("claude".to_string())
    } else if path_str.contains("/.codex/") {
        Some("codex".to_string())
    } else if path_str.contains("/.gemini/") {
        Some("gemini".to_string())
    } else if path_str.contains(".opencode/") {
        Some("opencode".to_string())
    } else {
        None
    }
}

/// Build the list of directories to watch.
pub fn candidate_watch_dirs(base: PathBuf) -> Vec<PathBuf> {
    let mut dirs = vec![base];

    if let Some(home) = dirs::home_dir() {
        let extras = [
            home.join(".codex").join("sessions"),
            home.join(".gemini").join("logs").join("sessions"),
        ];
        for extra in extras {
            if extra.exists() {
                dirs.push(extra);
            }
        }
    }

    dirs
}

pub async fn run_jsonl_watcher(
    watch_dir: PathBuf,
    tx: mpsc::Sender<RawIngestEvent>,
) -> anyhow::Result<()> {
    let (notify_tx, mut notify_rx) = mpsc::channel::<PathBuf>(512);

    let mut offsets: HashMap<PathBuf, u64> = HashMap::new();

    let watch_dirs = candidate_watch_dirs(watch_dir);

    for dir in &watch_dirs {
        if dir.exists() {
            discover_jsonl_files(dir, &mut offsets);
        }
    }

    // Initial scan: background task reads history for skill counts etc.
    {
        let scan_tx = tx.clone();
        let scan_paths: Vec<PathBuf> = offsets.keys().cloned().collect();
        tokio::spawn(async move {
            initial_scan_bg(&scan_paths, &scan_tx).await;
        });
    }

    let notify_tx_cb = notify_tx.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let is_relevant = matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_)
                );
                if is_relevant {
                    for path in event.paths {
                        if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                            let _ = notify_tx_cb.try_send(path);
                        }
                    }
                }
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(500)),
    )?;

    for dir in &watch_dirs {
        if dir.exists() {
            watcher.watch(dir, RecursiveMode::Recursive)?;
        }
    }

    let mut heartbeat_count = 0u32;

    loop {
        let path_opt = tokio::time::timeout(Duration::from_secs(2), notify_rx.recv()).await;

        match path_opt {
            Ok(None) => break,

            Ok(Some(path)) => {
                drain_file(&path, &mut offsets, &tx).await;
            }

            // Heartbeat: scan known files + periodically discover NEW files
            Err(_timeout) => {
                heartbeat_count += 1;

                // Every 5th heartbeat (~10s): discover new JSONL files
                if heartbeat_count.is_multiple_of(5) {
                    for dir in &watch_dirs {
                        if dir.exists() {
                            discover_new_jsonl_files(dir, &mut offsets);
                        }
                    }
                }

                // Scan all known files for new content
                let paths: Vec<PathBuf> = offsets.keys().cloned().collect();
                for path in paths {
                    drain_file(&path, &mut offsets, &tx).await;
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read recent JSONL files to populate initial state.
/// For large files (>1MB), only reads the last 1MB to avoid blocking.
/// Specifically scans for Skill tool_use events for skill counting.
async fn initial_scan_bg(
    paths: &[PathBuf],
    tx: &mpsc::Sender<RawIngestEvent>,
) {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(604800);

    for path in paths {
        let modified = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).map_err(std::io::Error::other))
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if modified < cutoff {
            continue;
        }

        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let mut reader = BufReader::new(file);

        // For large files, seek to last 1MB to avoid parsing 40MB+ files
        if file_len > 1_048_576 {
            let seek_pos = file_len.saturating_sub(1_048_576);
            let _ = reader.seek(SeekFrom::Start(seek_pos));
            // Skip partial first line after seek
            let mut discard = String::new();
            let _ = reader.read_line(&mut discard);
        }

        for line in reader.lines().map_while(Result::ok) {
            if let Some(mut event) = parse_jsonl_line(&line) {
                event.ai_tool = detect_ai_tool(path);
                if tx.send(event).await.is_err() {
                    return;
                }
            }
        }
        tokio::task::yield_now().await;
    }
}

/// Recursively find all `.jsonl` files under `dir` and record their current
/// EOF byte offset so we do NOT replay historical events.
fn discover_jsonl_files(dir: &PathBuf, offsets: &mut HashMap<PathBuf, u64>) {
    let walker = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map(|x| x == "jsonl").unwrap_or(false)
        });

    for entry in walker {
        let path = entry.path().to_path_buf();
        let eof = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        offsets.insert(path, eof);
    }
}

/// Discover NEW jsonl files that aren't in offsets yet.
/// New files start from offset 0 (read everything from the beginning).
fn discover_new_jsonl_files(dir: &PathBuf, offsets: &mut HashMap<PathBuf, u64>) {
    let walker = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map(|x| x == "jsonl").unwrap_or(false)
        });

    for entry in walker {
        let path = entry.path().to_path_buf();
        // Only add files we haven't seen before — start from 0
        offsets.entry(path).or_insert(0);
    }
}

/// Read any new lines from `path` since the last known offset.
async fn drain_file(
    path: &PathBuf,
    offsets: &mut HashMap<PathBuf, u64>,
    tx: &mpsc::Sender<RawIngestEvent>,
) {
    let offset = offsets.entry(path.clone()).or_insert(0);

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let file_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return,
    };

    if file_len < *offset {
        *offset = 0;
    }

    if file_len == *offset {
        return;
    }

    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(*offset)).is_err() {
        return;
    }

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if line.ends_with('\n') {
                    if let Some(mut event) = parse_jsonl_line(&line) {
                        event.ai_tool = detect_ai_tool(path);
                        let _ = tx.try_send(event);
                    }
                } else {
                    let rollback = line.len() as u64;
                    let new_pos = reader
                        .stream_position()
                        .unwrap_or(*offset + rollback);
                    *offset = new_pos - rollback;
                    return;
                }
            }
            Err(_) => break,
        }
    }

    *offset = reader.stream_position().unwrap_or(*offset);
}
