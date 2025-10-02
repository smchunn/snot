use std::path::{Path, PathBuf};
use std::fs;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use sha2::{Sha256, Digest};
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Event, EventKind};
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

pub struct FileWatcher {
    vault_path: PathBuf,
}

impl FileWatcher {
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    pub fn watch(&self) -> Result<Receiver<FileEvent>> {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher = notify::recommended_watcher(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) => {
                            for path in event.paths {
                                if Self::is_markdown_file(&path) {
                                    let _ = tx.send(FileEvent::Created(path));
                                }
                            }
                        }
                        EventKind::Modify(_) => {
                            for path in event.paths {
                                if Self::is_markdown_file(&path) {
                                    let _ = tx.send(FileEvent::Modified(path));
                                }
                            }
                        }
                        EventKind::Remove(_) => {
                            for path in event.paths {
                                if Self::is_markdown_file(&path) {
                                    let _ = tx.send(FileEvent::Deleted(path));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        ).context("Failed to create file watcher")?;

        watcher.watch(&self.vault_path, RecursiveMode::Recursive)
            .context("Failed to watch vault directory")?;

        // Keep watcher alive by leaking it
        // In a real application, you'd want to manage this properly
        std::mem::forget(watcher);

        Ok(rx)
    }

    fn is_markdown_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "md")
            .unwrap_or(false)
    }

    pub fn calculate_checksum(path: &Path) -> Result<String> {
        let content = fs::read(path)
            .context("Failed to read file for checksum calculation")?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    pub fn scan_vault(&self) -> Result<Vec<PathBuf>> {
        let mut markdown_files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.vault_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && Self::is_markdown_file(path) {
                markdown_files.push(path.to_path_buf());
            }
        }

        Ok(markdown_files)
    }
}
