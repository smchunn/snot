use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::note::NoteId;

/// Tag -> set of note IDs that have that tag.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TagIndex {
    index: HashMap<String, HashSet<NoteId>>,
}

impl TagIndex {
    pub fn insert(&mut self, tag: &str, note_id: &NoteId) {
        self.index
            .entry(tag.to_string())
            .or_default()
            .insert(note_id.clone());
    }

    pub fn remove(&mut self, tag: &str, note_id: &NoteId) {
        if let Some(set) = self.index.get_mut(tag) {
            set.remove(note_id);
            if set.is_empty() {
                self.index.remove(tag);
            }
        }
    }

    pub fn get(&self, tag: &str) -> Option<&HashSet<NoteId>> {
        self.index.get(tag)
    }

    pub fn all_tags(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }
}

/// Modified date -> set of note IDs modified at that time.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DateIndex {
    index: BTreeMap<DateTime<Utc>, HashSet<NoteId>>,
}

impl DateIndex {
    pub fn insert(&mut self, date: DateTime<Utc>, note_id: &NoteId) {
        self.index.entry(date).or_default().insert(note_id.clone());
    }

    pub fn remove(&mut self, date: &DateTime<Utc>, note_id: &NoteId) {
        if let Some(set) = self.index.get_mut(date) {
            set.remove(note_id);
            if set.is_empty() {
                self.index.remove(date);
            }
        }
    }

    pub fn range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<NoteId> {
        self.index
            .range(start..=end)
            .flat_map(|(_, ids)| ids.iter().cloned())
            .collect()
    }
}

/// File path -> note ID mapping.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PathIndex {
    index: HashMap<PathBuf, NoteId>,
}

impl PathIndex {
    pub fn insert(&mut self, path: PathBuf, note_id: NoteId) {
        self.index.insert(path, note_id);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.index.remove(path);
    }

    pub fn get(&self, path: &PathBuf) -> Option<&NoteId> {
        self.index.get(path)
    }
}

/// Alias -> set of note IDs that have that alias (normalized for lookup).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AliasIndex {
    index: HashMap<String, HashSet<NoteId>>,
}

#[allow(dead_code)]
impl AliasIndex {
    pub fn insert(&mut self, alias: &str, note_id: &NoteId) {
        self.index
            .entry(alias.to_lowercase())
            .or_default()
            .insert(note_id.clone());
    }

    pub fn remove(&mut self, alias: &str, note_id: &NoteId) {
        let key = alias.to_lowercase();
        if let Some(set) = self.index.get_mut(&key) {
            set.remove(note_id);
            if set.is_empty() {
                self.index.remove(&key);
            }
        }
    }

    pub fn get(&self, alias: &str) -> Option<&HashSet<NoteId>> {
        self.index.get(&alias.to_lowercase())
    }
}
