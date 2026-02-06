use std::fs;
use std::path::{Path, PathBuf};

use crate::db::Database;
use crate::error::{Result, SnotError};
use crate::note::{self, Note, NoteId};
use crate::parser::markdown;
use crate::watcher::scanner;

/// Central coordinator for all vault operations.
/// Owns the database and provides high-level operations.
pub struct Vault {
    pub path: PathBuf,
    pub db: Database,
}

impl Vault {
    /// Open an existing vault, loading the database from disk.
    pub fn open(vault_path: &Path) -> Result<Self> {
        let vault_path = vault_path
            .canonicalize()
            .map_err(|_| SnotError::VaultNotFound(vault_path.to_path_buf()))?;

        let db_path = Self::db_path(&vault_path);
        if !db_path.exists() {
            return Err(SnotError::DatabaseNotFound(db_path));
        }

        let db = Database::load(&db_path)?;

        Ok(Self {
            path: vault_path,
            db,
        })
    }

    /// Initialize a new vault at the given path.
    pub fn init(vault_path: &Path) -> Result<Self> {
        let snot_dir = vault_path.join(".snot");

        // Create vault and .snot directories
        fs::create_dir_all(&snot_dir)?;

        let db = Database::new();
        let vault_path = vault_path
            .canonicalize()
            .map_err(|_| SnotError::VaultNotFound(vault_path.to_path_buf()))?;

        let mut vault = Self {
            path: vault_path,
            db,
        };
        vault.save()?;
        Ok(vault)
    }

    /// Save the database to disk.
    pub fn save(&mut self) -> Result<()> {
        let db_path = Self::db_path(&self.path);
        self.db.save(&db_path)
    }

    /// Resolve a file path to a NoteId.
    pub fn resolve_note_id(&self, file_path: &Path) -> Result<NoteId> {
        note::note_id_from_path(file_path, &self.path)
    }

    /// Read, parse, checksum, and insert/update a single file.
    /// Returns true if the note was actually updated (checksum changed).
    pub fn ingest_file(&mut self, path: &Path, force: bool) -> Result<bool> {
        let checksum = scanner::calculate_checksum(path)?;

        // Skip if checksum unchanged (unless forced)
        if !force {
            if let Some(existing) = self.db.get_by_path(path) {
                if existing.checksum == checksum {
                    return Ok(false);
                }
            }
        }

        let content = fs::read_to_string(path)?;
        let parsed = markdown::parse(&content);
        let note_id = self.resolve_note_id(path)?;

        // Preserve created_at from existing note if it exists
        let created_at = self.db.get(&note_id).map(|n| n.created_at);

        let mut new_note = Note::new(note_id.clone(), parsed.title, path.to_path_buf(), checksum);
        new_note.aliases = parsed.aliases;
        new_note.tags = parsed.tags;

        if let Some(created) = created_at {
            new_note.created_at = created;
        }

        // Extract links for the graph
        let links = parsed.links;

        if self.db.get(&note_id).is_some() {
            self.db.update(&note_id, new_note, links);
        } else {
            self.db.insert(new_note, links);
        }

        Ok(true)
    }

    /// Delete a note by its file path.
    pub fn delete_file(&mut self, path: &Path) -> Result<()> {
        let note_id = self.resolve_note_id(path)?;
        self.db.delete(&note_id);
        Ok(())
    }

    /// Path to the database file within a vault.
    pub fn db_path(vault_path: &Path) -> PathBuf {
        vault_path.join(".snot/db.bin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_open() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("test_vault");

        // Init
        let vault = Vault::init(&vault_path).unwrap();
        assert!(vault.path.exists());
        assert!(Vault::db_path(&vault.path).exists());

        // Open
        let vault2 = Vault::open(&vault_path).unwrap();
        assert_eq!(vault.path, vault2.path);
    }

    #[test]
    fn test_ingest_file() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("vault");
        let mut vault = Vault::init(&vault_path).unwrap();

        // Create a test note
        let note_path = vault.path.join("test-note.md");
        fs::write(&note_path, "---\ntags: [work]\n---\n\n# Test Note\n\nContent with #inline-tag and [[other-note]]\n").unwrap();

        let changed = vault.ingest_file(&note_path, false).unwrap();
        assert!(changed);

        let note = vault.db.get(&"test-note".to_string()).unwrap();
        assert_eq!(note.title, "Test Note");
        assert!(note.tags.contains("work"));
        assert!(note.tags.contains("inline-tag"));

        // Check that links are in the graph
        let links = vault.db.graph().forward_links(&"test-note".to_string());
        assert!(links.contains("other-note"));

        // Ingest again without force — should skip
        let changed2 = vault.ingest_file(&note_path, false).unwrap();
        assert!(!changed2);

        // Force reingest
        let changed3 = vault.ingest_file(&note_path, true).unwrap();
        assert!(changed3);
    }

    #[test]
    fn test_delete_file() {
        let tmp = tempfile::tempdir().unwrap();
        let vault_path = tmp.path().join("vault");
        let mut vault = Vault::init(&vault_path).unwrap();

        let note_path = vault.path.join("to-delete.md");
        fs::write(&note_path, "# Delete Me\n").unwrap();

        vault.ingest_file(&note_path, false).unwrap();
        assert!(vault.db.get(&"to-delete".to_string()).is_some());

        vault.delete_file(&note_path).unwrap();
        assert!(vault.db.get(&"to-delete".to_string()).is_none());
    }
}
