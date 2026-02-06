use std::path::Path;

use anyhow::Result;

use crate::vault::Vault;

pub fn list_tags(vault_path: &Path) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let mut tags = vault.db.all_tags();
    tags.sort();

    println!("{}", serde_json::to_string_pretty(&tags)?);

    Ok(())
}
