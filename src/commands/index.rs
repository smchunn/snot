use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use rayon::prelude::*;
use crate::db::{Database, Note};
use crate::parser::MarkdownParser;
use crate::watcher::FileWatcher;

pub fn index_vault(vault_path: &Path, force_reindex: bool) -> Result<()> {
    let db_path = vault_path.join(".snot/db.bin");
    let mut db = Database::with_path(db_path.clone())?;

    let watcher = FileWatcher::new(vault_path.to_path_buf());
    let markdown_files = watcher.scan_vault()?;

    println!("Found {} markdown files", markdown_files.len());

    // Process files in parallel
    let notes: Vec<_> = markdown_files
        .par_iter()
        .filter_map(|path| {
            process_file(&db, path, vault_path, force_reindex)
                .map_err(|e| eprintln!("Error processing {}: {}", path.display(), e))
                .ok()
        })
        .flatten()
        .collect();

    // Insert all notes into database
    for note in notes {
        db.insert(note)?;
    }

    db.save()?;

    println!("Indexed {} notes", db.get_all().len());

    Ok(())
}

fn process_file(
    db: &Database,
    path: &Path,
    vault_path: &Path,
    force_reindex: bool,
) -> Result<Option<Note>> {
    let checksum = FileWatcher::calculate_checksum(path)?;

    // Check if file has changed
    if !force_reindex {
        if let Some(existing_note) = db.get_by_path(&path.to_path_buf()) {
            if existing_note.checksum == checksum {
                // File hasn't changed, skip
                return Ok(None);
            }
        }
    }

    let content = fs::read_to_string(path)
        .context("Failed to read file")?;

    let parsed = MarkdownParser::parse(&content)?;

    // Generate note ID from file path
    let note_id = generate_note_id(path, vault_path)?;

    let mut note = Note::new(
        note_id,
        parsed.title,
        path.to_path_buf(),
        checksum,
    );

    note.aliases = parsed.aliases;
    note.tags = parsed.tags;
    note.links = parsed.links;

    Ok(Some(note))
}

fn generate_note_id(file_path: &Path, vault_path: &Path) -> Result<String> {
    let relative_path = file_path.strip_prefix(vault_path)
        .context("File is not in vault")?;

    let id = relative_path
        .with_extension("")
        .to_string_lossy()
        .to_string()
        .replace('/', "-")
        .replace('\\', "-");

    Ok(MarkdownParser::normalize_note_id(&id))
}
