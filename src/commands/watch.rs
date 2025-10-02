use std::path::Path;
use anyhow::Result;
use crate::watcher::{FileWatcher, FileEvent};
use crate::db::Database;
use crate::parser::MarkdownParser;
use crate::db::Note;
use std::fs;

pub fn watch_vault(vault_path: &Path) -> Result<()> {
    let db_path = vault_path.join(".snot/db.bin");
    let mut db = Database::with_path(db_path)?;

    let watcher = FileWatcher::new(vault_path.to_path_buf());
    let rx = watcher.watch()?;

    println!("Watching vault at {}", vault_path.display());
    println!("Press Ctrl+C to stop");

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Err(e) = handle_file_event(&mut db, &event, vault_path) {
                    eprintln!("Error handling file event: {}", e);
                } else {
                    println!("Processed: {:?}", event);
                }
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn handle_file_event(
    db: &mut Database,
    event: &FileEvent,
    vault_path: &Path,
) -> Result<()> {
    match event {
        FileEvent::Created(path) | FileEvent::Modified(path) => {
            let content = fs::read_to_string(path)?;
            let parsed = MarkdownParser::parse(&content)?;
            let checksum = FileWatcher::calculate_checksum(path)?;

            let note_id = generate_note_id(path, vault_path)?;

            if let Some(_existing) = db.get(&note_id) {
                // Update existing note
                let mut note = Note::new(
                    note_id.clone(),
                    parsed.title,
                    path.to_path_buf(),
                    checksum,
                );
                note.aliases = parsed.aliases;
                note.tags = parsed.tags;
                note.links = parsed.links;

                db.update(&note_id, note)?;
            } else {
                // Create new note
                let mut note = Note::new(
                    note_id,
                    parsed.title,
                    path.to_path_buf(),
                    checksum,
                );
                note.aliases = parsed.aliases;
                note.tags = parsed.tags;
                note.links = parsed.links;

                db.insert(note)?;
            }

            db.save()?;
        }
        FileEvent::Deleted(path) => {
            let note_id = generate_note_id(path, vault_path)?;
            db.delete(&note_id)?;
            db.save()?;
        }
    }

    Ok(())
}

fn generate_note_id(file_path: &Path, vault_path: &Path) -> Result<String> {
    let relative_path = file_path.strip_prefix(vault_path)
        .unwrap_or(file_path);

    let id = relative_path
        .with_extension("")
        .to_string_lossy()
        .to_string()
        .replace('/', "-")
        .replace('\\', "-");

    Ok(MarkdownParser::normalize_note_id(&id))
}
