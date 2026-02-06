use std::path::Path;
use std::time::Duration;

use anyhow::Result;

use crate::vault::Vault;
use crate::watcher::{FileEvent, VaultWatcher};

pub fn watch_vault(vault_path: &Path) -> Result<()> {
    let mut vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let watcher = VaultWatcher::new(&vault.path)
        .map_err(|e| anyhow::anyhow!("Failed to start watcher: {}", e))?;

    println!("Watching vault at {}", vault.path.display());
    println!("Press Ctrl+C to stop");

    let debounce = Duration::from_millis(200);

    loop {
        let events = watcher.poll(debounce);
        if events.is_empty() {
            continue;
        }

        let mut had_changes = false;

        for event in &events {
            match event {
                FileEvent::Created(path) | FileEvent::Modified(path) => {
                    match vault.ingest_file(path, true) {
                        Ok(_) => {
                            had_changes = true;
                            println!("Updated: {}", path.display());
                        }
                        Err(e) => eprintln!("Error processing {}: {}", path.display(), e),
                    }
                }
                FileEvent::Deleted(path) => match vault.delete_file(path) {
                    Ok(_) => {
                        had_changes = true;
                        println!("Deleted: {}", path.display());
                    }
                    Err(e) => eprintln!("Error deleting {}: {}", path.display(), e),
                },
            }
        }

        // Save once per batch
        if had_changes {
            if let Err(e) = vault.save() {
                eprintln!("Error saving database: {}", e);
            }
        }
    }
}
