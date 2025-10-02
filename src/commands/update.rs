use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use crate::db::{Database, Note};
use crate::parser::MarkdownParser;
use crate::watcher::FileWatcher;

pub fn update_note(vault_path: &Path, file_path: &Path) -> Result<()> {
    let db_path = vault_path.join(".snot/db.bin");
    let mut db = Database::with_path(db_path)?;

    // Read and parse the file
    let content = fs::read_to_string(file_path)
        .context("Failed to read file")?;

    let parsed = MarkdownParser::parse(&content)?;
    let checksum = FileWatcher::calculate_checksum(file_path)?;

    // Generate note ID from file path
    let note_id = generate_note_id(file_path, vault_path)?;

    // Create or update the note
    let mut note = Note::new(
        note_id.clone(),
        parsed.title,
        file_path.to_path_buf(),
        checksum,
    );

    note.aliases = parsed.aliases;
    note.tags = parsed.tags;
    note.links = parsed.links;

    // Check if note exists, update or insert
    if db.get(&note_id).is_some() {
        db.update(&note_id, note)?;
    } else {
        db.insert(note)?;
    }

    db.save()?;

    Ok(())
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
