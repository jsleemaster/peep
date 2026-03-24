use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::protocol::jsonl_payload::parse_jsonl_line;
use crate::protocol::types::RawIngestEvent;

/// Watch `watch_dir` for `.jsonl` files and send new lines as `RawIngestEvent`s.
///
/// Only lines appended *after* the watcher starts are ingested — existing history
/// is skipped by seeking to EOF on each file before watching begins.
pub async fn run_jsonl_watcher(
    watch_dir: PathBuf,
    tx: mpsc::Sender<RawIngestEvent>,
) -> anyhow::Result<()> {
    // Bridge between the synchronous notify callback and the async processing loop.
    let (notify_tx, mut notify_rx) = mpsc::channel::<PathBuf>(512);

    // Track byte offsets per file so we only read new content.
    let mut offsets: HashMap<PathBuf, u64> = HashMap::new();

    // Discover existing .jsonl files and record their current EOF positions.
    if watch_dir.exists() {
        discover_jsonl_files(&watch_dir, &mut offsets);
    }

    // Build the notify watcher.  The callback runs on a thread pool, so it
    // only sends the modified path through the channel; heavy work happens in
    // the async loop below.
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

    if watch_dir.exists() {
        watcher.watch(&watch_dir, RecursiveMode::Recursive)?;
    }

    // Main async loop — drains the notify channel and processes modified files.
    loop {
        // Wait for a notification (or a 2-second heartbeat to catch any missed
        // events on platforms where notify is less reliable).
        let path_opt = tokio::time::timeout(Duration::from_secs(2), notify_rx.recv()).await;

        match path_opt {
            // Channel closed — watcher thread died; exit gracefully.
            Ok(None) => break,

            // Got a specific path notification.
            Ok(Some(path)) => {
                drain_file(&path, &mut offsets, &tx).await;
            }

            // Timeout — scan all known files for any growth we may have missed.
            Err(_timeout) => {
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

/// Recursively find all `.jsonl` files under `dir` and record their current
/// EOF byte offset so we do **not** replay historical events.
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

/// Read any new lines from `path` since the last known offset, parse them,
/// and forward valid events through `tx`.
async fn drain_file(
    path: &PathBuf,
    offsets: &mut HashMap<PathBuf, u64>,
    tx: &mpsc::Sender<RawIngestEvent>,
) {
    // For new files not yet in the map, start from the beginning.
    let offset = offsets.entry(path.clone()).or_insert(0);

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return, // file may have been deleted; skip silently
    };

    let file_len = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return,
    };

    // Handle file rotation / truncation.
    if file_len < *offset {
        *offset = 0;
    }

    if file_len == *offset {
        return; // nothing new
    }

    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(*offset)).is_err() {
        return;
    }

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Only process complete lines (terminated by '\n').
                if line.ends_with('\n') {
                    if let Some(event) = parse_jsonl_line(&line) {
                        // Non-blocking send; drop event if channel is full.
                        let _ = tx.try_send(event);
                    }
                } else {
                    // Incomplete line at the end — back up so we re-read it
                    // once the writer flushes the newline.
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

    // Update offset to current position.
    *offset = reader.stream_position().unwrap_or(*offset);
}
