use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use super::ast::Query;
use super::fuzzy;
use crate::db::Database;
use crate::note::{Note, NoteId};

/// Execute queries against the database.
pub struct QueryExecutor<'a> {
    db: &'a Database,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn execute(&self, query: &Query) -> Vec<&'a Note> {
        let ids = self.execute_ids(query);
        self.db.get_notes_by_ids(&ids)
    }

    /// Execute a query, returning matching note IDs.
    fn execute_ids(&self, query: &Query) -> HashSet<NoteId> {
        match query {
            Query::All => self.db.all_note_ids(),

            Query::Tag(tag) => self
                .db
                .get_by_tag(tag)
                .into_iter()
                .map(|n| n.id.clone())
                .collect(),

            Query::Title(text) => {
                let lower = text.to_lowercase();
                self.db
                    .get_all()
                    .into_iter()
                    .filter(|n| n.title.to_lowercase().contains(&lower))
                    .map(|n| n.id.clone())
                    .collect()
            }

            Query::Alias(text) => {
                let lower = text.to_lowercase();
                self.db
                    .get_all()
                    .into_iter()
                    .filter(|n| n.aliases.iter().any(|a| a.to_lowercase().contains(&lower)))
                    .map(|n| n.id.clone())
                    .collect()
            }

            Query::Fuzzy(text) => self
                .db
                .get_all()
                .into_iter()
                .filter(|n| {
                    let title_sim = fuzzy::trigram_similarity(&n.title, text);
                    let alias_sim = n
                        .aliases
                        .iter()
                        .map(|a| fuzzy::trigram_similarity(a, text))
                        .fold(0.0f64, f64::max);
                    let id_sim = fuzzy::trigram_similarity(&n.id, text);
                    title_sim.max(alias_sim).max(id_sim) >= fuzzy::FUZZY_THRESHOLD
                })
                .map(|n| n.id.clone())
                .collect(),

            Query::Content(text) => self.search_content(text),

            Query::LinksTo(note_id) => self.db.graph().backlinks(note_id),

            Query::LinksFrom(note_id) => self.db.graph().forward_links(note_id),

            Query::Neighborhood(note_id, depth) => self.db.graph().neighbors(note_id, *depth),

            Query::Orphans => {
                let linked = self.db.graph().all_linked_notes();
                let all = self.db.all_note_ids();
                all.difference(&linked).cloned().collect()
            }

            Query::DateRange(start, end) => self
                .db
                .get_in_date_range(*start, *end)
                .into_iter()
                .map(|n| n.id.clone())
                .collect(),

            Query::And(left, right) => {
                let left_ids = self.execute_ids(left);
                let right_ids = self.execute_ids(right);
                left_ids.intersection(&right_ids).cloned().collect()
            }

            Query::Or(left, right) => {
                let left_ids = self.execute_ids(left);
                let right_ids = self.execute_ids(right);
                left_ids.union(&right_ids).cloned().collect()
            }

            Query::Not(inner) => {
                let inner_ids = self.execute_ids(inner);
                let all = self.db.all_note_ids();
                all.difference(&inner_ids).cloned().collect()
            }
        }
    }

    /// Search file content using ripgrep, grep, or title/alias fallback.
    fn search_content(&self, text: &str) -> HashSet<NoteId> {
        let file_paths = self.db.get_all_file_paths();
        if file_paths.is_empty() {
            return HashSet::new();
        }

        let matching_paths = self
            .search_with_ripgrep(text, &file_paths)
            .or_else(|_| self.search_with_grep(text, &file_paths))
            .unwrap_or_default();

        // Convert paths back to note IDs
        let mut result = HashSet::new();
        for path in &matching_paths {
            if let Some(note) = self.db.get_by_path(path) {
                result.insert(note.id.clone());
            }
        }

        // Fallback: also include title/alias matches
        if matching_paths.is_empty() {
            let lower = text.to_lowercase();
            for note in self.db.get_all() {
                if note.title.to_lowercase().contains(&lower)
                    || note
                        .aliases
                        .iter()
                        .any(|a| a.to_lowercase().contains(&lower))
                {
                    result.insert(note.id.clone());
                }
            }
        }

        result
    }

    fn search_with_ripgrep(&self, text: &str, files: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
        let output = Command::new("rg")
            .arg("-i")
            .arg("-l")
            .arg("--")
            .arg(text)
            .args(files)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("ripgrep failed");
        }

        Ok(String::from_utf8(output.stdout)?
            .lines()
            .map(PathBuf::from)
            .collect())
    }

    fn search_with_grep(&self, text: &str, files: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
        let output = Command::new("grep")
            .arg("-i")
            .arg("-l")
            .arg("--")
            .arg(text)
            .args(files)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("grep failed");
        }

        Ok(String::from_utf8(output.stdout)?
            .lines()
            .map(PathBuf::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::note::Note;
    use std::path::PathBuf;

    fn setup_db() -> Database {
        let mut db = Database::new();

        let mut note1 = Note::new(
            "work-meeting".into(),
            "Work Meeting".into(),
            PathBuf::from("work-meeting.md"),
            "c1".into(),
        );
        note1.tags = ["work", "meeting"].iter().map(|s| s.to_string()).collect();
        note1.aliases = vec!["Daily Standup".to_string()];
        let links1: HashSet<NoteId> = ["project-plan".to_string()].into();
        db.insert(note1, links1);

        let mut note2 = Note::new(
            "project-plan".into(),
            "Project Plan".into(),
            PathBuf::from("project-plan.md"),
            "c2".into(),
        );
        note2.tags = ["work", "planning"].iter().map(|s| s.to_string()).collect();
        let links2: HashSet<NoteId> = ["research-notes".to_string()].into();
        db.insert(note2, links2);

        let mut note3 = Note::new(
            "personal-journal".into(),
            "Personal Journal".into(),
            PathBuf::from("personal-journal.md"),
            "c3".into(),
        );
        note3.tags = ["personal"].iter().map(|s| s.to_string()).collect();
        db.insert(note3, HashSet::new());

        let note4 = Note::new(
            "orphan-note".into(),
            "Orphan Note".into(),
            PathBuf::from("orphan-note.md"),
            "c4".into(),
        );
        db.insert(note4, HashSet::new());

        db
    }

    #[test]
    fn test_execute_tag() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let results = executor.execute(&Query::Tag("work".into()));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_execute_title() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let results = executor.execute(&Query::Title("meeting".into()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "work-meeting");
    }

    #[test]
    fn test_execute_alias() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let results = executor.execute(&Query::Alias("standup".into()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "work-meeting");
    }

    #[test]
    fn test_execute_fuzzy() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        // "meating" is a typo of "meeting"
        let results = executor.execute(&Query::Fuzzy("meating".into()));
        let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();
        assert!(
            ids.contains("work-meeting"),
            "Expected work-meeting in fuzzy results"
        );
    }

    #[test]
    fn test_execute_links_to() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        // work-meeting links to project-plan, so backlinks of project-plan = work-meeting
        let results = executor.execute(&Query::LinksTo("project-plan".into()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "work-meeting");
    }

    #[test]
    fn test_execute_links_from() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let results = executor.execute(&Query::LinksFrom("work-meeting".into()));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "project-plan");
    }

    #[test]
    fn test_execute_orphans() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let results = executor.execute(&Query::Orphans);
        let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();
        // personal-journal and orphan-note have no links
        assert!(ids.contains("personal-journal"));
        assert!(ids.contains("orphan-note"));
    }

    #[test]
    fn test_execute_neighborhood() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        // work-meeting -> project-plan -> research-notes
        let results = executor.execute(&Query::Neighborhood("work-meeting".into(), 2));
        let ids: HashSet<_> = results.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains("project-plan"));
    }

    #[test]
    fn test_execute_and() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let q = Query::And(
            Box::new(Query::Tag("work".into())),
            Box::new(Query::Title("meeting".into())),
        );
        let results = executor.execute(&q);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "work-meeting");
    }

    #[test]
    fn test_execute_or() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let q = Query::Or(
            Box::new(Query::Tag("work".into())),
            Box::new(Query::Tag("personal".into())),
        );
        let results = executor.execute(&q);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_execute_not() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let q = Query::Not(Box::new(Query::Tag("work".into())));
        let results = executor.execute(&q);
        assert_eq!(results.len(), 2); // personal-journal and orphan-note
    }

    #[test]
    fn test_execute_all() {
        let db = setup_db();
        let executor = QueryExecutor::new(&db);
        let results = executor.execute(&Query::All);
        assert_eq!(results.len(), 4);
    }
}
