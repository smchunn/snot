use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::query::{self, QueryExecutor};
use crate::vault::Vault;

pub fn query_notes(vault_path: &Path, query_str: &str) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let parsed = query::parse(query_str).map_err(|e| anyhow::anyhow!("{}", e))?;

    let executor = QueryExecutor::new(&vault.db);
    let results = executor.execute(&parsed);

    let json_results: Vec<_> = results
        .iter()
        .map(|note| {
            json!({
                "id": note.id,
                "title": note.title,
                "path": note.file_path,
                "tags": note.tags,
                "aliases": note.aliases,
                "created_at": note.created_at,
                "modified_at": note.modified_at,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_results)?);

    Ok(())
}
