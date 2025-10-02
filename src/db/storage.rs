use std::collections::{HashMap, HashSet, BTreeMap};
use std::path::PathBuf;
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

pub type NoteId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub title: String,
    pub aliases: Vec<String>,
    pub file_path: PathBuf,
    pub tags: HashSet<String>,
    pub links: HashSet<String>,
    pub backlinks: HashSet<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub checksum: String,
}

impl Note {
    pub fn new(
        id: NoteId,
        title: String,
        file_path: PathBuf,
        checksum: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            aliases: Vec::new(),
            file_path,
            tags: HashSet::new(),
            links: HashSet::new(),
            backlinks: HashSet::new(),
            created_at: now,
            modified_at: now,
            checksum,
        }
    }

    pub fn update_metadata(&mut self, checksum: String) {
        self.checksum = checksum;
        self.modified_at = Utc::now();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    notes: HashMap<NoteId, Note>,
    // Index for fast lookups
    tag_index: HashMap<String, HashSet<NoteId>>,
    // Index for date-based queries
    date_index: BTreeMap<DateTime<Utc>, HashSet<NoteId>>,
    // Path to note ID mapping
    path_index: HashMap<PathBuf, NoteId>,
    // Database file path
    #[serde(skip)]
    db_path: Option<PathBuf>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            tag_index: HashMap::new(),
            date_index: BTreeMap::new(),
            path_index: HashMap::new(),
            db_path: None,
        }
    }

    pub fn with_path(path: PathBuf) -> Result<Self> {
        let mut db = if path.exists() {
            Self::load(&path)?
        } else {
            Self::new()
        };
        db.db_path = Some(path);
        Ok(db)
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let mut file = File::open(path)
            .context("Failed to open database file")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .context("Failed to read database file")?;

        let db: Database = bincode::deserialize(&buffer)
            .context("Failed to deserialize database")?;
        Ok(db)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.db_path {
            if let Some(parent) = path.parent() {
                create_dir_all(parent)?;
            }

            let encoded = bincode::serialize(&self)
                .context("Failed to serialize database")?;

            let mut file = File::create(path)
                .context("Failed to create database file")?;
            file.write_all(&encoded)
                .context("Failed to write database file")?;
            file.sync_all()?;
        }
        Ok(())
    }

    // CRUD Operations

    pub fn insert(&mut self, mut note: Note) -> Result<()> {
        // Update indexes
        for tag in &note.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(note.id.clone());
        }

        self.date_index
            .entry(note.modified_at)
            .or_insert_with(HashSet::new)
            .insert(note.id.clone());

        self.path_index.insert(note.file_path.clone(), note.id.clone());

        // Update backlinks for linked notes
        for link in &note.links {
            if let Some(linked_note) = self.notes.get_mut(link) {
                linked_note.backlinks.insert(note.id.clone());
            }
        }

        // Add backlinks from notes that link to this note
        let backlinks: Vec<String> = self.notes
            .iter()
            .filter(|(_, n)| n.links.contains(&note.id))
            .map(|(id, _)| id.clone())
            .collect();

        for backlink_id in backlinks {
            note.backlinks.insert(backlink_id);
        }

        self.notes.insert(note.id.clone(), note);
        Ok(())
    }

    pub fn get(&self, id: &NoteId) -> Option<&Note> {
        self.notes.get(id)
    }

    pub fn get_by_path(&self, path: &PathBuf) -> Option<&Note> {
        self.path_index.get(path)
            .and_then(|id| self.notes.get(id))
    }

    pub fn update(&mut self, id: &NoteId, note: Note) -> Result<()> {
        // Remove old indexes - clone data we need before mutable borrows
        let (old_tags, old_modified_at, old_path, old_links) = if let Some(old_note) = self.notes.get(id) {
            (
                old_note.tags.clone(),
                old_note.modified_at,
                old_note.file_path.clone(),
                old_note.links.clone(),
            )
        } else {
            // If note doesn't exist, just insert it
            return self.insert(note);
        };

        // Now we can safely do mutable borrows
        for tag in &old_tags {
            if let Some(note_set) = self.tag_index.get_mut(tag) {
                note_set.remove(id);
            }
        }

        if let Some(note_set) = self.date_index.get_mut(&old_modified_at) {
            note_set.remove(id);
        }

        self.path_index.remove(&old_path);

        // Remove old backlinks
        for link in &old_links {
            if let Some(linked_note) = self.notes.get_mut(link) {
                linked_note.backlinks.remove(id);
            }
        }

        // Insert with new indexes
        self.insert(note)?;
        Ok(())
    }

    pub fn delete(&mut self, id: &NoteId) -> Result<()> {
        if let Some(note) = self.notes.remove(id) {
            // Remove from tag index
            for tag in &note.tags {
                if let Some(note_set) = self.tag_index.get_mut(tag) {
                    note_set.remove(id);
                }
            }

            // Remove from date index
            if let Some(note_set) = self.date_index.get_mut(&note.modified_at) {
                note_set.remove(id);
            }

            // Remove from path index
            self.path_index.remove(&note.file_path);

            // Remove backlinks from linked notes
            for link in &note.links {
                if let Some(linked_note) = self.notes.get_mut(link) {
                    linked_note.backlinks.remove(id);
                }
            }

            // Remove this note's ID from notes that had it as a backlink
            for backlink_id in &note.backlinks {
                if let Some(backlinking_note) = self.notes.get_mut(backlink_id) {
                    backlinking_note.links.remove(id);
                }
            }
        }
        Ok(())
    }

    // Query helpers

    pub fn get_by_tag(&self, tag: &str) -> Vec<&Note> {
        self.tag_index
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.notes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn get_all(&self) -> Vec<&Note> {
        self.notes.values().collect()
    }

    pub fn get_all_file_paths(&self) -> Vec<PathBuf> {
        self.notes.values().map(|note| note.file_path.clone()).collect()
    }

    pub fn get_notes_by_paths(&self, paths: &[PathBuf]) -> Vec<&Note> {
        paths.iter()
            .filter_map(|path| self.get_by_path(path))
            .collect()
    }

    pub fn get_backlinks(&self, id: &NoteId) -> Vec<&Note> {
        self.notes
            .get(id)
            .map(|note| {
                note.backlinks
                    .iter()
                    .filter_map(|backlink_id| self.notes.get(backlink_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_in_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&Note> {
        self.date_index
            .range(start..=end)
            .flat_map(|(_, ids)| ids.iter().filter_map(|id| self.notes.get(id)))
            .collect()
    }

    pub fn all_tags(&self) -> Vec<String> {
        self.tag_index.keys().cloned().collect()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
