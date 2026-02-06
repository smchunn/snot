use std::path::Path;

use anyhow::Result;

use crate::query::{self, QueryExecutor};
use crate::vault::Vault;

pub fn list_notes(vault_path: &Path, query_str: Option<&str>) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let notes = if let Some(q) = query_str {
        let parsed = query::parse(q).map_err(|e| anyhow::anyhow!("{}", e))?;
        let executor = QueryExecutor::new(&vault.db);
        executor.execute(&parsed)
    } else {
        vault.db.get_all()
    };

    // Output one path per line for FZF
    for note in notes {
        println!("{}", note.file_path.display());
    }

    Ok(())
}
