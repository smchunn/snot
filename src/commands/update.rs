use std::path::Path;

use anyhow::Result;

use crate::vault::Vault;

pub fn update_note(vault_path: &Path, file_path: &Path) -> Result<()> {
    let mut vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    vault
        .ingest_file(file_path, true)
        .map_err(|e| anyhow::anyhow!("Failed to update note: {}", e))?;

    vault
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save database: {}", e))?;

    Ok(())
}
