use std::path::Path;

use anyhow::Result;

use crate::vault::Vault;

pub fn init_vault(vault_path: &Path) -> Result<()> {
    Vault::init(vault_path)?;

    println!("Initialized vault at {}", vault_path.display());
    println!(
        "Database created at {}",
        Vault::db_path(vault_path).display()
    );

    Ok(())
}
