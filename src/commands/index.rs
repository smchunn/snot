use std::path::Path;

use anyhow::Result;

use crate::vault::Vault;
use crate::watcher::scanner;

pub fn index_vault(vault_path: &Path, force_reindex: bool) -> Result<()> {
    let mut vault = Vault::open(vault_path)
        .or_else(|_| Vault::init(vault_path))
        .map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let markdown_files = scanner::scan_vault(&vault.path)
        .map_err(|e| anyhow::anyhow!("Failed to scan vault: {}", e))?;

    println!("Found {} markdown files", markdown_files.len());

    // Process files - collect errors but don't stop
    let mut indexed = 0;
    let mut errors = 0;

    for path in &markdown_files {
        match vault.ingest_file(path, force_reindex) {
            Ok(true) => indexed += 1,
            Ok(false) => {} // skipped, unchanged
            Err(e) => {
                eprintln!("Error processing {}: {}", path.display(), e);
                errors += 1;
            }
        }
    }

    vault
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save database: {}", e))?;

    let total = vault.db.get_all().len();
    println!(
        "Indexed {} new/changed notes ({} total, {} errors)",
        indexed, total, errors
    );

    Ok(())
}
