use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::Result;

/// File system events relevant to the vault.
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

/// Watches a vault directory for file changes with proper lifecycle management.
pub struct VaultWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<FileEvent>,
}

impl VaultWatcher {
    /// Create a new watcher for the given vault path.
    pub fn new(vault_path: &Path) -> Result<Self> {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            for path in event.paths {
                                if is_markdown(&path) {
                                    let _ = tx.send(FileEvent::Created(path));
                                }
                            }
                        }
                        EventKind::Modify(_) => {
                            for path in event.paths {
                                if is_markdown(&path) {
                                    let _ = tx.send(FileEvent::Modified(path));
                                }
                            }
                        }
                        EventKind::Remove(_) => {
                            for path in event.paths {
                                if is_markdown(&path) {
                                    let _ = tx.send(FileEvent::Deleted(path));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        watcher
            .watch(vault_path, RecursiveMode::Recursive)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Poll for events, debouncing rapid changes to the same file.
    /// Returns a batch of events after no new events arrive within `debounce`.
    pub fn poll(&self, debounce: Duration) -> Vec<FileEvent> {
        let mut last_events: HashMap<PathBuf, FileEvent> = HashMap::new();

        // Wait for at least one event
        let mut last_event_time = match self.rx.recv() {
            Ok(event) => {
                let path = event_path(&event).to_path_buf();
                last_events.insert(path, event);
                Instant::now()
            }
            Err(_) => return Vec::new(),
        };

        // Collect more events within the debounce window
        loop {
            let remaining = debounce.saturating_sub(last_event_time.elapsed());
            if remaining.is_zero() {
                break;
            }

            match self.rx.recv_timeout(remaining) {
                Ok(event) => {
                    let path = event_path(&event).to_path_buf();
                    last_events.insert(path, event);
                    last_event_time = Instant::now();
                }
                Err(_) => break,
            }
        }

        last_events.into_values().collect()
    }
}

fn event_path(event: &FileEvent) -> &Path {
    match event {
        FileEvent::Created(p) | FileEvent::Modified(p) | FileEvent::Deleted(p) => p,
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md")
        .unwrap_or(false)
}
