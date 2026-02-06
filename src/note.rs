use std::collections::HashSet;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a note, derived from its file path within the vault.
/// Format: kebab-case, e.g. "work-meeting-notes"
pub type NoteId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub aliases: Vec<String>,
    pub file_path: PathBuf,
    pub tags: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub checksum: String,
}

impl Note {
    pub fn new(id: NoteId, title: String, file_path: PathBuf, checksum: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            aliases: Vec::new(),
            file_path,
            tags: HashSet::new(),
            created_at: now,
            modified_at: now,
            checksum,
        }
    }
}

/// Normalize an arbitrary string into a valid note ID (kebab-case).
pub fn normalize_note_id(name: &str) -> NoteId {
    name.trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Generate a note ID from a file path relative to the vault root.
pub fn note_id_from_path(
    file_path: &std::path::Path,
    vault_path: &std::path::Path,
) -> crate::error::Result<NoteId> {
    let relative = file_path.strip_prefix(vault_path).map_err(|_| {
        crate::error::SnotError::FileNotInVault {
            path: file_path.to_path_buf(),
        }
    })?;

    let raw = relative
        .with_extension("")
        .to_string_lossy()
        .replace(['/', '\\'], "-");

    Ok(normalize_note_id(&raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_normalize_note_id() {
        assert_eq!(normalize_note_id("My New Note"), "my-new-note");
        assert_eq!(normalize_note_id("  Spaces  "), "spaces");
        assert_eq!(normalize_note_id("UPPER_case"), "upper_case");
        assert_eq!(normalize_note_id("special!@#chars"), "specialchars");
    }

    #[test]
    fn test_note_id_from_path() {
        let vault = Path::new("/vault");
        let file = Path::new("/vault/work/meeting-notes.md");
        assert_eq!(
            note_id_from_path(file, vault).unwrap(),
            "work-meeting-notes"
        );
    }

    #[test]
    fn test_note_id_from_path_not_in_vault() {
        let vault = Path::new("/vault");
        let file = Path::new("/other/note.md");
        assert!(note_id_from_path(file, vault).is_err());
    }
}
