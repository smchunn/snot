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

    // Create note with basic template
    let title = name.trim();
    let content = format!(
        "# {}\n\nCreated: {}\n\n",
        title,
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    fs::write(&file_path, content)
        .context("Failed to write note file")?;

    // Output as JSON for Neovim to parse
    println!(
        "{{\"path\": \"{}\", \"title\": \"{}\"}}",
        file_path.display(),
        title
    );

    Ok(())
}
