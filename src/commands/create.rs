use std::path::Path;

use anyhow::Result;
use chrono::Local;
use serde_json::json;

use crate::note;

pub fn create_note(vault_path: &Path, name: &str) -> Result<()> {
    let normalized = note::normalize_note_id(name);
    let date = Local::now().format("%Y-%m-%d");
    let filename = format!("{}-{}.md", normalized, date);
    let file_path = vault_path.join(&filename);

    if file_path.exists() {
        anyhow::bail!("Note already exists: {}", file_path.display());
    }

    let title = name.trim();
    let note_id = format!("{}-{}", normalized, date);

    let content = format!(
        "---\nid: {}\naliases:\n  - {}\ntags: []\n---\n\n# {}\n\n",
        note_id, title, title
    );

    std::fs::write(&file_path, content)?;

    let output = json!({
        "path": file_path,
        "title": title,
        "id": note_id,
    });

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
