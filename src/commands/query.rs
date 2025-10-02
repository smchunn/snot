use std::path::Path;
use anyhow::Result;
use serde_json::json;
use crate::db::{Database, Query, QueryExecutor};

pub fn query_notes(vault_path: &Path, query_str: &str) -> Result<()> {
    let db_path = vault_path.join(".snot/db.bin");
    let db = Database::with_path(db_path)?;

    let query = Query::parse(query_str)?;
    let executor = QueryExecutor::new(&db);
    let results = executor.execute(&query);

    // Output as JSON for easy parsing by Neovim
    let json_results: Vec<_> = results
        .iter()
        .map(|note| {
            json!({
                "id": note.id,
                "title": note.title,
                "path": note.file_path,
                "tags": note.tags,
                "links": note.links,
                "created_at": note.created_at,
                "modified_at": note.modified_at,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_results)?);

    Ok(())
}
