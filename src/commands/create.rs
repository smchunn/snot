use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use chrono::Local;
use crate::parser::MarkdownParser;

pub fn create_note(vault_path: &Path, name: &str) -> Result<()> {
    // Transform name to kebab-case and append date
    let normalized = MarkdownParser::normalize_note_id(name);
    let date = Local::now().format("%Y-%m-%d");
    let filename = format!("{}-{}.md", normalized, date);

    let file_path = vault_path.join(&filename);

    // Check if file already exists
    if file_path.exists() {
        anyhow::bail!("Note already exists: {}", file_path.display());
    }

    // Create note with YAML frontmatter
    let title = name.trim();
    let note_id = format!("{}-{}", normalized, date);
    let content = format!(
        "---\nid: {}\naliases:\n  - {}\ntags: []\n---\n\n# {}\n\n",
        note_id,
        title,
        title
    );

    fs::write(&file_path, content)
        .context("Failed to write note file")?;

    // Output as JSON for Neovim to parse
    println!(
        "{{\"path\": \"{}\", \"title\": \"{}\", \"id\": \"{}\"}}",
        file_path.display(),
        title,
        note_id
    );

    Ok(())
}
