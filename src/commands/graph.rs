use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::vault::Vault;

pub fn graph_neighbors(vault_path: &Path, note_id: &str, depth: usize) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let neighbors = vault.db.graph().neighbors(&note_id.to_string(), depth);
    let notes = vault.db.get_notes_by_ids(&neighbors);

    let json_results: Vec<_> = notes
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

pub fn graph_orphans(vault_path: &Path) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let linked = vault.db.graph().all_linked_notes();
    let all_ids = vault.db.all_note_ids();
    let orphan_ids: std::collections::HashSet<_> = all_ids.difference(&linked).cloned().collect();
    let orphans = vault.db.get_notes_by_ids(&orphan_ids);

    let json_results: Vec<_> = orphans
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

pub fn graph_path(vault_path: &Path, from: &str, to: &str) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    match vault
        .db
        .graph()
        .shortest_path(&from.to_string(), &to.to_string())
    {
        Some(path) => {
            let notes: Vec<_> = path
                .iter()
                .filter_map(|id| vault.db.get(id))
                .map(|note| {
                    json!({
                        "id": note.id,
                        "title": note.title,
                        "path": note.file_path,
                    })
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&notes)?);
        }
        None => {
            println!("[]");
            eprintln!("No path found between '{}' and '{}'", from, to);
        }
    }

    Ok(())
}

pub fn graph_stats(vault_path: &Path) -> Result<()> {
    let vault =
        Vault::open(vault_path).map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

    let total_notes = vault.db.get_all().len();
    let linked = vault.db.graph().all_linked_notes();
    let orphan_count = vault.db.all_note_ids().difference(&linked).count();
    let most_linked = vault.db.graph().most_linked(10);

    let output = json!({
        "total_notes": total_notes,
        "orphan_count": orphan_count,
        "linked_count": linked.len(),
        "most_linked": most_linked.iter().map(|(id, count)| {
            json!({
                "id": id,
                "link_count": count,
                "title": vault.db.get(id).map(|n| n.title.as_str()).unwrap_or("(unknown)"),
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
