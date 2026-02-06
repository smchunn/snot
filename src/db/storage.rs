use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, SnotError};
use crate::note::{Note, NoteId};

use super::graph::LinkGraph;
use super::index::{AliasIndex, DateIndex, PathIndex, TagIndex};
use super::schema;

/// In-memory database storing note metadata with indexes and a link graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    notes: std::collections::HashMap<NoteId, Note>,
    tag_index: TagIndex,
    date_index: DateIndex,
    path_index: PathIndex,
    alias_index: AliasIndex,
    link_graph: LinkGraph,
}

impl Database {
    pub fn new() -> Self {
        Self {
            notes: std::collections::HashMap::new(),
            tag_index: TagIndex::default(),
            date_index: DateIndex::default(),
            path_index: PathIndex::default(),
            alias_index: AliasIndex::default(),
            link_graph: LinkGraph::new(),
        }
    }

    /// Load a database from a binary file with schema validation.
    pub fn load(path: &Path) -> Result<Self> {
        let data = fs::read(path)?;
        let payload = schema::read_header(&data)?;
        let db: Database =
            bincode::deserialize(payload).map_err(|e| SnotError::Serialization(e.to_string()))?;
        Ok(db)
    }

    /// Save the database to a binary file with schema header.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut buf = Vec::new();
        schema::write_header(&mut buf);

        let payload =
            bincode::serialize(self).map_err(|e| SnotError::Serialization(e.to_string()))?;
        buf.extend_from_slice(&payload);

        fs::write(path, &buf)?;
        Ok(())
    }

    // --- CRUD ---

    /// Insert a new note with its links.
    pub fn insert(&mut self, note: Note, links: HashSet<NoteId>) {
        let id = note.id.clone();

        // Update indexes
        for tag in &note.tags {
            self.tag_index.insert(tag, &id);
        }
        self.date_index.insert(note.modified_at, &id);
        self.path_index.insert(note.file_path.clone(), id.clone());
        for alias in &note.aliases {
            self.alias_index.insert(alias, &id);
        }

        // Update link graph
        self.link_graph.set_links(&id, links);

        self.notes.insert(id, note);
    }

    /// Update an existing note, re-indexing everything.
    pub fn update(&mut self, id: &NoteId, note: Note, links: HashSet<NoteId>) {
        // Remove old indexes
        if let Some(old) = self.notes.get(id) {
            for tag in &old.tags {
                self.tag_index.remove(tag, id);
            }
            self.date_index.remove(&old.modified_at, id);
            self.path_index.remove(&old.file_path);
            for alias in &old.aliases {
                self.alias_index.remove(alias, id);
            }
        }

        // Insert with new indexes (handles graph too)
        self.insert(note, links);
    }

    /// Delete a note by ID.
    pub fn delete(&mut self, id: &NoteId) {
        if let Some(note) = self.notes.remove(id) {
            for tag in &note.tags {
                self.tag_index.remove(tag, id);
            }
            self.date_index.remove(&note.modified_at, id);
            self.path_index.remove(&note.file_path);
            for alias in &note.aliases {
                self.alias_index.remove(alias, id);
            }
            self.link_graph.remove_note(id);
        }
    }

    // --- Lookups ---

    pub fn get(&self, id: &NoteId) -> Option<&Note> {
        self.notes.get(id)
    }

    pub fn get_by_path(&self, path: &Path) -> Option<&Note> {
        self.path_index
            .get(&path.to_path_buf())
            .and_then(|id| self.notes.get(id))
    }

    pub fn get_all(&self) -> Vec<&Note> {
        self.notes.values().collect()
    }

    pub fn get_all_file_paths(&self) -> Vec<std::path::PathBuf> {
        self.notes.values().map(|n| n.file_path.clone()).collect()
    }

    pub fn get_notes_by_ids(&self, ids: &HashSet<NoteId>) -> Vec<&Note> {
        ids.iter().filter_map(|id| self.notes.get(id)).collect()
    }

    // --- Index queries ---

    pub fn get_by_tag(&self, tag: &str) -> Vec<&Note> {
        self.tag_index
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.notes.get(id)).collect())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn get_by_alias(&self, alias: &str) -> Vec<&Note> {
        self.alias_index
            .get(alias)
            .map(|ids| ids.iter().filter_map(|id| self.notes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn get_in_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&Note> {
        let ids = self.date_index.range(start, end);
        ids.iter().filter_map(|id| self.notes.get(id)).collect()
    }

    pub fn all_tags(&self) -> Vec<String> {
        self.tag_index.all_tags()
    }

    pub fn all_note_ids(&self) -> HashSet<NoteId> {
        self.notes.keys().cloned().collect()
    }

    // --- Graph access ---

    pub fn graph(&self) -> &LinkGraph {
        &self.link_graph
    }

    /// Get backlinks for a note (notes that link to it).
    pub fn get_backlinks(&self, id: &NoteId) -> Vec<&Note> {
        let backers = self.link_graph.backlinks(id);
        backers
            .iter()
            .filter_map(|bid| self.notes.get(bid))
            .collect()
    }

    /// Get forward links for a note (notes it links to).
    #[allow(dead_code)]
    pub fn get_forward_links(&self, id: &NoteId) -> Vec<&Note> {
        let targets = self.link_graph.forward_links(id);
        targets
            .iter()
            .filter_map(|tid| self.notes.get(tid))
            .collect()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_note(id: &str, tags: &[&str]) -> (Note, HashSet<NoteId>) {
        let mut note = Note::new(
            id.to_string(),
            format!("Title of {}", id),
            PathBuf::from(format!("{}.md", id)),
            "checksum".to_string(),
        );
        note.tags = tags.iter().map(|t| t.to_string()).collect();
        (note, HashSet::new())
    }

    #[test]
    fn test_insert_and_get() {
        let mut db = Database::new();
        let (note, links) = make_note("test", &["work"]);
        db.insert(note, links);

        assert!(db.get(&"test".into()).is_some());
        assert_eq!(db.get(&"test".into()).unwrap().title, "Title of test");
    }

    #[test]
    fn test_tag_index() {
        let mut db = Database::new();
        let (note, links) = make_note("n1", &["work", "meeting"]);
        db.insert(note, links);

        let (note2, links2) = make_note("n2", &["work"]);
        db.insert(note2, links2);

        let work_notes = db.get_by_tag("work");
        assert_eq!(work_notes.len(), 2);

        let meeting_notes = db.get_by_tag("meeting");
        assert_eq!(meeting_notes.len(), 1);
    }

    #[test]
    fn test_update() {
        let mut db = Database::new();
        let (note, links) = make_note("n1", &["old-tag"]);
        db.insert(note, links);

        let (updated, links2) = make_note("n1", &["new-tag"]);
        db.update(&"n1".into(), updated, links2);

        assert!(db.get_by_tag("old-tag").is_empty());
        assert_eq!(db.get_by_tag("new-tag").len(), 1);
    }

    #[test]
    fn test_delete() {
        let mut db = Database::new();
        let (note, links) = make_note("n1", &["work"]);
        db.insert(note, links);

        db.delete(&"n1".into());
        assert!(db.get(&"n1".into()).is_none());
        assert!(db.get_by_tag("work").is_empty());
    }

    #[test]
    fn test_graph_links() {
        let mut db = Database::new();
        let (note_a, _) = make_note("a", &[]);
        let links_a: HashSet<NoteId> = ["b".to_string()].into();
        db.insert(note_a, links_a);

        let (note_b, _) = make_note("b", &[]);
        let links_b: HashSet<NoteId> = ["c".to_string()].into();
        db.insert(note_b, links_b);

        let backlinks = db.get_backlinks(&"b".into());
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].id, "a");

        let forward = db.get_forward_links(&"a".into());
        assert_eq!(forward.len(), 1);
        assert_eq!(forward[0].id, "b");
    }

    #[test]
    fn test_save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("db.bin");

        let mut db = Database::new();
        let (note, links) = make_note("test", &["tag1"]);
        db.insert(note, links);
        db.save(&db_path).unwrap();

        let loaded = Database::load(&db_path).unwrap();
        assert!(loaded.get(&"test".into()).is_some());
        assert_eq!(loaded.get_by_tag("tag1").len(), 1);
    }
}
