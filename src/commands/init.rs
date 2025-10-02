use std::path::{Path, PathBuf};
use std::fs::create_dir_all;
use anyhow::{Result, Context};
use crate::db::Database;

pub fn init_vault(vault_path: &Path) -> Result<()> {
    // Create vault directory if it doesn't exist
    create_dir_all(vault_path)
        .context("Failed to create vault directory")?;

    // Create .snot directory for database storage
    let snot_dir = vault_path.join(".snot");
    create_dir_all(&snot_dir)
        .context("Failed to create .snot directory")?;

    // Initialize empty database
    let db_path = snot_dir.join("db.bin");
    let db = Database::new();

    // Save empty database
    let encoded = bincode::serialize(&db)
        .context("Failed to serialize database")?;
    std::fs::write(&db_path, encoded)
        .context("Failed to write database file")?;

    println!("Initialized vault at {}", vault_path.display());
    println!("Database created at {}", db_path.display());

    Ok(())
}
