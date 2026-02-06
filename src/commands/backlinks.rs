use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::vault::Vault;

pub fn get_backlinks(vault_path: &Path, file_path: &Path) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let note = vault
        .db
        .get_by_path(file_path)
        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", file_path.display()))?;

    let backlinks = vault.db.get_backlinks(&note.id);

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
