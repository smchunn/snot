use std::path::{Path, PathBuf};
use anyhow::Result;
use serde_json::json;
use crate::db::Database;

pub fn get_backlinks(vault_path: &Path, file_path: &Path) -> Result<()> {
    let db_path = vault_path.join(".snot/db.bin");
    let db = Database::with_path(db_path)?;

    let file_path_buf = PathBuf::from(file_path);
    let note = db.get_by_path(&file_path_buf)
        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", file_path.display()))?;

    let backlinks = db.get_backlinks(&note.id);

    // Output as JSON
    let json_results: Vec<_> = backlinks
        .iter()
        .map(|note| {
            json!({
                "id": note.id,
                "title": note.title,
                "path": note.file_path,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_results)?);

    Ok(())
}
